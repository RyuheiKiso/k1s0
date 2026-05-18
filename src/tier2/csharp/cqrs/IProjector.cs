// k1s0 tier2 CQRS C# (.NET 8+) インターフェース定義
// Rust 実装（cqrs/src/projector.rs）と 4 言語等価強度を持つ C# 版
// Outbox リレー経由で受信したドメインイベントを読み取りモデルに投影する（設計方針 15）

// System.Guid などの基本型
using System;
// System.Collections.Generic: コレクション型
using System.Collections.Generic;
// System.Linq: IReadOnlyList<string>.Any() / Contains 拡張メソッドに使用する
using System.Linq;
// System.Text.Json: JSON ペイロードの型
using System.Text.Json;
// System.Threading.Tasks: 非同期処理に使用する
using System.Threading.Tasks;

// k1s0 tier2 CQRS 名前空間
namespace K1s0.Tier2.Cqrs;

/// <summary>
/// DomainEvent: ドメインイベントの構造体定義（Outbox から受信する共通エンベロープ形式）
/// Rust の DomainEvent 構造体に対応する
/// </summary>
public sealed record DomainEvent
{
    // イベント一意識別子（UUID v4）
    public required Guid Id { get; init; }
    // テナント識別子（RLS 述語の基底 / AuthContext からのみ注入する）
    public required Guid TenantId { get; init; }
    // 集約型名（業界中立語のみ使用可）
    public required string AggregateType { get; init; }
    // イベント種別名
    public required string EventType { get; init; }
    // ペイロード（JSON Value 形式 / PII フィールドは Outbox 書込前に redact 済み）
    public required JsonElement Payload { get; init; }
    // HLC タイムスタンプ（wall clock TTL 禁止規約により HLC を使用する）
    public required ulong HlcTimestamp { get; init; }
}

/// <summary>
/// IReadModelProjector: 読み取りモデル投影器インターフェース
/// Rust の ReadModelProjector トレイトに対応する
/// すべての読み取りモデル投影器が実装する契約
/// Project メソッドはドメインイベントを受け取り読み取りモデルを更新する
/// </summary>
public interface IReadModelProjector
{
    /// <summary>
    /// ドメインイベントを受け取り読み取りモデルを更新する
    /// 失敗した場合は例外をスローする（Outbox リレーがリトライする）
    /// </summary>
    // ProjectAsync メソッド（読み取りモデル投影操作）
    Task ProjectAsync(DomainEvent domainEvent);

    /// <summary>
    /// 投影器が処理対象とするイベント種別の一覧を返す
    /// 登録済み投影器のルーティングに使用する
    /// </summary>
    // HandledEventTypes プロパティ（処理対象イベント種別一覧）
    IReadOnlyList<string> HandledEventTypes { get; }
}

/// <summary>
/// ReadModelRegistry: 複数の IReadModelProjector を集約するレジストリ
/// イベント種別に基づいて対応する投影器へルーティングする
/// Rust の ReadModelRegistry 構造体に対応する
/// </summary>
public sealed class ReadModelRegistry
{
    // 登録済み投影器のリスト（IReadModelProjector 形式で保持する）
    private readonly List<IReadModelProjector> _projectors = new();

    /// <summary>
    /// 投影器をレジストリに登録する
    /// </summary>
    // Register メソッド（投影器登録操作）
    public void Register(IReadModelProjector projector)
    {
        // 投影器が null の場合は ArgumentNullException をスローする
        ArgumentNullException.ThrowIfNull(projector);
        // 投影器リストに追加する
        _projectors.Add(projector);
    }

    /// <summary>
    /// ドメインイベントを対応する投影器へルーティングして投影する
    /// </summary>
    // DispatchAsync メソッド（イベントルーティングおよび投影操作）
    public async Task DispatchAsync(DomainEvent domainEvent)
    {
        // domainEvent が null の場合は ArgumentNullException をスローする
        ArgumentNullException.ThrowIfNull(domainEvent);
        // 登録済み投影器を順に検索して対象イベント種別を処理する
        foreach (var projector in _projectors)
        {
            // 投影器がこのイベント種別を処理するか確認する
            if (projector.HandledEventTypes.Contains(domainEvent.EventType))
            {
                // 対応投影器に処理を委譲する
                await projector.ProjectAsync(domainEvent).ConfigureAwait(false);
            }
        }
    }
}
