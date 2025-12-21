// COPYRIGHT (C) 2025 M-Patek. ALL RIGHTS RESERVED.

use serde::{Serialize, Deserialize};
use crate::core::affine::AffineTuple;

pub const PROTOCOL_VERSION: u16 = 1;

#[derive(Serialize, Deserialize, Debug)]
pub enum HtpRequest {
    GetProof {
        user_id: String,
        request_id: u64,
    },
    GetGlobalRoot,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum HtpResponse {
    ProofBundle {
        request_id: u64,
        
        // [SECURITY FIX]: Privacy Enhancement
        // Removed `target_coord` to prevent leaking user position/existence in specific buckets.
        
        primary_path: Vec<AffineTuple>,
        orthogonal_anchors: Vec<AffineTuple>,
        epoch: u64,
    },
    
    GlobalRoot(AffineTuple),
    
    Error(String),
}
