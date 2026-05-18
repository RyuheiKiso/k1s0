// IMessaging.cs — k1s0 tier1 Library C# 実装: Messaging / EventBus の L1+ interface
// 11_メッセージング適合仕様.md §IMessagingProducer / §IMessagingConsumer（Kafka L1+ 深耕）に準拠する。
// Kafka の full API を Library 独自語彙で表現しつつ、AuthContext 伝播を強制する。
// OSS 型（Confluent.Kafka 等）を公開シグネチャに一切含まない。

// System: 基本型に使用する
using System;
// System.Collections.Generic: IReadOnlyDictionary / IReadOnlyList に使用する
using System.Collections.Generic;
// System.Runtime.CompilerServices: IAsyncEnumerable に使用する
using System.Runtime.CompilerServices;
// System.Threading: CancellationToken に使用する
using System.Threading;
// System.Threading.Tasks: Task / ValueTask に使用する
using System.Threading.Tasks;

// k1s0 tier1 名前空間
namespace K1s0.Tier1;

/// <summary>
/// OutboxMessage は Messaging L1+ (Kafka) の Outbox メッセージを宣言する型。
/// 11_メッセージング適合仕様.md §Outbox Pattern の必須フィールドに準拠する。
/// tenant 分離を保証するために TenantId を必須フィールドとして持つ。
/// </summary>
// OutboxMessage クラス定義
public sealed class OutboxMessage
{
    /// <summary>TenantId: メッセージの発行元テナント識別子（必須: partition routing に使用する）</summary>
    // TenantId プロパティ（必須）
    public required string TenantId { get; init; }

    /// <summary>Topic: Kafka トピック名（"{TenantId}.{事業ドメイン}" 形式を推奨する）</summary>
    // Topic プロパティ（必須）
    public required string Topic { get; init; }

    /// <summary>Key: Kafka メッセージキー（同一エンティティのメッセージ順序保証に使用する）</summary>
    // Key プロパティ（必須）
    public required string Key { get; init; }

    /// <summary>Payload: メッセージボディ（protobuf / JSON バイト列）</summary>
    // Payload プロパティ（必須）
    public required byte[] Payload { get; init; }

    /// <summary>Headers: Kafka メッセージヘッダー（trace_id / auth_class 等の横断属性）</summary>
    // Headers プロパティ
    public IReadOnlyDictionary<string, byte[]>? Headers { get; init; }

    /// <summary>PartitionKey: Kafka パーティションキー（null = Key を使用する）</summary>
    // PartitionKey プロパティ
    public string? PartitionKey { get; init; }

    /// <summary>SchemaId: Schema Registry で登録されたスキーマ ID（0 = スキーマ検証なし）</summary>
    // SchemaId プロパティ
    public long SchemaId { get; init; }
}

/// <summary>
/// DeliveredMessage は Messaging L1+ (Kafka) で受信したメッセージを宣言する型。
/// Consumer が Kafka から受け取ったメッセージを Library 独自語彙で表現する。
/// </summary>
// DeliveredMessage クラス定義
public sealed class DeliveredMessage
{
    /// <summary>TenantId: メッセージの発行元テナント識別子</summary>
    // TenantId プロパティ（必須）
    public required string TenantId { get; init; }

    /// <summary>Topic: 受信したトピック名</summary>
    // Topic プロパティ（必須）
    public required string Topic { get; init; }

    /// <summary>Key: メッセージキー</summary>
    // Key プロパティ（必須）
    public required string Key { get; init; }

    /// <summary>Payload: メッセージボディ</summary>
    // Payload プロパティ（必須）
    public required byte[] Payload { get; init; }

    /// <summary>Headers: メッセージヘッダー（必須）</summary>
    // Headers プロパティ（必須）
    public required IReadOnlyDictionary<string, byte[]> Headers { get; init; }

    /// <summary>Partition: 受信したパーティション番号</summary>
    // Partition プロパティ
    public int Partition { get; init; }

    /// <summary>Offset: 受信したオフセット</summary>
    // Offset プロパティ
    public long Offset { get; init; }

    /// <summary>SchemaId: Schema Registry のスキーマ ID</summary>
    // SchemaId プロパティ
    public long SchemaId { get; init; }
}

/// <summary>
/// MessagingProduceResult は PublishAsync の結果を宣言する型。
/// </summary>
// MessagingProduceResult レコード定義
public sealed record MessagingProduceResult(
    string Topic,
    int Partition,
    long Offset
);

