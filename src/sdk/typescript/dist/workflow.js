// 本ファイルは k1s0 TypeScript SDK の Workflow 動詞統一 facade。
import { createPromiseClient } from "@connectrpc/connect";
import { WorkflowService } from "./proto/k1s0/tier1/workflow/v1/workflow_service_connect.js";
import { WorkflowBackend, } from "./proto/k1s0/tier1/workflow/v1/workflow_service_pb.js";
/** WorkflowFacade は WorkflowService の動詞統一 facade。 */
export class WorkflowFacade {
    client;
    constructor(client) {
        this.client = client;
    }
    raw() {
        return createPromiseClient(WorkflowService, this.client.transport);
    }
    /** start はワークフロー開始。backend hint は BACKEND_AUTO（tier1 が振り分け）。
     * 返り値は { workflowId, runId }。
     * 短期 / 長期で意図的に振り分けたい時は runShort / runLong を使う。 */
    async start(workflowType, workflowId, input, idempotent = false) {
        return this.startWithBackend(workflowType, workflowId, input, idempotent, WorkflowBackend.BACKEND_AUTO);
    }
    /** runShort は短期ワークフロー（≤7 日、BACKEND_DAPR）として開始する（FR-T1-WORKFLOW-001）。
     * 短期ワークフローは Dapr Workflow building block で実行され、Pod 再起動でも履歴が保持される。 */
    async runShort(workflowType, workflowId, input, idempotent = false) {
        return this.startWithBackend(workflowType, workflowId, input, idempotent, WorkflowBackend.BACKEND_DAPR);
    }
    /** runLong は長期ワークフロー（上限なし、BACKEND_TEMPORAL）として開始する（FR-T1-WORKFLOW-002）。
     * Continue-as-New / cron / 高度な signal 機能が必要な場合に使う。 */
    async runLong(workflowType, workflowId, input, idempotent = false) {
        return this.startWithBackend(workflowType, workflowId, input, idempotent, WorkflowBackend.BACKEND_TEMPORAL);
    }
    /** startWithBackend は start / runShort / runLong の共通実装。 */
    async startWithBackend(workflowType, workflowId, input, idempotent, backend) {
        const resp = await this.raw().start({
            workflowType,
            workflowId,
            input,
            idempotent,
            // 共通規約 §「冪等性と再試行」: idempotent=true なら workflow_id を
            // dedup key として転用する（同 workflow_id 再投入は新 run を作らない）。
            idempotencyKey: idempotent ? workflowId : "",
            context: this.client.tenantContext(),
            backend,
        });
        return { workflowId: resp.workflowId, runId: resp.runId };
    }
    /** signal はシグナル送信。 */
    async signal(workflowId, signalName, payload) {
        await this.raw().signal({
            workflowId,
            signalName,
            payload,
            context: this.client.tenantContext(),
        });
    }
    /** query はワークフロー状態のクエリ（副作用なし）。 */
    async query(workflowId, queryName, payload) {
        const resp = await this.raw().query({
            workflowId,
            queryName,
            payload,
            context: this.client.tenantContext(),
        });
        return resp.result;
    }
    /** cancel は正常終了の依頼。 */
    async cancel(workflowId, reason) {
        await this.raw().cancel({
            workflowId,
            reason,
            context: this.client.tenantContext(),
        });
    }
    /** terminate は強制終了。 */
    async terminate(workflowId, reason) {
        await this.raw().terminate({
            workflowId,
            reason,
            context: this.client.tenantContext(),
        });
    }
    /** getStatus は現在状態 / runId / 出力 / エラーを取得する。 */
    async getStatus(workflowId) {
        return await this.raw().getStatus({
            workflowId,
            context: this.client.tenantContext(),
        });
    }
}
//# sourceMappingURL=workflow.js.map