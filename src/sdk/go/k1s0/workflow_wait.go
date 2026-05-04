// 本ファイルは k1s0 Go SDK の Workflow 外部イベント連携 helper。
//
// 設計正典:
//   docs/03_要件定義/20_機能要件/10_tier1_API要件/06_Workflow_API.md (FR-T1-WORKFLOW-005)
//
// 役割:
//   FR-T1-WORKFLOW-005「Workflow 定義内で WaitForEvent("event_name") を呼び、外部から
//   SignalWorkflow(workflow_id, "event_name", payload) で通知する。ポーリング不要」を
//   満たすため、tier2 アプリの client-side helper として SignalAndAwait を提供する。
//
// 役割分担:
//   - Workflow 定義内 WaitForEvent: tier2 が temporal worker SDK / dapr workflow SDK で
//     Channel.Receive 等を呼ぶ責務。tier1 facade SDK では実装しない (worker side の話)。
//   - 外部からの SignalWorkflow: tier1 facade の Signal RPC を呼び、SignalResponse 返却で
//     確定する。本 helper はこの client-side 経路を集約する。
//
// SignalAndAwait の意義:
//   外部システム (Webhook 受信 / 業務担当の承認 etc) が Signal を送る際に、Workflow が
//   確実に進行したことを GetStatus で確認したい使い方を支援する。Signal は async で
//   即時 OK 返却するため、Workflow が COMPLETED / FAILED / 特定 query 結果に到達した
//   ことを別途 polling する必要がある。

package k1s0

import (
	// 標準 context。
	"context"
	// poll interval / 全体 timeout 用。
	"time"

	// proto Workflow status enum。
	workflowv1 "github.com/k1s0/sdk-go/proto/v1/k1s0/tier1/workflow/v1"
)

// SignalAndAwaitConfig は SignalAndAwait の任意パラメータ。
type SignalAndAwaitConfig struct {
	// 完了判定の polling interval。0 で既定 1s。
	PollInterval time.Duration
	// 完了待機の最大時間。0 で既定 5min。
	Timeout time.Duration
	// 待機する最終状態の集合 (空なら COMPLETED / FAILED / CANCELED / TERMINATED 全部)。
	WaitStatuses []workflowv1.WorkflowStatus
}

// SignalAndAwait は外部からの Signal 送信と、Workflow が指定終端状態に到達するまでの
// 待機を 1 つの操作として提供する。FR-T1-WORKFLOW-005 のポーリング撲滅は Workflow
// 定義内の WaitForEvent 受信側で達成され、本関数は外部送信側の汎用ユーティリティ。
func (w *WorkflowClient) SignalAndAwait(
	ctx context.Context,
	workflowID, signalName string,
	payload []byte,
	cfg SignalAndAwaitConfig,
) (workflowv1.WorkflowStatus, error) {
	// 既定値解決: poll interval。
	if cfg.PollInterval <= 0 {
		// 1 秒間隔で polling する。
		cfg.PollInterval = 1 * time.Second
	}
	// 既定値解決: timeout。
	if cfg.Timeout <= 0 {
		// 全体 5 分の制限。
		cfg.Timeout = 5 * time.Minute
	}
	// 既定値解決: 待機状態の集合 (未指定なら全終端状態)。
	waitFor := cfg.WaitStatuses
	if len(waitFor) == 0 {
		waitFor = []workflowv1.WorkflowStatus{
			workflowv1.WorkflowStatus_COMPLETED,
			workflowv1.WorkflowStatus_FAILED,
			workflowv1.WorkflowStatus_CANCELED,
			workflowv1.WorkflowStatus_TERMINATED,
		}
	}
	// Step 1: Signal 送信 (即時 OK 返却、async)。
	if err := w.Signal(ctx, workflowID, signalName, payload); err != nil {
		// Signal 失敗 → ctx 状態を保ったまま即返却。
		return workflowv1.WorkflowStatus_RUNNING, err
	}
	// Step 2: 終端状態 polling。timeout 付き ctx を作る。
	pollCtx, cancel := context.WithTimeout(ctx, cfg.Timeout)
	// timer 解放を確実に。
	defer cancel()
	// 進捗 polling 用 timer。
	t := time.NewTimer(cfg.PollInterval)
	// timer の確実な解放。
	defer t.Stop()
	// poll loop。
	for {
		// 終端判定 → 最新 status を取得する。
		resp, err := w.GetStatus(pollCtx, workflowID)
		// gRPC エラーは即返却。
		if err != nil {
			return workflowv1.WorkflowStatus_RUNNING, err
		}
		// 待機対象状態に一致するかを線形検査する (集合は最大 4 要素なので O(n) で十分)。
		current := resp.GetStatus()
		// 各待機状態と現在状態を比較する。
		for _, s := range waitFor {
			// 一致したら終端 status を返す。
			if s == current {
				return current, nil
			}
		}
		// 未到達 → poll interval 待機して再試行する。
		t.Reset(cfg.PollInterval)
		// timer fire / ctx cancel のいずれかで unblock する。
		select {
		case <-t.C:
			// 次回 polling へ。
		case <-pollCtx.Done():
			// timeout / cancel → ctx エラーで返却。
			return current, pollCtx.Err()
		}
	}
}
