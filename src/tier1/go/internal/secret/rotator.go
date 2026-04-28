// 本ファイルは Secret 自動ローテーション（FR-T1-SECRETS-004）の実装。
//
// 設計正典:
//   docs/03_要件定義/20_機能要件/10_tier1_API要件/04_Secrets_API.md
//     - FR-T1-SECRETS-004（"Secret ローテーション自動化"、リリース時点 必須）
//   docs/03_要件定義/20_機能要件/40_tier1_API契約IDL/04_Secrets_API.md
//
// 役割:
//   secret ごとに rotate cadence（=ticker 間隔）を保持し、ticker 発火ごとに
//   SecretsAdapter.Rotate を呼ぶ。production の cron / k8s CronJob ではなく
//   t1-secret Pod 内 goroutine で実行することで、tier1 の責務（ローテーション層）
//   を Pod 境界で閉じ込める。
//
// 設定:
//   環境変数 ROTATION_SCHEDULE で per-secret cadence を渡す。
//   形式: "tenant_id/secret_name@duration[,tenant_id/secret_name@duration]..."
//   例:   "tenant_a/db-password@1h,tenant_a/api-key@24h"
//   duration は Go の time.ParseDuration 解釈（"30s" / "5m" / "1h" / "24h" 等）。
//   設定がない場合 Rotator は no-op（goroutine も起動しない）。
//
// 失敗ハンドリング:
//   - SecretsAdapter.Rotate が ErrSecretNotFound 等を返した場合は warn ログを残し、
//     次の tick で再試行する（rotation policy は不変、idempotent）。
//   - context.Canceled でループを抜ける（graceful shutdown）。

package secret

import (
	"context"
	"errors"
	"fmt"
	"log"
	"strings"
	"sync"
	"time"

	"github.com/k1s0/k1s0/src/tier1/go/internal/adapter/openbao"
)

// rotationTarget は 1 件のローテーション対象。
type rotationTarget struct {
	// テナント識別子。
	tenantID string
	// secret 名（OpenBao path）。
	name string
	// rotate 間隔。
	interval time.Duration
}

// String は デバッグ / ログ向けの整形。
func (t rotationTarget) String() string {
	return fmt.Sprintf("%s/%s@%s", t.tenantID, t.name, t.interval)
}

// Rotator は per-secret ticker を駆動するバックグラウンド構造体。
type Rotator struct {
	// 1 ticker = 1 secret。すべての ticker をまとめて停止できるよう wg を持つ。
	adapter openbao.SecretsAdapter
	// rotate 対象。
	targets []rotationTarget
	// shutdown 用 cancel。
	cancel context.CancelFunc
	// goroutine 終了同期。
	wg sync.WaitGroup
}

// NewRotatorFromEnv は環境変数 ROTATION_SCHEDULE からスケジュールを解釈して
// Rotator を返す。env が空文字 / 解釈不能なら何も登録しない（empty Rotator を返す）。
func NewRotatorFromEnv(adapter openbao.SecretsAdapter, scheduleEnv string) (*Rotator, error) {
	// 解析した targets を保持する。
	targets, err := parseRotationSchedule(scheduleEnv)
	if err != nil {
		// 解析失敗時は呼出元に返す（cmd/secret 側でログ + exit 判断）。
		return nil, err
	}
	// adapter 未注入は保護的に弾く（そもそも cmd 側で注入されているはず）。
	if adapter == nil {
		return nil, errors.New("rotator: secrets adapter is required")
	}
	// 構築のみ、ticker は Start 呼出時に立ち上がる。
	return &Rotator{adapter: adapter, targets: targets}, nil
}

