pub mod ast;
pub mod parser;
pub mod error;
pub mod typecheck;
pub mod cir;
pub mod lcir;
pub mod simplify;
pub mod codegen;
pub mod optimizer;
pub mod proof;
pub mod provenance;
pub mod preset;

use ast::*;
use error::Result;
use provenance::ProvenanceTrace;

#[derive(Debug, Clone)]
pub struct ConstraintDef {
    pub name: String,
    pub kind: ConstraintKind,
}

#[derive(Debug)]
pub struct ModuleMetadata {
    pub source_len: usize,
    pub num_constraints: usize,
    pub compile_time_us: u64,
}

#[derive(Debug)]
pub struct CompiledModule {
    pub bytecode: Vec<u8>,
    pub constraints: Vec<ConstraintDef>,
    pub proof: proof::ProofCertificate,
    pub provenance: ProvenanceTrace,
    pub metadata: ModuleMetadata,
}

/// Full compilation pipeline: source → bytecode + proof + provenance
pub fn compile(source: &str) -> Result<CompiledModule> {
    let start = std::time::Instant::now();
    let mut prov = ProvenanceTrace::new();
    prov.record("source", source.as_bytes(), "input source");

    // Parse
    let program = parser::Parser::new(source)?.parse()?;
    let ast_repr = format!("{:?}", program);
    prov.record("ast", ast_repr.as_bytes(), "parsed AST");

    // Type check
    let typed = typecheck::typecheck(&program)?;

    // Lower to CIR
    let mut cir_nodes = cir::lower_to_cir(&typed);
    let cir_repr = format!("{:?}", cir_nodes);
    prov.record("cir", cir_repr.as_bytes(), "constraint IR");

    // Simplify
    cir_nodes = simplify::simplify(&cir_nodes);
    prov.record("simplify", format!("{:?}", cir_nodes).as_bytes(), "simplified CIR");

    // Optimize
    let opt = optimizer::Optimizer::default();
    opt.optimize(&mut cir_nodes);
    prov.record("optimize", format!("{:?}", cir_nodes).as_bytes(), "optimized CIR");

    // Lower to LCIR
    let lcir_nodes = lcir::lower_cir_to_lcir(&cir_nodes);
    prov.record("lcir", format!("{:?}", lcir_nodes).as_bytes(), "lowered CIR");

    // Codegen
    let bytecode = codegen::codegen(&lcir_nodes);
    prov.record("bytecode", &bytecode, "FLUX-C v3 bytecode");

    // Proof
    let constraint_pairs: Vec<(String, ConstraintKind)> = typed.iter()
        .map(|t| (t.name.clone(), t.kind.clone()))
        .collect();

    let proof = proof::generate_proofs(
        source,
        &ast_repr,
        &cir_repr,
        &bytecode,
        &constraint_pairs,
    );

    let constraints: Vec<ConstraintDef> = typed.iter()
        .map(|t| ConstraintDef { name: t.name.clone(), kind: t.kind.clone() })
        .collect();

    let elapsed = start.elapsed();

    let num_constraints = constraints.len();

    Ok(CompiledModule {
        bytecode,
        constraints,
        proof,
        provenance: prov,
        metadata: ModuleMetadata {
            source_len: source.len(),
            num_constraints,
            compile_time_us: elapsed.as_micros() as u64,
        },
    })
}

