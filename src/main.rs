// COPYRIGHT (C) 2025 M-Patek. ALL RIGHTS RESERVED.

mod core;
mod topology;

use crate::core::param::SystemParameters;
use crate::core::primes::hash_to_prime;
use crate::core::affine::AffineTuple;
use crate::topology::tensor::HyperTensor;

fn main() {
    println!("=== Hyper-Tensor Protocol (Secure Showcase) ===");

    // [SECURITY FIX]: Use 2048-bit params
    let seed = b"Block #891234: 0000000000000000a1b2c3..."; 
    let params = SystemParameters::from_random_seed(seed, 2048); 

    let mut tensor = HyperTensor::new(4, 100, params.discriminant.clone());
    println!("[Topology] 4D-Tensor initialized. Capacity: 100^4 users.");

    let user_ids = vec!["Alice_001", "Bob_002", "Charlie_003"];

    for uid in user_ids {
        let p = hash_to_prime(uid, 64);
        
        let tuple = AffineTuple {
            p_factor: p.clone(),
            q_shift: crate::core::algebra::ClassGroupElement::identity(&params.discriminant), 
        };

        // [FIX]: Use insert_by_id which now uses real hashing
        tensor.insert_by_id(uid, tuple);
        
        println!("[Ingest] User {} mapped to Prime {}...", uid, p.to_string_radix(16));
    }

    println!("[Compute] Folding dimensions (with pruning)...");
    let start = std::time::Instant::now();
    
    // This will now use caching and pruning
    let global_root = tensor.calculate_global_root();
    
    let duration = start.elapsed();
    println!("[Success] Global Root Calculated in {:?}", duration);
    println!("Root P-Factor: {:x}...", global_root.p_factor);
}
