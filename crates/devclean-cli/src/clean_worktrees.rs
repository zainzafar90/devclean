use std::io::{self, BufRead, Write};

use devclean_core::areas::area_size;
use devclean_core::command::has;
use devclean_core::worktree_cleanup::{describe, plan, remove};
use devclean_core::{human, Area, Context};

use crate::style::bold;

/// Worktrees can hold work that exists nowhere else, so the CLI always lists them and asks first.
pub fn clean_worktrees(ctx: &Context) -> bool {
    println!("{}", bold("cleaning worktrees"));
    if !has("git") {
        println!("  git not installed, skipped");
        return true;
    }
    let before = area_size(ctx, Area::Worktrees);
    let candidates = plan(ctx, &mut |line| println!("  {line}"));
    candidates
        .iter()
        .for_each(|candidate| println!("  {}", describe(candidate)));
    let count = candidates.iter().filter(|candidate| candidate.removable()).count();
    if count == 0 {
        println!("  nothing to remove");
        return true;
    }
    let plural = if count == 1 { "" } else { "s" };
    if !confirmed(&format!("Remove {count} worktree{plural}? [y/N] ")) {
        println!("  cancelled, nothing changed");
        return true;
    }
    let failures = remove(&candidates, &mut |line| println!("  {line}"));
    failures.iter().for_each(|failure| println!("  error: {failure}"));
    let after = area_size(ctx, Area::Worktrees);
    println!(
        "  freed about {} (was {})",
        human(before.saturating_sub(after)),
        human(before)
    );
    failures.is_empty()
}

/// Only an explicit "y" or "yes" counts; an empty line, end of input or a read error mean no.
fn confirmed(question: &str) -> bool {
    print!("  {question}");
    if let Err(err) = io::stdout().flush() {
        eprintln!("devclean: could not show the question: {err}");
        return false;
    }
    let mut answer = String::new();
    match io::stdin().lock().read_line(&mut answer) {
        Ok(0) => {
            println!();
            false
        }
        Ok(_) => matches!(answer.trim().to_ascii_lowercase().as_str(), "y" | "yes"),
        Err(err) => {
            eprintln!("devclean: could not read the answer: {err}");
            false
        }
    }
}
