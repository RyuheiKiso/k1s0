import type { HealthCheck } from './index.js';

export interface HttpHealthCheckOptions {
  /** URL to check via HTTP GET. */
  url: string;
  /** Timeout in milliseconds (default: 5000). */
  timeoutMs?: number;
  /** Name for this health check (default: "http"). */
  name?: string;
}

export class HttpHealthCheck implements HealthCheck {
  readonly name: string;
  private readonly url: string;
  private readonly timeoutMs: number;

  constructor(options: HttpHealthCheckOptions) {
    this.name = options.name ?? 'http';
    this.url = options.url;
    this.timeoutMs = options.timeoutMs ?? 5000;
  }

  async check(): Promise<void> {
    const controller = new AbortController();
    const timer = setTimeout(() => controller.abort(), this.timeoutMs);

    try {
      const response = await fetch(this.url, {
        method: 'GET',
        signal: controller.signal,
      });

      if (!response.ok) {
        throw new Error(
          `HTTP ${this.url} returned status ${response.status}`,
        );
      }
    } catch (err) {
      if (err instanceof DOMException && err.name === 'AbortError') {
        throw new Error(`HTTP check timeout: ${this.url}`);
      }
      throw err;
    } finally {
      clearTimeout(timer);
    }
  }
}
