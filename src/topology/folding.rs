// COPYRIGHT (C) 2025 M-Patek. ALL RIGHTS RESERVED.

use super::tensor::HyperTensor;
use crate::core::affine::AffineTuple;
use std::collections::HashSet;

impl HyperTensor {
    /// Computes the Global Root of the Hyper-Tensor.
    /// [FIX]: Returns Result to handle calculation errors.
    pub fn calculate_global_root(&mut self) -> Result<AffineTuple, String> {
        if let Some(ref root) = self.cached_root {
            return Ok(root.clone());
        }

        let root = self.compute_root_internal()?;
        
        self.cached_root = Some(root.clone());
        Ok(root)
    }

    pub fn compute_root_internal(&self) -> Result<AffineTuple, String> {
        let active_prefixes = self.build_active_prefixes();
        // Start recursion
        self.fold_recursive(0, vec![], &active_prefixes)
    }

    fn build_active_prefixes(&self) -> HashSet<Vec<usize>> {
        let mut prefixes = HashSet::new();
        for coord in self.data.keys() {
            for i in 1..=self.dimensions {
                prefixes.insert(coord[0..i].to_vec());
            }
        }
        prefixes
    }

    fn fold_recursive(
        &self, 
        current_dim: usize, 
        fixed_coords: Vec<usize>,
        active_prefixes: &HashSet<Vec<usize>>
    ) -> Result<AffineTuple, String> {
        if current_dim == self.dimensions {
            return Ok(self.get(&fixed_coords));
        }

        // [FIX]: Recursion Depth Check (Simple safeguard)
        if current_dim > 20 { 
            return Err("Recursion limit exceeded".to_string());
        }

        if !fixed_coords.is_empty() && !active_prefixes.contains(&fixed_coords) {
             return Ok(AffineTuple::identity(&self.discriminant));
        }

        let mut layer_agg = AffineTuple::identity(&self.discriminant);

        for i in 0..self.side_length {
            let mut next_coords = fixed_coords.clone();
            next_coords.push(i);

            if active_prefixes.contains(&next_coords) {
                let sub_result = self.fold_recursive(current_dim + 1, next_coords, active_prefixes)?;
                // [FIX]: Handle Result from compose
                layer_agg = layer_agg.compose(&sub_result, &self.discriminant)?;
            }
        }

        Ok(layer_agg)
    }
}
