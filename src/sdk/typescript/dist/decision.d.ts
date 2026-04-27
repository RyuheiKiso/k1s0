import type { K1s0Client } from "./client.js";
/** DecisionFacade は DecisionService（評価）の動詞統一 facade。 */
export declare class DecisionFacade {
    private readonly client;
    constructor(client: K1s0Client);
    /** evaluate はルール評価（同期）。返り値は { outputJson, traceJson, elapsedUs }。 */
    evaluate(ruleId: string, ruleVersion: string, inputJson: Uint8Array, includeTrace?: boolean): Promise<{
        outputJson: Uint8Array;
        traceJson: Uint8Array;
        elapsedUs: bigint;
    }>;
    /** batchEvaluate はバッチ評価。 */
    batchEvaluate(ruleId: string, ruleVersion: string, inputs: Uint8Array[]): Promise<Uint8Array[]>;
}
//# sourceMappingURL=decision.d.ts.map