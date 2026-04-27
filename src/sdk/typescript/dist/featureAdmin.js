// 本ファイルは k1s0 TypeScript SDK の FeatureAdmin 動詞統一 facade。
import { createPromiseClient } from "@connectrpc/connect";
import { FeatureAdminService } from "./proto/k1s0/tier1/feature/v1/feature_service_connect.js";
/** FeatureAdminFacade は FeatureAdminService の動詞統一 facade。 */
export class FeatureAdminFacade {
    client;
    constructor(client) {
        this.client = client;
    }
    /** registerFlag は Flag 定義の登録（permission 種別は approvalId 必須）。 */
    async registerFlag(flag, changeReason, approvalId = "") {
        const raw = createPromiseClient(FeatureAdminService, this.client.transport);
        const resp = await raw.registerFlag({
            flag,
            changeReason,
            approvalId,
            context: this.client.tenantContext(),
        });
        return resp.version;
    }
    /** getFlag は Flag 定義の取得。version 省略で最新。 */
    async getFlag(flagKey, version) {
        const raw = createPromiseClient(FeatureAdminService, this.client.transport);
        const resp = await raw.getFlag({
            flagKey,
            version,
            context: this.client.tenantContext(),
        });
        return { flag: resp.flag, version: resp.version };
    }
    /** listFlags は Flag 定義の一覧。 */
    async listFlags(kind, state) {
        const raw = createPromiseClient(FeatureAdminService, this.client.transport);
        const resp = await raw.listFlags({
            kind,
            state,
            context: this.client.tenantContext(),
        });
        return resp.flags;
    }
}
//# sourceMappingURL=featureAdmin.js.map