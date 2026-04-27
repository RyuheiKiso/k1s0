// k1s0 SDK Client の Infrastructure 層ラッパー。
//
// docs 正典:
//   docs/05_実装/00_ディレクトリ設計/30_tier2レイアウト/04_サービス単位の内部構造.md
//
// 役割:
//   notification-hub から k1s0 SDK を呼ぶ際の境界。Domain / Application 層に SDK 型を漏出させない。

// Package external は外部システムへのアクセスを集約する Infrastructure 層。
package external

// 標準 / 内部 import。
import (
	// context 伝搬。
	"context"

	// k1s0 SDK 高水準 facade。
	"github.com/k1s0/sdk-go/k1s0"

	// notification-hub 設定。
	"github.com/k1s0/k1s0/src/tier2/go/services/notification-hub/internal/config"
	// shared dapr ラッパー。
	shareddapr "github.com/k1s0/k1s0/src/tier2/go/shared/dapr"
)

// K1s0Client は SDK Client の薄いラッパー。Binding 呼出のみ露出する。
type K1s0Client struct {
	// SDK Client。
	client *k1s0.Client
}

// NewK1s0Client は config から K1s0Client を組み立てる。
func NewK1s0Client(ctx context.Context, cfg config.K1s0Config) (*K1s0Client, error) {
	// shared dapr ラッパー経由で SDK Client を初期化する。
	c, err := shareddapr.NewClient(ctx, shareddapr.Config{
		// gRPC target。
		Target: cfg.Target,
		// テナント ID。
		TenantID: cfg.TenantID,
		// 主体識別子。
		Subject: cfg.Subject,
		// TLS 利用フラグ。
		UseTLS: cfg.UseTLS,
	})
	// 接続失敗は伝搬する。
	if err != nil {
		// 呼出元で fail-fast。
		return nil, err
	}
	// ラッパーを返す。
	return &K1s0Client{client: c}, nil
}

// Close は SDK Client を解放する。
func (c *K1s0Client) Close() error {
	// nil ガード。
	if c == nil || c.client == nil {
		// 何もしない。
		return nil
	}
	// SDK Client の Close を委譲する。
	return c.client.Close()
}

// BindingInvoke は k1s0 Binding 経由で外部出力を行う。
//
// 通知の実配信（SMTP / Slack / HTTP）は k1s0 Binding の Operation 引数で切替する。
// 戻り値の responseData / responseMetadata はチャネル次第で意味が異なる（caller 側で扱う）。
func (c *K1s0Client) BindingInvoke(ctx context.Context, name, operation string, data []byte, metadata map[string]string) ([]byte, map[string]string, error) {
	// SDK facade を呼ぶ。
	return c.client.Binding().Invoke(ctx, name, operation, data, metadata)
}
