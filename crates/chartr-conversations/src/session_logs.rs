//! External session logs. OpenCode's own CLI produces its full database export.
use crate::{NativeSession, transcripts::validate_native_id};
use anyhow::{Context, Result, ensure};
use serde::Deserialize;
use std::{
    fs::File,
    io::{BufReader, Read},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    time::{Duration, Instant},
};

/// Export locally without starting an agent. Keep the temporary file so an external
/// editor can still read it after this call (or Chartr) returns.
pub fn export_opencode_session(native: &NativeSession, cwd: Option<&Path>) -> Result<PathBuf> {
    validate_native_id(&native.id)?;
    let mut command = Command::new(opencode_executable());
    // An archived project's folder may have been removed; exports are indexed by ID.
    if let Some(cwd) = cwd.filter(|cwd| cwd.is_dir()) {
        command.current_dir(cwd);
    }
    export_with_command(command, &native.id, Duration::from_secs(30))
}

fn opencode_executable() -> PathBuf {
    if let Some(paths) = std::env::var_os("PATH") {
        for directory in std::env::split_paths(&paths).filter(|path| path.is_absolute()) {
            let path = directory.join("opencode");
            if path.is_file() {
                return path;
            }
        }
    }
    let home = std::env::var_os("HOME").map(PathBuf::from).unwrap_or_default();
    let installed = home.join(".opencode/bin/opencode");
    if installed.is_file() { installed } else { PathBuf::from("opencode") }
}

fn export_with_command(mut command: Command, id: &str, timeout: Duration) -> Result<PathBuf> {
    validate_native_id(id)?;
    let output = tempfile::Builder::new().prefix("chartr-session-").suffix(".json").tempfile()?;
    let errors = tempfile::tempfile()?;
    // Redirect to files: a full log may exceed pipe buffers or the sidebar reader's limits.
    let mut child = command
        .args(["export", id])
        .stdin(Stdio::null())
        .stdout(output.as_file().try_clone()?)
        .stderr(errors.try_clone()?)
        .spawn()
        .context("Could not run OpenCode to export this session. Check that it is installed")?;
    let started = Instant::now();
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) if started.elapsed() < timeout => {
                std::thread::sleep(Duration::from_millis(50))
            }
            result => {
                let _ = child.kill();
                let _ = child.wait();
                result.context("Waiting for the OpenCode export")?;
                anyhow::bail!(
                    "OpenCode session export timed out. Try again when OpenCode is available."
                );
            }
        }
    };
    if !status.success() {
        use std::io::{Seek, SeekFrom};
        let mut errors = errors;
        errors.seek(SeekFrom::Start(0))?;
        let mut detail = String::new();
        errors.take(4096).read_to_string(&mut detail).ok();
        anyhow::bail!("OpenCode could not export this session ({status}): {}", detail.trim());
    }
    validate_export(output.path(), id)?;
    let (_, path) = output.keep().context("Keeping the exported session log for the editor")?;
    Ok(path)
}

fn validate_export(path: &Path, id: &str) -> Result<()> {
    // Deserialize only the identity, skipping message bodies without allocating them.
    #[derive(Deserialize)]
    struct Export {
        info: Identity,
    }
    #[derive(Deserialize)]
    struct Identity {
        id: String,
    }
    let exported: Export = serde_json::from_reader(BufReader::new(File::open(path)?))
        .context("OpenCode returned an invalid session export")?;
    ensure!(
        exported.info.id == id,
        "OpenCode exported a different session; the file was not opened"
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn shell(script: &str) -> Command {
        let mut command = Command::new("/bin/sh");
        command.args(["-c", script, "fixture"]);
        command
    }

    #[test]
    fn export_passes_exact_id_and_keeps_complete_private_json() {
        let command = shell(
            r#"
            [ "$1" = export ] && [ "$2" = session-a ] && [ "$#" = 2 ] || exit 1
            printf '%s' '{"info":{"id":"session-a"},"messages":[{"parts":[{"type":"tool","state":{"output":"full tool output"}}]}]}'
        "#,
        );
        let path = export_with_command(command, "session-a", Duration::from_secs(2)).unwrap();
        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.contains("full tool output"));
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(std::fs::metadata(&path).unwrap().permissions().mode() & 0o777, 0o600);
        }
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn exports_reject_wrong_sessions_invalid_output_failed_commands_and_missing_executables() {
        for script in [
            r#"printf '%s' '{"info":{"id":"other"}}'"#,
            "printf 'not json'",
            "printf 'No such session' >&2; exit 1",
        ] {
            assert!(
                export_with_command(shell(script), "session-a", Duration::from_secs(2)).is_err()
            );
        }
        let command = Command::new("/nonexistent/chartr-test-opencode");
        assert!(export_with_command(command, "session-a", Duration::from_secs(2)).is_err());
        for id in ["--help", "../other", "a;touch bad"] {
            assert!(export_with_command(shell("exit 0"), id, Duration::from_secs(2)).is_err());
        }
    }

    #[test]
    fn stalled_exports_are_terminated() {
        let started = Instant::now();
        let error =
            export_with_command(shell("exec sleep 5"), "session-a", Duration::from_millis(50))
                .unwrap_err();
        assert!(error.to_string().contains("timed out"));
        assert!(started.elapsed() < Duration::from_secs(3));
    }

    #[test]
    #[ignore = "Requires installed OpenCode and an existing local session; reads and exports only"]
    fn installed_opencode_exports_a_real_session() {
        let listing = Command::new(opencode_executable())
            .args(["session", "list", "--format", "json", "--max-count", "1"])
            .stdin(Stdio::null())
            .output()
            .unwrap();
        assert!(listing.status.success(), "OpenCode session listing failed");
        let rows: serde_json::Value = serde_json::from_slice(&listing.stdout).unwrap();
        let id = rows[0]["id"].as_str().expect("A local OpenCode session is required");
        let native = NativeSession { id: id.into(), path: None };
        let path = export_opencode_session(&native, None).unwrap();
        assert!(path.is_file());
        std::fs::remove_file(path).unwrap();
    }
}
