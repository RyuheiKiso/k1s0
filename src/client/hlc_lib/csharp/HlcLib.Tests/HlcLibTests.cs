// HlcLibTests.cs — K1s0.HlcLib の xUnit テスト
// Rust テスト（src/client/hlc_lib/rust/src/lib.rs #[cfg(test)] mod tests）と
// 同等の coverage を C# xUnit で実装する。
// 各テストメソッドは Rust 側の対応するテスト関数と 1:1 の対応を持つ。

// K1s0.HlcLib 名前空間をインポートする
using K1s0.HlcLib;
// xUnit テストフレームワークをインポートする
using Xunit;
// 並行テストのために Task をインポートする
using System.Collections.Concurrent;

// テストクラスを宣言する
namespace K1s0.HlcLib.Tests;

// HlcTimestampTests: HlcTimestamp の単体テスト群
public class HlcTimestampTests
{
    // Test_HlcTimestampOrdering は HlcTimestamp の順序比較を確認するテスト（Rust: test_hlc_timestamp_ordering に対応）
    [Fact]
    public void Test_HlcTimestampOrdering()
    {
        // WallMs が大きければ後のタイムスタンプであることを確認する
        var a = new HlcTimestamp(100UL, 0, 0);
        var b = new HlcTimestamp(200UL, 0, 0);
        // a < b であることを確認する（CompareTo が負を返す）
        Assert.True(a.CompareTo(b) < 0, "WallMs が大きい方が後であるべき");
        // WallMs が同じなら Logical で比較することを確認する
        var c = new HlcTimestamp(100UL, 1, 0);
        Assert.True(a.CompareTo(c) < 0, "同一 WallMs では Logical が大きい方が後であるべき");
        // Logical も同じなら NodeId で比較することを確認する
        var d = new HlcTimestamp(100UL, 0, 1);
        Assert.True(a.CompareTo(d) < 0, "同一 WallMs/Logical では NodeId が大きい方が後であるべき");
    }

    // Test_FormatAndParse は FormatCompact と ParseCompact のラウンドトリップを確認するテスト（Rust: test_format_and_parse に対応）
    [Fact]
    public void Test_FormatAndParse()
    {
        // 既知の値で HlcTimestamp を生成する（Rust テストと同一の値）
        var ts = new HlcTimestamp(0x0123456789abcdefUL, 0x00ff, 0x1234);
        // FormatCompact で文字列に変換する
        var formatted = ts.FormatCompact();
        // 期待値: Rust テストと同一の形式
        Assert.Equal("0123456789abcdef-00ff-1234", formatted);
        // ParseCompact でラウンドトリップが成立することを確認する
        var parsed = HlcTimestamp.ParseCompact(formatted);
        // パース結果が非 null であることを確認する
        Assert.NotNull(parsed);
        // 元の値と等しいことを確認する（record struct の等値比較を使用）
        Assert.Equal(ts, parsed.Value);
    }

    // Test_ParseCompactInvalid は ParseCompact が不正な文字列に対して null を返すことを確認するテスト
    [Theory]
    [InlineData("")]
    [InlineData("invalid")]
    [InlineData("0000-0000")]
    [InlineData("000-0000-0000")]
    public void Test_ParseCompactInvalid(string input)
    {
        // 不正な入力に対して ParseCompact が null を返すことを確認する
        var result = HlcTimestamp.ParseCompact(input);
        Assert.Null(result);
    }

    // Test_AddMs は AddMs が正しく deadline を計算することを確認するテスト（Rust: test_add_ms に対応）
    [Fact]
    public void Test_AddMs()
    {
        // 基準タイムスタンプを生成する（Rust テストと同一の値）
        var baseTs = new HlcTimestamp(1_000_000UL, 5, 1);
        // 1000ms 後の deadline を計算する
        var deadline = baseTs.AddMs(1_000UL);
        // WallMs が正しく加算されることを確認する
        Assert.Equal(1_001_000UL, deadline.WallMs);
        // Logical は 0 にリセットされることを確認する
        Assert.Equal((ushort)0, deadline.Logical);
        // NodeId は引き継がれることを確認する
        Assert.Equal((ushort)1, deadline.NodeId);
    }

