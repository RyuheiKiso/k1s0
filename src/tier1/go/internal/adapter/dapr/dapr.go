// 本ファイルは tier1 Go の Dapr Go SDK アダプタ層の起点。
//
// 設計正典:
//   docs/04_概要設計/20_ソフトウェア方式設計/01_コンポーネント方式設計/02_Daprファサード層コンポーネント.md
//     - 5 モジュールパイプライン: API Router → Policy Enforcer → Dapr Adapter → Log Adapter / Metrics Emitter（DS-SW-COMP-020）
//   docs/05_実装/00_ディレクトリ設計/20_tier1レイアウト/03_go_module配置.md
//     - internal/adapter/dapr/ に Dapr building block ラッパを配置
//   docs/02_構想設計/adr/ADR-TIER1-001-go-rust-hybrid.md
//
// 役割（リリース時点 最小骨格）:
//   t1-state Pod が叩く 5 つの Dapr building block（State / PubSub / ServiceInvocation /
//   Output Binding / flagd-Binding）への薄いラッパを提供する。
//   本リリース時点では実 Dapr SDK との結線は未着手で、handler から呼ばれた時に
//   `codes.Unimplemented` を返す placeholder を提供する。
//
// 未実装（plan 04-04 〜 04-13）:
//   - github.com/dapr/go-sdk/client の actual import / connection
//   - tier1 ErrorDetail へのエラーマッピング（gRPC google.rpc.Status → ErrorDetail）
//   - retry / circuit-breaker（Dapr SDK 内蔵が一次、追加 CB はリリース時点 不採用）

// Package dapr は tier1 Go ファサードが Dapr Go SDK を呼ぶためのアダプタ層。
package dapr

// 標準 Go ライブラリ。
import (
	// context.Context をシグネチャに含めるため import する（Dapr SDK 慣用）。
	"context"
	// 接続失敗 / 未実装エラーの整形に使う。
	"errors"
)

// ErrNotWired は Dapr backend と未結線である旨を示すセンチネルエラー。
// handler 側で `errors.Is(err, dapr.ErrNotWired)` 判定し codes.Unimplemented に変換する。
var ErrNotWired = errors.New("tier1: Dapr backend not yet wired (plan 04-04 〜 04-13)")

// Client は tier1 Go ファサードから見た Dapr Go SDK のアダプタ。
// 本リリース時点では実 Dapr SDK の参照を持たず、placeholder のみ。
type Client struct {
	// 接続済 Dapr sidecar のアドレス（例: localhost:50001、Dapr 既定）。
	// 値の使用は plan 04-04 以降。
	sidecarAddress string
}

// Config は Client 初期化時に渡される設定。
type Config struct {
	// Dapr sidecar の gRPC 接続先（例: "localhost:50001"）。
	// 既定は dapr.io/app-port=50001 と整合。
	SidecarAddress string
}

// New は Config から Client を生成する。
// 失敗しないが、将来 Dapr SDK 接続時には接続検証エラーを返す可能性がある。
func New(_ context.Context, cfg Config) (*Client, error) {
	// アドレス未指定時は Dapr 慣用既定値を使う。
	addr := cfg.SidecarAddress
	// 空文字列を defaultDaprAddress に補完する。
	if addr == "" {
		// Dapr sidecar の既定 gRPC ポートに合わせる。
		addr = defaultDaprAddress
	}
	// Client インスタンスを返却する（実 SDK は plan 04-XX で wire）。
	return &Client{sidecarAddress: addr}, nil
}

// Close は Client が保持するコネクションを解放する。
// 本リリース時点では no-op。
func (c *Client) Close() error {
	// 実 SDK 接続未結線のため何もしない。
	return nil
}

// SidecarAddress は Client が想定する Dapr sidecar のアドレスを返す。
// 観測性 / デバッグ用途で main から参照される。
func (c *Client) SidecarAddress() string {
	// フィールドをそのまま返却する。
	return c.sidecarAddress
}

// defaultDaprAddress は Dapr sidecar の既定 gRPC ポート。
// docs 正典: docs/04_概要設計/.../02_Daprファサード層コンポーネント.md（Dapr sidecar 50001）
const defaultDaprAddress = "localhost:50001"
