//! Keep agent commands out of interactive shell editing and shell fallback.

use std::{io::Write, sync::Arc};

pub(super) struct StagedLaunch {
    pub start: Vec<u8>,
    pub input: Vec<u8>,
    pub files: Arc<LaunchFiles>,
}

pub(super) struct LaunchFiles {
    pub script: tempfile::TempPath,
    ready: tempfile::TempPath,
}

impl LaunchFiles {
    pub fn ready(&self) -> bool {
        std::fs::read(&self.ready).is_ok_and(|bytes| bytes == b"ready")
    }

    pub fn acknowledge(&self) {
        let _ = std::fs::remove_file(&self.ready);
    }

    pub fn pending(&self) -> bool {
        self.script.exists() || self.ready.exists()
    }
}

fn quoted_path(path: &std::path::Path) -> std::io::Result<String> {
    let path =
        path.to_str().filter(|path| !path.chars().any(char::is_control)).ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Launch directory must be UTF-8 without control characters",
            )
        })?;
    Ok(format!("'{}'", path.replace('\'', "'\\''")))
}

pub(super) fn stage(launch: &chartr_plugin::TerminalLaunch) -> std::io::Result<StagedLaunch> {
    if launch.command.trim().is_empty() || launch.command.contains('\0') {
        return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, "Invalid agent command"));
    }
    let script = tempfile::Builder::new().prefix("chartr-agent-").suffix(".sh").tempfile()?;
    let ready = tempfile::Builder::new().prefix("chartr-agent-ready-").tempfile()?.into_temp_path();
    let (mut file, script) = script.into_parts();
    let quoted_script = quoted_path(&script)?;
    let quoted_ready = quoted_path(&ready)?;
    // The interactive shell has been replaced before this acknowledgement. Only
    // then may typed input enter the PTY, including interrupt/editing characters.
    writeln!(file, "printf ready > {quoted_ready}")?;
    writeln!(file, "/bin/rm -f -- {quoted_script}")?;
    file.write_all(launch.command.as_bytes())?;
    file.write_all(b"\n")?;
    file.flush()?;

    // /bin/sh reads commands from the file, never from the queued prompt. EOF or
    // a failed command ends this shell too. `exit` covers failure of exec itself.
    let start = format!("exec /bin/sh {quoted_script}; exit\r").into_bytes();
    Ok(StagedLaunch {
        start,
        input: launch.input.clone(),
        files: Arc::new(LaunchFiles { script, ready }),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn launch_preserves_literal_arguments_and_unlinks_its_script() {
        for prompt in [
            "short\u{15} control\r\n'quotes'\t$HOME $(false) `false` \\backslash".to_owned(),
            "Unicode 🦀 'quotes' $HOME $(false) `false` \\backslash\r\n\n".repeat(4000),
        ] {
            let launch = chartr_plugin::TerminalLaunch {
                command: format!("printf '%s' '{}'", prompt.replace('\'', "'\\''")),
                input: Vec::new(),
            };
            let staged = stage(&launch).unwrap();
            assert!(staged.start.len() < 1024);
            assert!(
                !staged.start[..staged.start.len() - 1].iter().any(|byte| byte.is_ascii_control())
            );
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                assert_eq!(
                    std::fs::metadata(&staged.files.script).unwrap().permissions().mode() & 0o077,
                    0
                );
            }
            let command = std::str::from_utf8(&staged.start[..staged.start.len() - 1]).unwrap();
            let output =
                std::process::Command::new("/bin/sh").args(["-c", command]).output().unwrap();
            assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
            assert_eq!(output.stdout, prompt.as_bytes());
            assert!(!staged.files.script.exists());
            assert!(staged.files.ready());
            staged.files.acknowledge();
            assert!(!staged.files.pending());
        }
    }

    #[test]
    fn typed_input_is_separate_and_abandoned_launches_clean_up() {
        let launch = chartr_plugin::TerminalLaunch {
            command: "'agent' 'argument\r\nwith CRLF'".into(),
            input: b"typed\ninput\r".to_vec(),
        };
        let staged = stage(&launch).unwrap();
        assert_eq!(staged.input, launch.input);
        assert!(!staged.start.windows(5).any(|word| word == b"typed"));
        assert!(!staged.files.ready());
        let script = staged.files.script.to_path_buf();
        let ready = staged.files.ready.to_path_buf();
        drop(staged);
        assert!(!script.exists());
        assert!(!ready.exists());
    }

    fn wait_for(child: &mut std::process::Child, condition: impl Fn() -> bool) {
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
        while !condition() {
            if std::time::Instant::now() >= deadline {
                let _ = child.kill();
                let _ = child.wait();
                panic!("launch did not become ready");
            }
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
    }

    fn shell(staged: &StagedLaunch) -> std::process::Child {
        use std::process::{Command, Stdio};
        let mut child = Command::new("/bin/sh")
            .arg("-i")
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .unwrap();
        // Pipes have no PTY CR-to-LF translation.
        let mut start = staged.start.clone();
        *start.last_mut().unwrap() = b'\n';
        child.stdin.as_mut().unwrap().write_all(&start).unwrap();
        wait_for(&mut child, || staged.files.ready());
        child
    }

    fn exited(child: &mut std::process::Child) -> std::process::ExitStatus {
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
        loop {
            if let Some(status) = child.try_wait().unwrap() {
                return status;
            }
            if std::time::Instant::now() >= deadline {
                let _ = child.kill();
                let _ = child.wait();
                panic!("agent left an interactive shell alive");
            }
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
    }

    #[test]
    fn failed_and_finished_agents_never_execute_queued_prompt_as_shell_commands() {
        let root = tempfile::tempdir().unwrap();
        let marker = root.path().join("unexpected-command");
        let prompt = format!("printf executed > {}\n", quoted_path(&marker).unwrap());
        for (command, code) in [
            ("'/nonexistent-chartr-test-agent'", 127),
            ("sleep 0.05; exit 42", 42),
            ("sleep 0.05; exit 0", 0),
        ] {
            let staged = stage(&chartr_plugin::TerminalLaunch {
                command: command.into(),
                input: prompt.as_bytes().to_vec(),
            })
            .unwrap();
            let mut child = shell(&staged);
            // A fast failure may have closed stdin already, which is also safe.
            let _ = child.stdin.as_mut().unwrap().write_all(&staged.input);
            assert_eq!(exited(&mut child).code(), Some(code));
            assert!(!marker.exists(), "queued prompt was evaluated by a shell");
        }
    }

    #[test]
    fn a_slow_starting_agent_receives_queued_text_literally() {
        let root = tempfile::tempdir().unwrap();
        let received = root.path().join("received");
        let prompt = "Please inspect $HOME; $(printf ignored) 'quotes' \\backslash";
        let staged = stage(&chartr_plugin::TerminalLaunch {
            command: format!(
                "sleep 0.15; IFS= read -r prompt; printf '%s' \"$prompt\" > {}",
                quoted_path(&received).unwrap()
            ),
            input: format!("{prompt}\n").into_bytes(),
        })
        .unwrap();
        let mut child = shell(&staged);
        child.stdin.as_mut().unwrap().write_all(&staged.input).unwrap();
        assert!(exited(&mut child).success());
        assert_eq!(std::fs::read_to_string(received).unwrap(), prompt);
    }
}
