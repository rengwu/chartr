//! Bounded one-shot child processes shared by plugin operations and Git installation.

use std::{
    io::{self, Read},
    os::unix::process::CommandExt as _,
    process::{Child, Command, Output, Stdio},
    sync::atomic::{AtomicBool, Ordering},
    time::{Duration, Instant},
};

pub fn output(
    command: &mut Command,
    timeout: Duration,
    limit: usize,
    cancel: &AtomicBool,
) -> io::Result<Output> {
    if cancel.load(Ordering::Relaxed) {
        return Err(io::Error::new(io::ErrorKind::Interrupted, "process cancelled"));
    }
    let deadline = Instant::now() + timeout;
    let mut child = ProcessGuard {
        child: command
            .process_group(0)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?,
        reaped: false,
    };
    let mut stdout = child.child.stdout.take().expect("piped stdout");
    let mut stderr = child.child.stderr.take().expect("piped stderr");
    rustix::fs::fcntl_setfl(&stdout, rustix::fs::OFlags::NONBLOCK)?;
    rustix::fs::fcntl_setfl(&stderr, rustix::fs::OFlags::NONBLOCK)?;
    let (mut out, mut err) = (Vec::new(), Vec::new());
    loop {
        if cancel.load(Ordering::Relaxed) {
            return Err(io::Error::new(io::ErrorKind::Interrupted, "process cancelled"));
        }
        if Instant::now() >= deadline {
            return Err(io::Error::new(io::ErrorKind::TimedOut, "process timed out"));
        }
        let previous_len = out.len() + err.len();
        let out_closed = drain_pipe(&mut stdout, &mut out, limit.saturating_sub(err.len()))?;
        let err_closed = drain_pipe(&mut stderr, &mut err, limit.saturating_sub(out.len()))?;
        // Do not reap the group leader while descendants still own the pipes:
        // retain its PID until cleanup, avoiding signalling a recycled PID.
        if out_closed
            && err_closed
            && let Some(status) = child.child.try_wait()?
        {
            child.reaped = true;
            return Ok(Output { status, stdout: out, stderr: err });
        }
        if previous_len == out.len() + err.len() {
            std::thread::sleep(Duration::from_millis(5));
        }
    }
}

/// Read one bounded chunk per turn so continuously producing stdout cannot
/// starve stderr or the deadline check.
fn drain_pipe(pipe: &mut impl Read, bytes: &mut Vec<u8>, limit: usize) -> io::Result<bool> {
    let mut buffer = [0; 16 * 1024];
    match pipe.read(&mut buffer) {
        Ok(0) => Ok(true),
        Ok(count) => {
            if bytes.len() + count > limit {
                return Err(io::Error::other("process output limit exceeded"));
            }
            bytes.extend_from_slice(&buffer[..count]);
            Ok(false)
        }
        Err(error)
            if matches!(error.kind(), io::ErrorKind::WouldBlock | io::ErrorKind::Interrupted) =>
        {
            Ok(false)
        }
        Err(error) => Err(error),
    }
}

struct ProcessGuard {
    child: Child,
    reaped: bool,
}

impl Drop for ProcessGuard {
    fn drop(&mut self) {
        if !self.reaped {
            if let Some(pid) = rustix::process::Pid::from_raw(self.child.id() as i32) {
                let _ = rustix::process::kill_process_group(pid, rustix::process::Signal::KILL);
            }
            let _ = self.child.kill();
            let _ = self.child.wait();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cancellation_terminates_the_process_group_and_returns_promptly() {
        let scratch = tempfile::tempdir().unwrap();
        let cancel = AtomicBool::new(false);
        let started = Instant::now();
        std::thread::scope(|scope| {
            let cancel_ref = &cancel;
            let root = scratch.path();
            scope.spawn(move || {
                let deadline = Instant::now() + Duration::from_secs(3);
                while !root.join("ready").exists() && Instant::now() < deadline {
                    std::thread::sleep(Duration::from_millis(5));
                }
                cancel_ref.store(true, Ordering::Relaxed);
            });
            let error = output(
                Command::new("sh").current_dir(root).args([
                    "-c",
                    "(sleep 0.5; printf leaked > survived) & printf ready > ready; wait",
                ]),
                Duration::from_secs(10),
                1024,
                &cancel,
            )
            .unwrap_err();
            assert_eq!(error.kind(), io::ErrorKind::Interrupted);
        });
        assert!(started.elapsed() < Duration::from_secs(3));
        std::thread::sleep(Duration::from_millis(600));
        assert!(!scratch.path().join("survived").exists());
    }
}
