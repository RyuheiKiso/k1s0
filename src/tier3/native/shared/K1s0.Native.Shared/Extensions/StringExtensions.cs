// 文字列拡張メソッド（tier3 native 共通）。

namespace K1s0.Native.Shared.Extensions;

public static class StringExtensions
{
    // null / 空文字 / 空白のみのチェック。
    public static bool IsBlank(this string? value) => string.IsNullOrWhiteSpace(value);

    // 文字列を最大 n 文字に切り詰めて末尾に "…" を付ける。
    public static string Truncate(this string? value, int max)
    {
        if (string.IsNullOrEmpty(value))
        {
            return string.Empty;
        }
        return value.Length <= max ? value : value[..max] + "…";
    }
}
