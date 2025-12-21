// COPYRIGHT (C) 2025 M-Patek. ALL RIGHTS RESERVED.

use serde::{Serialize, Deserialize};
use crate::core::affine::AffineTuple;

pub const PROTOCOL_VERSION: u16 = 1;

#[derive(Serialize, Deserialize, Debug)]
pub struct RequestHeader {
    pub version: u16,
    pub timestamp: u64, // 防止重放攻击
    pub request_id: u64,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum HtpRequest {
    GetProof {
        header: RequestHeader,
        user_id: String,
    },
    GetGlobalRoot {
        header: RequestHeader,
    },
    // [NEW]: 支持网络写入/注册
    RegisterUser {
        header: RequestHeader,
        user_id: String,
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub enum HtpResponse {
    ProofBundle {
        request_id: u64,
        primary_path: Vec<AffineTuple>,
        orthogonal_anchors: Vec<AffineTuple>,
        epoch: u64,
    },
    GlobalRoot(AffineTuple),
    RegisterSuccess { 
        request_id: u64, 
        epoch: u64 
    },
    Error(String),
}
