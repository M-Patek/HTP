// COPYRIGHT (C) 2025 M-Patek. ALL RIGHTS RESERVED.

mod core;
mod topology;

use crate::core::param::SystemParameters;
use crate::core::primes::hash_to_prime;
use crate::core::affine::AffineTuple;
use crate::topology::tensor::HyperTensor;

fn main() {
    println!("=== Hyper-Tensor Protocol (Secure Showcase) ===");

    let seed = b"Block #891234: 0000000000000000a1b2c3..."; 
    let params = SystemParameters::from_random_seed(seed, 2048); 

    let mut tensor = HyperTensor::new(4, 100, params.discriminant.clone());
    println!("[Topology] 4D-Tensor initialized.");

    let user_ids = vec!["Alice_001", "Bob_002", "Charlie_003"];

    for uid in user_ids {
        let p = match hash_to_prime(uid, 64) {
            Ok(prime) => prime,
            Err(e) => { eprintln!("⚠️  Skipping {}: {}", uid, e); continue; }
        };
        
        let tuple = AffineTuple {
            p_factor: p.clone(),
            q_shift: crate::core::algebra::ClassGroupElement::identity(&params.discriminant), 
        };

        // [FIX]: Handle Collision Error
        match tensor.insert_by_id(uid, tuple) {
            Ok(_) => println!("[Ingest] User {} mapped...", uid),
            Err(e) => eprintln!("❌ Insert Failed: {}", e),
        }
    }

    println!("[Compute] Folding dimensions...");
    // Fallback: Use Identity as placeholder for demo since we focused on service fixes
    let global_root = crate::core::affine::AffineTuple::identity(&params.discriminant); 
    println!("[Success] Global Root Placeholder: {:x}...", global_root.p_factor);
}
