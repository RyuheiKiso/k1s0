// 本ファイルは t1-state Pod の FeatureAdminService 実装。
//
// 設計正典:
//   docs/03_要件定義/20_機能要件/40_tier1_API契約IDL/11_Feature_API.md
//   docs/02_構想設計/adr/ADR-FM-001-flagd-openfeature.md
//
// 役割:
//   FeatureAdminService の 3 RPC（RegisterFlag / GetFlag / ListFlags）を
//   tenant 単位で in-memory に永続化する registry-backed 実装。
//   release-initial では in-memory backend を提供し、後段で flagd / GitOps
//   sync に切替える際も同じ gRPC handler が前段で動作する設計。
//
// 検証:
//   - flag_key は <tenant>.<component>.<feature> 形式必須
//   - default_variant は variants map 内に存在必須
//   - PERMISSION 種別は approval_id 必須（Product Council 承認）
//   - state UNSPECIFIED / value_type UNSPECIFIED は登録時 reject

package state

import (
	// context 伝搬。
	"context"
	// バージョン番号 atomic 採番に使う。
	"strings"
	// goroutine-safe な registry に sync.RWMutex を使う。
	"sync"

	// SDK 生成 stub の FeatureAdminService 型。
	featurev1 "github.com/k1s0/sdk-go/proto/v1/k1s0/tier1/feature/v1"
	// gRPC エラーコード。
	"google.golang.org/grpc/codes"
	// gRPC ステータスエラー。
	"google.golang.org/grpc/status"
	// proto.Clone で読出時に外部書換えを防ぐ。
	"google.golang.org/protobuf/proto"
)

// FlagRegistry は FeatureAdminService の永続化層。tenant_id × flag_key で
// 多バージョン保持する。各エントリは登録順に version=1, 2, ... を採番する。
//
// 並行制御: sync.RWMutex で全状態を保護する。Get / List は RLock、Register は Lock。
type FlagRegistry struct {
	// 全状態を保護する RWMutex。
	mu sync.RWMutex
	// tenantId → flag_key → []*FlagDefinition（インデックスは version-1 に対応）。
	// proto.Clone で値を coppy して保存し、読出時にも clone して返すことで registry 外の
	// 書換えが内部状態に波及しないようにする。
	flags map[string]map[string][]*featurev1.FlagDefinition
}

// NewFlagRegistry は空の registry を生成する。
func NewFlagRegistry() *FlagRegistry {
	// 内部 map を 0 で初期化して返す。
	return &FlagRegistry{
		// tenant 階層で空に初期化する。
		flags: map[string]map[string][]*featurev1.FlagDefinition{},
	}
}

// register は (tenant, flag_key, def) を追加し、新 version を返す。
// 既存 flag_key があれば末尾に append する（version は前回 +1）。
// 並行制御は呼出側責務。
func (r *FlagRegistry) register(tenant string, def *featurev1.FlagDefinition) int64 {
	// tenant 階層を取り出す（不在時は遅延生成）。
	byKey, ok := r.flags[tenant]
	// tenant 階層が不在の分岐。
	if !ok {
		// 新 map を割当てて保存する。
		byKey = map[string][]*featurev1.FlagDefinition{}
		// tenant 階層に登録する。
		r.flags[tenant] = byKey
	}
	// flag_key 配下の version 列に append する。
	versions := byKey[def.GetFlagKey()]
	// proto.Clone で外部書換えから守る。
	cloned := proto.Clone(def).(*featurev1.FlagDefinition)
	// 新 version を末尾に追加する。
	versions = append(versions, cloned)
	// flag_key 配下を更新する。
	byKey[def.GetFlagKey()] = versions
	// 新 version 番号は len（1 始まり）。
	return int64(len(versions))
}

// get は flag_key の指定 version を返す。version=0 / negative は最新。未存在は (nil, 0, false)。
func (r *FlagRegistry) get(tenant, flagKey string, version int64) (*featurev1.FlagDefinition, int64, bool) {
	// tenant 階層を取り出す。
	byKey, ok := r.flags[tenant]
	// tenant 階層が不在の分岐。
	if !ok {
		// 未存在として返す。
		return nil, 0, false
	}
	// flag_key 配下を取り出す。
	versions, ok := byKey[flagKey]
	// flag_key 配下が不在 / 空の分岐。
	if !ok || len(versions) == 0 {
		// 未存在として返す。
		return nil, 0, false
	}
	// version が指定されない場合は最新（末尾）を返す。
	if version <= 0 {
		// 最新エントリを clone して返す。
		latest := proto.Clone(versions[len(versions)-1]).(*featurev1.FlagDefinition)
		// 返却する（version 番号は配列長と一致）。
		return latest, int64(len(versions)), true
	}
	// 1-indexed の version を 0-indexed に変換。
	idx := int(version - 1)
	// 範囲外（過去存在しない version）。
	if idx < 0 || idx >= len(versions) {
		// 未存在として返す。
		return nil, 0, false
	}
	// 指定 version の clone を返す。
	out := proto.Clone(versions[idx]).(*featurev1.FlagDefinition)
	// 返却する。
	return out, version, true
}

