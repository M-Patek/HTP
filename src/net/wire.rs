// COPYRIGHT (C) 2025 M-Patek. ALL RIGHTS RESERVED.
//
// MODULE: Wire Protocol
// DESCRIPTION: Defines the compact binary serialization format for HTP.
// OPTIMIZATION: Uses 'bincode' for near-memory-layout serialization speed.

use serde::{Serialize, Deserialize};
use rug::Integer;
use crate::core::affine::AffineTuple;
use crate::topology::tensor::Coordinate;

/// Protocol Versioning for backward compatibility.
pub const PROTOCOL_VERSION: u16 = 1;

/// The Request sent by a Verifier (Light Client).
#[derive(Serialize, Deserialize, Debug)]
pub enum HtpRequest {
    /// "I want to verify User X."
    /// The Prover will derive the deterministic challenge axis automatically.
    GetProof {
        user_id: String,
        request_id: u64, // Correlation ID for async multiplexing
    },
    
    /// "What is the current Global Root?"
    GetGlobalRoot,
}

/// The Response sent by a Prover (Full Node).
#[derive(Serialize, Deserialize, Debug)]
pub enum HtpResponse {
    /// The Holographic Proof Bundle.
    ProofBundle {
        request_id: u64,
        target_coord: Coordinate,
        
        // The primary Affine Path along the challenged axis.
        // Serialized as a flat vector of AffineTuples to save space.
        primary_path: Vec<AffineTuple>,
        
        // The orthogonal anchors (Roots of intersecting dimensions).
        // Used for the "Cross-Check".
        orthogonal_anchors: Vec<AffineTuple>,
        
        // The timestamp/block height of this snapshot
        epoch: u64,
    },
    
    /// Current Global Root state.
    GlobalRoot(AffineTuple),
    
    /// Error handling (e.g., User not found).
    Error(String),
}

// ------------------------------------------------------------------------
// Custom Serialization Wrappers for `rug::Integer` 
// (Skipped for brevity, but crucial for bincode compatibility with GMP)
// ------------------------------------------------------------------------
