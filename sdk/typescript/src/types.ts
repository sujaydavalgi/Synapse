/** Shared SDK response types mirroring OpenAPI schemas. */

export class ReadinessReport {
  score?: number;
  status?: string;
  missionReady?: boolean;
  raw: Record<string, unknown>;

  constructor(raw: Record<string, unknown>) {
    this.raw = raw;
    const report = (raw.report as Record<string, unknown> | undefined) ?? raw;
    const scoreVal = report.score as Record<string, unknown> | number | undefined;
    if (typeof scoreVal === "number") {
      this.score = scoreVal;
    } else if (scoreVal && typeof scoreVal.total === "number") {
      this.score = scoreVal.total;
    }
    if (typeof report.status === "string") {
      this.status = report.status;
    }
    if (typeof report.mission_ready === "boolean") {
      this.missionReady = report.mission_ready;
    }
  }

  static fromApi(raw: Record<string, unknown>): ReadinessReport {
    return new ReadinessReport(raw);
  }
}

export interface Entity {
  id: string;
  kind?: string;
  entityType?: string;
  displayName?: string;
  healthStatus?: string;
  readinessStatus?: string;
  trustStatus?: string;
  lifecycleState?: string;
  raw: Record<string, unknown>;
}

export type JsonValue = Record<string, unknown>;
