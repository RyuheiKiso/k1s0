// 本ファイルは t1-state Pod の StateService 5 RPC ハンドラ実装。
//
// 設計正典:
//   docs/03_要件定義/20_機能要件/40_tier1_API契約IDL/02_State_API.md
//   docs/04_概要設計/20_ソフトウェア方式設計/01_コンポーネント方式設計/02_Daprファサード層コンポーネント.md
//
// scope（リリース時点 placeholder）:
//   adapter.StateAdapter に委譲するが、adapter は ErrNotWired を返すため
//   全 RPC で codes.Unimplemented を返却する。
//   実 Dapr State Management（Valkey）結線は plan 04-04。

package state

// 標準 / 内部パッケージ。
import (
	// 全 RPC で context を伝搬する。
	"context"
	// Dapr adapter（ErrNotWired 判定用）。
	"github.com/k1s0/k1s0/src/tier1/go/internal/adapter/dapr"
	// 共通 idempotency cache（共通規約 §「冪等性と再試行」）。
	"github.com/k1s0/k1s0/src/tier1/go/internal/common"
	// SDK 生成 stub の StateService 型。
	statev1 "github.com/k1s0/sdk-go/proto/v1/k1s0/tier1/state/v1"
	// gRPC エラーコード。
	"google.golang.org/grpc/codes"
	// gRPC ステータスエラー。
	"google.golang.org/grpc/status"
	// errors.Is で sentinel エラー判定。
	"errors"
)

// stateHandler は StateService の handler 実装。
// Unimplemented を埋め込むことで、proto 側に新 RPC が追加されてもコンパイル可能。
type stateHandler struct {
	// 将来追加 RPC のため埋め込み（本リリース時点は全 5 RPC を override 済）。
	statev1.UnimplementedStateServiceServer
	// adapter 集合への参照。
	deps Deps
}

// idempotency cache へのアクセス helper（stateHandler は deps.Idempotency を経由）。

// Get は単一キー取得。adapter 経由 Valkey 取得（リリース時点 placeholder）。
func (h *stateHandler) Get(ctx context.Context, req *statev1.GetRequest) (*statev1.GetResponse, error) {
	// 入力 nil 防御（gRPC では通常起きないが defensive）。
	if req == nil {
		// gRPC 慣用のエラー返却。
		return nil, status.Error(codes.InvalidArgument, "tier1/state: nil request")
	}
	// NFR-E-AC-003: tenant_id 越境防止のため必須検証。
	tid, err := requireTenantID(req.GetContext(), "State.Get")
	if err != nil {
		return nil, err
	}
	// adapter 入力に変換する。
	areq := dapr.StateGetRequest{
		// proto の store フィールドをそのまま渡す。
		Store: req.GetStore(),
		// proto の key フィールドをそのまま渡す。
		Key: req.GetKey(),
		// TenantContext.tenant_id を adapter に渡す。
		TenantID: tid,
	}
	// adapter 呼出。
	aresp, err := h.deps.StateAdapter.Get(ctx, areq)
	// ErrNotWired は Unimplemented に翻訳する。
	if err != nil {
		// 翻訳 helper に委譲する。
		return nil, translateErr(err, "Get", "plan 04-04")
	}
	// adapter 応答を proto 応答に変換する。
	return &statev1.GetResponse{
		// 値本文。
		Data: aresp.Data,
		// ETag。
		Etag: aresp.Etag,
		// 未存在フラグ。
		NotFound: aresp.NotFound,
	}, nil
}

// Set は単一キー保存。
// 共通規約 §「冪等性と再試行」: idempotency_key 指定時は同一キーの再試行で副作用を
// 重複させず、初回 SetResponse を返す（24h TTL の cache でレスポンスを保持）。
func (h *stateHandler) Set(ctx context.Context, req *statev1.SetRequest) (*statev1.SetResponse, error) {
	// 入力 nil 防御。
	if req == nil {
		// 不正引数として返却する。
		return nil, status.Error(codes.InvalidArgument, "tier1/state: nil request")
	}
	// NFR-E-AC-003: tenant_id 越境防止のため必須検証。
	tid, err := requireTenantID(req.GetContext(), "State.Set")
	if err != nil {
		return nil, err
	}
	// 実 Set 実行クロージャ。idempotency cache hit 時は呼ばれない。
	doSet := func() (interface{}, error) {
		areq := dapr.StateSetRequest{
			Store:        req.GetStore(),
			Key:          req.GetKey(),
			Data:         req.GetData(),
			ExpectedEtag: req.GetExpectedEtag(),
			TTLSeconds:   req.GetTtlSec(),
			TenantID:     tid,
		}
		aresp, err := h.deps.StateAdapter.Set(ctx, areq)
		if err != nil {
			return nil, translateErr(err, "Set", "plan 04-04")
		}
		return &statev1.SetResponse{NewEtag: aresp.NewEtag}, nil
	}
	// 冪等性キー + cache が両方ある場合のみ dedup を適用する。
	idempKey := common.IdempotencyKey(tid, "State.Set", req.GetIdempotencyKey())
	if idempKey == "" || h.deps.Idempotency == nil {
		resp, err := doSet()
		if err != nil {
			return nil, err
		}
		return resp.(*statev1.SetResponse), nil
	}
	resp, err := h.deps.Idempotency.GetOrCompute(ctx, idempKey, doSet)
	if err != nil {
		return nil, err
	}
	return resp.(*statev1.SetResponse), nil
}

