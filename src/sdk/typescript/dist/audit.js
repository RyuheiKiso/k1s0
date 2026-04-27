// 本ファイルは k1s0 TypeScript SDK の Audit 動詞統一 facade。
import { createPromiseClient } from "@connectrpc/connect";
import { AuditService } from "./proto/k1s0/tier1/audit/v1/audit_service_connect.js";
import { AuditEvent } from "./proto/k1s0/tier1/audit/v1/audit_service_pb.js";
import { Timestamp } from "@bufbuild/protobuf";
/** AuditFacade は AuditService の動詞統一 facade。 */
export class AuditFacade {
    client;
    constructor(client) {
        this.client = client;
    }
    /** record は監査イベント記録。auditId を返す。 */
    async record(actor, action, resource, outcome, attributes = {}) {
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
    async query(fromDate, toDate, filters = {}, limit = 100) {
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
//# sourceMappingURL=audit.js.map