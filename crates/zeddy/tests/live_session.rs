//! Smoke tests against a real herdr daemon.
//!
//! Ignored by default: they start zeddy's private backend, run a shell in it,
//! and are therefore neither hermetic nor fast. Run them when the herdr pin
//! moves, which is the moment the CLI coupling in `zeddy-herdr::stream` can
//! break without any unit test noticing.
//!
//!     cargo test -p zeddy --test live_session -- --ignored --nocapture

use std::{
    process::{Child, Command, Stdio},
    time::{Duration, Instant},
};

use zeddy_herdr::{Geometry, Namespace, Sidecar, control::Client};
use zeddy_vt::{Size, Terminal};

struct Live {
    client: Client,
    child: Option<Child>,
    _root: tempfile::TempDir,
}

impl Live {
    fn start() -> Self {
        let root = tempfile::tempdir().expect("scratch backend root");
        let namespace = Namespace::rooted(root.path().join("chartr-zeddy/herdr"));
        namespace.prepare().expect("namespace directories");
        let sidecar = sidecar();
        let mut command = Command::new(sidecar.path());
        command.arg("server").stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null());
        for (key, value) in namespace.env() {
            match value {
                Some(value) => command.env(key, value),
                None => command.env_remove(key),
            };
        }
        let child = command.spawn().expect("the vendored private daemon starts");
        let client = Client::new(sidecar, namespace);
        client.reconnect(Duration::from_secs(10)).expect("the private daemon answers");
        Self { client, child: Some(child), _root: root }
    }

    fn crash(&mut self) {
        let child = self.child.as_mut().expect("daemon child");
        child.kill().expect("kill daemon");
        child.wait().expect("reap daemon");
        self.child = None;
    }
}

impl Drop for Live {
    fn drop(&mut self) {
        let _ = self.client.stop_daemon();
        if let Some(child) = self.child.as_mut() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}

fn sidecar() -> Sidecar {
    let herdr = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/debug/herdr")
        .canonicalize()
        .expect("build zeddy first so herdr is vendored beside it");
    Sidecar::at(herdr).expect("sidecar")
}

#[test]
#[ignore = "needs a real herdr daemon"]
fn a_shell_paints_something_within_a_few_seconds() {
    let live = Live::start();
    let client = &live.client;

    let cwd = std::env::temp_dir();
    let workspace = client.open_workspace(&cwd, Some("zeddy-live")).expect("a workspace");
    let session = client.start_session(&workspace, None).expect("a session");
    println!("session {} in {workspace}", session.id);

    let size = Size::new(80, 24);
    let attachment =
        client.attach(&session.id, Geometry::new(size.cols, size.rows)).expect("attach");
    let (mut frames, mut input) = attachment.split();

    let mut terminal = Terminal::new(size);
    input.send(b"echo zeddy-live-marker\r").expect("send");

    let deadline = Instant::now() + Duration::from_secs(15);
    let mut seen = 0;
    while Instant::now() < deadline {
        match frames.next_frame().expect("the stream stays valid") {
            Some(frame) => {
                seen += 1;
                println!(
                    "frame {} full={} {}x{} {} bytes",
                    frame.seq,
                    frame.full,
                    frame.geometry.cols,
                    frame.geometry.rows,
                    frame.bytes.len()
                );
                if frame.full {
                    terminal.resize(Size::new(frame.geometry.cols, frame.geometry.rows));
                }
                terminal.feed(&frame.bytes);
                if terminal.screen().to_text().contains("zeddy-live-marker") {
                    break;
                }
            }
            None => panic!("the stream closed after {seen} frames"),
        }
    }

    let text = terminal.screen().to_text();
    println!("--- screen ---\n{text}\n--- end ---");
    let _ = client.close_session(&session.id);

    assert!(seen > 0, "no frames arrived at all");
    assert!(text.contains("zeddy-live-marker"), "the echo never reached the screen");
}

#[test]
#[ignore = "needs a real herdr daemon"]
fn a_broken_transport_recovers_without_resurrecting_dead_sessions() {
    let mut live = Live::start();
    let workspace = live
        .client
        .open_workspace(&std::env::temp_dir(), Some("zeddy-recovery"))
        .expect("workspace");
    let session = live.client.start_session(&workspace, None).expect("session");
    let attachment = live.client.attach(&session.id, Geometry::new(80, 24)).expect("attach");
    let (mut frames, mut input) = attachment.split();
    live.crash();

    let (sent, received) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let _ = sent.send(frames.next_frame());
    });
    let stream = received
        .recv_timeout(Duration::from_secs(5))
        .expect("the frame stream notices daemon death");
    assert!(stream.is_err() || stream.expect("checked error").is_none());
    assert!(input.send(b"echo should-not-send\r").is_err());

    live.client.restart().expect("one clean replacement starts");
    live.client.reconnect(Duration::from_secs(10)).expect("replacement answers");
    assert!(
        live.client.sessions(None).expect("replacement session list").is_empty(),
        "a dead PTY must not be presented as the old session"
    );
}
