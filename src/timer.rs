//! A timer than can be paused and resumed.
use instant::Instant;
use std::time::Duration;

use gloo_timers::callback::Timeout;

// A timer than can be paused and resumed.
#[derive(Debug)]
pub struct Timer {
    pub timeout: Option<Timeout>,
    pub start: Instant,
    pub duration_left: Duration,
}

impl Timer {
    const MAX: Duration = Duration::from_secs(10);

    /// Create and immediately launch a timer.
    pub fn new<F>(duration: Duration, callback: F) -> Self
    where
        F: 'static + FnOnce(),
    {
        Self {
            timeout: Some(Timeout::new(clamp_duration(duration), callback)),
            start: Instant::now(),
            duration_left: duration,
        }
    }

    // Pause the timer. Does not take into account any duration extension
    // if `extend` was called before this.
    pub fn pause(&mut self) {
        self.timeout.take().map(Timeout::cancel);
        self.duration_left = Instant::now().duration_since(self.start);
    }

    // Extend the timer by `duration`.
    //
    // To be effective, the timer **must** be paused and `resume` must be called next.
    pub fn extend(&mut self, duration: Duration) {
        self.duration_left = self
            .duration_left
            .saturating_add(duration)
            .clamp(Duration::ZERO, Self::MAX);
    }

    // Resume the timer with the duration left since the last `pause`
    // (and possibly `extend`) call.
    pub fn resume<F>(&mut self, callback: F)
    where
        F: 'static + FnOnce(),
    {
        self.timeout = Some(Timeout::new(clamp_duration(self.duration_left), callback));
        self.start = Instant::now();
    }
}

fn clamp_duration(duration: Duration) -> u32 {
    duration.as_millis().clamp(0, u32::MAX as _) as _
}
