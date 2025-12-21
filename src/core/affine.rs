// COPYRIGHT (C) 2025 M-Patek. ALL RIGHTS RESERVED.

use super::algebra::ClassGroupElement;
use rug::Integer;

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

    /// [SECURITY FIX]: Return Result, Check Limits
    pub fn compose(&self, other: &Self, discriminant: &Integer) -> Result<Self, String> {
        // [SECURITY FIX]: Resource Exhaustion / OOM Protection
        // Check if P-factor grows too large (e.g. > 100MB representation)
        let p_bits = self.p_factor.significant_bits() + other.p_factor.significant_bits();
        if p_bits > 1024 * 1024 * 800 { 
             return Err("❌ Security Halt: Affine P-Factor exceeded safety limit (State Bloat Detected).".to_string());
        }

        let new_p = Integer::from(&self.p_factor * &other.p_factor);

        // Propagate math errors
        let q1_pow_p2 = self.q_shift.pow(&other.p_factor, discriminant)?;
        let new_q = q1_pow_p2.compose(&other.q_shift, discriminant)?;

        Ok(AffineTuple {
            p_factor: new_p,
            q_shift: new_q,
        })
    }
}
