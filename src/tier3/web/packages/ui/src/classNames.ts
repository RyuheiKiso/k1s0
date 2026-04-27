// classNames は条件付き class 文字列を組み立てる軽量ヘルパ。
//
// clsx / classnames 依存を避けるため独自実装。複雑なケース（オブジェクト記法等）が必要なら
// リリース時点 で clsx に置換する。

export type ClassValue = string | false | null | undefined;

// classNames(['a', cond && 'b']) のような呼出を許容する。
export function classNames(...values: ClassValue[]): string {
  return values.filter((v): v is string => typeof v === 'string' && v.length > 0).join(' ');
}
