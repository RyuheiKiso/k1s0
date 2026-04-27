import type { K1s0Client } from "./client.js";
import type { RuleVersionMeta } from "./proto/k1s0/tier1/decision/v1/decision_service_pb.js";
/** DecisionAdminFacade は DecisionAdminService の動詞統一 facade。 */
export declare class DecisionAdminFacade {
    private readonly client;
    constructor(client: K1s0Client);
    /** registerRule は JDM 文書の登録。 */
    registerRule(ruleId: string, jdmDocument: Uint8Array, sigstoreSignature: Uint8Array, commitHash: string): Promise<{
        ruleVersion: string;
        effectiveAtMs: bigint;
    }>;
    /** listVersions はバージョン一覧。 */
    listVersions(ruleId: string): Promise<RuleVersionMeta[]>;
    /** getRule は特定バージョンの取得。 */
    getRule(ruleId: string, ruleVersion: string): Promise<{
        jdmDocument: Uint8Array;
        meta?: RuleVersionMeta;
    }>;
}
//# sourceMappingURL=decisionAdmin.d.ts.map