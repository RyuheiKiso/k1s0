// redaction.rs — PII フィールドの redaction 機構（spec 10 §PII redact）
// Outbox に書き込む前に field_pii フィールドを *** に置換する
// serde_json の Value を走査して pii_fields リストに含まれるキーを redact する
// テナント分離適合仕様 10 の PiiSegregated table_class と連動する

// JSON 操作のためのモジュールをインポートする
use serde_json::Value;

// redaction 対象の PII フィールド名リスト（spec 10 §PII redact に準拠する）
// このリストに含まれるキーは Outbox 書込前に *** に置換される
const PII_FIELDS: &[&str] = &[
    // メールアドレス
    "email",
    // 電話番号
    "phone",
    // 住所
    "address",
    // 氏名（姓名）
    "name",
    // マイナンバー / 税 ID
    "tax_id",
    // 生年月日
    "birth_date",
    // クレジットカード番号
    "card_number",
    // 社会保障番号（海外テナント向け）
    "ssn",
    // パスポート番号
    "passport_number",
    // 銀行口座番号
    "bank_account",
];

// PII_FIELDS の定数を外部参照可能に公開する（テスト・検証用）
pub const PII_FIELD_NAMES: &[&str] = PII_FIELDS;

// JSON Value の中の PII フィールドを *** に再帰的に置換して返す関数
// 元の Value は消費されて redact 済みの Value が返される
pub fn redact_pii_fields(value: Value) -> Value {
    // Value の型を判別して適切な処理を行う
    match value {
        // Object の場合は各フィールドを順番に確認する
        Value::Object(mut map) => {
            // フィールドを走査して PII キーを *** に置換する
            for (key, val) in map.iter_mut() {
                // PII フィールド名と一致するキーを確認する
                if PII_FIELDS.contains(&key.as_str()) {
                    // PII フィールドを *** に置換する（型を問わず文字列で上書きする）
                    *val = Value::String("***".to_string());
                } else {
                    // PII でないフィールドはネストを再帰的に処理する
                    *val = redact_pii_fields(val.take());
                }
            }
            // redact 済みの Object を返す
            Value::Object(map)
        }
        // Array の場合は各要素を再帰的に処理する
        Value::Array(items) => {
            // 各要素に redact_pii_fields を適用して新しい配列を返す
            Value::Array(items.into_iter().map(redact_pii_fields).collect())
        }
        // String / Number / Bool / Null はそのまま返す（PII チェックは Object のキーのみ）
        other => other,
    }
}

// テスト: PII フィールドが正しく redact されることを確認する
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // PII フィールドが *** に置換されることを確認する
    #[test]
    fn test_pii_fields_are_redacted() {
        // PII を含む JSON を用意する
        let input = json!({
            "tenant_id": "uuid-1",
            "email": "alice@example.com",
            "phone": "090-1234-5678",
            "amount": 1000
        });
        // redaction を実行する
        let output = redact_pii_fields(input);
        // email が *** になっていることを確認する
        assert_eq!(output["email"], "***");
        // phone が *** になっていることを確認する
        assert_eq!(output["phone"], "***");
        // PII でない tenant_id は変更されていないことを確認する
        assert_eq!(output["tenant_id"], "uuid-1");
        // 数値フィールドの amount は変更されていないことを確認する
        assert_eq!(output["amount"], 1000);
    }

    // ネストした PII フィールドも redact されることを確認する
    #[test]
    fn test_nested_pii_redacted() {
        // ネストした PII を含む JSON を用意する
        let input = json!({
            "user": {
                "email": "bob@example.com",
                "id": "uuid-2"
            }
        });
        // redaction を実行する
        let output = redact_pii_fields(input);
        // ネストした email が *** になっていることを確認する
        assert_eq!(output["user"]["email"], "***");
        // id は変更されていないことを確認する
        assert_eq!(output["user"]["id"], "uuid-2");
    }

    // Array 内のオブジェクトの PII フィールドも redact されることを確認する
    #[test]
    fn test_array_pii_redacted() {
        // 配列内に PII を含む JSON を用意する
        let input = json!([
            {"name": "Alice", "order_id": "ORD-001"},
            {"name": "Bob", "order_id": "ORD-002"}
        ]);
        // redaction を実行する
        let output = redact_pii_fields(input);
        // 配列の各要素の name が *** になっていることを確認する
        assert_eq!(output[0]["name"], "***");
        assert_eq!(output[1]["name"], "***");
        // order_id は変更されていないことを確認する
        assert_eq!(output[0]["order_id"], "ORD-001");
    }

    // PII フィールドが存在しない場合は元の構造を保持することを確認する
    #[test]
    fn test_no_pii_fields_unchanged() {
        // PII を含まない JSON を用意する
        let input = json!({
            "order_id": "ORD-001",
            "status": "in_progress",
            "quantity": 5
        });
        // redaction を実行する
        let output = redact_pii_fields(input);
        // 全フィールドが変更されていないことを確認する
        assert_eq!(output["order_id"], "ORD-001");
        assert_eq!(output["status"], "in_progress");
        assert_eq!(output["quantity"], 5);
    }

    // 数値型の PII フィールドも *** 文字列に置換されることを確認する
    #[test]
    fn test_numeric_pii_field_redacted_to_string() {
        // 数値型の tax_id を含む JSON を用意する
        let input = json!({
            "tax_id": 123456789,
            "company": "ACME"
        });
        // redaction を実行する
        let output = redact_pii_fields(input);
        // 数値型の tax_id も *** 文字列に変換されることを確認する
        assert_eq!(output["tax_id"], "***");
        // company は変更されていないことを確認する
        assert_eq!(output["company"], "ACME");
    }

    // Outbox ペイロードの no-PII テスト: redact 後も PII キーが *** のみであることを確認する
    #[test]
    fn test_outbox_payload_after_redaction_no_pii_plaintext() {
        // PII を含む可能性のある Outbox ペイロードを用意する
        let input = json!({
            "aggregate_type": "Customer",
            "data": {
                "email": "charlie@example.com",
                "phone": "03-1234-5678",
                "order_id": "ORD-999",
                "amount": 5000
            },
            "metadata": {
                "trace_id": "abc-def",
                "version": 1
            }
        });
        // redaction を実行する
        let output = redact_pii_fields(input);
        // JSON 文字列に変換して PII 平文が含まれないことを確認する
        let json_str = serde_json::to_string(&output).unwrap();
        // PII 平文が含まれないことを確認する
        assert!(!json_str.contains("charlie@example.com"));
        assert!(!json_str.contains("03-1234-5678"));
        // PII でないフィールドは保持されることを確認する
        assert!(json_str.contains("ORD-999"));
        assert!(json_str.contains("5000"));
    }
}
