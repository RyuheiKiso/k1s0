// 本ファイルは k1s0 Go SDK の Saga 補償パターン helper。
//
// 設計正典:
//   docs/03_要件定義/20_機能要件/10_tier1_API要件/06_Workflow_API.md (FR-T1-WORKFLOW-003)
//
// 役割:
//   FR-T1-WORKFLOW-003 受け入れ基準「途中で失敗した場合、実行済み Activity の補償が
//   逆順で自動実行される / 補償漏れが構造的に発生しえない設計」を tier2 アプリ側の
//   client-side Saga executor として提供する。各 Step は独立した tier1 Workflow を
//   Start し、完了を GetStatus で polling する。Step が FAILED に倒れた時点で、
//   それまでに COMPLETED した Step の CompensationWorkflowType を逆順に Start する。
//
// 設計判断:
//   - Workflow 定義内で compensation を表現する設計 (Temporal SagaActivities) は
//     tier2 SDK 側の責務 (worker SDK 直接利用)。tier1 facade SDK は client-side で
//     多段 Workflow を chain できる軽量 helper を提供する。
//   - 補償失敗は最終的な err に collected errors を含めて返す (fail-soft で続行)。
//     呼出側が監査ログ用に individual compensation errors を取得できる。
//   - 完了判定は GetStatus polling。timeout / interval は呼出側が指定する。

package k1s0

import (
	// 標準 context。
	"context"
	// errors join (Go 1.20+) で複数補償エラーを集約。
	"errors"
	// 完了 polling の interval / 全体 timeout 用。
	"time"

	// proto Workflow status enum。
	workflowv1 "github.com/k1s0/sdk-go/proto/v1/k1s0/tier1/workflow/v1"
)

// SagaStep は Saga の単一ステップ。各 Step は独立した tier1 Workflow を実行し、
// CompensationWorkflowType は Step 失敗時に逆順で発火される補償 Workflow。
type SagaStep struct {
	// Step 名 (監査 / debug 用)。
	Name string
	// 順方向 Workflow の workflow_type (例: "OrderCreate")。
	WorkflowType string
	// 順方向 Workflow の input bytes (encoding は tier2 合意)。
	Input []byte
	// 補償 Workflow の workflow_type (例: "OrderCreateCompensate")。空文字なら補償スキップ。
	CompensationWorkflowType string
	// 補償 Workflow の input bytes (順方向結果を引き継ぐ場合は呼出側で詰めて渡す)。
	CompensationInput []byte
}

// SagaExecutionResult は Execute の戻り値。
type SagaExecutionResult struct {
	// 順方向に正常完了した Step の Name 列 (実行順)。
	CompletedSteps []string
	// 補償が発火した Step の Name 列 (逆順)。
	CompensatedSteps []string
	// 補償自体が失敗した case の集合 (補償漏れ警告)。
	CompensationErrors []error
}

// SagaConfig は Execute の任意パラメータ。
type SagaConfig struct {
	// 各 Step の workflow 完了 polling 間隔。0 で既定 1s。
	PollInterval time.Duration
	// 各 Step の最大待機時間。0 で既定 5min。
	StepTimeout time.Duration
}

// Saga は複数 SagaStep を順次実行する client-side executor。
type Saga struct {
	// tier1 Workflow client 参照。
	wf *WorkflowClient
	// 実行する Step 列 (順方向)。
	steps []SagaStep
}

// NewSaga は Saga executor を生成する。
func (w *WorkflowClient) NewSaga() *Saga {
	// 空 Step で初期化する。
	return &Saga{wf: w}
}

// Step は Saga に 1 ステップを追加する (chained builder)。
// CompensationWorkflowType が空文字の場合、当該 Step の補償はスキップされる
// (read-only な参照 Step / 副作用なし Step での明示)。
func (s *Saga) Step(name, workflowType string, input []byte, compensationWorkflowType string, compensationInput []byte) *Saga {
	// step を追加する。
	s.steps = append(s.steps, SagaStep{
		Name:                     name,
		WorkflowType:             workflowType,
		Input:                    input,
		CompensationWorkflowType: compensationWorkflowType,
		CompensationInput:        compensationInput,
	})
	// chained builder のため self を返す。
	return s
}

