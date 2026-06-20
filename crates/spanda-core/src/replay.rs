//! Deterministic mission trace recording and replay for simulation runs.
//!
use crate::error::SpandaError;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// One recorded simulation frame for deterministic replay.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TraceFrame {
    pub sim_time_ms: f64,
    pub event: String,
    #[serde(default)]
    pub payload: serde_json::Value,
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
        self.frames.push(TraceFrame {
            sim_time_ms,
            event: event.into(),
            payload,
        });
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
