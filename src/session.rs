use crate::context::{IntentSpec, SemanticUnit, SessionMeta, SessionStatus,
    SnapshotKind,};
use crate::git_api;
use crate::snapshot;
use crate::store::{Store, StoreError};

/// Session manager error
#[derive(Debug)]
pub enum SessionError {
    Store(StoreError),
    Git(git_api::GitError),
    NoActiveSession,
    SessionNotActive,
    AlreadyInSession,
    NoChanges,
    User(String),
}

impl std::fmt::Display for SessionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SessionError::Store(e) => write!(f, "{e}"),
            SessionError::Git(e) => write!(f, "{e}"),
            SessionError::NoActiveSession => write!(f, "no active session"),
            SessionError::SessionNotActive => {
                write!(f, "current session is not active")
            }
            SessionError::AlreadyInSession => {
                write!(f, "already in an active session")
            }
            SessionError::NoChanges => write!(f, "no changes to record"),
            SessionError::User(s) => write!(f, "{s}"),
        }
    }
}

impl std::error::Error for SessionError {}

impl From<StoreError> for SessionError {
    fn from(e: StoreError) -> Self {
        SessionError::Store(e)
    }
}

impl From<git_api::GitError> for SessionError {
    fn from(e: git_api::GitError) -> Self {
        SessionError::Git(e)
    }
}

pub type Result<T> = std::result::Result<T, SessionError>;

/// Session manager
pub struct SessionManager {
    store: Store,
}

/// Status output structure
#[derive(Debug)]
pub struct SessionStatusInfo {
    pub id: String,
    pub branch: String,
    pub status: String,
    pub prompt: String,
    pub snapshot_count: u32,
    pub semantic_units: Vec<SemanticUnit>,
}

/// Log entry
#[derive(Debug)]
pub struct LogEntry {
    pub snapshot_id: String,
    pub timestamp: String,
    pub description: String,
    pub intent: String,
    pub risk: String,
}

/// Review data
#[derive(Debug)]
pub struct ReviewData {
    pub base_branch: String,
    pub base_commit: String,
    pub session_id: String,
    pub groups: Vec<ReviewGroup>,
}

#[derive(Debug)]
pub struct ReviewGroup {
    pub intent: String,
    pub scope: String,
    pub files: Vec<(String, u32, u32)>, // (filename, added, removed)
}

impl SessionManager {
    /// Initialize the cx repository (cx init)
    pub fn init() -> Result<()> {
        // First verify we are in a git repository
        if !git_api::is_git_repo() {
            return Err(SessionError::Store(StoreError::NotAGitRepo));
        }
        Store::init()?;
        println!("✓ Initialized .cx/ metadata store");
        Ok(())
    }

    /// Open existing storage, return a SessionManager
    pub fn open() -> Result<SessionManager> {
        let store = Store::open()?;
        Ok(SessionManager { store })
    }

    /// Start a new session (cx start)
    pub fn start(&self, prompt: &str, author: &str) -> Result<()> {
        // Check if already in an active session (already on a session branch)
        if let Ok(Some(existing_sid)) = git_api::current_session_id() {
            if let Ok(meta) = self.get_store().read_session_meta(&existing_sid) {
                if meta.status == SessionStatus::Active {
                    return Err(SessionError::AlreadyInSession);
                }
            }
        }

        let base_commit = git_api::rev_parse_head()?;
        let base_branch = git_api::current_branch()?;

        // Generate session ID
        let sid = snapshot::compute_session_id(prompt, &base_commit);
        let branch = snapshot::session_branch_name(&sid, prompt);

        // Stash uncommitted changes
        let stashed = git_api::has_uncommitted_changes()?;
        if stashed {
            git_api::stash()?;
        }

        // Create and switch to a new branch
        git_api::checkout_new_branch(&branch)?;

        // Pop stash if we had uncommitted changes
        if stashed {
            git_api::stash_pop()?;
        }

        // Write session metadata
        let now = chrono::Local::now().to_rfc3339();
        let meta = SessionMeta {
            id: sid.clone(),
            branch,
            prompt: prompt.to_string(),
            conversation_summary: None,
            author: author.to_string(),
            status: SessionStatus::Active,
            created_at: now,
            base_branch,
            base_commit,
            snapshot_count: 0,
        };
        self.get_store().write_session_meta(&meta)?;

        println!("✓ Started session {sid} on branch {}", meta.branch);
        println!("  Prompt: {prompt}");
        Ok(())
    }

