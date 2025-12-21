// COPYRIGHT (C) 2025 M-Patek. ALL RIGHTS RESERVED.

use super::algebra::ClassGroupElement;
use rug::Integer;

/// The Affine Tuple A = (P, Q).
/// Represents the transformation: S -> S^P * Q
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)] // Added derive
pub struct AffineTuple {
    pub p_factor: Integer,      // The Prime (or composite product of primes)
    pub q_shift: ClassGroupElement, // The Class Group Element shift
}

impl AffineTuple {
    /// Creates the Identity Tuple (1, Identity_Cl).
    pub fn identity(discriminant: &Integer) -> Self {
        AffineTuple {
            p_factor: Integer::from(1),
            q_shift: ClassGroupElement::identity(discriminant),
        }
    }

    /// The core NON-COMMUTATIVE composition logic.
    /// Formula: A_merge = A1 (+) A2 = (P1*P2, Q1^P2 * Q2)
    pub fn compose(&self, other: &Self, discriminant: &Integer) -> Self {
        // [SECURITY FIX]: Resource Exhaustion Protection
        // 限制 P-Factor 的最大位数。如果 P 太大，意味着聚合层数过多或遭受攻击。
        // 假设正常情况下不会超过 100MB 的大数 (根据业务调整)。
        // 这里的 check 防止内存耗尽攻击 (OOM)。
        let p_bits = self.p_factor.significant_bits() + other.p_factor.significant_bits();
        if p_bits > 1024 * 1024 * 800 { // Limit to ~100MB (Example)
             panic!("❌ Security Halt: Affine P-Factor exceeded safety limit (State Bloat Detected).");
        }

        // 1. New P = P1 * P2
        let new_p = Integer::from(&self.p_factor * &other.p_factor);

        // 2. New Q = (Q1 ^ P2) * Q2
        let q1_pow_p2 = self.q_shift.pow(&other.p_factor, discriminant);
        let new_q = q1_pow_p2.compose(&other.q_shift, discriminant);

        AffineTuple {
            p_factor: new_p,
            q_shift: new_q,
        }
    }
}
