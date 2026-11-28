//! Frame-clock animation for the naturally sized results scroller.

use relm4::gtk::{self, glib, prelude::*};
use std::{cell::Cell, rc::Rc};

const RESIZE_DURATION_US: f64 = 180_000.0;

#[derive(Debug)]
pub(super) struct ResultsHeightAnimator {
    limit: i32,
    tick: Option<gtk::TickCallbackId>,
    running: Rc<Cell<bool>>,
}

impl ResultsHeightAnimator {
    pub fn new() -> Self {
        Self {
            limit: 1,
            tick: None,
            running: Rc::new(Cell::new(false)),
        }
    }

    pub fn set_limit(&mut self, scroller: &gtk::ScrolledWindow, limit: i32) {
        self.stop(scroller);
        self.limit = limit.max(1);
        scroller.set_max_content_height(self.limit);
    }

    /// Freeze the current height while the list is replaced; a running animation
    /// contributes its interpolated height instead of the widget's last allocation.
    pub fn begin_update(&mut self, scroller: &gtk::ScrolledWindow) -> i32 {
        let height = if self.running.get() {
            scroller.min_content_height()
        } else {
            scroller.height()
        };
        self.stop(scroller);
        height
    }

    pub fn stop(&mut self, scroller: &gtk::ScrolledWindow) {
        if let Some(tick) = self.tick.take()
            && self.running.replace(false)
        {
            tick.remove();
        }
        scroller.set_min_content_height(-1);
        scroller.set_max_content_height(self.limit);
    }

    pub fn animate_to(&mut self, scroller: &gtk::ScrolledWindow, from: i32, natural_height: i32) {
        if !scroller.is_mapped()
            || gtk::Settings::default().is_some_and(|settings| !settings.is_gtk_enable_animations())
        {
            return;
        }
        let start = from.clamp(1, self.limit);
        let target = natural_height.clamp(1, self.limit);
        if start == target {
            return;
        }

        // Pin the viewport while the list's natural height changes. The parent
        // panel remains centered by GTK as the viewport grows or shrinks.
        set_height(scroller, start);
        self.running.set(true);
        let running = self.running.clone();
        let limit = self.limit;
        let started_at = Cell::new(None);
        self.tick = Some(scroller.add_tick_callback(move |scroller, clock| {
            let now = clock.frame_time();
            let first_frame = started_at.get().unwrap_or_else(|| {
                started_at.set(Some(now));
                now
            });
            let progress = ((now - first_frame) as f64 / RESIZE_DURATION_US).clamp(0.0, 1.0);
            let ease_out = 1.0 - (1.0 - progress).powi(3);
            let height = (f64::from(start) + f64::from(target - start) * ease_out).round() as i32;
            set_height(scroller, height);
            if progress >= 1.0 {
                scroller.set_min_content_height(-1);
                scroller.set_max_content_height(limit);
                running.set(false);
                glib::ControlFlow::Break
            } else {
                glib::ControlFlow::Continue
            }
        }));
    }
}

fn set_height(scroller: &gtk::ScrolledWindow, height: i32) {
    // GTK checks min <= max after each setter, not just after both have run.
    let current_max = scroller.max_content_height();
    if current_max >= 0 && height > current_max {
        scroller.set_max_content_height(height);
        scroller.set_min_content_height(height);
    } else {
        scroller.set_min_content_height(height);
        scroller.set_max_content_height(height);
    }
}
