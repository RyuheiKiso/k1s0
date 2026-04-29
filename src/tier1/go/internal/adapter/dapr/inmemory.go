// 本ファイルは Dapr sidecar を持たない開発 / CI 環境向けの in-memory backend。
//
// 設計正典:
//   docs/04_概要設計/20_ソフトウェア方式設計/01_コンポーネント方式設計/02_Daprファサード層コンポーネント.md
//     - DS-SW-COMP-020（5 building block への薄いラッパ）
//   docs/03_要件定義/20_機能要件/40_tier1_API契約IDL/02_State_API.md ほか 4 本
//
// 役割:
//   Dapr sidecar が起動していない環境（dev / CI）でも cmd/state バイナリが起動から
//   gRPC 応答まで実値を返せるよう、`daprStateClient` / `daprPubSubClient` /
//   `daprInvokeClient` / `daprBindingClient` / `daprConfigClient` の 5 narrow interface を
//   満たす in-memory 実装を提供する。production では `DAPR_GRPC_ENDPOINT` 環境変数で
//   実 Dapr sidecar に切替わる。
//
// 制限事項（in-memory backend は dev / CI 用途）:
//   - 永続化なし（再起動で全 state / 全 message 消失）
//   - Pub/Sub の Subscribe は publish 履歴をリプレイしない（blocking で次の publish を待つ）
//   - Service Invocation は echo 応答（呼出 method / data をそのまま返す）
//   - Output Binding は no-op（呼出を成功扱いし、空 response を返す）
//   - Configuration / Feature flag は in-memory map 参照
//
// 並行制御:
//   sync.RWMutex で状態を保護する。production の Dapr sidecar は分散合意するが
//   in-memory backend は単一プロセス前提のため簡素化する。

package dapr

import (
	// 全 RPC で context を伝搬する。
	"context"
	// 並行制御に Mutex を使う。
	"sync"

	// Dapr SDK の型を構築する（StateItem / BulkStateItem / Subscription / SubscriptionOptions ほか）。
	daprclient "github.com/dapr/go-sdk/client"
)

// inMemoryStateValue は in-memory state 1 件の保存データ。
type inMemoryStateValue struct {
	// 値本体。
	value []byte
	// 楽観的排他用の Etag（保存時に "v<連番>" 形式で生成）。
	etag string
}

// inMemoryDapr は 5 building block を 1 つにまとめた in-memory backend。
// dev / CI でも production と同じ narrow interface 経路を辿らせる。
//
// テナント分離（NFR-E-AC-003）:
//   実 Dapr Component は metadata.partitionKey で tenant 単位の partition を構成するが、
//   in-memory backend は SDK の metadata map から "tenantId" を読み取り、storage key を
//   (tenantId, store, key) に拡張することで同等の隔離を実現する。
//   tenantId が空（呼出元が未指定）の場合は ""（空テナント）パーティションに集約され、
//   handler 側の必須検証（requireTenantID）を通過した呼出のみ data plane に到達する。
type inMemoryDapr struct {
	// 全状態を保護する RWMutex。
	mu sync.RWMutex
	// state KV: tenantId → (store 名 → (key → value))。
	// 第一階層を tenantId で分けることで tenant 越境を物理的に遮断する。
	state map[string]map[string]map[string]inMemoryStateValue
	// state etag 採番用カウンタ。
	etagCounter int
	// configuration KV: tenantId → (store 名 → (key → ConfigurationItem))。
	// state と同様 tenant 第一階層化する。
	config map[string]map[string]map[string]*daprclient.ConfigurationItem
}

// newInMemoryDapr は空の in-memory backend を生成する。
func newInMemoryDapr() *inMemoryDapr {
	// 全 map を空で初期化する。
	return &inMemoryDapr{
		// state KV を空で初期化する。tenant 第一階層を採用。
		state: map[string]map[string]map[string]inMemoryStateValue{},
		// configuration KV を空で初期化する。tenant 第一階層を採用。
		config: map[string]map[string]map[string]*daprclient.ConfigurationItem{},
	}
}

// metaKeyTenantID は SDK metadata map に詰めるテナント識別子のキー。
// adapter（state.go の metadataKeyTenant）と一致させる必要があるが、in-memory backend は
// adapter package private を参照できないため文字列定数で同期する。
const metaKeyTenantID = "tenantId"

