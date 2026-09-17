//! Keep large commands out of interactive shell history and line editing.

use std::{io::Write, sync::Arc};

/// Agent input consists of a shell command followed by CR and, for typed delivery,
/// optional TUI input. Only the shell command is staged; subsequent bytes stay raw.
pub(super) fn stage(input: &[u8]) -> std::io::Result<(Vec<u8>, Option<Arc<tempfile::TempPath>>)> {
    let Some(end) = input.iter().position(|byte| *byte == b'\r') else {
        return Ok((input.to_vec(), None));
    };
    let command = &input[..end];
    if command.len() <= 4096 && !command.contains(&b'\n') {
        return Ok((input.to_vec(), None));
    }

    // NamedTempFile creates an unpredictable, owner-only file. Keep ownership until
    // the session ends, including when delivery fails before the shell reads it.
    let script = tempfile::Builder::new().prefix("chartr-agent-").suffix(".sh").tempfile()?;
    let (mut file, path) = script.into_parts();
    let quoted_path = format!("'{}'", path.to_string_lossy().replace('\'', "'\\''"));
    // The shell has opened the script by this point, so unlinking it does not affect
    // parsing or the agent. The full prompt never becomes an interactive history entry.
    writeln!(file, "/bin/rm -f -- {quoted_path}")?;
    file.write_all(command)?;
    file.write_all(b"\n")?;
    file.flush()?;

    let mut staged = format!(". {quoted_path}").into_bytes();
    staged.extend_from_slice(&input[end..]);
    Ok((staged, Some(Arc::new(path))))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn long_multiline_launch_preserves_exact_data_and_unlinks_its_script() {
        let prompt = "Unicode 🦀 'quotes' $HOME $(false) `false` \\backslash\n\n".repeat(4000);
        let command = format!("printf '%s' '{}'\r", prompt.replace('\'', "'\\''"));
        let (input, script) = stage(command.as_bytes()).unwrap();
        let script = script.unwrap();
        assert!(input.len() < 1024);
        assert!(!input.contains(&b'\n'));
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(std::fs::metadata(&*script).unwrap().permissions().mode() & 0o077, 0);
        }
        let command = std::str::from_utf8(&input[..input.len() - 1]).unwrap();
        let output = std::process::Command::new("/bin/sh").args(["-c", command]).output().unwrap();
        assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
        assert_eq!(output.stdout, prompt.as_bytes());
        assert!(!script.exists());
    }

    #[test]
    fn short_launches_and_typed_prompt_bytes_keep_their_delivery_semantics() {
        let ordinary = b"'agent'\rFirst line\nSecond line\r";
        let (input, script) = stage(ordinary).unwrap();
        assert_eq!(input, ordinary);
        assert!(script.is_none());

        let long_command = format!("'agent' '{}'\rtyped\ninput\r", "x".repeat(5000));
        let (input, script) = stage(long_command.as_bytes()).unwrap();
        assert!(input.ends_with(b"\rtyped\ninput\r"));
        let path = script.as_ref().unwrap().to_path_buf();
        assert!(path.exists());
        drop(script);
        assert!(!path.exists(), "a launch that never runs must clean up its staged prompt");
    }
}
