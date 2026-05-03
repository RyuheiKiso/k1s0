// src/sdk/dotnet/src/K1s0.Sdk.TestFixtures/MockBuilder.cs
//
// k1s0 .NET SDK test-fixtures: Mock builder fluent API（領域 3、ADR-TEST-010 §3）。
// リリース時点で 3 service（State / Audit / PubSub）を提供。
// 採用初期で +3 (Workflow / Decision / Secret)、運用拡大時で残 6 を追加。

namespace K1s0.Sdk.TestFixtures;

// 12 service の mock builder への entry point
public sealed class MockBuilderRoot
{
    // 既定 tenant（builder の WithTenant 未指定時に使う）
    private readonly string _defaultTenant;

    internal MockBuilderRoot(string defaultTenant)
    {
        _defaultTenant = defaultTenant;
    }

    // State service の mock builder
    public StateMockBuilder State() => new(_defaultTenant);

    // Audit service の mock builder
    public AuditMockBuilder Audit() => new(_defaultTenant);

    // PubSub service の mock builder
    public PubSubMockBuilder PubSub() => new(_defaultTenant);

    // Workflow / Decision / Secret は採用初期で real 実装する phase marker（標準 .NET 未実装例外ではなく ADR cite 付き独自型を投げる、SHIP_STATUS §9）
    public WorkflowMockBuilder Workflow()
        => throw new FixturePhaseUnsupportedException("Workflow", "採用初期");
    public DecisionMockBuilder Decision()
        => throw new FixturePhaseUnsupportedException("Decision", "採用初期");
    public SecretMockBuilder Secret()
        => throw new FixturePhaseUnsupportedException("Secret", "採用初期");
}

// FixturePhaseUnsupportedException は ADR-TEST-010 §3 段階展開で「採用初期 / 運用拡大時で
// real 実装する」と確定済の builder を呼んだ時に投げる phase marker exception。
// .NET 標準の汎用未実装例外は「未着手」「放置」を示唆するため意図的に避け、
// ADR 名 + 実装 phase を message に含める独自型で「設計上の段階展開」であることを明示する。
public sealed class FixturePhaseUnsupportedException : InvalidOperationException
{
    public string Service { get; }
    public string Phase { get; }
    public FixturePhaseUnsupportedException(string service, string phase)
        : base($"ADR-TEST-010 PHASE: {service} mock builder は{phase}で実装（リリース時点 phase marker）")
    {
        Service = service;
        Phase = phase;
    }
}

// State service mock data の fluent builder
public sealed class StateMockBuilder
{
    private string _tenant;
    private string _key = string.Empty;
    private byte[] _value = Array.Empty<byte>();
    private int _ttl = 0;

    internal StateMockBuilder(string tenant) => _tenant = tenant;

    public StateMockBuilder WithTenant(string tenant) { _tenant = tenant; return this; }
    public StateMockBuilder WithKey(string key) { _key = key; return this; }
    public StateMockBuilder WithValue(byte[] value) { _value = value; return this; }
    public StateMockBuilder WithTTL(int seconds) { _ttl = seconds; return this; }

    // build: 最終的な StateEntry を返す（採用初期で contracts/proto 型に置換）
    public StateEntry Build() => new()
    {
        Tenant = _tenant,
        Key = _key,
        Value = _value,
        TTL = _ttl,
    };
}

// State service の wire 形式
public sealed record StateEntry
{
    public string Tenant { get; init; } = string.Empty;
    public string Key { get; init; } = string.Empty;
    public byte[] Value { get; init; } = Array.Empty<byte>();
    public int TTL { get; init; }
}

// Audit service mock data の fluent builder
public sealed class AuditMockBuilder
{
    private string _tenant;
    private int _entryCount = 0;
    private long _startSeq = 0;

    internal AuditMockBuilder(string tenant) => _tenant = tenant;

    public AuditMockBuilder WithTenant(string tenant) { _tenant = tenant; return this; }
    public AuditMockBuilder WithEntries(int n) { _entryCount = n; return this; }
    public AuditMockBuilder WithSequence(long seq) { _startSeq = seq; return this; }

    public IReadOnlyList<AuditEntry> Build()
    {
        var entries = new List<AuditEntry>(_entryCount);
        for (int i = 0; i < _entryCount; i++)
        {
            entries.Add(new AuditEntry
            {
                Tenant = _tenant,
                Sequence = _startSeq + i,
                // 採用初期で SHA-256 prev_id chain を計算
                PrevID = string.Empty,
            });
        }
        return entries;
    }
}

public sealed record AuditEntry
{
    public string Tenant { get; init; } = string.Empty;
    public long Sequence { get; init; }
    public string PrevID { get; init; } = string.Empty;
}

// PubSub service mock data の fluent builder
public sealed class PubSubMockBuilder
{
    private string _tenant;
    private string _topic = string.Empty;
    private int _messages = 0;
    private int _delayMs = 0;

    internal PubSubMockBuilder(string tenant) => _tenant = tenant;

    public PubSubMockBuilder WithTenant(string tenant) { _tenant = tenant; return this; }
    public PubSubMockBuilder WithTopic(string topic) { _topic = topic; return this; }
    public PubSubMockBuilder WithMessages(int n) { _messages = n; return this; }
    public PubSubMockBuilder WithDelayMs(int ms) { _delayMs = ms; return this; }

    public IReadOnlyList<PubSubMessage> Build()
    {
        var msgs = new List<PubSubMessage>(_messages);
        for (int i = 0; i < _messages; i++)
        {
            msgs.Add(new PubSubMessage
            {
                Tenant = _tenant,
                Topic = _topic,
                SeqID = i,
            });
        }
        return msgs;
    }
}

public sealed record PubSubMessage
{
    public string Tenant { get; init; } = string.Empty;
    public string Topic { get; init; } = string.Empty;
    public long SeqID { get; init; }
}

// 採用初期で実装する builder の placeholder type
public sealed class WorkflowMockBuilder { }
public sealed class DecisionMockBuilder { }
public sealed class SecretMockBuilder { }
