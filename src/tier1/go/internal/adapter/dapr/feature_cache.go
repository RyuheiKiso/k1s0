// 本ファイルは FeatureAdapter のキャッシュ層実装。
//
// 設計正典:
//   docs/03_要件定義/20_機能要件/10_tier1_API要件/11_Feature_API.md
//     - FR-T1-FEATURE-001 受け入れ基準「評価 p99 < 10ms（キャッシュヒット時）」
//     - FR-T1-FEATURE-004 受け入れ基準「評価結果のキャッシュ（30 秒 TTL）」
//
// 役割:
//   既存 FeatureAdapter を decorate し、Boolean / String / Number / Object 評価結果を
//   TTL 付き in-memory cache に保持する。flagd は Configuration API 経由で sidecar に
//   gRPC 1 hop 必要であり、cache hit ヒット時はそのコストを完全に避けて
//   p99 < 10ms（受け入れ基準）を成立させる。
//
// キャッシュキー:
//   (tenant_id, flag_key, evaluation_context_canonical, value_kind) の 4 タプル。
//   evaluation_context は targetingKey / userId / role 等を含むため、同 flag でも
//   context が違えば結果が異なる。canonical は key 昇順で k=v を | 連結した string。
//
// 不採用としたキー候補:
//   - flag_key + tenant_id のみ: targeting 結果を取り違える危険があるため不可
//   - sha256(全フィールド): 必要なら将来 hashing に切替可能だが、現状は string 連結で
//     十分（context は常識的に <100 バイト規模、map キーとして問題ない）

package dapr

import (
	// 全 adapter で context を伝搬する。
	"context"
	// canonical key 用の昇順ソート。
	"sort"
	// canonical key 用の文字列結合。
	"strings"
	// 並行アクセス保護に sync.Mutex を使う。
	"sync"
	// TTL 評価で時刻計算する。
	"time"
)

// 既定キャッシュ TTL（FR-T1-FEATURE-001 / 004 受け入れ基準: 30 秒）。
const defaultFeatureCacheTTL = 30 * time.Second

// featureCacheTTLCeiling は TTL 上限。30 秒の倍率を超える値（300 秒等）を環境変数で
// 設定された場合の保護として上限を設ける。flag 反映遅延が長すぎると緊急 kill switch が
// 効かなくなるリスクを抑える（共通規約「業務継続優先」と整合）。
const featureCacheTTLCeiling = 5 * time.Minute

// featureValueKind はキャッシュ entry の値型を区別する識別子。
// 同じ flag_key でも boolean / string / number / object で評価結果が異なるため
// 必ずキーに含める。
type featureValueKind int

const (
	// boolean 評価結果。
	featureKindBoolean featureValueKind = iota + 1
	// string 評価結果。
	featureKindString
	// number 評価結果。
	featureKindNumber
	// object 評価結果。
	featureKindObject
)

// featureCacheKey は flag 評価 cache の検索キー。
type featureCacheKey struct {
	// テナント識別子（テナント間で flag 値を漏らさない L2 的分離）。
	tenantID string
	// flag キー。
	flagKey string
	// evaluation_context を昇順 k=v|... で連結した正規形。
	contextCanon string
	// 値型（boolean / string / number / object）。
	kind featureValueKind
}

// featureCacheEntry は cache の 1 エントリ。値型ごとに異なる struct を抱えるため
// interface{} で保持する（cast は値型確定の getter で行う）。
type featureCacheEntry struct {
	// 値（FlagBooleanResponse / FlagStringResponse / FlagNumberResponse / FlagObjectResponse）。
	value interface{}
	// 失効時刻。time.Now() を上回ると expired。
	expireAt time.Time
}

// CachedFeatureAdapter は FeatureAdapter を wrap し、Evaluate 系 4 RPC に TTL キャッシュを付与する。
type CachedFeatureAdapter struct {
	// ベース adapter（flagd 直結 / mock 等）。
	base FeatureAdapter
	// キャッシュ TTL。0 で defaultFeatureCacheTTL（30 秒）が適用される。
	ttl time.Duration
	// 排他制御。
	mu sync.Mutex
	// cache 本体（map で型安全 + 削除コスト均一）。
	entries map[featureCacheKey]featureCacheEntry
}

// NewCachedFeatureAdapter は base を wrap した cache adapter を返す。
// ttl=0 / 負値 / featureCacheTTLCeiling 超は defaultFeatureCacheTTL（30 秒）にフォールバック。
func NewCachedFeatureAdapter(base FeatureAdapter, ttl time.Duration) *CachedFeatureAdapter {
	// ttl 既定補完 + 上限チェック。
	effective := ttl
	if effective <= 0 || effective > featureCacheTTLCeiling {
		// 既定 30 秒（FR-T1-FEATURE-001 / 004）。
		effective = defaultFeatureCacheTTL
	}
	// キャッシュ初期化。
	return &CachedFeatureAdapter{
		base:    base,
		ttl:     effective,
		entries: map[featureCacheKey]featureCacheEntry{},
	}
}

