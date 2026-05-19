// scada_opcua.rs — SCADA OPC-UA アダプタ（設計方針 16 / 外部システム統合）
// OPC-UA DataChange notification を tier2 生産進捗 proto に変換する
// external_proto_mapping.yaml の scada_opcua セクションと整合する

// anyhow: Result 型に使用する
use anyhow::{anyhow, Context, Result};
// serde: JSON シリアライズ / デシリアライズに使用する
use serde::{Deserialize, Serialize};
// serde_json: Variant 値のパースに使用する
use serde_json::Value as JsonValue;
// tracing: 構造化ロギングに使用する
use tracing::debug;

// OpcUaDataChangeNotification は OPC-UA DataChange 通知メッセージを保持する
// external_proto_mapping.yaml の source_message: "OpcUaDataChangeNotification" に対応する
#[derive(Debug, Clone, Deserialize)]
pub struct OpcUaDataChangeNotification {
    // NodeId: 値が変化した OPC-UA ノード識別子（文字列形式）
    #[serde(rename = "NodeId")]
    pub node_id: String,
    // Value_Value: 変化した値（OPC-UA Variant — JSON では任意の JSON 値として格納する）
    #[serde(rename = "Value_Value")]
    pub value_value: JsonValue,
    // Value_SourceTimestamp: 値のソースタイムスタンプ（ISO8601 形式 or .NET DateTimeOffset 文字列）
    #[serde(rename = "Value_SourceTimestamp")]
    pub value_source_timestamp: String,
    // StatusCode: OPC-UA ステータスコード（uint32、0=Good）
    #[serde(rename = "StatusCode")]
    pub status_code: u32,
    // WorkorderId: 作業指示 ID（Variant — 文字列形式の JSON 値）
    #[serde(rename = "WorkorderId")]
    pub work_order_id: JsonValue,
}

// MappedProductionProgress は tier2 ProductionProgress proto に対応するデータ構造体
// target_proto: manufacturing.production.v1.ProductionProgress に対応する
#[derive(Debug, Clone, Serialize)]
pub struct MappedProductionProgress {
    // machine_id: 機器 ID（NodeId から identity 変換）
    pub machine_id: String,
    // production_count: 生産数（OPC-UA Variant から int64 に変換）
    pub production_count: i64,
    // produced_at: 生産日時（RFC3339 形式）
    pub produced_at: String,
    // status_ok: ステータス正常フラグ（StatusCode 0=Good → true）
    pub status_ok: bool,
    // work_order_id: 作業指示 ID（Variant 文字列から identity 変換）
    pub work_order_id: String,
}

// OpcUaAdapter は OPC-UA DataChange 通知を tier2 proto 形式に変換するアダプタ
pub struct OpcUaAdapter;

impl OpcUaAdapter {
    // new は OpcUaAdapter を生成する
    pub fn new() -> Self {
        // インスタンスを返す
        Self
    }

    // map_data_change_to_production は OPC-UA 通知を MappedProductionProgress に変換する
    // external_proto_mapping.yaml の scada_opcua マッピング定義に従う
    pub fn map_data_change_to_production(
        &self,
        notif: &OpcUaDataChangeNotification,
    ) -> Result<MappedProductionProgress> {
        // NodeId を identity 変換する（そのままコピーする）
        let machine_id = notif.node_id.clone();
        // Value_Value（OPC-UA Variant）から int64 に変換する
        let production_count = opcua_variant_to_int64(&notif.value_value)
            .with_context(|| format!("Value_Value → int64 変換失敗: {:?}", notif.value_value))?;
        // Value_SourceTimestamp を RFC3339 に変換する（OPC-UA DateTimeOffset 形式対応）
        let produced_at = opcua_datetime_to_rfc3339(&notif.value_source_timestamp)
            .with_context(|| format!("SourceTimestamp 変換失敗: {}", notif.value_source_timestamp))?;
        // StatusCode を bool に変換する（0=Good=true）
        let status_ok = notif.status_code == 0;
        // WorkorderId Variant を文字列に変換する
        let work_order_id = opcua_variant_to_string(&notif.work_order_id)
            .with_context(|| format!("WorkorderId → string 変換失敗: {:?}", notif.work_order_id))?;
        // 変換ログを記録する
        debug!(
            machine_id = %machine_id,
            production_count = production_count,
            status_ok = status_ok,
            "OPC-UA DataChange → ProductionProgress 変換完了",
        );
        // 変換結果を返す
        Ok(MappedProductionProgress {
            // 各フィールドを設定する
            machine_id,
            production_count,
            produced_at,
            status_ok,
            work_order_id,
        })
    }
}

// Default trait 実装
impl Default for OpcUaAdapter {
    // default は OpcUaAdapter を生成する
    fn default() -> Self {
        // コンストラクタを呼び出す
        Self::new()
    }
}

