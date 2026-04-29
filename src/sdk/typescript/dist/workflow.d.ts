import type { K1s0Client } from "./client.js";
import { type GetStatusResponse } from "./proto/k1s0/tier1/workflow/v1/workflow_service_pb.js";
/** WorkflowFacade は WorkflowService の動詞統一 facade。 */
export declare class WorkflowFacade {
    private readonly client;
    constructor(client: K1s0Client);
    private raw;
    /** start はワークフロー開始。backend hint は BACKEND_AUTO（tier1 が振り分け）。
     * 返り値は { workflowId, runId }。
     * 短期 / 長期で意図的に振り分けたい時は runShort / runLong を使う。 */
    start(workflowType: string, workflowId: string, input: Uint8Array, idempotent?: boolean): Promise<{
        workflowId: string;
        runId: string;
    }>;
    /** runShort は短期ワークフロー（≤7 日、BACKEND_DAPR）として開始する（FR-T1-WORKFLOW-001）。
     * 短期ワークフローは Dapr Workflow building block で実行され、Pod 再起動でも履歴が保持される。 */
    runShort(workflowType: string, workflowId: string, input: Uint8Array, idempotent?: boolean): Promise<{
        workflowId: string;
        runId: string;
    }>;
    /** runLong は長期ワークフロー（上限なし、BACKEND_TEMPORAL）として開始する（FR-T1-WORKFLOW-002）。
     * Continue-as-New / cron / 高度な signal 機能が必要な場合に使う。 */
    runLong(workflowType: string, workflowId: string, input: Uint8Array, idempotent?: boolean): Promise<{
        workflowId: string;
        runId: string;
    }>;
    /** startWithBackend は start / runShort / runLong の共通実装。 */
    private startWithBackend;
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