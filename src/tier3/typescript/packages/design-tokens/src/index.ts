// src/tier3/typescript/packages/design-tokens/src/index.ts
// design-tokens パッケージの公開 API

// カラートークン・タイポグラフィ・スペーシング・アニメーショントークンをエクスポートする
export { COLOR_TOKENS, TYPOGRAPHY_TOKENS, SPACING_TOKENS, ANIMATION_TOKENS } from "./tokens.js";
// コントラスト比計算と WCAG AA 基準検証関数をエクスポートする
export { calculateContrastRatio, meetsWCAGAA } from "./contrast.js";
