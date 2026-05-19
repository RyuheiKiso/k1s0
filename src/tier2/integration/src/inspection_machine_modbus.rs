// inspection_machine_modbus.rs — 検査機器 Modbus TCP アダプタ（設計方針 16 / 外部システム統合）
// Modbus holding register 読み取り結果を tier2 品質検査 proto に変換する
// external_proto_mapping.yaml の inspection_machine_modbus セクションと整合する

// anyhow: Result 型に使用する
use anyhow::{anyhow, Context, Result};
// serde: JSON シリアライズに使用する
use serde::{Deserialize, Serialize};
// tracing: 構造化ロギングに使用する
use tracing::debug;

// ModbusHoldingRegisterFrame は Modbus holding register 読み取り結果を保持する
// external_proto_mapping.yaml の source_message: "ModbusHoldingRegisterFrame" に対応する
// register_0〜register_6 は Modbus コイル番号に対応するレジスタ値（u16）
#[derive(Debug, Clone, Deserialize)]
pub struct ModbusHoldingRegisterFrame {
    // register_0: 計測値 float32 の高 word（big-endian）
    pub register_0: u16,
    // register_1: 計測値 float32 の低 word（big-endian）
    pub register_1: u16,
    // register_2: 機器 ID（uint16）
    pub register_2: u16,
    // register_3: 検査種別（uint16、0=dimensional, 1=surface, 2=weight）
    pub register_3: u16,
    // register_4: 計測時刻 Unix timestamp の高 word（uint32 big-endian）
    pub register_4: u16,
    // register_5: 計測時刻 Unix timestamp の低 word（uint32 big-endian）
    pub register_5: u16,
    // register_6: 合否判定（uint16、0=pass, 1=fail）
    pub register_6: u16,
}

// InspectionType は検査種別を表す列挙型
#[derive(Debug, Clone, Serialize, PartialEq)]
pub enum InspectionType {
    // 寸法検査
    Dimensional,
    // 表面検査
    Surface,
    // 重量検査
    Weight,
    // 不明な検査種別
    Unknown(u16),
}

// MappedInspectionResult は tier2 InspectionResult proto に対応するデータ構造体
// target_proto: manufacturing.quality.v1.InspectionResult に対応する
#[derive(Debug, Clone, Serialize)]
pub struct MappedInspectionResult {
    // measured_value: 計測値（float32 → f64）
    pub measured_value: f64,
    // machine_id: 機器 ID 文字列（MACHINE-{id:04} 形式）
    pub machine_id: String,
    // inspection_type: 検査種別文字列（"dimensional" / "surface" / "weight" / "unknown"）
    pub inspection_type: String,
    // measured_at: 計測日時（RFC3339 形式）
    pub measured_at: String,
    // pass_fail: 合否（true=pass, false=fail）
    pub pass_fail: bool,
}

// ModbusAdapter は Modbus holding register フレームを tier2 proto 形式に変換するアダプタ
pub struct ModbusAdapter;

impl ModbusAdapter {
    // new は ModbusAdapter を生成する
    pub fn new() -> Self {
        // インスタンスを返す
        Self
    }

    // map_holding_register_to_inspection は Modbus フレームを MappedInspectionResult に変換する
    // external_proto_mapping.yaml の inspection_machine_modbus マッピング定義に従う
    pub fn map_holding_register_to_inspection(
        &self,
        frame: &ModbusHoldingRegisterFrame,
    ) -> Result<MappedInspectionResult> {
        // register_0, register_1 から float32 計測値を変換する（big-endian 2 word）
        let measured_value = decode_2word_float32_be(frame.register_0, frame.register_1)?;
        // register_2 から機器 ID 文字列を生成する
        let machine_id = format!("MACHINE-{:04}", frame.register_2);
        // register_3 から検査種別を変換する
        let inspection_type = match frame.register_3 {
            // 0 は寸法検査を示す
            0 => "dimensional".to_string(),
            // 1 は表面検査を示す
            1 => "surface".to_string(),
            // 2 は重量検査を示す
            2 => "weight".to_string(),
            // それ以外は不明とする
            other => format!("unknown_{other}"),
        };
        // register_4, register_5 から Unix timestamp を取得して RFC3339 に変換する
        let unix_ts = decode_2word_uint32_be(frame.register_4, frame.register_5);
        // Unix timestamp を RFC3339 形式の文字列に変換する
        let measured_at = unix_ts_to_rfc3339(unix_ts)?;
        // register_6 から合否を変換する（0=pass=true, その他=fail=false）
        let pass_fail = frame.register_6 == 0;
        // 変換ログを記録する
        debug!(
            machine_id = %machine_id,
            inspection_type = %inspection_type,
            pass_fail = pass_fail,
            "Modbus → InspectionResult 変換完了",
        );
        // 変換結果を返す
        Ok(MappedInspectionResult {
            // 各フィールドを設定する
            measured_value,
            machine_id,
            inspection_type,
            measured_at,
            pass_fail,
        })
    }
}

