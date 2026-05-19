// edi_x12.rs — ANSI X12 EDI 取引先電文アダプタ（設計方針 16 / 外部システム統合）
// X12 850 Purchase Order トランザクションを tier2 proto の PurchaseOrder に変換する
// external_proto_mapping.yaml の edi_x12 セクションと整合する

// anyhow: Result 型に使用する
use anyhow::{anyhow, Context, Result};
// chrono: 日付変換に使用する
use chrono::NaiveDate;
// serde: JSON シリアライズに使用する
use serde::{Deserialize, Serialize};
// tracing: 構造化ロギングに使用する
use tracing::debug;

// X12_850_PurchaseOrder は ANSI X12 850 発注トランザクションの主要セグメントを保持する
// external_proto_mapping.yaml の source_message: "X12_850_PurchaseOrder" に対応する
#[derive(Debug, Clone, Deserialize)]
pub struct X12850PurchaseOrder {
    // BEG03_OrderNumber: 発注番号（BEG セグメントの要素 03）
    #[serde(rename = "BEG03_OrderNumber")]
    pub beg03_order_number: String,
    // BEG05_OrderDate: 発注日（YYYYMMDD 形式、BEG セグメントの要素 05）
    #[serde(rename = "BEG05_OrderDate")]
    pub beg05_order_date: String,
    // PO1_01_OrderedQuantity: 発注数量（PO1 セグメントの要素 01、文字列形式）
    #[serde(rename = "PO1_01_OrderedQuantity")]
    pub po1_01_ordered_quantity: String,
    // PO1_03_UnitPrice: 単価（PO1 セグメントの要素 03、小数点文字列形式）
    #[serde(rename = "PO1_03_UnitPrice")]
    pub po1_03_unit_price: String,
    // N1_02_Name: 取引先名称（N1 セグメントの要素 02）
    #[serde(rename = "N1_02_Name")]
    pub n1_02_name: String,
}

// MappedPurchaseOrder は tier2 PurchaseOrder proto に対応するデータ構造体
// target_proto: manufacturing.procurement.v1.PurchaseOrder に対応する
#[derive(Debug, Clone, Serialize)]
pub struct MappedPurchaseOrder {
    // order_id: 発注番号（BEG03 から identity 変換）
    pub order_id: String,
    // ordered_at: 発注日時（RFC3339 形式）
    pub ordered_at: String,
    // quantity: 発注数量（int64）
    pub quantity: i64,
    // unit_price_jpy: 単価（円単位 int64）
    pub unit_price_jpy: i64,
    // customer_code: 取引先コード（N1_02 から identity 変換）
    pub customer_code: String,
}

// X12Adapter は X12 EDI 電文を tier2 proto 形式に変換するアダプタ
pub struct X12Adapter;

impl X12Adapter {
    // new は X12Adapter を生成する
    pub fn new() -> Self {
        // インスタンスを返す
        Self
    }

    // map_850_to_purchase_order は X12 850 発注電文を MappedPurchaseOrder に変換する
    // external_proto_mapping.yaml の edi_x12 マッピング定義に従う
    pub fn map_850_to_purchase_order(
        &self,
        x12: &X12850PurchaseOrder,
    ) -> Result<MappedPurchaseOrder> {
        // BEG03 発注番号を identity 変換する（そのままコピーする）
        let order_id = x12.beg03_order_number.clone();
        // BEG05 発注日を YYYYMMDD から RFC3339 に変換する
        let ordered_at = convert_yyyymmdd_to_rfc3339(&x12.beg05_order_date)
            .with_context(|| format!("BEG05 日付変換失敗: {}", x12.beg05_order_date))?;
        // PO1_01 発注数量を文字列から int64 に変換する
        let quantity = x12
            .po1_01_ordered_quantity
            .trim()
            .parse::<i64>()
            .with_context(|| format!("PO1_01 数量変換失敗: {}", x12.po1_01_ordered_quantity))?;
        // PO1_03 単価を小数点文字列から円単位 int64 に変換する
        let unit_price_jpy = convert_decimal_string_to_jpy(&x12.po1_03_unit_price)
            .with_context(|| format!("PO1_03 単価変換失敗: {}", x12.po1_03_unit_price))?;
        // N1_02 取引先名称を identity 変換する
        let customer_code = x12.n1_02_name.clone();
        // 変換ログを記録する
        debug!(
            order_id = %order_id,
            quantity = quantity,
            "X12 850 → PurchaseOrder 変換完了",
        );
        // 変換結果を返す
        Ok(MappedPurchaseOrder {
            // 各フィールドを設定する
            order_id,
            ordered_at,
            quantity,
            unit_price_jpy,
            customer_code,
        })
    }
}

