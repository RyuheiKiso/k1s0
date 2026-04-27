// @k1s0/i18n エントリポイント。
//
// 軽量 i18n: 翻訳キー → ロケール固有文字列を解決する最小実装。
// i18next の重依存を避け、format 補間（{name} 等の差し替え）のみ提供する。
// 翻訳カバレッジが拡大したら i18next-react に置換する（リリース時点）。

import jaLocale from './locales/ja.json';
import enLocale from './locales/en.json';

// 対応するロケール一覧（追加時はここに登録する）。
export type Locale = 'ja' | 'en';

// 翻訳辞書の型（key -> string の単純マップ）。
export type TranslationDict = Record<string, string>;

// すべてのロケールを束ねた map。
const translations: Record<Locale, TranslationDict> = {
  ja: jaLocale,
  en: enLocale,
};

// I18nClient はロケール固定で翻訳機能を提供する。
export interface I18nClient {
  // 現在のロケール。
  readonly locale: Locale;
  // キーを解決する。未存在なら key 自体を返す（debug fallback）。
  t(key: string, vars?: Record<string, string | number>): string;
}

// 与えられたロケールに固定した I18nClient を返す。
export function createI18n(locale: Locale): I18nClient {
  // ロケール辞書を引く（未対応は ja フォールバック）。
  const dict: TranslationDict = translations[locale] ?? translations.ja;
  // 翻訳関数。
  const t = (key: string, vars?: Record<string, string | number>): string => {
    // dict から値を取り、未存在なら key 自体を返す。
    const raw = dict[key] ?? key;
    // vars がなければ raw をそのまま返す。
    if (!vars) {
      return raw;
    }
    // {name} スタイルの placeholder を素朴に置換する。
    return Object.entries(vars).reduce<string>(
      (acc, [k, v]) => acc.split(`{${k}}`).join(String(v)),
      raw,
    );
  };
  return { locale, t };
}

// ロケール一覧を返すユーティリティ（UI のロケール選択に使う）。
export function availableLocales(): Locale[] {
  return Object.keys(translations) as Locale[];
}
