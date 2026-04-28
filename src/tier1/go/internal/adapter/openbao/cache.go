// 本ファイルは SecretsAdapter のキャッシュ層実装。
//
// 設計正典:
//   docs/03_要件定義/20_機能要件/10_tier1_API要件/04_Secrets_API.md
//     - FR-T1-SECRETS-001（"取得結果は短時間（30 秒）のインメモリキャッシュで高速化"）
//     - "キャッシュ TTL（デフォルト 30 秒）は Component YAML で調整可能"
//
// 役割:
//   既存 SecretsAdapter を decorate し、Get 結果のみを TTL 付き sync.Map に保持する。
//   BulkGet / ListAndGet / Rotate はキャッシュを bypass する（これらは管理操作 / 一括
//   操作で、cache hit ratio が低く benefit が薄い）。
//   tenant_id / name / version をキーにする。version=0 は "latest" としてキャッシュする。
//   Rotate 成功時は同 tenant_id / name の latest を invalidate する。

package openbao

import (
	// 全 adapter で context を伝搬する。
	"context"
	// 並行アクセス保護に sync.Mutex を使う。
	"sync"
	// TTL 評価で時刻計算する。
	"time"
)

// 既定キャッシュ TTL（FR-T1-SECRETS-001 受け入れ基準: "デフォルト 30 秒"）。
const defaultSecretCacheTTL = 30 * time.Second

// secretCacheKey は cache の検索キー。
type secretCacheKey struct {
	tenantID string
	name     string
	// 0 は "latest" を表す。
	version int
}

// secretCacheEntry は cache の 1 エントリ。
type secretCacheEntry struct {
	resp     SecretGetResponse
	expireAt time.Time
}

// CachedSecretsAdapter は SecretsAdapter を wrap し、Get に TTL キャッシュを付与する。
type CachedSecretsAdapter struct {
	// ベース adapter（OpenBao 直結 / mock 等）。
	base SecretsAdapter
	// キャッシュ TTL。0 で無効（パススルー）、既定 30 秒。
	ttl time.Duration
	// 排他制御。
	mu sync.Mutex
	// cache 本体（map で型安全 + 削除コスト均一）。
	entries map[secretCacheKey]secretCacheEntry
}

// NewCachedSecretsAdapter は base を wrap した cache adapter を返す。
// ttl=0 を渡すと defaultSecretCacheTTL（30 秒）が使われる。
func NewCachedSecretsAdapter(base SecretsAdapter, ttl time.Duration) *CachedSecretsAdapter {
	// ttl 既定補完。
	effective := ttl
	if effective <= 0 {
		// 既定 30 秒（FR-T1-SECRETS-001）。
		effective = defaultSecretCacheTTL
	}
	// キャッシュ初期化。
	return &CachedSecretsAdapter{
		base:    base,
		ttl:     effective,
		entries: map[secretCacheKey]secretCacheEntry{},
	}
}

// makeKey は req から cache key を作る。
func makeKey(req SecretGetRequest) secretCacheKey {
	// version=0 は "latest" として扱う。
	return secretCacheKey{
		tenantID: req.TenantID,
		name:     req.Name,
		version:  req.Version,
	}
}

// Get は cache hit 時は base を呼ばずに即返す。miss 時は base を呼んで cache に格納する。
func (c *CachedSecretsAdapter) Get(ctx context.Context, req SecretGetRequest) (SecretGetResponse, error) {
	// cache 検索。
	if cached, ok := c.lookup(makeKey(req)); ok {
		// hit。
		return cached, nil
	}
	// miss。base に委譲する。
	resp, err := c.base.Get(ctx, req)
	if err != nil {
		// エラーは cache に入れない。
		return resp, err
	}
	// cache に格納する。
	c.store(makeKey(req), resp)
	return resp, nil
}

// BulkGet は cache を bypass し base に委譲する。
func (c *CachedSecretsAdapter) BulkGet(ctx context.Context, names []string, tenantID string) (map[string]SecretGetResponse, error) {
	// cache 通さず base に渡す。
	return c.base.BulkGet(ctx, names, tenantID)
}

// ListAndGet は cache を bypass し base に委譲する。
func (c *CachedSecretsAdapter) ListAndGet(ctx context.Context, tenantID string) (map[string]SecretGetResponse, error) {
	// cache 通さず base に渡す。
	return c.base.ListAndGet(ctx, tenantID)
}

// Rotate は base に委譲し、成功時は同 tenant_id / name の latest cache を invalidate する。
func (c *CachedSecretsAdapter) Rotate(ctx context.Context, req SecretRotateRequest) (SecretGetResponse, error) {
	// base で bump する。
	resp, err := c.base.Rotate(ctx, req)
	if err != nil {
		// 失敗時は cache 不変。
		return resp, err
	}
	// 成功時は対象 secret の latest cache を invalidate する。
	c.invalidate(secretCacheKey{tenantID: req.TenantID, name: req.Name, version: 0})
	return resp, nil
}

// lookup は cache を検索し、有効なら entry を返す。
func (c *CachedSecretsAdapter) lookup(key secretCacheKey) (SecretGetResponse, bool) {
	// 短時間ロック。
	c.mu.Lock()
	defer c.mu.Unlock()
	// エントリ取得。
	entry, ok := c.entries[key]
	if !ok {
		// cache miss。
		return SecretGetResponse{}, false
	}
	// 期限切れは miss 扱い + 削除する（lazy eviction）。
	if time.Now().After(entry.expireAt) {
		// 削除して miss を返す。
		delete(c.entries, key)
		return SecretGetResponse{}, false
	}
	// hit。
	return entry.resp, true
}

// store は entry を cache に格納する。
func (c *CachedSecretsAdapter) store(key secretCacheKey, resp SecretGetResponse) {
	// 短時間ロック。
	c.mu.Lock()
	defer c.mu.Unlock()
	// 期限を計算して格納する。
	c.entries[key] = secretCacheEntry{
		resp:     resp,
		expireAt: time.Now().Add(c.ttl),
	}
}

// invalidate は対象キーの cache を削除する（Rotate 成功時のみ呼ぶ）。
func (c *CachedSecretsAdapter) invalidate(key secretCacheKey) {
	// 排他削除。
	c.mu.Lock()
	defer c.mu.Unlock()
	// エントリ削除（不在時は no-op）。
	delete(c.entries, key)
}