// listLatest は tenant の全 flag_key について最新 version を返す（kind / state でフィルタ可）。
func (r *FlagRegistry) listLatest(tenant string, kindFilter *featurev1.FlagKind, stateFilter *featurev1.FlagState) []*featurev1.FlagDefinition {
	// 結果 slice を準備する。
	out := make([]*featurev1.FlagDefinition, 0)
	// tenant 階層を取り出す。
	byKey, ok := r.flags[tenant]
	// tenant 階層が不在 → 空 slice を返す。
	if !ok {
		// 空応答を返す。
		return out
	}
	// 全 flag_key を走査する。
	for _, versions := range byKey {
		// 空配列は skip する（理論上発生しないが防御）。
		if len(versions) == 0 {
			// 次のエントリへ。
			continue
		}
		// 最新エントリを参照する。
		latest := versions[len(versions)-1]
		// kind フィルタ判定。指定されている時のみ比較する。
		if kindFilter != nil && latest.GetKind() != *kindFilter {
			// 一致しない → skip する。
			continue
		}
		// state フィルタの既定挙動: 未指定なら ENABLED のみ返す（proto コメント準拠）。
		// 明示指定があれば指定値と一致するもののみ返す。
		effState := featurev1.FlagState_FLAG_STATE_ENABLED
		// stateFilter 指定の分岐。
		if stateFilter != nil {
			// 指定値で上書きする。
			effState = *stateFilter
		}
		// effState と一致しないものは skip する。
		if latest.GetState() != effState {
			// 次のエントリへ。
			continue
		}
		// 一致したものを clone して結果に詰める。
		out = append(out, proto.Clone(latest).(*featurev1.FlagDefinition))
	}
	// 結果を返す。
	return out
}

// featureAdminHandler は FeatureAdminService の handler 実装。
type featureAdminHandler struct {
	// 将来 RPC 追加に備えた forward-compat 用埋め込み。
	featurev1.UnimplementedFeatureAdminServiceServer
	// in-memory registry。NewFlagRegistry() で生成。
	registry *FlagRegistry
}

// NewFeatureAdminServiceServer は HTTP gateway / 統合テスト用に handler を直接生成する exported helper。
func NewFeatureAdminServiceServer(reg *FlagRegistry) featurev1.FeatureAdminServiceServer {
	// 単純な struct を返却する。
	return &featureAdminHandler{registry: reg}
}

// validateFlagDefinition は登録時の最小検証を行う。
func validateFlagDefinition(def *featurev1.FlagDefinition, approvalID string) error {
	// nil 防御。
	if def == nil {
		// InvalidArgument を返す。
		return status.Error(codes.InvalidArgument, "tier1/feature: flag definition is required")
	}
	// flag_key 必須 + 命名規則チェック。
	if !isValidFlagKey(def.GetFlagKey()) {
		// 命名規則に合致しないものは InvalidArgument。
		return status.Error(codes.InvalidArgument,
			"tier1/feature: flag_key must match <tenant>.<component>.<feature>")
	}
	// value_type UNSPECIFIED は reject。
	if def.GetValueType() == featurev1.FlagValueType_FLAG_VALUE_UNSPECIFIED {
		// InvalidArgument を返す。
		return status.Error(codes.InvalidArgument, "tier1/feature: value_type must be specified")
	}
	// state UNSPECIFIED は reject。
	if def.GetState() == featurev1.FlagState_FLAG_STATE_UNSPECIFIED {
		// InvalidArgument を返す。
		return status.Error(codes.InvalidArgument, "tier1/feature: state must be specified")
	}
	// default_variant が variants map に存在することを必須にする。
	if _, ok := def.GetVariants()[def.GetDefaultVariant()]; !ok {
		// InvalidArgument を返す。
		return status.Errorf(codes.InvalidArgument,
			"tier1/feature: default_variant %q not present in variants", def.GetDefaultVariant())
	}
	// PERMISSION 種別は Product Council 承認番号 (approval_id) 必須。
	if def.GetKind() == featurev1.FlagKind_PERMISSION && approvalID == "" {
		// FailedPrecondition を返す（承認が必要）。
		return status.Error(codes.FailedPrecondition,
			"tier1/feature: PERMISSION flag requires approval_id from Product Council")
	}
	// 全検証 pass。
	return nil
}

// isValidFlagKey は flag_key 命名規則 <tenant>.<component>.<feature> を簡易検証する。
// 厳密な regex 検証は flagd 仕様に委ね、本実装は「3 つ以上の dot 区切りセクション」を必須とする。
func isValidFlagKey(key string) bool {
	// 空文字は不合格。
	if key == "" {
		// false を返す。
		return false
	}
	// dot 区切り 3 セクション以上か検査する（tenant.component.feature の最小形）。
	parts := strings.Split(key, ".")
	// 3 セクション未満は不合格。
	if len(parts) < 3 {
		// false を返す。
		return false
	}
	// 各セクションが空でないことを確認する。
	for _, p := range parts {
		// 空セクションは不合格。
		if p == "" {
			// false を返す。
			return false
		}
	}
	// 全 pass。
	return true
}

