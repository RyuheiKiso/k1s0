// pii_pipeline.ts — field_pii annotation 駆動の自動 PII strip pipeline
// spec 11 §整合 6: PII フィールドを Outbox 書込前に自動除去する pipeline
// reducer / persistence 入口で必須適用する

// PII として strip するフィールド名リスト（spec に準拠する）
const PII_FIELD_NAMES: ReadonlySet<string> = new Set([
  // メールアドレス
  'email',
  // 電話番号
  'phone',
  // 住所
  'address',
  // 氏名
  'name',
  // マイナンバー / 税 ID
  'tax_id',
  // 生年月日
  'birth_date',
  // クレジットカード番号
  'card_number',
]);

// PII フィールドを再帰的に strip して *** に置換する関数
export function stripPiiFields<T>(value: T): T {
  // null / undefined はそのまま返す
  if (value === null || value === undefined) {
    return value;
  }
  // 配列の場合は各要素を再帰的に処理する
  if (Array.isArray(value)) {
    return value.map(stripPiiFields) as unknown as T;
  }
  // オブジェクトの場合はフィールドを確認する
  if (typeof value === 'object') {
    // 結果オブジェクトを組み立てる
    const result: Record<string, unknown> = {};
    // 各フィールドを走査する
    for (const [key, val] of Object.entries(value as Record<string, unknown>)) {
      // PII フィールド名と一致する場合は *** に置換する
      if (PII_FIELD_NAMES.has(key)) {
        result[key] = '***';
      } else {
        // ネストした値を再帰的に処理する
        result[key] = stripPiiFields(val);
      }
    }
    // strip 済みオブジェクトを返す
    return result as T;
  }
  // プリミティブ値はそのまま返す
  return value;
}
