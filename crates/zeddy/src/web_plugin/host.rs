//! Blocking web-plugin operations. Call only from the instance's background worker.

use std::{
    io::{self, Read},
    os::unix::process::CommandExt as _,
    process::{Child, Command, Stdio},
    time::{Duration, Instant},
};

use serde_json::{Value, json};

pub(super) const MAX_REQUEST_BYTES: usize = 1024 * 1024;
const MAX_OUTPUT_BYTES: usize = 8 * 1024 * 1024;
const OPERATION_TIMEOUT: Duration = Duration::from_secs(30);
const MAX_REDIRECTS: usize = 10;

pub(super) fn read_file(file: std::fs::File) -> io::Result<String> {
    let mut bytes = Vec::new();
    file.take(MAX_OUTPUT_BYTES as u64 + 1).read_to_end(&mut bytes)?;
    if bytes.len() > MAX_OUTPUT_BYTES {
        return Err(io::Error::other("file exceeds 8 MiB"));
    }
    String::from_utf8(bytes).map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))
}

fn allowed_url(url: &url::Url, allowed: &[String]) -> Result<(), String> {
    if !matches!(url.scheme(), "http" | "https") {
        return Err("only HTTP and HTTPS network actions are allowed".into());
    }
    let host = url.host_str().ok_or("the URL has no host")?;
    if allowed.iter().any(|entry| {
        let allowed_host = url::Url::parse(entry)
            .ok()
            .and_then(|url| url.host_str().map(str::to_owned))
            .unwrap_or_else(|| entry.trim_start_matches("*.").to_owned());
        host == allowed_host
            || (entry.starts_with("*.") && host.ends_with(&format!(".{allowed_host}")))
    }) {
        Ok(())
    } else {
        Err(format!("network access to `{host}` is not declared"))
    }
}

pub(super) fn fetch(requested: &str, allowed: &[String]) -> Result<Value, String> {
    fetch_with_timeout(requested, allowed, OPERATION_TIMEOUT)
}

fn fetch_with_timeout(
    requested: &str,
    allowed: &[String],
    timeout: Duration,
) -> Result<Value, String> {
    let mut url = url::Url::parse(requested).map_err(|error| error.to_string())?;
    let deadline = Instant::now() + timeout;
    // Keep connection pooling within the operation; never let ureq follow a URL
    // before that destination has passed the same permission check as the first.
    let agent: ureq::Agent = ureq::Agent::config_builder().max_redirects(0).build().into();
    for redirects in 0..=MAX_REDIRECTS {
        allowed_url(&url, allowed)?;
        let remaining = deadline
            .checked_duration_since(Instant::now())
            .filter(|duration| !duration.is_zero())
            .ok_or("network request timed out")?;
        let mut response = agent
            .get(url.as_str())
            .config()
            .timeout_global(Some(remaining))
            .build()
            .call()
            .map_err(|error| error.to_string())?;
        if matches!(response.status().as_u16(), 301 | 302 | 303 | 307 | 308)
            && let Some(location) = response.headers().get("location")
        {
            if redirects == MAX_REDIRECTS {
                return Err("too many network redirects".into());
            }
            url = url
                .join(location.to_str().map_err(|error| error.to_string())?)
                .map_err(|error| error.to_string())?;
            continue;
        }
        let status = response.status().as_u16();
        let body = response
            .body_mut()
            .with_config()
            .limit(MAX_OUTPUT_BYTES as u64)
            .read_to_string()
            .map_err(|error| error.to_string())?;
        return Ok(json!({ "status": status, "body": body }));
    }
    unreachable!("the last redirect returns an error")
}

pub(super) fn run_process(command: &str, args: &[String]) -> Result<Value, String> {
    run_process_with_limits(command, args, OPERATION_TIMEOUT, MAX_OUTPUT_BYTES)
        .map_err(|error| error.to_string())
}

