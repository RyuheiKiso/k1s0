namespace K1s0.System.TestHelper.Fixtures;

/// <summary>テスト用フィクスチャビルダー。</summary>
public static class FixtureBuilder
{
    /// <summary>ランダム UUID を生成する。</summary>
    public static string Uuid() => Guid.NewGuid().ToString();

    /// <summary>ランダムなテスト用メールアドレスを生成する。</summary>
    public static string Email() => $"test-{Guid.NewGuid().ToString()[..8]}@example.com";

    /// <summary>ランダムなテスト用ユーザー名を生成する。</summary>
    public static string Name() => $"user-{Guid.NewGuid().ToString()[..8]}";

    /// <summary>指定範囲のランダム整数を生成する。</summary>
    public static int Int(int min = 0, int max = 100)
    {
        if (min >= max)
        {
            return min;
        }

        return Random.Shared.Next(min, max);
    }

    /// <summary>テスト用テナント ID を生成する。</summary>
    public static string TenantId() => $"tenant-{Guid.NewGuid().ToString()[..8]}";
}
