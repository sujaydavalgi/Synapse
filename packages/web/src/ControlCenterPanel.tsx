import { useCallback, useEffect, useState } from "react";
import {
  DigitalThreadGraph,
  type DigitalThreadDeviceLink,
  type DigitalThreadGraphEdge,
  type DigitalThreadGraphNode,
} from "./DigitalThreadGraph";

type DashboardData = {
  device_pool: {
    total: number;
    healthy: number;
    active?: number;
    degraded: number;
    discovered: number;
    quarantined?: number;
    failed: number;
  };
  fleet_agent_count: number;
  alert_count: number;
};

type FleetAgent = {
  robot_name: string;
  url: string;
  token?: string;
};

type DeviceEntry = {
  id: string;
  device_type: string;
  lifecycle_state: string;
  assigned_robot?: string;
  logical_name?: string;
  trust_level?: string;
};

type RobotEntry = {
  id: string;
  model?: string;
  hardware_profile?: string;
};

type FleetEntry = {
  id: string;
  robot_count: number;
};

type ReadinessImpact = {
  mission_ready: boolean;
  impact: {
    blocked_count: number;
    total_devices: number;
  };
};

type Tab =
  | "dashboard"
  | "devices"
  | "fleet"
  | "discovery"
  | "provisioning"
  | "mapping"
  | "config"
  | "health"
  | "readiness"
  | "drift"
  | "alerts"
  | "security"
  | "ota"
  | "sre"
  | "compliance"
  | "audit"
  | "digital-thread"
  | "executive"
  | "traceability";

type SreSummary = {
  availability_percent: number;
  incidents_open?: number;
  mttr_hint_ms?: number | null;
  mtbf_hint_ms?: number | null;
  slo?: { target_percent?: number; met?: boolean };
  health_trends?: {
    degraded_percent?: number;
    failed_percent?: number;
    offline_percent?: number;
  };
  readiness_trends?: { sample_count?: number; warnings?: string[] };
};

type IncidentRow = {
  id: string;
  title: string;
  status: string;
  severity: string;
};

type Props = {
  apiBase: string;
};