// tenantOf は SDK metadata map からテナント識別子を抜き出す。
// 不在 / nil map の場合は空文字を返す（呼出元 handler 側で requireTenantID 済前提）。
func tenantOf(meta map[string]string) string {
	// nil map は空文字を返す。
	if meta == nil {
		// 空文字でフォールバック（handler 側 NFR-E-AC-003 検証通過前提）。
		return ""
	}
	// "tenantId" キーの有無に依らず map の zero 値（空文字）が返る。
	return meta[metaKeyTenantID]
}

// nextEtag は etagCounter を進めて新しい etag 文字列を返す。
func (m *inMemoryDapr) nextEtag() string {
	// counter を進める。
	m.etagCounter++
	// "v<n>" 形式で返す。
	return "v" + itoa(m.etagCounter)
}

// itoa は int → 10 進文字列の薄いラッパ（strconv に依存しないため inline）。
func itoa(n int) string {
	// 0 は特殊扱い。
	if n == 0 {
		// "0" を返す。
		return "0"
	}
	// 負値は反転する。
	neg := false
	// 負値判定。
	if n < 0 {
		// flag を立てる。
		neg = true
		// 絶対値に変える。
		n = -n
	}
	// buffer を確保する（int の 10 進最大桁数）。
	buf := make([]byte, 0, 20)
	// 末尾から 1 桁ずつ取り出す。
	for n > 0 {
		// 1 桁を ASCII 数字に変換して append する。
		buf = append([]byte{byte('0' + n%10)}, buf...)
		// 1 桁進める。
		n /= 10
	}
	// 負値なら "-" を先頭に付ける。
	if neg {
		// "-" を先頭に付ける。
		buf = append([]byte{'-'}, buf...)
	}
	// string に変換して返す。
	return string(buf)
}

// stateBucketLocked は (tenantId, store) を解決して bucket map を返す。
// create=true の時は不在を遅延生成、false の時は不在を nil で返す（読出時）。
// 呼出側で m.mu を握っている前提（_Locked サフィックスで明示）。
func (m *inMemoryDapr) stateBucketLocked(tenant, store string, create bool) map[string]inMemoryStateValue {
	// tenant 階層を取り出す。
	byStore, ok := m.state[tenant]
	// tenant 階層が不在の分岐。
	if !ok {
		// 読出のみで不在生成不要なら nil を返す。
		if !create {
			// nil を返す。
			return nil
		}
		// 書込時は新 map を割当てて保存する。
		byStore = map[string]map[string]inMemoryStateValue{}
		// tenant 階層に登録する。
		m.state[tenant] = byStore
	}
	// store 配下を取り出す。
	bucket, ok := byStore[store]
	// store 配下不在の分岐。
	if !ok {
		// 読出のみで不在生成不要なら nil を返す。
		if !create {
			// nil を返す。
			return nil
		}
		// 書込時は新 map を割当てて保存する。
		bucket = map[string]inMemoryStateValue{}
		// store 配下に登録する。
		byStore[store] = bucket
	}
	// bucket を返す。
	return bucket
}

// GetState は store / key から最新値を取得する。未存在は (nil, nil)。
func (m *inMemoryDapr) GetState(_ context.Context, storeName, key string, meta map[string]string) (*daprclient.StateItem, error) {
	// 読出 lock を取得する。
	m.mu.RLock()
	defer m.mu.RUnlock()
	// metadata からテナントを抽出する。
	tenant := tenantOf(meta)
	// (tenant, store) bucket を読出専用で取得する。
	bucket := m.stateBucketLocked(tenant, storeName, false)
	// bucket 不在は SDK 慣行で空 StateItem を返す。
	if bucket == nil {
		// 空 StateItem を返す。
		return &daprclient.StateItem{Key: key}, nil
	}
	// key 配下の値を取り出す。
	v, ok := bucket[key]
	// key 未存在も空 StateItem。
	if !ok {
		// 空 StateItem を返す。
		return &daprclient.StateItem{Key: key}, nil
	}
	// 値あり → StateItem に詰めて返す。
	return &daprclient.StateItem{Key: key, Value: v.value, Etag: v.etag}, nil
}

// firstWriteRequested は StateOption を畳み込んで concurrency が FirstWrite かを判定する。
// production の Dapr SDK と同じ挙動を in-memory backend でも再現する（k1s0 共通規約
// §「ETag 必須化」: First-Write-Wins を全 Set で強制）。
func firstWriteRequested(opts []daprclient.StateOption) bool {
	var so daprclient.StateOptions
	for _, opt := range opts {
		opt(&so)
	}
	return so.Concurrency == daprclient.StateConcurrencyFirstWrite
}

