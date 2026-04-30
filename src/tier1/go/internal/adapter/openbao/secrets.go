// 本ファイルは OpenBao KVv2 の SecretsAdapter 実装。
//
// 設計正典:
//   docs/03_要件定義/20_機能要件/40_tier1_API契約IDL/04_Secrets_API.md
//
// 役割（plan 04-06 結線済）:
//   handler.go が呼び出す Get / BulkGet / Rotate を OpenBao Go SDK の
//   KVv2.Get / GetVersion / Put で実装する。
//
// L2 テナント分離（NFR-E-AC-003、共通規約 §「マルチテナント分離」）:
//   docs `<tenant_id>/<resource>` 規定に従い、OpenBao 側の物理 secretPath を
//   `<tenantID>/<name>` で wrap する（secretPath ヘルパ）。tenantID が空の場合は
//   prefix を付けず raw 名を使う（test fake 経路、handler 上位 requireTenantID で空を弾く）。
//   ListAndGet も同形式（`<tenantID>/`）を使用し、Get と List の path 規約を一致させる。
//
// 値型の扱い:
//   OpenBao KVSecret.Data は `map[string]interface{}` 型。proto は `map[string]string`
//   型なので、文字列でないフィールドは fmt.Sprintf("%v", v) で文字列化する。
//   バイナリ secret や JSON object を扱う場合は呼出側で base64 / JSON エンコードする運用。
//
// Rotate のセマンティクス:
//   OpenBao は KV 自体に "rotate" コマンドを持たない（database engine など特定 engine のみ）。
//   汎用の Rotate API としては「現在値を読み、同じ値で新規バージョンを書き込む」ことで
//   バージョン番号を bump する操作にとどめる。実際の値生成（DB password 再発行など）は
//   呼出側ロジックの責務とし、本 adapter はバージョン管理層を提供する。

package openbao

import (
	"context"
	"errors"
	"fmt"
	"strings"

	bao "github.com/openbao/openbao/api/v2"
)

// SecretGetRequest は単一 secret 取得の入力。
type SecretGetRequest struct {
	// シークレット名（KVv2 の secretPath、テナント prefix 付き）。
	Name string
	// 指定バージョン（0 / 負の値で最新）。
	Version int
	// テナント識別子（境界検証用、本 adapter では path に含める運用）。
	TenantID string
}

// SecretGetResponse は単一 secret 取得の応答。
type SecretGetResponse struct {
	// key=value マップ（OpenBao の Data を string 化したもの）。
	Values map[string]string
	// バージョン。
	Version int32
}

// SecretRotateRequest は Rotate の入力。
type SecretRotateRequest struct {
	// シークレット名。
	Name string
	// テナント識別子。
	TenantID string
}

// SecretsAdapter は SecretsService の操作集合。
type SecretsAdapter interface {
	// 単一 secret 取得。
	Get(ctx context.Context, req SecretGetRequest) (SecretGetResponse, error)
	// 複数 secret 取得（呼出側が name リストを渡す）。
	BulkGet(ctx context.Context, names []string, tenantID string) (map[string]SecretGetResponse, error)
	// テナント配下の全 secret を列挙して取得する（FR-T1-SECRETS-002 の BulkGet 経路）。
	// 内部で Lister.List + Get の N 回呼出を行う。
	ListAndGet(ctx context.Context, tenantID string) (map[string]SecretGetResponse, error)
	// Rotate（KVv2 ではバージョン bump のみを担当、値生成は呼出側責務）。
	Rotate(ctx context.Context, req SecretRotateRequest) (SecretGetResponse, error)
}

// openbaoSecretsAdapter は Client（narrow interface）越しに SDK を呼ぶ実装。
type openbaoSecretsAdapter struct {
	client *Client
}

// NewSecretsAdapter は Client から SecretsAdapter を生成する。
func NewSecretsAdapter(client *Client) SecretsAdapter {
	return &openbaoSecretsAdapter{client: client}
}

