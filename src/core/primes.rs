// COPYRIGHT (C) 2025 M-Patek. ALL RIGHTS RESERVED.

use rug::Integer;
use blake3::Hasher;

/// [SECURITY FIX]: Returns Result instead of Panic
pub fn hash_to_prime(user_id: &str, bit_size: u32) -> Result<Integer, String> {
    let mut nonce = 0u64;
    let max_attempts = 10_000; // DoS Protection (Limit CPU usage)
    
    while nonce < max_attempts {
        let mut hasher = Hasher::new();
        hasher.update(user_id.as_bytes());
        hasher.update(&nonce.to_le_bytes());
        let hash = hasher.finalize();

        let mut candidate = Integer::from_digits(hash.as_bytes(), rug::integer::Order::Lsf);
        candidate.set_bit(bit_size - 1, true);
        candidate.set_bit(0, true);

        if candidate.mod_u(3) == 0 || candidate.mod_u(5) == 0 {
            nonce += 1;
            continue;
        }

        if candidate.is_probably_prime(25) != rug::integer::IsPrime::No {
            return Ok(candidate);
        }

        nonce += 1;
    }
    
    Err(format!("❌ Failed to generate prime for '{}' after {} attempts.", user_id, max_attempts))
}
