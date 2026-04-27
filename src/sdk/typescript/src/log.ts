// 本ファイルは k1s0 TypeScript SDK の Log 動詞統一 facade。
import { createPromiseClient } from "@connectrpc/connect";
import type { K1s0Client } from "./client.js";
import { LogService } from "./proto/k1s0/tier1/log/v1/log_service_connect.js";
import { Severity, LogEntry } from "./proto/k1s0/tier1/log/v1/log_service_pb.js";
import { Timestamp } from "@bufbuild/protobuf";

/** LogFacade は LogService の動詞統一 facade。 */
export class LogFacade {
  private readonly client: K1s0Client;

  constructor(client: K1s0Client) {
    this.client = client;
  }

  /** send は単一エントリ送信。 */
  async send(
    severity: Severity,
    body: string,
    attributes: Record<string, string> = {},
  ): Promise<void> {
    const raw = createPromiseClient(LogService, this.client.transport);
    await raw.send({
      entry: new LogEntry({
        timestamp: Timestamp.now(),
        severity,
        body,
        attributes,
      }),
      context: this.client.tenantContext(),
    });
  }

  info = (body: string, attrs: Record<string, string> = {}) => this.send(Severity.INFO, body, attrs);
  warn = (body: string, attrs: Record<string, string> = {}) => this.send(Severity.WARN, body, attrs);
  error = (body: string, attrs: Record<string, string> = {}) => this.send(Severity.ERROR, body, attrs);
  debug = (body: string, attrs: Record<string, string> = {}) => this.send(Severity.DEBUG, body, attrs);
}
