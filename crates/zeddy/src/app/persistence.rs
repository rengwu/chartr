//! Coalesced workspace snapshots and ordered off-thread database writes.

use super::*;

const SAVE_INTERVAL: Duration = Duration::from_millis(250);
const RETRY_INTERVAL: Duration = Duration::from_secs(2);

impl Zeddy {
    pub(super) fn capture_window_bounds(&mut self, window: &Window) {
        let bounds = match window.window_bounds() {
            gpui::WindowBounds::Windowed(bounds)
            | gpui::WindowBounds::Maximized(bounds)
            | gpui::WindowBounds::Fullscreen(bounds) => bounds,
        };
        self.window_bounds = Some(crate::persistence::WindowBounds {
            x: bounds.origin.x / px(1.),
            y: bounds.origin.y / px(1.),
            width: bounds.size.width / px(1.),
            height: bounds.size.height / px(1.),
        });
    }

    pub(super) fn schedule_persistence(&mut self, cx: &mut Context<Self>) {
        if self.state.is_none() {
            return;
        }
        self.persistence_dirty = true;
        if self.persistence_task.is_some() {
            return;
        }
        let executor = cx.background_executor().clone();
        self.persistence_task = Some(cx.spawn(async move |this, cx| {
            let mut delay = SAVE_INTERVAL;
            loop {
                executor.timer(delay).await;
                let Ok(Some(request)) = this.update(cx, |this, cx| {
                    if !this.persistence_dirty {
                        this.persistence_task = None;
                        return None;
                    }
                    this.persistence_dirty = false;
                    let snapshot = this.snapshot(cx);
                    this.state.as_mut().map(|state| state.request(snapshot))
                }) else {
                    break;
                };
                // Only one request is in flight. Changes made while it saves
                // set the dirty bit; the next turn captures the latest state.
                let result = executor.spawn(async move { request.save() }).await;
                delay = if let Err(error) = result {
                    let _ = this.update(cx, |this, cx| {
                        this.persistence_dirty = true;
                        let problem = error.to_string();
                        if this.problem.as_ref() != Some(&problem) {
                            this.problem = Some(problem);
                            cx.notify();
                        }
                    });
                    RETRY_INTERVAL
                } else {
                    SAVE_INTERVAL
                };
            }
        }));
    }

    /// Shutdown is the only synchronous save path after initialization. GPUI's
    /// quit-future budget is only 200 ms, so flush before returning from close/
    /// quit callbacks instead of risking loss of the final coalesced changes.
    pub(crate) fn flush_state(&mut self, cx: &App) {
        self.persistence_task = None;
        self.persistence_dirty = false;
        if self.state.is_none() {
            return;
        }
        let snapshot = self.snapshot(cx);
        if let Some(state) = self.state.as_mut()
            && let Err(error) = state.request(snapshot).save()
        {
            eprintln!("Could not save workspace state on close: {error}");
            self.problem = Some(error.to_string());
        }
    }
}
