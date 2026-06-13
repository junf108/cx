use serde::{Deserialize, Serialize};

// ─── Snapshot ────────────────────────────────────────────────

/// Core type: a recorded change snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snapshot {
    /// Content hash (SHA256 of canonical JSON)
    pub sid: String,
    /// Parent snapshot IDs (first is the base commit on main)
    pub parents: Vec<String>,
    /// Snapshot kind
    pub kind: SnapshotKind,
    /// Change context
    pub context: Context,
    /// Semantic units list
    pub semantic_units: Vec<SemanticUnit>,
    /// Author
    pub author: String,
    /// ISO 8601 timestamp
    pub timestamp: String,
}

/// Snapshot kind
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SnapshotKind {
    Manual,
    AiSession,
    AutoSave,
}

// ─── Context ─────────────────────────────────────────────────

/// Change context: describes why a change was made
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Context {
    /// Original requirement that triggered this change
    pub prompt: String,
    /// Conversation summary (optional)
    pub conversation_summary: Option<String>,
    /// Turn index within the session
    pub turn_index: u32,
}

// ─── SemanticUnit ────────────────────────────────────────────

/// Semantic unit: smallest change unit grouped by intent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticUnit {
    /// Intent kind
    pub intent: IntentKind,
    /// Scope, e.g. "user_module", "login_page"
    pub scope: String,
    /// One-line description
    pub description: String,
    /// Risk level
    pub risk: RiskLevel,
    /// Affected files
    pub affected_files: Vec<String>,
}

/// Intent kind
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IntentKind {
    Refactor,
    Feature,
    Fix,
    Style,
    Docs,
    Dependency,
    Test,
    Chore,
}

impl std::fmt::Display for IntentKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IntentKind::Refactor => write!(f, "refactor"),
            IntentKind::Feature => write!(f, "feature"),
            IntentKind::Fix => write!(f, "fix"),
            IntentKind::Style => write!(f, "style"),
            IntentKind::Docs => write!(f, "docs"),
            IntentKind::Dependency => write!(f, "dependency"),
            IntentKind::Test => write!(f, "test"),
            IntentKind::Chore => write!(f, "chore"),
        }
    }
}

impl std::str::FromStr for IntentKind {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "refactor" => Ok(IntentKind::Refactor),
            "feature" => Ok(IntentKind::Feature),
            "fix" => Ok(IntentKind::Fix),
            "style" => Ok(IntentKind::Style),
            "docs" => Ok(IntentKind::Docs),
            "dependency" => Ok(IntentKind::Dependency),
            "test" => Ok(IntentKind::Test),
            "chore" => Ok(IntentKind::Chore),
            _ => Err(format!("unknown intent kind: {s}")),
        }
    }
}

/// Risk level
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
}

impl std::fmt::Display for RiskLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RiskLevel::Low => write!(f, "low"),
            RiskLevel::Medium => write!(f, "medium"),
            RiskLevel::High => write!(f, "high"),
        }
    }
}

impl std::fmt::Display for IntentSpec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} (scope: {}, risk: {})", self.kind, self.scope, self.risk)
    }
}

impl std::str::FromStr for RiskLevel {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "low" => Ok(RiskLevel::Low),
            "medium" => Ok(RiskLevel::Medium),
            "high" => Ok(RiskLevel::High),
            _ => Err(format!("unknown risk level: {s}")),
        }
    }
}

// ─── Session metadata ────────────────────────────────────────

/// Session status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SessionStatus {
    Active,
    Merged,
    Abandoned,
}

impl std::fmt::Display for SessionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SessionStatus::Active => write!(f, "active"),
            SessionStatus::Merged => write!(f, "merged"),
            SessionStatus::Abandoned => write!(f, "abandoned"),
        }
    }
}

/// Session metadata (.cx/sessions/<sid>/meta.json)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMeta {
    pub id: String,
    pub branch: String,
    pub prompt: String,
    pub conversation_summary: Option<String>,
    pub author: String,
    pub status: SessionStatus,
    pub created_at: String,
    pub base_branch: String,
    pub base_commit: String,
    /// Snapshot count (maintained by storage layer)
    #[serde(default)]
    pub snapshot_count: u32,
}


// ─── IntentSpec (parse --intent argument) ─────────────────────

/// Parsed intent specification
#[derive(Debug, Clone)]
pub struct IntentSpec {
    pub kind: IntentKind,
    pub scope: String,
    pub risk: RiskLevel,
}

/// Parse the "--intent" argument value
/// Format: type,scope=x[,risk=low]
/// First value without `=` is treated as the intent type
pub fn parse_intent_spec(spec: &str) -> Result<IntentSpec, String> {
    let mut kind: Option<IntentKind> = None;
    let mut scope = String::new();
    let mut risk = RiskLevel::Low;

    for part in spec.split(',') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        if let Some(eq_pos) = part.find('=') {
            let key = &part[..eq_pos];
            let val = &part[eq_pos + 1..];
            match key {
                "scope" => scope = val.to_string(),
                "risk" => risk = val.parse()?,
                _ => return Err(format!("unknown intent key: {key}")),
            }
        } else if kind.is_none() {
            kind = Some(part.parse()?);
        } else {
            return Err(format!("unexpected bare value: {part}"));
        }
    }

    let kind = kind.ok_or_else(|| "missing intent type (e.g. feature)".to_string())?;
    Ok(IntentSpec { kind, scope, risk })
}
