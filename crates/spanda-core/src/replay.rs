//! Deterministic mission trace recording and replay for simulation runs.
//!
use crate::error::{PoseState, SpandaError, VelocityState};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// Robot state captured for frame-by-frame playback without re-running program logic.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReplayStateSnapshot {
    pub pose: PoseState,
    pub velocity: VelocityState,
    pub emergency_stop: bool,
    #[serde(default)]
    pub active_mode: Option<String>,
}

/// One recorded simulation frame for deterministic replay.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TraceFrame {
    pub sim_time_ms: f64,
    pub event: String,
    #[serde(default)]
    pub payload: serde_json::Value,
    #[serde(default)]
    pub state: Option<ReplayStateSnapshot>,
}

/// Full mission trace file format.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MissionTrace {
    pub version: u32,
    pub source: String,
    #[serde(default)]
    pub deterministic: bool,
    pub frames: Vec<TraceFrame>,
}

impl MissionTrace {
    pub fn new(source: impl Into<String>) -> Self {
        // Create an empty mission trace for a source program.
        //
        // Parameters:
        // - `source` — `.sd` file path or label
        //
        // Returns:
        // Empty trace container.
        //
        // Options:
        // None.
        //
        // Example:
        // let trace = MissionTrace::new("rover.sd");

        // Initialize metadata with an empty frame list.
        Self {
            version: 1,
            source: source.into(),
            deterministic: true,
            frames: Vec::new(),
        }
    }

    pub fn record(
        &mut self,
        sim_time_ms: f64,
        event: impl Into<String>,
        payload: serde_json::Value,
    ) {
        // Append one trace frame at the current simulation time.
        //
        // Parameters:
        // - `sim_time_ms` — simulation clock in milliseconds
        // - `event` — event label
        // - `payload` — structured payload
        //
        // Returns:
        // Nothing.
        //
        // Options:
        // None.
        //
        // Example:
        // trace.record(10.0, "task_tick", json!({"task":"sense"}));

        // Push the frame in arrival order for deterministic playback.
        self.record_with_state(sim_time_ms, event, payload, None);
    }

    pub fn record_with_state(
        &mut self,
        sim_time_ms: f64,
        event: impl Into<String>,
        payload: serde_json::Value,
        state: Option<ReplayStateSnapshot>,
    ) {
        // Append one trace frame with optional world-state snapshot.
        //
        // Parameters:
        // - `sim_time_ms` — simulation clock in milliseconds
        // - `event` — event label
        // - `payload` — structured payload
        // - `state` — optional robot snapshot for playback mode
        //
        // Returns:
        // Nothing.
        //
        // Options:
        // None.
        //
        // Example:
        // trace.record_with_state(10.0, "scheduler_tick", json!({}), Some(snapshot));

        self.frames.push(TraceFrame {
            sim_time_ms,
            event: event.into(),
            payload,
            state,
        });
        if self.frames.iter().any(|f| f.state.is_some()) {
            self.version = 2;
        }
    }

    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<(), SpandaError> {
        // Serialize the trace to JSON on disk.
        //
        // Parameters:
        // - `path` — output `.trace` file path
        //
        // Returns:
        // Ok on successful write.
        //
        // Options:
        // None.
        //
        // Example:
        // trace.save("mission.trace")?;

        // Encode as pretty JSON for human inspection and tooling.
        let json = serde_json::to_string_pretty(self).map_err(|err| SpandaError::Runtime {
            message: format!("Failed to encode trace: {err}"),
            line: 0,
        })?;
        fs::write(path, json).map_err(|err| SpandaError::Runtime {
            message: format!("Failed to write trace file: {err}"),
            line: 0,
        })
    }

    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, SpandaError> {
        // Load a mission trace from disk.
        //
        // Parameters:
        // - `path` — input `.trace` file path
        //
        // Returns:
        // Parsed mission trace.
        //
        // Options:
        // None.
        //
        // Example:
        // let trace = MissionTrace::load("mission.trace")?;

        // Read and decode the JSON trace file.
        let text = fs::read_to_string(path.as_ref()).map_err(|err| SpandaError::Runtime {
            message: format!("Failed to read trace file: {err}"),
            line: 0,
        })?;
        serde_json::from_str(&text).map_err(|err| SpandaError::Runtime {
            message: format!("Invalid trace file: {err}"),
            line: 0,
        })
    }

    pub fn frames_from(&self, offset_ms: f64) -> &[TraceFrame] {
        // Return trace frames starting at or after the requested offset.
        //
        // Parameters:
        // - `offset_ms` — replay start offset in milliseconds
        //
        // Returns:
        // Slice of frames at/after the offset.
        //
        // Options:
        // None.
        //
        // Example:
        // let slice = trace.frames_from(30_000.0);

        // Find the first frame at or after the offset timestamp.
        let idx = self
            .frames
            .iter()
            .position(|frame| frame.sim_time_ms >= offset_ms)
            .unwrap_or(self.frames.len());
        &self.frames[idx..]
    }
}