// Default trait 実装
impl Default for ModbusAdapter {
    // default は ModbusAdapter を生成する
    fn default() -> Self {
        // コンストラクタを呼び出す
        Self::new()
    }
}

// decode_2word_float32_be は 2 つの u16 word から big-endian float32 をデコードする
// Modbus 2-word float: high_word = word0, low_word = word1
fn decode_2word_float32_be(high_word: u16, low_word: u16) -> Result<f64> {
    // 2 つの u16 を u32 に結合する（big-endian: high_word が上位 16 bit）
    let bits = ((high_word as u32) << 16) | (low_word as u32);
    // IEEE 754 float32 としてデコードする
    let f32_val = f32::from_bits(bits);
    // NaN / Infinity のチェックを行う
    if f32_val.is_nan() || f32_val.is_infinite() {
        // NaN または Infinity は無効な計測値としてエラーを返す
        return Err(anyhow!("Modbus float32 デコード失敗: NaN または Infinity"));
    }
    // f32 を f64 に変換して返す
    Ok(f32_val as f64)
}

// decode_2word_uint32_be は 2 つの u16 word から big-endian uint32 をデコードする
fn decode_2word_uint32_be(high_word: u16, low_word: u16) -> u32 {
    // 2 つの u16 を u32 に結合する（big-endian）
    ((high_word as u32) << 16) | (low_word as u32)
}

// unix_ts_to_rfc3339 は Unix timestamp（u32）を RFC3339 形式に変換する
fn unix_ts_to_rfc3339(unix_ts: u32) -> Result<String> {
    // Unix timestamp が 0 の場合はデフォルト値を返す
    if unix_ts == 0 {
        // epoch 0 は無効な計測時刻としてエラーを返す
        return Err(anyhow!("Modbus Unix timestamp が 0 です"));
    }
    // 秒単位の Unix timestamp から年月日時分秒を計算する（簡易実装）
    // 実際の production では chrono::DateTime::from_timestamp を使用することを推奨する
    let seconds = unix_ts as i64;
    // 1970-01-01 からの経過秒数を RFC3339 にフォーマットする
    // 簡易実装: NaiveDateTime を直接構築する
    use chrono::{NaiveDateTime, TimeZone, Utc};
    // NaiveDateTime に変換する
    let naive = NaiveDateTime::from_timestamp_opt(seconds, 0)
        .ok_or_else(|| anyhow!("Unix timestamp 変換失敗: {unix_ts}"))?;
    // UTC DateTime に変換して RFC3339 形式で返す
    Ok(Utc.from_utc_datetime(&naive).to_rfc3339())
}

// ModbusAdapter のユニットテスト
#[cfg(test)]
mod tests {
    // 親モジュールの全シンボルをインポートする
    use super::*;

    // 基本的な Modbus フレーム変換テスト
    #[test]
    fn test_map_holding_register_basic() {
        // IEEE 754 float32 の 1.0 を 2 word に分割する（big-endian）
        // 1.0f32 = 0x3F800000
        let float_high: u16 = 0x3F80;
        // 低 word を設定する
        let float_low: u16 = 0x0000;
        // テスト用 Modbus フレームを生成する
        let frame = ModbusHoldingRegisterFrame {
            // 計測値 1.0 の高 word
            register_0: float_high,
            // 計測値 1.0 の低 word
            register_1: float_low,
            // 機器 ID 5
            register_2: 5,
            // 検査種別: 寸法検査（0）
            register_3: 0,
            // 計測時刻 Unix timestamp 高 word（2024-01-01 00:00:00 UTC = 1704067200）
            register_4: 0x6594,
            // 計測時刻 Unix timestamp 低 word
            register_5: 0x4F80,
            // 合否: pass（0）
            register_6: 0,
        };
        // アダプタを生成する
        let adapter = ModbusAdapter::new();
        // 変換を実行する
        let result = adapter.map_holding_register_to_inspection(&frame).unwrap();
        // 計測値が正しく変換されることを確認する
        assert!((result.measured_value - 1.0).abs() < 0.001, "計測値 1.0 が期待値");
        // 機器 ID が正しく変換されることを確認する
        assert_eq!(result.machine_id, "MACHINE-0005");
        // 検査種別が正しく変換されることを確認する
        assert_eq!(result.inspection_type, "dimensional");
        // 合否が正しく変換されることを確認する
        assert!(result.pass_fail, "register_6=0 は pass");
    }
}
