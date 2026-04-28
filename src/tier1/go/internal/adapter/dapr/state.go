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
// テナント prefix:
//   ABAC（NFR-E-AC-003）に従い、Dapr metadata `tenantId` でテナント識別子を
//   sidecar 側に伝搬する。component 側で metadata.partitionKey として使うか、
//   key 自体に prefix を付与するかは Dapr Component 設定に委ねる（k1s0 既定は
//   metadata 伝搬のみ、prefix 付与は handler 上位の TenantContext で実施済前提）。
//
// 楽観的排他（ETag）:
//   ExpectedEtag が空のリクエスト → SaveState / DeleteState（無条件）
//   ExpectedEtag が非空のリクエスト → SaveStateWithETag / DeleteStateWithETag
//   conflict 時、Dapr SDK は status code を含む error を返す。本層では透過的に上位へ。

package dapr

import (
	// 全 RPC で context を伝搬する。
	"context"
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
func (a *daprStateAdapter) Get(ctx context.Context, req StateGetRequest) (StateGetResponse, error) {
	// Dapr SDK 呼び出し。Component 側 store 名が空の場合は SDK が即時 InvalidArgument を返す。
	item, err := a.client.stateClient().GetState(ctx, req.Store, req.Key, buildMeta(req.TenantID, 0))
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
func (a *daprStateAdapter) Set(ctx context.Context, req StateSetRequest) (StateSetResponse, error) {
	// metadata 構築（テナント + TTL）。
	meta := buildMeta(req.TenantID, req.TTLSeconds)
	// 楽観的排他の有無で SDK メソッドを切り替える。
	if req.ExpectedEtag == "" {
		if err := a.client.stateClient().SaveState(ctx, req.Store, req.Key, req.Data, meta); err != nil {
			return StateSetResponse{}, err
		}
	} else {
		if err := a.client.stateClient().SaveStateWithETag(ctx, req.Store, req.Key, req.Data, req.ExpectedEtag, meta); err != nil {
			return StateSetResponse{}, err
		}
	}
	// SDK の SaveState は ETag を返さないため空文字を返す（必要時は handler で Get を再実行）。
	return StateSetResponse{NewEtag: ""}, nil
}

// Delete は単一キーを Dapr State から削除する。
// ExpectedEtag が空なら DeleteState、非空なら DeleteStateWithETag を呼ぶ。
func (a *daprStateAdapter) Delete(ctx context.Context, req StateSetRequest) error {
	// metadata 構築（テナント識別子のみ。TTL は Delete に無関係）。
	meta := buildMeta(req.TenantID, 0)
	// 楽観的排他なし。
	if req.ExpectedEtag == "" {
		return a.client.stateClient().DeleteState(ctx, req.Store, req.Key, meta)
	}
	// 楽観的排他あり: SDK の ETag struct に詰める。
	etag := &daprclient.ETag{Value: req.ExpectedEtag}
	// opts は nil（Concurrency / Consistency は Dapr SDK 既定に委ねる）。
	return a.client.stateClient().DeleteStateWithETag(ctx, req.Store, req.Key, etag, meta, nil)
}