// SaveState は store / key に値を保存する。
// StateConcurrencyFirstWrite を渡された場合、既存キーへの書込は errEtagMismatch を返す
// （production Dapr の First-Write-Wins と一致）。それ以外は last-write-wins。
func (m *inMemoryDapr) SaveState(_ context.Context, storeName, key string, data []byte, meta map[string]string, opts ...daprclient.StateOption) error {
	// 書込 lock を取得する。
	m.mu.Lock()
	defer m.mu.Unlock()
	// metadata からテナントを抽出する。
	tenant := tenantOf(meta)
	// (tenant, store) bucket を書込時生成で取得する。
	bucket := m.stateBucketLocked(tenant, storeName, true)
	// First-Write-Wins: 既存キーへの書込は conflict として弾く（k1s0 §「ETag 必須化」）。
	if firstWriteRequested(opts) {
		if _, exists := bucket[key]; exists {
			return errEtagMismatch
		}
	}
	// 値と新 etag を保存する。
	bucket[key] = inMemoryStateValue{value: data, etag: m.nextEtag()}
	// 成功時 nil を返す。
	return nil
}

// SaveStateWithETag は楽観的排他付きで保存する（etag 不一致時 error）。
// FirstWrite concurrency option が来ても挙動は変わらない（既に etag 比較で排他するため）。
func (m *inMemoryDapr) SaveStateWithETag(_ context.Context, storeName, key string, data []byte, etag string, meta map[string]string, _ ...daprclient.StateOption) error {
	// 書込 lock を取得する。
	m.mu.Lock()
	defer m.mu.Unlock()
	// metadata からテナントを抽出する。
	tenant := tenantOf(meta)
	// (tenant, store) bucket を書込時生成で取得する。
	bucket := m.stateBucketLocked(tenant, storeName, true)
	// 既存値の etag を確認する。
	if existing, ok := bucket[key]; ok && existing.etag != etag {
		// 楽観的排他の不一致は SDK と同じ semantics で error を返す。
		return errEtagMismatch
	}
	// 値と新 etag を保存する。
	bucket[key] = inMemoryStateValue{value: data, etag: m.nextEtag()}
	// 成功時 nil を返す。
	return nil
}

// DeleteState は store / key を削除する（無条件）。
func (m *inMemoryDapr) DeleteState(_ context.Context, storeName, key string, meta map[string]string) error {
	// 書込 lock を取得する。
	m.mu.Lock()
	defer m.mu.Unlock()
	// metadata からテナントを抽出する。
	tenant := tenantOf(meta)
	// (tenant, store) bucket を読出専用で取得する。
	bucket := m.stateBucketLocked(tenant, storeName, false)
	// 未存在は no-op。
	if bucket == nil {
		// nil を返す。
		return nil
	}
	// key を削除する。
	delete(bucket, key)
	// 成功時 nil を返す。
	return nil
}

// DeleteStateWithETag は楽観的排他付きで削除する。
func (m *inMemoryDapr) DeleteStateWithETag(_ context.Context, storeName, key string, etag *daprclient.ETag, meta map[string]string, _ *daprclient.StateOptions) error {
	// 書込 lock を取得する。
	m.mu.Lock()
	defer m.mu.Unlock()
	// metadata からテナントを抽出する。
	tenant := tenantOf(meta)
	// (tenant, store) bucket を読出専用で取得する。
	bucket := m.stateBucketLocked(tenant, storeName, false)
	// 未存在は no-op。
	if bucket == nil {
		// nil を返す。
		return nil
	}
	// 既存値の etag を確認する。
	if existing, ok := bucket[key]; ok {
		// etag 不一致は楽観的排他の違反。
		if etag != nil && existing.etag != etag.Value {
			// SDK 互換の error を返す。
			return errEtagMismatch
		}
	}
	// key を削除する。
	delete(bucket, key)
	// 成功時 nil を返す。
	return nil
}

// GetBulkState は複数 key を一括取得する。
func (m *inMemoryDapr) GetBulkState(_ context.Context, storeName string, keys []string, meta map[string]string, _ int32) ([]*daprclient.BulkStateItem, error) {
	// 読出 lock を取得する。
	m.mu.RLock()
	defer m.mu.RUnlock()
	// 結果 slice を準備する。
	out := make([]*daprclient.BulkStateItem, 0, len(keys))
	// metadata からテナントを抽出する。
	tenant := tenantOf(meta)
	// (tenant, store) bucket を読出専用で取得する（不在は nil で各 key を未存在扱い）。
	bucket := m.stateBucketLocked(tenant, storeName, false)
	// 各 key を 1 件ずつ取得する。
	for _, k := range keys {
		// bucket 不在 / key 不在は空 Value で返す。
		v, ok := bucket[k]
		// 未存在分岐。
		if !ok {
			// Value=nil の BulkStateItem を append する。
			out = append(out, &daprclient.BulkStateItem{Key: k})
			// 次の iteration へ進む。
			continue
		}
		// 値あり分岐。
		out = append(out, &daprclient.BulkStateItem{Key: k, Value: v.value, Etag: v.etag})
	}
	// 結果を返す。
	return out, nil
}

