// src/sdk/dotnet/src/K1s0.Sdk.TestFixtures/K1s0Fixture.cs
//
// k1s0 .NET SDK test-fixtures: K1s0Fixture (xUnit IAsyncLifetime 互換) +
// FixtureRoot.SetupAsync 静的 entry。
//
// 設計正典:
//   ADR-TEST-010 §3 領域 1（Setup / Teardown）
//   docs/05_実装/30_CI_CD設計/35_e2e_test_design/30_test_fixtures/01_4言語対称API.md

namespace K1s0.Sdk.TestFixtures;

// 利用者は xUnit IClassFixture<K1s0Fixture> または直接 IAsyncLifetime として使う
public sealed class K1s0Fixture
{
    // Setup 時の Options（再利用 + debug 用）
    public Options Options { get; }

    // 12 service の mock data builder への entry point
    public MockBuilderRoot MockBuilder { get; }

    // private constructor — 直接 new ではなく FixtureRoot.SetupAsync 経由で生成
    private K1s0Fixture(Options options)
    {
        Options = options;
        MockBuilder = new MockBuilderRoot(options.Tenant);
    }

    // SetupAsync は kind cluster 起動 + k1s0 install + SDK client の前提整備を行う
    // 採用初期で kind / helm / kubectl の Process 起動を実装。
    // リリース時点では skeleton（cluster 起動済前提で fixture を返す）。
    public static async Task<K1s0Fixture> SetupAsync(Options options)
    {
        // 念のため async 文脈を確保（採用初期の実装で意味を持つ）
        await Task.Yield();
        return new K1s0Fixture(options);
    }

    // 後片付け（採用初期で tools/e2e/user/down.sh を Process spawn）
    public async Task TeardownAsync()
    {
        await Task.Yield();
        // skeleton: 採用初期で down.sh spawn を実装
    }

    // tier1 facade Pod が Ready になるまで待機（採用初期で kubectl wait wrapper）
    public async Task WaitForTier1FacadeReadyAsync()
    {
        await Task.Yield();
    }

    // SDK client 生成（採用初期で K1s0.Sdk.Grpc.K1s0Client wrapper として実装）
    public K1s0SdkClient NewSDKClient(string tenant)
    {
        // tenant 未指定時は Options の Tenant
        if (string.IsNullOrEmpty(tenant))
        {
            tenant = Options.Tenant;
        }
        return new K1s0SdkClient(tenant, this);
    }
}

// SDK client の薄い wrapper（採用初期で K1s0.Sdk.Grpc.K1s0Client を内包）
public sealed class K1s0SdkClient
{
    public string Tenant { get; }
    private readonly K1s0Fixture _fixture;

    internal K1s0SdkClient(string tenant, K1s0Fixture fixture)
    {
        Tenant = tenant;
        _fixture = fixture;
    }

    // State.Set RPC（採用初期で K1s0Client.State.SetAsync wrapper）
    public Task SetStateAsync(string key, byte[] value)
    {
        // skeleton（採用初期で実装）
        _ = (key, value);
        return Task.CompletedTask;
    }

    // State.Get RPC
    public Task<byte[]?> GetStateAsync(string key)
    {
        _ = key;
        return Task.FromResult<byte[]?>(null);
    }
}
