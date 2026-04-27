import type { K1s0Client } from "./client.js";
import type { GetStatusResponse } from "./proto/k1s0/tier1/workflow/v1/workflow_service_pb.js";
/** WorkflowFacade は WorkflowService の動詞統一 facade。 */
export declare class WorkflowFacade {
    private readonly client;
    constructor(client: K1s0Client);
    private raw;
    /** start はワークフロー開始。返り値は { workflowId, runId }。 */
    start(workflowType: string, workflowId: string, input: Uint8Array, idempotent?: boolean): Promise<{
        workflowId: string;
        runId: string;
    }>;
    /** signal はシグナル送信。 */
    signal(workflowId: string, signalName: string, payload: Uint8Array): Promise<void>;
    /** query はワークフロー状態のクエリ（副作用なし）。 */
    query(workflowId: string, queryName: string, payload: Uint8Array): Promise<Uint8Array>;
    /** cancel は正常終了の依頼。 */
    cancel(workflowId: string, reason: string): Promise<void>;
    /** terminate は強制終了。 */
    terminate(workflowId: string, reason: string): Promise<void>;
    /** getStatus は現在状態 / runId / 出力 / エラーを取得する。 */
    getStatus(workflowId: string): Promise<GetStatusResponse>;
}
//# sourceMappingURL=workflow.d.ts.map