// canonicalContext は EvaluationContext を「昇順 k=v|...」の正規形 string にする。
// map の iteration 順は非決定なので明示的に昇順ソートする（cache hit 率の安定化）。
func canonicalContext(ctx map[string]string) string {
	if len(ctx) == 0 {
		return ""
	}
	keys := make([]string, 0, len(ctx))
	for k := range ctx {
		keys = append(keys, k)
	}
	sort.Strings(keys)
	parts := make([]string, 0, len(keys))
	for _, k := range keys {
		parts = append(parts, k+"="+ctx[k])
	}
	return strings.Join(parts, "|")
}

// makeFeatureKey は req と kind から cache key を作る。
func (c *CachedFeatureAdapter) makeFeatureKey(req FlagEvaluateRequest, kind featureValueKind) featureCacheKey {
	return featureCacheKey{
		tenantID:     req.TenantID,
		flagKey:      req.FlagKey,
		contextCanon: canonicalContext(req.EvaluationContext),
		kind:         kind,
	}
}

// lookup は key の有効な entry を返す。expired / 不在は (nil, false)。
func (c *CachedFeatureAdapter) lookup(key featureCacheKey, now time.Time) (interface{}, bool) {
	c.mu.Lock()
	defer c.mu.Unlock()
	entry, ok := c.entries[key]
	if !ok {
		return nil, false
	}
	if !entry.expireAt.After(now) {
		// expired entry を即時 GC（同 key の次回呼出で base hit させる）。
		delete(c.entries, key)
		return nil, false
	}
	return entry.value, true
}

// store は key に value を TTL 付きで保存する。
func (c *CachedFeatureAdapter) store(key featureCacheKey, value interface{}, now time.Time) {
	c.mu.Lock()
	defer c.mu.Unlock()
	c.entries[key] = featureCacheEntry{
		value:    value,
		expireAt: now.Add(c.ttl),
	}
}

// EvaluateBoolean は cache hit 時は base を呼ばずに値を返す。miss 時は base を呼んで TTL 付きで保存する。
//
// FR-T1-FEATURE-001 受け入れ基準: 「評価失敗時はデフォルト値を返す」要件のため、
// base error は cache に保存しない（次回呼出で再試行する）。
func (c *CachedFeatureAdapter) EvaluateBoolean(ctx context.Context, req FlagEvaluateRequest) (FlagBooleanResponse, error) {
	now := time.Now()
	key := c.makeFeatureKey(req, featureKindBoolean)
	if v, ok := c.lookup(key, now); ok {
		// cache hit: 型は store 時に確定しているため安全に cast する。
		return v.(FlagBooleanResponse), nil
	}
	resp, err := c.base.EvaluateBoolean(ctx, req)
	if err != nil {
		return resp, err
	}
	c.store(key, resp, now)
	return resp, nil
}

// EvaluateString は string 評価結果に同じ cache を適用する。
func (c *CachedFeatureAdapter) EvaluateString(ctx context.Context, req FlagEvaluateRequest) (FlagStringResponse, error) {
	now := time.Now()
	key := c.makeFeatureKey(req, featureKindString)
	if v, ok := c.lookup(key, now); ok {
		return v.(FlagStringResponse), nil
	}
	resp, err := c.base.EvaluateString(ctx, req)
	if err != nil {
		return resp, err
	}
	c.store(key, resp, now)
	return resp, nil
}

// EvaluateNumber は number 評価結果に同じ cache を適用する。
func (c *CachedFeatureAdapter) EvaluateNumber(ctx context.Context, req FlagEvaluateRequest) (FlagNumberResponse, error) {
	now := time.Now()
	key := c.makeFeatureKey(req, featureKindNumber)
	if v, ok := c.lookup(key, now); ok {
		return v.(FlagNumberResponse), nil
	}
	resp, err := c.base.EvaluateNumber(ctx, req)
	if err != nil {
		return resp, err
	}
	c.store(key, resp, now)
	return resp, nil
}

// EvaluateObject は object 評価結果に同じ cache を適用する。ValueJSON は []byte なので
// caller 側が誤って書き換えても cache が壊れないよう、defensive copy で保存する。
func (c *CachedFeatureAdapter) EvaluateObject(ctx context.Context, req FlagEvaluateRequest) (FlagObjectResponse, error) {
	now := time.Now()
	key := c.makeFeatureKey(req, featureKindObject)
	if v, ok := c.lookup(key, now); ok {
		// cache 内の bytes が呼出側に共有されないよう新しい slice を返す。
		entry := v.(FlagObjectResponse)
		copied := make([]byte, len(entry.ValueJSON))
		copy(copied, entry.ValueJSON)
		return FlagObjectResponse{
			ValueJSON: copied,
			Variant:   entry.Variant,
			Reason:    entry.Reason,
		}, nil
	}
	resp, err := c.base.EvaluateObject(ctx, req)
	if err != nil {
		return resp, err
	}
	// cache に保存する slice も呼出側のものと共有させない（caller が write すると
	// 次の hit で壊れた値が出回る）。
	stored := make([]byte, len(resp.ValueJSON))
	copy(stored, resp.ValueJSON)
	c.store(key, FlagObjectResponse{ValueJSON: stored, Variant: resp.Variant, Reason: resp.Reason}, now)
	return resp, nil
}

// TTL は test / 観測用の getter。本番経路から呼ばれることはない。
func (c *CachedFeatureAdapter) TTL() time.Duration { return c.ttl }
