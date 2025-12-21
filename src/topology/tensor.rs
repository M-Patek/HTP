// COPYRIGHT (C) 2025 PHOENIX PROJECT. ALL RIGHTS RESERVED.

use std::collections::HashMap;
use rug::Integer;
use crate::core::affine::AffineTuple;

/// Represents a d-dimensional coordinate.
pub type Coordinate = Vec<usize>;

/// The Hyper-Tensor Structure.
/// Maps Coordinates -> AffineTuples.
pub struct HyperTensor {
    dimensions: usize,
    side_length: usize,
    discriminant: Integer,
    
    // Sparse storage: Only stores active nodes. 
    // Missing keys imply AffineTuple::identity().
    data: HashMap<Coordinate, AffineTuple>,
}

impl HyperTensor {
    pub fn new(dim: usize, len: usize, discriminant: Integer) -> Self {
        HyperTensor {
            dimensions: dim,
            side_length: len,
            discriminant,
            data: HashMap::new(),
        }
    }

    /// Maps a linear User ID to a Tensor Coordinate (Base-L conversion).
    /// e.g., ID 12345 -> [5, 45, 12, 0]
    pub fn map_id_to_coord(&self, user_id: u64) -> Coordinate {
        let mut coord = Vec::with_capacity(self.dimensions);
        let mut temp = user_id;
        let l = self.side_length as u64;

        for _ in 0..self.dimensions {
            coord.push((temp % l) as usize);
            temp /= l;
        }
        coord
    }

    /// Insert or Update a node.
    /// In a real system, this would trigger a partial tree update.
    pub fn insert(&mut self, user_id: u64, tuple: AffineTuple) {
        let coord = self.map_id_to_coord(user_id);
        self.data.insert(coord, tuple);
    }
    
    /// Retrieve a node. Returns Identity if empty.
    pub fn get(&self, coord: &Coordinate) -> AffineTuple {
        match self.data.get(coord) {
            Some(tuple) => tuple.clone(),
            None => AffineTuple::identity(&self.discriminant),
        }
    }
}