// parseRotationSchedule は ROTATION_SCHEDULE 文字列を parse する。
// 空文字 → 空 slice、形式不正 → error。
func parseRotationSchedule(s string) ([]rotationTarget, error) {
	// 完全に空 / 空白のみは "schedule なし" として扱う。
	trimmed := strings.TrimSpace(s)
	if trimmed == "" {
		return nil, nil
	}
	// "," で項目分割する。
	parts := strings.Split(trimmed, ",")
	out := make([]rotationTarget, 0, len(parts))
	for _, p := range parts {
		// 各項目をトリム。
		entry := strings.TrimSpace(p)
		if entry == "" {
			// 空項目は skip。
			continue
		}
		// "tenant/name@duration" を分解する。
		atIdx := strings.LastIndex(entry, "@")
		if atIdx < 0 {
			return nil, fmt.Errorf("rotator: invalid schedule entry %q (expected 'tenant/name@duration')", entry)
		}
		// tenant/name と duration に分ける。
		tenantName := strings.TrimSpace(entry[:atIdx])
		dur := strings.TrimSpace(entry[atIdx+1:])
		// "tenant/name" を "/" で分ける（最初の "/" を区切りに使う）。
		slashIdx := strings.Index(tenantName, "/")
		if slashIdx < 0 {
			return nil, fmt.Errorf("rotator: invalid schedule entry %q (expected 'tenant/name')", entry)
		}
		tenant := strings.TrimSpace(tenantName[:slashIdx])
		name := strings.TrimSpace(tenantName[slashIdx+1:])
		if tenant == "" || name == "" {
			return nil, fmt.Errorf("rotator: invalid schedule entry %q (empty tenant or name)", entry)
		}
		// duration を Go の time.Duration として解釈する。
		interval, err := time.ParseDuration(dur)
		if err != nil {
			return nil, fmt.Errorf("rotator: invalid duration %q in %q: %w", dur, entry, err)
		}
		// 0 / 負値は明示拒否（無限ループ防止）。
		if interval <= 0 {
			return nil, fmt.Errorf("rotator: non-positive duration %q in %q", dur, entry)
		}
		// 結果に追加する。
		out = append(out, rotationTarget{tenantID: tenant, name: name, interval: interval})
	}
	return out, nil
}

// Targets は登録された rotation 対象を返す（観測 / debug 用）。
func (r *Rotator) Targets() []string {
	if r == nil {
		return nil
	}
	out := make([]string, 0, len(r.targets))
	for _, t := range r.targets {
		out = append(out, t.String())
	}
	return out
}

// Start は context が cancel されるまで各 secret の ticker を駆動する。
// 既に Start 済 / targets が空の場合は no-op。
func (r *Rotator) Start(ctx context.Context) {
	if r == nil || len(r.targets) == 0 {
		return
	}
	// 既存の cancel があれば再 start を弾く。
	if r.cancel != nil {
		log.Printf("tier1/secret: rotator already started, skipping")
		return
	}
	// shutdown 用 child context を作る。
	child, cancel := context.WithCancel(ctx)
	r.cancel = cancel
	// 各 target に goroutine を 1 つ立ち上げる。
	for _, target := range r.targets {
		// 値コピー（goroutine への capture 安全化）。
		t := target
		r.wg.Add(1)
		go r.run(child, t)
	}
	log.Printf("tier1/secret: rotator started with %d schedule(s): %v",
		len(r.targets), r.Targets())
}

// Stop は全 goroutine を cancel し、終了を待ち合わせる。
func (r *Rotator) Stop() {
	if r == nil || r.cancel == nil {
		return
	}
	r.cancel()
	r.wg.Wait()
	r.cancel = nil
	log.Printf("tier1/secret: rotator stopped")
}

// run は単一 target の ticker ループ。context.Done で graceful exit。
func (r *Rotator) run(ctx context.Context, t rotationTarget) {
	defer r.wg.Done()
	// time.NewTicker は immediate fire しない。最初の rotate は interval 後。
	// "起動直後 rotate" を望む運用は別途 cron-like を導入する想定（plan 04-15）。
	tick := time.NewTicker(t.interval)
	defer tick.Stop()
	for {
		select {
		case <-ctx.Done():
			return
		case <-tick.C:
			r.rotateOnce(ctx, t)
		}
	}
}

// rotateOnce は 1 回の Rotate 呼出を行い、結果をログする。
func (r *Rotator) rotateOnce(ctx context.Context, t rotationTarget) {
	// SecretsAdapter.Rotate を呼ぶ。
	resp, err := r.adapter.Rotate(ctx, openbao.SecretRotateRequest{
		Name:     t.name,
		TenantID: t.tenantID,
	})
	// エラーは warn にとどめる（次 tick で再試行）。
	if err != nil {
		log.Printf("tier1/secret: rotator: %s rotation failed: %v", t, err)
		return
	}
	// 成功時は version bump をログに残す（監査経路は別途 Audit RPC）。
	log.Printf("tier1/secret: rotator: %s rotated to version %d", t, resp.Version)
}
