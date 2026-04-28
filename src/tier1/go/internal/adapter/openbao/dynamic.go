// 本ファイルは OpenBao Database Engine（動的 Secret 発行）アダプタ。
//
// 設計正典:
//   docs/03_要件定義/20_機能要件/10_tier1_API要件/04_Secrets_API.md
//     - FR-T1-SECRETS-002（動的 Secret 発行、TTL 1 時間既定 / 24 時間上限）
//   docs/03_要件定義/20_機能要件/40_tier1_API契約IDL/04_Secrets_API.md
//     - GetDynamic RPC（FR-T1-SECRETS-002 経路）
//   docs/02_構想設計/adr/ADR-SEC-002-openbao.md
//
// 役割:
//   - 動的 Secret（DB 認証情報・Kafka ACL・MinIO STS 等）を OpenBao の Database Engine
//     経由で発行する narrow interface を提供する。
//   - dev / CI 用の in-memory backend を同梱し、外部 OpenBao 不要で API テストできる。
//   - production では OpenBao の generic Logical().Read("<engine>/creds/<role>") に
//     委譲する productionDynamicClient を使う。

package openbao

import (
	// 全 RPC で context を伝搬する。
	"context"
	// in-memory 動的シークレット発行で credential（username/password）を生成するため。
	"crypto/rand"
	"encoding/hex"
	// 排他制御。
	"sync"
	// TTL 制御で時刻と integer 比較を行う。
	"time"
)

// 既定 TTL（FR-T1-SECRETS-002 受け入れ基準: "デフォルト 1 時間"）。
const defaultDynamicTTLSec int32 = 3600

// 最大 TTL（FR-T1-SECRETS-002 受け入れ基準: "最大 24 時間"）。
const maxDynamicTTLSec int32 = 86400

// DynamicSecretRequest は GetDynamic 操作の adapter 入力。
type DynamicSecretRequest struct {
	// 発行エンジン名（"postgres" / "mysql" / "kafka" 等、OpenBao 上の database engine 種別）。
	Engine string
	// 役割（OpenBao 側で予め定義された role）。
	Role string
	// テナント識別子（テナント越境発行を弾くため必須）。
	TenantID string
	// TTL 秒数（0 で defaultDynamicTTLSec、上限 maxDynamicTTLSec に clamp する）。
	TTLSeconds int32
}

// DynamicSecretResponse は GetDynamic 操作の応答。
type DynamicSecretResponse struct {
	// credential 一式（"username" / "password" 等）。
	Values map[string]string
	// OpenBao の lease ID（renewal / revoke 用、削除時に呼び返す）。
	LeaseID string
	// 実際に付与された TTL 秒数。
	TTLSeconds int32
	// 発効時刻（Unix epoch ミリ秒）。
	IssuedAtMs int64
}

// DynamicAdapter は動的 Secret 発行の操作集合。
type DynamicAdapter interface {
	// 動的 Secret 発行。失敗時は ErrNotWired / ErrSecretNotFound 等を返す。
	GetDynamic(ctx context.Context, req DynamicSecretRequest) (DynamicSecretResponse, error)
}

// clampTTL は要求 TTL を仕様範囲内に整える。
// 0 は default、上限超過は max に clamp する。
func clampTTL(requested int32) int32 {
	// 0 / 負値は default。
	if requested <= 0 {
		// 既定の 1 時間を返す。
		return defaultDynamicTTLSec
	}
	// 上限超過は 24 時間に clamp する。
	if requested > maxDynamicTTLSec {
		// 最大値を返す。
		return maxDynamicTTLSec
	}
	// 範囲内なら そのまま。
	return requested
}