    /// Record changes as a snapshot (cx apply)
    pub fn apply(
        &self,
        message: &str,
        intent_spec: &IntentSpec,
        author: &str,
        _no_verify: bool,
    ) -> Result<()> {
        let sid = git_api::current_session_id()?
            .ok_or(SessionError::NoActiveSession)?;

        let meta = self.get_store().read_session_meta(&sid)?;
        if meta.status != SessionStatus::Active {
            return Err(SessionError::SessionNotActive);
        }

        if !git_api::has_uncommitted_changes()? {
            return Err(SessionError::NoChanges);
        }

        // Stage all changes
        git_api::add_all()?;

        // Get the current turn index (snapshot number)
        let turn_index = {
            let snapshots = self.get_store().list_snapshots(&sid)?;
            snapshots.len() as u32 + 1
        };

        // Get list of changed files
        let affected_files: Vec<String> = git_api::diff_cached_names()?
            .into_iter()
            .filter(|f| !f.starts_with(".cx/"))
            .collect();

        // Build the snapshot
        let now = chrono::Local::now().to_rfc3339();
        let semantic_unit = snapshot::build_semantic_unit(
            intent_spec,
            message.to_string(),
            affected_files,
        );
        let parents: Vec<String> = {
            let snapshots = self.get_store().list_snapshots(&sid)?;
            if snapshots.is_empty() {
                vec![meta.base_commit.clone()]
            } else {
                vec![turn_index.saturating_sub(1).to_string()]
            }
        };
        let snapshot = snapshot::create_snapshot(
            parents,
            SnapshotKind::AiSession,
            meta.prompt.clone(),
            turn_index,
            vec![semantic_unit],
            author.to_string(),
            now,
        );

        // Save the snapshot (filename = turn_index.json) to .cx/
        self.get_store().save_snapshot(&sid, turn_index, &snapshot)?;

        // Re-stage everything (including .cx/ files), then commit together
        git_api::add_all()?;
        let commit_hash = git_api::commit(&format!("cx: {message}"))?;

        println!("✓ Applied snapshot #{}", turn_index);
        println!("  Commit: {}", commit_hash.chars().take(12).collect::<String>());
        println!("  Intent: {intent_spec}");
        Ok(())
    }    /// Show current session status (cx status)
    pub fn status(&self) -> Result<SessionStatusInfo> {
        let sid = git_api::current_session_id()?
            .ok_or(SessionError::NoActiveSession)?;

        let meta = self.get_store().read_session_meta(&sid)?;

        // Collect all semantic units
        let snapshot_ids = self.get_store().list_snapshots(&sid)?;
        let mut all_units = Vec::new();
        for sid_hash in &snapshot_ids {
            if let Ok(snap) = self.get_store().load_snapshot(&sid, sid_hash.parse::<u32>().unwrap_or(0)) {
                all_units.extend(snap.semantic_units);
            }
        }

        Ok(SessionStatusInfo {
            id: meta.id,
            branch: meta.branch,
            status: meta.status.to_string(),
            prompt: meta.prompt,
            snapshot_count: meta.snapshot_count,
            semantic_units: all_units,
        })
    }

    /// End the current session (cx end)
    pub fn end(&self, merge: bool) -> Result<()> {
        let sid = git_api::current_session_id()?
            .ok_or(SessionError::NoActiveSession)?;

        let meta = self.get_store().read_session_meta(&sid)?;
        if meta.status != SessionStatus::Active {
            return Err(SessionError::SessionNotActive);
        }

        let branch = meta.branch.clone();
        let base_branch = meta.base_branch.clone();

        if merge {
            // Switch back to the base branch
            git_api::checkout(&base_branch)?;
            // Merge
            println!("  Merging {branch} into {base_branch}...");
            git_api::merge(&branch)?;
            // Delete the session branch
            git_api::delete_branch(&branch)?;

            // Update status
            let mut meta = self.get_store().read_session_meta(&sid)?;
            meta.status = SessionStatus::Merged;
            self.get_store().write_session_meta(&meta)?;

            println!("✓ Session {sid} merged into {base_branch}");
        } else {
            // Abandon: update meta first (still on session branch), commit, then switch back
            let mut meta = self.get_store().read_session_meta(&sid)?;
            meta.status = SessionStatus::Abandoned;
            self.get_store().write_session_meta(&meta)?;
            // Commit the meta change to the session branch before switching
            git_api::add_all()?;
            let _ = git_api::commit("cx: end session (abandon)");

            git_api::checkout(&base_branch)?;
            git_api::delete_branch(&branch)?;

            println!("✓ Session {sid} abandoned");
        }


        Ok(())
    }