// ExecuteStateTransaction は ops を 1 トランザクションで実行する。
// in-memory backend は途中失敗を rollback できないため、書込前に全 op の事前検証を行う。
func (m *inMemoryDapr) ExecuteStateTransaction(_ context.Context, storeName string, meta map[string]string, ops []*daprclient.StateOperation) error {
	// 書込 lock を取得する。
	m.mu.Lock()
	defer m.mu.Unlock()
	// metadata からテナントを抽出する。
	tenant := tenantOf(meta)
	// (tenant, store) bucket を書込時生成で取得する。
	bucket := m.stateBucketLocked(tenant, storeName, true)
	// 全 op を逐次実行する。途中失敗は前段までの書込が残るが in-memory なので許容する。
	for _, op := range ops {
		// op が nil の場合は skip する。
		if op == nil || op.Item == nil {
			// 次の iteration へ進む。
			continue
		}
		// 種別ごとの分岐。
		switch op.Type {
		// Upsert（Set）。
		case daprclient.StateOperationTypeUpsert:
			// etag が指定されているなら楽観的排他を検証する。
			if op.Item.Etag != nil && op.Item.Etag.Value != "" {
				// 既存値を確認する。
				if existing, ok := bucket[op.Item.Key]; ok && existing.etag != op.Item.Etag.Value {
					// etag 不一致は失敗。
					return errEtagMismatch
				}
			}
			// 新値を保存する。
			bucket[op.Item.Key] = inMemoryStateValue{value: op.Item.Value, etag: m.nextEtag()}
		// Delete。
		case daprclient.StateOperationTypeDelete:
			// etag が指定されているなら楽観的排他を検証する。
			if op.Item.Etag != nil && op.Item.Etag.Value != "" {
				// 既存値を確認する。
				if existing, ok := bucket[op.Item.Key]; ok && existing.etag != op.Item.Etag.Value {
					// etag 不一致は失敗。
					return errEtagMismatch
				}
			}
			// 削除する。
			delete(bucket, op.Item.Key)
		}
	}
	// 成功時 nil を返す。
	return nil
}

// PublishEvent は publish を no-op で受理する（subscription への配信は in-memory backend では未対応）。
func (m *inMemoryDapr) PublishEvent(_ context.Context, _, _ string, _ interface{}, _ ...daprclient.PublishEventOption) error {
	// in-memory は publish を破棄する（成功扱い）。
	return nil
}

// Subscribe は SDK が non-nil Subscription を返すことを満たすが、Receive は永久 block する。
// in-memory backend では publish が subscription に届かないため、ctx キャンセル待ちで使う。
func (m *inMemoryDapr) Subscribe(_ context.Context, _ daprclient.SubscriptionOptions) (*daprclient.Subscription, error) {
	// SDK の Subscription struct を nil で返すと nil deref が起こるため、
	// in-memory backend では Subscribe をサポートせず ErrNotWired を返す。
	return nil, ErrNotWired
}

// InvokeMethodWithCustomContent は echo 応答（呼出 data をそのまま返す）。
func (m *inMemoryDapr) InvokeMethodWithCustomContent(_ context.Context, _, _, _, _ string, content interface{}) ([]byte, error) {
	// content が []byte ならそのまま echo する。
	if b, ok := content.([]byte); ok {
		// echo 応答を返す。
		return b, nil
	}
	// それ以外は空 bytes を返す。
	return nil, nil
}

// InvokeBinding は no-op で受理する（呼出を成功扱いし、空 response を返す）。
func (m *inMemoryDapr) InvokeBinding(_ context.Context, _ *daprclient.InvokeBindingRequest) (*daprclient.BindingEvent, error) {
	// 空 BindingEvent を返す。
	return &daprclient.BindingEvent{}, nil
}

