// COPYRIGHT (C) 2025 M-Patek. ALL RIGHTS RESERVED.

use super::tensor::HyperTensor;
use crate::core::affine::AffineTuple;

impl HyperTensor {
    /// Computes the Global Root of the Hyper-Tensor.
    /// [PERFORMANCE FIX]: Uses Caching and Pruning to prevent DoS.
    pub fn calculate_global_root(&mut self) -> AffineTuple {
        // [FIX]: Check cache first
        if let Some(ref root) = self.cached_root {
            return root.clone();
        }

        // Cache miss: Compute expensive fold
        let root = self.fold_recursive(0, vec![]);
        
        // Update cache
        self.cached_root = Some(root.clone());
        root
    }

    /// [PERFORMANCE FIX]: Helper to check if a subtree contains data
    /// Used for pruning empty branches.
    fn subtree_has_data(&self, prefix: &[usize]) -> bool {
        // O(N) scan is faster than O(L^d) traversal for sparse tensors.
        // Production: Use Merkle Tree bitmasks.
        self.data.keys().any(|k| k.starts_with(prefix))
    }

    /// Internal recursive function with PRUNING.
    fn fold_recursive(&self, current_dim: usize, fixed_coords: Vec<usize>) -> AffineTuple {
        if current_dim == self.dimensions {
            return self.get(&fixed_coords);
        }

        // [FIX]: DoS Protection - Prune empty branches
        if !self.subtree_has_data(&fixed_coords) {
             return AffineTuple::identity(&self.discriminant);
        }

        let mut layer_agg = AffineTuple::identity(&self.discriminant);

        for i in 0..self.side_length {
            let mut next_coords = fixed_coords.clone();
            next_coords.push(i);

            // [FIX]: Double Check Pruning for sub-branches
            if self.subtree_has_data(&next_coords) {
                let sub_result = self.fold_recursive(current_dim + 1, next_coords);
                layer_agg = layer_agg.compose(&sub_result, &self.discriminant);
            }
            // else: skip (Identity composition is No-Op)
        }

        layer_agg
    }
}
