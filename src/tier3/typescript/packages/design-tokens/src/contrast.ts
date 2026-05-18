// tier3 design-tokens コントラスト比計算
// WCAG 2.1 AA 基準 (4.5:1) の検証ロジック

// 相対輝度を計算するヘルパー関数 (sRGB 線形化)
function linearize(value: number): number {
  // sRGB 値を [0, 1] に正規化する
  const v = value / 255;
  // sRGB 線形化変換式を適用する (WCAG の計算式)
  return v <= 0.04045 ? v / 12.92 : Math.pow((v + 0.055) / 1.055, 2.4);
}

// RGB 値から相対輝度を計算する関数
export function relativeLuminance(r: number, g: number, b: number): number {
  // WCAG 2.1 の相対輝度計算式に従う (ITU-R BT.709)
  return 0.2126 * linearize(r) + 0.7152 * linearize(g) + 0.0722 * linearize(b);
}

// 2 色のコントラスト比を計算する関数
export function calculateContrastRatio(
  // 前景色 [R, G, B]
  foreground: [number, number, number],
  // 背景色 [R, G, B]
  background: [number, number, number],
): number {
  // それぞれの相対輝度を計算する
  const L1 = relativeLuminance(...foreground);
  // 背景色の相対輝度を計算する
  const L2 = relativeLuminance(...background);
  // 明るい方を分子にしてコントラスト比を計算する
  const [lighter, darker] = L1 > L2 ? [L1, L2] : [L2, L1];
  // WCAG 2.1 のコントラスト比計算式を適用する
  return (lighter + 0.05) / (darker + 0.05);
}

// WCAG AA 基準を満たすかチェックする関数
export function meetsWCAGAA(ratio: number): boolean {
  // WCAG AA は通常テキストで 4.5:1 以上が必要
  return ratio >= 4.5;
}
