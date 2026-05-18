// k1s0 tier3 pseudolocalization（疑似ローカライズ）ユーティリティ
// 翻訳文字列の幅溢れ検出 / 未翻訳テキスト検出に使用する
// ASCII 文字を類似の accented pseudo 文字に変換し、文字列長を 30% 伸長する

// --------- 文字変換マップ ---------

// ASCII 文字から pseudo accented 文字へのマッピングテーブル
// CLDR pseudolocalization 標準に準じた文字を選択する
const PSEUDO_CHAR_MAP: Readonly<Record<string, string>> = {
  // a → ä（ウムラウト a）
  a: "ä",
  // b → ƀ（ストライクスルー b）
  b: "ƀ",
  // c → ç（セディーユ c）
  c: "ç",
  // d → ð（エス-zed d）
  d: "ð",
  // e → è（グレーブ e）
  e: "è",
  // f → ƒ（フック f）
  f: "ƒ",
  // g → ĝ（サーカムフレックス g）
  g: "ĝ",
  // h → ħ（ストライクスルー h）
  h: "ħ",
  // i → î（サーカムフレックス i）
  i: "î",
  // j → ĵ（サーカムフレックス j）
  j: "ĵ",
  // k → ķ（セディーユ k）
  k: "ķ",
  // l → ļ（セディーユ l）
  l: "ļ",
  // m → m̃（チルダ m）
  m: "m̃",
  // n → ñ（チルダ n）
  n: "ñ",
  // o → ö（ウムラウト o）
  o: "ö",
  // p → þ（Thorn p）
  p: "þ",
  // q → q̈（ウムラウト q）
  q: "q̈",
  // r → ŗ（セディーユ r）
  r: "ŗ",
  // s → š（カロン s）
  s: "š",
  // t → ţ（セディーユ t）
  t: "ţ",
  // u → ü（ウムラウト u）
  u: "ü",
  // v → v̈（ウムラウト v）
  v: "v̈",
  // w → ŵ（サーカムフレックス w）
  w: "ŵ",
  // x → x̃（チルダ x）
  x: "x̃",
  // y → ŷ（サーカムフレックス y）
  y: "ŷ",
  // z → ž（カロン z）
  z: "ž",
  // A → Ä（ウムラウト A）
  A: "Ä",
  // B → Ƀ（ストライクスルー B）
  B: "Ƀ",
  // C → Ç（セディーユ C）
  C: "Ç",
  // D → Ð（エス-zed D）
  D: "Ð",
  // E → É（アキュート E）
  E: "É",
  // F → F̃（チルダ F）
  F: "F̃",
  // G → Ĝ（サーカムフレックス G）
  G: "Ĝ",
  // H → Ħ（ストライクスルー H）
  H: "Ħ",
  // I → Î（サーカムフレックス I）
  I: "Î",
  // J → Ĵ（サーカムフレックス J）
  J: "Ĵ",
  // K → Ķ（セディーユ K）
  K: "Ķ",
  // L → Ļ（セディーユ L）
  L: "Ļ",
  // M → M̃（チルダ M）
  M: "M̃",
  // N → Ñ（チルダ N）
  N: "Ñ",
  // O → Ö（ウムラウト O）
  O: "Ö",
  // P → Þ（Thorn P）
  P: "Þ",
  // Q → Q̈（ウムラウト Q）
  Q: "Q̈",
  // R → Ŗ（セディーユ R）
  R: "Ŗ",
  // S → Š（カロン S）
  S: "Š",
  // T → Ţ（セディーユ T）
  T: "Ţ",
  // U → Ü（ウムラウト U）
  U: "Ü",
  // V → V̈（ウムラウト V）
  V: "V̈",
  // W → Ŵ（サーカムフレックス W）
  W: "Ŵ",
  // X → X̃（チルダ X）
  X: "X̃",
  // Y → Ŷ（サーカムフレックス Y）
  Y: "Ŷ",
  // Z → Ž（カロン Z）
  Z: "Ž",
};

