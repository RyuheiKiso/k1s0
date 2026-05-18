/**
 * k1s0 tier2 i18n formatter TypeScript インターフェース定義
 * ICU 風の number / date / currency / unit formatter の TypeScript インターフェース
 * Rust 実装（rust/src/i18n_formatter.rs）と 4 言語等価強度を持つ TypeScript 版
 * wall clock TTL 禁止規約準拠: 日付は HLC タイムスタンプから変換した上で fmt へ渡す
 */

/**
 * NumberFormatOptions: 数値フォーマットオプション
 * ICU NumberFormat オプションに相当するオプション集合
 * Rust の NumberFormatOptions 構造体に対応する
 */
// NumberFormatOptions インターフェース定義
export interface NumberFormatOptions {
  // 小数点以下の最小桁数（指定なしは undefined）
  readonly minimumFractionDigits?: number;
  // 小数点以下の最大桁数（指定なしは undefined）
  readonly maximumFractionDigits?: number;
  // 千単位区切り文字を使用するか（省略時は true）
  readonly useGrouping?: boolean;
}

/**
 * DateFormatStyle: 日付フォーマットの種別（ICU の DateTimeStyle に対応する）
 * Rust の DateFormat enum に対応する
 */
// DateFormatStyle 型定義（文字列リテラル union）
export type DateFormatStyle =
  // 短形式（例: 2024/01/15）
  | "short"
  // 中形式（例: 2024年1月15日）
  | "medium"
  // 長形式（例: 2024年1月15日 月曜日）
  | "long"
  // カスタム書式文字列（ICU パターン形式）
  | "custom";

/**
 * ILocaleFormatter: ICU 風 number / date / currency / unit formatter の TypeScript インターフェース
 * Rust の LocaleFormatter トレイトに対応する
 * 4 言語等価原則により Rust / C# / Go / TypeScript で同じシグネチャを持つ
 * wall clock を直接 TTL に使うことを禁止し、HLC timestamp を経由する設計を維持する
 */
// ILocaleFormatter インターフェース定義
export interface ILocaleFormatter {
  /**
   * formatNumber: 数値をロケール形式にフォーマットする
   * value: フォーマット対象の数値
   * locale: IETF BCP 47 ロケールタグ（例: "ja-JP" / "en-US"）
   * options: フォーマットオプション（小数点桁数 / 区切り文字など）
   * フォーマットされた文字列を返す
   */
  // formatNumber メソッド（数値ロケールフォーマット操作）
  formatNumber(value: number, locale: string, options?: NumberFormatOptions): string;

  /**
   * formatDate: 日付をロケール形式にフォーマットする
   * timestampMillis: ミリ秒エポック形式の日付（HLC タイムスタンプから変換する）
   * locale: IETF BCP 47 ロケールタグ（例: "ja-JP" / "en-US"）
   * style: 日付フォーマット種別（"short" / "medium" / "long" / "custom"）
   * customPattern: style が "custom" の場合に使用する ICU パターン文字列
   * フォーマットされた文字列を返す
   */
  // formatDate メソッド（日付ロケールフォーマット操作）
  formatDate(timestampMillis: number, locale: string, style: DateFormatStyle, customPattern?: string): string;

  /**
   * formatCurrency: 通貨金額をロケール形式にフォーマットする
   * amount: フォーマット対象の金額
   * currency: ISO 4217 通貨コード（例: "JPY" / "USD" / "EUR"）
   * locale: IETF BCP 47 ロケールタグ（例: "ja-JP" / "en-US"）
   * フォーマットされた文字列を返す
   */
  // formatCurrency メソッド（通貨ロケールフォーマット操作）
  formatCurrency(amount: number, currency: string, locale: string): string;

  /**
   * formatUnit: 単位付き数値をロケール形式にフォーマットする
   * value: フォーマット対象の数値
   * unit: CLDR unit identifier（例: "kilogram" / "meter" / "liter"）
   * locale: IETF BCP 47 ロケールタグ（例: "ja-JP" / "en-US"）
   * フォーマットされた文字列を返す
   */
  // formatUnit メソッド（単位付き数値ロケールフォーマット操作）
  formatUnit(value: number, unit: string, locale: string): string;
}

/**
 * LocaleFormatterStub: テスト / stub 実装
 * 実際のロケール変換は行わず、引数をそのまま文字列化して返す
 * production / development 区別禁止規約に従い、本実装は必ず ICU backend を使用すること
 */
// LocaleFormatterStub クラス定義（ILocaleFormatter を実装する）
export class LocaleFormatterStub implements ILocaleFormatter {
  // formatNumber スタブ実装（ロケール変換なしで数値を文字列化する）
  formatNumber(value: number, _locale: string, _options?: NumberFormatOptions): string {
    // スタブとして数値を文字列化して返す（ICU 実装では CLDR データを使用する）
    return String(value);
  }

  // formatDate スタブ実装（エポック ms をそのまま文字列化して返す）
  formatDate(timestampMillis: number, _locale: string, _style: DateFormatStyle, _customPattern?: string): string {
    // スタブとしてエポックミリ秒を文字列化して返す（ICU 実装では CLDR カレンダーを使用する）
    return String(timestampMillis);
  }

  // formatCurrency スタブ実装（通貨コードと金額を連結して返す）
  formatCurrency(amount: number, currency: string, _locale: string): string {
    // スタブとして "{currency} {amount}" 形式で返す
    return `${currency} ${amount}`;
  }

  // formatUnit スタブ実装（単位と数値を連結して返す）
  formatUnit(value: number, unit: string, _locale: string): string {
    // スタブとして "{value} {unit}" 形式で返す
    return `${value} ${unit}`;
  }
}
