// 本ファイルは k1s0 TypeScript SDK の Audit 動詞統一 facade。
import { createPromiseClient } from "@connectrpc/connect";
import type { K1s0Client } from "./client.js";
import { AuditService } from "./proto/k1s0/tier1/audit/v1/audit_service_connect.js";
import { AuditEvent } from "./proto/k1s0/tier1/audit/v1/audit_service_pb.js";
import { Timestamp } from "@bufbuild/protobuf";

/** AuditFacade は AuditService の動詞統一 facade。 */
export class AuditFacade {
  private readonly client: K1s0Client;

  constructor(client: K1s0Client) {
    this.client = client;
  }

  /** record は監査イベント記録。auditId を返す。 */
  async record(
    actor: string,
    action: string,
    resource: string,
    outcome: string,
    attributes: Record<string, string> = {},
  ): Promise<string> {
    const raw = createPromiseClient(AuditService, this.client.transport);
    const resp = await raw.record({
      event: new AuditEvent({
        timestamp: Timestamp.now(),
        actor,
        action,
        resource,
        outcome,
        attributes,
      }),
      context: this.client.tenantContext(),
    });
    return resp.auditId;
  }

  /** query は監査イベント検索（時刻範囲 + filter）。 */
  async query(
    fromDate: Date,
    toDate: Date,
    filters: Record<string, string> = {},
    limit = 100,
  ): Promise<AuditEvent[]> {
    const raw = createPromiseClient(AuditService, this.client.transport);
    const resp = await raw.query({
      from: Timestamp.fromDate(fromDate),
      to: Timestamp.fromDate(toDate),
      filters,
      limit,
      context: this.client.tenantContext(),
    });
    return resp.events;
  }
}
