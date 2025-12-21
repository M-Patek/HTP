// COPYRIGHT (C) 2025 M-Patek. ALL RIGHTS RESERVED.

use serde::{Serialize, Deserialize};
use rug::Integer;
use crate::core::affine::AffineTuple;
use crate::topology::tensor::Coordinate;

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
        // Removed `target_coord` to prevent leaking user position in the tensor.
        // Client derives this from their own ID + Public Parameters.
        
        primary_path: Vec<AffineTuple>,
        orthogonal_anchors: Vec<AffineTuple>,
        epoch: u64,
    },
    
    GlobalRoot(AffineTuple),
    
    Error(String),
}
