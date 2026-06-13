use std::fs;
use std::path::{Path, PathBuf};

use crate::context::{SessionMeta, Snapshot};

/// Storage layer error
#[derive(Debug)]
pub enum StoreError {
    Io(std::io::Error),
    Json(serde_json::Error),
    NotAGitRepo,
    AlreadyInitialized,
    NotInitialized,
    SessionNotFound(String),
    SnapshotNotFound(String),
}

impl std::fmt::Display for StoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StoreError::Io(e) => write!(f, "I/O error: {e}"),
            StoreError::Json(e) => write!(f, "JSON error: {e}"),
            StoreError::NotAGitRepo => write!(f, "not a git repository"),
            StoreError::AlreadyInitialized => write!(f, ".cx/ already initialized"),
            StoreError::NotInitialized => write!(f, ".cx/ not initialized (run `cx init` first)"),
            StoreError::SessionNotFound(s) => write!(f, "session not found: {s}"),
            StoreError::SnapshotNotFound(s) => write!(f, "snapshot not found: {s}"),
        }
    }
}

impl std::error::Error for StoreError {}

impl From<std::io::Error> for StoreError {
    fn from(e: std::io::Error) -> Self {
        StoreError::Io(e)
    }
}

impl From<serde_json::Error> for StoreError {
    fn from(e: serde_json::Error) -> Self {
        StoreError::Json(e)
    }
}

pub type Result<T> = std::result::Result<T, StoreError>;

/// .cx/ storage manager
pub struct Store {
    #[allow(dead_code)]
    root: PathBuf,
    cx_dir: PathBuf,
}

impl Store {
    /// Find git repository root
    pub fn find_git_root() -> Result<PathBuf> {
        let output = std::process::Command::new("git")
            .args(["rev-parse", "--show-toplevel"])
            .output()
            .map_err(|_| StoreError::NotAGitRepo)?;
        if !output.status.success() {
            return Err(StoreError::NotAGitRepo);
        }
        let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
        Ok(PathBuf::from(path))
    }

    /// Initialize .cx/ at the git repository root
    pub fn init() -> Result<Store> {
        let root = Self::find_git_root()?;
        let cx_dir = root.join(".cx");
        if cx_dir.exists() {
            return Err(StoreError::AlreadyInitialized);
        }
        fs::create_dir_all(cx_dir.join("sessions"))?;
        fs::write(cx_dir.join("sessions/.gitkeep"), "")?;
        let store = Store { root, cx_dir };
        // Commit .cx/ to the current branch so it persists across checkouts
        let _ = std::process::Command::new("git")
            .args(["add", ".cx/"])
            .output();
        let _ = std::process::Command::new("git")
            .args(["commit", "-m", "cx: init"])
            .output();
        Ok(store)
    }

    /// Open an existing .cx/ directory
    pub fn open() -> Result<Store> {
        let root = Self::find_git_root()?;
        let cx_dir = root.join(".cx");
        if !cx_dir.exists() {
            return Err(StoreError::NotInitialized);
        }
        Ok(Store { root, cx_dir })
    }

    /// Get the git repository root
    #[allow(dead_code)]
pub fn git_root(&self) -> &Path {
        &self.root
    }

    /// Return the .cx/sessions/ directory path
    pub fn cx_sessions_dir(&self) -> PathBuf {
        self.cx_dir.join("sessions")
    }

    // ─── Session Metadata ─────────────────────────────────────

    fn session_dir(&self, sid: &str) -> PathBuf {
        self.cx_dir.join("sessions").join(sid)
    }

    fn session_meta_path(&self, sid: &str) -> PathBuf {
        self.session_dir(sid).join("meta.json")
    }

    pub fn read_session_meta(&self, sid: &str) -> Result<SessionMeta> {
        let path = self.session_meta_path(sid);
        if !path.exists() {
            return Err(StoreError::SessionNotFound(sid.to_string()));
        }
        let data = fs::read_to_string(&path)?;
        Ok(serde_json::from_str(&data)?)
    }

    pub fn write_session_meta(&self, meta: &SessionMeta) -> Result<()> {
        let dir = self.session_dir(&meta.id);
        fs::create_dir_all(&dir)?;
        let path = dir.join("meta.json");
        let data = serde_json::to_string_pretty(meta)?;
        fs::write(&path, data)?;
        Ok(())
    }

    // ─── Snapshots ───────────────────────────────────────────

    fn snapshots_dir(&self, sid: &str) -> PathBuf {
        self.session_dir(sid).join("snapshots")
    }

    pub fn save_snapshot(&self, sid: &str, turn_index: u32, snapshot: &Snapshot) -> Result<()> {
        let dir = self.snapshots_dir(sid);
        fs::create_dir_all(&dir)?;
        let path = dir.join(format!("{turn_index}.json"));
        let data = serde_json::to_string_pretty(snapshot)?;
        fs::write(&path, data)?;
        // Update snapshot_count in session metadata
        let mut meta = self.read_session_meta(sid)?;
        meta.snapshot_count = std::cmp::max(meta.snapshot_count, turn_index);
        self.write_session_meta(&meta)?;
        Ok(())
    }

    pub fn load_snapshot(&self, sid: &str, turn_index: u32) -> Result<Snapshot> {
        let dir = self.snapshots_dir(sid);
        let path = dir.join(format!("{turn_index}.json"));
        if !path.exists() {
            return Err(StoreError::SnapshotNotFound(turn_index.to_string()));
        }
        let data = fs::read_to_string(&path)?;
        Ok(serde_json::from_str(&data)?)
    }

    pub fn list_snapshots(&self, sid: &str) -> Result<Vec<String>> {
        let dir = self.snapshots_dir(sid);
        if !dir.exists() {
            return Ok(Vec::new());
        }
        let mut sids: Vec<String> = Vec::new();
        for entry in fs::read_dir(&dir)? {
            let entry = entry?;
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if let Some(stripped) = name.strip_suffix(".json") {
                sids.push(stripped.to_string());
            }
        }
        sids.sort();
        Ok(sids)
    }

    #[allow(dead_code)]
pub fn count_snapshots(&self, sid: &str) -> Result<u32> {
        Ok(self.list_snapshots(sid)?.len() as u32)
    }
}
