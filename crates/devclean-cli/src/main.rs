mod clean_worktrees;
mod report;
mod style;

use std::process::ExitCode;

use devclean_core::schedule;
use devclean_core::{clean_area, human, Area, CleanOutcome, Context, AUTHOR, VERSION};

use style::bold;

fn usage(ctx: &Context) -> String {
    let areas: Vec<&str> = Area::ALL.iter().map(|area| area.name()).collect();
    let safe: Vec<&str> = Area::safe().map(Area::name).collect();
    format!(
        "devclean {VERSION} — memory report and cache clean-up for macOS

  devclean                   report memory, disk and cache sizes
  devclean clean <area>      clean one area
  devclean clean safe        clean every area that rebuilds itself ({})
  devclean clean ... --dry-run
                             show what would be cleaned, change nothing
  devclean schedule on|off|status
                             daily 03:00 clean of Docker leftovers (launchd)
  devclean version

Areas: {}
Worktrees: list repositories (one path per line) in {}; cleaning them always asks first",
        safe.join(" "),
        areas.join(" "),
        ctx.config_file.display()
    )
}

fn usage_error(ctx: &Context) -> ExitCode {
    eprintln!("{}", usage(ctx));
    ExitCode::from(2)
}

fn main() -> ExitCode {
    let ctx = match Context::from_env() {
        Ok(ctx) => ctx,
        Err(err) => {
            eprintln!("devclean: {err}");
            return ExitCode::FAILURE;
        }
    };
    let args: Vec<String> = std::env::args().skip(1).collect();
    let args: Vec<&str> = args.iter().map(String::as_str).collect();
    match args.as_slice() {
        [] | ["report"] => finish(report::print_report(&ctx)),
        ["clean", rest @ ..] => clean(&ctx, rest),
        ["schedule"] => schedule_command(&ctx, "status"),
        ["schedule", action] => schedule_command(&ctx, action),
        ["scheduled"] => scheduled(&ctx),
        ["version" | "--version" | "-v"] => {
            println!("devclean {VERSION} — built by {AUTHOR}");
            ExitCode::SUCCESS
        }
        ["help" | "--help" | "-h"] => {
            println!("{}", usage(&ctx));
            ExitCode::SUCCESS
        }
        _ => usage_error(&ctx),
    }
}

fn finish(result: Result<(), devclean_core::CoreError>) -> ExitCode {
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("devclean: {err}");
            ExitCode::FAILURE
        }
    }
}

fn clean(ctx: &Context, args: &[&str]) -> ExitCode {
    let dry_run = args.iter().any(|arg| matches!(*arg, "--dry-run" | "-n"));
    let targets: Vec<&str> = args.iter().copied().filter(|arg| !arg.starts_with('-')).collect();
    let areas: Vec<Area> = match targets.as_slice() {
        ["safe"] => Area::safe().collect(),
        [name] => match name.parse() {
            Ok(area) => vec![area],
            Err(err) => {
                eprintln!("{err}");
                return usage_error(ctx);
            }
        },
        _ => return usage_error(ctx),
    };
    let mut ok = true;
    for area in areas {
        ok &= if area == Area::Worktrees && !dry_run {
            clean_worktrees::clean_worktrees(ctx)
        } else {
            print_clean(ctx, area, dry_run)
        };
    }
    if ok {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

fn print_clean(ctx: &Context, area: Area, dry_run: bool) -> bool {
    println!("{}", bold(&format!("cleaning {area}")));
    let outcome: CleanOutcome = clean_area(ctx, area, dry_run, &mut |line| println!("  {line}"));
    for failure in &outcome.failures {
        println!("  error: {failure}");
    }
    if dry_run {
        println!("  dry run: nothing changed ({} now)", human(outcome.before));
    } else {
        println!(
            "  freed about {} (was {})",
            human(outcome.freed()),
            human(outcome.before)
        );
    }
    outcome.failures.is_empty()
}

fn schedule_command(ctx: &Context, action: &str) -> ExitCode {
    let log = ctx.log_file();
    match action {
        "on" => {
            let program = match std::env::current_exe().and_then(|exe| exe.canonicalize()) {
                Ok(program) => program,
                Err(err) => {
                    eprintln!("devclean: cannot locate own binary: {err}");
                    return ExitCode::FAILURE;
                }
            };
            let result = schedule::enable(ctx, &program);
            if result.is_ok() {
                println!("daily clean at 03:00 is on (log: {})", log.display());
            }
            finish(result)
        }
        "off" => {
            let result = schedule::disable(ctx);
            if result.is_ok() {
                println!("daily clean is off");
            }
            finish(result)
        }
        "status" => {
            let status = schedule::status(ctx);
            if status.enabled {
                println!("daily clean is on (log: {})", log.display());
                status.recent_lines.iter().for_each(|line| println!("{line}"));
            } else {
                println!("daily clean is off");
            }
            ExitCode::SUCCESS
        }
        _ => usage_error(ctx),
    }
}

fn scheduled(ctx: &Context) -> ExitCode {
    match schedule::scheduled_run(ctx, &mut std::io::stdout()) {
        Ok(true) => ExitCode::SUCCESS,
        Ok(false) => ExitCode::FAILURE,
        Err(err) => {
            eprintln!("devclean: could not write log: {err}");
            ExitCode::FAILURE
        }
    }
}
