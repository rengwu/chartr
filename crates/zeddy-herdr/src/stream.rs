//! The data plane: one session's stream of screen repaints.
//!
//! herdr exposes this as a CLI stream rather than a socket method, so attaching
//! means spawning `herdr terminal session control <pane>` and speaking NDJSON
//! over its stdio.
//!
//! # Frames are diffs
//!
//! Only the first frame after an attach or a resize repaints the whole screen.
//! Every frame after it is a delta that assumes its predecessors were applied,
//! so a consumer must feed **every** frame, in order, to **one** emulator, and
//! must never skip one to catch up. [`Frames`] enforces exactly that: it
//! refuses a stream that does not start with a full repaint, and refuses a gap,
//! a repeat, or a rewind in the sequence. A broken stream surfaces as an error
//! rather than as a screen that looks plausible and is wrong.
//!
//! # Reading and writing are two halves
//!
//! [`Frames::next_frame`] blocks until herdr has something to say, which for an idle
//! session is "never". Anything that also has to deliver a keystroke the
//! instant it is typed cannot hold both ends on one thread, so [`Attachment`]
//! splits: the reader goes to a thread of its own, the [`Input`] half stays
//! with whatever is driving the session. The child outlives whichever half is
//! dropped first, and dies when both are gone.

use std::{
    io::{BufRead, BufReader, Write},
    process::{Child, ChildStdin, ChildStdout, Command, Stdio},
    sync::{Arc, Mutex},
};

use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};

use crate::{
    Error, Geometry, Namespace, PaneId, Result, Sidecar, control,
    protocol::{RawFrame, StreamCommand, StreamMessage},
};

/// One repaint, decoded and ready to feed to a VT parser.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Frame {
    /// ANSI bytes. Feed them to the emulator verbatim.
    pub bytes: Vec<u8>,
    /// Whether this frame repaints the whole screen rather than a region of it.
    pub full: bool,
    /// herdr's monotonic counter.
    pub seq: u64,
    /// The geometry this frame was painted for. After a resize, the first frame
    /// carrying the new geometry is also the one that repaints in full.
    pub geometry: Geometry,
}

/// The child process, shared by both halves so that dropping one does not end
/// the attachment the other is still using.
#[derive(Debug)]
struct Process(Mutex<Child>);

impl Drop for Process {
    fn drop(&mut self) {
        if let Ok(mut child) = self.0.lock() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}

/// A live attachment to one session.
#[derive(Debug)]
pub struct Attachment {
    frames: Frames,
    input: Input,
}

impl Attachment {
    /// Attach to `pane` at `geometry`.
    ///
    /// The geometry is passed at attach time rather than sent afterwards so
    /// that the very first full repaint is already the right size — a terminal
    /// that opens at 80×24 and corrects itself a frame later is a visible flash.
    pub fn open(
        sidecar: &Sidecar,
        namespace: &Namespace,
        pane: &PaneId,
        geometry: Geometry,
    ) -> Result<Self> {
        let mut command = Command::new(sidecar.path());
        command
            .args(["terminal", "session", "control", &pane.0])
            .args(["--cols".to_owned(), geometry.cols.to_string()])
            .args(["--rows".to_owned(), geometry.rows.to_string()])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null());
        control::apply(&mut command, namespace);

        let mut child = command.spawn()?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| Error::Protocol("herdr's frame stream has no stdout".to_owned()))?;
        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| Error::Protocol("herdr's frame stream has no stdin".to_owned()))?;

        let process = Arc::new(Process(Mutex::new(child)));
        Ok(Self {
            frames: Frames {
                reader: BufReader::new(stdout),
                sequence: Sequence::default(),
                _process: process.clone(),
            },
            input: Input { stdin, _process: process },
        })
    }

    /// Hand the two halves out, so the reader can go to its own thread.
    pub fn split(self) -> (Frames, Input) {
        (self.frames, self.input)
    }
}

/// The frame contract, as a state machine of its own.
///
/// Split out from [`Frames`] so the rule can be tested without a child process
/// — and so there is exactly one place that decides whether a frame is safe to
/// paint.
#[derive(Debug, Default)]
struct Sequence {
    /// The `seq` the next frame must carry. `None` before the first one, which
    /// is also the one that must be a full repaint.
    expected: Option<u64>,
}

impl Sequence {
    /// Check a frame against the contract, and decode it if it holds.
    fn accept(&mut self, raw: RawFrame) -> Result<Frame> {
        match self.expected {
            None if !raw.full => {
                return Err(Error::Protocol(format!(
                    "the stream opened with a diff (seq {}) instead of a full repaint",
                    raw.seq
                )));
            }
            Some(expected) if raw.seq != expected => {
                return Err(Error::Protocol(format!(
                    "frame {} arrived where {expected} was due; the screen would be wrong",
                    raw.seq
                )));
            }
            _ => {}
        }
        self.expected = Some(raw.seq + 1);

        let bytes = BASE64
            .decode(raw.bytes.as_bytes())
            .map_err(|err| Error::Protocol(format!("frame {} is not base64: {err}", raw.seq)))?;
        Ok(Frame {
            bytes,
            full: raw.full,
            seq: raw.seq,
            geometry: Geometry::new(raw.width, raw.height),
        })
    }
}

