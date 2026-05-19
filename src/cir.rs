use crate::ast::*;
use crate::typecheck::TypedConstraint;

#[derive(Debug, Clone)]
pub struct CirNode {
    pub id: u32,
    pub name: String,
    pub op: CirOp,
    pub priority: Priority,
    pub unit: Unit,
    pub temporal: Temporal,
    pub batch_size: Option<u32>,
}

#[derive(Debug, Clone)]
pub enum CirOp {
    RangeCheck { min: f64, max: f64 },
    LessThan { value: f64 },
    GreaterThan { value: f64 },
    And(Vec<CirOp>),
    Clamp { min: f64, max: f64 },
    VecRangeCheck { min: f64, max: f64, count: u32 },
}

pub fn lower_to_cir(constraints: &[TypedConstraint]) -> Vec<CirNode> {
    let mut nodes = Vec::new();
    for (i, c) in constraints.iter().enumerate() {
        let op = if let Some(count) = c.batch_size {
            if let ConstraintKind::Range(ref r) = c.kind {
                CirOp::VecRangeCheck { min: r.min, max: r.max, count }
            } else {
                CirOp::RangeCheck { min: 0.0, max: 0.0 }
            }
        } else {
            match &c.kind {
                ConstraintKind::Range(r) => CirOp::RangeCheck { min: r.min, max: r.max },
                ConstraintKind::LessThan(v) => CirOp::LessThan { value: *v },
                ConstraintKind::GreaterThan(v) => CirOp::GreaterThan { value: *v },
                ConstraintKind::And(conds) => {
                    let ops: Vec<CirOp> = conds.iter().map(|c| match c {
                        ConstraintKind::LessThan(v) => CirOp::LessThan { value: *v },
                        ConstraintKind::GreaterThan(v) => CirOp::GreaterThan { value: *v },
                        ConstraintKind::Range(r) => CirOp::RangeCheck { min: r.min, max: r.max },
                        _ => CirOp::RangeCheck { min: 0.0, max: 0.0 },
                    }).collect();
                    CirOp::And(ops)
                }
                ConstraintKind::ClampedTo(r) => CirOp::Clamp { min: r.min, max: r.max },
                _ => CirOp::RangeCheck { min: 0.0, max: 0.0 },
            }
        };

        nodes.push(CirNode {
            id: i as u32,
            name: c.name.clone(),
            op,
            priority: c.priority.clone(),
            unit: c.unit.clone(),
            temporal: c.temporal.clone(),
            batch_size: c.batch_size,
        });
    }
    nodes
}
