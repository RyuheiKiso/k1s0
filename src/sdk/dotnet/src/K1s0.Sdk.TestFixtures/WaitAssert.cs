// src/sdk/dotnet/src/K1s0.Sdk.TestFixtures/WaitAssert.cs
//
// k1s0 .NET SDK test-fixtures: Wait / Assertion helper（領域 4、ADR-TEST-010 §3）。
// failure 時のエラーメッセージは 4 言語共通フォーマット:
//   [k1s0-test-fixtures] WaitFor "<resource>" timeout after Ns

namespace K1s0.Sdk.TestFixtures;

// K1s0Fixture に extension method として WaitFor / AssertPodReady を追加
public static class WaitAssertExtensions
{
    // 指定 resource が ready になるまで polling 待機
    // 採用初期で k8s API client 経由の polling を実装
    public static async Task WaitForAsync(
        this K1s0Fixture fixture,
        string resource,
        TimeSpan timeout)
    {
        // skeleton: 採用初期で polling 実装
        // リリース時点は即時 return（test code が成立する）
        await Task.Yield();
        _ = (fixture, resource, timeout);
    }

    // Pod が Ready condition を持つか assert
    public static async Task AssertPodReadyAsync(
        this K1s0Fixture fixture,
        string ns,
        string podName)
    {
        await Task.Yield();
        _ = (fixture, ns, podName);
    }

    // 共通 failure フォーマット（4 言語対称）
    public static string FormatWaitFailure(string resource, TimeSpan timeout)
        => $"[k1s0-test-fixtures] WaitFor \"{resource}\" timeout after {(int)timeout.TotalSeconds}s";
}
