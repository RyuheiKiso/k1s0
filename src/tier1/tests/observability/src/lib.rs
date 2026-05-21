// lib.rs — tier1 観測適合仕様（spec 03）PII redaction property test
// docs/04_詳細設計/01_適合仕様/03_観測適合仕様.md §PII redaction は単一の真 + Collector 強制 に準拠する。
// signals.lock.yaml の 5 signal class（logs / metrics / traces / profiles / audit）の
// redaction 要件を property test として物理化する。
// OTel Collector processor の OTTL redact ロジックを Rust で模倣し、
// PII フィールドが出力 attribute に残らないことを assert する。

// serde_json: attribute map の構築と検証に使用する
use serde_json::{Map, Value};

// ============================================================
// PII フィールドリスト（redaction.yaml の SoT と同期する）
// ============================================================

// PII_FIELDS は spec 03 §PII redaction が対象とする 10 フィールドのリスト。
// redaction.yaml の delete_key ステートメントと 1:1 対応する（drift 防止）。
// テスト専用定数のため dead_code lint を抑制する
#[allow(dead_code)]
const PII_FIELDS: &[&str] = &[
    // メールアドレス
    "email",
    // 電話番号
    "phone",
    // 住所
    "address",
    // 氏名
    "name",
    // 納税者番号
    "tax_id",
    // 生年月日
    "birth_date",
    // クレジットカード番号
    "card_number",
    // 社会保障番号
    "ssn",
    // パスポート番号
    "passport_number",
    // 銀行口座番号
    "bank_account",
];

// ============================================================
// OTTL redaction ロジックの Rust モデル実装
// ============================================================

// apply_pii_redaction は OTel Collector の transform/redact_pii_* processor の
// OTTL `delete_key(attributes, <field>) where attributes["pii_redact_required"] == true`
// ステートメントを Rust で模倣する関数。
// attributes: 変換対象の attribute map（JSON Object）
// 戻り値: PII フィールドが除去された attribute map
// テスト専用関数のため dead_code lint を抑制する
#[allow(dead_code)]
fn apply_pii_redaction(mut attributes: Map<String, Value>) -> Map<String, Value> {
    // pii_redact_required フラグを確認する（存在しない or false の場合は何もしない）
    let should_redact = attributes
        .get("pii_redact_required")
        .and_then(Value::as_bool)
        .unwrap_or(false);

    // pii_redact_required が false の場合は attribute をそのまま返す
    if !should_redact {
        return attributes;
    }

    // PII_FIELDS の各フィールドを attribute map から削除する
    for &field in PII_FIELDS {
        // delete_key(attributes, field) の模倣: フィールドが存在すれば削除する
        attributes.remove(field);
    }

    // 処理済み attribute map を返す
    attributes
}

// ============================================================
// テスト群
// ============================================================

#[cfg(test)]
mod tests {
    // 親モジュールの apply_pii_redaction と定数を import する
    use super::*;

    // ------------------------------------------------------------
    // テスト補助関数: PII フィールドを含む attribute map を構築する
    // ------------------------------------------------------------

