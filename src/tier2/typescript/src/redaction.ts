// k1s0 tier2 PII フィールド redaction 実装（TypeScript 版）
// tier2/CLAUDE.md §PII 含有フィールドは field_pii annotation + Outbox 書込時に redact
// Rust 実装（rust/src/redaction.rs）の 4 言語等価実装

// PII フィールドとして扱うキーの集合（field_pii annotation 対応）
const PII_FIELD_KEYS: ReadonlySet<string> = new Set([
  // 個人氏名フィールド
  "name",
  // メールアドレスフィールド
  "email",
  // 電話番号フィールド
  "phone",
  // 住所フィールド
  "address",
  // 生年月日フィールド
  "date_of_birth",
  // 社会保障番号フィールド
  "ssn",
]);

// redact: Record から PII フィールドを除去した新しい Record を返す
// src: redact 対象の Record（変更しない）
export function redact(src: Readonly<Record<string, unknown>>): Record<string, unknown> {
  // 結果オブジェクトを初期化する
  const result: Record<string, unknown> = {};
  // 各 key-value ペアを走査する
  for (const [key, value] of Object.entries(src)) {
    // PII フィールドであれば除外する（redact）
    if (PII_FIELD_KEYS.has(key)) {
      // PII フィールドはスキップする
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
