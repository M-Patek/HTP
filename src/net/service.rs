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
    
    let mut buf = Vec::new();
    let mut stream_limit = recv.take(1024 * 1024);
    
    if let Err(e) = stream_limit.read_to_end(&mut buf).await {
         return Err(Box::new(e));
    }
    
    if buf.is_empty() { return Ok(()); }

    let request: HtpRequest = bincode::deserialize(&buf)?;

    let response = match request {
        HtpRequest::GetProof { user_id, request_id } => {
            // [CRITICAL FIX]: DoS Protection - Downgrade to READ Lock
            // 修复了“读请求抢占写锁”导致的串行化 DoS 问题。
            // 现在 Proof 生成是并发的 (Read-only)。
            let guard = tensor.read().await;
            
            let coord = guard.map_id_to_coord_hash(&user_id);
            
            // 为了避免由于 calculate_global_root 需要 &mut 而导致的编译错误或死锁，
            // 我们在服务层只读取缓存。如果缓存未命中，我们返回 Identity 或在后台计算。
            // (生产环境中应有一个专门的 Background Worker 更新 Root)
            let global_root = match &guard.cached_root {
                Some(r) => r.clone(),
                None => crate::core::affine::AffineTuple::identity(&guard.discriminant) 
            };
            
            // [SECURITY FIX]: Bias Removal (Rejection Sampling)
            // 修复了取模偏差。
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
                
                // Rejection Sampling: 消除取模偏差
                if rand_val < (u64::MAX - (u64::MAX % guard.dimensions as u64)) {
                    break (rand_val % guard.dimensions as u64) as usize;
                }
                ctr += 1;
            };
            
            let path = guard.get_segment_tree_path(&coord, challenge_axis);
            let anchors = guard.get_orthogonal_anchors(&coord, challenge_axis);
            
            HtpResponse::ProofBundle {
                request_id,
                // [SECURITY FIX]: Privacy - Do NOT send explicit coordinates
                primary_path: path,
                orthogonal_anchors: anchors,
                epoch: 1,
            }
        },
        
        HtpRequest::GetGlobalRoot => {
            let guard = tensor.read().await;
            let root = guard.cached_root.clone().unwrap_or_else(|| crate::core::affine::AffineTuple::identity(&guard.discriminant));
            HtpResponse::GlobalRoot(root)
        }
    };

    let resp_bytes = bincode::serialize(&response)?;
    send.write_all(&resp_bytes).await?;
    send.finish().await?;

    Ok(())
}
