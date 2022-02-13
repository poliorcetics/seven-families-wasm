//! A timer that returns the time left on stop.
use instant::Instant;
use std::time::Duration;

use gloo_timers::callback::Timeout;

/// A timer that returns the time left on stop.
///
/// On `drop` it is cancelled.
#[derive(Debug)]
pub struct Timer {
    timeout: Option<Timeout>,
    start: Instant,
    duration: Duration,
}

impl Timer {
    /// Create and immediately launch a timer.
    pub fn new<F>(duration: Duration, callback: F) -> Self
    where
        F: 'static + FnOnce(),
    {
        Self {
            timeout: Some(Timeout::new(clamp_duration(duration), callback)),
            start: Instant::now(),
            duration,
        }
    }

    // Stop the timer and return the time left before it was supposed to end.
    pub fn stop(&mut self) -> Duration {
        self.timeout.take().map(Timeout::cancel);
        let elapsed = Instant::now().duration_since(self.start);
        self.duration.saturating_sub(elapsed)
    }
}

impl Drop for Timer {
    fn drop(&mut self) {
        self.timeout.take().map(Timeout::cancel);
    }
}

fn clamp_duration(duration: Duration) -> u32 {
    duration.as_millis().clamp(0, u32::MAX as _) as _
}
