// tier3 design-tokens WCAG AA コントラスト比テスト
// コントラスト比計算ロジックが正しく動作することを確認する

// vitest の test/expect をインポートする
import { test, expect } from 'vitest';
// コントラスト比計算関数と WCAG AA 検証関数をインポートする
import { calculateContrastRatio, meetsWCAGAA } from '../src/contrast.js';

// テスト: 黒 on 白のコントラスト比が 21:1 であることを確認する
test('black on white has maximum contrast ratio', () => {
  // 黒 [0,0,0] on 白 [255,255,255] のコントラスト比は 21:1
  const ratio = calculateContrastRatio([0, 0, 0], [255, 255, 255]);
  // 小数点以下の丸め誤差を許容して 21 に近いことを確認する
  expect(ratio).toBeCloseTo(21, 0);
});

// テスト: WCAG AA 基準 (4.5:1) を満たす場合は true を返すことを確認する
test('meetsWCAGAA returns true for ratio >= 4.5', () => {
  // 4.5:1 ちょうどの場合は基準を満たす
  expect(meetsWCAGAA(4.5)).toBe(true);
  // 21:1 の場合も基準を満たす
  expect(meetsWCAGAA(21)).toBe(true);
});

// テスト: WCAG AA 基準を満たさない場合は false を返すことを確認する
test('meetsWCAGAA returns false for ratio < 4.5', () => {
  // 3:1 の場合は基準を満たさない
  expect(meetsWCAGAA(3)).toBe(false);
  // 4.49 の場合は基準を満たさない
  expect(meetsWCAGAA(4.49)).toBe(false);
});

// テスト: 黒 on 白のコントラスト比が WCAG AA 基準を満たすことを確認する
test('black on white meets WCAG AA', () => {
  // 黒 on 白のコントラスト比を計算する
  const ratio = calculateContrastRatio([0, 0, 0], [255, 255, 255]);
  // WCAG AA 基準を満たすことをアサートする
  expect(meetsWCAGAA(ratio)).toBe(true);
});

// テスト: 白 on 白のコントラスト比は WCAG AA 基準を満たさないことを確認する
test('white on white does not meet WCAG AA', () => {
  // 白 on 白のコントラスト比を計算する
  const ratio = calculateContrastRatio([255, 255, 255], [255, 255, 255]);
  // WCAG AA 基準を満たさないことをアサートする
  expect(meetsWCAGAA(ratio)).toBe(false);
});
