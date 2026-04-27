import type { K1s0Client } from "./client.js";
import type { FlagMetadata } from "./proto/k1s0/tier1/feature/v1/feature_service_pb.js";
/** FeatureFacade は FeatureService の動詞統一 facade。 */
export declare class FeatureFacade {
    private readonly client;
    constructor(client: K1s0Client);
    private buildReq;
    /** evaluateBoolean は boolean Flag 評価。 */
    evaluateBoolean(flagKey: string, evalCtx?: Record<string, string>): Promise<{
        value: boolean;
        metadata?: FlagMetadata;
    }>;
    /** evaluateString は string Flag 評価。 */
    evaluateString(flagKey: string, evalCtx?: Record<string, string>): Promise<{
        value: string;
        metadata?: FlagMetadata;
    }>;
    /** evaluateNumber は number Flag 評価。 */
    evaluateNumber(flagKey: string, evalCtx?: Record<string, string>): Promise<{
        value: number;
        metadata?: FlagMetadata;
    }>;
    /** evaluateObject は object Flag 評価（JSON シリアライズ済 bytes）。 */
    evaluateObject(flagKey: string, evalCtx?: Record<string, string>): Promise<{
        valueJson: Uint8Array;
        metadata?: FlagMetadata;
    }>;
}
//# sourceMappingURL=feature.d.ts.map