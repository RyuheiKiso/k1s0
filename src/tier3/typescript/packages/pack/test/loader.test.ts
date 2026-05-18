// tier3 pack loader テスト
// pack のロードロジックが正常に動作することを確認する

// vitest の test/expect/beforeEach をインポートする
import { test, expect, beforeEach } from 'vitest';
// loadPackConfig / isValidPackId / clearPackConfigCache / isScreenEnabled をインポートする
import { loadPackConfig, isValidPackId, clearPackConfigCache, isScreenEnabled } from '../src/loader.js';

// 各テスト前にキャッシュをクリアする
beforeEach(() => {
  // テスト間でキャッシュが共有されないようにクリアする
  clearPackConfigCache();
});

// テスト: manufacturing pack の設定がロードされることを確認する
test('loadPackConfig returns config for manufacturing', () => {
  // manufacturing pack をロードする
  const config = loadPackConfig('manufacturing');
  // pack ID が manufacturing であることを確認する
  expect(config.packId).toBe('manufacturing');
  // バージョンが存在することを確認する
  expect(config.version).toBeTruthy();
});

// テスト: retail pack の設定がロードされることを確認する
test('loadPackConfig returns config for retail', () => {
  // retail pack をロードする
  const config = loadPackConfig('retail');
  // pack ID が retail であることを確認する
  expect(config.packId).toBe('retail');
});

// テスト: logistics pack の設定がロードされることを確認する
test('loadPackConfig returns config for logistics', () => {
  // logistics pack をロードする
  const config = loadPackConfig('logistics');
  // pack ID が logistics であることを確認する
  expect(config.packId).toBe('logistics');
});

// テスト: isValidPackId が有効な pack ID に対して true を返すことを確認する
test('isValidPackId returns true for valid pack ids', () => {
  // manufacturing は有効な pack ID である
  expect(isValidPackId('manufacturing')).toBe(true);
  // retail は有効な pack ID である
  expect(isValidPackId('retail')).toBe(true);
  // logistics は有効な pack ID である
  expect(isValidPackId('logistics')).toBe(true);
});

// テスト: isValidPackId が無効な pack ID に対して false を返すことを確認する
test('isValidPackId returns false for invalid pack ids', () => {
  // unknown_pack は無効な pack ID である
  expect(isValidPackId('unknown_pack')).toBe(false);
  // 空文字列は無効な pack ID である
  expect(isValidPackId('')).toBe(false);
});

// テスト: manufacturing pack で plant 画面が有効であることを確認する
test('isScreenEnabled returns true for enabled screen', () => {
  // manufacturing の plant 画面は有効である
  expect(isScreenEnabled('manufacturing', 'plant')).toBe(true);
});

// テスト: manufacturing pack で存在しない画面は無効であることを確認する
test('isScreenEnabled returns false for unknown screen', () => {
  // manufacturing に存在しない画面は無効である
  expect(isScreenEnabled('manufacturing', 'nonexistent_screen')).toBe(false);
});
