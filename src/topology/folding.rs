// COPYRIGHT (C) 2025 M-Patek. ALL RIGHTS RESERVED.

use super::tensor::HyperTensor;
use crate::core::affine::AffineTuple;
use std::collections::HashSet;

impl HyperTensor {
    /// Computes the Global Root of the Hyper-Tensor.
    /// [PERFORMANCE FIX]: 使用了真正的预处理索引，将复杂度从 O(L^d * N) 降低到 O(L^d + N)。
    pub fn calculate_global_root(&mut self) -> AffineTuple {
        // Check cache
        if let Some(ref root) = self.cached_root {
            return root.clone();
        }

        // Cache miss: Compute
        let root = self.compute_root_internal();
        
        // Update cache
        self.cached_root = Some(root.clone());
        root
    }

    /// Read-only computation helper (can be used by readers)
    pub fn compute_root_internal(&self) -> AffineTuple {
        // [ALGORITHMIC OPTIMIZATION]: 
        // 在递归之前，先构建一个 "Active Prefixes" 集合。
        // 这样查询 "subtree_has_data" 就变成了 O(1) 的哈希查找，而不是 O(N) 的全表扫描。
        let active_prefixes = self.build_active_prefixes();
        
        self.fold_recursive(0, vec![], &active_prefixes)
    }

    /// 预计算所有存在的路径前缀
    fn build_active_prefixes(&self) -> HashSet<Vec<usize>> {
        let mut prefixes = HashSet::new();
        for coord in self.data.keys() {
            // 对于每个存在的坐标，将其所有父路径加入集合
            for i in 1..=self.dimensions {
                prefixes.insert(coord[0..i].to_vec());
            }
        }
        prefixes
    }

    /// Internal recursive function with EFFICIENT PRUNING.
    fn fold_recursive(
        &self, 
        current_dim: usize, 
        fixed_coords: Vec<usize>,
        active_prefixes: &HashSet<Vec<usize>>
    ) -> AffineTuple {
        if current_dim == self.dimensions {
            return self.get(&fixed_coords);
        }

        // [FIX]: O(1) Pruning Lookup
        // 如果当前路径前缀不在预计算的集合中，说明该分支下没有任何数据，直接返回 Identity。
        if !fixed_coords.is_empty() && !active_prefixes.contains(&fixed_coords) {
             return AffineTuple::identity(&self.discriminant);
        }

        let mut layer_agg = AffineTuple::identity(&self.discriminant);

        for i in 0..self.side_length {
            let mut next_coords = fixed_coords.clone();
            next_coords.push(i);

            // 递归前先检查下一跳是否存在于前缀集中
            if active_prefixes.contains(&next_coords) {
                let sub_result = self.fold_recursive(current_dim + 1, next_coords, active_prefixes);
                layer_agg = layer_agg.compose(&sub_result, &self.discriminant);
            }
            // else: skip (Identity composition is No-Op)
        }

        layer_agg
    }
    
    // Deprecated inefficient helper removed.
}
