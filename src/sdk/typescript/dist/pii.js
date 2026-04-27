// 本ファイルは k1s0 TypeScript SDK の Pii 動詞統一 facade。
import { createPromiseClient } from "@connectrpc/connect";
import { PiiService } from "./proto/k1s0/tier1/pii/v1/pii_service_connect.js";
/** PiiFacade は PiiService の動詞統一 facade。 */
export class PiiFacade {
    client;
    constructor(client) {
        this.client = client;
    }
    /** classify は PII 種別の検出。 */
    async classify(text) {
        const raw = createPromiseClient(PiiService, this.client.transport);
        const resp = await raw.classify({ text, context: this.client.tenantContext() });
        return { findings: resp.findings, containsPii: resp.containsPii };
    }
    /** mask はマスキング。 */
    async mask(text) {
        const raw = createPromiseClient(PiiService, this.client.transport);
        const resp = await raw.mask({ text, context: this.client.tenantContext() });
        return { maskedText: resp.maskedText, findings: resp.findings };
    }
}
//# sourceMappingURL=pii.js.map