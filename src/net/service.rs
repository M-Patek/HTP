// COPYRIGHT (C) 2025 M-Patek. ALL RIGHTS RESERVED.

use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::io::AsyncReadExt;
use quinn::{Endpoint, RecvStream, SendStream};
use bincode;
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
    
    // Limit request size to prevent memory exhaustion
    let mut stream_limit = recv.take(1024 * 1024);
    let mut buf = Vec::new();
    
    if let Err(e) = stream_limit.read_to_end(&mut buf).await {
         return Err(Box::new(e));
    }
    
    if buf.is_empty() { return Ok(()); }

    let request: HtpRequest = bincode::deserialize(&buf)?;

    let response = match request {
        HtpRequest::GetProof { user_id, request_id } => {
            // [CRITICAL FIX]: DoS Protection & Liveness
            // 使用读锁以允许并发查询。
            let guard = tensor.read().await;
            
            let coord = guard.map_id_to_coord_hash(&user_id);
            
            // [SELF-INSPECTION FIX]: 修复了返回假数据的 Bug。
            // 如果缓存存在，直接使用；
            // 如果缓存不存在（被写入操作清空），则使用只读方法 `compute_root_internal` 实时计算。
            // 虽然这比直接读缓存慢，但保证了数据的正确性（Consistency），且不会阻塞其他读者。
            let global_root = match &guard.cached_root {
                Some(r) => r.clone(),
                None => {
                    // 只有在缓存未命中时才付出计算代价
                    guard.compute_root_internal()
                }
            };
            
            // [SECURITY FIX]: Bias Removal (Rejection Sampling)
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
                
                // Rejection Sampling
                if rand_val < (u64::MAX - (u64::MAX % guard.dimensions as u64)) {
                    break (rand_val % guard.dimensions as u64) as usize;
                }
                ctr += 1;
            };
            
            let path = guard.get_segment_tree_path(&coord, challenge_axis);
            let anchors = guard.get_orthogonal_anchors(&coord, challenge_axis);
            
            HtpResponse::ProofBundle {
                request_id,
                primary_path: path,
                orthogonal_anchors: anchors,
                epoch: 1,
            }
        },
        
        HtpRequest::GetGlobalRoot => {
            let guard = tensor.read().await;
            // 同样的逻辑，确保 Root 总是最新的
            let root = guard.cached_root.clone().unwrap_or_else(|| guard.compute_root_internal());
            HtpResponse::GlobalRoot(root)
        }
    };

    let resp_bytes = bincode::serialize(&response)?;
    send.write_all(&resp_bytes).await?;
    send.finish().await?;

    Ok(())
}
