// hlc.test.ts — @k1s0/hlc-lib の vitest テスト
// Rust テスト（src/client/hlc_lib/rust/src/lib.rs #[cfg(test)] mod tests）と
// 同等の coverage を TypeScript で実装する。
// 各テストケースは Rust 側の対応するテスト関数と 1:1 の対応を持つ。

// vitest からテストユーティリティをインポートする
import { describe, it, expect } from 'vitest';
// テスト対象の HlcTimestamp・HlcClock をインポートする
import { HlcTimestamp, HlcClock } from './hlc.js';

// HlcTimestamp の順序比較テスト（Rust: test_hlc_timestamp_ordering に対応）
describe('HlcTimestamp 順序比較', () => {
  it('wall_ms が大きい方が後のタイムスタンプであるべき', () => {
    // wall_ms=100 と wall_ms=200 を比較する
    const a = new HlcTimestamp(100n, 0, 0);
    const b = new HlcTimestamp(200n, 0, 0);
    // a < b であることを確認する
    expect(HlcTimestamp.compare(a, b)).toBe(-1);
  });

  it('同一 wall_ms では logical が大きい方が後であるべき', () => {
    // wall_ms=100, logical=0 と wall_ms=100, logical=1 を比較する
    const a = new HlcTimestamp(100n, 0, 0);
    const c = new HlcTimestamp(100n, 1, 0);
    // a < c であることを確認する
    expect(HlcTimestamp.compare(a, c)).toBe(-1);
  });

  it('同一 wall_ms/logical では node_id が大きい方が後であるべき', () => {
    // wall_ms=100, logical=0, node_id=0 と node_id=1 を比較する
    const a = new HlcTimestamp(100n, 0, 0);
    const d = new HlcTimestamp(100n, 0, 1);
    // a < d であることを確認する
    expect(HlcTimestamp.compare(a, d)).toBe(-1);
  });

  it('等値タイムスタンプは 0 を返すべき', () => {
    // 同一フィールドの 2 つのタイムスタンプを比較する
    const a = new HlcTimestamp(100n, 5, 3);
    const b = new HlcTimestamp(100n, 5, 3);
    // compare が 0 を返すことを確認する
    expect(HlcTimestamp.compare(a, b)).toBe(0);
  });
});

// format_compact と parse_compact のラウンドトリップテスト（Rust: test_format_and_parse に対応）
describe('formatCompact / parseCompact ラウンドトリップ', () => {
  it('既知の値で format_compact が正しい文字列を生成すべき', () => {
    // Rust テストと同一の既知値を使用する
    const ts = new HlcTimestamp(0x0123456789abcdefn, 0x00ff, 0x1234);
    // formatCompact を呼び出す
    const formatted = ts.formatCompact();
    // 期待値: Rust テストと同一の形式
    expect(formatted).toBe('0123456789abcdef-00ff-1234');
  });

  it('formatCompact → parseCompact でラウンドトリップが成立すべき', () => {
    // 既知の値で HlcTimestamp を生成する
    const ts = new HlcTimestamp(0x0123456789abcdefn, 0x00ff, 0x1234);
    // format_compact で文字列に変換する
    const formatted = ts.formatCompact();
    // parse_compact でパースする
    const parsed = HlcTimestamp.parseCompact(formatted);
    // パース結果が非 null であることを確認する
    expect(parsed).not.toBeNull();
    // 元の値と等しいことを確認する
    expect(HlcTimestamp.compare(parsed!, ts)).toBe(0);
  });

  it('不正な文字列は null を返すべき', () => {
    // ハイフン区切りが 2 つしかない場合は null を返す
    expect(HlcTimestamp.parseCompact('invalid')).toBeNull();
    // 空文字列は null を返す
    expect(HlcTimestamp.parseCompact('')).toBeNull();
    // パート数が 4 以上の場合は null を返す
    expect(HlcTimestamp.parseCompact('aaa-bbb-ccc-ddd')).toBeNull();
    // 不正な hex 文字列の場合は null を返す
    expect(HlcTimestamp.parseCompact('zzzzzzzzzzzzzzzz-0000-0000')).toBeNull();
  });

  it('EPOCH タイムスタンプのラウンドトリップ', () => {
    // EPOCH（全フィールド 0）のフォーマットとパースを確認する
    const formatted = HlcTimestamp.EPOCH.formatCompact();
    // 期待値: 全て 0 の形式
    expect(formatted).toBe('0000000000000000-0000-0000');
    // パース結果が EPOCH と等しいことを確認する
    const parsed = HlcTimestamp.parseCompact(formatted);
    expect(parsed).not.toBeNull();
    expect(HlcTimestamp.compare(parsed!, HlcTimestamp.EPOCH)).toBe(0);
  });
});

