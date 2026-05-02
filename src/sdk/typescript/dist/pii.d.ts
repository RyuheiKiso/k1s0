import type { K1s0Client } from "./client.js";
import type { PiiFinding } from "./proto/k1s0/tier1/pii/v1/pii_service_pb.js";
/** PiiFacade は PiiService の動詞統一 facade。 */
export declare class PiiFacade {
    private readonly client;
    constructor(client: K1s0Client);
    /** classify は PII 種別の検出。 */
    classify(text: string): Promise<{
        findings: PiiFinding[];
        containsPii: boolean;
    }>;
    /** mask はマスキング。 */
    mask(text: string): Promise<{
        maskedText: string;
        findings: PiiFinding[];
    }>;
    /**
     * pseudonymize は FR-T1-PII-002（決定論的仮名化）の facade。
     * 同一 salt + 同一 fieldType + 同一 value で同一の URL-safe base64 仮名値を返す。
     * salt / value / fieldType いずれかが空文字の場合は server 側で InvalidArgument を返す。
     */
    pseudonymize(fieldType: string, value: string, salt: string): Promise<string>;
}
//# sourceMappingURL=pii.d.ts.map