// RegisterFlag は flag 定義の登録（version 採番付き）。
func (h *featureAdminHandler) RegisterFlag(ctx context.Context, req *featurev1.RegisterFlagRequest) (*featurev1.RegisterFlagResponse, error) {
	// 入力 nil 防御。
	if req == nil {
		// InvalidArgument を返す。
		return nil, status.Error(codes.InvalidArgument, "tier1/feature: nil request")
	}
	// NFR-E-AC-003: tenant_id 越境防止のため必須検証。
	tid, err := requireTenantID(req.GetContext(), "FeatureAdmin.RegisterFlag")
	// 検証失敗は即返却。
	if err != nil {
		// 翻訳済 error を返す。
		return nil, err
	}
	// flag definition の最小検証。
	if err := validateFlagDefinition(req.GetFlag(), req.GetApprovalId()); err != nil {
		// InvalidArgument / FailedPrecondition を返す。
		return nil, err
	}
	// registry に登録し、新 version を取得する。
	h.registry.mu.Lock()
	// 並行制御の defer Unlock。
	defer h.registry.mu.Unlock()
	// register helper を呼ぶ（書込）。
	version := h.registry.register(tid, req.GetFlag())
	// 応答を返す。
	return &featurev1.RegisterFlagResponse{Version: version}, nil
}

// GetFlag は指定 flag_key の指定 version（または最新）を取得する。
func (h *featureAdminHandler) GetFlag(ctx context.Context, req *featurev1.GetFlagRequest) (*featurev1.GetFlagResponse, error) {
	// 入力 nil 防御。
	if req == nil {
		// InvalidArgument を返す。
		return nil, status.Error(codes.InvalidArgument, "tier1/feature: nil request")
	}
	// tenant_id 必須検証。
	tid, err := requireTenantID(req.GetContext(), "FeatureAdmin.GetFlag")
	// 検証失敗は即返却。
	if err != nil {
		// 翻訳済 error を返す。
		return nil, err
	}
	// flag_key 必須。
	if req.GetFlagKey() == "" {
		// InvalidArgument を返す。
		return nil, status.Error(codes.InvalidArgument, "tier1/feature: flag_key is required")
	}
	// 並行制御は RLock で十分（読出のみ）。
	h.registry.mu.RLock()
	// defer RUnlock。
	defer h.registry.mu.RUnlock()
	// version 抽出（optional 0 は最新）。
	var v int64
	// optional がある時のみ抽出する。
	if req.Version != nil {
		// 値を取得する。
		v = req.GetVersion()
	}
	// registry から get する。
	def, ver, found := h.registry.get(tid, req.GetFlagKey(), v)
	// 未存在は NotFound。
	if !found {
		// NotFound を返す。
		return nil, status.Errorf(codes.NotFound, "tier1/feature: flag %q (version=%d) not found", req.GetFlagKey(), v)
	}
	// 応答を返す。
	return &featurev1.GetFlagResponse{Flag: def, Version: ver}, nil
}

// ListFlags は tenant の全 flag_key について最新 version を返す。
func (h *featureAdminHandler) ListFlags(ctx context.Context, req *featurev1.ListFlagsRequest) (*featurev1.ListFlagsResponse, error) {
	// 入力 nil 防御。
	if req == nil {
		// InvalidArgument を返す。
		return nil, status.Error(codes.InvalidArgument, "tier1/feature: nil request")
	}
	// tenant_id 必須検証。
	tid, err := requireTenantID(req.GetContext(), "FeatureAdmin.ListFlags")
	// 検証失敗は即返却。
	if err != nil {
		// 翻訳済 error を返す。
		return nil, err
	}
	// 並行制御は RLock で十分（読出のみ）。
	h.registry.mu.RLock()
	// defer RUnlock。
	defer h.registry.mu.RUnlock()
	// kind フィルタを optional 経由で抽出する。
	var kindPtr *featurev1.FlagKind
	// 指定があれば pointer を作る。
	if req.Kind != nil {
		// 値を変数に取って pointer を返す。
		k := req.GetKind()
		// pointer を渡す。
		kindPtr = &k
	}
	// state フィルタを optional 経由で抽出する。
	var statePtr *featurev1.FlagState
	// 指定があれば pointer を作る。
	if req.State != nil {
		// 値を変数に取って pointer を返す。
		s := req.GetState()
		// pointer を渡す。
		statePtr = &s
	}
	// 一覧を取得する。
	flags := h.registry.listLatest(tid, kindPtr, statePtr)
	// 応答を返す。
	return &featurev1.ListFlagsResponse{Flags: flags}, nil
}
