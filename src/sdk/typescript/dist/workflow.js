// 本ファイルは k1s0 TypeScript SDK の Workflow 動詞統一 facade。
import { createPromiseClient } from "@connectrpc/connect";
import { WorkflowService } from "./proto/k1s0/tier1/workflow/v1/workflow_service_connect.js";
/** WorkflowFacade は WorkflowService の動詞統一 facade。 */
export class WorkflowFacade {
    client;
    constructor(client) {
        this.client = client;
    }
    raw() {
        return createPromiseClient(WorkflowService, this.client.transport);
    }
    /** start はワークフロー開始。返り値は { workflowId, runId }。 */
    async start(workflowType, workflowId, input, idempotent = false) {
        const resp = await this.raw().start({
            workflowType,
            workflowId,
            input,
            idempotent,
            context: this.client.tenantContext(),
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