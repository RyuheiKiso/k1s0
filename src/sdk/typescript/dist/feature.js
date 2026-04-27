// 本ファイルは k1s0 TypeScript SDK の Feature 動詞統一 facade（評価部のみ）。
import { createPromiseClient } from "@connectrpc/connect";
import { FeatureService } from "./proto/k1s0/tier1/feature/v1/feature_service_connect.js";
/** FeatureFacade は FeatureService の動詞統一 facade。 */
export class FeatureFacade {
    client;
    constructor(client) {
        this.client = client;
    }
    buildReq(flagKey, evalCtx) {
        return {
            flagKey,
            evaluationContext: evalCtx,
            context: this.client.tenantContext(),
        };
    }
    /** evaluateBoolean は boolean Flag 評価。 */
    async evaluateBoolean(flagKey, evalCtx = {}) {
        const raw = createPromiseClient(FeatureService, this.client.transport);
        const resp = await raw.evaluateBoolean(this.buildReq(flagKey, evalCtx));
        return { value: resp.value, metadata: resp.metadata };
    }
    /** evaluateString は string Flag 評価。 */
    async evaluateString(flagKey, evalCtx = {}) {
        const raw = createPromiseClient(FeatureService, this.client.transport);
        const resp = await raw.evaluateString(this.buildReq(flagKey, evalCtx));
        return { value: resp.value, metadata: resp.metadata };
    }
    /** evaluateNumber は number Flag 評価。 */
    async evaluateNumber(flagKey, evalCtx = {}) {
        const raw = createPromiseClient(FeatureService, this.client.transport);
        const resp = await raw.evaluateNumber(this.buildReq(flagKey, evalCtx));
        return { value: resp.value, metadata: resp.metadata };
    }
    /** evaluateObject は object Flag 評価（JSON シリアライズ済 bytes）。 */
    async evaluateObject(flagKey, evalCtx = {}) {
        const raw = createPromiseClient(FeatureService, this.client.transport);
        const resp = await raw.evaluateObject(this.buildReq(flagKey, evalCtx));
        return { valueJson: resp.valueJson, metadata: resp.metadata };
    }
}
//# sourceMappingURL=feature.js.map