// Default trait 実装: X12Adapter::new() と同等
impl Default for X12Adapter {
    // default は X12Adapter を生成する
    fn default() -> Self {
        // コンストラクタを呼び出す
        Self::new()
    }
}

// convert_yyyymmdd_to_rfc3339 は YYYYMMDD 形式の日付文字列を RFC3339 に変換する
// 例: "20240115" → "2024-01-15T00:00:00+09:00"
fn convert_yyyymmdd_to_rfc3339(yyyymmdd: &str) -> Result<String> {
    // 文字列長を確認する（YYYYMMDD は 8 文字）
    if yyyymmdd.len() != 8 {
        // 長さ不一致はエラーとする
        return Err(anyhow!("YYYYMMDD 形式ではない: {yyyymmdd}"));
    }
    // NaiveDate にパースする（chrono を使用する）
    let date = NaiveDate::parse_from_str(yyyymmdd, "%Y%m%d")
        .with_context(|| format!("YYYYMMDD パース失敗: {yyyymmdd}"))?;
    // RFC3339 形式の文字列を生成する（UTC 00:00:00 として出力する）
    Ok(format!("{}T00:00:00Z", date.format("%Y-%m-%d")))
}

// convert_decimal_string_to_jpy は "12345.67" 形式の小数点文字列を円単位 int64 に変換する
// 小数点以下は切り捨てる（外部 EDI の精度に合わせる）
fn convert_decimal_string_to_jpy(price_str: &str) -> Result<i64> {
    // 文字列をトリムする
    let trimmed = price_str.trim();
    // 小数点の有無で分岐する
    if let Some(dot_pos) = trimmed.find('.') {
        // 小数点前の整数部分を取得する
        let integer_part = &trimmed[..dot_pos];
        // 整数部分を i64 に変換する（円単位）
        integer_part
            .parse::<i64>()
            .with_context(|| format!("単価整数部変換失敗: {integer_part}"))
    } else {
        // 小数点なしの場合はそのまま変換する
        trimmed
            .parse::<i64>()
            .with_context(|| format!("単価変換失敗: {trimmed}"))
    }
}

// X12Adapter のユニットテスト
#[cfg(test)]
mod tests {
    // 親モジュールの全シンボルをインポートする
    use super::*;

    // X12 850 の基本的な変換テスト
    #[test]
    fn test_map_850_basic() {
        // テスト用 X12 850 発注電文を生成する
        let x12 = X12850PurchaseOrder {
            // 発注番号を設定する
            beg03_order_number: "PO-2024-001".to_string(),
            // 発注日を設定する（YYYYMMDD 形式）
            beg05_order_date: "20240115".to_string(),
            // 発注数量を設定する
            po1_01_ordered_quantity: "100".to_string(),
            // 単価を設定する（円単位）
            po1_03_unit_price: "1500.50".to_string(),
            // 取引先名称を設定する
            n1_02_name: "SUPPLIER-001".to_string(),
        };
        // アダプタを生成する
        let adapter = X12Adapter::new();
        // 変換を実行する
        let result = adapter.map_850_to_purchase_order(&x12).unwrap();
        // 発注番号が正しく変換されることを確認する
        assert_eq!(result.order_id, "PO-2024-001");
        // 発注日が RFC3339 に変換されることを確認する
        assert_eq!(result.ordered_at, "2024-01-15T00:00:00Z");
        // 数量が正しく変換されることを確認する
        assert_eq!(result.quantity, 100);
        // 単価が円単位で正しく変換されることを確認する（小数点以下切り捨て）
        assert_eq!(result.unit_price_jpy, 1500);
        // 取引先コードが正しく変換されることを確認する
        assert_eq!(result.customer_code, "SUPPLIER-001");
    }
}