    // build_pii_attributes は 10 個の PII フィールドと非 PII フィールドを含む
    // attribute map を構築するテスト補助関数。
    // pii_redact_required の値を引数で制御できる。
    fn build_pii_attributes(pii_redact_required: bool) -> Map<String, Value> {
        // serde_json::Map で attribute を構築する
        let mut attrs = Map::new();

        // pii_redact_required フラグを設定する
        attrs.insert(
            "pii_redact_required".to_string(),
            Value::Bool(pii_redact_required),
        );

        // PII フィールド 10 件を設定する（テスト値はダミー文字列）
        // email フィールド
        attrs.insert("email".to_string(), Value::String("user@example.com".to_string()));
        // phone フィールド
        attrs.insert("phone".to_string(), Value::String("+81-90-1234-5678".to_string()));
        // address フィールド
        attrs.insert("address".to_string(), Value::String("Tokyo, Japan".to_string()));
        // name フィールド
        attrs.insert("name".to_string(), Value::String("Yamada Taro".to_string()));
        // tax_id フィールド
        attrs.insert("tax_id".to_string(), Value::String("123456789".to_string()));
        // birth_date フィールド
        attrs.insert("birth_date".to_string(), Value::String("1990-01-01".to_string()));
        // card_number フィールド
        attrs.insert("card_number".to_string(), Value::String("4111111111111111".to_string()));
        // ssn フィールド
        attrs.insert("ssn".to_string(), Value::String("123-45-6789".to_string()));
        // passport_number フィールド
        attrs.insert("passport_number".to_string(), Value::String("TK1234567".to_string()));
        // bank_account フィールド
        attrs.insert("bank_account".to_string(), Value::String("001-1234567".to_string()));

        // 非 PII フィールドを追加する（テナント識別子・アクター識別子）
        // tenant_id: マルチテナント識別子（PII ではない）
        attrs.insert("tenant_id".to_string(), Value::String("tenant-abc-123".to_string()));
        // actor_id: アクター（ユーザー）識別子（PII ではない: pseudonym）
        attrs.insert("actor_id".to_string(), Value::String("actor-xyz-789".to_string()));
        // trace_id: cross-signal correlation key（PII ではない）
        attrs.insert("trace_id".to_string(), Value::String("4bf92f3577b34da6a3ce929d0e0e4736".to_string()));
        // service_name: サービス名（PII ではない）
        attrs.insert("service_name".to_string(), Value::String("tier1-gateway".to_string()));

        attrs
    }

    // ------------------------------------------------------------
    // テスト 1: pii_redact_required = true の場合 PII フィールドが除去されること
    // ------------------------------------------------------------

    // pii_not_in_log_output_after_redact は
    // pii_redact_required = true の log record attribute を redact processor に通したとき
    // PII フィールドが attribute に残らないことを assert する。
    // spec 03 §PII redact defense-in-depth layer D（runtime PII 物理削除）の property test。
    #[test]
    fn pii_not_in_log_output_after_redact() {
        // pii_redact_required = true の attribute を構築する
        let attributes = build_pii_attributes(true);

        // redaction processor を適用する
        let result = apply_pii_redaction(attributes);

        // 10 個の PII フィールドが result に存在しないことを確認する
        for &field in PII_FIELDS {
            // PII フィールドが attribute に残っていないこと
            assert!(
                !result.contains_key(field),
                "PII field '{}' must not remain in attributes after redaction (spec 03 §PII redact)",
                field
            );
        }

        // pii_redact_required フラグ自体は processor が保持する（フラグは管理属性）
        // ただし spec では明示されていないため result に残存することを assert しない
    }

    // ------------------------------------------------------------
    // テスト 2: 非 PII フィールドが redact 後も保持されること
    // ------------------------------------------------------------

    // non_pii_fields_preserved は
    // pii_redact_required = true の attribute を redact processor に通したとき
    // tenant_id / actor_id 等の非 PII フィールドが保持されることを assert する。
    // 過剰削除防止（false negative）の property test。
    #[test]
    fn non_pii_fields_preserved() {
        // pii_redact_required = true の attribute を構築する
        let attributes = build_pii_attributes(true);

        // redaction processor を適用する
        let result = apply_pii_redaction(attributes);

        // tenant_id: テナント識別子が保持されていること
        assert!(
            result.contains_key("tenant_id"),
            "non-PII field 'tenant_id' must be preserved after redaction"
        );
        // actor_id: アクター識別子が保持されていること
        assert!(
            result.contains_key("actor_id"),
            "non-PII field 'actor_id' must be preserved after redaction"
        );
        // trace_id: cross-signal correlation key が保持されていること
        assert!(
            result.contains_key("trace_id"),
            "non-PII field 'trace_id' must be preserved after redaction (cross-signal primary key)"
        );
        // service_name: サービス名が保持されていること
        assert!(
            result.contains_key("service_name"),
            "non-PII field 'service_name' must be preserved after redaction"
        );

        // 保持された値が元の値と一致すること（変質していないこと）
        // tenant_id の値が不変であること
        assert_eq!(
            result["tenant_id"],
            Value::String("tenant-abc-123".to_string()),
            "tenant_id value must not be modified by redaction"
        );
        // trace_id の値が不変であること
        assert_eq!(
            result["trace_id"],
            Value::String("4bf92f3577b34da6a3ce929d0e0e4736".to_string()),
            "trace_id value must not be modified by redaction"
        );
    }

