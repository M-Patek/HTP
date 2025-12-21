// COPYRIGHT (C) 2025 M-Patek. ALL RIGHTS RESERVED.

use super::algebra::ClassGroupElement;
use rug::Integer;

/// The Affine Tuple A = (P, Q).
/// Represents the transformation: S -> S^P * Q
#[derive(Clone, Debug)]
pub struct AffineTuple {
    pub p_factor: Integer,      // The Prime (or composite product of primes)
    pub q_shift: ClassGroupElement, // The Class Group Element shift
}

impl AffineTuple {
    /// Creates the Identity Tuple (1, Identity_Cl).
    /// Used for empty tensor cells.
    pub fn identity(discriminant: &Integer) -> Self {
        AffineTuple {
            p_factor: Integer::from(1),
            q_shift: ClassGroupElement::identity(discriminant),
        }
    }

    /// The core NON-COMMUTATIVE composition logic.
    /// Formula: A_merge = A1 (+) A2 = (P1*P2, Q1^P2 * Q2)
    ///
    /// Proof of Associativity ensures this can be parallelized in Segment Trees.
    pub fn compose(&self, other: &Self, discriminant: &Integer) -> Self {
        // 1. New P = P1 * P2
        let new_p = Integer::from(&self.p_factor * &other.p_factor);

        // 2. New Q = (Q1 ^ P2) * Q2
        // Note the order! This is where non-commutativity happens.
        let q1_pow_p2 = self.q_shift.pow(&other.p_factor, discriminant);
        let new_q = q1_pow_p2.compose(&other.q_shift, discriminant);

        AffineTuple {
            p_factor: new_p,
            q_shift: new_q,
        }
    }
}
