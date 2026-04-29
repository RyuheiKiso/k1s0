// k1s0 SDK ラッパー（BFF 内部から tier1 / tier2 を呼ぶ境界）。
//
// テナント上書き:
//   1 SDK Client インスタンスを複数エンドユーザの request 間で共有する。
//   各 request の tenant_id / subject は auth middleware が context へ attach 済なので、
//   SDK 呼出前に context-helpers から取り出して k1s0.WithTenant で SDK ctx に伝搬する。
//   これにより static cfg.TenantID で全 request を上書きしてしまう越境（NFR-E-AC-003 違反）を防ぐ。

// Package k1s0client は tier1 / tier2 への呼出を集約する Infrastructure 層相当。
package k1s0client

// 標準 / 内部 import。
import (
	// context 伝搬。
	"context"
	// 文字列整形。
	"fmt"
	// timeout。
	"time"

	// k1s0 高水準 facade。
	"github.com/k1s0/sdk-go/k1s0"

	// auth middleware の context helpers（per-request tenant_id / subject の解決）。
	"github.com/k1s0/k1s0/src/tier3/bff/internal/auth"
	// 設定。
	"github.com/k1s0/k1s0/src/tier3/bff/internal/config"
)

// Client は k1s0 SDK Client の薄いラッパー。
type Client struct {
	// SDK Client（接続を保持）。
	client *k1s0.Client
}

// New は config から Client を組み立てる。
func New(ctx context.Context, cfg config.K1s0Config) (*Client, error) {
	// dial timeout を 10 秒で被せる。
	dialCtx, cancel := context.WithTimeout(ctx, 10*time.Second)
	defer cancel()
	// SDK Client を初期化する。
	c, err := k1s0.New(dialCtx, k1s0.Config{
		Target:   cfg.Target,
		TenantID: cfg.TenantID,
		Subject:  cfg.Subject,
		UseTLS:   cfg.UseTLS,
	})
	// 失敗時はラップして伝搬する。
	if err != nil {
		return nil, fmt.Errorf("k1s0client.New: failed to dial %s: %w", cfg.Target, err)
	}
	return &Client{client: c}, nil
}

// Close は SDK Client を解放する。
func (c *Client) Close() error {
	if c == nil || c.client == nil {
		return nil
	}
	return c.client.Close()
}

// withTenantFromRequest は auth middleware が attach した tenant_id / subject を
// SDK 呼出 ctx に伝搬する。middleware が前段にいない（test 経路など）場合は
// ctx をそのまま返し、SDK は cfg.TenantID にフォールバックする。
func withTenantFromRequest(ctx context.Context) context.Context {
	tenantID := auth.TenantIDFromContext(ctx)
	if tenantID == "" {
		// middleware 未経由 / off mode 由来でない経路は cfg fallback に任せる。
		return ctx
	}
	subject := auth.SubjectFromContext(ctx)
	return k1s0.WithTenant(ctx, tenantID, subject)
}

// StateGet は k1s0 State から指定キーを取得する。
// auth middleware の tenant_id を SDK へ伝搬する。
func (c *Client) StateGet(ctx context.Context, store, key string) (data []byte, etag string, found bool, err error) {
	return c.client.State().Get(withTenantFromRequest(ctx), store, key)
}

// StateSave は k1s0 State にキーを保存する。
// auth middleware の tenant_id を SDK へ伝搬する。
func (c *Client) StateSave(ctx context.Context, store, key string, data []byte) (string, error) {
	return c.client.State().Save(withTenantFromRequest(ctx), store, key, data)
}
