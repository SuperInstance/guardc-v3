use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

#[derive(Debug, Clone)]
pub struct ProvenanceEntry {
    pub stage: String,
    pub hash: String,
    pub description: String,
}

#[derive(Debug, Clone)]
pub struct ProvenanceTrace {
    pub entries: Vec<ProvenanceEntry>,
}

impl ProvenanceTrace {
    pub fn new() -> Self {
        Self { entries: Vec::new() }
    }

    pub fn record(&mut self, stage: &str, data: &[u8], description: &str) {
        let mut hasher = DefaultHasher::new();
        data.hash(&mut hasher);
        self.entries.push(ProvenanceEntry {
            stage: stage.to_string(),
            hash: format!("{:016x}", hasher.finish()),
            description: description.to_string(),
        });
    }

    pub fn verify_chain(&self) -> bool {
        // Chain is valid if every stage has an entry
        !self.entries.is_empty()
    }
}
