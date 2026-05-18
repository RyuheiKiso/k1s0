// k1s0 tier2 Outbox relay C# (.NET 8+) 実装
// Rust 実装（outbox.rs）と 4 言語等価強度を持つ C# 版
// atomic_triple_write と同一 txn で書かれた Outbox エントリを Kafka に非同期転送する
// Debezium CDC 経由の転送（直接 produce は禁止）

// Guid / Exception 等の基本型
using System;
// コレクション型
using System.Collections.Generic;
// 非同期処理に使用する
using System.Threading;
using System.Threading.Tasks;

// k1s0 tier2 名前空間
namespace K1s0.Tier2;

/// <summary>
/// Outbox ペイロード（PII 平文を含まない設計）
/// Kafka に転送するデータを格納する
/// </summary>
public sealed record OutboxPayload(
    // イベントの種別識別子（aggregate の型名を表す）
    string AggregateType,
    // イベントの内容（PII は redact 済みまたは構造的に不在）
    string Data,
    // メタデータ（trace_id / version 等を JSON 文字列で格納する）
    string Metadata
);

/// <summary>
/// Outbox テーブルのエントリ（Domain Event を Kafka に転送するための中継記録）
/// Rust の OutboxEntry と意味的に等価な C# 版
/// </summary>
public sealed record OutboxMessage(
    // Outbox エントリの主キー（Guid）
    Guid Id,
    // テナント ID（Debezium CDC が Kafka routing に使用する）
    Guid TenantId,
    // 関連する aggregate の ID
    Guid AggregateId,
    // イベント種別（Debezium が Kafka topic routing に使用する）
    string EventType,
    // Kafka に転送するペイロード（PII は含まない）
    OutboxPayload Payload,
    // 冪等性キー（同一メッセージの二重投入を防止する）
    string IdempotencyKey,
    // 書込日時（UTC）
    DateTimeOffset CreatedAt,
    // Debezium が処理済みにする日時（null = 未処理）
    DateTimeOffset? ProcessedAt = null
)
{
    // IdempotencyKeyTTL: 冪等性キーの有効期限（24 時間）
    // 同一の idempotency_key でのダブル書込みを防止する
    public static readonly TimeSpan IdempotencyKeyTTL = TimeSpan.FromHours(24);

    /// <summary>
    /// 冪等性キーが TTL を超過しているかを返す
    /// TTL は IdempotencyKeyTTL（24 時間）で定義される
    /// </summary>
    public bool IsExpired()
    {
        // 作成日時から IdempotencyKeyTTL を加算した時刻が現在時刻より前かを確認する
        return DateTimeOffset.UtcNow > CreatedAt.Add(IdempotencyKeyTTL);
    }
}

/// <summary>
/// IOutboxStore&lt;T&gt;: Outbox の永続化インターフェース
/// tenant_id は引数で受け取らず、内部的に TenantContext から取得する
/// T は IOutboxEntry インターフェースを実装する型に制限する（sealed record は制約に使えないため interface を使う）
/// </summary>
public interface IOutboxEntry { }

/// <summary>
/// IOutboxStore&lt;T&gt;: Outbox の永続化インターフェース
/// </summary>
public interface IOutboxStore<T> where T : IOutboxEntry
{
    /// <summary>
    /// Outbox エントリを永続化する（atomic_triple_write と同一 txn で呼ぶ）
    /// msg が null の場合は ArgumentNullException をスローする
    /// </summary>
    Task SaveAsync(T msg, CancellationToken cancellationToken = default);

    /// <summary>
    /// aggregate_id に紐づく未処理の Outbox エントリを取得する
    /// tenant_id は引数で受け取らず、実装内部で TenantContext から注入する
    /// </summary>
    Task<IReadOnlyList<T>> FindAsync(Guid aggregateId, CancellationToken cancellationToken = default);

    /// <summary>
    /// Outbox エントリを配信済みにする（Debezium CDC が呼ぶ）
    /// ProcessedAt を現在時刻で更新する
    /// </summary>
    Task MarkDeliveredAsync(Guid id, CancellationToken cancellationToken = default);
}
