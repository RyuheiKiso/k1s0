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
}
//# sourceMappingURL=pii.d.ts.map