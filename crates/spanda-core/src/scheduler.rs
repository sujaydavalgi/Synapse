//! Scheduler clock modes for cooperative sim-time vs wall-clock RTOS ticks.

use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

/// How the runtime advances task scheduling time.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum SchedulerClock {
    /// Discrete-event simulation (default, deterministic).
    #[default]
    Sim,
    /// Wall-clock scheduling with real sleeps between ticks.
    Wall,
}

impl SchedulerClock {
    pub fn as_str(self) -> &'static str {
        // Return a stable label for logs and telemetry.
        //
        // Parameters:
        // None.
        //
        // Returns:
        // Clock mode name.
        //
        // Options:
        // None.
        //
        // Example:
        // let label = SchedulerClock::Wall.as_str();

        match self {
            SchedulerClock::Sim => "sim",
            SchedulerClock::Wall => "wall",
        }
    }
}

/// Sleep until an absolute wall-clock deadline when in wall mode.
pub fn sleep_until(deadline: Instant) {
    // Block the current thread until the deadline is reached.
    //
    // Parameters:
    // - `deadline` — monotonic instant to wait for
    //
    // Returns:
    // Nothing.
    //
    // Options:
    // None.
    //
    // Example:
    // sleep_until(Instant::now() + Duration::from_millis(10));

    let now = Instant::now();
    if deadline > now {
        std::thread::sleep(deadline - now);
    }
}

/// Compute elapsed milliseconds between two monotonic instants.
pub fn elapsed_ms(start: Instant, end: Instant) -> f64 {
    // Convert an instant delta to milliseconds.
    //
    // Parameters:
    // - `start` — interval start
    // - `end` — interval end
    //
    // Returns:
    // Elapsed time in milliseconds.
    //
    // Options:
    // None.
    //
    // Example:
    // let ms = elapsed_ms(t0, Instant::now());

    end.duration_since(start).as_secs_f64() * 1000.0
}

/// Advance a wall-clock tick anchor by the nominal period.
pub fn advance_wall_tick(anchor: &mut Instant, period_ms: f64) -> Instant {
    // Move the next tick deadline forward by one scheduler period.
    //
    // Parameters:
    // - `anchor` — mutable next-tick deadline
    // - `period_ms` — nominal tick period in milliseconds
    //
    // Returns:
    // The deadline waited for this tick.
    //
    // Options:
    // None.
    //
    // Example:
    // let deadline = advance_wall_tick(&mut next, 10.0);

    let deadline = *anchor;
    *anchor += Duration::from_secs_f64(period_ms / 1000.0);
    deadline
}
