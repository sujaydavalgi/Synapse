/**
 * OTLP/JSON export helpers for the persistent telemetry store.
 * @module
 */

import { readAllEvents, readHeartbeatIndexForStore } from "./telemetry-store.js";

function computeEventCounts(): Record<string, number> {
  const counts = {
    device: 0,
    sensor: 0,
    heartbeat: 0,
    device_heartbeat: 0,
    health: 0,
    session: 0,
    runtime_metrics: 0,
  };
  for (const event of readAllEvents()) {
    if (event.kind in counts) {
      counts[event.kind as keyof typeof counts] += 1;
    }
  }
  return counts;
}

/** Render OTLP/JSON metrics from the current persistent store snapshot. */
export function renderOtlpJson(): string {
  const counts = computeEventCounts();
  const index = readHeartbeatIndexForStore();
  const nowNano = `${Date.now()}000000`;
  const metrics = [
    ...Object.entries(counts).map(([kind, count]) => ({
      name: "spanda.telemetry.events",
      gauge: {
        dataPoints: [{
          asDouble: count,
          attributes: [{ key: "kind", value: { stringValue: kind } }],
          timeUnixNano: nowNano,
        }],
      },
    })),
  ];
  for (const [task, timestamp] of Object.entries(index.tasks)) {
    metrics.push({
      name: "spanda.task.heartbeat.last_timestamp_ms",
      gauge: {
        dataPoints: [{
          asDouble: timestamp,
          attributes: [{ key: "task", value: { stringValue: task } }],
          timeUnixNano: nowNano,
        }],
      },
    });
  }
  return JSON.stringify({
    resourceMetrics: [{
      resource: { attributes: [{ key: "service.name", value: { stringValue: "spanda" } }] },
      scopeMetrics: [{ scope: { name: "spanda.telemetry" }, metrics }],
    }],
  }, null, 2);
}
