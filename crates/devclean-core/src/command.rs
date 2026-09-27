use std::ffi::OsString;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::OnceLock;
use std::thread;
use std::time::Duration;

use wait_timeout::ChildExt;

use crate::error::{CoreError, Result};

pub const QUICK: Duration = Duration::from_secs(30);
pub const LONG: Duration = Duration::from_secs(600);

const PATH_MARKER: &str = "__DEVCLEAN_PATH__";

/// The PATH the user's login shell builds, so a Finder or launchd launch finds the same tools
/// as their terminal (GUI apps start with a bare PATH where stale shims can win).
fn login_shell_path() -> Option<OsString> {
    static PATH: OnceLock<Option<OsString>> = OnceLock::new();
    PATH.get_or_init(|| {
        let shell = std::env::var_os("SHELL").unwrap_or_else(|| "/bin/zsh".into());
        let script = format!("printf '{PATH_MARKER}%s{PATH_MARKER}' \"$PATH\"");
        let mut child = Command::new(shell)
            .args(["-ilc", &script])
            .current_dir(home_dir())
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .ok()?;
        let stdout = drain(child.stdout.take());
        match child.wait_timeout(QUICK / 6) {
            Ok(Some(_)) => {}
            _ => {
                let _ = child.kill();
                let _ = child.wait();
                return None;
            }
        }
        let text = stdout.join().ok()?;
        text.split(PATH_MARKER)
            .nth(1)
            .filter(|path| !path.is_empty())
            .map(OsString::from)
    })
    .clone()
}

fn home_dir() -> PathBuf {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir)
}

/// The login shell's PATH (or this process's), then the user's own tool folders before
/// system ones, so e.g. `~/Library/pnpm/pnpm` wins over a stale `/usr/local/bin/pnpm` shim.
pub fn search_path() -> OsString {
    let base = login_shell_path()
        .or_else(|| std::env::var_os("PATH"))
        .unwrap_or_default();
    let mut dirs: Vec<PathBuf> = std::env::split_paths(&base).collect();
    let home = home_dir();
    let extras = [
        home.join(".local/bin"),
        home.join("Library/pnpm"),
        PathBuf::from("/opt/homebrew/bin"),
        PathBuf::from("/usr/local/bin"),
        PathBuf::from("/usr/bin"),
        PathBuf::from("/bin"),
        PathBuf::from("/usr/sbin"),
    ];
    for extra in extras {
        if !dirs.contains(&extra) {
            dirs.push(extra);
        }
    }
    std::env::join_paths(dirs).unwrap_or_default()
}

pub fn find_tool(name: &str) -> Option<PathBuf> {
    std::env::split_paths(&search_path())
        .map(|dir| dir.join(name))
        .find(|candidate| is_executable(candidate))
}

pub fn has(name: &str) -> bool {
    find_tool(name).is_some()
}

fn is_executable(path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    path.metadata()
        .map(|meta| meta.is_file() && meta.permissions().mode() & 0o111 != 0)
        .unwrap_or(false)
}

#[derive(Debug)]
pub struct Output {
    pub stdout: String,
    pub stderr: String,
    pub status: std::process::ExitStatus,
}

/// Runs a tool found on the search path; kills it and returns `Timeout` if it overruns.
pub fn run(program: &str, args: &[&str], timeout: Duration) -> Result<Output> {
    let path = find_tool(program).ok_or_else(|| CoreError::NotInstalled {
        program: program.to_string(),
    })?;
    let mut child = Command::new(&path)
        .args(args)
        .env("PATH", search_path())
        .current_dir(home_dir())
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|source| CoreError::Spawn {
            program: program.to_string(),
            source,
        })?;
    let stdout = drain(child.stdout.take());
    let stderr = drain(child.stderr.take());
    let status = match child.wait_timeout(timeout) {
        Ok(Some(status)) => status,
        Ok(None) => {
            let _ = child.kill();
            let _ = child.wait();
            return Err(CoreError::Timeout {
                program: program.to_string(),
                seconds: timeout.as_secs(),
            });
        }
        Err(source) => {
            return Err(CoreError::Spawn {
                program: program.to_string(),
                source,
            })
        }
    };
    Ok(Output {
        stdout: stdout.join().unwrap_or_default(),
        stderr: stderr.join().unwrap_or_default(),
        status,
    })
}

/// Like `run`, but a non-zero exit is an error carrying the first stderr line.
pub fn run_ok(program: &str, args: &[&str], timeout: Duration) -> Result<String> {
    let output = run(program, args, timeout)?;
    if output.status.success() {
        return Ok(output.stdout);
    }
    Err(CoreError::CommandFailed {
        program: format!("{program} {}", args.join(" ")),
        status: output.status.to_string(),
        stderr: output.stderr.lines().next().unwrap_or("").trim().to_string(),
    })
}

fn drain<R: Read + Send + 'static>(pipe: Option<R>) -> thread::JoinHandle<String> {
    thread::spawn(move || {
        let mut text = String::new();
        if let Some(mut pipe) = pipe {
            let mut bytes = Vec::new();
            let _ = pipe.read_to_end(&mut bytes);
            text = String::from_utf8_lossy(&bytes).into_owned();
        }
        text
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_tool_is_not_installed() {
        let err = run("devclean-no-such-tool", &[], QUICK).unwrap_err();
        assert!(matches!(err, CoreError::NotInstalled { .. }));
    }

    #[test]
    fn slow_command_times_out() {
        let err = run("sleep", &["5"], Duration::from_millis(200)).unwrap_err();
        assert!(matches!(err, CoreError::Timeout { .. }));
    }

    #[test]
    fn user_tool_folders_come_before_system_ones_when_missing_from_path() {
        let path = search_path();
        let dirs: Vec<PathBuf> = std::env::split_paths(&path).collect();
        let home = home_dir();
        let position = |dir: &PathBuf| dirs.iter().position(|candidate| candidate == dir);
        let user = position(&home.join("Library/pnpm")).expect("pnpm folder on the search path");
        if let Some(system) = position(&PathBuf::from("/usr/local/bin")) {
            let from_base = std::env::split_paths(&login_shell_path().unwrap_or_default())
                .any(|dir| dir.as_path() == Path::new("/usr/local/bin"));
            assert!(from_base || user < system);
        }
    }

    #[test]
    fn commands_run_from_the_home_folder() {
        let out = run_ok("pwd", &[], QUICK).unwrap();
        assert_eq!(PathBuf::from(out.trim()), home_dir());
    }

    #[test]
    fn failing_command_reports_stderr() {
        let err = run_ok("ls", &["/devclean-missing"], QUICK).unwrap_err();
        assert!(err.to_string().contains("devclean-missing"));
    }
}
