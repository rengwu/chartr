//! One live session: an attachment, an emulator, and the thread between them.
//!
//! # Why a thread and not a task
//!
//! [`Frames::next_frame`] blocks until herdr has something to say, which for an idle
//! session is never. Parking a GPUI executor task on that would hold an
//! executor thread hostage per idle session, so each attachment gets a real
//! thread of its own, and the only thing that crosses back to the window is a
//! wakeup.
//!
//! # Why the window never reads a frame
//!
//! The reader thread applies frames to the emulator itself, under a mutex, and
//! then says only "something changed". The window's job is to take a snapshot
//! when it paints. That keeps frame application off the frame path entirely: a
//! session producing a thousand repaints a second costs the window one redraw
//! per vsync, not a thousand.

use std::sync::{Arc, Mutex};

use futures::channel::mpsc;
use zeddy_herdr::{
    Geometry, PaneId,
    control::{self, Client},
    stream::{Frame, Input},
};
use zeddy_vt::{Screen, Size, Terminal};

/// A wakeup from a session's reader thread. Carries nothing: the state is in
/// the emulator, and the message only says to look at it.
pub type Wakeup = ();

/// The reader half's outcome, once it stops.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ended {
    /// herdr closed the stream: the session exited, or something else took it.
    Closed,
    /// The stream broke. The message is the one to show in the pane.
    Failed(String),
}

/// One attached session.
pub struct Session {
    pub info: control::Session,
    terminal: Arc<Mutex<Terminal>>,
    ended: Arc<Mutex<Option<Ended>>>,
    input: Arc<Mutex<Input>>,
    size: Size,
}

impl Session {
    /// Attach to a pane and start reading it.
    ///
    /// The wakeup sender is cloned per session; the app holds one receiver for
    /// all of them and redraws when any session speaks.
    pub fn attach(
        client: &Client,
        info: control::Session,
        size: Size,
        wakeups: mpsc::UnboundedSender<Wakeup>,
    ) -> zeddy_herdr::Result<Self> {
        let attachment = client.attach(&info.id, geometry(size))?;
        let (mut frames, input) = attachment.split();

        let terminal = Arc::new(Mutex::new(Terminal::new(size)));
        let ended = Arc::new(Mutex::new(None));

        std::thread::Builder::new()
            .name(format!("zeddy-session-{}", info.id))
            .spawn({
                let terminal = terminal.clone();
                let ended = ended.clone();
                move || {
                    let outcome = loop {
                        match frames.next_frame() {
                            Ok(Some(frame)) => {
                                let mut terminal = terminal.lock().expect("terminal mutex");
                                apply_frame(&mut terminal, &frame);
                            }
                            Ok(None) => break Ended::Closed,
                            Err(err) => break Ended::Failed(err.to_string()),
                        }
                        // Sent after the frame is applied, so a redraw woken by
                        // this always sees it. A closed receiver means the
                        // window is gone, and so is the reason to keep reading.
                        if wakeups.unbounded_send(()).is_err() {
                            return;
                        }
                    };
                    *ended.lock().expect("ended mutex") = Some(outcome);
                    let _ = wakeups.unbounded_send(());
                }
            })
            .expect("spawn a session reader thread");

        Ok(Self { info, terminal, ended, input: Arc::new(Mutex::new(input)), size })
    }

    pub fn id(&self) -> &PaneId {
        &self.info.id
    }

    pub fn size(&self) -> Size {
        self.size
    }

    /// The screen as it stands. Cheap enough to call once per paint.
    pub fn screen(&self) -> Screen {
        self.terminal.lock().expect("terminal mutex").screen()
    }

    /// Whether the reader has stopped, and why.
    pub fn ended(&self) -> Option<Ended> {
        self.ended.lock().expect("ended mutex").clone()
    }

    /// The live title inferred by the control plane: detected agent, foreground
    /// process, then Herdr's persistent tab label.
    pub fn title(&self) -> String {
        self.info.title().to_owned()
    }

    /// Send typed bytes to the session.
    pub fn send(&mut self, bytes: &[u8]) -> zeddy_herdr::Result<()> {
        self.input.lock().expect("session input mutex").send(bytes)
    }

    /// Tell the session how many cells it now has.
    ///
    /// A no-op at the same size, because a resize costs a full repaint and the
    /// window recomputes its cell count on every layout pass. The local grid is
    /// resized before the command crosses the process boundary, so a divider
    /// drag reflows on the next paint instead of waiting for herdr's repaint.
    pub fn resize(&mut self, size: Size) -> zeddy_herdr::Result<()> {
        if size == self.size {
            return Ok(());
        }
        self.size = size;
        self.terminal.lock().expect("terminal mutex").resize(size);
        self.input.lock().expect("session input mutex").resize(geometry(size))
    }

    /// Detach cleanly, leaving the session running for the next launch.
    pub fn release(&mut self) {
        let _ = self.input.lock().expect("session input mutex").release();
    }

    pub fn access(&self) -> SessionAccess {
        SessionAccess { info: self.info.clone(), input: self.input.clone() }
    }
}

#[derive(Clone)]
pub struct SessionAccess {
    pub info: control::Session,
    input: Arc<Mutex<Input>>,
}

impl SessionAccess {
    pub fn send(&self, bytes: &[u8]) -> zeddy_herdr::Result<()> {
        self.input.lock().expect("session input mutex").send(bytes)
    }
}

impl std::fmt::Debug for Session {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Session")
            .field("id", &self.info.id)
            .field("size", &self.size)
            .finish_non_exhaustive()
    }
}

/// The two crates below zeddy each have their own name for a grid, and neither
/// should have to know about the other. These two functions are the seam.
fn geometry(size: Size) -> Geometry {
    Geometry::new(size.cols, size.rows)
}

fn size_of(geometry: Geometry) -> Size {
    Size::new(geometry.cols, geometry.rows)
}

/// Paint a frame only when it was produced for the grid the window currently
/// owns.
///
/// Resizing the local emulator immediately leaves a small interval in which
/// herdr can still deliver frames queued for the previous geometry. Feeding
/// one of those into the new grid would wrap and position its contents against
/// the wrong width. The stream still consumes those frames to preserve its
/// sequence contract; herdr's full repaint for the current geometry resumes
/// painting.
fn apply_frame(terminal: &mut Terminal, frame: &Frame) -> bool {
    if terminal.size() != size_of(frame.geometry) {
        return false;
    }
    terminal.feed(&frame.bytes);
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_two_grid_types_round_trip() {
        assert_eq!(size_of(geometry(Size::new(120, 40))), Size::new(120, 40));
    }

    #[test]
    fn a_frame_for_the_current_grid_is_applied() {
        let mut terminal = Terminal::new(Size::new(120, 40));
        let frame = Frame {
            bytes: b"current".to_vec(),
            full: true,
            seq: 1,
            geometry: Geometry::new(120, 40),
        };

        assert!(apply_frame(&mut terminal, &frame));
        assert_eq!(terminal.screen().to_text().lines().next(), Some("current"));
    }

    #[test]
    fn a_queued_full_repaint_for_the_previous_grid_is_ignored() {
        let mut terminal = Terminal::new(Size::new(120, 40));
        let frame =
            Frame { bytes: b"stale".to_vec(), full: true, seq: 1, geometry: Geometry::new(80, 24) };

        assert!(!apply_frame(&mut terminal, &frame));
        assert_eq!(terminal.screen().to_text().lines().next(), Some(""));
        assert_eq!(terminal.size(), Size::new(120, 40));
    }
}
