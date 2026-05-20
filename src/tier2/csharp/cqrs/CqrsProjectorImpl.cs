// k1s0 tier2 CQRS C# (.NET 8+) 実装クラス
// Rust 実装（cqrs/src/projector.rs ReadModelProjector トレイト）と 4 言語等価強度を持つ C# 実装版
// IReadModelProjector インターフェースを実装し、Outbox リレー経由ドメインイベントを読み取りモデルに投影する

// System.Guid などの基本型
using System;
// System.Collections.Generic: IReadOnlyList / List に使用する
using System.Collections.Generic;
// System.Text.Json: JsonElement ペイロード処理に使用する
using System.Text.Json;
// System.Threading.Tasks: 非同期処理に使用する
using System.Threading.Tasks;

// k1s0 tier2 CQRS 名前空間
namespace K1s0.Tier2.Cqrs;

/// <summary>
/// CqrsProjectorImpl: IReadModelProjector の骨格実装クラス
/// Rust の ReadModelProjector トレイト実装と等価な C# 実装を提供する
/// StateChange / OutboxRelay イベントを受け取り、読み取りモデルへの投影ログを記録する
/// 本実装は汎用的な骨格 Projector であり、業界固有ロジックを含まない
/// </summary>
public sealed class CqrsProjectorImpl : IReadModelProjector
{
    // _handledEventTypes: この投影器が処理対象とするイベント種別の一覧
    // Rust の handled_event_types() が返す &[&str] と等価である
    private readonly IReadOnlyList<string> _handledEventTypes;

    // _projectedEvents: 投影済みイベントの記録（in-memory 骨格実装 / 本番は ReadModel ストアに書く）
    private readonly List<(Guid TenantId, Guid EventId, string EventType, ulong HlcTimestamp)> _projectedEvents
        = new();

    // _lock: _projectedEvents への同時アクセスを制御する排他オブジェクト
    private readonly object _lock = new();

    /// <summary>
    /// CqrsProjectorImpl を生成する（処理対象イベント種別を受け取る）
    /// handledEventTypes: この投影器が処理する対象イベント種別名のリスト
    /// </summary>
    public CqrsProjectorImpl(IReadOnlyList<string> handledEventTypes)
    {
        // handledEventTypes が null の場合は ArgumentNullException をスローする
        _handledEventTypes = handledEventTypes ?? throw new ArgumentNullException(nameof(handledEventTypes));
    }

    /// <summary>
    /// デフォルトコンストラクタ: StateChange / OutboxRelay イベントを処理対象とする
    /// Rust の default() 実装と等価である
    /// </summary>
    public CqrsProjectorImpl()
    {
        // デフォルトで StateChange と OutboxRelay の 2 種別を処理対象とする
        _handledEventTypes = new List<string>
        {
            // aggregate 状態変更イベント
            "StateChange",
            // Outbox リレーイベント（Debezium CDC 経由で受信する）
            "OutboxRelay",
        }.AsReadOnly();
    }

    /// <summary>
    /// 投影器が処理対象とするイベント種別の一覧を返す
    /// ReadModelRegistry のルーティングに使用する
    /// </summary>
    public IReadOnlyList<string> HandledEventTypes => _handledEventTypes;

    /// <summary>
    /// ドメインイベントを受け取り読み取りモデルを更新する
    /// 失敗した場合は例外をスローする（Outbox リレーがリトライする）
    /// Rust の ReadModelProjector::project と等価な C# 非同期実装
    /// </summary>
    public Task ProjectAsync(DomainEvent domainEvent)
    {
        // domainEvent が null の場合は ArgumentNullException をスローする
        ArgumentNullException.ThrowIfNull(domainEvent);

        // テナント識別子が空 GUID の場合は不正なイベントとして拒否する（P3: tenant_id 整合性保証）
        if (domainEvent.TenantId == Guid.Empty)
        {
            // TenantId = Empty は AuthContext なし / 不正な流入を示す
            throw new InvalidOperationException(
                $"P3 テナント識別子が空です: eventId={domainEvent.Id:D}, eventType={domainEvent.EventType}");
        }

        // HLC タイムスタンプが 0 の場合は不正なイベントとして拒否する（wall-clock TTL 禁止規約）
        if (domainEvent.HlcTimestamp == 0UL)
        {
            // HlcTimestamp = 0 は HLC が未設定であることを示す
            throw new InvalidOperationException(
                $"HLC タイムスタンプが未設定です: eventId={domainEvent.Id:D}, eventType={domainEvent.EventType}");
        }

        // 読み取りモデルへの投影を記録する（in-memory 骨格実装 / 本番は ReadModel ストアに書く）
        lock (_lock)
        {
            // 投影済みイベントリストに追加する（TenantId / EventId / EventType / HlcTimestamp を記録する）
            _projectedEvents.Add((
                // テナント識別子
                TenantId: domainEvent.TenantId,
                // イベント識別子
                EventId: domainEvent.Id,
                // イベント種別
                EventType: domainEvent.EventType,
                // HLC タイムスタンプ（wall-clock 禁止 / HLC 値を使用する）
                HlcTimestamp: domainEvent.HlcTimestamp
            ));
        }

        // 投影完了を返す（例外なく終了した場合は正常終了とみなす）
        return Task.CompletedTask;
    }

    /// <summary>
    /// 投影済みイベントの件数を返す（テスト・検証用）
    /// Rust のユニットテストで projectors が空か確認するパターンに対応する
    /// </summary>
    public int ProjectedEventCount
    {
        get
        {
            // _projectedEvents の件数を排他ロックで取得する
            lock (_lock)
            {
                // リストの件数を返す
                return _projectedEvents.Count;
            }
        }
    }
}