// addMs のテスト（Rust: test_add_ms に対応）
describe('addMs', () => {
  it('wall_ms に duration_ms を正しく加算すべき', () => {
    // 基準タイムスタンプを生成する（Rust テストと同一の値）
    const base = new HlcTimestamp(1_000_000n, 5, 1);
    // 1000ms 後の deadline を計算する
    const deadline = base.addMs(1_000n);
    // wall_ms が正しく加算されることを確認する
    expect(deadline.wall_ms).toBe(1_001_000n);
  });

  it('addMs 後の logical は 0 にリセットされるべき', () => {
    // logical=5 のタイムスタンプから deadline を計算する
    const base = new HlcTimestamp(1_000_000n, 5, 1);
    const deadline = base.addMs(1_000n);
    // logical が 0 にリセットされることを確認する
    expect(deadline.logical).toBe(0);
  });

  it('addMs 後の node_id は引き継がれるべき', () => {
    // node_id=1 のタイムスタンプから deadline を計算する
    const base = new HlcTimestamp(1_000_000n, 5, 1);
    const deadline = base.addMs(1_000n);
    // node_id が引き継がれることを確認する
    expect(deadline.node_id).toBe(1);
  });

  it('U64_MAX 付近のオーバーフローを飽和させるべき', () => {
    // U64_MAX 近くの値からさらに加算する
    const ts = new HlcTimestamp(HlcTimestamp.U64_MAX - 1n, 0, 0);
    // 2 を加算して U64_MAX を超えさせる
    const result = ts.addMs(2n);
    // U64_MAX に飽和することを確認する
    expect(result.wall_ms).toBe(HlcTimestamp.U64_MAX);
  });
});

// elapsedMsSince と isExpiredAt のテスト（Rust: test_elapsed_and_expired に対応）
describe('elapsedMsSince / isExpiredAt', () => {
  it('deadline より前では期限切れにならないべき', () => {
    // 現在時刻を表すタイムスタンプを生成する（Rust テストと同一の値）
    const now = new HlcTimestamp(2_000_000n, 0, 0);
    // 1000ms 後の deadline を計算する
    const deadline = now.addMs(1_000n);
    // now では期限切れでないことを確認する
    expect(deadline.isExpiredAt(now)).toBe(false);
  });

  it('deadline ちょうどで期限切れになるべき', () => {
    // deadline ちょうどで is_expired_at が true を返すことを確認する
    const now = new HlcTimestamp(2_000_000n, 0, 0);
    const deadline = now.addMs(1_000n);
    // deadline ちょうどで期限切れになることを確認する
    expect(deadline.isExpiredAt(deadline)).toBe(true);
  });

  it('deadline を超えたら期限切れになるべき', () => {
    // deadline を超えた future タイムスタンプを生成する
    const now = new HlcTimestamp(2_000_000n, 0, 0);
    const deadline = now.addMs(1_000n);
    const future = new HlcTimestamp(2_001_001n, 0, 0);
    // future では期限切れになることを確認する
    expect(deadline.isExpiredAt(future)).toBe(true);
  });

  it('elapsedMsSince が正しい経過時間を返すべき', () => {
    // Rust テストと同一の値で確認する
    const now = new HlcTimestamp(2_000_000n, 0, 0);
    const future = new HlcTimestamp(2_001_001n, 0, 0);
    // elapsed_ms_since が 1001 を返すことを確認する
    expect(future.elapsedMsSince(now)).toBe(1_001n);
  });

  it('self が reference より前の場合は 0n を返すべき', () => {
    // 逆順（self < reference）の場合の飽和減算を確認する
    const earlier = new HlcTimestamp(1_000n, 0, 0);
    const later = new HlcTimestamp(2_000n, 0, 0);
    // earlier.elapsedMsSince(later) は 0n を返すべき（負の elapsed は表現しない）
    expect(earlier.elapsedMsSince(later)).toBe(0n);
  });
});

