// k1s0 tier2 i18n formatter C# (.NET 8+) インターフェース定義
// ICU 風の number / date / currency / unit formatter の C# インターフェース
// Rust 実装（rust/src/i18n_formatter.rs）と 4 言語等価強度を持つ C# 版
// wall clock TTL 禁止規約準拠: 日付は HLC タイムスタンプから変換した上で fmt へ渡す

// System.DateTime 型（日付フォーマット引数に使用する）
using System;
// System.Threading.Tasks: 非同期処理に使用する（将来の非同期ロケールデータ読み込み対応）
using System.Threading.Tasks;

// k1s0 tier2 名前空間
namespace K1s0.Tier2;

/// <summary>
/// NumberFormatOptions: 数値フォーマットオプション
/// ICU NumberFormat オプションに相当するオプション集合
/// Rust の NumberFormatOptions 構造体に対応する
/// </summary>
public sealed record NumberFormatOptions
{
    // 小数点以下の最小桁数（指定なしは null）
    public int? MinimumFractionDigits { get; init; }
    // 小数点以下の最大桁数（指定なしは null）
    public int? MaximumFractionDigits { get; init; }
    // 千単位区切り文字を使用するか
    public bool UseGrouping { get; init; } = true;
}

/// <summary>
/// DateFormatStyle: 日付フォーマットの種別（ICU の DateTimeStyle に対応する）
/// Rust の DateFormat enum に対応する
/// </summary>
public enum DateFormatStyle
{
    /// <summary>短形式（例: 2024/01/15）</summary>
    Short,
    /// <summary>中形式（例: 2024年1月15日）</summary>
    Medium,
    /// <summary>長形式（例: 2024年1月15日 月曜日）</summary>
    Long,
    /// <summary>カスタム書式文字列（ICU パターン形式）</summary>
    Custom,
}

/// <summary>
/// ILocaleFormatter: ICU 風 number / date / currency / unit formatter の C# インターフェース
/// Rust の LocaleFormatter トレイトに対応する
/// 4 言語等価原則により Rust / C# / Go / TypeScript で同じシグネチャを持つ
/// wall clock を直接 TTL に使うことを禁止し、HLC timestamp を経由する設計を維持する
/// </summary>
public interface ILocaleFormatter
{
    /// <summary>
    /// FormatNumber: 数値をロケール形式にフォーマットする
    /// value: フォーマット対象の数値（double）
    /// locale: IETF BCP 47 ロケールタグ（例: "ja-JP" / "en-US"）
    /// options: フォーマットオプション（小数点桁数 / 区切り文字など）
    /// フォーマットされた文字列を返す
    /// </summary>
    // FormatNumber メソッド（数値ロケールフォーマット操作）
    string FormatNumber(double value, string locale, NumberFormatOptions? options = null);

    /// <summary>
    /// FormatDate: 日付をロケール形式にフォーマットする
    /// timestampMillis: ミリ秒エポック形式の日付（HLC タイムスタンプから変換する）
    /// locale: IETF BCP 47 ロケールタグ（例: "ja-JP" / "en-US"）
    /// style: 日付フォーマット種別（Short / Medium / Long / Custom）
    /// customPattern: style が Custom の場合に使用する ICU パターン文字列
    /// フォーマットされた文字列を返す
    /// </summary>
    // FormatDate メソッド（日付ロケールフォーマット操作）
    string FormatDate(long timestampMillis, string locale, DateFormatStyle style, string? customPattern = null);

    /// <summary>
    /// FormatCurrency: 通貨金額をロケール形式にフォーマットする
    /// amount: フォーマット対象の金額（double）
    /// currency: ISO 4217 通貨コード（例: "JPY" / "USD" / "EUR"）
    /// locale: IETF BCP 47 ロケールタグ（例: "ja-JP" / "en-US"）
    /// フォーマットされた文字列を返す
    /// </summary>
    // FormatCurrency メソッド（通貨ロケールフォーマット操作）
    string FormatCurrency(double amount, string currency, string locale);

    /// <summary>
    /// FormatUnit: 単位付き数値をロケール形式にフォーマットする
    /// value: フォーマット対象の数値（double）
    /// unit: CLDR unit identifier（例: "kilogram" / "meter" / "liter"）
    /// locale: IETF BCP 47 ロケールタグ（例: "ja-JP" / "en-US"）
    /// フォーマットされた文字列を返す
    /// </summary>
    // FormatUnit メソッド（単位付き数値ロケールフォーマット操作）
    string FormatUnit(double value, string unit, string locale);
}
