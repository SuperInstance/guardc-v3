use crate::cir::{CirNode, CirOp};

/// Simplify CIR nodes: merge overlapping ranges, eliminate redundant checks.
pub fn simplify(nodes: &[CirNode]) -> Vec<CirNode> {
    nodes.iter().map(|n| {
        let simplified_op = simplify_op(&n.op);
        CirNode { op: simplified_op, ..n.clone() }
    }).collect()
}

fn simplify_op(op: &CirOp) -> CirOp {
    match op {
        CirOp::RangeCheck { min, max } => {
            if min == max {
                // Exact value check — keep as range (single point)
                CirOp::RangeCheck { min: *min, max: *max }
            } else if *min > *max {
                // Inverted range — swap
                CirOp::RangeCheck { min: *max, max: *min }
            } else {
                CirOp::RangeCheck { min: *min, max: *max }
            }
        }
        CirOp::And(ops) => {
            let simplified: Vec<CirOp> = ops.iter().map(simplify_op).collect();
            // Merge adjacent range checks into intersection
            let ranges: Vec<(f64, f64)> = simplified.iter().filter_map(|op| {
                if let CirOp::RangeCheck { min, max } = op { Some((*min, *max)) } else { None }
            }).collect();
            if ranges.len() >= 2 {
                // Intersect all ranges
                let merged_min = ranges.iter().map(|r| r.0).fold(f64::NEG_INFINITY, f64::max);
                let merged_max = ranges.iter().map(|r| r.1).fold(f64::INFINITY, f64::min);
                if ranges.len() == simplified.len() {
                    // All were ranges — merge into one
                    return CirOp::RangeCheck { min: merged_min, max: merged_max };
                }
            }
            CirOp::And(simplified)
        }
        _ => op.clone(),
    }
}