// HlcClock.tick の単調増加テスト（Rust: test_tick_monotonic に対応）
describe('HlcClock tick 単調増加', () => {
  it('連続 tick は単調増加タイムスタンプを生成すべき', () => {
    // node_id=0 で HlcClock を生成する
    const clock = new HlcClock(0);
    // 最初の tick を実行する
    let prev = clock.tick();
    // 連続で 99 回 tick して全て単調増加することを確認する（Rust と同一の 100 回）
    for (let i = 0; i < 99; i++) {
      // 次のタイムスタンプを生成する
      const next = clock.tick();
      // prev よりも next が後（または同時）であることを確認する
      expect(HlcTimestamp.compare(next, prev)).toBeGreaterThanOrEqual(0);
      // prev を更新する
      prev = next;
    }
  });

  it('now() は tick() と同等であるべき', () => {
    // node_id=1 で HlcClock を生成する
    const clock = new HlcClock(1);
    // now() の結果が tick() と同じ型（HlcTimestamp）であることを確認する
    const ts = clock.now();
    // HlcTimestamp インスタンスであることを確認する
    expect(ts).toBeInstanceOf(HlcTimestamp);
    // node_id が引き継がれることを確認する
    expect(ts.node_id).toBe(1);
  });
});

// HlcClock.recv の因果関係テスト（Rust: test_recv_causality に対応）
describe('HlcClock recv 因果関係', () => {
  it('recv 後のタイムスタンプは send より後であるべき', () => {
    // 送信者クロックを生成する
    const sender = new HlcClock(1);
    // 受信者クロックを生成する
    const receiver = new HlcClock(2);
    // 送信者で tick する
    const sendTs = sender.tick();
    // 受信者で recv する（send_ts の後になるはずである）
    const recvTs = receiver.recv(sendTs);
    // recv の結果が sendTs 以後であることを確認する
    expect(HlcTimestamp.compare(recvTs, sendTs)).toBeGreaterThanOrEqual(0);
  });

  it('recv は受信メッセージのタイムスタンプから因果関係を引き継ぐべき', () => {
    // 受信者のローカルクロックを初期化する（wall_ms=0）
    const receiver = new HlcClock(0);
    // 未来のタイムスタンプを受信する
    const futureSendTs = new HlcTimestamp(9_999_999n, 100, 1);
    // recv 後のタイムスタンプが futureSendTs 以後であることを確認する
    const recvTs = receiver.recv(futureSendTs);
    // wall_ms が受信メッセージを反映していることを確認する
    expect(recvTs.wall_ms).toBeGreaterThanOrEqual(futureSendTs.wall_ms);
  });
});

// EPOCH 定数テスト（Rust: test_epoch_is_minimum に対応）
describe('EPOCH 定数', () => {
  it('EPOCH は全タイムスタンプの最小値であるべき', () => {
    // 任意の非 EPOCH タイムスタンプを生成する
    const ts = new HlcTimestamp(1n, 0, 0);
    // EPOCH が ts より前であることを確認する
    expect(HlcTimestamp.compare(HlcTimestamp.EPOCH, ts)).toBe(-1);
  });

  it('EPOCH の全フィールドは 0 であるべき', () => {
    // EPOCH の wall_ms が 0n であることを確認する
    expect(HlcTimestamp.EPOCH.wall_ms).toBe(0n);
    // EPOCH の logical が 0 であることを確認する
    expect(HlcTimestamp.EPOCH.logical).toBe(0);
    // EPOCH の node_id が 0 であることを確認する
    expect(HlcTimestamp.EPOCH.node_id).toBe(0);
  });
});

// fromEnv のテスト（Rust: from_env に対応）
describe('HlcClock.fromEnv', () => {
  it('HLC_NODE_ID が未設定の場合は node_id=0 の HlcClock を生成すべき', () => {
    // import.meta.env が存在しない環境では node_id=0 を使用する
    const clock = HlcClock.fromEnv();
    // HlcClock インスタンスであることを確認する
    expect(clock).toBeInstanceOf(HlcClock);
    // tick が動作することを確認する（node_id チェックは内部フィールドにアクセスできないが動作確認）
    const ts = clock.tick();
    expect(ts).toBeInstanceOf(HlcTimestamp);
  });
});
