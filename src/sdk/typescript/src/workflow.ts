// 本ファイルは k1s0 TypeScript SDK の Workflow 動詞統一 facade。
import { createPromiseClient, type PromiseClient } from "@connectrpc/connect";
import type { K1s0Client } from "./client.js";
import { WorkflowService } from "./proto/k1s0/tier1/workflow/v1/workflow_service_connect.js";
import type { GetStatusResponse } from "./proto/k1s0/tier1/workflow/v1/workflow_service_pb.js";

/** WorkflowFacade は WorkflowService の動詞統一 facade。 */
export class WorkflowFacade {
  private readonly client: K1s0Client;

  constructor(client: K1s0Client) {
    this.client = client;
  }

  private raw(): PromiseClient<typeof WorkflowService> {
    return createPromiseClient(WorkflowService, this.client.transport);
  }

  /** start はワークフロー開始。返り値は { workflowId, runId }。 */
  async start(
    workflowType: string,
    workflowId: string,
    input: Uint8Array,
    idempotent = false,
  ): Promise<{ workflowId: string; runId: string }> {
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
