mod core;
mod topology;

use crate::core::param::SystemParameters;
use crate::core::primes::hash_to_prime;
use crate::core::affine::AffineTuple;
use crate::topology::tensor::HyperTensor;

fn main() {
    println!("=== Hyper-Tensor Protocol (Technical Showcase) ===");

    // ---------------------------------------------------------
    // 第一步：运行时生成“宇宙参数” (Trustless Setup)
    // ---------------------------------------------------------
    // 在真实世界中，这个 seed 来自比特币最新区块哈希
    let seed = b"Block #891234: 0000000000000000a1b2c3..."; 
    let params = SystemParameters::from_random_seed(seed, 128); // 生成 128位的 Delta

    // ---------------------------------------------------------
    // 第二步：初始化空的超张量 (Hyper-Tensor Init)
    // ---------------------------------------------------------
    // 创建一个 4维张量，每维长度 100
    let mut tensor = HyperTensor::new(4, 100, params.discriminant.clone());
    println!("[Topology] 4D-Tensor initialized. Capacity: 100^4 users.");

    // ---------------------------------------------------------
    // 第三步：模拟用户注册 (Runtime Data Ingestion)
    // ---------------------------------------------------------
    let user_ids = vec!["Alice_001", "Bob_002", "Charlie_003"];

    for uid in user_ids {
        // 1. 自动计算身份素数 P
        let p = hash_to_prime(uid, 64);
        
        // 2. 自动生成时空因子 (这里简化为 Identity，实际会根据 depth 变化)
        // 3. 封装为仿射元组
        let tuple = AffineTuple {
            p_factor: p.clone(),
            q_shift: crate::core::algebra::ClassGroupElement::identity(&params.discriminant), 
        };

        // 4. 插入张量 (映射 ID -> 坐标 -> 存入 Sparse Map)
        tensor.insert_by_id(uid, tuple);
        
        println!("[Ingest] User {} mapped to Prime {}...", uid, p.to_string_radix(16));
    }

    // ---------------------------------------------------------
    // 第四步：计算全局根 (Folding)
    // ---------------------------------------------------------
    println!("[Compute] Folding dimensions...");
    let start = std::time::Instant::now();
    
    let global_root = tensor.calculate_global_root();
    
    let duration = start.elapsed();
    println!("[Success] Global Root Calculated in {:?}", duration);
    println!("Root P-Factor: {:x}...", global_root.p_factor); // 实际上是所有素数的积
}
