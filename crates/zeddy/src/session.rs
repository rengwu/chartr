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
    stream::Input,
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
    input: Input,
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
                                // A full repaint after a resize is measured
                                // against a grid of its own size, so the
                                // emulator follows the frame rather than the
                                // window: applying an 80-column repaint to a
                                // 120-column grid would wrap it wrongly.
                                let mut terminal = terminal.lock().expect("terminal mutex");
                                if frame.full {
                                    terminal.resize(size_of(frame.geometry));
                                }
                                terminal.feed(&frame.bytes);
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

        Ok(Self { info, terminal, ended, input, size })
    }

    pub fn id(&self) -> &PaneId {
        &self.info.id
    }

    /// The screen as it stands. Cheap enough to call once per paint.
    pub fn screen(&self) -> Screen {
        self.terminal.lock().expect("terminal mutex").screen()
    }

    /// Whether the reader has stopped, and why.
    pub fn ended(&self) -> Option<Ended> {
        self.ended.lock().expect("ended mutex").clone()
    }

    /// The title to show: whatever the program set, else what herdr called it.
    pub fn title(&self) -> String {
        self.terminal
            .lock()
            .expect("terminal mutex")
            .screen()
            .title
            .filter(|title| !title.trim().is_empty())
            .unwrap_or_else(|| self.info.title.clone())
    }

    /// Send typed bytes to the session.
    pub fn send(&mut self, bytes: &[u8]) -> zeddy_herdr::Result<()> {
        self.input.send(bytes)
    }

    /// Tell the session how many cells it now has.
    ///
    /// A no-op at the same size, because a resize costs a full repaint and the
    /// window recomputes its cell count on every layout pass.
    pub fn resize(&mut self, size: Size) -> zeddy_herdr::Result<()> {
        if size == self.size {
            return Ok(());
        }
        self.size = size;
        self.input.resize(geometry(size))
    }

    /// Detach cleanly, leaving the session running for the next launch.
    pub fn release(&mut self) {
        let _ = self.input.release();
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_two_grid_types_round_trip() {
        assert_eq!(size_of(geometry(Size::new(120, 40))), Size::new(120, 40));
    }
}
