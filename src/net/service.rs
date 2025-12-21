// COPYRIGHT (C) 2025 M-Patek. ALL RIGHTS RESERVED.

use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::io::AsyncReadExt;
use quinn::{Endpoint, RecvStream, SendStream};
use bincode::{self, Options}; // [FIX]: Import Options trait
use blake3;

use crate::topology::tensor::HyperTensor;
use crate::net::wire::{HtpRequest, HtpResponse};

pub async fn run_prover_service(endpoint: Endpoint, tensor: Arc<RwLock<HyperTensor>>) {
    while let Some(conn) = endpoint.accept().await {
        let tensor_ref = tensor.clone();
        tokio::spawn(async move {
            let connection = match conn.await {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("[Net] Handshake failed: {}", e);
                    return;
                }
            };

            while let Ok((send, recv)) = connection.accept_bi().await {
                let t = tensor_ref.clone();
                tokio::spawn(async move {
                    if let Err(e) = handle_stream(t, send, recv).await {
                        eprintln!("[Net] Stream error: {}", e);
                    }
                });
            }
        });
    }
}

async fn handle_stream(
    tensor: Arc<RwLock<HyperTensor>>, 
    mut send: SendStream, 
    mut recv: RecvStream
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    
    // [SECURITY FIX]: Transport Layer Size Limit (1MB)
    let mut stream_limit = recv.take(1024 * 1024); 
    let mut buf = Vec::new();
    
    if let Err(e) = stream_limit.read_to_end(&mut buf).await {
         return Err(Box::new(e));
    }
    
    if buf.is_empty() { return Ok(()); }

    // [SECURITY FIX]: Deserialization Bomb Protection
    // Limit the size of decoded objects to prevent memory exhaustion
    let safe_config = bincode::DefaultOptions::new()
        .with_limit(5 * 1024 * 1024) 
        .with_fixint_encoding()
        .allow_trailing_bytes();

    let request: HtpRequest = safe_config.deserialize(&buf)?;

    // [FIX]: Robust Error Handling via Helper
    let response = match process_request(&tensor, request).await {
        Ok(resp) => resp,
        Err(e) => HtpResponse::Error(format!("Internal Error: {}", e)),
    };

    let resp_bytes = bincode::serialize(&response)?;
    send.write_all(&resp_bytes).await?;
    send.finish().await?;

    Ok(())
}

// [FIX]: Separation of Concerns & Error Propagation
async fn process_request(tensor: &Arc<RwLock<HyperTensor>>, request: HtpRequest) -> Result<HtpResponse, String> {
    match request {
        HtpRequest::GetProof { user_id, request_id } => {
            // [CRITICAL FIX]: Use Read Lock for Concurrency (Anti-DoS)
            let guard = tensor.read().await;
            let coord = guard.map_id_to_coord_hash(&user_id);
            
            // [CRITICAL FIX]: Fallback to Internal Calc if Cache Miss (Consistency)
            let global_root = match &guard.cached_root {
                Some(r) => r.clone(),
                None => guard.compute_root_internal()?
            };
            
            // [SECURITY FIX]: Rejection Sampling (Bias Removal)
            let mut hasher = blake3::Hasher::new();
            hasher.update(&global_root.p_factor.to_digits(rug::integer::Order::Lsf));
            hasher.update(user_id.as_bytes());
            let mut ctr = 0u64;
            
            let challenge_axis = loop {
                hasher.update(&ctr.to_le_bytes());
                let hash_output = hasher.finalize();
                let mut bytes = [0u8; 8];
                bytes.copy_from_slice(&hash_output.as_bytes()[0..8]);
                let rand_val = u64::from_le_bytes(bytes);
                
                // Reject values in the "unfair" range
                if rand_val < (u64::MAX - (u64::MAX % guard.dimensions as u64)) {
                    break (rand_val % guard.dimensions as u64) as usize;
                }
                ctr += 1;
            };
            
            let path = guard.get_segment_tree_path(&coord, challenge_axis);
            let anchors = guard.get_orthogonal_anchors(&coord, challenge_axis);
            
            // [SECURITY FIX]: Privacy - No target_coord returned
            Ok(HtpResponse::ProofBundle {
                request_id,
                primary_path: path,
                orthogonal_anchors: anchors,
                epoch: 1,
            })
        },
        
        HtpRequest::GetGlobalRoot => {
            let guard = tensor.read().await;
            let root = match &guard.cached_root {
                Some(r) => r.clone(),
                None => guard.compute_root_internal()?
            };
            Ok(HtpResponse::GlobalRoot(root))
        }
    }
}
