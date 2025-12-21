// COPYRIGHT (C) 2025 M-Patek. ALL RIGHTS RESERVED.

use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use rug::Integer;
use crate::core::affine::AffineTuple;
use blake3;

pub type Coordinate = Vec<usize>;

pub struct HyperTensor {
    pub dimensions: usize,
    pub side_length: usize,
    pub discriminant: Integer,
    pub data: HashMap<Coordinate, AffineTuple>,
    pub cached_root: Option<AffineTuple>, 
}

impl HyperTensor {
    pub fn new(dim: usize, len: usize, discriminant: Integer) -> Self {
        HyperTensor {
            dimensions: dim,
            side_length: len,
            discriminant,
            data: HashMap::new(),
            cached_root: None,
        }
    }

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
    
    pub fn map_id_to_coord_hash(&self, user_id: &str) -> Coordinate {
        let mut hasher = blake3::Hasher::new();
        hasher.update(user_id.as_bytes());
        hasher.update(b":htp:coord:"); 
        let hash_output = hasher.finalize();
        let mut bytes = [0u8; 8];
        bytes.copy_from_slice(&hash_output.as_bytes()[0..8]);
        let id_hash_u64 = u64::from_le_bytes(bytes);
        self.map_id_to_coord(id_hash_u64)
    }
    
    pub fn get_segment_tree_path(&self, coord: &Coordinate, _axis: usize) -> Vec<AffineTuple> {
        vec![self.get(coord)]
    }
    
    pub fn get_orthogonal_anchors(&self, _coord: &Coordinate, axis: usize) -> Vec<AffineTuple> {
        let mut anchors = Vec::new();
        for dim in 0..self.dimensions {
            if dim == axis { continue; }
            anchors.push(AffineTuple::identity(&self.discriminant));
        }
        anchors
    }

    pub fn insert(&mut self, user_id: u64, tuple: AffineTuple) -> Result<(), String> {
        let coord = self.map_id_to_coord(user_id);
        if self.data.contains_key(&coord) {
             return Err(format!("💥 Collision detected at {:?}. Write rejected.", coord));
        }
        self.data.insert(coord, tuple);
        self.cached_root = None;
        Ok(())
    }
    
    pub fn insert_by_id(&mut self, user_id: &str, tuple: AffineTuple) -> Result<(), String> {
        let coord = self.map_id_to_coord_hash(user_id);
        if self.data.contains_key(&coord) {
             return Err(format!("💥 Collision detected for User '{}' at {:?}.", user_id, coord));
        }
        self.data.insert(coord, tuple);
        self.cached_root = None;
        Ok(())
    }
    
    pub fn get(&self, coord: &Coordinate) -> AffineTuple {
        match self.data.get(coord) {
            Some(tuple) => tuple.clone(),
            None => AffineTuple::identity(&self.discriminant),
        }
    }
}