// Execute は Step を順次実行する。途中失敗で実行済み Step の補償を逆順に発火する。
// FR-T1-WORKFLOW-003 受け入れ基準: 「補償漏れが構造的に発生しえない設計」を満たす。
func (s *Saga) Execute(ctx context.Context, cfg SagaConfig) (SagaExecutionResult, error) {
	// poll interval / step timeout の既定値解決。
	if cfg.PollInterval <= 0 {
		// 1 秒間隔で polling する。
		cfg.PollInterval = 1 * time.Second
	}
	if cfg.StepTimeout <= 0 {
		// step ごとの上限は 5 分。
		cfg.StepTimeout = 5 * time.Minute
	}
	// 結果集計用の構造体を初期化する。
	result := SagaExecutionResult{}
	// 各 Step を順次実行する (ループ内で fail 検出時に補償経路へ jump)。
	for i, step := range s.steps {
		// Step を Start する。workflow_id は tier1 採番に任せる (空文字)。
		_, _, err := s.wf.Start(ctx, step.WorkflowType, "", step.Input, false)
		// Start 失敗は即補償経路へ。
		if err != nil {
			// 補償を逆順で実行する (i は失敗 Step、i-1 以前が COMPLETED)。
			s.compensateBackwards(ctx, i, &result)
			// 順方向失敗を呼出側に通知する。補償エラーは result.CompensationErrors に。
			return result, err
		}
		// 完了待機 (GetStatus polling、step ごとの timeout)。
		stepCtx, cancel := context.WithTimeout(ctx, cfg.StepTimeout)
		// step 完了を polling で確認する。
		status, statusErr := s.waitForCompletion(stepCtx, step.WorkflowType, cfg.PollInterval)
		// timeout / cancel スコープを解放する。
		cancel()
		// polling エラー (timeout 含む) → 補償経路。
		if statusErr != nil {
			// 当該 Step は不確定状態だが、補償は念のため発火する (idempotent compensation 前提)。
			s.compensateBackwards(ctx, i+1, &result)
			return result, statusErr
		}
		// FAILED は順方向失敗とみなして補償経路へ。
		if status == workflowv1.WorkflowStatus_FAILED {
			// 当該 Step を含めて補償を逆順発火する。
			s.compensateBackwards(ctx, i+1, &result)
			// 失敗を呼出側に通知する。
			return result, errors.New("k1s0/saga: step " + step.Name + " ended in FAILED")
		}
		// COMPLETED → 次 Step へ進む。
		result.CompletedSteps = append(result.CompletedSteps, step.Name)
	}
	// 全 Step 完了。
	return result, nil
}

// compensateBackwards は指定 index 以前 (排他、つまり [0, upTo) の範囲) の Step を
// 逆順に補償発火する。空の CompensationWorkflowType は skip。補償失敗は
// result.CompensationErrors に集約する (続行可能、fail-soft)。
func (s *Saga) compensateBackwards(ctx context.Context, upTo int, result *SagaExecutionResult) {
	// 上限を範囲内にクランプする。
	if upTo > len(s.steps) {
		upTo = len(s.steps)
	}
	// 逆順 (i-1 .. 0) に走査する。
	for j := upTo - 1; j >= 0; j-- {
		// 現在の Step を取り出す。
		step := s.steps[j]
		// 補償未指定 Step は skip する。
		if step.CompensationWorkflowType == "" {
			continue
		}
		// 補償 Workflow を Start する (idempotent compensation 前提)。
		_, _, err := s.wf.Start(ctx, step.CompensationWorkflowType, "", step.CompensationInput, false)
		// 失敗は collected errors に集約する。
		if err != nil {
			result.CompensationErrors = append(result.CompensationErrors, err)
			continue
		}
		// 成功した補償 Step の Name を記録する。
		result.CompensatedSteps = append(result.CompensatedSteps, step.Name)
	}
}

// waitForCompletion は ctx timeout 内で workflow が COMPLETED / FAILED になるまで polling する。
// CANCELED / TERMINATED もファイナル状態として返す。
func (s *Saga) waitForCompletion(ctx context.Context, workflowID string, pollInterval time.Duration) (workflowv1.WorkflowStatus, error) {
	// timer は close 不要 (Stop で Drain)。
	t := time.NewTimer(pollInterval)
	// 初回は即時 GetStatus を呼ぶ。
	defer t.Stop()
	// ループで polling する。
	for {
		// ctx 解約を最優先で監視する。
		select {
		case <-ctx.Done():
			// timeout / cancel → エラー返却。
			return workflowv1.WorkflowStatus_RUNNING, ctx.Err()
		default:
		}
		// 現在の status を取得する。
		resp, err := s.wf.GetStatus(ctx, workflowID)
		// gRPC エラー → 中断。
		if err != nil {
			return workflowv1.WorkflowStatus_RUNNING, err
		}
		// ファイナル状態かを判定する (COMPLETED / FAILED / CANCELED / TERMINATED)。
		switch resp.GetStatus() {
		case workflowv1.WorkflowStatus_COMPLETED, workflowv1.WorkflowStatus_FAILED,
			workflowv1.WorkflowStatus_CANCELED, workflowv1.WorkflowStatus_TERMINATED:
			return resp.GetStatus(), nil
		}
		// 未確定 → poll interval 待機して再試行する。
		t.Reset(pollInterval)
		select {
		case <-t.C:
			// 次回 polling へ。
		case <-ctx.Done():
			// timeout / cancel。
			return workflowv1.WorkflowStatus_RUNNING, ctx.Err()
		}
	}
}
