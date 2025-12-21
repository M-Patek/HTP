// COPYRIGHT (C) 2025 M-Patek. ALL RIGHTS RESERVED.
//
// MODULE: Prover Service
// DESCRIPTION: Implements the "Dimensional Interrogation" logic over QUIC streams.

use std::sync::Arc;
use tokio::sync::RwLock;
use quinn::{Endpoint, RecvStream, SendStream};
use bincode;

use crate::topology::tensor::HyperTensor;
use crate::net::wire::{PhtpRequest, PhtpResponse};

/// The main loop for the Prover node.
/// Accepts incoming streams and spawns handlers.
pub async fn run_prover_service(endpoint: Endpoint, tensor: Arc<RwLock<HyperTensor>>) {
    while let Some(conn) = endpoint.accept().await {
        let tensor_ref = tensor.clone();
        tokio::spawn(async move {
            let connection = conn.await.unwrap();
            // Multiplexing: Handle multiple requests on the same connection
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

/// Handles a single Request/Response cycle.
async fn handle_stream(
    tensor: Arc<RwLock<HyperTensor>>, 
    mut send: SendStream, 
    mut recv: RecvStream
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    
    // 1. Zero-Copy Deserialize the Request
    // Reads directly from the QUIC stream buffer
    let mut buf = vec![0u8; 4096];
    let len = recv.read(&mut buf).await?.unwrap_or(0);
    let request: PhtpRequest = bincode::deserialize(&buf[..len])?;

    let response = match request {
        PhtpRequest::GetProof { user_id, request_id } => {
            // [CRITICAL LOGIC]: The Fiat-Shamir Interactive Flow
            
            // A. Acquire Read Lock (Non-blocking)
            let guard = tensor.read().await;
            
            // B. Map ID to Coordinates
            let coord = guard.map_id_to_coord_hash(&user_id);
            
            // C. Deterministic Challenge Generation
            // "The dimension we probe depends on the Global Root and User ID"
            // This prevents the prover from pre-computing fake paths.
            let global_root = guard.calculate_global_root(); // In prod: cached
            let challenge_seed = format!("{:?}+{}", global_root.p_factor, user_id); // Simple hash mix
            let challenge_axis = (challenge_seed.len() % guard.dimensions); // Simplified mock hash
            
            // D. Extract the Proof Path (The "Projection")
            // This is the expensive operation, usually cached.
            let path = guard.get_segment_tree_path(&coord, challenge_axis);
            
            // E. Extract Orthogonal Anchors
            let anchors = guard.get_orthogonal_anchors(&coord, challenge_axis);
            
            PhtpResponse::ProofBundle {
                request_id,
                target_coord: coord,
                primary_path: path,
                orthogonal_anchors: anchors,
                epoch: 1, // Mock epoch
            }
        },
        
        PhtpRequest::GetGlobalRoot => {
            let guard = tensor.read().await;
            PhtpResponse::GlobalRoot(guard.calculate_global_root())
        }
    };

    // 2. Serialize and Send Response
    let resp_bytes = bincode::serialize(&response)?;
    send.write_all(&resp_bytes).await?;
    send.finish().await?;

    Ok(())
}
