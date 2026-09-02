//! Smoke tests against a real Herdr daemon.
//!
//! Ignored by default: they start Chartr's private backend. Run them when the
//! Herdr pin moves, which is when the direct-attach CLI contract can change.
//!
//!     cargo test -p zeddy --test live_session -- --ignored --nocapture

use std::{
    process::{Child, Command, Stdio},
    time::Duration,
};

use zeddy_herdr::{Namespace, Sidecar, control::Client};

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
fn a_created_session_has_a_native_direct_attach_target() {
    let live = Live::start();
    let workspace =
        live.client.open_workspace(&std::env::temp_dir(), Some("zeddy-live")).expect("workspace");
    let session = live.client.start_session(&workspace, None).expect("session");
    let attach = live.client.direct_attach(&session.terminal);

    assert!(!session.terminal.0.is_empty());
    assert_eq!(attach.program, std::path::Path::new("/usr/bin/env"));
    assert_eq!(
        &attach.args[attach.args.len() - 4..],
        ["terminal", "attach", session.terminal.0.as_str(), "--takeover"]
    );
    assert!(attach.env.contains_key("HERDR_SOCKET_PATH"));
    live.client.close_session(&session.id).expect("close session");
}

#[test]
#[ignore = "needs a real herdr daemon"]
fn a_broken_backend_recovers_without_resurrecting_dead_sessions() {
    let mut live = Live::start();
    let workspace = live
        .client
        .open_workspace(&std::env::temp_dir(), Some("zeddy-recovery"))
        .expect("workspace");
    live.client.start_session(&workspace, None).expect("session");
    live.crash();

    live.client.restart().expect("one clean replacement starts");
    live.client.reconnect(Duration::from_secs(10)).expect("replacement answers");
    assert!(
        live.client.sessions(None).expect("replacement session list").is_empty(),
        "a dead PTY must not be presented as the old session"
    );
}
