// 本ファイルは Dapr in-memory backend のうち State 以外（PubSub stub / Invoke / Binding /
// Configuration）と 5 building block 一括組立 factory（NewClientWithInMemoryBackends）を担う。
//
// 設計正典:
//   docs/04_概要設計/20_ソフトウェア方式設計/01_コンポーネント方式設計/02_Daprファサード層コンポーネント.md
//     - DS-SW-COMP-020（5 building block への薄いラッパ、in-memory backend を含む）
//
// 分割の経緯:
//   元 inmemory.go が 518 行となり src/CLAUDE.md「1 ファイル 500 行以内」を超過していたため、
//   State 系（バケット管理 / GetState / SaveState / SaveStateWithETag / DeleteState /
//   DeleteStateWithETag / GetBulkState / ExecuteStateTransaction）を inmemory.go に残し、
//   PubSub / Invoke / Binding / Configuration の小型 stub と factory / sentinel error を本ファイルに移した。

package dapr

import (
	// 全 RPC で context を伝搬する。
	"context"

	// Dapr SDK の型を構築する。
	daprclient "github.com/dapr/go-sdk/client"
)

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
	// in-memory pubsub bus を生成する（NewPubSubAdapter から参照される）。
	bus := newPubSubBus()
	// Client に in-memory backend を埋め込む（address は識別ラベル）。
	return &Client{
		// アドレスは "in-memory" 固定（観測用ラベル）。
		sidecarAddress: "in-memory",
		// state narrow interface に in-memory 実装を割当てる。
		state: mem,
		// pubsub narrow interface に in-memory 実装を割当てる（NewPubSubAdapter は bus を優先する）。
		pubsub: mem,
		// invoke narrow interface に in-memory 実装を割当てる。
		invoke: mem,
		// binding narrow interface に in-memory 実装を割当てる。
		binding: mem,
		// configuration narrow interface に in-memory 実装を割当てる。
		config: mem,
		// in-memory pubsub bus（PubSubAdapter が non-nil 時に SDK 経路を bypass する）。
		pubsubBus: bus,
	}
}
