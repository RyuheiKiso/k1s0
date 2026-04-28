// 本ファイルは Dapr State Management building block のアダプタ。
//
// 設計正典:
//   docs/04_概要設計/20_ソフトウェア方式設計/01_コンポーネント方式設計/02_Daprファサード層コンポーネント.md
//     - State API → Valkey Cluster（Dapr State Management）
//   docs/03_要件定義/20_機能要件/40_tier1_API契約IDL/02_State_API.md
//
// 役割（plan 04-04 結線済）:
//   handler.go が呼び出す I/O を封じ込め、Dapr Go SDK の State Management API を
//   narrow interface（dapr.go の daprStateClient）越しに呼び出す。
//   テナント prefix / TTL / 楽観的排他（ETag）の翻訳もここで担う。
//
// テナント prefix（L2 物理分離、NFR-E-AC-003 / 共通規約 §「マルチテナント分離」）:
//   docs/03_要件定義/20_機能要件/10_tier1_API要件/00_tier1_API共通規約.md の
//   "L2（ルーティング）: バックエンドのキー / トピック / バケット / パーティションに
//    `<tenant_id>/` を自動付与" 要件を adapter 層で強制する（tenant_prefix.go）。
//   handler から渡された Key を物理 SDK 呼出前に `<tenant_id>/` で wrap し、
//   GetBulkState 等の応答キーは strip して tier2/tier3 に透過させる。
//   metadata.tenantId は Dapr Component 側の partition / ACL 連携用に併送する
//   （Kafka ACL / OpenBao Policy など Component 固有の経路で利用）。
//
// 楽観的排他（ETag）:
//   ExpectedEtag が空のリクエスト → SaveState / DeleteState（無条件）
//   ExpectedEtag が非空のリクエスト → SaveStateWithETag / DeleteStateWithETag
//   conflict 時、Dapr SDK は status code を含む error を返す。本層では透過的に上位へ。

package dapr

import (
	// 全 RPC で context を伝搬する。
	"context"
	// 想定外の TransactOpKind に対する error 整形。
	"fmt"
	// TTL 秒数を string として metadata に詰めるため。
	"strconv"

	// Dapr SDK の State 関連型を参照する（ETag struct や StateItem）。
	daprclient "github.com/dapr/go-sdk/client"
)

// metadataKeyTenant は Dapr metadata に詰めるテナント識別子のキー。
// Component 設定側で `metadata.partitionKey` 等に対応付ける運用想定。
const metadataKeyTenant = "tenantId"

// metadataKeyTTL は Dapr State の TTL 指定キー（Dapr SDK 内部定義に整合）。
const metadataKeyTTL = "ttlInSeconds"

// StateGetRequest は State Get 操作の adapter 入力。
// proto の k1s0.tier1.state.v1.GetRequest と等価だが、handler 側で TenantContext
// を分離して渡す形にする（Dapr SDK が tenant prefix を付与）。
type StateGetRequest struct {
	// Dapr Component 名（例: valkey-default）。
	Store string
	// テナント prefix 付与済キー。
	Key string
	// テナント識別子（Dapr metadata に渡す）。
	TenantID string
}

// StateGetResponse は State Get の応答。
type StateGetResponse struct {
	// 値本文（bytes 透過）。
	Data []byte
	// 楽観的排他用の ETag。
	Etag string
	// キー未存在時 true。
	NotFound bool
}

// StateSetRequest は Set / Delete 共通の入力。
type StateSetRequest struct {
	// Dapr Component 名。
	Store string
	// キー。
	Key string
	// 値本文（Set 時のみ）。
	Data []byte
	// 期待 ETag（楽観的排他、空は無条件）。
	ExpectedEtag string
	// TTL 秒数（0 で永続）。
	TTLSeconds int32
	// テナント識別子。
	TenantID string
}

// StateSetResponse は Set 応答（新 ETag を含む）。
type StateSetResponse struct {
	// 保存後の ETag。Dapr SDK の SaveState は ETag を返さないため、
	// 後続 Get で取得するか、Component 側が ETag を生成しない場合は空。
	NewEtag string
}

// StateAdapter は State Management building block の操作集合。
// handler 側は本 interface に依存し、テスト時は mock 実装を注入できる。
type StateAdapter interface {
	// 単一キー取得。
	Get(ctx context.Context, req StateGetRequest) (StateGetResponse, error)
	// 単一キー保存。
	Set(ctx context.Context, req StateSetRequest) (StateSetResponse, error)
	// 単一キー削除。
	Delete(ctx context.Context, req StateSetRequest) error
	// 複数キーの一括取得（parallelism は呼び出し時の同時実行 worker 数、0 で SDK 既定）。
	BulkGet(ctx context.Context, req StateBulkGetRequest) ([]StateBulkGetItem, error)
	// 複数操作（Set / Delete）の transactional 実行。
	Transact(ctx context.Context, req StateTransactRequest) error
}

