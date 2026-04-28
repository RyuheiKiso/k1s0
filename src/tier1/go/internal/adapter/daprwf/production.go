// 本ファイルは Dapr Workflow building block の production adapter（FR-T1-WORKFLOW-001）。
//
// 設計正典:
//   docs/03_要件定義/20_機能要件/10_tier1_API要件/06_Workflow_API.md
//     - FR-T1-WORKFLOW-001（短期 Workflow は Dapr Workflow building block）
//     - FR-T1-WORKFLOW-002（長期は Temporal、本 adapter の対象外）
//   docs/04_概要設計/20_ソフトウェア方式設計/01_コンポーネント方式設計/01_tier1全体コンポーネント俯瞰.md
//     - DS-SW-COMP-010（t1-workflow Pod、Dapr Workflow / Temporal pluggable）
//
// 役割:
//   Dapr sidecar の Workflow Beta1 API（StartWorkflowBeta1 / GetWorkflowBeta1 /
//   TerminateWorkflowBeta1 / RaiseEventWorkflowBeta1 / PauseWorkflowBeta1 /
//   ResumeWorkflowBeta1）を WorkflowAdapter interface に翻訳する。
//
// テスタビリティ設計:
//   `daprWorkflowClient` narrow interface を使い、production では SDK の GRPCClient を、
//   test では fake を注入する。SDK の Client 全体ではなく必要メソッドだけに限定して
//   抽象化することで、SDK の major upgrade 影響面を最小化する。
//
// 制約と注釈:
//   - Query は Dapr Workflow Beta1 API に存在しないため、本 adapter は Unimplemented を返す。
//     業務要件として Query が必要な場合は Temporal backend（BACKEND_TEMPORAL）を使うこと。
//   - tenant_id は StartWorkflowRequest.Options["tenant_id"] に格納し、Component 側で
//     options を tag として記録する運用を前提とする（NFR-E-AC-003 越境防止の一段目）。

package daprwf

import (
	// 全 RPC で context を伝搬する。
	"context"
	// SDK エラーを ErrNotFound に翻訳するため。
	"errors"
	// 想定外 status のフォーマット。
	"fmt"
	// gRPC 詳細 status コードで NotFound 判定を行う。
	"strings"

	// Dapr SDK の workflow client 型。
	daprclient "github.com/dapr/go-sdk/client"
)

// daprWorkflowClient は本パッケージが Dapr SDK から **実際に使う workflow メソッド**
// だけを集めた narrow interface。`*daprclient.GRPCClient` がこれを満たすため
// production では SDK インスタンスをそのまま注入し、test では fake を注入する。
//
// 抽象を SDK 全体ではなく必要メソッドに絞る理由:
//   - 試験 fake が小さく済む
//   - Dapr SDK のメジャーアップグレードで影響する surface を最小化
type daprWorkflowClient interface {
	// 新規 workflow 開始（Beta1 spec）。
	StartWorkflowBeta1(ctx context.Context, req *daprclient.StartWorkflowRequest) (*daprclient.StartWorkflowResponse, error)
	// workflow 状態取得（Beta1 spec）。
	GetWorkflowBeta1(ctx context.Context, req *daprclient.GetWorkflowRequest) (*daprclient.GetWorkflowResponse, error)
	// workflow 強制終了（Beta1 spec）。
	TerminateWorkflowBeta1(ctx context.Context, req *daprclient.TerminateWorkflowRequest) error
	// workflow へのシグナル送信（Beta1 spec、k1s0 Signal 動詞のマッピング先）。
	RaiseEventWorkflowBeta1(ctx context.Context, req *daprclient.RaiseEventWorkflowRequest) error
	// workflow 一時停止（Beta1 spec、k1s0 Cancel 動詞のマッピング先）。
	PauseWorkflowBeta1(ctx context.Context, req *daprclient.PauseWorkflowRequest) error
	// workflow 再開（Beta1 spec、本 adapter は使わないが完全性のため interface に含める）。
	ResumeWorkflowBeta1(ctx context.Context, req *daprclient.ResumeWorkflowRequest) error
}

// productionDaprWorkflow は SDK Beta1 API を WorkflowAdapter に変換する production 実装。
type productionDaprWorkflow struct {
	// Dapr SDK の workflow narrow client（production は GRPCClient、test は fake）。
	client daprWorkflowClient
	// 使用する Dapr Workflow Component 名（既定 "dapr"）。
	component string
}

// defaultWorkflowComponent は Dapr Workflow Component の既定名。
// Dapr 公式 SDK の DefaultWorkflowComponent と整合させる。
const defaultWorkflowComponent = "dapr"

// NewProduction は Dapr SDK の workflow client から production adapter を生成する。
// component が空文字なら "dapr" を使う。
func NewProduction(client daprWorkflowClient, component string) WorkflowAdapter {
	if component == "" {
		component = defaultWorkflowComponent
	}
	return &productionDaprWorkflow{client: client, component: component}
}