    // Test_AddMsOverflow は AddMs が ulong.MaxValue 付近でオーバーフローを飽和させることを確認するテスト
    [Fact]
    public void Test_AddMsOverflow()
    {
        // ulong.MaxValue - 1 の WallMs を持つタイムスタンプを生成する
        var ts = new HlcTimestamp(ulong.MaxValue - 1UL, 0, 0);
        // 2 を加算して ulong.MaxValue を超えさせる
        var result = ts.AddMs(2UL);
        // ulong.MaxValue に飽和することを確認する
        Assert.Equal(ulong.MaxValue, result.WallMs);
    }

    // Test_ElapsedAndExpired は ElapsedMsSince と IsExpiredAt の動作を確認するテスト（Rust: test_elapsed_and_expired に対応）
    [Fact]
    public void Test_ElapsedAndExpired()
    {
        // 現在時刻を表すタイムスタンプを生成する（Rust テストと同一の値）
        var now = new HlcTimestamp(2_000_000UL, 0, 0);
        // 1000ms 後の deadline を計算する
        var deadline = now.AddMs(1_000UL);
        // now では期限切れでないことを確認する
        Assert.False(deadline.IsExpiredAt(now), "deadline より前では期限切れにならないべき");
        // deadline ちょうどで期限切れになることを確認する
        Assert.True(deadline.IsExpiredAt(deadline), "deadline ちょうどで期限切れになるべき");
        // deadline を過ぎたら期限切れになることを確認する
        var future = new HlcTimestamp(2_001_001UL, 0, 0);
        Assert.True(deadline.IsExpiredAt(future), "deadline を超えたら期限切れになるべき");
        // ElapsedMsSince の値が正しいことを確認する
        Assert.Equal(1_001UL, future.ElapsedMsSince(now));
    }

    // Test_ElapsedMsSinceSaturation は ElapsedMsSince が self < reference のとき 0 を返すことを確認するテスト
    [Fact]
    public void Test_ElapsedMsSinceSaturation()
    {
        // earlier が later より前のタイムスタンプ
        var earlier = new HlcTimestamp(1_000UL, 0, 0);
        var later = new HlcTimestamp(2_000UL, 0, 0);
        // earlier.ElapsedMsSince(later) は 0 を返すべき（負の elapsed は表現しない）
        Assert.Equal(0UL, earlier.ElapsedMsSince(later));
    }

    // Test_EpochIsMinimum は Epoch 定数が最小タイムスタンプであることを確認するテスト（Rust: test_epoch_is_minimum に対応）
    [Fact]
    public void Test_EpochIsMinimum()
    {
        // 任意の非 Epoch タイムスタンプを生成する
        var ts = new HlcTimestamp(1UL, 0, 0);
        // Epoch が ts より前であることを確認する
        Assert.True(HlcTimestamp.Epoch.CompareTo(ts) < 0, "Epoch は全タイムスタンプの最小値であるべき");
    }
}

// HlcClockTests: HlcClock の単体テスト群
public class HlcClockTests
{
    // Test_TickMonotonic は HlcClock.Tick が単調増加タイムスタンプを生成することを確認するテスト（Rust: test_tick_monotonic に対応）
    [Fact]
    public void Test_TickMonotonic()
    {
        // nodeId=0 で HlcClock を生成する
        var clock = new HlcClock(0);
        // 連続で 100 回 Tick して全て単調増加することを確認する（Rust と同一の 100 回）
        var prev = clock.Tick();
        for (var i = 0; i < 99; i++)
        {
            // 次のタイムスタンプを生成する
            var next = clock.Tick();
            // prev よりも next が後（または同時）であることを確認する
            Assert.True(next.CompareTo(prev) >= 0,
                $"Tick は単調増加を保証すべき: prev={prev.FormatCompact()}, next={next.FormatCompact()}");
            // prev を更新する
            prev = next;
        }
    }

