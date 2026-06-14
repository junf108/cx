mod cli;
mod context;
mod display;
mod git_api;
mod session;
mod snapshot;
mod store;

use clap::Parser;
use cli::{Cli, Command};
use session::SessionManager;

fn determine_author() -> String {
    if let Ok(name) = git_api::run_git(&["config", "user.name"]) {
        if !name.is_empty() {
            return format!("user:{name}");
        }
    }
    "user".to_string()
}

fn run() -> session::Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Command::Init => SessionManager::init(),

        Command::Start => {
            let mgr = match SessionManager::open() {
                Ok(mgr) => mgr,
                Err(_) => {
                    SessionManager::init()?;
                    SessionManager::open()?
                }
            };
            let author = determine_author();
            mgr.start(&author)
        }

        Command::Apply {
            message,
            summary,
            intent,
            no_verify,
        } => {
            let spec = context::parse_intent_spec(intent)
                .map_err(|e| session::SessionError::User(e))?;
            let mgr = SessionManager::open()?;
            let author = determine_author();
            mgr.apply(message, summary.clone(), &spec, &author, *no_verify)
        }

        Command::Status => {
            let mgr = SessionManager::open()?;
            let info = mgr.status()?;
            display::print_status(&info);
            Ok(())
        }

        Command::End { merge, abandon } => {
            if *merge == *abandon {
                return Err(session::SessionError::User(
                    "specify --merge or --abandon".to_string(),
                ));
            }
            let mgr = SessionManager::open()?;
            mgr.end(*merge)
        }

        Command::Log { session_id } => {
            let mgr = session::SessionManager::open()?;
            let entries = mgr.log(session_id.as_deref())?;
            display::print_log(&entries, session_id.as_deref());
            Ok(())
        }

        Command::Review { snapshot_id } => {
            let mgr = SessionManager::open()?;
            let data = mgr.review(snapshot_id.as_deref())?;
            display::print_review(&data);
            Ok(())
        }
    }

}

fn main() {
    if let Err(e) = run() {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}