    /// View snapshot history (cx log)
    pub fn log(&self, session_id: Option<&str>) -> Result<Vec<LogEntry>> {
        let entries = if let Some(sid) = session_id {
            // View a specific session
            self.get_store().read_session_meta(&sid)?;
            let snapshots = self.get_store().list_snapshots(&sid)?;
            let mut entries = Vec::new();
            for sid_hash in &snapshots {
                if let Ok(snap) = self.get_store().load_snapshot(&sid, sid_hash.parse::<u32>().unwrap_or(0)) {
                    if let Some(unit) = snap.semantic_units.first() {
                        entries.push(LogEntry {
                            snapshot_id: snap.sid.chars().take(12).collect(),
                            timestamp: snap.timestamp.clone(),
                            description: unit.description.clone(),
                            intent: unit.intent.to_string(),
                            risk: unit.risk.to_string(),
                        });
                    } else {
                        entries.push(LogEntry {
                            snapshot_id: snap.sid.chars().take(12).collect(),
                            timestamp: snap.timestamp.clone(),
                            description: String::new(),
                            intent: String::new(),
                            risk: String::new(),
                        });
                    }
                }
            }
            entries
        } else {
            // Scan .cx/sessions/ directory to list all sessions
            let sessions_dir = self.get_store().cx_sessions_dir();
            if !sessions_dir.exists() {
                return Ok(Vec::new());
            }
            let mut entries: Vec<_> = Vec::new();
            if let Ok(dir) = std::fs::read_dir(&sessions_dir) {
                for entry in dir.flatten() {
                    let sid = entry.file_name().to_string_lossy().to_string();
                    if let Ok(meta) = self.get_store().read_session_meta(&sid) {
                        entries.push(meta);
                    }
                }
            }
            entries.sort_by(|a, b| a.created_at.cmp(&b.created_at));
            for meta in entries.iter().rev().take(10) {
                println!(
                    "{}  {}  {}  {} snapshots",
                    meta.id,
                    meta.status,
                    meta.prompt.chars().take(50).collect::<String>(),
                    meta.snapshot_count,
                );
            }
            return Ok(Vec::new());
        };

        // Sort by timestamp
        let mut entries = entries;
        entries.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
        Ok(entries)
    }

    /// Review changes grouped by intent (cx review)
    pub fn review(&self, snapshot_id: Option<&str>) -> Result<ReviewData> {
        let sid = git_api::current_session_id()?
            .ok_or(SessionError::NoActiveSession)?;
        let meta = self.get_store().read_session_meta(&sid)?;

        let snapshot_ids = self.get_store().list_snapshots(&sid)?;

        // If a specific snapshot_id was provided
        let target_ids: Vec<&str> = if let Some(sid_hash) = snapshot_id {
            vec![sid_hash]
        } else {
            snapshot_ids.iter().map(|s| s.as_str()).collect()
        };

        let mut groups: Vec<ReviewGroup> = Vec::new();
        for sid_hash in &target_ids {
            if let Ok(snap) = self.get_store().load_snapshot(&sid, sid_hash.parse::<u32>().unwrap_or(0)) {
                for unit in &snap.semantic_units {
                    // Get detailed diff stats for the file
                    let mut files: Vec<(String, u32, u32)> = Vec::new();
                    for f in &unit.affected_files {
                        // Diff using base_commit..HEAD or the current commit
                        if let Ok(stat) = git_api::diff_stat(&meta.base_commit, "HEAD") {
                            if stat.contains(f) {
                                files.push((f.clone(), 0, 0)); // Simplified for now
                            } else {
                                files.push((f.clone(), 0, 0));
                            }
                        }
                    }

                    groups.push(ReviewGroup {
                        intent: unit.intent.to_string(),
                        scope: unit.scope.clone(),
                        files,
                    });
                }
            }
        }

        Ok(ReviewData {
            base_branch: meta.base_branch,
            base_commit: meta.base_commit,
            session_id: meta.id,
            groups,
        })
    }

    fn get_store(&self) -> &Store {
        &self.store
    }
}
