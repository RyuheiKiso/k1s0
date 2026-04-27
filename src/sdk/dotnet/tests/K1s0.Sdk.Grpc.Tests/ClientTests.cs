// 本ファイルは K1s0.Sdk.Grpc の最小単体テスト雛形（xunit）。

using K1s0.Sdk;
using K1s0.Sdk.Generated.K1s0.Tier1.Common.V1;
using K1s0.Sdk.Generated.K1s0.Tier1.Log.V1;
using Xunit;

namespace K1s0.Sdk.Grpc.Tests;

public sealed class ClientTests
{
    private static K1s0Client CreateClient() => new(new K1s0Config
    {
        // 実接続は行わない、構築のみ検証する目的のためダミー URL を渡す。
        Target = "http://localhost:50001",
        TenantId = "tenant-A",
        Subject = "svc-foo",
    });

    [Fact]
    public void K1s0Config_HoldsValues()
    {
        var cfg = new K1s0Config { Target = "http://x", TenantId = "t", Subject = "s" };
        Assert.Equal("http://x", cfg.Target);
        Assert.Equal("t", cfg.TenantId);
        Assert.Equal("s", cfg.Subject);
    }

    [Fact]
    public void K1s0Client_Exposes14Facades()
    {
        using var client = CreateClient();

        Assert.NotNull(client.State);
        Assert.NotNull(client.PubSub);
        Assert.NotNull(client.Secrets);
        Assert.NotNull(client.Log);
        Assert.NotNull(client.Workflow);
        Assert.NotNull(client.Decision);
        Assert.NotNull(client.Audit);
        Assert.NotNull(client.Pii);
        Assert.NotNull(client.Feature);
        Assert.NotNull(client.Binding);
        Assert.NotNull(client.Invoke);
        Assert.NotNull(client.Telemetry);
        Assert.NotNull(client.DecisionAdmin);
        Assert.NotNull(client.FeatureAdmin);
        Assert.NotNull(client.Raw);
    }

    [Fact]
    public void K1s0Client_ExposesAll14RawClients()
    {
        using var client = CreateClient();

        Assert.NotNull(client.Raw.Audit);
        Assert.NotNull(client.Raw.Binding);
        Assert.NotNull(client.Raw.Decision);
        Assert.NotNull(client.Raw.DecisionAdmin);
        Assert.NotNull(client.Raw.Feature);
        Assert.NotNull(client.Raw.FeatureAdmin);
        Assert.NotNull(client.Raw.Log);
        Assert.NotNull(client.Raw.Pii);
        Assert.NotNull(client.Raw.PubSub);
        Assert.NotNull(client.Raw.Secrets);
        Assert.NotNull(client.Raw.ServiceInvoke);
        Assert.NotNull(client.Raw.State);
        Assert.NotNull(client.Raw.Telemetry);
        Assert.NotNull(client.Raw.Workflow);
    }
}

public sealed class IdlAlignmentTests
{
    [Fact]
    public void Severity_AlignsWithOtel()
    {
        // docs 正典 docs/03_要件定義/20_機能要件/40_tier1_API契約IDL/07_Log_API.md
        // OTel SeverityNumber 仕様と一致すること。
        Assert.Equal(0, (int)Severity.Trace);
        Assert.Equal(5, (int)Severity.Debug);
        Assert.Equal(9, (int)Severity.Info);
        Assert.Equal(13, (int)Severity.Warn);
        Assert.Equal(17, (int)Severity.Error);
        Assert.Equal(21, (int)Severity.Fatal);
    }

    [Fact]
    public void K1s0ErrorCategory_AlignsWithIdl()
    {
        // docs 正典 docs/03_要件定義/20_機能要件/40_tier1_API契約IDL/00_共通型定義.md
        Assert.Equal(0, (int)K1s0ErrorCategory.K1S0ErrorUnspecified);
        Assert.Equal(1, (int)K1s0ErrorCategory.K1S0ErrorInvalidArgument);
        Assert.Equal(9, (int)K1s0ErrorCategory.K1S0ErrorDeadlineExceeded);
    }
}
