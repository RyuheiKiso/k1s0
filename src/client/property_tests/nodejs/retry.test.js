// tier1 client SDK retry backoff property テスト (Node.js)
// 各行の上に日本語コメントを記載すること

// Node.js 組み込みアサートモジュールをインポートする
const assert = require('assert');

// exponential backoff の計算関数 (テスト対象)
function calculateBackoff(retryCount, initialDelayMs, multiplier, maxDelayMs) {
  // リトライ回数に応じて待機時間を指数的に増加させる
  const delay = initialDelayMs * Math.pow(multiplier, retryCount);
  // 最大待機時間を超えないようにクランプする
  return Math.min(delay, maxDelayMs);
}

// テスト: 最初のリトライ待機時間が initialDelay であることを確認する
function testFirstRetryDelay() {
  // リトライ回数 0 での待機時間は initialDelayMs と等しい
  const delay = calculateBackoff(0, 100, 2.0, 10000);
  // 期待値と実測値を比較する
  assert.strictEqual(delay, 100, 'First retry should use initial delay');
  // テスト通過を報告する
  console.log('PASS: testFirstRetryDelay');
}

// テスト: 2 回目のリトライ待機時間が 2 倍になることを確認する
function testSecondRetryDelay() {
  // 2 倍のバックオフで 2 回目は 200ms になる
  const delay = calculateBackoff(1, 100, 2.0, 10000);
  // 期待値と実測値を比較する
  assert.strictEqual(delay, 200, 'Second retry should double the delay');
  // テスト通過を報告する
  console.log('PASS: testSecondRetryDelay');
}

// テスト: maxDelay を超えないことを確認する
function testMaxDelayClamp() {
  // 多数リトライしても maxDelayMs を超えない
  const delay = calculateBackoff(100, 100, 2.0, 10000);
  // 期待値と実測値を比較する
  assert.strictEqual(delay, 10000, 'Delay should be capped at maxDelayMs');
  // テスト通過を報告する
  console.log('PASS: testMaxDelayClamp');
}

// 全テストを実行する
testFirstRetryDelay();
// 2 番目のテストを実行する
testSecondRetryDelay();
// 3 番目のテストを実行する
testMaxDelayClamp();
// 全テスト通過を報告する
console.log('All property tests passed');