fn run_process_with_limits(
    command: &str,
    args: &[String],
    timeout: Duration,
    limit: usize,
) -> io::Result<Value> {
    let deadline = Instant::now() + timeout;
    let mut child = ProcessGuard {
        child: Command::new(command)
            .args(args)
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
        if Instant::now() >= deadline {
            return Err(io::Error::new(io::ErrorKind::TimedOut, "plugin process timed out"));
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
            return Ok(json!({ "status": status.code(),
                "stdout": String::from_utf8_lossy(&out), "stderr": String::from_utf8_lossy(&err) }));
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
                return Err(io::Error::other("plugin process output limit exceeded"));
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
    use std::{
        io::{BufRead as _, Write as _},
        net::TcpListener,
    };

    fn server(replies: Vec<String>) -> (String, std::thread::JoinHandle<()>) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let url = format!("http://{}", listener.local_addr().unwrap());
        let worker = std::thread::spawn(move || {
            listener.set_nonblocking(true).unwrap();
            for reply in replies {
                let deadline = Instant::now() + Duration::from_secs(3);
                let mut stream = loop {
                    match listener.accept() {
                        Ok((stream, _)) => break stream,
                        Err(error)
                            if error.kind() == io::ErrorKind::WouldBlock
                                && Instant::now() < deadline =>
                        {
                            std::thread::sleep(Duration::from_millis(5));
                        }
                        Err(error) => panic!("missing test HTTP request: {error}"),
                    }
                };
                stream.set_read_timeout(Some(Duration::from_secs(2))).unwrap();
                let mut reader = io::BufReader::new(&stream);
                loop {
                    let mut line = String::new();
                    if reader.read_line(&mut line).unwrap() == 0 || line == "\r\n" {
                        break;
                    }
                }
                let _ = stream.write_all(reply.as_bytes());
            }
        });
        (url, worker)
    }

    fn redirect(location: &str) -> String {
        format!(
            "HTTP/1.1 302 Found\r\nLocation: {location}\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
        )
    }

    #[test]
    fn redirects_cannot_leave_the_network_allowlist() {
        let (url, server) = server(vec![redirect("http://localhost:1/forbidden")]);
        let error = fetch(&url, &["127.0.0.1".into()]).unwrap_err();
        assert!(error.contains("network access to `localhost` is not declared"), "{error}");
        server.join().unwrap();
    }

    #[test]
    fn relative_redirects_to_allowed_destinations_still_work() {
        let (url, server) = server(vec![
            redirect("/next"),
            "HTTP/1.1 200 OK\r\nContent-Length: 2\r\nConnection: close\r\n\r\nok".into(),
        ]);
        let response = fetch(&url, &["127.0.0.1".into()]).unwrap();
        assert_eq!(response["status"], 200);
        assert_eq!(response["body"], "ok");
        server.join().unwrap();
    }

    #[test]
    fn redirect_loops_are_bounded_and_non_http_redirects_are_denied() {
        let (url, worker) = server(vec![redirect("/loop"); MAX_REDIRECTS + 1]);
        assert!(fetch(&url, &["127.0.0.1".into()]).unwrap_err().contains("too many"));
        worker.join().unwrap();
        let (url, worker) = server(vec![redirect("file:///etc/passwd")]);
        assert!(fetch(&url, &["127.0.0.1".into()]).unwrap_err().contains("only HTTP"));
        worker.join().unwrap();
    }

    #[test]
    fn stalled_http_responses_time_out() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let url = format!("http://{}", listener.local_addr().unwrap());
        let (release, wait) = std::sync::mpsc::channel::<()>();
        let server = std::thread::spawn(move || {
            let (_stream, _) = listener.accept().unwrap();
            let _ = wait.recv_timeout(Duration::from_secs(2));
        });
        let start = Instant::now();
        assert!(
            fetch_with_timeout(&url, &["127.0.0.1".into()], Duration::from_millis(50)).is_err()
        );
        assert!(start.elapsed() < Duration::from_secs(1));
        let _ = release.send(());
        server.join().unwrap();
    }

    #[test]
    fn process_results_preserve_exit_status_and_both_streams() {
        let value =
            run_process("sh", &["-c".into(), "printf out; printf err >&2; exit 7".into()]).unwrap();
        assert_eq!(value, json!({ "status": 7, "stdout": "out", "stderr": "err" }));
    }

    #[test]
    fn processes_and_inherited_output_pipes_have_a_deadline() {
        for script in ["sleep 10", "sleep 10 & exit 0"] {
            let start = Instant::now();
            let error = run_process_with_limits(
                "sh",
                &["-c".into(), script.into()],
                Duration::from_millis(50),
                1024,
            )
            .unwrap_err();
            assert_eq!(error.kind(), io::ErrorKind::TimedOut);
            assert!(start.elapsed() < Duration::from_secs(1));
        }
    }

    #[test]
    fn continuous_process_output_is_bounded() {
        let error = run_process_with_limits(
            "sh",
            &["-c".into(), "while :; do printf lots-of-output; done".into()],
            Duration::from_secs(2),
            1024,
        )
        .unwrap_err();
        assert!(error.to_string().contains("output limit"), "{error}");
    }

    #[test]
    fn file_reads_are_bounded() {
        let file = tempfile::tempfile().unwrap();
        file.set_len(MAX_OUTPUT_BYTES as u64 + 1).unwrap();
        assert!(read_file(file).unwrap_err().to_string().contains("exceeds"));
    }
}
