//! Smoke tests against a real herdr daemon.
//!
//! Ignored by default: they start zeddy's private backend, run a shell in it,
//! and are therefore neither hermetic nor fast. Run them when the herdr pin
//! moves, which is the moment the CLI coupling in `zeddy-herdr::stream` can
//! break without any unit test noticing.
//!
//!     cargo test -p zeddy --test live_session -- --ignored --nocapture

use std::time::{Duration, Instant};

use zeddy_herdr::{Geometry, Namespace, Sidecar, control::Client};
use zeddy_vt::{Size, Terminal};

fn client() -> Client {
    let herdr = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/debug/herdr")
        .canonicalize()
        .expect("build zeddy first so herdr is vendored beside it");
    Client::new(Sidecar::at(herdr).expect("sidecar"), Namespace::private())
}

#[test]
#[ignore = "needs a real herdr daemon"]
fn a_shell_paints_something_within_a_few_seconds() {
    let client = client();
    client.connect(Duration::from_secs(10)).expect("the private daemon comes up");

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
