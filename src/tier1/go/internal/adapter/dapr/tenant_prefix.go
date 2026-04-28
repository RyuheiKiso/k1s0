// 本ファイルは tier1 の L2 テナント分離（NFR-E-AC-003）を adapter 層で強制するヘルパ。
//
// 設計正典:
//   docs/03_要件定義/20_機能要件/10_tier1_API要件/00_tier1_API共通規約.md
//     §「マルチテナント分離」:
//       L2（ルーティング）: バックエンドのキー / トピック / バケット / パーティションに
//       `<tenant_id>/` を自動付与
//       L2 のキー空間分離は tier2/tier3 から不可視。tier2 が SetState("foo", ...) と
//       呼んだ場合、物理キーは <tenant_id>/foo になる。
//
// 役割:
//   - 物理キー / トピック / バケットに `<tenant_id>/` を強制付与する
//   - 応答（BulkGet / Subscribe Receive）から prefix を剥がして tier2/tier3 に透過させる
//   - tier_id が空の呼出は短絡（handler 側 requireTenantID で弾かれる前提）
//
// 適用境界:
//   adapter 層（dapr.StateAdapter / dapr.PubSubAdapter / dapr.BindingAdapter）。
//   handler 層は「tier2 が見るキー」を扱い続け、prefix の存在を意識しない。

package dapr

// tenantSeparator は <tenant_id> と元キー / トピック / バケットを区切る区切り文字。
// docs 共通規約に "<tenant_id>/foo" 形式と明記されているため "/" 固定。
const tenantSeparator = "/"

// prefixKey は物理キーに `<tenant_id>/` を付与する。
// tenantID が空文字の場合は元キーをそのまま返す（in-memory backend の test フィクスチャ用）。
// production の handler は requireTenantID で空文字を InvalidArgument に翻訳しているため、
// adapter 到達時は通常 tenantID が非空。
func prefixKey(tenantID, key string) string {
	// テナント未指定（test 専用 / handler を経由しない経路）はそのまま返す。
	if tenantID == "" {
		return key
	}
	// 既に prefix 付きの key を二重 prefix しないため hasTenantPrefix で短絡する。
	// （handler 経由では起こらないが、tier1 内の中継テストで重複を避ける defensive 措置）
	if hasTenantPrefix(tenantID, key) {
		return key
	}
	return tenantID + tenantSeparator + key
}

// stripKey は応答キーから `<tenant_id>/` を取り除いて tier2/tier3 視点のキーに戻す。
// prefix が付いていない場合（test 中継 / 不正な component 応答）は元キーをそのまま返す。
func stripKey(tenantID, key string) string {
	// tenantID 空ならそのまま返す（test 専用経路）。
	if tenantID == "" {
		return key
	}
	// `<tenant_id>/` の正確な前方一致のみ剥がす。一致しない場合は破壊しない。
	prefix := tenantID + tenantSeparator
	if len(key) < len(prefix) {
		return key
	}
	if key[:len(prefix)] != prefix {
		return key
	}
	return key[len(prefix):]
}

// prefixKeys は []string の各要素に prefix を付与した新スライスを返す。元スライスは破壊しない。
func prefixKeys(tenantID string, keys []string) []string {
	// tenantID 空時は元スライスをそのまま返す（割当を発生させない）。
	if tenantID == "" {
		return keys
	}
	// 新スライスを確保して 1 件ずつ詰め直す。
	out := make([]string, len(keys))
	for i, k := range keys {
		out[i] = prefixKey(tenantID, k)
	}
	return out
}

// hasTenantPrefix は key が `<tenant_id>/` で始まっているかを判定する。
// 二重 prefix 防止 / strip 判定に使用する。
func hasTenantPrefix(tenantID, key string) bool {
	prefix := tenantID + tenantSeparator
	if len(key) < len(prefix) {
		return false
	}
	return key[:len(prefix)] == prefix
}
