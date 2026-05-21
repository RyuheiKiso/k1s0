// k1s0 tier2 PII フィールド redaction 実装（Go 版）
// tier2/CLAUDE.md §PII 含有フィールドは field_pii annotation + Outbox 書込時に redact
// Rust 実装（rust/src/redaction.rs）の 4 言語等価実装
package redaction

// piiFieldKeys: PII フィールドとして扱うキーの集合（field_pii annotation 対応）
// Rust 実装（rust/src/redaction.rs）の PII_FIELDS と同一 10 キーを維持する
var piiFieldKeys = map[string]struct{}{
	// メールアドレスフィールド
	"email": {},
	// 電話番号フィールド
	"phone": {},
	// 住所フィールド
	"address": {},
	// 個人氏名フィールド
	"name": {},
	// マイナンバー / 税 ID フィールド
	"tax_id": {},
	// 生年月日フィールド（birth_date に統一）
	"birth_date": {},
	// クレジットカード番号フィールド
	"card_number": {},
	// 社会保障番号フィールド
	"ssn": {},
	// パスポート番号フィールド
	"passport_number": {},
	// 銀行口座番号フィールド
	"bank_account": {},
}

// Redact: map から PII フィールドを *** に置換した新しい map を返す
// PII フィールドのキーは piiFieldKeys に基づいて判定する
func Redact(src map[string]interface{}) map[string]interface{} {
	// 結果 map を初期化する
	result := make(map[string]interface{}, len(src))
	// 各 key-value ペアを走査する
	for k, v := range src {
		// PII フィールドであれば *** 文字列に置換する（キーは保持する）
		if _, isPii := piiFieldKeys[k]; isPii {
			// キーを保持しつつ値を *** に置換する
			result[k] = "***"
			// 次のキーへ進む
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
