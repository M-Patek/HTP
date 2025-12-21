// COPYRIGHT (C) 2025 PHOENIX PROJECT. ALL RIGHTS RESERVED.

use super::tensor::HyperTensor;
use crate::core::affine::AffineTuple;

impl HyperTensor {
    /// Computes the Global Root of the Hyper-Tensor.
    /// This performs the recursive dimensional folding.
    /// 
    /// In production, this result is cached using Merkle Trees.
    pub fn calculate_global_root(&self) -> AffineTuple {
        // Recursively fold from Dimension 0 to D-1
        self.fold_recursive(0, vec![])
    }

    /// Internal recursive function.
    /// fixed_coords: The coordinates fixed by higher recursion levels.
    fn fold_recursive(&self, current_dim: usize, fixed_coords: Vec<usize>) -> AffineTuple {
        // Base Case: If we have fixed all dimensions, we are at a single cell.
        if current_dim == self.dimensions {
            return self.get(&fixed_coords);
        }

        // Recursive Step: Aggregate along the current_dim.
        // We need to combine L results from the deeper recursion level.
        
        // Initialize accumulator with Identity
        let mut layer_agg = AffineTuple::identity(&self.discriminant);

        for i in 0..self.side_length {
            let mut next_coords = fixed_coords.clone();
            next_coords.push(i);

            // Recursion: Get the folded value of the sub-tensor
            let sub_result = self.fold_recursive(current_dim + 1, next_coords);

            // Combine: layer_agg = layer_agg (+) sub_result
            // This represents the Segment Tree aggregation for this "line".
            layer_agg = layer_agg.compose(&sub_result, &self.discriminant);
        }

        layer_agg
    }
}
