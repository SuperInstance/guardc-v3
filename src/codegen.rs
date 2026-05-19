use crate::lcir::{LcirNode, LcirOp};

/// FLUX-C v3 opcodes
pub const OP_RANGE_CHECK: u8 = 0x01;
pub const OP_RANGE_CHECK_F32: u8 = 0x02;
pub const OP_LESS_THAN: u8 = 0x03;
pub const OP_GREATER_THAN: u8 = 0x04;
pub const OP_AND_CHAIN: u8 = 0x05;
pub const OP_CLAMP: u8 = 0x06;
pub const OP_VEC_RANGE: u8 = 0x07;
pub const OP_NOP: u8 = 0x00;
pub const OP_HALT: u8 = 0xFF;

/// FLUX-C v3 bytecode header
pub const MAGIC: [u8; 4] = [b'F', b'L', b'X', b'3'];
pub const VERSION: u8 = 3;

pub fn codegen(nodes: &[LcirNode]) -> Vec<u8> {
    let mut bytecode = Vec::new();

    // Header
    bytecode.extend_from_slice(&MAGIC);
    bytecode.push(VERSION);
    bytecode.push(nodes.len() as u8);

    for node in nodes {
        emit_node(&mut bytecode, node);
    }

    bytecode.push(OP_HALT);
    bytecode
}

fn emit_node(buf: &mut Vec<u8>, node: &LcirNode) {
    match &node.op {
        LcirOp::RangeCheck { min_i16, max_i16 } => {
            buf.push(OP_RANGE_CHECK);
            buf.push(node.priority_byte);
            buf.extend_from_slice(&min_i16.to_le_bytes());
            buf.extend_from_slice(&max_i16.to_le_bytes());
            buf.push(node.temporal_type);
            if node.temporal_type > 0 {
                buf.extend_from_slice(&(node.temporal_value as u32).to_le_bytes());
            }
        }
        LcirOp::RangeCheckF32 { min_bits, max_bits } => {
            buf.push(OP_RANGE_CHECK_F32);
            buf.push(node.priority_byte);
            buf.extend_from_slice(&min_bits.to_le_bytes());
            buf.extend_from_slice(&max_bits.to_le_bytes());
            buf.push(node.temporal_type);
            if node.temporal_type > 0 {
                buf.extend_from_slice(&(node.temporal_value as u32).to_le_bytes());
            }
        }
        LcirOp::LessThan { value_bits } => {
            buf.push(OP_LESS_THAN);
            buf.push(node.priority_byte);
            buf.extend_from_slice(&value_bits.to_le_bytes());
        }
        LcirOp::GreaterThan { value_bits } => {
            buf.push(OP_GREATER_THAN);
            buf.push(node.priority_byte);
            buf.extend_from_slice(&value_bits.to_le_bytes());
        }
        LcirOp::AndChain { count } => {
            buf.push(OP_AND_CHAIN);
            buf.push(node.priority_byte);
            buf.push(*count);
        }
        LcirOp::Clamp { min_i16, max_i16 } => {
            buf.push(OP_CLAMP);
            buf.push(node.priority_byte);
            buf.extend_from_slice(&min_i16.to_le_bytes());
            buf.extend_from_slice(&max_i16.to_le_bytes());
        }
        LcirOp::VecRangeCheck { min_i16, max_i16, count } => {
            buf.push(OP_VEC_RANGE);
            buf.push(node.priority_byte);
            buf.extend_from_slice(&min_i16.to_le_bytes());
            buf.extend_from_slice(&max_i16.to_le_bytes());
            buf.push(*count);
        }
        LcirOp::Nop => {
            buf.push(OP_NOP);
        }
    }
}

/// Interpret FLUX-C bytecode for testing — returns true if value passes all constraints
pub fn interpret(bytecode: &[u8], values: &[f64]) -> Vec<bool> {
    let mut results = Vec::new();
    if bytecode.len() < 6 { return results; }
    if &bytecode[0..4] != &MAGIC { return results; }

    let num_constraints = bytecode[5] as usize;
    let mut pc: usize = 6;
    let mut val_idx = 0;

    for _ in 0..num_constraints {
        if pc >= bytecode.len() { break; }
        let op = bytecode[pc];
        pc += 1;

        if op == OP_HALT { break; }

        let val = values.get(val_idx).copied().unwrap_or(0.0);

        match op {
            OP_RANGE_CHECK => {
                let _priority = bytecode[pc]; pc += 1;
                let min = i16::from_le_bytes([bytecode[pc], bytecode[pc+1]]); pc += 2;
                let max = i16::from_le_bytes([bytecode[pc], bytecode[pc+1]]); pc += 2;
                let tt = bytecode[pc]; pc += 1;
                if tt > 0 { pc += 4; } // skip temporal value
                results.push(val >= min as f64 && val <= max as f64);
                val_idx += 1;
            }
            OP_RANGE_CHECK_F32 => {
                let _priority = bytecode[pc]; pc += 1;
                let min_bits = u32::from_le_bytes([bytecode[pc], bytecode[pc+1], bytecode[pc+2], bytecode[pc+3]]); pc += 4;
                let max_bits = u32::from_le_bytes([bytecode[pc], bytecode[pc+1], bytecode[pc+2], bytecode[pc+3]]); pc += 4;
                let tt = bytecode[pc]; pc += 1;
                if tt > 0 { pc += 4; }
                let min = f32::from_bits(min_bits) as f64;
                let max = f32::from_bits(max_bits) as f64;
                results.push(val >= min && val <= max);
                val_idx += 1;
            }
            OP_VEC_RANGE => {
                let _priority = bytecode[pc]; pc += 1;
                let min = i16::from_le_bytes([bytecode[pc], bytecode[pc+1]]); pc += 2;
                let max = i16::from_le_bytes([bytecode[pc], bytecode[pc+1]]); pc += 2;
                let count = bytecode[pc] as usize; pc += 1;
                for i in 0..count {
                    let v = values.get(val_idx + i).copied().unwrap_or(0.0);
                    results.push(v >= min as f64 && v <= max as f64);
                }
                val_idx += count;
            }
            OP_CLAMP => {
                let _priority = bytecode[pc]; pc += 1;
                let min = i16::from_le_bytes([bytecode[pc], bytecode[pc+1]]); pc += 2;
                let max = i16::from_le_bytes([bytecode[pc], bytecode[pc+1]]); pc += 2;
                results.push(val >= min as f64 && val <= max as f64);
                val_idx += 1;
            }
            OP_LESS_THAN => {
                let _priority = bytecode[pc]; pc += 1;
                let vbits = u32::from_le_bytes([bytecode[pc], bytecode[pc+1], bytecode[pc+2], bytecode[pc+3]]); pc += 4;
                let threshold = f32::from_bits(vbits) as f64;
                results.push(val < threshold);
                val_idx += 1;
            }
            OP_GREATER_THAN => {
                let _priority = bytecode[pc]; pc += 1;
                let vbits = u32::from_le_bytes([bytecode[pc], bytecode[pc+1], bytecode[pc+2], bytecode[pc+3]]); pc += 4;
                let threshold = f32::from_bits(vbits) as f64;
                results.push(val > threshold);
                val_idx += 1;
            }
            OP_AND_CHAIN => {
                pc += 2; // skip priority + count
                results.push(true);
            }
            _ => {
                // unknown op — skip
                results.push(false);
            }
        }
    }

    results
}