    // ------------------------------------------------------------
    // テスト 3: pii_redact_required = false の場合 PII フィールドが保持されること
    // ------------------------------------------------------------

    // redact_required_false_preserves_all は
    // pii_redact_required = false の attribute を redact processor に通したとき
    // PII フィールドが保持されることを assert する。
    // OTTL の `where attributes["pii_redact_required"] == true` 条件節の
    // フラグ制御が正しく機能することを確認する property test。
    #[test]
    fn redact_required_false_preserves_all() {
        // pii_redact_required = false の attribute を構築する
        let attributes = build_pii_attributes(false);

        // redaction processor を適用する
        let result = apply_pii_redaction(attributes);

        // pii_redact_required = false の場合は PII フィールドが保持されること
        for &field in PII_FIELDS {
            // PII フィールドが attribute に残っていること
            assert!(
                result.contains_key(field),
                "PII field '{}' must be preserved when pii_redact_required == false",
                field
            );
        }

        // 非 PII フィールドも保持されていること
        // tenant_id が保持されていること
        assert!(
            result.contains_key("tenant_id"),
            "non-PII field 'tenant_id' must be preserved when pii_redact_required == false"
        );
        // actor_id が保持されていること
        assert!(
            result.contains_key("actor_id"),
            "non-PII field 'actor_id' must be preserved when pii_redact_required == false"
        );
    }

    // ------------------------------------------------------------
    // テスト 4: pii_redact_required フラグが存在しない場合 PII フィールドが保持されること
    // ------------------------------------------------------------

    // pii_redact_flag_absent_preserves_all は
    // pii_redact_required attribute が存在しない attribute を redact processor に通したとき
    // PII フィールドが保持されることを assert する（フラグ不在 = false と等価）。
    #[test]
    fn pii_redact_flag_absent_preserves_all() {
        // pii_redact_required フラグを持たない attribute を構築する
        let mut attributes = Map::new();
        // email フィールドを設定する（PII フィールドの代表）
        attributes.insert("email".to_string(), Value::String("user@example.com".to_string()));
        // tenant_id フィールドを設定する（非 PII フィールド）
        attributes.insert("tenant_id".to_string(), Value::String("tenant-abc".to_string()));

        // redaction processor を適用する（pii_redact_required なし）
        let result = apply_pii_redaction(attributes);

        // email が保持されていること（フラグ不在 = redact しない）
        assert!(
            result.contains_key("email"),
            "PII field 'email' must be preserved when pii_redact_required flag is absent"
        );
        // tenant_id が保持されていること
        assert!(
            result.contains_key("tenant_id"),
            "non-PII field 'tenant_id' must be preserved when pii_redact_required flag is absent"
        );
    }

    // ------------------------------------------------------------
    // テスト 5: metrics signal の datapoint attribute redaction
    // ------------------------------------------------------------

    // metrics_datapoint_pii_redacted は
    // metrics signal の datapoint attribute に PII フィールドが混入した場合に
    // redact processor が物理削除することを assert する。
    // spec 03 §metrics の低 cardinality 要件の防御的二重防護（defense-in-depth layer E）。
    #[test]
    fn metrics_datapoint_pii_redacted() {
        // metrics datapoint に PII フィールドが混入したケースをシミュレートする
        let mut attributes = Map::new();
        // pii_redact_required フラグを設定する
        attributes.insert("pii_redact_required".to_string(), Value::Bool(true));
        // email フィールドが混入した（低 cardinality 違反だが processor が防衛的に削除する）
        attributes.insert("email".to_string(), Value::String("leak@example.com".to_string()));
        // 正常な metrics 属性を設定する（保持されること）
        attributes.insert("http.method".to_string(), Value::String("GET".to_string()));
        attributes.insert("http.status_code".to_string(), Value::Number(200.into()));

        // redaction processor を適用する
        let result = apply_pii_redaction(attributes);

        // email が削除されていること
        assert!(
            !result.contains_key("email"),
            "PII field 'email' must be removed from metrics datapoint attributes"
        );
        // http.method が保持されていること
        assert!(
            result.contains_key("http.method"),
            "non-PII field 'http.method' must be preserved in metrics datapoint"
        );
        // http.status_code が保持されていること
        assert!(
            result.contains_key("http.status_code"),
            "non-PII field 'http.status_code' must be preserved in metrics datapoint"
        );
    }
}
