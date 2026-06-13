use sha2::{Digest, Sha256};


use crate::context::{
    Context, IntentSpec, SemanticUnit, Snapshot, SnapshotKind,
};

/// Compute snapshot content hash (SHA256 of canonical JSON)
pub fn compute_sid(snapshot: &Snapshot) -> String {
    let canonical = serde_json::to_string(snapshot).unwrap_or_default();
    let mut hasher = Sha256::new();
    hasher.update(canonical.as_bytes());
    let result = hasher.finalize();
    hex_encode(&result)
}

fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn hex_encode_first_n(bytes: &[u8], n: usize) -> String {
    bytes.iter().take(n).map(|b| format!("{b:02x}")).collect()
}

/// Generate a session ID
/// Format: s_ + first 8 hex chars of SHA256(prompt + base_commit)
pub fn compute_session_id(prompt: &str, base_commit: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(prompt.as_bytes());
    hasher.update(base_commit.as_bytes());
    let result = hasher.finalize();
    format!("s_{}", hex_encode_first_n(&result, 4))
}

/// Generate a branch name
pub fn session_branch_name(sid: &str, prompt: &str) -> String {
    let slug: String = prompt
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '-' || *c == '_' || *c == ' ')
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<&str>>()
        .join("-")
        .to_lowercase()
        .chars()
        .take(30)
        .collect();
    format!("cx/{sid}-{slug}")
}

/// Create a new Snapshot
pub fn create_snapshot(
    parents: Vec<String>,
    kind: SnapshotKind,
    prompt: String,
    turn_index: u32,
    semantic_units: Vec<SemanticUnit>,
    author: String,
    timestamp: String,
) -> Snapshot {
    let context = Context {
        prompt,
        conversation_summary: None,
        turn_index,
    };

    let mut snapshot = Snapshot {
        sid: String::new(),
        parents,
        kind,
        context,
        semantic_units,
        author,
        timestamp,
    };
    snapshot.sid = compute_sid(&snapshot);
    snapshot
}

/// Build a SemanticUnit from intent spec and file list
pub fn build_semantic_unit(
    spec: &IntentSpec,
    description: String,
    affected_files: Vec<String>,
) -> SemanticUnit {
    SemanticUnit {
        intent: spec.kind.clone(),
        scope: spec.scope.clone(),
        description,
        risk: spec.risk.clone(),
        affected_files,
    }
}
