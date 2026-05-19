use crate::cir::CirOp;

/// Peephole + vectorization optimizer
pub struct Optimizer {
    pub vectorization_enabled: bool,
    pub constant_folding_enabled: bool,
}

impl Default for Optimizer {
    fn default() -> Self {
        Self {
            vectorization_enabled: true,
            constant_folding_enabled: true,
        }
    }
}

#[derive(Debug)]
pub struct OptimizationReport {
    pub original_ops: usize,
    pub optimized_ops: usize,
    pub vectorized_groups: usize,
    pub bytes_saved: usize,
}

impl Optimizer {
    /// Group adjacent range checks into vectorized ops
    pub fn vectorize(&self, ops: &[CirOp]) -> (Vec<CirOp>, usize) {
        if !self.vectorization_enabled || ops.len() < 2 {
            return (ops.to_vec(), 0);
        }

        let mut result = Vec::new();
        let mut i = 0;
        let mut groups = 0;

        while i < ops.len() {
            // Look for a run of range checks
            if let CirOp::RangeCheck { min, max } = &ops[i] {
                let run_min = *min;
                let run_max = *max;
                let mut run_len = 1;
                let mut j = i + 1;

                while j < ops.len() {
                    if let CirOp::RangeCheck { min, max } = &ops[j] {
                        if min == &run_min && max == &run_max {
                            run_len += 1;
                            j += 1;
                            continue;
                        }
                    }
                    break;
                }

                if run_len >= 2 {
                    result.push(CirOp::VecRangeCheck {
                        min: run_min,
                        max: run_max,
                        count: run_len as u32,
                    });
                    groups += 1;
                    i = j;
                    continue;
                }
            }
            result.push(ops[i].clone());
            i += 1;
        }

        (result, groups)
    }

    /// Run all optimization passes
    pub fn optimize(&self, nodes: &mut Vec<crate::cir::CirNode>) -> OptimizationReport {
        let original_count = nodes.len();
        let ops: Vec<CirOp> = nodes.iter().map(|n| n.op.clone()).collect();
        let (optimized_ops, vectorized_groups) = self.vectorize(&ops);

        // Rebuild nodes with optimized ops
        let mut result = Vec::new();
        let mut op_idx = 0;
        let mut node_idx = 0;

        while op_idx < optimized_ops.len() && node_idx < nodes.len() {
            let mut node = nodes[node_idx].clone();
            node.op = optimized_ops[op_idx].clone();

            // If this was a vectorized group, skip the merged nodes
            if let CirOp::VecRangeCheck { count, .. } = &optimized_ops[op_idx] {
                node.batch_size = Some(*count);
                node_idx += *count as usize;
            } else {
                node_idx += 1;
            }

            node.id = result.len() as u32;
            result.push(node);
            op_idx += 1;
        }

        // If there are remaining nodes, add them
        while node_idx < nodes.len() {
            let mut node = nodes[node_idx].clone();
            node.id = result.len() as u32;
            result.push(node);
            node_idx += 1;
        }

        let optimized_count = result.len();
        *nodes = result;

        OptimizationReport {
            original_ops: original_count,
            optimized_ops: optimized_count,
            vectorized_groups,
            bytes_saved: 0, // computed after codegen
        }
    }
}