/// Compile a preset directly (without parsing)
pub fn compile_preset(preset: &PresetDecl) -> Result<CompiledModule> {
    let mut guards = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for guard in &preset.guards {
        typecheck::check_guard(guard, &mut seen, &mut guards)?;
    }
    let typed = guards;
    let source = format!("PRESET {}", preset.name);
    let ast_repr = format!("{:?}", preset);

    let mut cir_nodes = cir::lower_to_cir(&typed);
    cir_nodes = simplify::simplify(&cir_nodes);
    let opt = optimizer::Optimizer::default();
    opt.optimize(&mut cir_nodes);
    let lcir_nodes = lcir::lower_cir_to_lcir(&cir_nodes);
    let bytecode = codegen::codegen(&lcir_nodes);

    let cir_repr = format!("{:?}", cir_nodes);
    let constraint_pairs: Vec<(String, ConstraintKind)> = typed.iter()
        .map(|t| (t.name.clone(), t.kind.clone()))
        .collect();

    let proof = proof::generate_proofs(&source, &ast_repr, &cir_repr, &bytecode, &constraint_pairs);

    let mut prov = ProvenanceTrace::new();
    prov.record("preset", source.as_bytes(), "preset source");
    prov.record("bytecode", &bytecode, "FLUX-C v3 bytecode");

    let constraints: Vec<ConstraintDef> = typed.iter()
        .map(|t| ConstraintDef { name: t.name.clone(), kind: t.kind.clone() })
        .collect();

    let num_constraints = constraints.len();

    Ok(CompiledModule {
        bytecode,
        constraints,
        proof,
        provenance: prov,
        metadata: ModuleMetadata {
            source_len: source.len(),
            num_constraints,
            compile_time_us: 0,
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_range() {
        let source = "GUARD battery_temp in [15, 55]";
        let module = compile(source).unwrap();
        assert_eq!(module.constraints.len(), 1);
        assert_eq!(module.constraints[0].name, "battery_temp");
        assert!(module.bytecode.len() > 6); // header + at least one op
        assert!(module.proof.all_discharged);
    }

    #[test]
    fn test_priority_high() {
        let source = "GUARD cabin_pressure in [75, 101] with priority HIGH";
        let module = compile(source).unwrap();
        assert_eq!(module.constraints[0].name, "cabin_pressure");
    }

    #[test]
    fn test_temporal_constraint() {
        let source = "GUARD temp_rate in [-5, 5] per second for 10 seconds";
        let module = compile(source).unwrap();
        assert_eq!(module.constraints[0].name, "temp_rate");
    }

    #[test]
    fn test_batch_constraint() {
        let source = "BATCH sensor_array in [-40, 85]";
        let module = compile(source).unwrap();
        assert_eq!(module.constraints[0].name, "sensor_array");
    }

    #[test]
    fn test_preset_aviation() {
        let source = r#"
PRESET aviation:
    GUARD cabin_temp_C in [-55, 70] with priority HIGH
    GUARD cabin_pressure_kPa in [75, 101] with priority CRITICAL
    GUARD fuel_flow_pct in [0, 100]
    GUARD hydraulic_pct in [60, 100]
"#;
        let module = compile(source).unwrap();
        assert_eq!(module.constraints.len(), 4);
        assert!(module.bytecode.len() > 10);
        assert!(module.proof.all_discharged);
    }

    #[test]
    fn test_proof_chain() {
        let source = "GUARD temp in [0, 100]";
        let module = compile(source).unwrap();
        assert!(proof::verify_chain(&module.proof, source, &module.bytecode));
    }

    #[test]
    fn test_provenance_trace() {
        let source = "GUARD x in [0, 10]";
        let module = compile(source).unwrap();
        assert!(module.provenance.verify_chain());
        assert!(module.provenance.entries.len() >= 5); // source, ast, cir, simplify, bytecode
    }

    #[test]
    fn test_all_10_presets_compile() {
        let presets = preset::get_all_presets();
        assert_eq!(presets.len(), 10);

        for preset in &presets {
            let module = compile_preset(preset).unwrap();
            assert!(module.bytecode.len() > 6, "preset {} produced no bytecode", preset.name);
            assert!(module.proof.all_discharged, "preset {} has unproven obligations", preset.name);
        }
    }

    #[test]
    fn test_bytecode_size_per_preset() {
        let presets = preset::get_all_presets();
        for preset in &presets {
            let module = compile_preset(preset).unwrap();
            assert!(
                module.bytecode.len() < 256,
                "preset {} bytecode too large: {} bytes",
                preset.name,
                module.bytecode.len()
            );
        }
    }

    #[test]
    fn test_interpret_range_pass() {
        let source = "GUARD temp in [0, 100]";
        let module = compile(source).unwrap();
        let results = codegen::interpret(&module.bytecode, &[50.0]);
        assert_eq!(results, vec![true]);
    }

    #[test]
    fn test_interpret_range_fail() {
        let source = "GUARD temp in [0, 100]";
        let module = compile(source).unwrap();
        let results = codegen::interpret(&module.bytecode, &[150.0]);
        assert_eq!(results, vec![false]);
    }

    #[test]
    fn test_interpret_boundary() {
        let source = "GUARD temp in [0, 100]";
        let module = compile(source).unwrap();
        let results = codegen::interpret(&module.bytecode, &[0.0, 100.0, -1.0, 101.0]);
        // Each constraint uses one value, but we have only one constraint
        assert_eq!(results.len(), 1);
        assert!(results[0]); // value 0.0 is in [0, 100]
    }

    #[test]
    fn test_multiple_constraints() {
        let source = r#"
GUARD temp in [15, 55]
GUARD pressure in [75, 101]
GUARD voltage in [200, 260]
"#;
        let module = compile(source).unwrap();
        assert_eq!(module.constraints.len(), 3);
        assert_eq!(module.constraints[0].name, "temp");
        assert_eq!(module.constraints[1].name, "pressure");
        assert_eq!(module.constraints[2].name, "voltage");
    }

    #[test]
    fn test_duplicate_name_error() {
        let source = r#"
GUARD temp in [0, 100]
GUARD temp in [10, 50]
"#;
        let result = compile(source);
        assert!(result.is_err());
    }

    #[test]
    fn test_inverted_range_error() {
        let source = "GUARD temp in [100, 0]";
        let result = compile(source);
        assert!(result.is_err());
    }

    #[test]
    fn test_handler_log() {
        let source = "HANDLE violations WITH log";
        let module = compile(source).unwrap();
        // No constraints, but should parse without error
        assert_eq!(module.constraints.len(), 0);
    }

    #[test]
    fn test_handler_broadcast() {
        let source = "HANDLE violations WITH broadcast";
        let module = compile(source).unwrap();
        assert_eq!(module.constraints.len(), 0);
    }

    #[test]
    fn test_compilation_speed() {
        let source = "GUARD temp in [0, 100]";
        let start = std::time::Instant::now();
        for _ in 0..1000 {
            let _ = compile(source).unwrap();
        }
        let elapsed = start.elapsed();
        let per_constraint = elapsed.as_micros() / 1000;
        assert!(
            per_constraint < 1000,
            "compilation too slow: {}µs per constraint",
            per_constraint
        );
    }

    #[test]
    fn test_preset_golden_vectors() {
        // Aviation: cabin_temp in [-55, 70]
        let preset = &preset::get_all_presets()[0];
        assert_eq!(preset.name, "aviation");

        let module = compile_preset(preset).unwrap();

        // cabin_temp_C: [-55, 70]
        let results = codegen::interpret(&module.bytecode, &[-55.0]);
        assert!(results[0], "cabin_temp lower bound should pass");

        let results = codegen::interpret(&module.bytecode, &[70.0]);
        assert!(results[0], "cabin_temp upper bound should pass");

        let results = codegen::interpret(&module.bytecode, &[-56.0]);
        assert!(!results[0], "cabin_temp below lower should fail");
    }

    #[test]
    fn test_simplify_merges_ranges() {
        let source = "GUARD x in [0, 100]";
        let module = compile(source).unwrap();
        // Simplification should not break a single range
        assert_eq!(module.constraints.len(), 1);
    }

    #[test]
    fn test_magic_header() {
        let source = "GUARD x in [0, 10]";
        let module = compile(source).unwrap();
        assert_eq!(&module.bytecode[0..4], b"FLX3");
        assert_eq!(module.bytecode[4], 3); // version
    }

    #[test]
    fn test_full_pipeline_with_comments() {
        let source = r#"
# Temperature monitoring
GUARD battery_temp in [15, 55]

# Pressure monitoring
GUARD cabin_pressure in [75, 101] with priority HIGH
"#;
        let module = compile(source).unwrap();
        assert_eq!(module.constraints.len(), 2);
        assert!(module.proof.all_discharged);
        assert!(module.provenance.verify_chain());
    }
}
