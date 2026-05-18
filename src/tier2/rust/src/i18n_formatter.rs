// k1s0 tier2 i18n formatter Rust インターフェース定義
// ICU 風の number / date / currency / unit formatter の Rust トレイト
// wall clock TTL 禁止規約準拠: 日付は HLC タイムスタンプから変換した上で fmt へ渡す

// anyhow: Result 型に使用する
use anyhow::Result;

/// LocaleFormatterOptions: 数値フォーマット オプション
/// ICU NumberFormat オプションに相当するオプション集合
#[derive(Debug, Clone, Default)]
pub struct NumberFormatOptions {
    // 小数点以下の最小桁数（指定なしは None）
    pub minimum_fraction_digits: Option<u8>,
    // 小数点以下の最大桁数（指定なしは None）
    pub maximum_fraction_digits: Option<u8>,
    // 千単位区切り文字を使用するか
    pub use_grouping: bool,
}

/// DateFormat: 日付フォーマットの種別（ICU の DateTimeStyle に対応する）
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DateFormat {
    // 短形式（例: 2024/01/15）
    Short,
    // 中形式（例: 2024年1月15日）
    Medium,
    // 長形式（例: 2024年1月15日 月曜日）
    Long,
    // カスタム書式文字列（ICU パターン形式）
    Custom(String),
}

/// LocaleFormatter トレイト: ICU 風 number / date / currency / unit formatter の契約
/// 4 言語等価原則により Rust / C# / Go / TypeScript で同じシグネチャを持つ
/// wall clock を直接 TTL に使うことを禁止し、HLC timestamp を経由する設計を維持する
pub trait LocaleFormatter: Send + Sync {
    /// formatNumber: 数値をロケール形式にフォーマットする
    /// value: フォーマット対象の数値（f64）
    /// locale: IETF BCP 47 ロケールタグ（例: "ja-JP" / "en-US"）
    /// options: フォーマットオプション（小数点桁数 / 区切り文字など）
    /// フォーマットされた文字列を返す
    fn format_number(&self, value: f64, locale: &str, options: &NumberFormatOptions) -> Result<String>;

    /// formatDate: 日付をロケール形式にフォーマットする
    /// timestamp_millis: ミリ秒エポック形式の日付（HLC タイムスタンプから変換する）
    /// locale: IETF BCP 47 ロケールタグ（例: "ja-JP" / "en-US"）
    /// format: 日付フォーマット種別（Short / Medium / Long / Custom）
    /// フォーマットされた文字列を返す
    fn format_date(&self, timestamp_millis: i64, locale: &str, format: &DateFormat) -> Result<String>;

    /// formatCurrency: 通貨金額をロケール形式にフォーマットする
    /// amount: フォーマット対象の金額（f64）
    /// currency: ISO 4217 通貨コード（例: "JPY" / "USD" / "EUR"）
    /// locale: IETF BCP 47 ロケールタグ（例: "ja-JP" / "en-US"）
    /// フォーマットされた文字列を返す
    fn format_currency(&self, amount: f64, currency: &str, locale: &str) -> Result<String>;

    /// formatUnit: 単位付き数値をロケール形式にフォーマットする
    /// value: フォーマット対象の数値（f64）
    /// unit: CLDR unit identifier（例: "kilogram" / "meter" / "liter"）
    /// locale: IETF BCP 47 ロケールタグ（例: "ja-JP" / "en-US"）
    /// フォーマットされた文字列を返す
    fn format_unit(&self, value: f64, unit: &str, locale: &str) -> Result<String>;
}

/// LocaleFormatterStub: テスト / stub 実装
/// 実際のロケール変換は行わず、引数をそのまま文字列化して返す
/// production / development 区別禁止規約に従い、本実装は必ず ICU backend を使用すること
pub struct LocaleFormatterStub;

impl LocaleFormatter for LocaleFormatterStub {
    // format_number スタブ実装（ロケール変換なしで f64 を文字列化する）
    fn format_number(&self, value: f64, _locale: &str, _options: &NumberFormatOptions) -> Result<String> {
        // スタブとして数値を文字列化して返す（ICU 実装では CLDR データを使用する）
        Ok(value.to_string())
    }

    // format_date スタブ実装（エポック ms をそのまま文字列化して返す）
    fn format_date(&self, timestamp_millis: i64, _locale: &str, _format: &DateFormat) -> Result<String> {
        // スタブとしてエポックミリ秒を文字列化して返す（ICU 実装では CLDR カレンダーを使用する）
        Ok(timestamp_millis.to_string())
    }

    // format_currency スタブ実装（通貨コードと金額を連結して返す）
    fn format_currency(&self, amount: f64, currency: &str, _locale: &str) -> Result<String> {
        // スタブとして "{currency} {amount}" 形式で返す
        Ok(format!("{currency} {amount}"))
    }

    // format_unit スタブ実装（単位と数値を連結して返す）
    fn format_unit(&self, value: f64, unit: &str, _locale: &str) -> Result<String> {
        // スタブとして "{value} {unit}" 形式で返す
        Ok(format!("{value} {unit}"))
    }
}
