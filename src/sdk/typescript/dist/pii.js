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
    /**
     * pseudonymize は FR-T1-PII-002（決定論的仮名化）の facade。
     * 同一 salt + 同一 fieldType + 同一 value で同一の URL-safe base64 仮名値を返す。
     * salt / value / fieldType いずれかが空文字の場合は server 側で InvalidArgument を返す。
     */
    async pseudonymize(fieldType, value, salt) {
        const raw = createPromiseClient(PiiService, this.client.transport);
        const resp = await raw.pseudonymize({
            fieldType,
            value,
            salt,
            context: this.client.tenantContext(),
        });
        return resp.pseudonym;
    }
}
//# sourceMappingURL=pii.js.map