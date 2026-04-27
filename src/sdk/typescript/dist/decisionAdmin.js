// 本ファイルは k1s0 TypeScript SDK の DecisionAdmin 動詞統一 facade。
import { createPromiseClient } from "@connectrpc/connect";
import { DecisionAdminService } from "./proto/k1s0/tier1/decision/v1/decision_service_connect.js";
/** DecisionAdminFacade は DecisionAdminService の動詞統一 facade。 */
export class DecisionAdminFacade {
    client;
    constructor(client) {
        this.client = client;
    }
    /** registerRule は JDM 文書の登録。 */
    async registerRule(ruleId, jdmDocument, sigstoreSignature, commitHash) {
        const raw = createPromiseClient(DecisionAdminService, this.client.transport);
        const resp = await raw.registerRule({
            ruleId,
            jdmDocument,
            sigstoreSignature,
            commitHash,
            context: this.client.tenantContext(),
        });
        return { ruleVersion: resp.ruleVersion, effectiveAtMs: resp.effectiveAtMs };
    }
    /** listVersions はバージョン一覧。 */
    async listVersions(ruleId) {
        const raw = createPromiseClient(DecisionAdminService, this.client.transport);
        const resp = await raw.listVersions({ ruleId, context: this.client.tenantContext() });
        return resp.versions;
    }
    /** getRule は特定バージョンの取得。 */
    async getRule(ruleId, ruleVersion) {
        const raw = createPromiseClient(DecisionAdminService, this.client.transport);
        const resp = await raw.getRule({ ruleId, ruleVersion, context: this.client.tenantContext() });
        return { jdmDocument: resp.jdmDocument, meta: resp.meta };
    }
}
//# sourceMappingURL=decisionAdmin.js.map