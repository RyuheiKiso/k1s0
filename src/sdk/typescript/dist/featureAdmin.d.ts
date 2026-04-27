import type { K1s0Client } from "./client.js";
import type { FlagDefinition, FlagKind, FlagState } from "./proto/k1s0/tier1/feature/v1/feature_service_pb.js";
/** FeatureAdminFacade は FeatureAdminService の動詞統一 facade。 */
export declare class FeatureAdminFacade {
    private readonly client;
    constructor(client: K1s0Client);
    /** registerFlag は Flag 定義の登録（permission 種別は approvalId 必須）。 */
    registerFlag(flag: FlagDefinition, changeReason: string, approvalId?: string): Promise<bigint>;
    /** getFlag は Flag 定義の取得。version 省略で最新。 */
    getFlag(flagKey: string, version?: bigint): Promise<{
        flag?: FlagDefinition;
        version: bigint;
    }>;
    /** listFlags は Flag 定義の一覧。 */
    listFlags(kind?: FlagKind, state?: FlagState): Promise<FlagDefinition[]>;
}
//# sourceMappingURL=featureAdmin.d.ts.map