// --------- pseudolocalize 関数 ---------

// pseudoloc オプション型
export interface PseudolocOptions {
  // 文字列長の伸長倍率（デフォルト: 1.3 = 30% 伸長）
  readonly expansionRatio?: number;
  // ブラケットで囲む（デフォルト: true、視覚的に擬似ロケールと識別しやすくする）
  readonly bracket?: boolean;
}

// ASCII 文字を pseudo accented 文字に変換する（1 文字変換）
function convertChar(char: string): string {
  // マッピングテーブルに存在する文字はマップ後の文字を返す
  return PSEUDO_CHAR_MAP[char] ?? char;
}

// 文字列を pseudolocalize する
// 1. ASCII 文字を accented pseudo 文字に変換する
// 2. 文字列長を expansionRatio 倍に伸長する（幅溢れ検出のため）
// 3. オプションでブラケット「[...]」で囲む
export function pseudolocalize(
  input: string,
  options: PseudolocOptions = {},
): string {
  // デフォルトの伸長倍率（1.3 = 30% 伸長）を設定する
  const expansionRatio = options.expansionRatio ?? 1.3;
  // デフォルトのブラケット設定（true = ブラケットで囲む）を設定する
  const bracket = options.bracket ?? true;

  // 入力が空文字の場合はそのまま返す
  if (input.length === 0) {
    // 空文字はそのまま返す
    return input;
  }

  // 各文字を pseudo accented 文字に変換する
  const converted = Array.from(input)
    .map(convertChar)
    .join("");

  // 伸長後の目標文字数を計算する
  const targetLength = Math.ceil(input.length * expansionRatio);
  // 伸長に必要な追加文字数を計算する
  const paddingNeeded = Math.max(0, targetLength - converted.length);

  // 伸長用のパディング文字列を生成する（"xxx..." で埋める）
  const padding = "x".repeat(paddingNeeded);

  // 変換済み文字列にパディングを追加する
  const expanded = converted + padding;

  // ブラケットで囲む場合は "[...]" を付加する
  if (bracket) {
    // ブラケットで囲んで返す
    return `[${expanded}]`;
  }

  // ブラケットなしで返す
  return expanded;
}

// --------- 翻訳辞書の一括 pseudolocalize ---------

// 翻訳辞書（Record<string, string>）の全値を pseudolocalize して返す
export function pseudolocalizeDict(
  dict: Readonly<Record<string, string>>,
  options: PseudolocOptions = {},
): Record<string, string> {
  // 空オブジェクトから始める
  const result: Record<string, string> = {};
  // 各 key の値を pseudolocalize する
  for (const [key, value] of Object.entries(dict)) {
    // 変換した値を格納する
    result[key] = pseudolocalize(value, options);
  }
  // 変換済み辞書を返す
  return result;
}

// --------- 幅溢れ検出ユーティリティ ---------

// テキスト幅を推定する（DOM 非依存の簡易実装）
// 正確な幅は Canvas measureText で計算するが、pseudoloc では文字数を代替指標とする
export function estimateTextWidth(text: string): number {
  // Unicode 文字列の文字数を返す（サロゲートペアを考慮して Array.from を使用する）
  return Array.from(text).length;
}

// pseudolocalize した結果が元の文字列より伸長されているか確認する
export function isExpanded(
  original: string,
  pseudolocalized: string,
  expectedRatio: number = 1.3,
): boolean {
  // 元の文字数を取得する
  const originalLen = estimateTextWidth(original);
  // pseudolocalize 後の文字数を取得する（ブラケット "[]" の 2 文字を除く）
  const pseudoLen = estimateTextWidth(pseudolocalized) - 2;
  // 伸長比率が期待値以上かどうかを確認する
  return pseudoLen >= Math.ceil(originalLen * expectedRatio);
}
