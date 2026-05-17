// k1s0 tier3 i18n ICU メッセージパーサー
// ICU Message Format の基本サブセット（{varName} 置換 + plural 対応）を実装する
// 参考: https://unicode-org.github.io/icu/userguide/format_parse/messages/
// 完全な ICU 実装は intl-messageformat ライブラリを推奨するが、
// ここでは外部依存を最小化するために基本サブセットのみ実装する

// 変数マップの型（変数名 → 文字列 or 数値）
export type ICUVars = Record<string, string | number>;

// plural ルールの型（ICU plural keyword → テンプレート文字列）
// ICU 仕様: "zero" / "one" / "two" / "few" / "many" / "other" をサポートする
type PluralRules = Record<string, string>;

// plural キーワードを数値から判定する（ja-JP / en-US の簡易実装）
// en-US: count === 1 → "one", それ以外 → "other"
// ja-JP: 全て "other"（日本語には複数形の文法的区別がない）
function getPluralKeyword(count: number, locale?: string): string {
    // ロケールが ja-JP の場合は全て "other" を返す（日本語は複数形なし）
    if (locale === "ja-JP" || locale === "ja") {
        // 日本語では複数形の区別を行わない
        return "other";
    }
    // en-US: count が 1 の場合は "one"、それ以外は "other"
    if (count === 1) {
        // 英語の単数形
        return "one";
    }
    // 英語の複数形（デフォルト）
    return "other";
}

// plural ルールブロック内の "#" を実際の数値に置換する
// ICU 仕様: plural テンプレート内の "#" は現在の数値に展開される
function replacePoundSign(template: string, count: number): string {
    // "#" を実際の数値に置換する（全箇所置換）
    return template.replace(/#/g, String(count));
}

// plural ルール文字列をパースして PluralRules オブジェクトに変換する
// 入力例: "one {#件} other {#件}"
// 出力例: { "one": "#件", "other": "#件" }
function parsePluralRules(rulesStr: string): PluralRules {
    // plural ルールの結果マップを初期化する
    const rules: PluralRules = {};
    // 正規表現: "keyword {template}" のパターンを抽出する
    // キーワード: zero / one / two / few / many / other（ICU 標準）
    const rulePattern = /\b(zero|one|two|few|many|other)\s*\{([^}]*)\}/g;
    // 全マッチを処理する
    let match: RegExpExecArray | null;
    // exec でイテレーションして全ルールを抽出する
    while ((match = rulePattern.exec(rulesStr)) !== null) {
        // キーワードとテンプレートを抽出する
        const keyword = match[1];
        // テンプレート文字列（前後の空白を除去する）
        const template = match[2].trim();
        // ルールマップに追加する
        rules[keyword] = template;
    }
    // パース結果を返す
    return rules;
}

// ICU の plural 構文を解析して展開する
// 構文: "{varName, plural, one {#件} other {#件}}"
// locale: plural キーワード判定に使用するロケール識別子（省略時は en-US ルールを使用）
function expandPlural(varName: string, rulesStr: string, vars: ICUVars, locale?: string): string {
    // 変数値を取得する（数値型に変換する）
    const rawValue = vars[varName];
    // 変数が未定義の場合は 0 として扱う
    const count = typeof rawValue === "number" ? rawValue : Number(rawValue ?? 0);
    // plural ルールをパースする
    const rules = parsePluralRules(rulesStr);
    // 数値に対応する plural キーワードを取得する
    const keyword = getPluralKeyword(count, locale);
    // キーワードに対応するテンプレートを選択する（なければ "other" にフォールバック）
    const template = rules[keyword] ?? rules["other"] ?? String(count);
    // "#" を実際の数値に置換して返す
    return replacePoundSign(template, count);
}

// ICU メッセージテンプレートをパースして変数を展開する
// 対応構文:
//   1. {varName}            — 変数展開（文字列 / 数値）
//   2. {varName, plural, one {#件} other {#件}} — 複数形展開
// template: ICU 形式のテンプレート文字列
// vars: 変数マップ（変数名 → 値）
// locale: plural 判定に使用するロケール識別子（省略可能）
export function parseICUMessage(template: string, vars: ICUVars, locale?: string): string {
    // plural 構文の正規表現パターン（{varName, plural, ...} にマッチする）
    // 非貪欲マッチで最小のブロックを抽出する
    const pluralPattern = /\{(\w+)\s*,\s*plural\s*,\s*((?:[^{}]|\{[^{}]*\})*)\}/g;
    // まず plural 構文を処理する（内側に {} を含むため先に処理する）
    let result = template.replace(pluralPattern, (_, varName: string, rulesStr: string) => {
        // plural 構文を展開して返す
        return expandPlural(varName, rulesStr, vars, locale);
    });
    // 次に単純な変数展開 {varName} を処理する（plural 処理後に実施する）
    const varPattern = /\{(\w+)\}/g;
    // 全変数プレースホルダーを実際の値に置換する
    result = result.replace(varPattern, (_, varName: string) => {
        // 変数が定義されている場合は値を、未定義の場合はプレースホルダーをそのまま返す
        const value = vars[varName];
        // 未定義変数はプレースホルダー文字列として残す（デバッグを容易にする）
        return value !== undefined ? String(value) : `{${varName}}`;
    });
    // 展開済みメッセージを返す
    return result;
}