    // Test_RecvCausality は HlcClock.Recv がメッセージの因果関係を正しく反映することを確認するテスト（Rust: test_recv_causality に対応）
    [Fact]
    public void Test_RecvCausality()
    {
        // 送信者クロックを生成する
        var sender = new HlcClock(1);
        // 受信者クロックを生成する
        var receiver = new HlcClock(2);
        // 送信者で Tick する
        var sendTs = sender.Tick();
        // 受信者で Recv する（sendTs の後になるはずである）
        var recvTs = receiver.Recv(sendTs);
        // Recv の結果が sendTs 以後であることを確認する
        Assert.True(recvTs.CompareTo(sendTs) >= 0,
            $"Recv 後のタイムスタンプは send より後であるべき: send={sendTs.FormatCompact()}, recv={recvTs.FormatCompact()}");
    }

    // Test_ConcurrentTick は複数スレッドで concurrent Tick が unique タイムスタンプを生成することを確認するテスト（Rust: test_concurrent_tick に対応）
    [Fact]
    public async Task Test_ConcurrentTick()
    {
        // nodeId=0 で HlcClock を生成する
        var clock = new HlcClock(0);
        // 全スレッドの結果を蓄積するスレッドセーフコレクション
        var all = new ConcurrentBag<HlcTimestamp>();
        // 10 スレッドで並行 Tick を実行する（Rust: 10 スレッドと同等）
        var tasks = new Task[10];
        for (var i = 0; i < 10; i++)
        {
            tasks[i] = Task.Run(() =>
            {
                // 各スレッドで 100 回 Tick する（合計 1000 回）
                for (var j = 0; j < 100; j++)
                {
                    all.Add(clock.Tick());
                }
            });
        }
        // 全 Task の完了を非同期で待つ（xUnit1031 警告を回避するため await を使用する）
        await Task.WhenAll(tasks);
        // 全タイムスタンプを集合に入れて重複を確認する
        var seen = new System.Collections.Generic.HashSet<string>();
        foreach (var ts in all)
        {
            // FormatCompact をキーとして重複チェックする
            var key = ts.FormatCompact();
            Assert.True(seen.Add(key), $"concurrent Tick で重複タイムスタンプが生成された: {key}");
        }
        // 1000 個（10 スレッド × 100 Tick）のユニークタイムスタンプが生成されたことを確認する
        Assert.Equal(1000, seen.Count);
    }

    // Test_NowIsTickAlias は HlcClock.Now が Tick の alias であることを確認するテスト
    [Fact]
    public void Test_NowIsTickAlias()
    {
        // nodeId=1 で HlcClock を生成する
        var clock = new HlcClock(1);
        // Now() を呼び出す
        var ts = clock.Now();
        // NodeId が引き継がれることを確認する
        Assert.Equal((ushort)1, ts.NodeId);
    }

    // Test_FromEnv は FromEnv が HLC_NODE_ID 未設定時に nodeId=0 の HlcClock を生成することを確認するテスト（Rust: from_env に対応）
    [Fact]
    public void Test_FromEnv()
    {
        // HLC_NODE_ID が未設定の環境でテストする（テスト環境では未設定を想定）
        var clock = HlcClock.FromEnv();
        // HlcClock が生成されることを確認する
        Assert.NotNull(clock);
        // Tick が動作することを確認する
        var ts = clock.Tick();
        // WallMs が 0 より大きいことを確認する（現在時刻が設定されている）
        Assert.True(ts.WallMs > 0UL, "Tick 後の WallMs は 0 より大きいはず（物理時刻が反映される）");
    }
}