// kvSecretToResponse は OpenBao KVSecret を SecretGetResponse に詰め替える。
// Data の値は interface{} なので fmt.Sprintf("%v") で string 化する。
func kvSecretToResponse(s *bao.KVSecret) SecretGetResponse {
	if s == nil {
		return SecretGetResponse{}
	}
	values := make(map[string]string, len(s.Data))
	for k, v := range s.Data {
		// nil は空文字、その他は %v で文字列化（数値や bool は string 化される）。
		if v == nil {
			values[k] = ""
			continue
		}
		// すでに string 型なら直接使う（%v だと余計なフォーマットが入らないが念のため）。
		if str, ok := v.(string); ok {
			values[k] = str
			continue
		}
		values[k] = fmt.Sprintf("%v", v)
	}
	resp := SecretGetResponse{Values: values}
	if s.VersionMetadata != nil {
		resp.Version = int32(s.VersionMetadata.Version)
	}
	return resp
}

// kvSecretToData は KVSecret.Data から Put 用の data map を作る。
// values フィールドを反転して再度 Put する用途（Rotate のバージョン bump）。
func kvSecretToData(s *bao.KVSecret) map[string]interface{} {
	if s == nil {
		return nil
	}
	out := make(map[string]interface{}, len(s.Data))
	for k, v := range s.Data {
		out[k] = v
	}
	return out
}

// secretPath は L2 テナント分離のため物理 secret path を `<tenantID>/<name>` に wrap する。
// tenantID が空 / name が既に prefix 済 の場合は元値をそのまま返す（二重 prefix 抑止）。
func secretPath(tenantID, name string) string {
	if tenantID == "" {
		return name
	}
	prefix := tenantID + "/"
	if len(name) >= len(prefix) && name[:len(prefix)] == prefix {
		return name
	}
	return prefix + name
}

// stripSecretPath は応答 key から `<tenantID>/` を取り除いて tier2/tier3 視点の name に戻す。
func stripSecretPath(tenantID, name string) string {
	if tenantID == "" {
		return name
	}
	prefix := tenantID + "/"
	if len(name) >= len(prefix) && name[:len(prefix)] == prefix {
		return name[len(prefix):]
	}
	return name
}

// Get は単一 secret を OpenBao KVv2 から取得する。
// Version > 0 の場合は GetVersion、それ以外は Get（最新）を呼ぶ。
// L2 分離: 物理 secretPath は `<tenantID>/<name>` で wrap する。
func (a *openbaoSecretsAdapter) Get(ctx context.Context, req SecretGetRequest) (SecretGetResponse, error) {
	var (
		secret *bao.KVSecret
		err    error
	)
	// L2 テナント分離: 物理 path を `<tenantID>/<name>` で wrap する。
	physPath := secretPath(req.TenantID, req.Name)
	if req.Version > 0 {
		secret, err = a.client.kvClientFor().GetVersion(ctx, physPath, req.Version)
	} else {
		secret, err = a.client.kvClientFor().Get(ctx, physPath)
	}
	if err != nil {
		// OpenBao SDK は 404 / 未存在 を「string メッセージで包んだ通常 error」で返す
		// （sentinel ErrSecretNotFound ではない）。本 adapter で sentinel に正規化し、
		// 上位 translateErr が codes.NotFound (= HTTP 404) に翻訳できるようにする。
		// ErrSecretNotFound でない Internal 翻訳は kv2 SDK の永続的 not-found を
		// codes.Internal 化してしまい、schemathesis が contract drift を検出する。
		if msg := err.Error(); strings.Contains(msg, "secret not found") ||
			strings.Contains(msg, "404") {
			return SecretGetResponse{}, ErrSecretNotFound
		}
		return SecretGetResponse{}, err
	}
	if secret == nil {
		return SecretGetResponse{}, ErrSecretNotFound
	}
	return kvSecretToResponse(secret), nil
}

