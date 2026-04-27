import type { K1s0Client } from "./client.js";
import { AuditEvent } from "./proto/k1s0/tier1/audit/v1/audit_service_pb.js";
/** AuditFacade は AuditService の動詞統一 facade。 */
export declare class AuditFacade {
    private readonly client;
    constructor(client: K1s0Client);
    /** record は監査イベント記録。auditId を返す。 */
    record(actor: string, action: string, resource: string, outcome: string, attributes?: Record<string, string>): Promise<string>;
    /** query は監査イベント検索（時刻範囲 + filter）。 */
    query(fromDate: Date, toDate: Date, filters?: Record<string, string>, limit?: number): Promise<AuditEvent[]>;
}
//# sourceMappingURL=audit.d.ts.map