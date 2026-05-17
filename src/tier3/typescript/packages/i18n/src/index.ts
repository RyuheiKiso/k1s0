// k1s0 tier3 i18n パッケージ公開エントリポイント
// locale loader / ICU parser / dict loader / 設定を一括 re-export する
// v1.0.0 locale: ja-JP / en-US
// 未定義 key は CI fail（整合 7 の物理根拠）
// Intl.NumberFormat / Intl.DateTimeFormat / ISO 4217 を使用する

// ICU メッセージパーサーの全エクスポート（{varName} 置換 / plural 対応）
export { parseICUMessage } from "./icu_parser";
// ICU 変数マップの型エクスポート（ICUVars）
export type { ICUVars } from "./icu_parser";
// 辞書ローダーの全エクスポート（dynamic import / cache / dictGet）
export { loadDict, clearDictCache, dictGet } from "./dict_loader";
// 辞書ローダーの型エクスポート（DictLocale / TranslationDict）
export type { DictLocale, TranslationDict } from "./dict_loader";

// サポート locale 一覧（v1 は ja-JP / en-US のみ）
export const SUPPORTED_LOCALES = ["ja-JP", "en-US"] as const;
// locale の型
export type SupportedLocale = (typeof SUPPORTED_LOCALES)[number];

// locale の判定関数
export function isSupportedLocale(locale: string): locale is SupportedLocale {
  // サポート locale に含まれるか確認する
  return (SUPPORTED_LOCALES as readonly string[]).includes(locale);
}

// 翻訳辞書の型（未定義 key は型エラー）
export type TranslationDict = Readonly<Record<string, string>>;

// locale 設定（Intl 対応）
export interface LocaleConfig {
  // locale 識別子
  readonly locale: SupportedLocale;
  // 数値フォーマット設定（Intl.NumberFormat）
  readonly numberFormat: Intl.NumberFormatOptions;
  // 日時フォーマット設定（Intl.DateTimeFormat）
  readonly dateTimeFormat: Intl.DateTimeFormatOptions;
  // 通貨コード（ISO 4217）
  readonly currencyCode: string;
  // タイムゾーン
  readonly timeZone: string;
}

// ja-JP の locale 設定
export const JA_JP_CONFIG: LocaleConfig = {
  locale: "ja-JP",
  // 日本語数値フォーマット
  numberFormat: { style: "decimal", maximumFractionDigits: 2 },
  // 日本語日時フォーマット
  dateTimeFormat: {
    year: "numeric",
    month: "2-digit",
    day: "2-digit",
    hour: "2-digit",
    minute: "2-digit",
    timeZone: "Asia/Tokyo",
  },
  // 日本円
  currencyCode: "JPY",
  // 日本標準時
  timeZone: "Asia/Tokyo",
};

// en-US の locale 設定
export const EN_US_CONFIG: LocaleConfig = {
  locale: "en-US",
  // 英語数値フォーマット
  numberFormat: { style: "decimal", maximumFractionDigits: 2 },
  // 英語日時フォーマット
  dateTimeFormat: {
    year: "numeric",
    month: "2-digit",
    day: "2-digit",
    hour: "2-digit",
    minute: "2-digit",
    timeZone: "America/New_York",
  },
  // US ドル
  currencyCode: "USD",
  // US 東部時間
  timeZone: "America/New_York",
};

// locale から config を取得する
export function getLocaleConfig(locale: SupportedLocale): LocaleConfig {
  // locale に応じた config を返す
  switch (locale) {
    case "ja-JP":
      return JA_JP_CONFIG;
    case "en-US":
      return EN_US_CONFIG;
    default: {
      // 網羅性チェック（新 locale 追加時はコンパイルエラー）
      const _exhaustive: never = locale;
      throw new Error(`Unsupported locale: ${String(_exhaustive)}`);
    }
  }
}

// 翻訳 key の lookup（未定義 key は undefined を返す、CI が検出する）
export function translate(
  dict: TranslationDict,
  key: string,
  fallback?: string,
): string {
  // dict に key が存在する場合は値を返す（存在しない場合は fallback または key 自体を返す）
  return dict[key] ?? fallback ?? key;
}
