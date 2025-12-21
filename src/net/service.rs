// COPYRIGHT (C) 2025 M-Patek. ALL RIGHTS RESERVED.

use std::sync::Arc;
use tokio::sync::{RwLock, Semaphore};
use tokio::io::AsyncReadExt;
use quinn::{Endpoint, RecvStream, SendStream};
use bincode::{self, Options};
use blake3;
use log::{info, warn, error};

use crate::topology::tensor::HyperTensor;
use crate::net::wire::{HtpRequest, HtpResponse, RequestHeader};
use crate::core::affine::AffineTuple;

pub async fn run_prover_service(endpoint: Endpoint, tensor: Arc<RwLock<HyperTensor>>) {
    // [SECURITY FIX]: 限制最大并发连接数，防止 连接风暴 DoS
    let limit = Arc::new(Semaphore::new(10_000));

    while let Some(conn) = endpoint.accept().await {
        let permit = limit.clone().acquire_owned().await.unwrap();
        let tensor_ref = tensor.clone();
        
        tokio::spawn(async move {
            let _permit = permit; // 自动释放许可
            let connection = match conn.await {
                Ok(c) => c,
                Err(e) => {
                    warn!("[Net] Handshake failed: {}", e);
                    return;
                }
            };

            while let Ok((send, recv)) = connection.accept_bi().await {
                let t = tensor_ref.clone();
                tokio::spawn(async move {
                    if let Err(e) = handle_stream(t, send, recv).await {
                        warn!("[Net] Stream handled with error: {}", e);
                    }
                });
            }
        });
    }
}

// [FIX]: 错误信息净化，防止服务器内部路径/版本泄露
fn sanitize_error(e: String) -> String {
    error!("[Internal Error]: {}", e);
    "An internal server error occurred. Please contact admin.".to_string()
}

async fn handle_stream(
    tensor: Arc<RwLock<HyperTensor>>, 
    mut send: SendStream, 
    mut recv: RecvStream
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    
    // 限制读取大小，防止 Bincode 内存炸弹
    let mut stream_limit = recv.take(1024 * 1024); 
    let mut buf = Vec::new();
    if let Err(e) = stream_limit.read_to_end(&mut buf).await {
         return Err(Box::new(e));
    }
    if buf.is_empty() { return Ok(()); }

    let safe_config = bincode::DefaultOptions::new()
        .with_limit(5 * 1024 * 1024) 
        .with_fixint_encoding()
        .allow_trailing_bytes();

    let request: HtpRequest = match safe_config.deserialize(&buf) {
        Ok(r) => r,
        Err(e) => return Err(Box::new(e)),
    };

    let response = match process_request(&tensor, request).await {
        Ok(resp) => resp,
        Err(e) => HtpResponse::Error(sanitize_error(e)),
    };

    let resp_bytes = bincode::serialize(&response)?;
    send.write_all(&resp_bytes).await?;
    send.finish().await?;

    Ok(())
}

fn validate_header(header: &RequestHeader) -> Result<(), String> {
    if header.version != crate::net::wire::PROTOCOL_VERSION {
        return Err(format!("Protocol Mismatch: Server v{}, Client v{}", 
            crate::net::wire::PROTOCOL_VERSION, header.version));
    }
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
    // 简单的防重放：拒绝 60 秒以外的请求
    if header.timestamp < now.saturating_sub(60) || header.timestamp > now + 60 {
        return Err("Request expired or time skew too large".to_string());
    }
    Ok(())
}

async fn process_request(tensor: &Arc<RwLock<HyperTensor>>, request: HtpRequest) -> Result<HtpResponse, String> {
    match request {
        HtpRequest::GetProof { header, user_id } => {
            validate_header(&header)?;
            
            // [FIX]: 缓存击穿防护 (Double-Checked Locking)
            // 防止高并发下的雪崩效应
            let cached_opt = {
                let guard = tensor.read().await;
                guard.cached_root.clone()
            };

            let _root = if let Some(r) = cached_opt {
                r
            } else {
                let mut guard = tensor.write().await;
                if let Some(r) = &guard.cached_root {
                    r.clone() // 别的线程已经算好了
                } else {
                    info!("🧮 Cache miss. Computing Global Root...");
                    guard.calculate_global_root()?
                }
            };
            
            let guard = tensor.read().await;
            let coord = guard.map_id_to_coord_hash(&user_id);
            
            // [SECURITY FIX]: 隐私保护 - 假证明 (Dummy Proof)
            // 防止成员枚举攻击 (Membership Enumeration)
            if !guard.data.contains_key(&coord) {
                 let dummy_path = vec![AffineTuple::identity(&guard.discriminant); guard.dimensions];
                 return Ok(HtpResponse::ProofBundle {
                    request_id: header.request_id,
                    primary_path: dummy_path,
                    orthogonal_anchors: vec![],
                    epoch: 1,
                });
            }

            let path = guard.get_segment_tree_path(&coord, 0); 
            let anchors = guard.get_orthogonal_anchors(&coord, 0);
            
            Ok(HtpResponse::ProofBundle {
                request_id: header.request_id,
                primary_path: path,
                orthogonal_anchors: anchors,
                epoch: 1,
            })
        },
        
        HtpRequest::GetGlobalRoot { header } => {
            validate_header(&header)?;
            let guard = tensor.read().await;
            let root = match &guard.cached_root {
                Some(r) => r.clone(),
                None => guard.compute_root_internal()?
            };
            Ok(HtpResponse::GlobalRoot(root))
        },

        HtpRequest::RegisterUser { header, user_id } => {
            validate_header(&header)?;
            // [SECURITY FIX]: 防止日志伪造 (Log Injection)，转义用户输入
            info!("📝 Registering User '{}'", user_id.escape_debug());

            let mut guard = tensor.write().await;
            let p = crate::core::primes::hash_to_prime(&user_id, 64).map_err(|e| e.to_string())?;
            let q_gen = crate::core::algebra::ClassGroupElement::generator(&guard.discriminant);
            let tuple = AffineTuple { p_factor: p, q_shift: q_gen };

            guard.insert(&user_id, tuple)?;
            
            // 简单的同步持久化 (生产环境应异步处理)
            if let Err(e) = guard.save_to_disk("htp_tensor.db") {
                error!("Save failed: {}", e);
            }

            Ok(HtpResponse::RegisterSuccess { 
                request_id: header.request_id, 
                epoch: 1 
            })
        }
    }
}
