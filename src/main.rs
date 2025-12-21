// COPYRIGHT (C) 2025 M-Patek. ALL RIGHTS RESERVED.

mod core;
mod topology;
mod net; // 引入 net 以便 cargo check 能通过

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
        
        // [FIX]: 真正的非交换初始化 (Non-commutative Evolution)
        // 使用 Generator 而不是 Identity
        let g = crate::core::algebra::ClassGroupElement::generator(&params.discriminant);
        
        let tuple = AffineTuple {
            p_factor: p.clone(),
            q_shift: g, 
        };

        match tensor.insert(uid, tuple) {
            Ok(_) => println!("[Ingest] User {} mapped (Non-commutative).", uid),
            Err(e) => eprintln!("❌ Insert Failed: {}", e),
        }
    }

    println!("[Compute] Folding dimensions...");
    match tensor.calculate_global_root() {
        Ok(root) => println!("[Success] Global Root: {:x}...", root.p_factor),
        Err(e) => eprintln!("Calculation failed: {}", e),
    }
}