// StateBulkGetRequest は BulkGet の入力。
type StateBulkGetRequest struct {
	Store       string
	Keys        []string
	TenantID    string
	Parallelism int32
}

// StateBulkGetItem は BulkGet の応答 1 件。
type StateBulkGetItem struct {
	Key      string
	Data     []byte
	Etag     string
	NotFound bool
	Error    string
}

// TransactOpKind は Transact 内の 1 操作の種別。
type TransactOpKind int

const (
	TransactOpSet    TransactOpKind = 1
	TransactOpDelete TransactOpKind = 2
)

// TransactOp は Transact の 1 操作。
type TransactOp struct {
	Kind TransactOpKind
	// Set 時: Key + Data + ExpectedEtag + TTLSeconds、Delete 時: Key + ExpectedEtag。
	Key          string
	Data         []byte
	ExpectedEtag string
	TTLSeconds   int32
}

// StateTransactRequest は Transact の入力（複数 ops を 1 トランザクションで実行）。
type StateTransactRequest struct {
	Store    string
	Ops      []TransactOp
	TenantID string
}

// daprStateAdapter は Client（narrow interface）を介して Dapr SDK を呼ぶ実装。
type daprStateAdapter struct {
	// Dapr Client への参照。state-用 narrow interface（daprStateClient）を持つ。
	client *Client
}

// NewStateAdapter は Client から StateAdapter を生成する。
func NewStateAdapter(client *Client) StateAdapter {
	return &daprStateAdapter{client: client}
}

// buildMeta はテナント識別子と TTL を Dapr metadata map に変換する。
func buildMeta(tenantID string, ttlSeconds int32) map[string]string {
	// 空 map ではなく nil を返すと Dapr SDK 側で metadata 不在と扱う。
	if tenantID == "" && ttlSeconds == 0 {
		return nil
	}
	// 必要なキーのみを含める（不要キーは nil 値として送らない）。
	meta := make(map[string]string, 2)
	if tenantID != "" {
		meta[metadataKeyTenant] = tenantID
	}
	if ttlSeconds > 0 {
		meta[metadataKeyTTL] = strconv.FormatInt(int64(ttlSeconds), 10)
	}
	return meta
}

// Get は単一キーを Dapr State から取得する。
// SDK が StateItem.Value == nil を返した場合は NotFound=true で応答する。
// 物理キーは prefixKey で `<tenant_id>/` を付与する（L2 テナント分離）。
func (a *daprStateAdapter) Get(ctx context.Context, req StateGetRequest) (StateGetResponse, error) {
	// L2 テナント分離: 物理キーに `<tenant_id>/` を付与する。
	physKey := prefixKey(req.TenantID, req.Key)
	// Dapr SDK 呼び出し。Component 側 store 名が空の場合は SDK が即時 InvalidArgument を返す。
	item, err := a.client.stateClient().GetState(ctx, req.Store, physKey, buildMeta(req.TenantID, 0))
	if err != nil {
		// 接続不可 / Component 未定義 等を上位へ透過する。
		return StateGetResponse{}, err
	}
	// SDK は item.Value == nil でキー未存在を表現する。
	if item == nil || len(item.Value) == 0 {
		return StateGetResponse{NotFound: true}, nil
	}
	// 値とともに Etag を返却する。
	return StateGetResponse{
		Data:     item.Value,
		Etag:     item.Etag,
		NotFound: false,
	}, nil
}

// Set は単一キーを Dapr State に保存する。
// ExpectedEtag が空なら SaveState、非空なら SaveStateWithETag を選択する。
// 物理キーは prefixKey で `<tenant_id>/` を付与する（L2 テナント分離）。
func (a *daprStateAdapter) Set(ctx context.Context, req StateSetRequest) (StateSetResponse, error) {
	// L2 テナント分離: 物理キーに `<tenant_id>/` を付与する。
	physKey := prefixKey(req.TenantID, req.Key)
	// metadata 構築（テナント + TTL）。
	meta := buildMeta(req.TenantID, req.TTLSeconds)
	// 楽観的排他の有無で SDK メソッドを切り替える。
	if req.ExpectedEtag == "" {
		if err := a.client.stateClient().SaveState(ctx, req.Store, physKey, req.Data, meta); err != nil {
			return StateSetResponse{}, err
		}
	} else {
		if err := a.client.stateClient().SaveStateWithETag(ctx, req.Store, physKey, req.Data, req.ExpectedEtag, meta); err != nil {
			return StateSetResponse{}, err
		}
	}
	// SDK の SaveState は ETag を返さないため空文字を返す（必要時は handler で Get を再実行）。
	return StateSetResponse{NewEtag: ""}, nil
}

