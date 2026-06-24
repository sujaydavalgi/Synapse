import { useCallback, useState } from "react";
import {
  evaluateReadinessSource,
  readinessDashboardFromReports,
  type ReadinessReport,
  type ReadinessDashboard,
} from "@spanda/core/readiness.js";
import { fetchAgentReadiness } from "./readiness-agent.js";

type Props = {
  source: string;
};

function statusClass(status: string): string {
  const normalized = status.toLowerCase();
  if (normalized.includes("ready") && !normalized.includes("not")) return "ready";
  if (normalized.includes("degraded")) return "degraded";
  return "not-ready";
}

export function OperationsPanel({ source }: Props) {
  const [report, setReport] = useState<ReadinessReport | null>(null);
  const [dashboard, setDashboard] = useState<ReadinessDashboard | null>(null);
  const [agentUrl, setAgentUrl] = useState("");
  const [agentReport, setAgentReport] = useState<ReadinessReport | null>(null);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [includeRuntime, setIncludeRuntime] = useState(true);
  const [injectFaults, setInjectFaults] = useState(false);

  const handleEvaluate = useCallback(() => {
    setError(null);
    try {
      const result = evaluateReadinessSource(source, {
        includeRuntime,
        injectHealthFaults: injectFaults,
      });
      setReport(result);
      setDashboard(readinessDashboardFromReports([result]));
      setAgentReport(null);
    } catch (e) {
      setError(String(e));
      setReport(null);
    }
  }, [source, includeRuntime, injectFaults]);

  const handleAgentFetch = useCallback(async () => {
    if (!agentUrl.trim()) {
      setError("Enter an agent base URL (e.g. http://127.0.0.1:8787)");
      return;
    }
    setBusy(true);
    setError(null);
    try {
      const body = await fetchAgentReadiness(agentUrl.trim(), includeRuntime, injectFaults);
      const readiness = body.readiness;
      if (!readiness) {
        throw new Error("Agent returned no readiness payload");
      }
      setAgentReport({
        status: (readiness.status as ReadinessReport["status"]) ?? "Unknown",
        mission_ready: readiness.mission_ready ?? body.mission_ready ?? false,
        score: {
          total: readiness.score?.total ?? 0,
          maximum: readiness.score?.maximum ?? 100,
          factors: readiness.score?.factors ?? [],
        },
        issues: (readiness.issues ?? []).map((issue) => ({
          factor: issue.factor,
          severity: issue.severity as ReadinessReport["issues"][0]["severity"],
          message: issue.message,
        })),
        target: readiness.target,
        robots: readiness.robots ?? [],
      });
      setDashboard(null);
    } catch (e) {
      setError(String(e));
      setAgentReport(null);
    } finally {
      setBusy(false);
    }
  }, [agentUrl, includeRuntime, injectFaults]);

  const active = agentReport ?? report;

  return (
    <div className="operations">
      <div className="toolbar operations-toolbar">
        <label className="checkbox">
          <input
            type="checkbox"
            checked={includeRuntime}
            onChange={(e) => setIncludeRuntime(e.target.checked)}
          />
          Runtime health
        </label>
        <label className="checkbox">
          <input
            type="checkbox"
            checked={injectFaults}
            onChange={(e) => setInjectFaults(e.target.checked)}
          />
          Inject health faults
        </label>
        <button type="button" onClick={handleEvaluate} disabled={busy}>
          Evaluate readiness
        </button>
        <input
          type="url"
          className="agent-url"
          placeholder="Agent URL for live /v1/readiness"
          value={agentUrl}
          onChange={(e) => setAgentUrl(e.target.value)}
        />
        <button type="button" className="primary" onClick={handleAgentFetch} disabled={busy}>
          Fetch from agent
        </button>
      </div>

      {error && <div className="error">{error}</div>}

      {active && (
        <div className="panel operations-summary">
          <h2>Operational readiness</h2>
          <dl>
            <dt>Status</dt>
            <dd>
              <span className={`readiness-badge ${statusClass(active.status)}`}>{active.status}</span>
            </dd>
            <dt>Mission ready</dt>
            <dd>{active.mission_ready ? "YES" : "NO"}</dd>
            <dt>Score</dt>
            <dd>
              {active.score.total}/{active.score.maximum}
            </dd>
            {active.target && (
              <>
                <dt>Target</dt>
                <dd>{active.target}</dd>
              </>
            )}
            {active.robots.length > 0 && (
              <>
                <dt>Robots</dt>
                <dd>{active.robots.join(", ")}</dd>
              </>
            )}
          </dl>
        </div>
      )}

      {dashboard && !agentReport && (
        <div className="panel">
          <h2>Fleet dashboard</h2>
          <dl>
            <dt>Overall score</dt>
            <dd>{dashboard.overall_score}/100</dd>
            <dt>Mission ready</dt>
            <dd>{dashboard.mission_ready_count}</dd>
            <dt>Degraded</dt>
            <dd>{dashboard.degraded_count}</dd>
            <dt>Not ready</dt>
            <dd>{dashboard.not_ready_count}</dd>
          </dl>
          {dashboard.top_issues.length > 0 && (
            <>
              <h3>Top issues</h3>
              <ul>
                {dashboard.top_issues.map((issue, i) => (
                  <li key={i}>{issue}</li>
                ))}
              </ul>
            </>
          )}
        </div>
      )}

      {active && active.score.factors.length > 0 && (
        <div className="panel">
          <h2>Factor scores</h2>
          <table className="factor-table">
            <thead>
              <tr>
                <th>Factor</th>
                <th>Score</th>
                <th>Weight</th>
              </tr>
            </thead>
            <tbody>
              {active.score.factors.map((factor) => (
                <tr key={factor.factor}>
                  <td>{factor.factor}</td>
                  <td>{factor.score}</td>
                  <td>{factor.weight}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}

      {active && active.issues.length > 0 && (
        <div className="panel">
          <h2>Issues</h2>
          <ul>
            {active.issues.map((issue, i) => (
              <li key={i}>
                [{issue.severity}] {issue.factor}: {issue.message}
              </li>
            ))}
          </ul>
        </div>
      )}

      {agentReport && (
        <p className="agent-hint">Showing live agent readiness (overrides local evaluation display).</p>
      )}
    </div>
  );
}
