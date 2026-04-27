// 本ファイルは k1s0 Go SDK の Workflow 動詞統一 facade。
package k1s0

import (
	"context"

	workflowv1 "github.com/k1s0/sdk-go/proto/v1/k1s0/tier1/workflow/v1"
)

// WorkflowClient は WorkflowService の動詞統一 facade。
type WorkflowClient struct{ client *Client }

// Workflow は親 Client から WorkflowClient を返す。
func (c *Client) Workflow() *WorkflowClient { return c.workflow }

// Start はワークフロー開始。idempotent=true なら同 workflow_id の重複は既存実行を返す。
func (w *WorkflowClient) Start(ctx context.Context, workflowType, workflowID string, input []byte, idempotent bool) (returnedID, runID string, err error) {
	resp, e := w.client.raw.Workflow.Start(ctx, &workflowv1.StartRequest{
		WorkflowType: workflowType,
		WorkflowId:   workflowID,
		Input:        input,
		Idempotent:   idempotent,
		Context:      w.client.tenantContext(),
	})
	if e != nil {
		return "", "", e
	}
	return resp.GetWorkflowId(), resp.GetRunId(), nil
}

// Signal はシグナル送信。
func (w *WorkflowClient) Signal(ctx context.Context, workflowID, signalName string, payload []byte) error {
	_, e := w.client.raw.Workflow.Signal(ctx, &workflowv1.SignalRequest{
		WorkflowId: workflowID,
		SignalName: signalName,
		Payload:    payload,
		Context:    w.client.tenantContext(),
	})
	return e
}

// Query はワークフロー状態のクエリ（副作用なし）。
func (w *WorkflowClient) Query(ctx context.Context, workflowID, queryName string, payload []byte) ([]byte, error) {
	resp, e := w.client.raw.Workflow.Query(ctx, &workflowv1.QueryRequest{
		WorkflowId: workflowID,
		QueryName:  queryName,
		Payload:    payload,
		Context:    w.client.tenantContext(),
	})
	if e != nil {
		return nil, e
	}
	return resp.GetResult(), nil
}

// Cancel は正常終了の依頼。
func (w *WorkflowClient) Cancel(ctx context.Context, workflowID, reason string) error {
	_, e := w.client.raw.Workflow.Cancel(ctx, &workflowv1.CancelRequest{
		WorkflowId: workflowID,
		Reason:     reason,
		Context:    w.client.tenantContext(),
	})
	return e
}

// Terminate は強制終了。
func (w *WorkflowClient) Terminate(ctx context.Context, workflowID, reason string) error {
	_, e := w.client.raw.Workflow.Terminate(ctx, &workflowv1.TerminateRequest{
		WorkflowId: workflowID,
		Reason:     reason,
		Context:    w.client.tenantContext(),
	})
	return e
}

// GetStatus は現在状態 / run_id / 出力 / エラーを取得する。
func (w *WorkflowClient) GetStatus(ctx context.Context, workflowID string) (*workflowv1.GetStatusResponse, error) {
	return w.client.raw.Workflow.GetStatus(ctx, &workflowv1.GetStatusRequest{
		WorkflowId: workflowID,
		Context:    w.client.tenantContext(),
	})
}
