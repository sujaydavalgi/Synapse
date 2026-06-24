/**
 * Fetch live readiness from a deploy or fleet agent.
 * @module
 */

export type AgentReadinessResponse = {
  ok: boolean;
  mission_ready?: boolean;
  readiness?: {
    status?: string;
    mission_ready?: boolean;
    score?: { total: number; maximum: number; factors?: ReadinessReport["score"]["factors"] };
    issues?: Array<{ factor: string; severity: string; message: string }>;
    target?: string;
    robots?: string[];
  };
};

export async function fetchAgentReadiness(
  agentUrl: string,
  runtime = true,
  injectHealthFaults = false,
): Promise<AgentReadinessResponse> {
  // Description:
  //     FetchAgentReadiness.
  //
  // Inputs:
  //     agentUrl: string
  //         Caller-supplied agentUrl.
  //     runtime = true: input value
  //         Caller-supplied runtime = true.
  //     injectHealthFaults = false: input value
  //         Caller-supplied injectHealthFaults = false.
  //
  // Outputs:
  //     result: Promise<AgentReadinessResponse>
  //         Return value from `fetchAgentReadiness`.
  //
  // Example:

  //     const result = fetchAgentReadiness(agentUrl, runtime = true, injectHealthFaults = false);

  const base = agentUrl.replace(/\/$/, "");
  const query = new URLSearchParams();
  if (runtime) query.set("runtime", "true");
  if (injectHealthFaults) query.set("inject_health_faults", "true");
  const suffix = query.toString() ? `?${query.toString()}` : "";
  const response = await fetch(`${base}/v1/readiness${suffix}`, {
    headers: { Accept: "application/json" },
  });
  if (!response.ok) {
    throw new Error(`Agent readiness HTTP ${response.status}`);
  }
  return (await response.json()) as AgentReadinessResponse;
}
