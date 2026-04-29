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
    /**
     * verifyChain は監査ハッシュチェーンの整合性を検証する（FR-T1-AUDIT-002）。
     * fromDate / toDate に未指定（undefined）を渡すと全範囲を対象にする。
     */
    verifyChain(fromDate?: Date, toDate?: Date): Promise<VerifyChainResult>;
}
/** VerifyChain（FR-T1-AUDIT-002）の応答を SDK 利用者向けに整理した型。 */
export interface VerifyChainResult {
    /** チェーン整合性が取れていれば true。 */
    valid: boolean;
    /** 検証対象だったイベント件数。 */
    checkedCount: number;
    /** 不整合検出時、最初に失敗した sequence_number（1-based）。valid 時は 0。 */
    firstBadSequence: number;
    /** 不整合の理由。valid 時は空文字。 */
    reason: string;
}
//# sourceMappingURL=audit.d.ts.map