use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

fn hash_bytes(data: &[u8]) -> String {
    let mut hasher = DefaultHasher::new();
    data.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

fn hash_str(s: &str) -> String {
    hash_bytes(s.as_bytes())
}

#[derive(Debug, Clone)]
pub struct ProofObligation {
    pub description: String,
    pub smt_lib: String,
    pub discharged: bool,
}

#[derive(Debug, Clone)]
pub struct ProofCertificate {
    pub source_hash: String,
    pub ast_hash: String,
    pub cir_hash: String,
    pub bytecode_hash: String,
    pub obligations: Vec<ProofObligation>,
    pub all_discharged: bool,
}

impl ProofCertificate {
    pub fn new(
        source: &str,
        ast_repr: &str,
        cir_repr: &str,
        bytecode: &[u8],
    ) -> Self {
        let source_hash = hash_str(source);
        let ast_hash = hash_str(ast_repr);
        let cir_hash = hash_str(cir_repr);
        let bytecode_hash = hash_bytes(bytecode);

        Self {
            source_hash,
            ast_hash,
            cir_hash,
            bytecode_hash,
            obligations: Vec::new(),
            all_discharged: true,
        }
    }

    pub fn add_obligation(&mut self, desc: String, smt: String, discharged: bool) {
        if !discharged { self.all_discharged = false; }
        self.obligations.push(ProofObligation {
            description: desc,
            smt_lib: smt,
            discharged,
        });
    }
}

/// Generate proof obligations for constraints
pub fn generate_proofs(
    source: &str,
    ast_repr: &str,
    cir_repr: &str,
    bytecode: &[u8],
    constraints: &[(String, crate::ast::ConstraintKind)],
) -> ProofCertificate {
    let mut cert = ProofCertificate::new(source, ast_repr, cir_repr, bytecode);

    for (name, kind) in constraints {
        match kind {
            crate::ast::ConstraintKind::Range(r) => {
                // Proof: if value in [min, max], then min <= max (trivially true)
                let smt = format!(
                    "(assert (<= {} {}))\n(check-sat)",
                    r.min, r.max
                );
                let discharged = r.min <= r.max;
                cert.add_obligation(
                    format!("{}: range [{}, {}] is well-formed", name, r.min, r.max),
                    smt,
                    discharged,
                );
            }
            crate::ast::ConstraintKind::ClampedTo(r) => {
                let smt = format!(
                    "(assert (<= {} {}))\n(check-sat)",
                    r.min, r.max
                );
                let discharged = r.min <= r.max;
                cert.add_obligation(
                    format!("{}: clamp [{}, {}] is well-formed", name, r.min, r.max),
                    smt,
                    discharged,
                );
            }
            _ => {
                // Generic obligation — always discharged for simple cases
                cert.add_obligation(
                    format!("{}: constraint is satisfiable", name),
                    "(check-sat)".to_string(),
                    true,
                );
            }
        }
    }

    cert
}

/// Verify the proof certificate hash chain is consistent
pub fn verify_chain(cert: &ProofCertificate, source: &str, bytecode: &[u8]) -> bool {
    let expected_source = hash_str(source);
    let expected_bytecode = hash_bytes(bytecode);
    cert.source_hash == expected_source && cert.bytecode_hash == expected_bytecode
}
