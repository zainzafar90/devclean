use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use serde::Serialize;

use crate::areas::Area;
use crate::clean::clean_area;
use crate::clock::local_timestamp;
use crate::command::{run, run_ok, QUICK};
use crate::context::Context;
use crate::error::{CoreError, Result};
use crate::format::human;

pub const LABEL: &str = "com.devclean.daily";

pub fn plist_path(home: &Path) -> PathBuf {
    home.join(format!("Library/LaunchAgents/{LABEL}.plist"))
}

fn xml_escape(text: &str) -> String {
    text.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
}

/// LaunchAgent that runs `<program> scheduled` every day at 03:00 and appends to the log.
pub fn plist_xml(program: &Path, log: &Path) -> String {
    let program = xml_escape(&program.to_string_lossy());
    let log = xml_escape(&log.to_string_lossy());
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>Label</key><string>{LABEL}</string>
  <key>ProgramArguments</key>
  <array><string>{program}</string><string>scheduled</string></array>
  <key>StartCalendarInterval</key><dict><key>Hour</key><integer>3</integer><key>Minute</key><integer>0</integer></dict>
  <key>StandardOutPath</key><string>{log}</string>
  <key>StandardErrorPath</key><string>{log}</string>
</dict>
</plist>
"#
    )
}

fn domain() -> String {
    // SAFETY: getuid never fails.
    format!("gui/{}", unsafe { libc::getuid() })
}

fn service() -> String {
    format!("{}/{LABEL}", domain())
}

pub fn enable(ctx: &Context, program: &Path) -> Result<()> {
    let plist = plist_path(&ctx.home);
    let log = ctx.log_file();
    for dir in [plist.parent(), log.parent()].into_iter().flatten() {
        fs::create_dir_all(dir).map_err(|err| CoreError::io(dir, err))?;
    }
    fs::write(&plist, plist_xml(program, &log)).map_err(|err| CoreError::io(&plist, err))?;
    let _ = run("launchctl", &["bootout", &service()], QUICK);
    run_ok("launchctl", &["bootstrap", &domain(), &plist.to_string_lossy()], QUICK)?;
    Ok(())
}

pub fn disable(ctx: &Context) -> Result<()> {
    let _ = run("launchctl", &["bootout", &service()], QUICK);
    let plist = plist_path(&ctx.home);
    match fs::remove_file(&plist) {
        Err(err) if err.kind() != std::io::ErrorKind::NotFound => Err(CoreError::io(&plist, err)),
        _ => Ok(()),
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LastRun {
    pub started: String,
    pub finished: Option<String>,
    pub freed: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScheduleStatus {
    pub enabled: bool,
    pub log_file: PathBuf,
    pub recent_lines: Vec<String>,
    pub last_run: Option<LastRun>,
}

pub fn status(ctx: &Context) -> ScheduleStatus {
    let enabled = run("launchctl", &["print", &service()], QUICK).is_ok_and(|out| out.status.success());
    let log_file = ctx.log_file();
    let log = fs::read_to_string(&log_file).unwrap_or_default();
    let lines: Vec<&str> = log.lines().collect();
    let recent_lines = lines[lines.len().saturating_sub(3)..]
        .iter()
        .map(|l| l.to_string())
        .collect();
    ScheduleStatus {
        enabled,
        log_file,
        recent_lines,
        last_run: parse_last_run(&log),
    }
}

/// The last `start`/`done` pair in the scheduled-run log, with the freed amount between them.
pub fn parse_last_run(log: &str) -> Option<LastRun> {
    let lines: Vec<&str> = log.lines().collect();
    let start = lines.iter().rposition(|line| line.ends_with(" start"))?;
    let rest = &lines[start + 1..];
    let finished = rest
        .iter()
        .find(|line| line.ends_with(" done"))
        .map(|line| line.trim_end_matches(" done").to_string());
    let freed = rest
        .iter()
        .find_map(|line| line.trim().strip_prefix("freed about "))
        .map(str::to_string);
    Some(LastRun {
        started: lines[start].trim_end_matches(" start").to_string(),
        finished,
        freed,
    })
}

/// The launchd job: clean Docker leftovers and log it.
pub fn scheduled_run(ctx: &Context, out: &mut dyn Write) -> std::io::Result<bool> {
    writeln!(out, "{} start", local_timestamp())?;
    let mut write_error = None;
    let before = crate::areas::area_size(ctx, Area::Docker);
    writeln!(out, "  cleaning docker ({})", human(before))?;
    let outcome = clean_area(ctx, Area::Docker, false, &mut |line| {
        if let Err(err) = writeln!(out, "    {line}") {
            write_error = Some(err);
        }
    });
    if let Some(err) = write_error {
        return Err(err);
    }
    for failure in &outcome.failures {
        writeln!(out, "    error: {failure}")?;
    }
    writeln!(out, "    freed about {}", human(outcome.freed()))?;
    writeln!(out, "{} done", local_timestamp())?;
    Ok(outcome.failures.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plist_runs_program_daily_at_three() {
        let xml = plist_xml(
            Path::new("/Users/dev/.local/bin/devclean"),
            Path::new("/Users/dev/Library/Logs/devclean.log"),
        );
        assert!(xml.contains("<string>com.devclean.daily</string>"));
        assert!(
            xml.contains("<array><string>/Users/dev/.local/bin/devclean</string><string>scheduled</string></array>")
        );
        assert!(xml.contains("<key>Hour</key><integer>3</integer><key>Minute</key><integer>0</integer>"));
        assert_eq!(
            xml.matches("<string>/Users/dev/Library/Logs/devclean.log</string>")
                .count(),
            2
        );
    }

    #[test]
    fn plist_escapes_paths() {
        let xml = plist_xml(Path::new("/Apps/R&D <x>/devclean"), Path::new("/l"));
        assert!(xml.contains("/Apps/R&amp;D &lt;x&gt;/devclean"));
    }

    #[test]
    fn plist_is_valid_for_plutil() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("p.plist");
        fs::write(&file, plist_xml(Path::new("/bin/devclean"), Path::new("/tmp/l.log"))).unwrap();
        let out = std::process::Command::new("plutil")
            .arg("-lint")
            .arg(&file)
            .output()
            .unwrap();
        assert!(out.status.success());
    }

    #[test]
    fn finds_last_run_in_log() {
        let log = "2026-09-26 03:00:00 start\n  cleaning docker (1 GB)\n    freed about 10 MB\n2026-09-26 03:00:05 done\n\
                   2026-09-27 03:00:00 start\n  cleaning docker (409 MB)\n    freed about 0 B\n2026-09-27 03:00:02 done\n";
        let last = parse_last_run(log).unwrap();
        assert_eq!(last.started, "2026-09-27 03:00:00");
        assert_eq!(last.finished.as_deref(), Some("2026-09-27 03:00:02"));
        assert_eq!(last.freed.as_deref(), Some("0 B"));
        assert!(parse_last_run("").is_none());
        let running = parse_last_run("2026-09-27 03:00:00 start\n").unwrap();
        assert!(running.finished.is_none());
    }
}
