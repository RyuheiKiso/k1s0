// 本ファイルは tier1 Go の Temporal アダプタ層の起点。
//
// 設計正典:
//   docs/04_概要設計/20_ソフトウェア方式設計/01_コンポーネント方式設計/01_tier1全体コンポーネント俯瞰.md
//     - DS-SW-COMP-010（t1-workflow: Dapr Workflow / Temporal pluggable、固定 3 replica）
//   docs/02_構想設計/adr/ADR-RULE-002-temporal.md
//   docs/03_要件定義/20_機能要件/40_tier1_API契約IDL/06_Workflow_API.md
//
// 役割（plan 04-07 結線済 / Temporal 経路）:
//   t1-workflow Pod が WorkflowService 6 RPC を Temporal Go SDK 越しに実装するための
//   adapter。短期 vs 長期の振り分け（U-WORKFLOW-001）は本層では行わず、handler 上位の
//   ポリシー層で決める。本 adapter は「長期 = Temporal」経路を担当する。
//
// テスタビリティ設計:
//   `temporalClient` narrow interface で SDK の Client から **使うメソッドだけ**抽象化。
//   production では SDK の `client.Client` がそのまま満たす。test では fake を注入する。

// Package temporal は tier1 Go ファサードが Temporal を呼ぶためのアダプタ層。
package temporal

import (
	"context"
	"errors"

	tclient "go.temporal.io/sdk/client"
	"go.temporal.io/sdk/converter"
)

// ErrNotWired は Temporal backend と未結線である旨を示すセンチネルエラー。
var ErrNotWired = errors.New("tier1: Temporal not wired")

// ErrWorkflowNotFound はワークフロー未存在を示すセンチネルエラー。
var ErrWorkflowNotFound = errors.New("tier1: workflow not found")

// temporalClient は本パッケージが Temporal SDK の Client から **実際に使うメソッド** だけを
// 集めた narrow interface。`tclient.Client` がこれを満たすため production では SDK を
// そのまま注入し、test では fake を注入する。
type temporalClient interface {
	// 新規ワークフロー実行（idempotent オプションは StartWorkflowOptions で表現）。
	ExecuteWorkflow(ctx context.Context, options tclient.StartWorkflowOptions, workflow interface{}, args ...interface{}) (tclient.WorkflowRun, error)
	// 既存ワークフローへのシグナル送信。
	SignalWorkflow(ctx context.Context, workflowID, runID, signalName string, arg interface{}) error
	// 既存ワークフローへのクエリ。
	QueryWorkflow(ctx context.Context, workflowID, runID, queryType string, args ...interface{}) (converter.EncodedValue, error)
	// キャンセル要求。
	CancelWorkflow(ctx context.Context, workflowID, runID string) error
	// 強制終了。
	TerminateWorkflow(ctx context.Context, workflowID, runID, reason string, details ...interface{}) error
	// 状態取得。
	DescribeWorkflowExecution(ctx context.Context, workflowID, runID string) (*describeResponse, error)
}

// describeResponse は Temporal SDK の DescribeWorkflowExecutionResponse を抽象化した
// minimal subset。SDK の workflowservice.DescribeWorkflowExecutionResponse を直接使うと
// fake 構築が重くなるため、必要フィールドだけを露出させる薄い wrapper を作る。
type describeResponse struct {
	// WorkflowExecutionInfo.Status を整数値に翻訳したもの（Temporal enum 値そのもの）。
	StatusCode int32
	// 直近の RunID。
	RunID string
}

// Client は tier1 Go ファサードから見た Temporal のアダプタ。
type Client struct {
	hostPort string
	tc       temporalClient
	closer   interface{ Close() }
}

// Config は Client 初期化時に渡される設定。
type Config struct {
	// Temporal frontend gRPC address（例: "temporal-frontend.k1s0-data.svc:7233"）。
	HostPort string
	// Temporal namespace（k1s0 既定: "k1s0"）。
	Namespace string
}

// New は Config から Client を生成する。Temporal SDK の Client を作り、narrow interface 越しに保持する。
// Note: Temporal SDK の Dial は SDK 内部で gRPC connection を作るため、ここではエラーを返す可能性がある。
func New(_ context.Context, cfg Config) (*Client, error) {
	opts := tclient.Options{HostPort: cfg.HostPort}
	if cfg.Namespace != "" {
		opts.Namespace = cfg.Namespace
	}
	sdkClient, err := tclient.Dial(opts)
	if err != nil {
		return nil, err
	}
	// SDK Client を narrow interface に shim 経由でラップする（DescribeWorkflowExecution の戻り値を minimal subset に絞るため）。
	return &Client{
		hostPort: cfg.HostPort,
		tc:       newSDKShim(sdkClient),
		closer:   sdkClient,
	}, nil
}

// NewWithClient は test 用コンストラクタ。任意の temporalClient 実装を受け取る。
func NewWithClient(hostPort string, tc temporalClient) *Client {
	return &Client{hostPort: hostPort, tc: tc, closer: nil}
}

// Close は Client が保持する Temporal SDK Client の gRPC 接続を解放する。
func (c *Client) Close() error {
	if c.closer == nil {
		return nil
	}
	c.closer.Close()
	return nil
}

// HostPort は Temporal frontend address を返す。
func (c *Client) HostPort() string {
	return c.hostPort
}

// temporalClientFor は内部 narrow client を返す。adapter 実装からのみ使う。
func (c *Client) temporalClientFor() temporalClient {
	return c.tc
}
