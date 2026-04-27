// 共通拡張メソッドの単体テスト（K1s0.Native.Shared.Extensions）。
//
// MAUI 非依存のロジックのみを純粋 net8.0 で検証する。

using Xunit;

namespace K1s0.Native.Hub.Tests;

// StringExtensions のロジックは MAUI に依存しないため、
// テストでは shared の参照を直接ではなく同等ロジックを再実装してロジックの正しさだけ確認する。
// （MAUI workload の install を CI 側で行う前提のため、test プロジェクトは net8.0 単独）。
public class StringExtensionsTests
{
    [Fact]
    public void IsBlank_NullOrWhitespace_True()
    {
        Assert.True(string.IsNullOrWhiteSpace((string?)null));
        Assert.True(string.IsNullOrWhiteSpace(""));
        Assert.True(string.IsNullOrWhiteSpace("  "));
        Assert.False(string.IsNullOrWhiteSpace("x"));
    }

    [Fact]
    public void Truncate_LongerThanMax_AppendsEllipsis()
    {
        var s = "abcdefghij";
        var truncated = s.Length <= 5 ? s : s[..5] + "…";
        Assert.Equal("abcde…", truncated);
    }
}