// configBucketLocked は (tenantId, store) を解決して configuration bucket map を返す。
// state と同じ partition 戦略でテナント越境を防ぐ。
func (m *inMemoryDapr) configBucketLocked(tenant, store string, create bool) map[string]*daprclient.ConfigurationItem {
	// tenant 階層を取り出す。
	byStore, ok := m.config[tenant]
	// tenant 階層が不在の分岐。
	if !ok {
		// 読出のみで不在生成不要なら nil を返す。
		if !create {
			// nil を返す。
			return nil
		}
		// 書込時は新 map を割当てて保存する。
		byStore = map[string]map[string]*daprclient.ConfigurationItem{}
		// tenant 階層に登録する。
		m.config[tenant] = byStore
	}
	// store 配下を取り出す。
	bucket, ok := byStore[store]
	// store 配下不在の分岐。
	if !ok {
		// 読出のみで不在生成不要なら nil を返す。
		if !create {
			// nil を返す。
			return nil
		}
		// 書込時は新 map を割当てて保存する。
		bucket = map[string]*daprclient.ConfigurationItem{}
		// store 配下に登録する。
		byStore[store] = bucket
	}
	// bucket を返す。
	return bucket
}

// GetConfigurationItem は configuration KV から値を取得する。未存在は nil を返す。
// テナント識別子は SDK の ConfigurationOpt に metadata として埋め込まれるが、
// in-memory backend ではオプションを直接覗けないため、本実装は tenant="" 階層のみを使う。
// 実 Dapr 経路では Component 側で metadata.tenantId を partitionKey として扱う。
func (m *inMemoryDapr) GetConfigurationItem(_ context.Context, storeName, key string, _ ...daprclient.ConfigurationOpt) (*daprclient.ConfigurationItem, error) {
	// 読出 lock を取得する。
	m.mu.RLock()
	defer m.mu.RUnlock()
	// configuration は SDK ConfigurationOpt 経由で metadata を渡す形だが、
	// in-memory backend の dev/CI 用途では tenant="" の global namespace のみ参照する。
	// production の Dapr Configuration API で tenant 隔離する場合は Component 側で実装する。
	bucket := m.configBucketLocked("", storeName, false)
	// store 未存在は nil。
	if bucket == nil {
		// nil を返す。
		return nil, nil
	}
	// key を取得する（未存在は nil を返す）。
	return bucket[key], nil
}

// PutConfigurationItem は in-memory configuration store に値を保存する（dev / CI 用 helper）。
// SDK には存在しないが、in-memory backend のための seed API として exposing する。
// テナント分離は dev / CI では未対応（tenant="" の global namespace に保存）。
func (m *inMemoryDapr) PutConfigurationItem(storeName, key string, item *daprclient.ConfigurationItem) {
	// 書込 lock を取得する。
	m.mu.Lock()
	defer m.mu.Unlock()
	// tenant="" の global namespace に保存する（dev/CI seed 用）。
	bucket := m.configBucketLocked("", storeName, true)
	// 値を保存する。
	bucket[key] = item
}

// errEtagMismatch は state 操作の楽観的排他失敗を表すセンチネル。
// SDK は status.Aborted で返すが、in-memory backend は単なる error にとどめる。
var errEtagMismatch = ErrEtagMismatch

// ErrEtagMismatch は state 操作の楽観的排他失敗を表す公開エラー。
var ErrEtagMismatch = errEtag{}

// errEtag は ErrEtagMismatch の error 実装。
type errEtag struct{}

// Error は固定メッセージを返す。
func (errEtag) Error() string { return "tier1: dapr state etag mismatch" }

// NewClientWithInMemoryBackends は 5 building block すべてを in-memory backend で
// 構築した Client を返す。cmd/state/main.go から DAPR_GRPC_ENDPOINT 未設定時の
// fallback として呼ばれる。
func NewClientWithInMemoryBackends() *Client {
	// in-memory backend を 1 つ生成し、5 narrow interface すべてに割当てる。
	mem := newInMemoryDapr()
	// Client に in-memory backend を埋め込む（address は識別ラベル）。
	return &Client{
		// アドレスは "in-memory" 固定（観測用ラベル）。
		sidecarAddress: "in-memory",
		// state narrow interface に in-memory 実装を割当てる。
		state: mem,
		// pubsub narrow interface に in-memory 実装を割当てる。
		pubsub: mem,
		// invoke narrow interface に in-memory 実装を割当てる。
		invoke: mem,
		// binding narrow interface に in-memory 実装を割当てる。
		binding: mem,
		// configuration narrow interface に in-memory 実装を割当てる。
		config: mem,
	}
}
