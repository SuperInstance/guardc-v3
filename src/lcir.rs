use crate::cir::{CirNode, CirOp};

#[derive(Debug, Clone)]
pub struct LcirNode {
    pub id: u32,
    pub name: String,
    pub op: LcirOp,
    pub priority_byte: u8,
    pub temporal_type: u8, // 0=none, 1=per_second, 2=for_seconds
    pub temporal_value: u64,
}

#[derive(Debug, Clone)]
pub enum LcirOp {
    RangeCheck { min_i16: i16, max_i16: i16 },       // quantized to i16
    RangeCheckF32 { min_bits: u32, max_bits: u32 },   // f32 bits
    LessThan { value_bits: u32 },
    GreaterThan { value_bits: u32 },
    AndChain { count: u8 },
    Clamp { min_i16: i16, max_i16: i16 },
    VecRangeCheck { min_i16: i16, max_i16: i16, count: u8 },
    Nop,
}

fn quantize_i16(v: f64) -> i16 {
    (v.round().clamp(-32768.0, 32767.0)) as i16
}

fn f32_bits(v: f64) -> u32 {
    (v as f32).to_bits()
}

fn priority_byte(p: &crate::ast::Priority) -> u8 {
    match p {
        crate::ast::Priority::Low => 0,
        crate::ast::Priority::Medium => 1,
        crate::ast::Priority::High => 2,
        crate::ast::Priority::Critical => 3,
    }
}

fn temporal_type(t: &crate::ast::Temporal) -> (u8, u64) {
    match t {
        crate::ast::Temporal::None => (0, 0),
        crate::ast::Temporal::PerSecond => (1, 1),
        crate::ast::Temporal::PerMinute => (1, 60),
        crate::ast::Temporal::ForSeconds(s) => (2, *s),
        crate::ast::Temporal::ForMinutes(m) => (2, m * 60),
    }
}

pub fn lower_cir_to_lcir(nodes: &[CirNode]) -> Vec<LcirNode> {
    nodes.iter().map(|n| {
        let (tt, tv) = temporal_type(&n.temporal);
        let op = match &n.op {
            CirOp::RangeCheck { min, max } => {
                if *min >= -32768.0 && *max <= 32767.0 {
                    LcirOp::RangeCheck { min_i16: quantize_i16(*min), max_i16: quantize_i16(*max) }
                } else {
                    LcirOp::RangeCheckF32 { min_bits: f32_bits(*min), max_bits: f32_bits(*max) }
                }
            }
            CirOp::LessThan { value } => LcirOp::LessThan { value_bits: f32_bits(*value) },
            CirOp::GreaterThan { value } => LcirOp::GreaterThan { value_bits: f32_bits(*value) },
            CirOp::And(ops) => {
                // Flatten: the AndChain will be followed by 'count' child ops
                LcirOp::AndChain { count: ops.len() as u8 }
            }
            CirOp::Clamp { min, max } => LcirOp::Clamp { min_i16: quantize_i16(*min), max_i16: quantize_i16(*max) },
            CirOp::VecRangeCheck { min, max, count } => {
                LcirOp::VecRangeCheck { min_i16: quantize_i16(*min), max_i16: quantize_i16(*max), count: *count as u8 }
            }
        };

        LcirNode {
            id: n.id,
            name: n.name.clone(),
            op,
            priority_byte: priority_byte(&n.priority),
            temporal_type: tt,
            temporal_value: tv,
        }
    }).collect()
}