// opcua_variant_to_int64 は OPC-UA Variant（JSON 値）を int64 に変換する
// JSON の数値型・文字列型の両方に対応する
fn opcua_variant_to_int64(variant: &JsonValue) -> Result<i64> {
    // JSON 数値の場合はそのまま変換する
    if let Some(n) = variant.as_i64() {
        // i64 を返す
        return Ok(n);
    }
    // JSON 文字列の場合は parse する
    if let Some(s) = variant.as_str() {
        // 文字列を i64 に変換する
        return s
            .trim()
            .parse::<i64>()
            .with_context(|| format!("Variant 文字列 → i64 変換失敗: {s}"));
    }
    // その他の型はエラーとする
    Err(anyhow!("OPC-UA Variant を int64 に変換できない型: {variant:?}"))
}

// opcua_variant_to_string は OPC-UA Variant（JSON 値）を文字列に変換する
fn opcua_variant_to_string(variant: &JsonValue) -> Result<String> {
    // JSON 文字列の場合はそのまま返す
    if let Some(s) = variant.as_str() {
        // 文字列を返す
        return Ok(s.to_string());
    }
    // null の場合は空文字列を返す
    if variant.is_null() {
        // 空文字列を返す
        return Ok(String::new());
    }
    // その他の型は to_string で文字列化する
    Ok(variant.to_string())
}

// opcua_datetime_to_rfc3339 は OPC-UA DateTimeOffset 形式の文字列を RFC3339 に変換する
// .NET の DateTimeOffset.ToString("O") 形式（例: "2024-01-15T12:34:56.0000000+09:00"）に対応する
fn opcua_datetime_to_rfc3339(dt_str: &str) -> Result<String> {
    // 文字列が空の場合はエラーを返す
    if dt_str.is_empty() {
        // 空文字列はエラーとする
        return Err(anyhow!("OPC-UA DateTime が空です"));
    }
    // .NET DateTimeOffset の形式（ISO8601 互換）をパースする
    // chrono の parse_from_rfc3339 が対応できる形式に整形する
    // ".NET の 7 桁小数秒（0000000）を RFC3339 の 6 桁に丸める
    let normalized = if let Some(dot_pos) = dt_str.find('.') {
        // 小数秒の部分を 6 桁に制限する
        let before_dot = &dt_str[..dot_pos];
        // tz オフセット部分を取得する（+XX:XX または Z）
        let remaining = &dt_str[dot_pos + 1..];
        // tz の開始位置を探す（+ または - または Z を探す）
        let tz_start = remaining.find(|c: char| c == '+' || c == '-' || c == 'Z')
            .unwrap_or(remaining.len());
        // 小数秒部分（最大 6 桁）
        let frac = &remaining[..tz_start.min(6)];
        // tz オフセット部分
        let tz = &remaining[tz_start..];
        // 組み立てる
        format!("{before_dot}.{frac}{tz}")
    } else {
        // 小数秒なしの場合はそのまま使用する
        dt_str.to_string()
    };
    // chrono で RFC3339 パースを試みる
    use chrono::{DateTime, FixedOffset};
    // RFC3339 としてパースする
    DateTime::<FixedOffset>::parse_from_rfc3339(&normalized)
        .with_context(|| format!("OPC-UA DateTime RFC3339 パース失敗: {normalized}"))?
        .to_rfc3339()
        .pipe(Ok)
}

// pipe ヘルパー: 関数チェーンを読みやすくする
trait Pipe: Sized {
    // pipe は self を引数に関数を適用する
    fn pipe<U, F: FnOnce(Self) -> U>(self, f: F) -> U {
        // 関数を適用して結果を返す
        f(self)
    }
}

// Pipe を文字列に実装する
impl Pipe for String {}

// OpcUaAdapter のユニットテスト
#[cfg(test)]
mod tests {
    // 親モジュールの全シンボルをインポートする
    use super::*;
    // serde_json::json! マクロをインポートする
    use serde_json::json;

    // 基本的な OPC-UA DataChange 変換テスト
    #[test]
    fn test_map_data_change_basic() {
        // テスト用 OPC-UA DataChange 通知を生成する
        let notif = OpcUaDataChangeNotification {
            // NodeId を設定する
            node_id: "ns=2;i=1001".to_string(),
            // 生産数 250 を Variant として設定する
            value_value: json!(250i64),
            // ソースタイムスタンプを設定する（ISO8601 形式）
            value_source_timestamp: "2024-01-15T09:30:00.000000+09:00".to_string(),
            // ステータスコード 0（Good）を設定する
            status_code: 0,
            // 作業指示 ID を設定する
            work_order_id: json!("WO-2024-001"),
        };
        // アダプタを生成する
        let adapter = OpcUaAdapter::new();
        // 変換を実行する
        let result = adapter.map_data_change_to_production(&notif).unwrap();
        // 機器 ID が正しく変換されることを確認する
        assert_eq!(result.machine_id, "ns=2;i=1001");
        // 生産数が正しく変換されることを確認する
        assert_eq!(result.production_count, 250);
        // ステータスが正常であることを確認する
        assert!(result.status_ok, "StatusCode=0 は Good");
        // 作業指示 ID が正しく変換されることを確認する
        assert_eq!(result.work_order_id, "WO-2024-001");
    }
}
