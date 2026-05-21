// k1s0 tier2 PII フィールド redaction 実装（TypeScript 版）
// tier2/CLAUDE.md §PII 含有フィールドは field_pii annotation + Outbox 書込時に redact
// Rust 実装（rust/src/redaction.rs）の 4 言語等価実装

// PII フィールドとして扱うキーの集合（field_pii annotation 対応）
// Rust 実装（rust/src/redaction.rs）の PII_FIELDS と同一 10 キーを維持する
const PII_FIELD_KEYS: ReadonlySet<string> = new Set([
  // メールアドレスフィールド
  "email",
  // 電話番号フィールド
  "phone",
  // 住所フィールド
  "address",
  // 個人氏名フィールド
  "name",
  // マイナンバー / 税 ID フィールド
  "tax_id",
  // 生年月日フィールド（birth_date に統一）
  "birth_date",
  // クレジットカード番号フィールド
  "card_number",
  // 社会保障番号フィールド
  "ssn",
  // パスポート番号フィールド
  "passport_number",
  // 銀行口座番号フィールド
  "bank_account",
]);

// redact: Record から PII フィールドを *** に置換した新しい Record を返す
// src: redact 対象の Record（変更しない）
export function redact(src: Readonly<Record<string, unknown>>): Record<string, unknown> {
  // 結果オブジェクトを初期化する
  const result: Record<string, unknown> = {};
  // 各 key-value ペアを走査する
  for (const [key, value] of Object.entries(src)) {
    // PII フィールドであれば *** 文字列に置換する（キーは保持する）
    if (PII_FIELD_KEYS.has(key)) {
      // キーを保持しつつ値を *** に置換する
      result[key] = "***";
      // 次のキーへ進む
      continue;
    }
    // 非 PII フィールドはそのまま結果に追加する
    result[key] = value;
  }
  // redact 済みオブジェクトを返す
  return result;
}

// isPiiField: キーが PII フィールドかどうかを返す
export function isPiiField(key: string): boolean {
  // PII_FIELD_KEYS にキーが存在するか確認する
  return PII_FIELD_KEYS.has(key);
}