/// <summary>
/// IMessagingProducer は Messaging L1+ (Kafka) の Producer interface を宣言する。
/// Kafka の full API を Library 独自語彙で表現する。
/// AuthContext が伝播されていることを前提とする（tenant 分離必須）。
/// OSS 型（Confluent.Kafka.IProducer 等）を一切含まない。
/// </summary>
// IMessagingProducer インターフェース定義
public interface IMessagingProducer
{
    /// <summary>
    /// PublishAsync は単一メッセージを Kafka トピックに送信する。
    /// msg.TenantId と AuthContext.TenantId の一致を実装側で検証する。
    /// </summary>
    // PublishAsync メソッド: メッセージを送信する
    Task<MessagingProduceResult> PublishAsync(OutboxMessage msg, CancellationToken cancellationToken = default);

    /// <summary>
    /// PublishBatchAsync は複数メッセージを一括送信する（Transaction Producer を使用する）。
    /// 全メッセージの TenantId が AuthContext.TenantId と一致することを検証する。
    /// 1 件でも失敗した場合は全件ロールバックする（Kafka Transaction 保証）。
    /// </summary>
    // PublishBatchAsync メソッド: 複数メッセージを一括送信する
    Task<IReadOnlyList<MessagingProduceResult>> PublishBatchAsync(IReadOnlyList<OutboxMessage> msgs, CancellationToken cancellationToken = default);

    /// <summary>
    /// CloseAsync は Producer をグレースフルにシャットダウンする。
    /// </summary>
    // CloseAsync メソッド: Producer をシャットダウンする
    Task CloseAsync(CancellationToken cancellationToken = default);
}

/// <summary>
/// IMessagingConsumerHandler は Consumer が受信したメッセージを処理するデリゲート型。
/// </summary>
// IMessagingConsumerHandler デリゲート定義
public delegate Task MessagingConsumerHandler(DeliveredMessage msg, CancellationToken cancellationToken);

/// <summary>
/// IMessagingConsumer は Messaging L1+ (Kafka) の Consumer interface を宣言する。
/// Kafka Consumer Group の full API を Library 独自語彙で表現する。
/// OSS 型（Confluent.Kafka.IConsumer 等）を一切含まない。
/// </summary>
// IMessagingConsumer インターフェース定義
public interface IMessagingConsumer
{
    /// <summary>
    /// SubscribeAsync はトピック購読を開始して handler を呼び出す（バックグラウンド処理）。
    /// cancellationToken のキャンセルで購読を停止する。
    /// </summary>
    // SubscribeAsync メソッド: トピック購読を開始する
    Task SubscribeAsync(MessagingConsumerHandler handler, CancellationToken cancellationToken = default);

    /// <summary>
    /// CommitAsync は指定オフセットを明示的にコミットする（AutoCommit=false 時に使用する）。
    /// </summary>
    // CommitAsync メソッド: オフセットをコミットする
    Task CommitAsync(DeliveredMessage msg, CancellationToken cancellationToken = default);

    /// <summary>
    /// SeekAsync は指定パーティション・オフセットにカーソルを移動する（リプレイ用途）。
    /// </summary>
    // SeekAsync メソッド: パーティション・オフセットにシークする
    Task SeekAsync(string topic, int partition, long offset, CancellationToken cancellationToken = default);

    /// <summary>
    /// CloseAsync は Consumer グループをグレースフルにシャットダウンする。
    /// </summary>
    // CloseAsync メソッド: Consumer をシャットダウンする
    Task CloseAsync(CancellationToken cancellationToken = default);
}

/// <summary>
/// IOutboxRelay は Outbox Pattern の中継 interface を宣言する。
/// DB Outbox テーブルから Kafka に at-least-once でメッセージを転送する。
/// </summary>
// IOutboxRelay インターフェース定義
public interface IOutboxRelay
{
    /// <summary>
    /// PollAsync は Outbox テーブルから未送信メッセージを取得して Kafka に転送する。
    /// cancellationToken のキャンセルでポーリングを停止する。
    /// </summary>
    // PollAsync メソッド: Outbox メッセージを転送する
    Task PollAsync(CancellationToken cancellationToken = default);

    /// <summary>
    /// StopAsync はポーリングを停止してグレースフルシャットダウンする。
    /// </summary>
    // StopAsync メソッド: ポーリングを停止する
    Task StopAsync(CancellationToken cancellationToken = default);
}
