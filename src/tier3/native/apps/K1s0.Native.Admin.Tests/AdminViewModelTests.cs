// AdminViewModel の単体テスト。
//
// MAUI に依存しない純粋ロジックのみを検証する（TFM net8.0、MAUI workload 不要）。
// IK1s0Service / AuditEvent / AdminViewModel 相当のロジックを test 内に等価再実装し、
// Items への詰め替えと StatusMessage 遷移が期待通りに動くことを確認する。
//
// アプリ本体（K1s0.Native.Admin）は MAUI workload を要するため、test project から
// 直接 ProjectReference せず、ロジック互換のシム型で検証する（CI が MAUI workload
// を install しなくても test が走るようにする方針、Hub.Tests と同じ）。

using System.Collections.ObjectModel;
using Xunit;

namespace K1s0.Native.Admin.Tests;

// アプリ本体と同形のレコード（MAUI 非依存）。
public sealed record AuditEventShim(
    long OccurredAtMillis,
    string Actor,
    string Action,
    string Resource,
    string Outcome);

// アプリ本体の IK1s0Service と同形の最小 interface（MAUI 非依存）。
public interface IK1s0ServiceShim
{
    Task<IReadOnlyList<AuditEventShim>> QueryAuditAsync(int hours, int limit, CancellationToken ct = default);
}

// アプリ本体の AdminViewModel のロジック等価シム（MAUI / INotifyPropertyChanged 抜き）。
public sealed class AdminViewModelShim
{
    private readonly IK1s0ServiceShim _service;
    public ObservableCollection<AuditEventShim> Items { get; } = new();
    public string StatusMessage { get; private set; } = "ready";

    public AdminViewModelShim(IK1s0ServiceShim service) => _service = service;

    public async Task ExecuteQueryAsync(int hours, int limit)
    {
        StatusMessage = "querying…";
        try
        {
            var events = await _service.QueryAuditAsync(hours, limit);
            Items.Clear();
            foreach (var e in events)
            {
                Items.Add(e);
            }
            StatusMessage = $"ok ({events.Count} events)";
        }
        catch (Exception ex)
        {
            StatusMessage = $"error: {ex.Message}";
        }
    }
}

// Items への詰め替え / StatusMessage 遷移の挙動を検証する。
public class AdminViewModelTests
{
    private sealed class FakeService : IK1s0ServiceShim
    {
        public IReadOnlyList<AuditEventShim>? Result { get; init; }
        public Exception? Throws { get; init; }
        public int CallCount { get; private set; }
        public int LastHours { get; private set; }
        public int LastLimit { get; private set; }

        public Task<IReadOnlyList<AuditEventShim>> QueryAuditAsync(int hours, int limit, CancellationToken ct = default)
        {
            CallCount++;
            LastHours = hours;
            LastLimit = limit;
            if (Throws is not null)
            {
                throw Throws;
            }
            return Task.FromResult(Result ?? Array.Empty<AuditEventShim>());
        }
    }

    [Fact]
    public async Task ExecuteQuery_Success_PopulatesItemsAndOkStatus()
    {
        var fake = new FakeService
        {
            Result = new[]
            {
                new AuditEventShim(1_700_000_000_000, "alice", "READ", "user/1", "SUCCESS"),
                new AuditEventShim(1_700_000_001_000, "bob", "WRITE", "user/2", "DENIED"),
            },
        };
        var vm = new AdminViewModelShim(fake);
        await vm.ExecuteQueryAsync(24, 50);

        Assert.Equal(2, vm.Items.Count);
        Assert.Equal("alice", vm.Items[0].Actor);
        Assert.Equal("DENIED", vm.Items[1].Outcome);
        Assert.Equal("ok (2 events)", vm.StatusMessage);
        Assert.Equal(1, fake.CallCount);
        Assert.Equal(24, fake.LastHours);
        Assert.Equal(50, fake.LastLimit);
    }

    [Fact]
    public async Task ExecuteQuery_EmptyResult_ClearsItemsAndOkStatus()
    {
        var fake = new FakeService { Result = Array.Empty<AuditEventShim>() };
        var vm = new AdminViewModelShim(fake);
        // 既に items がある状態で呼ばれた時に clear される動作を再確認する。
        vm.Items.Add(new AuditEventShim(0, "stale", "x", "x", "x"));
        await vm.ExecuteQueryAsync(1, 10);

        Assert.Empty(vm.Items);
        Assert.Equal("ok (0 events)", vm.StatusMessage);
    }

    [Fact]
    public async Task ExecuteQuery_UpstreamThrows_SetsErrorStatusWithoutMutatingItems()
    {
        var fake = new FakeService { Throws = new HttpRequestException("BFF returned 502") };
        var vm = new AdminViewModelShim(fake);
        var pre = new AuditEventShim(0, "kept", "x", "x", "x");
        vm.Items.Add(pre);
        await vm.ExecuteQueryAsync(24, 50);

        // 例外時は Items は変更しない（API 失敗で表示が消えるのを防ぐ）。
        Assert.Single(vm.Items);
        Assert.Same(pre, vm.Items[0]);
        Assert.StartsWith("error:", vm.StatusMessage);
        Assert.Contains("BFF returned 502", vm.StatusMessage);
    }
}
