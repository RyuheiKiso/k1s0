// 本ファイルは OpenBao Logical().List() を Lister interface 越しに呼ぶ production 実装。
//
// 設計正典:
//   docs/03_要件定義/20_機能要件/40_tier1_API契約IDL/04_Secrets_API.md
//     - FR-T1-SECRETS-002（テナント配下の全シークレットを BulkGet で取得）
//
// 役割:
//   OpenBao KVv2 には List API が直接存在しないため、underlying `*bao.Client` の
//   `Logical().List(metadataPath)` を使って secret 名一覧を取得する。
//   `metadataPath` は `<mount>/metadata/<prefix>` 形式（KVv2 のメタデータ path 規約）。
//
// 戻り値の型:
//   `*bao.Secret.Data["keys"]` は `[]interface{}` 型で、各要素は string。
//   フォルダ末尾は "/" を含むためそのまま返却する（呼出側で trim する運用）。

package openbao

import (
	// 全 RPC で context を伝搬する。
	"context"
	// path 結合に使う。
	"path"

	// OpenBao SDK の Logical / Secret 型を参照する。
	bao "github.com/openbao/openbao/api/v2"
)

// productionLister は OpenBao Logical().List() を Lister interface 越しに公開する shim。
// `*bao.Client` を保持し、KVv2 mount 名を組み合わせて metadata path を構築する。
type productionLister struct {
	// SDK の logical client（List() を呼ぶ）。
	logical *bao.Logical
	// KVv2 mount 名（例: "secret"）。metadata path 構築に使う。
	mount string
}

// newProductionLister は SDK Client と mount 名から Lister を生成する。
func newProductionLister(client *bao.Client, mount string) Lister {
	// productionLister を初期化する。
	return &productionLister{
		// Logical handle を保持する。
		logical: client.Logical(),
		// mount 名を保持する。
		mount: mount,
	}
}

// List は prefix 配下の secret 名（メタデータ path）を OpenBao から取得する。
// ListWithContext を使って context タイムアウト / cancel を SDK に伝搬する。
func (l *productionLister) List(ctx context.Context, prefix string) ([]string, error) {
	// metadata path を組み立てる（KVv2 規約: "<mount>/metadata/<prefix>"）。
	metaPath := path.Join(l.mount, "metadata", prefix)
	// SDK の ListWithContext を呼ぶ（cancel 伝搬に必要）。
	secret, err := l.logical.ListWithContext(ctx, metaPath)
	// SDK エラーは透過する。
	if err != nil {
		// error をそのまま返却する。
		return nil, err
	}
	// 未存在 / 空 path は空 slice を返す。
	if secret == nil || secret.Data == nil {
		// 空 slice を返却する。
		return nil, nil
	}
	// keys field（[]interface{}）を取り出す。
	rawKeys, ok := secret.Data["keys"].([]interface{})
	// keys 不在は空 slice。
	if !ok {
		// 空 slice を返却する。
		return nil, nil
	}
	// string 型に詰め替える slice を準備する。
	out := make([]string, 0, len(rawKeys))
	// for-range で 1 件ずつ string 化する。
	for _, k := range rawKeys {
		// type assertion で string を取り出す。
		if s, ok := k.(string); ok {
			// prefix を補って完全 path に整形する。
			out = append(out, path.Join(prefix, s))
		}
	}
	// 結果を返す。
	return out, nil
}