// BulkGet は複数 secret を順次取得する。
// OpenBao は bulk-get の専用 API を持たないため、name 毎に Get を呼ぶ単純実装。
// 1 件失敗しても他の secret は返したいため、エラーは map に統合せず最初のエラーで全体失敗とする。
// （部分成功運用が必要な場合は後続 PR で per-key エラーマップを返す形に拡張）。
// L2 分離: 各 name は Get 経由で物理 path に wrap される（adapter の secretPath が適用）。
func (a *openbaoSecretsAdapter) BulkGet(ctx context.Context, names []string, tenantID string) (map[string]SecretGetResponse, error) {
	out := make(map[string]SecretGetResponse, len(names))
	for _, name := range names {
		resp, err := a.Get(ctx, SecretGetRequest{Name: name, TenantID: tenantID})
		if err != nil {
			// NotFound は skip（部分結果として扱う）、他のエラーは即時返却。
			if errors.Is(err, ErrSecretNotFound) {
				continue
			}
			return nil, err
		}
		out[name] = resp
	}
	return out, nil
}

// ListAndGet はテナント prefix `<tenantID>/` 配下の secret を列挙し、
// 各 secret の最新値を Get で取得して map にまとめる。Lister 未注入時は ErrNotWired。
// proto BulkGet（FR-T1-SECRETS-002 の「テナントに割当された全シークレット」）の実装経路。
//
// L2 分離: List path と Get path 規約を統一（`<tenantID>/`）。Lister.List は物理 path を返し、
// 個別 Get に渡すときは secretPath 二重 prefix 抑止が効く（path に既に prefix が含まれるため）。
// 応答 map のキーは stripSecretPath で raw 名に戻して tier2/tier3 に透過させる。
func (a *openbaoSecretsAdapter) ListAndGet(ctx context.Context, tenantID string) (map[string]SecretGetResponse, error) {
	// tenantID 必須。
	if tenantID == "" {
		// 不正引数として透過（handler 側で InvalidArgument に翻訳）。
		return nil, errEmptyTenant
	}
	// Lister 未注入時は未結線扱い。
	lister := a.client.listerFor()
	if lister == nil {
		// ErrNotWired を返却（handler 側で Unimplemented に翻訳）。
		return nil, ErrNotWired
	}
	// テナント prefix（`<tenantID>/`）を構築する。docs `<tenant_id>/<resource>` 規約と一致。
	prefix := tenantID + "/"
	// Lister で path 配下の name 一覧を取得する（path は通常 prefix 込で返る）。
	names, err := lister.List(ctx, prefix)
	// SDK エラーは透過する。
	if err != nil {
		// error をそのまま返却する。
		return nil, err
	}
	// 各 name の Get を並べる map を準備する。
	out := make(map[string]SecretGetResponse, len(names))
	// 1 件ずつ取得する。
	for _, name := range names {
		// Get で最新値を取得する（secretPath が二重 prefix を抑止する）。
		resp, gerr := a.Get(ctx, SecretGetRequest{Name: name, TenantID: tenantID})
		// NotFound（直前 List 後に削除など）は skip する。
		if gerr != nil {
			// 部分結果性を担保するため continue する。
			continue
		}
		// prefix を取り除いた相対 name を key にする（呼出側 UX 配慮）。
		short := stripSecretPath(tenantID, name)
		// 結果 map に詰める。
		out[short] = resp
	}
	// 結果を返す。
	return out, nil
}

// errEmptyTenant は ListAndGet 呼出時に tenantID が空である旨を表す内部 sentinel。
var errEmptyTenant = errors.New("tier1/secrets: tenant_id required for BulkGet")

// Rotate は KVv2 の現在値を読み、同じ値で新規バージョンを書き込む。
// これでバージョン番号が bump され、監査ログ・version 履歴に「rotate イベント」として
// 残る。実際の値生成（DB password 再発行など）は呼出側責務（本 adapter は bump のみ）。
// L2 分離: 物理 secretPath は `<tenantID>/<name>` で wrap する。
func (a *openbaoSecretsAdapter) Rotate(ctx context.Context, req SecretRotateRequest) (SecretGetResponse, error) {
	physPath := secretPath(req.TenantID, req.Name)
	cur, err := a.client.kvClientFor().Get(ctx, physPath)
	if err != nil {
		return SecretGetResponse{}, err
	}
	if cur == nil {
		return SecretGetResponse{}, ErrSecretNotFound
	}
	// 同じ data を Put することで version が +1 される。
	put, err := a.client.kvClientFor().Put(ctx, physPath, kvSecretToData(cur))
	if err != nil {
		return SecretGetResponse{}, err
	}
	return kvSecretToResponse(put), nil
}