// inMemoryDynamic は dev / CI 用の動的 Secret 発行 backend。
//
// 実 OpenBao Database Engine と同じセマンティクスを最低限満たす:
//   - リクエストごとに新規 credential（username/password）を発行する
//   - lease ID を発行し、TTL 経過で expired として認識される
//   - 同 lease の renewal / revoke は本リリースでは未実装（API は IsActive で代替）
//
// 制限:
//   - DB に実ユーザを作らない（dev 環境では useless だが、API 動作確認には十分）
//   - 永続化なし、再起動で全 lease 消失
type inMemoryDynamic struct {
	// 排他制御（leases への並行 append 保護）。
	mu sync.Mutex
	// 全 lease を audit 目的で保持する。
	leases map[string]*leaseRecord
	// lease ID 採番カウンタ。
	counter int
}

// leaseRecord は in-memory 1 lease の保存データ。
type leaseRecord struct {
	// engine（"postgres" 等）。
	engine string
	// role 名。
	role string
	// テナント識別子（テナント境界検証）。
	tenantID string
	// 発効時刻（UnixMilli）。
	issuedAtMs int64
	// 期限切れ時刻（UnixMilli、issuedAtMs + ttl_sec*1000）。
	expiresAtMs int64
}

// NewInMemoryDynamic は空 backend を生成する。
func NewInMemoryDynamic() DynamicAdapter {
	// 空 map で初期化する。
	return &inMemoryDynamic{leases: map[string]*leaseRecord{}}
}

// GetDynamic は新規 credential を発行する。
func (m *inMemoryDynamic) GetDynamic(_ context.Context, req DynamicSecretRequest) (DynamicSecretResponse, error) {
	// 必須フィールド検証。
	if req.Engine == "" || req.Role == "" {
		// errEmptyTenant に意味的に近い「不正引数」を errors として返す（handler が翻訳）。
		return DynamicSecretResponse{}, errEmptyTenant
	}
	// テナント越境を弾く。
	if req.TenantID == "" {
		// テナント未指定は弾く。
		return DynamicSecretResponse{}, errEmptyTenant
	}
	// 排他取得。
	m.mu.Lock()
	defer m.mu.Unlock()
	// counter を進める。
	m.counter++
	// lease ID は engine + counter から決定的に生成する。
	leaseID := req.Engine + "/creds/" + req.Role + "/lease-" + itoaInline(m.counter)
	// random username / password を生成する（16 / 32 byte）。
	username := "u-" + randHex(8)
	password := randHex(16)
	// TTL clamp。
	ttl := clampTTL(req.TTLSeconds)
	// 発効時刻は now、期限は now + ttl。
	now := time.Now().UnixMilli()
	expires := now + int64(ttl)*1000
	// audit 用 lease record。
	m.leases[leaseID] = &leaseRecord{
		engine:      req.Engine,
		role:        req.Role,
		tenantID:    req.TenantID,
		issuedAtMs:  now,
		expiresAtMs: expires,
	}
	// 応答を組み立てる。
	return DynamicSecretResponse{
		Values: map[string]string{
			// production と互換の field 名（OpenBao Database Engine の標準）。
			"username": username,
			"password": password,
		},
		LeaseID:    leaseID,
		TTLSeconds: ttl,
		IssuedAtMs: now,
	}, nil
}

// itoaInline は依存最小の int → string 変換（strconv 不使用）。
func itoaInline(n int) string {
	if n == 0 {
		return "0"
	}
	neg := false
	if n < 0 {
		neg = true
		n = -n
	}
	buf := make([]byte, 0, 12)
	for n > 0 {
		buf = append([]byte{byte('0' + n%10)}, buf...)
		n /= 10
	}
	if neg {
		buf = append([]byte{'-'}, buf...)
	}
	return string(buf)
}

// randHex は n bytes の crypto/rand を hex string で返す。
func randHex(nBytes int) string {
	// バッファ確保。
	buf := make([]byte, nBytes)
	// crypto/rand から読み込む（戻り値 err は満杯で nil 以外あり得ないため捨てる）。
	_, _ = rand.Read(buf)
	// hex エンコードして返す。
	return hex.EncodeToString(buf)
}