export function ControlCenterPanel({ apiBase }: Props) {
  const [tab, setTab] = useState<Tab>("dashboard");
  const [dashboard, setDashboard] = useState<DashboardData | null>(null);
  const [agents, setAgents] = useState<FleetAgent[]>([]);
  const [devices, setDevices] = useState<DeviceEntry[]>([]);
  const [robots, setRobots] = useState<RobotEntry[]>([]);
  const [fleets, setFleets] = useState<FleetEntry[]>([]);
  const [mapping, setMapping] = useState<Record<string, unknown> | null>(null);
  const [readiness, setReadiness] = useState<ReadinessImpact | null>(null);
  const [deviceDetail, setDeviceDetail] = useState<Record<string, unknown> | null>(null);
  const [selectedDevice, setSelectedDevice] = useState<string | null>(null);
  const [selectedRobot, setSelectedRobot] = useState<string>("");
  const [discoveryLog, setDiscoveryLog] = useState<string | null>(null);
  const [provisionLog, setProvisionLog] = useState<string | null>(null);
  const [sreSummary, setSreSummary] = useState<SreSummary | null>(null);
  const [incidents, setIncidents] = useState<IncidentRow[]>([]);
  const [driftData, setDriftData] = useState<{
    baselineId: string;
    report: Record<string, unknown>;
  } | null>(null);
  const [alertsList, setAlertsList] = useState<Record<string, unknown>[]>([]);
  const [trustReport, setTrustReport] = useState<Record<string, unknown> | null>(null);
  const [rbacMatrix, setRbacMatrix] = useState<Record<string, unknown> | null>(null);
  const [trustPackageName, setTrustPackageName] = useState("spanda-mqtt");
  const [otaStatus, setOtaStatus] = useState<Record<string, unknown> | null>(null);
  const [complianceExport, setComplianceExport] = useState<Record<string, unknown> | null>(null);
  const [complianceProfile, setComplianceProfile] = useState("defense");
  const [complianceProfiles, setComplianceProfiles] = useState<
    { name: string; description?: string; verified?: boolean }[]
  >([]);
  const [auditData, setAuditData] = useState<Record<string, unknown> | null>(null);
  const [scorecard, setScorecard] = useState<Record<string, unknown> | null>(null);
  const [digitalThread, setDigitalThread] = useState<Record<string, unknown> | null>(null);
  const [threadCapabilityFilter, setThreadCapabilityFilter] = useState("");
  const [threadDeviceFilter, setThreadDeviceFilter] = useState("");
  const [threadLifecycleFilter, setThreadLifecycleFilter] = useState("");
  const [selectedThreadNode, setSelectedThreadNode] = useState<string | null>(null);
  const [configApprovals, setConfigApprovals] = useState<Record<string, unknown>[]>([]);
  const [evidenceRecords, setEvidenceRecords] = useState<Record<string, unknown>[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  const base = apiBase.replace(/\/$/, "");
  const apiKey =
    (import.meta as { env?: { VITE_SPANDA_API_KEY?: string } }).env?.VITE_SPANDA_API_KEY ?? "";

  const authHeaders = (): HeadersInit => {
    const headers: Record<string, string> = { "Content-Type": "application/json" };
    if (apiKey) headers.Authorization = `Bearer ${apiKey}`;
    return headers;
  };

  const load = useCallback(async () => {
    setBusy(true);
    setError(null);
    try {
      const [dashRes, fleetRes, devRes, robotRes, fleetListRes, treeRes] =
        await Promise.all([
          fetch(`${base}/v1/dashboard`),
          fetch(`${base}/v1/fleet/agents`),
          fetch(`${base}/v1/devices`),
          fetch(`${base}/v1/robots`),
          fetch(`${base}/v1/fleets`),
          fetch(`${base}/v1/device-tree`),
        ]);
      if (!dashRes.ok) throw new Error(`dashboard ${dashRes.status}`);
      setDashboard(await dashRes.json());
      if (fleetRes.ok) {
        const fleetBody = await fleetRes.json();
        setAgents(fleetBody.agents ?? []);
      }
      if (devRes.ok) {
        const devBody = await devRes.json();
        setDevices(devBody.devices ?? []);
      }
      if (robotRes.ok) {
        const robotBody = await robotRes.json();
        const nextRobots = robotBody.robots ?? [];
        setRobots(nextRobots);
        if (!selectedRobot && nextRobots.length > 0) {
          setSelectedRobot(nextRobots[0].id);
        }
      }
      if (fleetListRes.ok) {
        const fleetBody = await fleetListRes.json();
        setFleets(fleetBody.fleets ?? []);
      }
      if (treeRes.ok) {
        const treeBody = await treeRes.json();
        setMapping(treeBody.mapping ?? null);
      }
    } catch (e) {
      setError(String(e));
    } finally {
      setBusy(false);
    }
  }, [base, selectedRobot]);

  useEffect(() => {
    void load();
  }, [load]);

  const robotId = selectedRobot || robots[0]?.id || "rover-001";

  const runReadiness = async () => {
    setBusy(true);
    try {
      const res = await fetch(`${base}/v1/readiness/run`, { method: "POST" });
      if (!res.ok) throw new Error(`readiness ${res.status}`);
      setReadiness(await res.json());
    } catch (e) {
      setError(String(e));
    } finally {
      setBusy(false);
    }
  };

  const runDiscovery = async () => {
    setBusy(true);
    setDiscoveryLog(null);
    try {
      const res = await fetch(`${base}/v1/devices/discover`, {
        method: "POST",
        headers: authHeaders(),
        body: JSON.stringify({
          transports: ["mdns", "subnet", "ble", "usb", "can", "mqtt", "ros2"],
          timeout_ms: 2000,
        }),
      });
      if (!res.ok) throw new Error(`discover ${res.status}`);
      const body = await res.json();
      setDiscoveryLog(JSON.stringify(body, null, 2));
      await load();
    } catch (e) {
      setError(String(e));
    } finally {
      setBusy(false);
    }
  };

  const inspectDevice = async (id: string) => {
    setSelectedDevice(id);
    setTab("provisioning");
    setProvisionLog(null);
    try {
      const res = await fetch(`${base}/v1/devices/${encodeURIComponent(id)}`);
      if (res.ok) {
        const body = await res.json();
        setDeviceDetail(body.device ?? null);
      }
    } catch (e) {
      setError(String(e));
    }
  };

  const provisionDevice = async () => {
    if (!selectedDevice) return;
    setBusy(true);
    setProvisionLog(null);
    try {
      const robot = robotId;
      const res = await fetch(
        `${base}/v1/devices/${encodeURIComponent(selectedDevice)}/provision`,
        {
          method: "POST",
          headers: authHeaders(),
          body: JSON.stringify({ robot_id: robot }),
        },
      );
      const body = await res.json();
      setProvisionLog(JSON.stringify(body.report ?? body, null, 2));
      await load();
      await inspectDevice(selectedDevice);
    } catch (e) {
      setError(String(e));
    } finally {
      setBusy(false);
    }
  };

  const quarantineDevice = async () => {
    if (!selectedDevice) return;
    setBusy(true);
    try {
      const res = await fetch(
        `${base}/v1/devices/${encodeURIComponent(selectedDevice)}/quarantine`,
        { method: "POST", headers: authHeaders() },
      );
      if (!res.ok) throw new Error(`quarantine ${res.status}`);
      await load();
      await inspectDevice(selectedDevice);
    } catch (e) {
      setError(String(e));
    } finally {
      setBusy(false);
    }
  };

  const assignDevice = async () => {
    if (!selectedDevice) return;
    setBusy(true);
    try {
      const robot = robotId;
      const res = await fetch(
        `${base}/v1/devices/${encodeURIComponent(selectedDevice)}/assign`,
        {
          method: "POST",
          headers: authHeaders(),
          body: JSON.stringify({ robot_id: robot }),
        },
      );
      if (!res.ok) throw new Error(`assign ${res.status}`);
      await load();
      await inspectDevice(selectedDevice);
    } catch (e) {
      setError(String(e));
    } finally {
      setBusy(false);
    }
  };

  const trustDevice = async () => {
    if (!selectedDevice) return;
    setBusy(true);
    try {
      const res = await fetch(
        `${base}/v1/devices/${encodeURIComponent(selectedDevice)}/trust`,
        { method: "POST", headers: authHeaders() },
      );
      if (!res.ok) throw new Error(`trust ${res.status}`);
      await load();
      await inspectDevice(selectedDevice);
    } catch (e) {
      setError(String(e));
    } finally {
      setBusy(false);
    }
  };

  const loadSre = async () => {
    setBusy(true);
    try {
      const [summaryRes, incidentsRes] = await Promise.all([
        fetch(`${base}/v1/sre/summary`),
        fetch(`${base}/v1/sre/incidents`),
      ]);
      if (!summaryRes.ok) throw new Error(`sre summary ${summaryRes.status}`);
      setSreSummary(await summaryRes.json());
      if (incidentsRes.ok) {
        const body = await incidentsRes.json();
        setIncidents(body.incidents ?? []);
      }
    } catch (e) {
      setError(String(e));
    } finally {
      setBusy(false);
    }
  };

  const createIncident = async () => {
    if (!apiKey) return;
    setBusy(true);
    try {
      const res = await fetch(`${base}/v1/sre/incidents`, {
        method: "POST",
        headers: authHeaders(),
        body: JSON.stringify({
          title: "Control Center incident",
          description: "Opened from @spanda/web panel",
          severity: "warning",
        }),
      });
      if (!res.ok) throw new Error(`create incident ${res.status}`);
      await loadSre();
    } catch (e) {
      setError(String(e));
    } finally {
      setBusy(false);
    }
  };

  const ackIncident = async (incidentId: string) => {
    if (!apiKey) return;
    setBusy(true);
    try {
      const res = await fetch(`${base}/v1/sre/incidents/${encodeURIComponent(incidentId)}/ack`, {
        method: "POST",
        headers: authHeaders(),
        body: JSON.stringify({ assignee: "operator" }),
      });
      if (!res.ok) throw new Error(`ack incident ${res.status}`);
      await loadSre();
    } catch (e) {
      setError(String(e));
    } finally {
      setBusy(false);
    }
  };

  const resolveIncident = async (incidentId: string) => {
    if (!apiKey) return;
    setBusy(true);
    try {
      const res = await fetch(
        `${base}/v1/sre/incidents/${encodeURIComponent(incidentId)}/resolve`,
        { method: "POST", headers: authHeaders() },
      );
      if (!res.ok) throw new Error(`resolve incident ${res.status}`);
      await loadSre();
    } catch (e) {
      setError(String(e));
    } finally {
      setBusy(false);
    }
  };

  useEffect(() => {
    if (tab === "sre") void loadSre();
    if (tab === "drift") void loadDrift();
    if (tab === "alerts") void loadAlerts();
    if (tab === "security") void loadSecurity();
    if (tab === "ota") void loadOta();
    if (tab === "compliance") void loadComplianceProfiles();
    if (tab === "audit") void loadAudit();
    if (tab === "executive") void loadExecutive();
    if (tab === "digital-thread") void loadDigitalThread();
    if (tab === "config") void loadConfig();
  }, [tab, base, trustPackageName, apiKey]);

  const loadConfig = async () => {
    setBusy(true);
    try {
      const approvalsRes = await fetch(`${base}/v1/config/approvals`);
      if (approvalsRes.ok) {
        const body = await approvalsRes.json();
        setConfigApprovals(body.approvals ?? []);
      }
    } catch (e) {
      setError(String(e));
    } finally {
      setBusy(false);
    }
  };

  const requestConfigApproval = async (snapshotId: string) => {
    if (!apiKey) return;
    setBusy(true);
    try {
      const res = await fetch(`${base}/v1/config/approvals`, {
        method: "POST",
        headers: authHeaders(),
        body: JSON.stringify({
          snapshot_id: snapshotId,
          note: "Control Center approval request",
        }),
      });
      if (!res.ok) throw new Error(`approval request ${res.status}`);
      await loadConfig();
    } catch (e) {
      setError(String(e));
    } finally {
      setBusy(false);
    }
  };

  const resolveConfigApproval = async (approvalId: string, approve: boolean) => {
    if (!apiKey) return;
    setBusy(true);
    try {
      const action = approve ? "approve" : "reject";
      const res = await fetch(
        `${base}/v1/config/approvals/${encodeURIComponent(approvalId)}/${action}`,
        { method: "POST", headers: authHeaders(), body: JSON.stringify({}) },
      );
      if (!res.ok) throw new Error(`approval ${action} ${res.status}`);
      await loadConfig();
    } catch (e) {
      setError(String(e));
    } finally {
      setBusy(false);
    }
  };

  const loadDrift = async () => {
    setBusy(true);
    try {
      const snapsRes = await fetch(`${base}/v1/config/snapshots`);
      if (!snapsRes.ok) throw new Error(`snapshots ${snapsRes.status}`);
      const snaps = await snapsRes.json();
      const first = (snaps.snapshots ?? [])[0];
      if (!first?.id) {
        setDriftData(null);
        return;
      }
      const driftRes = await fetch(
        `${base}/v1/drift?baseline_id=${encodeURIComponent(first.id)}`,
      );
      if (!driftRes.ok) throw new Error(`drift ${driftRes.status}`);
      const body = await driftRes.json();
      setDriftData({ baselineId: first.id, report: body.report ?? body });
    } catch (e) {
      setError(String(e));
    } finally {
      setBusy(false);
    }
  };

  const loadAlerts = async () => {
    setBusy(true);
    try {
      const res = await fetch(`${base}/v1/alerts`);
      if (!res.ok) throw new Error(`alerts ${res.status}`);
      const body = await res.json();
      setAlertsList(body.alerts ?? []);
    } catch (e) {
      setError(String(e));
    } finally {
      setBusy(false);
    }
  };

  const loadSecurity = async () => {
    setBusy(true);
    try {
      const [trustRes, rbacRes] = await Promise.all([
        fetch(`${base}/v1/trust/package?name=${encodeURIComponent(trustPackageName)}`),
        fetch(`${base}/v1/rbac/matrix`),
      ]);
      if (!trustRes.ok) throw new Error(`trust ${trustRes.status}`);
      if (!rbacRes.ok) throw new Error(`rbac ${rbacRes.status}`);
      setTrustReport(await trustRes.json());
      setRbacMatrix(await rbacRes.json());
    } catch (e) {
      setError(String(e));
    } finally {
      setBusy(false);
    }
  };

  const loadOta = async () => {
    setBusy(true);
    try {
      const res = await fetch(`${base}/v1/ota/status`);
      if (!res.ok) throw new Error(`ota ${res.status}`);
      setOtaStatus(await res.json());
    } catch (e) {
      setError(String(e));
    } finally {
      setBusy(false);
    }
  };

  const loadCompliance = async () => {
    if (!apiKey) return;
    setBusy(true);
    try {
      const [exportRes, evidenceRes] = await Promise.all([
        fetch(`${base}/v1/compliance/export?profile=${encodeURIComponent(complianceProfile)}`, {
          headers: authHeaders(),
        }),
        fetch(`${base}/v1/compliance/evidence`, { headers: authHeaders() }),
      ]);
      if (!exportRes.ok) throw new Error(`compliance ${exportRes.status}`);
      setComplianceExport(await exportRes.json());
      if (evidenceRes.ok) {
        const evidenceBody = await evidenceRes.json();
        setEvidenceRecords(evidenceBody.evidence ?? []);
      }
    } catch (e) {
      setError(String(e));
    } finally {
      setBusy(false);
    }
  };

  const loadComplianceProfiles = async () => {
    try {
      const res = await fetch(`${base}/v1/compliance/profiles`);
      if (!res.ok) throw new Error(`profiles ${res.status}`);
      const body = await res.json();
      const signed = Array.isArray(body.signed_catalog) ? body.signed_catalog : [];
      const builtin = Array.isArray(body.profiles) ? body.profiles : [];
      const merged = signed.length
        ? signed.map((entry: { name: string; description?: string; verified?: boolean }) => ({
            name: entry.name,
            description: entry.description,
            verified: entry.verified,
          }))
        : builtin.map((name: string) => ({ name }));
      setComplianceProfiles(merged);
      if (merged.length && !merged.some((entry) => entry.name === complianceProfile)) {
        setComplianceProfile(merged[0]?.name ?? "defense");
      }
    } catch (e) {
      setError(String(e));
    }
  };

  const loadAudit = async () => {
    if (!apiKey) return;
    setBusy(true);
    try {
      const res = await fetch(`${base}/v1/audit/mutations`, { headers: authHeaders() });
      if (!res.ok) throw new Error(`audit ${res.status}`);
      setAuditData(await res.json());
    } catch (e) {
      setError(String(e));
    } finally {
      setBusy(false);
    }
  };

  const loadExecutive = async () => {
    setBusy(true);
    try {
      const res = await fetch(`${base}/v1/executive/scorecard`);
      if (!res.ok) throw new Error(`scorecard ${res.status}`);
      const body = await res.json();
      setScorecard(body.scorecard ?? body);
    } catch (e) {
      setError(String(e));
    } finally {
      setBusy(false);
    }
  };

  const loadDigitalThread = async () => {
    setBusy(true);
    try {
      const params = new URLSearchParams();
      if (threadCapabilityFilter.trim()) {
        params.set("capability", threadCapabilityFilter.trim());
      }
      if (threadDeviceFilter.trim()) {
        params.set("device_id", threadDeviceFilter.trim());
      }
      if (threadLifecycleFilter.trim()) {
        params.set("lifecycle_phase", threadLifecycleFilter.trim());
      }
      const query = params.toString();
      const res = await fetch(
        `${base}/v1/digital-thread/query${query ? `?${query}` : ""}`,
      );
      if (!res.ok) throw new Error(`digital-thread ${res.status}`);
      const body = await res.json();
      setDigitalThread(body.digital_thread ?? body);
      setSelectedThreadNode(null);
    } catch (e) {
      setError(String(e));
    } finally {
      setBusy(false);
    }
  };

  const exportReports = async () => {
    setBusy(true);
    try {
      const res = await fetch(`${base}/v1/device-reports`);
      if (!res.ok) throw new Error(`reports ${res.status}`);
      const body = await res.json();
      const blob = new Blob([JSON.stringify(body.reports ?? body, null, 2)], {
        type: "application/json",
      });
      const url = URL.createObjectURL(blob);
      const a = document.createElement("a");
      a.href = url;
      a.download = "spanda-device-reports.json";
      a.click();
      URL.revokeObjectURL(url);
    } catch (e) {
      setError(String(e));
    } finally {
      setBusy(false);
    }
  };

  const pool = dashboard?.device_pool;

  const threadLifecycleRows =
    (digitalThread?.lifecycle_rows as { node_id: string; phase: string }[] | undefined) ?? [];
  const threadGraphNodes = (digitalThread?.graph as { nodes?: DigitalThreadGraphNode[] })
    ?.nodes ?? [];
  const threadGraphEdges = (digitalThread?.graph as { edges?: DigitalThreadGraphEdge[] })
    ?.edges ?? [];
  const threadDeviceLinks =
    (digitalThread?.device_links as DigitalThreadDeviceLink[] | undefined) ?? [];

  const tabs: Tab[] = [
    "dashboard",
    "devices",
    "fleet",
    "discovery",
    "provisioning",
    "mapping",
    "config",
    "health",
    "readiness",
    "drift",
    "alerts",
    "security",
    "ota",
    "sre",
    "compliance",
    "audit",
    "digital-thread",
    "executive",
    "traceability",
  ];

  return (
    <div className="panel control-center">
      <h2>Spanda Control Center</h2>
      <p className="demo-hint">
        API: <code>{base}</code> — run <code>spanda control-center serve --config spanda.toml</code>
      </p>
      <div className="toolbar">
        {tabs.map((name) => (
          <button
            key={name}
            type="button"
            className={tab === name ? "primary" : undefined}
            onClick={() => setTab(name)}
          >
            {name.charAt(0).toUpperCase() + name.slice(1)}
          </button>
        ))}
        <button type="button" onClick={() => void load()} disabled={busy}>
          Refresh
        </button>
      </div>
      {error && <div className="error">{error}</div>}

      {tab === "dashboard" && pool && (
        <dl>
          <dt>Devices</dt>
          <dd>{pool.total}</dd>
          <dt>Active / Healthy</dt>
          <dd>{pool.active ?? pool.healthy}</dd>
          <dt>Discovered</dt>
          <dd>{pool.discovered}</dd>
          <dt>Quarantined</dt>
          <dd>{pool.quarantined ?? 0}</dd>
          <dt>Fleet agents</dt>
          <dd>{dashboard?.fleet_agent_count ?? 0}</dd>
          <dt>Alerts</dt>
          <dd>{dashboard?.alert_count ?? 0}</dd>
        </dl>
      )}

      {tab === "devices" && (
        <table>
          <thead>
            <tr>
              <th>ID</th>
              <th>Type</th>
              <th>Lifecycle</th>
              <th>Trust</th>
              <th>Robot</th>
              <th>Logical</th>
            </tr>
          </thead>
          <tbody>
            {devices.map((d) => (
              <tr key={d.id}>
                <td>
                  <button type="button" onClick={() => void inspectDevice(d.id)}>
                    {d.id}
                  </button>
                </td>
                <td>{d.device_type}</td>
                <td>{d.lifecycle_state}</td>
                <td>{d.trust_level ?? "unknown"}</td>
                <td>{d.assigned_robot ?? "—"}</td>
                <td>{d.logical_name ?? "—"}</td>
              </tr>
            ))}
          </tbody>
        </table>
      )}

      {tab === "fleet" && (
        <>
          <h3>Fleets</h3>
          <ul>
            {fleets.map((f) => (
              <li key={f.id}>
                <strong>{f.id}</strong> — {f.robot_count} robots
              </li>
            ))}
          </ul>
          <h3>Robots</h3>
          <ul>
            {robots.map((r) => (
              <li key={r.id}>
                <strong>{r.id}</strong>
                {r.hardware_profile ? ` (${r.hardware_profile})` : ""}
              </li>
            ))}
          </ul>
          <h3>Agents</h3>
          <ul>
            {agents.length === 0 && <li>No fleet agents registered</li>}
            {agents.map((a) => (
              <li key={a.robot_name}>
                <strong>{a.robot_name}</strong> — {a.url}
              </li>
            ))}
          </ul>
        </>
      )}

      {tab === "discovery" && (
        <div>
          <p>Run multi-transport discovery (mDNS, subnet, BLE, USB, CAN, MQTT, ROS2 stubs).</p>
          <button type="button" onClick={() => void runDiscovery()} disabled={busy}>
            Discover devices
          </button>
          {discoveryLog && <pre>{discoveryLog}</pre>}
        </div>
      )}

      {tab === "provisioning" && (
        <div>
          <p>
            Provisioning: discover → verify → trust → firmware → health → capabilities → assign →
            ready.
          </p>
          {!apiKey && (
            <p className="demo-hint">
              Set <code>VITE_SPANDA_API_KEY</code> for provision/assign/trust/quarantine mutations.
            </p>
          )}
          {selectedDevice ? (
            <>
              <p>
                Selected: <code>{selectedDevice}</code>
              </p>
              <label>
                Target robot{" "}
                <select
                  value={robotId}
                  onChange={(event) => setSelectedRobot(event.target.value)}
                >
                  {robots.map((robot) => (
                    <option key={robot.id} value={robot.id}>
                      {robot.id}
                    </option>
                  ))}
                  {robots.length === 0 && <option value="rover-001">rover-001</option>}
                </select>
              </label>
              <div className="toolbar">
                <button type="button" onClick={() => void trustDevice()} disabled={busy || !apiKey}>
                  Trust / Approve
                </button>
                <button type="button" onClick={() => void provisionDevice()} disabled={busy || !apiKey}>
                  Provision
                </button>
                <button type="button" onClick={() => void assignDevice()} disabled={busy || !apiKey}>
                  Assign to fleet
                </button>
                <button type="button" onClick={() => void quarantineDevice()} disabled={busy || !apiKey}>
                  Quarantine
                </button>
              </div>
              {deviceDetail && <pre>{JSON.stringify(deviceDetail, null, 2)}</pre>}
              {provisionLog && <pre>{provisionLog}</pre>}
            </>
          ) : (
            <p>Select a device from the Devices tab.</p>
          )}
        </div>
      )}

      {tab === "mapping" && (
        <div>
          <button type="button" onClick={() => void exportReports()} disabled={busy}>
            Export device reports
          </button>
          {mapping && <pre>{JSON.stringify(mapping, null, 2)}</pre>}
        </div>
      )}

      {tab === "health" && pool && (
        <dl>
          <dt>Healthy / Active</dt>
          <dd>{pool.active ?? pool.healthy}</dd>
          <dt>Degraded</dt>
          <dd>{pool.degraded}</dd>
          <dt>Failed</dt>
          <dd>{pool.failed}</dd>
        </dl>
      )}

      {tab === "readiness" && (
        <div>
          <button type="button" onClick={() => void runReadiness()} disabled={busy}>
            Run readiness check
          </button>
          {readiness && (
            <dl>
              <dt>Mission ready</dt>
              <dd>{readiness.mission_ready ? "yes" : "no"}</dd>
              <dt>Blocked devices</dt>
              <dd>{readiness.impact.blocked_count}</dd>
            </dl>
          )}
          {pool && (
            <ul>
              {devices.map((d) => (
                <li key={d.id}>
                  {d.id} ({d.lifecycle_state})
                </li>
              ))}
            </ul>
          )}
        </div>
      )}

      {tab === "sre" && sreSummary && (
        <div>
          <dl>
            <dt>Availability %</dt>
            <dd>{sreSummary.availability_percent.toFixed(1)}</dd>
            <dt>SLO target %</dt>
            <dd>{(sreSummary.slo?.target_percent ?? 99).toFixed(1)}</dd>
            <dt>SLO met</dt>
            <dd>{sreSummary.slo?.met ? "yes" : "no"}</dd>
            <dt>Open incidents</dt>
            <dd>{sreSummary.incidents_open ?? 0}</dd>
            <dt>MTTR hint (ms)</dt>
            <dd>{sreSummary.mttr_hint_ms ?? "—"}</dd>
            <dt>MTBF hint (ms)</dt>
            <dd>{sreSummary.mtbf_hint_ms ?? "—"}</dd>
            {sreSummary.health_trends && (
              <>
                <dt>Degraded %</dt>
                <dd>{sreSummary.health_trends.degraded_percent?.toFixed(1) ?? "—"}</dd>
                <dt>Failed %</dt>
                <dd>{sreSummary.health_trends.failed_percent?.toFixed(1) ?? "—"}</dd>
              </>
            )}
            {sreSummary.readiness_trends && (
              <>
                <dt>Readiness samples</dt>
                <dd>{sreSummary.readiness_trends.sample_count ?? 0}</dd>
              </>
            )}
          </dl>
          <button type="button" onClick={() => void createIncident()} disabled={busy || !apiKey}>
            Open incident
          </button>
          {!apiKey && (
            <p className="demo-hint">
              Set <code>VITE_SPANDA_API_KEY</code> to ack/resolve incidents.
            </p>
          )}
          <ul>
            {incidents.length === 0 && <li>No incidents</li>}
            {incidents.map((incident) => (
              <li key={incident.id}>
                <strong>{incident.title}</strong> — {incident.status} ({incident.severity})
                {incident.status === "open" && apiKey && (
                  <button type="button" onClick={() => void ackIncident(incident.id)} disabled={busy}>
                    Ack
                  </button>
                )}
                {incident.status !== "resolved" && apiKey && (
                  <button
                    type="button"
                    onClick={() => void resolveIncident(incident.id)}
                    disabled={busy}
                  >
                    Resolve
                  </button>
                )}
              </li>
            ))}
          </ul>
        </div>
      )}

      {tab === "drift" && (
        <div>
          {!driftData && <p>Save a config snapshot first (`POST /v1/config/snapshots`).</p>}
          {driftData && (
            <>
              <p>
                Baseline: <code>{driftData.baselineId}</code> — passed:{" "}
                <strong>{String(driftData.report.passed ?? "—")}</strong>
              </p>
              <h3>By dimension</h3>
              <dl>
                {Object.entries(
                  (driftData.report.by_dimension as Record<string, number>) ?? {},
                ).map(([name, count]) => (
                  <div key={name}>
                    <dt>{name}</dt>
                    <dd>{count}</dd>
                  </div>
                ))}
              </dl>
              <pre>{JSON.stringify(driftData.report, null, 2)}</pre>
              {apiKey && (
                <button
                  type="button"
                  onClick={() => void requestConfigApproval(driftData.baselineId)}
                  disabled={busy}
                >
                  Request publish approval
                </button>
              )}
            </>
          )}
        </div>
      )}

      {tab === "config" && (
        <div>
          <h3>Approval queue</h3>
          {!apiKey && (
            <p className="demo-hint">
              Set <code>VITE_SPANDA_API_KEY</code> with Approve role to resolve requests.
            </p>
          )}
          <ul>
            {configApprovals.length === 0 && <li>No approval requests</li>}
            {configApprovals.map((approval) => (
              <li key={String(approval.id)}>
                <code>{String(approval.id)}</code> — snapshot{" "}
                <code>{String(approval.snapshot_id)}</code> — {String(approval.status)}
                {approval.status === "pending" && apiKey && (
                  <>
                    <button
                      type="button"
                      onClick={() => void resolveConfigApproval(String(approval.id), true)}
                      disabled={busy}
                    >
                      Approve
                    </button>
                    <button
                      type="button"
                      onClick={() => void resolveConfigApproval(String(approval.id), false)}
                      disabled={busy}
                    >
                      Reject
                    </button>
                  </>
                )}
              </li>
            ))}
          </ul>
        </div>
      )}

      {tab === "alerts" && (
        <ul>
          {alertsList.length === 0 && <li>No alerts</li>}
          {alertsList.map((alert) => (
            <li key={String(alert.id)}>
              <strong>{String(alert.severity)}</strong> — {String(alert.message)} (
              {String(alert.source)})
            </li>
          ))}
        </ul>
      )}

      {tab === "security" && (
        <div>
          <label>
            Package{" "}
            <input
              value={trustPackageName}
              onChange={(event) => setTrustPackageName(event.target.value)}
            />
          </label>
          <button type="button" onClick={() => void loadSecurity()} disabled={busy}>
            Evaluate trust
          </button>
          {trustReport && (
            <>
              <h3>Package trust</h3>
              <pre>{JSON.stringify(trustReport, null, 2)}</pre>
            </>
          )}
          {rbacMatrix && (
            <>
              <h3>RBAC matrix</h3>
              <pre>{JSON.stringify(rbacMatrix.matrix ?? rbacMatrix, null, 2)}</pre>
            </>
          )}
        </div>
      )}

      {tab === "ota" && (
        <div>
          <p>Plan rollouts via <code>POST /v1/ota/plan</code>.</p>
          {otaStatus && <pre>{JSON.stringify(otaStatus, null, 2)}</pre>}
        </div>
      )}

      {tab === "compliance" && (
        <div>
          <div className="digital-thread-filters">
            <label>
              Profile
              <select
                value={complianceProfile}
                onChange={(event) => setComplianceProfile(event.target.value)}
              >
                {(complianceProfiles.length
                  ? complianceProfiles
                  : [{ name: "defense" }, { name: "medical" }, { name: "iso26262" }]
                ).map((entry) => (
                  <option key={entry.name} value={entry.name}>
                    {entry.name}
                    {entry.verified ? " (signed)" : ""}
                  </option>
                ))}
              </select>
            </label>
            <button type="button" onClick={() => void loadCompliance()} disabled={busy || !apiKey}>
              Export profile
            </button>
          </div>
          {!apiKey && (
            <p className="demo-hint">
              Set <code>VITE_SPANDA_API_KEY</code> for compliance export.
            </p>
          )}
          {complianceExport && <pre>{JSON.stringify(complianceExport, null, 2)}</pre>}
          {evidenceRecords.length > 0 && (
            <>
              <h3>Immutable evidence log</h3>
              <pre>{JSON.stringify(evidenceRecords, null, 2)}</pre>
            </>
          )}
        </div>
      )}

      {tab === "audit" && (
        <div>
          <button type="button" onClick={() => void loadAudit()} disabled={busy || !apiKey}>
            Load mutation audit
          </button>
          {auditData && <pre>{JSON.stringify(auditData, null, 2)}</pre>}
        </div>
      )}

      {tab === "executive" && scorecard && (
        <pre>{JSON.stringify(scorecard, null, 2)}</pre>
      )}

      {tab === "digital-thread" && digitalThread && (
        <div>
          <div className="digital-thread-filters">
            <label>
              Capability
              <input
                value={threadCapabilityFilter}
                onChange={(event) => setThreadCapabilityFilter(event.target.value)}
                placeholder="e.g. navigate"
              />
            </label>
            <label>
              Device id
              <input
                value={threadDeviceFilter}
                onChange={(event) => setThreadDeviceFilter(event.target.value)}
                placeholder="e.g. gps-001"
              />
            </label>
            <label>
              Lifecycle phase
              <select
                value={threadLifecycleFilter}
                onChange={(event) => setThreadLifecycleFilter(event.target.value)}
              >
                <option value="">All phases</option>
                <option value="requirement">Requirement</option>
                <option value="design">Design</option>
                <option value="deploy">Deploy</option>
                <option value="operate">Operate</option>
                <option value="retire">Retire</option>
              </select>
            </label>
            <button type="button" onClick={() => void loadDigitalThread()} disabled={busy}>
              Query
            </button>
          </div>
          <p className="demo-hint">
            {String(digitalThread.matched_node_count ?? 0)} nodes,{" "}
            {String(digitalThread.matched_edge_count ?? 0)} edges
            {threadLifecycleRows.length > 0
              ? ` — lifecycle phases tracked: ${Object.keys(
                  (digitalThread.lifecycle_summary as Record<string, number>) ?? {},
                ).join(", ")}`
              : ""}{" "}
            — click a node to highlight neighbors
          </p>
          <div className="digital-thread-legend">
            <span className="legend-mission">Mission</span>
            <span className="legend-robot">Robot</span>
            <span className="legend-capability">Capability</span>
            <span className="legend-hardware">Hardware</span>
            <span className="legend-provider">Provider</span>
            <span className="legend-package">Package</span>
            <span className="legend-safety">Safety</span>
          </div>
          <DigitalThreadGraph
            nodes={threadGraphNodes}
            edges={threadGraphEdges}
            deviceLinks={threadDeviceLinks}
            lifecycleRows={threadLifecycleRows}
            selectedId={selectedThreadNode}
            onSelectNode={setSelectedThreadNode}
          />
          <h3>Chain summary</h3>
          <ul>
            {((digitalThread.chain_summary as string[]) ?? []).map((step) => (
              <li key={step}>{step}</li>
            ))}
          </ul>
          <details>
            <summary>Raw report JSON</summary>
            <pre>{JSON.stringify(digitalThread, null, 2)}</pre>
          </details>
        </div>
      )}

      {tab === "traceability" && (
        <ul>
          {devices.map((d) => (
            <li key={d.id}>
              {d.id} — trust={d.trust_level ?? "unknown"} logical={d.logical_name ?? "—"}
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}
