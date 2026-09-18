//! Verify the actual attach stream, including the child-controlled mouse modes.

use super::*;
use std::{
    fs::File,
    io::{Read, Write},
    os::{fd::FromRawFd, unix::process::CommandExt},
    time::Instant,
};

struct Attach {
    child: Child,
    master: Option<File>,
}

impl Attach {
    fn start(attach: chartr_herdr::control::DirectAttach) -> Self {
        use rustix::{fs::OFlags, io::FdFlags};
        let (mut master_fd, mut slave_fd) = (-1, -1);
        let mut size = libc::winsize { ws_row: 24, ws_col: 80, ws_xpixel: 0, ws_ypixel: 0 };
        // openpty initializes both descriptors on success; File then owns them.
        let (master, slave) = unsafe {
            assert_eq!(
                libc::openpty(
                    &mut master_fd,
                    &mut slave_fd,
                    std::ptr::null_mut(),
                    std::ptr::null_mut(),
                    &mut size
                ),
                0
            );
            (File::from_raw_fd(master_fd), File::from_raw_fd(slave_fd))
        };
        rustix::io::fcntl_setfd(&master, FdFlags::CLOEXEC).unwrap();
        rustix::io::fcntl_setfd(&slave, FdFlags::CLOEXEC).unwrap();
        let flags = rustix::fs::fcntl_getfl(&master).unwrap();
        rustix::fs::fcntl_setfl(&master, flags | OFlags::NONBLOCK).unwrap();
        let controlling = slave.try_clone().unwrap();
        let mut command = Command::new(attach.program);
        command
            .args(attach.args)
            .envs(attach.env)
            .env("TERM", "xterm-256color")
            .stdin(slave.try_clone().unwrap())
            .stdout(slave.try_clone().unwrap())
            .stderr(slave);
        // Only async-signal-safe syscalls run between fork and exec.
        unsafe {
            command.pre_exec(move || {
                rustix::process::setsid()?;
                rustix::process::ioctl_tiocsctty(&controlling)?;
                Ok(())
            });
        }
        Self { child: command.spawn().unwrap(), master: Some(master) }
    }

    fn until(&mut self, ready: impl Fn(&[u8]) -> bool) -> Vec<u8> {
        let deadline = Instant::now() + Duration::from_secs(5);
        let mut output = Vec::new();
        loop {
            let mut chunk = [0; 65536];
            match self.master.as_mut().unwrap().read(&mut chunk) {
                Ok(0) => panic!("attach exited: {}", String::from_utf8_lossy(&output)),
                Ok(n) => output.extend_from_slice(&chunk[..n]),
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {}
                Err(error) => panic!("reading attach: {error}"),
            }
            if ready(&output) {
                return output;
            }
            assert!(
                Instant::now() < deadline,
                "attach timed out: {}",
                String::from_utf8_lossy(&output)
            );
            std::thread::sleep(Duration::from_millis(10));
        }
    }

    fn send(&mut self, bytes: &[u8]) {
        self.master.as_mut().unwrap().write_all(bytes).unwrap();
    }
}

impl Drop for Attach {
    fn drop(&mut self) {
        // Closing the master also releases a child blocked draining its terminal.
        self.master.take();
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn contains(bytes: &[u8], needle: &[u8]) -> bool {
    bytes.windows(needle.len()).any(|window| window == needle)
}

fn run(live: &Live, pane: &chartr_herdr::PaneId, command: &str) {
    let mut process = Command::new(sidecar().path());
    process.args(["pane", "run", &pane.0, command]);
    for (key, value) in live.client.namespace().env() {
        match value {
            Some(value) => process.env(key, value),
            None => process.env_remove(key),
        };
    }
    let output = process.output().unwrap();
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
}

fn scroll_offset(live: &Live, pane: &chartr_herdr::PaneId) -> u64 {
    use std::{
        io::{BufRead, BufReader},
        os::unix::net::UnixStream,
    };
    let mut socket = UnixStream::connect(live.client.namespace().socket()).unwrap();
    socket.set_read_timeout(Some(Duration::from_secs(2))).unwrap();
    socket.set_write_timeout(Some(Duration::from_secs(2))).unwrap();
    writeln!(
        socket,
        "{}",
        serde_json::json!({
            "id": "mouse-test", "method": "pane.get", "params": {"pane_id": pane.0}
        })
    )
    .unwrap();
    let mut response = String::new();
    BufReader::new(socket).read_line(&mut response).unwrap();
    let response: serde_json::Value = serde_json::from_str(&response).unwrap();
    response["result"]["pane"]["scroll"]["offset_from_bottom"].as_u64().expect("pane scroll offset")
}

#[test]
#[ignore = "needs a real herdr daemon and PTY"]
fn shell_selection_does_not_capture_mouse_but_tui_gestures_still_arrive() {
    let live = Live::start();
    let session = live.client.create_workspace(live._root.path(), Some("mouse-test")).unwrap();
    let mut attach = Attach::start(live.client.direct_attach(&session.terminal));
    run(&live, &session.id, "printf '\\103HARTR_READY\\n'");
    let startup = attach.until(|bytes| contains(bytes, b"CHARTR_READY"));
    assert!(!contains(&startup, b"\x1b[?1000h"), "a plain shell must not capture selection");

    let gestures = b"\x1b[<0;2;2M\x1b[<32;3;2M\x1b[<0;3;2m";
    let received = live._root.path().join("mouse-input");
    run(
        &live,
        &session.id,
        &format!(
            "stty raw -echo; printf '\\033[?1002h\\033[?1006h'; dd bs=1 count={} of='{}' 2>/dev/null; printf '\\033[?1002l\\033[?1006l'; stty sane; printf '\\103HARTR_DONE\\n'",
            gestures.len(),
            received.display()
        ),
    );
    attach.until(|bytes| contains(bytes, b"\x1b[?1000h"));
    // Relaunching Chartr must restore an already-running TUI's mouse demand.
    drop(attach);
    let mut attach = Attach::start(live.client.direct_attach(&session.terminal));
    attach.until(|bytes| contains(bytes, b"\x1b[?1000h"));
    attach.send(gestures);
    let stopped = attach.until(|bytes| contains(bytes, b"CHARTR_DONE"));
    assert!(contains(&stopped, b"\x1b[?1000l"), "selection must return after TUI mouse mode ends");
    assert_eq!(std::fs::read(received).unwrap(), gestures, "preserve press, drag, and release");

    run(
        &live,
        &session.id,
        "i=0; while [ $i -lt 100 ]; do printf 'history-%s\\n' $i; i=$((i+1)); done; printf '\\103HARTR_HISTORY_DONE\\n'",
    );
    attach.until(|bytes| contains(bytes, b"CHARTR_HISTORY_DONE"));
    assert_eq!(scroll_offset(&live, &session.id), 0);
    attach.send(b"\x1b[<64;1;1M");
    // Scrolling must repaint older history, even though shell mouse capture is off.
    attach.until(|bytes| contains(bytes, b"history-"));
    let deadline = Instant::now() + Duration::from_secs(2);
    while scroll_offset(&live, &session.id) == 0 {
        assert!(Instant::now() < deadline, "wheel input must scroll the backend history");
        std::thread::sleep(Duration::from_millis(10));
    }
}
