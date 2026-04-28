// 本ファイルは OpenBao KVv2 の SecretsAdapter 実装。
//
// 設計正典:
//   docs/03_要件定義/20_機能要件/40_tier1_API契約IDL/04_Secrets_API.md
//
// 役割（plan 04-06 結線済）:
//   handler.go が呼び出す Get / BulkGet / Rotate を OpenBao Go SDK の
//   KVv2.Get / GetVersion / Put で実装する。
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

// Get は単一 secret を OpenBao KVv2 から取得する。
// Version > 0 の場合は GetVersion、それ以外は Get（最新）を呼ぶ。
func (a *openbaoSecretsAdapter) Get(ctx context.Context, req SecretGetRequest) (SecretGetResponse, error) {
	var (
		secret *bao.KVSecret
		err    error
	)
	if req.Version > 0 {
		secret, err = a.client.kvClientFor().GetVersion(ctx, req.Name, req.Version)
	} else {
		secret, err = a.client.kvClientFor().Get(ctx, req.Name)
	}
	if err != nil {
		// OpenBao SDK は 404 系を含めて error 返却するため、上位で NotFound に翻訳する。
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
func (a *openbaoSecretsAdapter) BulkGet(ctx context.Context, names []string, _ string) (map[string]SecretGetResponse, error) {
	out := make(map[string]SecretGetResponse, len(names))
	for _, name := range names {
		resp, err := a.Get(ctx, SecretGetRequest{Name: name})
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

// Rotate は KVv2 の現在値を読み、同じ値で新規バージョンを書き込む。
// これでバージョン番号が bump され、監査ログ・version 履歴に「rotate イベント」として
// 残る。実際の値生成（DB password 再発行など）は呼出側責務（本 adapter は bump のみ）。
func (a *openbaoSecretsAdapter) Rotate(ctx context.Context, req SecretRotateRequest) (SecretGetResponse, error) {
	cur, err := a.client.kvClientFor().Get(ctx, req.Name)
	if err != nil {
		return SecretGetResponse{}, err
	}
	if cur == nil {
		return SecretGetResponse{}, ErrSecretNotFound
	}
	// 同じ data を Put することで version が +1 される。
	put, err := a.client.kvClientFor().Put(ctx, req.Name, kvSecretToData(cur))
	if err != nil {
		return SecretGetResponse{}, err
	}
	return kvSecretToResponse(put), nil
}