// Delete は単一キー削除。
func (h *stateHandler) Delete(ctx context.Context, req *statev1.DeleteRequest) (*statev1.DeleteResponse, error) {
	// 入力 nil 防御。
	if req == nil {
		// 不正引数として返却する。
		return nil, status.Error(codes.InvalidArgument, "tier1/state: nil request")
	}
	// NFR-E-AC-003: tenant_id 越境防止のため必須検証。
	tid, err := requireTenantID(req.GetContext(), "State.Delete")
	if err != nil {
		return nil, err
	}
	// adapter 入力に変換する（StateSetRequest を流用）。
	areq := dapr.StateSetRequest{
		// store。
		Store: req.GetStore(),
		// key。
		Key: req.GetKey(),
		// 期待 ETag。
		ExpectedEtag: req.GetExpectedEtag(),
		// テナント。
		TenantID: tid,
	}
	// adapter 呼出。
	if err := h.deps.StateAdapter.Delete(ctx, areq); err != nil {
		// 翻訳して返却する。
		return nil, translateErr(err, "Delete", "plan 04-04")
	}
	// 成功応答。
	return &statev1.DeleteResponse{Deleted: true}, nil
}

// BulkGet は複数キーの一括取得（adapter.BulkGet 経由）。
func (h *stateHandler) BulkGet(ctx context.Context, req *statev1.BulkGetRequest) (*statev1.BulkGetResponse, error) {
	if req == nil {
		return nil, status.Error(codes.InvalidArgument, "tier1/state: nil request")
	}
	// NFR-E-AC-003: tenant_id 越境防止のため必須検証。
	tid, err := requireTenantID(req.GetContext(), "State.BulkGet")
	if err != nil {
		return nil, err
	}
	areq := dapr.StateBulkGetRequest{
		Store:    req.GetStore(),
		Keys:     req.GetKeys(),
		TenantID: tid,
	}
	items, err := h.deps.StateAdapter.BulkGet(ctx, areq)
	if err != nil {
		return nil, translateErr(err, "BulkGet", "plan 04-04")
	}
	results := make(map[string]*statev1.GetResponse, len(items))
	for _, item := range items {
		results[item.Key] = &statev1.GetResponse{
			Data:     item.Data,
			Etag:     item.Etag,
			NotFound: item.NotFound,
		}
	}
	return &statev1.BulkGetResponse{Results: results}, nil
}

// Transact はトランザクション境界付き複数操作（adapter.Transact 経由）。
func (h *stateHandler) Transact(ctx context.Context, req *statev1.TransactRequest) (*statev1.TransactResponse, error) {
	if req == nil {
		return nil, status.Error(codes.InvalidArgument, "tier1/state: nil request")
	}
	// NFR-E-AC-003: tenant_id 越境防止のため必須検証。
	tid, err := requireTenantID(req.GetContext(), "State.Transact")
	if err != nil {
		return nil, err
	}
	ops := make([]dapr.TransactOp, 0, len(req.GetOperations()))
	for _, op := range req.GetOperations() {
		switch x := op.GetOp().(type) {
		case *statev1.TransactOp_Set:
			s := x.Set
			ops = append(ops, dapr.TransactOp{
				Kind:         dapr.TransactOpSet,
				Key:          s.GetKey(),
				Data:         s.GetData(),
				ExpectedEtag: s.GetExpectedEtag(),
				TTLSeconds:   s.GetTtlSec(),
			})
		case *statev1.TransactOp_Delete:
			d := x.Delete
			ops = append(ops, dapr.TransactOp{
				Kind:         dapr.TransactOpDelete,
				Key:          d.GetKey(),
				ExpectedEtag: d.GetExpectedEtag(),
			})
		default:
			return nil, status.Error(codes.InvalidArgument, "tier1/state: unknown TransactOp variant")
		}
	}
	areq := dapr.StateTransactRequest{
		Store:    req.GetStore(),
		Ops:      ops,
		TenantID: tid,
	}
	if err := h.deps.StateAdapter.Transact(ctx, areq); err != nil {
		return &statev1.TransactResponse{Committed: false}, translateErr(err, "Transact", "plan 04-04")
	}
	return &statev1.TransactResponse{Committed: true}, nil
}

// translateErr は adapter エラーを gRPC ステータスエラーに翻訳する。
// ErrNotWired は Unimplemented に、ETag mismatch / 既存キー衝突は AlreadyExists（Conflict）に、
// それ以外は Internal に翻訳する。
// docs §「エラー型 K1s0Error」: AlreadyExists / Conflict は ETag 不一致・冪等性キー衝突を表す。
func translateErr(err error, rpc string, plan string) error {
	// ErrNotWired は計画に従い Unimplemented とする。
	if errors.Is(err, dapr.ErrNotWired) {
		// 呼出 RPC 名と計画 ID を含めたメッセージを返却する。
		return status.Errorf(codes.Unimplemented, "tier1/state: %s not yet wired to Dapr backend (%s)", rpc, plan)
	}
	// ETag 不一致 / First-Write 違反（既存キーへの無 ETag 書込）は AlreadyExists（Conflict）。
	// docs §「エラー型 K1s0Error」: AlreadyExists / Conflict — ETag 不一致、冪等性キー衝突。
	if errors.Is(err, dapr.ErrEtagMismatch) {
		return status.Errorf(codes.AlreadyExists,
			"tier1/state: %s: etag mismatch or first-write conflict", rpc)
	}
	// production Dapr SDK は conflict を gRPC Aborted で返すため、status code 経由でも検知する。
	if st, ok := status.FromError(err); ok {
		switch st.Code() {
		case codes.Aborted, codes.AlreadyExists, codes.FailedPrecondition:
			return status.Errorf(codes.AlreadyExists,
				"tier1/state: %s: etag mismatch or first-write conflict: %s", rpc, st.Message())
		}
	}
	// 想定外エラーは Internal にマップする。
	return status.Errorf(codes.Internal, "tier1/state: %s adapter error: %v", rpc, err)
}
