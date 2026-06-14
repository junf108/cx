use crate::session::{LogEntry, ReviewData, SessionStatusInfo};

/// Print session status
pub fn print_status(info: &SessionStatusInfo) {
    println!("Session: {}", info.id);
    println!("Branch:  {}", info.branch);
    println!("Status:  {} ({} snapshots)", info.status, info.snapshot_count);

    if info.semantic_units.is_empty() {
        println!("No changes recorded yet.");
        return;
    }

    println!("Changes by intent:");
    for unit in &info.semantic_units {
        let risk_tag = if unit.risk.to_string() == "high" {
            "  [risk: high]"
        } else if unit.risk.to_string() == "medium" {
            "  [risk: medium]"
        } else {
            ""
        };
        println!(
            "  {} ({} file{}){}",
            unit.intent,
            unit.affected_files.len(),
            if unit.affected_files.len() == 1 { "" } else { "s" },
            risk_tag,
        );
        for f in &unit.affected_files {
            println!("    {}  {}", '\u{2500}', f);
        }
    }
}

/// Print log entries
pub fn print_log(entries: &[LogEntry], session_id: Option<&str>) {
    if let Some(sid) = session_id {
        println!("Session: {sid}");
        println!();
        for entry in entries {
            // Truncate timestamp to seconds
            let ts = if entry.timestamp.len() > 19 {
                &entry.timestamp[..19]
            } else {
                &entry.timestamp
            };
            println!("  {}  {}  {}  [{}]  risk: {}",
                entry.snapshot_id,
                ts,
                entry.description,
                entry.intent,
                entry.risk,
            );
        }
    }
}

/// Print review data
pub fn print_review(data: &ReviewData) {
    println!(
        "Reviewing session {} (vs {}@{})",
        data.session_id,
        data.base_branch,
        &data.base_commit[..12.min(data.base_commit.len())]
    );
    println!();

    if data.groups.is_empty() {
        println!("No snapshots to review.");
        return;
    }

    for group in &data.groups {
        let header = format!("{} ({})", group.intent, group.scope);
        let line = "\u{2500}".repeat(header.len().max(30));
        println!("{}", header);
        println!("{}", line);

        if group.files.is_empty() {
            println!("  (no file changes)");
        } else {
            for (filename, added, removed) in &group.files {
                let stat = if *added > 0 || *removed > 0 {
                    format!(" +{added}/-{removed}")
                } else {
                    String::new()
                };
                println!("  {}{}", filename, stat);
            }
        }
        println!();
    }
}
