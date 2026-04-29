// 本ファイルは k1s0 TypeScript SDK の Workflow 動詞統一 facade。
import { createPromiseClient, type PromiseClient } from "@connectrpc/connect";
import type { K1s0Client } from "./client.js";
import { WorkflowService } from "./proto/k1s0/tier1/workflow/v1/workflow_service_connect.js";
import {
  WorkflowBackend,
  type GetStatusResponse,
} from "./proto/k1s0/tier1/workflow/v1/workflow_service_pb.js";

/** WorkflowFacade は WorkflowService の動詞統一 facade。 */
export class WorkflowFacade {
  private readonly client: K1s0Client;

  constructor(client: K1s0Client) {
    this.client = client;
  }

  private raw(): PromiseClient<typeof WorkflowService> {
    return createPromiseClient(WorkflowService, this.client.transport);
  }

  /** start はワークフロー開始。backend hint は BACKEND_AUTO（tier1 が振り分け）。
   * 返り値は { workflowId, runId }。
   * 短期 / 長期で意図的に振り分けたい時は runShort / runLong を使う。 */
  async start(
    workflowType: string,
    workflowId: string,
    input: Uint8Array,
    idempotent = false,
  ): Promise<{ workflowId: string; runId: string }> {
    return this.startWithBackend(
      workflowType,
      workflowId,
      input,
      idempotent,
      WorkflowBackend.BACKEND_AUTO,
    );
  }

  /** runShort は短期ワークフロー（≤7 日、BACKEND_DAPR）として開始する（FR-T1-WORKFLOW-001）。
   * 短期ワークフローは Dapr Workflow building block で実行され、Pod 再起動でも履歴が保持される。 */
  async runShort(
    workflowType: string,
    workflowId: string,
    input: Uint8Array,
    idempotent = false,
  ): Promise<{ workflowId: string; runId: string }> {
    return this.startWithBackend(
      workflowType,
      workflowId,
      input,
      idempotent,
      WorkflowBackend.BACKEND_DAPR,
    );
  }

  /** runLong は長期ワークフロー（上限なし、BACKEND_TEMPORAL）として開始する（FR-T1-WORKFLOW-002）。
   * Continue-as-New / cron / 高度な signal 機能が必要な場合に使う。 */
  async runLong(
    workflowType: string,
    workflowId: string,
    input: Uint8Array,
    idempotent = false,
  ): Promise<{ workflowId: string; runId: string }> {
    return this.startWithBackend(
      workflowType,
      workflowId,
      input,
      idempotent,
      WorkflowBackend.BACKEND_TEMPORAL,
    );
  }

  /** startWithBackend は start / runShort / runLong の共通実装。 */
  private async startWithBackend(
    workflowType: string,
    workflowId: string,
    input: Uint8Array,
    idempotent: boolean,
    backend: WorkflowBackend,
  ): Promise<{ workflowId: string; runId: string }> {
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
  async signal(workflowId: string, signalName: string, payload: Uint8Array): Promise<void> {
    await this.raw().signal({
      workflowId,
      signalName,
      payload,
      context: this.client.tenantContext(),
    });
  }

  /** query はワークフロー状態のクエリ（副作用なし）。 */
  async query(workflowId: string, queryName: string, payload: Uint8Array): Promise<Uint8Array> {
    const resp = await this.raw().query({
      workflowId,
      queryName,
      payload,
      context: this.client.tenantContext(),
    });
    return resp.result;
  }

  /** cancel は正常終了の依頼。 */
  async cancel(workflowId: string, reason: string): Promise<void> {
    await this.raw().cancel({
      workflowId,
      reason,
      context: this.client.tenantContext(),
    });
  }

  /** terminate は強制終了。 */
  async terminate(workflowId: string, reason: string): Promise<void> {
    await this.raw().terminate({
      workflowId,
      reason,
      context: this.client.tenantContext(),
    });
  }

  /** getStatus は現在状態 / runId / 出力 / エラーを取得する。 */
  async getStatus(workflowId: string): Promise<GetStatusResponse> {
    return await this.raw().getStatus({
      workflowId,
      context: this.client.tenantContext(),
    });
  }
}
