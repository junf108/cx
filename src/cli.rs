use clap::{Parser, Subcommand};

/// cx — AI-native code management tool
#[derive(Parser)]
#[command(name = "cx", version, about)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Initialize .cx/ metadata store in the current git repository
    Init,
    /// Start a new AI session
    Start {
        /// The prompt or requirement that starts this session
        prompt: String,
    },
    /// Record current changes as a snapshot
    Apply {
        /// Description of the change
        #[arg(short = 'm', long)]
        message: String,

        /// Semantic label, format: type,scope=x[,risk=low]
        #[arg(long)]
        intent: String,

        /// Skip git hooks
        #[arg(long)]
        no_verify: bool,
    },
    /// Show current session status
    Status,
    /// End the current session
    End {
        /// Merge session branch into base branch
        #[arg(long)]
        merge: bool,

        /// Abandon session branch
        #[arg(long)]
        abandon: bool,
    },
    /// View snapshot history
    Log {
        /// Optional session ID to inspect
        session_id: Option<String>,
    },
    /// Review snapshots grouped by intent
    Review {
        /// Optional specific snapshot ID
        snapshot_id: Option<String>,
    },
}
