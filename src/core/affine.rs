// COPYRIGHT (C) 2025 M-Patek. ALL RIGHTS RESERVED.

use super::algebra::ClassGroupElement;
use rug::Integer;

/// The Affine Tuple A = (P, Q).
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)] 
pub struct AffineTuple {
    pub p_factor: Integer,      
    pub q_shift: ClassGroupElement, 
}

impl AffineTuple {
    pub fn identity(discriminant: &Integer) -> Self {
        AffineTuple {
            p_factor: Integer::from(1),
            q_shift: ClassGroupElement::identity(discriminant),
        }
    }

    /// [SECURITY FIX]: Return Result to prevent Crash-on-Error
    pub fn compose(&self, other: &Self, discriminant: &Integer) -> Result<Self, String> {
        // [SECURITY FIX]: Resource Exhaustion Protection
        let p_bits = self.p_factor.significant_bits() + other.p_factor.significant_bits();
        if p_bits > 1024 * 1024 * 800 { 
             // [FIX]: Return error instead of panic
             return Err("❌ Security Halt: Affine P-Factor exceeded safety limit (State Bloat Detected).".to_string());
        }

        // 1. New P = P1 * P2
        let new_p = Integer::from(&self.p_factor * &other.p_factor);

        // 2. New Q = (Q1 ^ P2) * Q2
        // Propagate errors from underlying algebra
        let q1_pow_p2 = self.q_shift.pow(&other.p_factor, discriminant)?;
        let new_q = q1_pow_p2.compose(&other.q_shift, discriminant)?;

        Ok(AffineTuple {
            p_factor: new_p,
            q_shift: new_q,
        })
    }
}
