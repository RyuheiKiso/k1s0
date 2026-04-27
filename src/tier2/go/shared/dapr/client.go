// 本ファイルは tier2 Go の k1s0 SDK Client 初期化共通ラッパー。
//
// docs 正典:
//   docs/05_実装/00_ディレクトリ設計/30_tier2レイアウト/03_go_services配置.md
//   docs/05_実装/00_ディレクトリ設計/30_tier2レイアウト/04_サービス単位の内部構造.md
//
// scope:
//   k1s0 SDK Client（src/sdk/go/k1s0）の初期化を tier2 Go 全サービスで共通化する。
//   tier2 サービスの Infrastructure 層から呼び出し、Domain 層には k1s0 SDK 型を露出させない。
//
// stability: Alpha（リリース時点 開始、リリース時点 で Beta 目標）

// Package dapr は tier2 Go の k1s0 SDK / Dapr 統合ラッパー。
package dapr

// 標準 / 内部 import。
import (
	// context 伝搬。
	"context"
	// エラー文字列整形。
	"fmt"
	// デフォルト値計算。
	"strings"
	// timeout 設定。
	"time"

	// k1s0 高水準 facade（State / PubSub / Secrets 等）。
	"github.com/k1s0/sdk-go/k1s0"
	// gRPC ランタイム（DialOption 拡張に利用）。
	"google.golang.org/grpc"
)

// Config は tier2 Go サービスから k1s0 SDK Client を構築する際の設定。
type Config struct {
	// k1s0 tier1 facade の gRPC 接続先（例: "tier1-state.k1s0-system.svc.cluster.local:50001"）。
	Target string
	// テナント ID（全 RPC の TenantContext.tenant_id に自動付与される）。
	TenantID string
	// 主体識別子（subject、tier2 サービス自身の identity）。
	Subject string
	// TLS を使うか（本番 true / dev local-stack false）。
	UseTLS bool
	// gRPC dial timeout（デフォルト 10 秒）。
	DialTimeout time.Duration
	// 追加 DialOption（OTel interceptor 等を呼出側で渡す）。
	DialOptions []grpc.DialOption
}

// Validate は最低限の必須項目をチェックする。
func (c *Config) Validate() error {
	// Target は接続先のため必須。
	if strings.TrimSpace(c.Target) == "" {
		// 設定不備として error を返す。
		return fmt.Errorf("dapr.Config: Target is required")
	}
	// TenantID は tier1 ガード（PII deny-by-default）に必須。
	if strings.TrimSpace(c.TenantID) == "" {
		// 設定不備として error を返す。
		return fmt.Errorf("dapr.Config: TenantID is required")
	}
	// Subject は監査ログ identity 解決に必須。
	if strings.TrimSpace(c.Subject) == "" {
		// 設定不備として error を返す。
		return fmt.Errorf("dapr.Config: Subject is required")
	}
	// チェック通過。
	return nil
}

// applyDefaults は未設定値にデフォルトを当てる。
func (c *Config) applyDefaults() {
	// DialTimeout が 0 ならデフォルト 10 秒。
	if c.DialTimeout <= 0 {
		// 10 秒を採用。
		c.DialTimeout = 10 * time.Second
	}
}

// NewClient は k1s0 SDK Client を初期化する。
//
// 利用側責務:
//   - 戻り値の Client は呼出元が defer Close() で解放する
//   - DialOptions に OTel interceptor / retry policy を付与するのも呼出元の責務
func NewClient(ctx context.Context, cfg Config) (*k1s0.Client, error) {
	// 設定を検証する。
	if err := cfg.Validate(); err != nil {
		// 不正な設定は呼出元に伝搬する。
		return nil, err
	}
	// デフォルト値を適用する。
	cfg.applyDefaults()
	// dial timeout を context にかぶせる。
	dialCtx, cancel := context.WithTimeout(ctx, cfg.DialTimeout)
	// timeout を解放する。
	defer cancel()
	// k1s0 SDK の Config を組み立てる。
	sdkCfg := k1s0.Config{
		Target:      cfg.Target,
		TenantID:    cfg.TenantID,
		Subject:     cfg.Subject,
		UseTLS:      cfg.UseTLS,
		DialOptions: cfg.DialOptions,
	}
	// k1s0 Client を構築する（SDK 内部で grpc.NewClient を呼ぶ）。
	client, err := k1s0.New(dialCtx, sdkCfg)
	// 接続失敗時はラップして返す。
	if err != nil {
		// caller でログ出力可能にエラーをラップ。
		return nil, fmt.Errorf("dapr.NewClient: failed to dial %s: %w", cfg.Target, err)
	}
	// Client を返す（呼出元で defer client.Close()）。
	return client, nil
}
