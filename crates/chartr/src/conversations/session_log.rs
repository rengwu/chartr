//! macOS's generic file opener cannot report failures through GPUI. Check the
//! result ourselves so unassociated JSONL files can fall back to a text editor.
use anyhow::{Context, Result, ensure};
use std::{
    ffi::OsStr,
    path::Path,
    process::{Command, Stdio},
};

pub(super) fn open(path: &Path) -> Result<()> {
    ensure!(path.is_file(), "The session log file is no longer available.");
    open_with(path, |args| {
        let output = Command::new("/usr/bin/open")
            .args(args)
            .stdin(Stdio::null())
            .output()
            .context("Could not launch the system file opener")?;
        ensure!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr).trim());
        Ok(())
    })
}

fn open_with(path: &Path, mut launch: impl FnMut(&[&OsStr]) -> Result<()>) -> Result<()> {
    let file = path.as_os_str();
    // Preserve an existing association, then try the default text editor and
    // finally TextEdit if the configured text editor is also unavailable.
    if launch(&[OsStr::new("--"), file]).is_ok() {
        return Ok(());
    }
    if launch(&[OsStr::new("-t"), OsStr::new("--"), file]).is_ok() {
        return Ok(());
    }
    launch(&[OsStr::new("-e"), OsStr::new("--"), file])
        .context("Could not open the session log in a text editor")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unassociated_logs_fall_back_to_text_editors_without_changing_the_path() {
        let path = Path::new("/tmp/session with spaces;$(literal)/wire.jsonl");
        for successful_attempt in 0..3 {
            let mut calls = Vec::new();
            open_with(path, |args| {
                calls.push(args.iter().map(|arg| arg.to_os_string()).collect::<Vec<_>>());
                ensure!(calls.len() > successful_attempt, "No associated application");
                Ok(())
            })
            .unwrap();
            assert_eq!(calls.len(), successful_attempt + 1);
            for (index, args) in calls.iter().enumerate() {
                assert_eq!(args.last().unwrap(), path.as_os_str());
                assert_eq!(args[args.len() - 2], "--");
                if index > 0 {
                    assert_eq!(args[0], if index == 1 { "-t" } else { "-e" });
                }
            }
        }
    }

    #[test]
    fn failure_of_every_opener_is_reported() {
        let mut attempts = 0;
        let result = open_with(Path::new("/tmp/wire.jsonl"), |_| {
            attempts += 1;
            anyhow::bail!("Unable to launch editor")
        });
        assert_eq!(attempts, 3);
        assert!(result.unwrap_err().to_string().contains("text editor"));
    }
}
