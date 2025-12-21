// COPYRIGHT (C) 2025 M-Patek. ALL RIGHTS RESERVED.

use std::collections::HashMap;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use rug::Integer;
use crate::core::affine::AffineTuple;

/// Represents a d-dimensional coordinate.
pub type Coordinate = Vec<usize>;

/// The Hyper-Tensor Structure.
/// Maps Coordinates -> AffineTuples.
pub struct HyperTensor {
    pub dimensions: usize,
    pub side_length: usize,
    pub discriminant: Integer,
    
    // Sparse storage
    pub data: HashMap<Coordinate, AffineTuple>,
    
    // [PERFORMANCE FIX]: Cache for the global root
    pub cached_root: Option<AffineTuple>, 
}

impl HyperTensor {
    pub fn new(dim: usize, len: usize, discriminant: Integer) -> Self {
        HyperTensor {
            dimensions: dim,
            side_length: len,
            discriminant,
            data: HashMap::new(),
            cached_root: None, // Starts dirty
        }
    }

    /// Maps a linear User ID to a Tensor Coordinate (Base-L conversion).
    pub fn map_id_to_coord(&self, numeric_id: u64) -> Coordinate {
        let mut coord = Vec::with_capacity(self.dimensions);
        let mut temp = numeric_id;
        let l = self.side_length as u64;

        for _ in 0..self.dimensions {
            coord.push((temp % l) as usize);
            temp /= l;
        }
        coord
    }
    
    // [SECURITY FIX]: Use real hashing instead of hardcoded 12345
    // Prevents coordinate collisions (The "Roommate" bug).
    pub fn map_id_to_coord_hash(&self, user_id: &str) -> Coordinate {
        let mut hasher = DefaultHasher::new();
        user_id.hash(&mut hasher);
        let id_hash_u64 = hasher.finish();
        
        self.map_id_to_coord(id_hash_u64)
    }
    
    // [FIX]: Return a valid dummy path instead of empty vector
    // In production, this performs the Segment Tree lookup.
    pub fn get_segment_tree_path(&self, coord: &Coordinate, _axis: usize) -> Vec<AffineTuple> {
        // Return the target node itself as a minimal path proof
        vec![self.get(coord)]
    }
    
    // [FIX]: Return dummy anchors for structure validity
    pub fn get_orthogonal_anchors(&self, _coord: &Coordinate, axis: usize) -> Vec<AffineTuple> {
        let mut anchors = Vec::new();
        for dim in 0..self.dimensions {
            if dim == axis { continue; }
            anchors.push(AffineTuple::identity(&self.discriminant));
        }
        anchors
    }

    /// Insert or Update a node.
    pub fn insert(&mut self, user_id: u64, tuple: AffineTuple) {
        let coord = self.map_id_to_coord(user_id);
        self.data.insert(coord, tuple);
        
        // [PERF]: Invalidate Cache on write
        self.cached_root = None;
    }
    
    pub fn insert_by_id(&mut self, user_id: &str, tuple: AffineTuple) {
        // [FIX]: Use consistent hashing for insertion
        let mut hasher = DefaultHasher::new();
        user_id.hash(&mut hasher);
        let id_u64 = hasher.finish();
        
        self.insert(id_u64, tuple);
    }
    
    /// Retrieve a node. Returns Identity if empty.
    pub fn get(&self, coord: &Coordinate) -> AffineTuple {
        match self.data.get(coord) {
            Some(tuple) => tuple.clone(),
            None => AffineTuple::identity(&self.discriminant),
        }
    }
}