// BulkGet は複数キーを Dapr GetBulkState で一括取得する。
// parallelism が 0 なら SDK の既定値を使う（実装は内部で min(len(keys), 100)）。
// 物理キーは prefixKeys で `<tenant_id>/` を付与し、応答は stripKey で剥がして tier2/tier3 に透過させる。
func (a *daprStateAdapter) BulkGet(ctx context.Context, req StateBulkGetRequest) ([]StateBulkGetItem, error) {
	parallelism := req.Parallelism
	if parallelism <= 0 {
		parallelism = 10
	}
	// L2 テナント分離: 物理キー列を `<tenant_id>/` で wrap する。
	physKeys := prefixKeys(req.TenantID, req.Keys)
	items, err := a.client.stateClient().GetBulkState(ctx, req.Store, physKeys, buildMeta(req.TenantID, 0), parallelism)
	if err != nil {
		return nil, err
	}
	out := make([]StateBulkGetItem, 0, len(items))
	for _, it := range items {
		// Dapr は未存在 / エラーを Item.Error フィールドで表現する。
		not_found := it.Error == "" && len(it.Value) == 0
		// 応答キーから `<tenant_id>/` を剥がして tier2/tier3 視点のキーに戻す。
		out = append(out, StateBulkGetItem{
			Key:      stripKey(req.TenantID, it.Key),
			Data:     it.Value,
			Etag:     it.Etag,
			NotFound: not_found,
			Error:    it.Error,
		})
	}
	return out, nil
}

// Transact は複数 ops を 1 トランザクションで実行する。
// Dapr SDK の ExecuteStateTransaction を呼び、ops を SDK の StateOperation 列に変換する。
// 各 op の Key は prefixKey で `<tenant_id>/` を付与する（L2 テナント分離）。
func (a *daprStateAdapter) Transact(ctx context.Context, req StateTransactRequest) error {
	dops := make([]*daprclient.StateOperation, 0, len(req.Ops))
	for _, op := range req.Ops {
		// L2 テナント分離: 物理キーに `<tenant_id>/` を付与する。
		physKey := prefixKey(req.TenantID, op.Key)
		switch op.Kind {
		case TransactOpSet:
			dops = append(dops, &daprclient.StateOperation{
				Type: daprclient.StateOperationTypeUpsert,
				Item: &daprclient.SetStateItem{
					Key:   physKey,
					Value: op.Data,
					Etag:  &daprclient.ETag{Value: op.ExpectedEtag},
				},
			})
		case TransactOpDelete:
			dops = append(dops, &daprclient.StateOperation{
				Type: daprclient.StateOperationTypeDelete,
				Item: &daprclient.SetStateItem{
					Key:  physKey,
					Etag: &daprclient.ETag{Value: op.ExpectedEtag},
				},
			})
		default:
			return fmt.Errorf("tier1/state: unknown TransactOpKind %d", op.Kind)
		}
	}
	return a.client.stateClient().ExecuteStateTransaction(ctx, req.Store, buildMeta(req.TenantID, 0), dops)
}

// Delete は単一キーを Dapr State から削除する。
// ExpectedEtag が空なら DeleteState、非空なら DeleteStateWithETag を呼ぶ。
// 物理キーは prefixKey で `<tenant_id>/` を付与する（L2 テナント分離）。
func (a *daprStateAdapter) Delete(ctx context.Context, req StateSetRequest) error {
	// L2 テナント分離: 物理キーに `<tenant_id>/` を付与する。
	physKey := prefixKey(req.TenantID, req.Key)
	// metadata 構築（テナント識別子のみ。TTL は Delete に無関係）。
	meta := buildMeta(req.TenantID, 0)
	// 楽観的排他なし。
	if req.ExpectedEtag == "" {
		return a.client.stateClient().DeleteState(ctx, req.Store, physKey, meta)
	}
	// 楽観的排他あり: SDK の ETag struct に詰める。
	etag := &daprclient.ETag{Value: req.ExpectedEtag}
	// opts は nil（Concurrency / Consistency は Dapr SDK 既定に委ねる）。
	return a.client.stateClient().DeleteStateWithETag(ctx, req.Store, physKey, etag, meta, nil)
}
