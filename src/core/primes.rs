// COPYRIGHT (C) 2025 M-Patek. ALL RIGHTS RESERVED.

use rug::Integer;
use blake3::Hasher;

/// 运行时生成：将任意用户 ID 转换为一个唯一的素数
/// [SECURITY FIX]: Returns Result instead of Panicking.
/// 修复了 DoS 漏洞：之前的 panic! 会导致服务器在恶意输入下崩溃。
pub fn hash_to_prime(user_id: &str, bit_size: u32) -> Result<Integer, String> {
    let mut nonce = 0u64;
    let max_attempts = 10_000; // Safety Limit
    
    while nonce < max_attempts {
        let mut hasher = Hasher::new();
        hasher.update(user_id.as_bytes());
        hasher.update(&nonce.to_le_bytes());
        let hash = hasher.finalize();

        let mut candidate = Integer::from_digits(hash.as_bytes(), rug::integer::Order::Lsf);
        
        // 强制最高位为1 (保证大小)，最低位为1 (保证奇数)
        candidate.set_bit(bit_size - 1, true);
        candidate.set_bit(0, true);

        // 1. 快速筛选 (Small Prime Sieve)
        if candidate.mod_u(3) == 0 || candidate.mod_u(5) == 0 {
            nonce += 1;
            continue;
        }

        // 2. 强素数测试
        if candidate.is_probably_prime(25) != rug::integer::IsPrime::No {
            return Ok(candidate);
        }

        nonce += 1;
    }
    
    // [Fix]: Return error gracefully
    Err(format!("❌ Failed to generate prime for '{}' after {} attempts.", user_id, max_attempts))
}
