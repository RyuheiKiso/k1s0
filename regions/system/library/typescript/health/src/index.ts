export type HealthStatus = 'healthy' | 'degraded' | 'unhealthy';

export interface CheckResult {
  status: HealthStatus;
  message?: string;
}

export interface HealthResponse {
  status: HealthStatus;
  checks: Record<string, CheckResult>;
  timestamp: string;
}

export interface HealthCheck {
  name: string;
  check(): Promise<void>;
}

export class HealthChecker {
  private checks: HealthCheck[] = [];

  add(check: HealthCheck): void {
    this.checks.push(check);
  }

  async runAll(): Promise<HealthResponse> {
    const results: Record<string, CheckResult> = {};
    let overallStatus: HealthStatus = 'healthy';

    for (const hc of this.checks) {
      try {
        await hc.check();
        results[hc.name] = { status: 'healthy' };
      } catch (err) {
        const message = err instanceof Error ? err.message : String(err);
        results[hc.name] = { status: 'unhealthy', message };
        overallStatus = 'unhealthy';
      }
    }

    return {
      status: overallStatus,
      checks: results,
      timestamp: new Date().toISOString(),
    };
  }
}