/// The reading half: repaints, in order, or an error.
#[derive(Debug)]
pub struct Frames {
    reader: BufReader<ChildStdout>,
    sequence: Sequence,
    _process: Arc<Process>,
}

impl Frames {
    /// Block until the next repaint arrives.
    ///
    /// `Ok(None)` means herdr closed the stream cleanly — the session ended, or
    /// something else took it over. That is an outcome, not a failure.
    pub fn next_frame(&mut self) -> Result<Option<Frame>> {
        loop {
            let mut line = String::new();
            if self.reader.read_line(&mut line)? == 0 {
                return Ok(None);
            }
            if line.trim().is_empty() {
                continue;
            }
            let message: StreamMessage = serde_json::from_str(&line)
                .map_err(|err| Error::Protocol(format!("unreadable frame: {err}")))?;
            return match message {
                StreamMessage::Closed(_) => Ok(None),
                StreamMessage::Frame(raw) => self.sequence.accept(raw).map(Some),
            };
        }
    }
}

/// The writing half: keystrokes and geometry.
#[derive(Debug)]
pub struct Input {
    stdin: ChildStdin,
    _process: Arc<Process>,
}

impl Input {
    /// Write raw bytes to the session's PTY.
    pub fn send(&mut self, bytes: &[u8]) -> Result<()> {
        self.write(&StreamCommand::Input { bytes: BASE64.encode(bytes) })
    }

    /// Resize the PTY, which delivers `SIGWINCH` to whatever is running in it.
    ///
    /// The next frame after this will be a full repaint at the new size.
    pub fn resize(&mut self, geometry: Geometry) -> Result<()> {
        self.write(&StreamCommand::Resize { cols: geometry.cols, rows: geometry.rows })
    }

    /// Detach cleanly, leaving the session running for the next attach.
    pub fn release(&mut self) -> Result<()> {
        self.write(&StreamCommand::Release)
    }

    fn write(&mut self, command: &StreamCommand) -> Result<()> {
        let mut line = serde_json::to_vec(command)
            .map_err(|err| Error::Protocol(format!("cannot encode a stream command: {err}")))?;
        line.push(b'\n');
        self.stdin.write_all(&line)?;
        self.stdin.flush()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    //! The frame contract is the part worth testing without a backend.
    //! Attaching for real needs a live daemon and lives in the ignored smoke
    //! tests.

    use super::*;

    fn raw(seq: u64, full: bool) -> RawFrame {
        RawFrame { bytes: BASE64.encode(b"hi"), full, seq, width: 80, height: 24 }
    }

    #[test]
    fn a_stream_must_open_with_a_full_repaint() {
        let err = Sequence::default().accept(raw(0, false)).expect_err("a diff cannot be first");
        assert!(err.to_string().contains("full repaint"), "{err}");
    }

    #[test]
    fn frames_in_order_are_accepted_and_decoded() {
        let mut sequence = Sequence::default();
        let first = sequence.accept(raw(0, true)).expect("first frame");
        assert_eq!(first.bytes, b"hi");
        assert_eq!(first.geometry, Geometry::new(80, 24));
        sequence.accept(raw(1, false)).expect("second frame");
        sequence.accept(raw(2, false)).expect("third frame");
    }

    #[test]
    fn a_gap_is_an_error_and_not_a_wrong_screen() {
        let mut sequence = Sequence::default();
        sequence.accept(raw(0, true)).expect("first frame");
        let err = sequence.accept(raw(2, false)).expect_err("frame 1 never arrived");
        assert!(err.to_string().contains("frame 2 arrived where 1 was due"), "{err}");
    }

    #[test]
    fn a_repeat_is_rejected_too() {
        let mut sequence = Sequence::default();
        sequence.accept(raw(0, true)).expect("first frame");
        sequence.accept(raw(1, false)).expect("second frame");
        assert!(sequence.accept(raw(1, false)).is_err());
    }

    #[test]
    fn a_stream_may_resume_from_any_sequence_number() {
        // A re-attach does not restart herdr's counter, so the contract is
        // "starts full, then contiguous" and not "starts at zero".
        let mut sequence = Sequence::default();
        sequence.accept(raw(9_001, true)).expect("re-attach repaints in full");
        sequence.accept(raw(9_002, false)).expect("and continues from there");
    }

    #[test]
    fn a_frame_that_is_not_base64_names_itself() {
        let mut bad = raw(0, true);
        bad.bytes = "not base64!!".to_owned();
        let err = Sequence::default().accept(bad).expect_err("undecodable");
        assert!(err.to_string().contains("frame 0 is not base64"), "{err}");
    }
}