// Start は Dapr Workflow を開始する。idempotent / tenant_id は Options metadata 経由で渡す。
func (p *productionDaprWorkflow) Start(ctx context.Context, req StartRequest) (StartResponse, error) {
	// Options に tenant_id / idempotent を詰める。Component 側で取り出して利用する。
	options := map[string]string{}
	if req.TenantID != "" {
		options["tenant_id"] = req.TenantID
	}
	if req.Idempotent {
		options["idempotent"] = "true"
	}
	resp, err := p.client.StartWorkflowBeta1(ctx, &daprclient.StartWorkflowRequest{
		// 空文字なら SDK が UUID を採番する。
		InstanceID:        req.WorkflowID,
		WorkflowComponent: p.component,
		// workflow_type は Component 側で workflow function 名として解決される。
		WorkflowName: req.WorkflowType,
		Options:      options,
		Input:        req.Input,
		// 入力は []byte なので serialize を抑制する。
		SendRawInput: true,
	})
	if err != nil {
		return StartResponse{}, err
	}
	// Dapr Beta1 は run_id を expose しないため、instance_id を兼用する。
	return StartResponse{
		WorkflowID: resp.InstanceID,
		// run_id は instance_id と同値で扱う（Beta1 spec 制約）。
		RunID: resp.InstanceID,
	}, nil
}

// Signal は Dapr Workflow に外部イベントを送る（RaiseEventWorkflowBeta1）。
func (p *productionDaprWorkflow) Signal(ctx context.Context, req SignalRequest) error {
	if err := p.client.RaiseEventWorkflowBeta1(ctx, &daprclient.RaiseEventWorkflowRequest{
		InstanceID:        req.WorkflowID,
		WorkflowComponent: p.component,
		EventName:         req.SignalName,
		EventData:         req.Payload,
		// payload は []byte なので serialize を抑制する。
		SendRawData: true,
	}); err != nil {
		return translateNotFound(err)
	}
	return nil
}

// Query は Dapr Workflow Beta1 では非対応。明示的に Unimplemented エラーを返す。
// 業務要件で Query が必要な場合は Temporal backend を使うこと（FR-T1-WORKFLOW-002）。
func (p *productionDaprWorkflow) Query(_ context.Context, _ QueryRequest) (QueryResponse, error) {
	return QueryResponse{}, errors.New("tier1/daprwf: Query not supported on Dapr Workflow Beta1 (use BACKEND_TEMPORAL)")
}

// Cancel は Dapr Workflow を一時停止する（PauseWorkflowBeta1）。
// Dapr Workflow に "cancel" の concept は無いため、Pause を採用する。
// 完全停止が必要な場合は Terminate を使う。
func (p *productionDaprWorkflow) Cancel(ctx context.Context, req CancelRequest) error {
	if err := p.client.PauseWorkflowBeta1(ctx, &daprclient.PauseWorkflowRequest{
		InstanceID:        req.WorkflowID,
		WorkflowComponent: p.component,
	}); err != nil {
		return translateNotFound(err)
	}
	return nil
}

// Terminate は Dapr Workflow を強制終了する（TerminateWorkflowBeta1）。
// reason は Dapr SDK API 上 expose されていないため、観測性のため Component 側 ログ運用とする。
func (p *productionDaprWorkflow) Terminate(ctx context.Context, req TerminateRequest) error {
	if err := p.client.TerminateWorkflowBeta1(ctx, &daprclient.TerminateWorkflowRequest{
		InstanceID:        req.WorkflowID,
		WorkflowComponent: p.component,
	}); err != nil {
		return translateNotFound(err)
	}
	return nil
}

// GetStatus は Dapr Workflow の状態を取得して WorkflowStatusValue に変換する。
func (p *productionDaprWorkflow) GetStatus(ctx context.Context, req GetStatusRequest) (GetStatusResponse, error) {
	resp, err := p.client.GetWorkflowBeta1(ctx, &daprclient.GetWorkflowRequest{
		InstanceID:        req.WorkflowID,
		WorkflowComponent: p.component,
	})
	if err != nil {
		return GetStatusResponse{}, translateNotFound(err)
	}
	return GetStatusResponse{
		Status: mapDaprStatus(resp.RuntimeStatus),
		// Beta1 は run_id を expose しないため instance_id を返す。
		RunID: resp.InstanceID,
	}, nil
}

// mapDaprStatus は Dapr Workflow の runtime status 文字列を WorkflowStatusValue に変換する。
// Dapr SDK の runtime status は "RUNNING" / "COMPLETED" / "CONTINUED_AS_NEW" / "FAILED" /
// "CANCELED" / "TERMINATED" / "PENDING" / "SUSPENDED" のいずれか（durabletask-go 由来）。
func mapDaprStatus(s string) WorkflowStatusValue {
	switch strings.ToUpper(s) {
	case "RUNNING", "PENDING", "CONTINUED_AS_NEW", "SUSPENDED":
		return StatusRunning
	case "COMPLETED":
		return StatusCompleted
	case "FAILED":
		return StatusFailed
	case "CANCELED":
		return StatusCanceled
	case "TERMINATED":
		return StatusTerminated
	default:
		// 想定外文字列は Running 扱い（observability 上は警告ログを出す運用）。
		return StatusRunning
	}
}

// translateNotFound は Dapr SDK のエラー文字列が "not found" を含む場合 ErrNotFound に翻訳する。
// Dapr SDK は構造化エラーを返さない（gRPC status code を直接 expose しない）ため、
// 文字列マッチで判定する。誤判定を避けるため "not found" の正確な部分一致のみを拾う。
func translateNotFound(err error) error {
	if err == nil {
		return nil
	}
	// SDK のエラー文字列に "not found" を含む場合のみ ErrNotFound に翻訳する。
	if strings.Contains(strings.ToLower(err.Error()), "not found") {
		return ErrNotFound
	}
	return fmt.Errorf("tier1/daprwf: %w", err)
}
