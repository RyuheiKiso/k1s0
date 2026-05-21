// k1s0 tier2 PII フィールド redaction 実装（Go 版）
// tier2/CLAUDE.md §PII 含有フィールドは field_pii annotation + Outbox 書込時に redact
// Rust 実装（rust/src/redaction.rs）の 4 言語等価実装
package redaction

// piiFieldKeys: PII フィールドとして扱うキーの集合（field_pii annotation 対応）
var piiFieldKeys = map[string]struct{}{
	// 個人氏名フィールド
	"name": {},
	// メールアドレスフィールド
	"email": {},
	// 電話番号フィールド
	"phone": {},
	// 住所フィールド
	"address": {},
	// 生年月日フィールド
	"date_of_birth": {},
	// 社会保障番号フィールド
	"ssn": {},
}

// Redact: map から PII フィールドを除去した新しい map を返す
// PII フィールドのキーは piiFieldKeys に基づいて判定する
func Redact(src map[string]interface{}) map[string]interface{} {
	// 結果 map を初期化する
	result := make(map[string]interface{}, len(src))
	// 各 key-value ペアを走査する
	for k, v := range src {
		// PII フィールドであれば除外する（redact）
		if _, isPii := piiFieldKeys[k]; isPii {
			// PII フィールドはスキップする
			continue
		}
		// 非 PII フィールドはそのまま結果に追加する
		result[k] = v
	}
	// redact 済み map を返す
	return result
}

// IsPiiField: キーが PII フィールドかどうかを返す
func IsPiiField(key string) bool {
	// piiFieldKeys にキーが存在するか確認する
	_, isPii := piiFieldKeys[key]
	// 結果を返す
	return isPii
}
