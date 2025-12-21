// COPYRIGHT (C) 2025 M-Patek. ALL RIGHTS RESERVED.

use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::io::AsyncReadExt; // [FIX]: Needed for read_to_end
use quinn::{Endpoint, RecvStream, SendStream};
use bincode;
use blake3; // [FIX]: For secure challenge

use crate::topology::tensor::HyperTensor;
use crate::net::wire::{HtpRequest, HtpResponse};

pub async fn run_prover_service(endpoint: Endpoint, tensor: Arc<RwLock<HyperTensor>>) {
    while let Some(conn) = endpoint.accept().await {
        let tensor_ref = tensor.clone();
        tokio::spawn(async move {
            // [FIX]: Graceful Handshake Handling
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
    
    // [FIX]: Dynamic Buffer with Limit (Prevent Overflow/Truncation)
    let mut buf = Vec::new();
    let mut stream_limit = recv.take(1024 * 1024); // 1MB Limit
    
    if let Err(e) = stream_limit.read_to_end(&mut buf).await {
         return Err(Box::new(e));
    }
    
    if buf.is_empty() { return Ok(()); }

    let request: HtpRequest = bincode::deserialize(&buf)?;

    let response = match request {
        HtpRequest::GetProof { user_id, request_id } => {
            // [CRITICAL FIX]: Acquire WRITE lock to update cache during calc
            let mut guard = tensor.write().await;
            
            // Map using Secure Hash
            let coord = guard.map_id_to_coord_hash(&user_id);
            
            // Calculate Global Root (Cached)
            let global_root = guard.calculate_global_root();
            
            // [SECURITY FIX]: Fiat-Shamir with BLAKE3
            // Replaces string length modulus
            let mut hasher = blake3::Hasher::new();
            hasher.update(&global_root.p_factor.to_digits(rug::integer::Order::Lsf));
            hasher.update(user_id.as_bytes());
            let hash_output = hasher.finalize();
            
            let mut bytes = [0u8; 8];
            bytes.copy_from_slice(&hash_output.as_bytes()[0..8]);
            let randomness = u64::from_le_bytes(bytes);
            
            let challenge_axis = (randomness as usize) % guard.dimensions;
            
            let path = guard.get_segment_tree_path(&coord, challenge_axis);
            let anchors = guard.get_orthogonal_anchors(&coord, challenge_axis);
            
            HtpResponse::ProofBundle {
                request_id,
                target_coord: coord,
                primary_path: path,
                orthogonal_anchors: anchors,
                epoch: 1,
            }
        },
        
        HtpRequest::GetGlobalRoot => {
            let mut guard = tensor.write().await;
            HtpResponse::GlobalRoot(guard.calculate_global_root())
        }
    };

    let resp_bytes = bincode::serialize(&response)?;
    send.write_all(&resp_bytes).await?;
    send.finish().await?;

    Ok(())
}