pub fn parse_replay_offset(raw: &str) -> Result<f64, SpandaError> {
    // Parse replay offset strings such as `T+00:30` into milliseconds.
    //
    // Parameters:
    // - `raw` — CLI offset argument
    //
    // Returns:
    // Offset in milliseconds.
    //
    // Options:
    // None.
    //
    // Example:
    // let ms = parse_replay_offset("T+00:30")?;

    // Accept plain millisecond values directly.
    if let Ok(ms) = raw.parse::<f64>() {
        return Ok(ms);
    }

    // Parse `T+mm:ss` or `T+hh:mm:ss` formatted offsets.
    let value = raw.strip_prefix("T+").ok_or_else(|| SpandaError::Runtime {
        message: format!("Invalid replay offset '{raw}'; expected T+mm:ss or milliseconds"),
        line: 0,
    })?;
    let parts: Vec<&str> = value.split(':').collect();
    let total_secs = match parts.as_slice() {
        [mins, secs] => {
            mins.parse::<f64>().unwrap_or(0.0) * 60.0 + secs.parse::<f64>().unwrap_or(0.0)
        }
        [hours, mins, secs] => {
            hours.parse::<f64>().unwrap_or(0.0) * 3600.0
                + mins.parse::<f64>().unwrap_or(0.0) * 60.0
                + secs.parse::<f64>().unwrap_or(0.0)
        }
        _ => {
            return Err(SpandaError::Runtime {
                message: format!("Invalid replay offset '{raw}'; expected T+mm:ss"),
                line: 0,
            })
        }
    };
    Ok(total_secs * 1000.0)
}

/// Result of comparing an expected mission trace to a fresh recorded run.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TraceVerification {
    pub ok: bool,
    pub matched: usize,
    pub mismatches: Vec<String>,
}

pub fn verify_traces(
    expected: &MissionTrace,
    actual: &MissionTrace,
    from_ms: f64,
) -> TraceVerification {
    // Compare two mission traces from the same offset for deterministic replay checks.
    //
    // Parameters:
    // - `expected` — reference trace loaded from disk
    // - `actual` — trace recorded during a replay run
    // - `from_ms` — comparison start offset in milliseconds
    //
    // Returns:
    // Verification summary with mismatched frame details.
    //
    // Options:
    // None.
    //
    // Example:
    // let report = verify_traces(&expected, &actual, 0.0);

    // Align both traces from the requested offset.
    let exp = expected.frames_from(from_ms);
    let act = actual.frames_from(from_ms);
    let mut mismatches = Vec::new();
    let shared = exp.len().min(act.len());
    for index in 0..shared {
        if exp[index].event != act[index].event {
            mismatches.push(format!(
                "frame {index}: expected event '{}', got '{}'",
                exp[index].event, act[index].event
            ));
        } else if (exp[index].sim_time_ms - act[index].sim_time_ms).abs() > 0.001 {
            mismatches.push(format!(
                "frame {index} event '{}': expected t={:.3}ms, got t={:.3}ms",
                exp[index].event, exp[index].sim_time_ms, act[index].sim_time_ms
            ));
        }
    }
    if exp.len() != act.len() {
        mismatches.push(format!(
            "frame count mismatch: expected {}, got {}",
            exp.len(),
            act.len()
        ));
    }
    TraceVerification {
        ok: mismatches.is_empty(),
        matched: shared,
        mismatches,
    }
}

/// Target that can receive replayed state snapshots during playback.
pub trait ReplayStateTarget {
    fn apply_replay_state(&mut self, snapshot: &ReplayStateSnapshot);
}

/// Summary of a frame-by-frame playback run.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlaybackReport {
    pub frames_applied: usize,
    pub states_applied: usize,
    pub events: Vec<String>,
}

pub fn playback_frames<T: ReplayStateTarget>(
    frames: &[TraceFrame],
    target: &mut T,
    wall_clock: bool,
) -> PlaybackReport {
    // Apply trace frames sequentially without executing program logic.
    //
    // Parameters:
    // - `frames` — slice of frames to play back
    // - `target` — backend receiving state snapshots
    // - `wall_clock` — sleep between frames using recorded timestamps
    //
    // Returns:
    // Playback summary with applied frame counts.
    //
    // Options:
    // None.
    //
    // Example:
    // let report = playback_frames(trace.frames_from(0.0), &mut sim, true);

    let mut states_applied = 0usize;
    let mut events = Vec::new();
    let mut prev_sim_ms = 0.0;

    for frame in frames {
        if wall_clock {
            let delta_ms = frame.sim_time_ms - prev_sim_ms;
            if delta_ms > 0.0 {
                std::thread::sleep(std::time::Duration::from_secs_f64(delta_ms / 1000.0));
            }
            prev_sim_ms = frame.sim_time_ms;
        }
        if let Some(state) = &frame.state {
            target.apply_replay_state(state);
            states_applied += 1;
        }
        events.push(frame.event.clone());
    }

    PlaybackReport {
        frames_applied: frames.len(),
        states_applied,
        events,
    }
}
