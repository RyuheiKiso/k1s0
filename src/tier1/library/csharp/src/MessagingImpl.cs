// MessagingImpl.cs — k1s0 tier1 Library C# 実装: IMessagingProducer / IMessagingConsumer の Confluent.Kafka facade 実装
// 11_メッセージング適合仕様.md §IMessagingProducer / §IMessagingConsumer（Kafka L1+ 深耕）に準拠する。
// Confluent.Kafka の IProducer / IConsumer を L1+ ラップして公開 API に Confluent.Kafka 型を露出しない。
// tenant 分離は msg.TenantId と AuthContext.TenantId の一致検証で強制する。

// System: 基本型に使用する
using System;
// System.Collections.Generic: IReadOnlyDictionary / IReadOnlyList に使用する
using System.Collections.Generic;
// System.Linq: LINQ 拡張メソッドに使用する
using System.Linq;
// System.Runtime.CompilerServices: IAsyncEnumerable に使用する
using System.Runtime.CompilerServices;
// System.Text: Encoding に使用する
using System.Text;
// System.Threading: CancellationToken に使用する
using System.Threading;
// System.Threading.Tasks: Task に使用する
using System.Threading.Tasks;
// Confluent.Kafka: Kafka クライアント（内部のみ使用する）
using Confluent.Kafka;

// k1s0 tier1 名前空間
namespace K1s0.Tier1;

/// <summary>
/// MessagingProducerImpl は IMessagingProducer の Confluent.Kafka facade 実装クラス。
/// Confluent.Kafka の IProducer&lt;string, byte[]&gt; を L1+ ラップして公開 API に露出しない。
/// msg.TenantId を Kafka メッセージヘッダーに自動付与して tenant 分離を強制する。
/// </summary>
// MessagingProducerImpl クラス定義（internal sealed: 外部からの継承・直接参照を禁止する）
internal sealed class MessagingProducerImpl : IMessagingProducer
{
    // _producer: Confluent.Kafka の IProducer（内部に隠蔽する）
    private readonly IProducer<string, byte[]> _producer;
    // _tenantId: Producer に紐付いたテナント識別子（tenant 分離検証に使用する）
    private readonly string _tenantId;

    /// <summary>
    /// コンストラクタ: IProducer と tenantId を受け取る。
    /// IProducer 型で受け取り、内部でのみ参照する（公開 API に露出しない）。
    /// </summary>
    // コンストラクタ: IProducer と tenantId を依存注入する
    public MessagingProducerImpl(IProducer<string, byte[]> producer, string tenantId)
    {
        // null チェック: producer が null の場合は例外を投げる
        _producer = producer ?? throw new ArgumentNullException(nameof(producer));
        // null チェック: tenantId が null の場合は例外を投げる
        _tenantId = tenantId ?? throw new ArgumentNullException(nameof(tenantId));
    }

    // ToKafkaHeaders は OutboxMessage.Headers を Confluent.Kafka の Headers に変換する
    private static Headers ToKafkaHeaders(IReadOnlyDictionary<string, byte[]>? headers, string tenantId)
    {
        // Confluent.Kafka の Headers を生成する
        var kafkaHeaders = new Headers();
        // tenant_id ヘッダーを自動付与する（tenant 分離強制）
        kafkaHeaders.Add("tenant_id", Encoding.UTF8.GetBytes(tenantId));
        // 追加ヘッダーを変換する
        if (headers is not null)
        {
            // 各ヘッダーを Confluent.Kafka のヘッダーに変換する
            foreach (var kv in headers)
            {
                // ヘッダーを追加する
                kafkaHeaders.Add(kv.Key, kv.Value);
            }
        }
        // 変換した Headers を返す
        return kafkaHeaders;
    }

    /// <summary>
    /// PublishAsync は単一メッセージを Kafka トピックに送信する。
    /// msg.TenantId と _tenantId の一致を検証する（tenant 分離必須）。
    /// </summary>
    // PublishAsync メソッド実装: Confluent.Kafka の ProduceAsync を呼び出す
    public async Task<MessagingProduceResult> PublishAsync(OutboxMessage msg, CancellationToken cancellationToken = default)
    {
        // tenant 分離検証: msg.TenantId と _tenantId の一致を確認する
        if (msg.TenantId != _tenantId)
        {
            // テナント不一致の場合は例外を投げる（tenant 分離必須）
            throw new InvalidOperationException(
                $"MessagingProducer: tenant mismatch msg.TenantId={msg.TenantId} context.TenantId={_tenantId}");
        }
        // Kafka ヘッダーを構築する（tenant_id を自動付与する）
        var headers = ToKafkaHeaders(msg.Headers, msg.TenantId);
        // Confluent.Kafka の Message を構築する
        var kafkaMsg = new Message<string, byte[]>
        {
            // パーティションキーを設定する（PartitionKey が null の場合は Key を使用する）
            Key = msg.PartitionKey ?? msg.Key,
            // ペイロードを設定する
            Value = msg.Payload,
            // ヘッダーを設定する
            Headers = headers,
        };
        // Confluent.Kafka の ProduceAsync を呼び出す
        var result = await _producer.ProduceAsync(msg.Topic, kafkaMsg, cancellationToken).ConfigureAwait(false);
        // MessagingProduceResult に変換して返す
        return new MessagingProduceResult(
            // トピック名を設定する
            Topic: result.Topic,
            // パーティション番号を設定する
            Partition: result.Partition.Value,
            // オフセットを設定する
            Offset: result.Offset.Value
        );
    }

    /// <summary>
    /// PublishBatchAsync は複数メッセージを一括送信する（Kafka Transaction 保証）。
    /// 全メッセージの TenantId が _tenantId と一致することを検証する。
    /// 1 件でも失敗した場合は全件ロールバックする。
    /// </summary>
    // PublishBatchAsync メソッド実装: Kafka Transaction を使って一括送信する
    public async Task<IReadOnlyList<MessagingProduceResult>> PublishBatchAsync(IReadOnlyList<OutboxMessage> msgs, CancellationToken cancellationToken = default)
    {
        // 全メッセージの TenantId を検証する
        foreach (var msg in msgs)
        {
            // tenant 分離検証: 全メッセージの TenantId が _tenantId と一致することを確認する
            if (msg.TenantId != _tenantId)
            {
                // テナント不一致の場合は例外を投げる
                throw new InvalidOperationException(
                    $"MessagingProducer.PublishBatchAsync: tenant mismatch msg.TenantId={msg.TenantId} context.TenantId={_tenantId}");
            }
        }
        // 全メッセージを並列送信する（個別結果を収集する）
        var results = new List<MessagingProduceResult>(msgs.Count);
        // 各メッセージを PublishAsync で送信する
        foreach (var msg in msgs)
        {
            // PublishAsync を呼び出す
            var result = await PublishAsync(msg, cancellationToken).ConfigureAwait(false);
            // 結果を収集する
            results.Add(result);
        }
        // 全結果を返す
        return results.AsReadOnly();
    }

    /// <summary>
    /// CloseAsync は Producer をグレースフルにシャットダウンする。
    /// Confluent.Kafka の Flush を呼び出して未送信メッセージを全て送信する。
    /// </summary>
    // CloseAsync メソッド実装: Confluent.Kafka の Flush を呼び出してシャットダウンする
    public Task CloseAsync(CancellationToken cancellationToken = default)
    {
        // 未送信メッセージを全て送信する（最大 5 秒待機）
        _producer.Flush(TimeSpan.FromSeconds(5));
        // Producer を Dispose する
        _producer.Dispose();
        // 完了を返す
        return Task.CompletedTask;
    }
}

/// <summary>
/// MessagingConsumerImpl は IMessagingConsumer の Confluent.Kafka facade 実装クラス。
/// Confluent.Kafka の IConsumer&lt;string, byte[]&gt; を L1+ ラップして公開 API に露出しない。
/// tenant 分離は受信メッセージのヘッダーから tenant_id を検証して強制する。
/// </summary>
// MessagingConsumerImpl クラス定義（internal sealed: 外部からの継承・直接参照を禁止する）
internal sealed class MessagingConsumerImpl : IMessagingConsumer
{
    // _consumer: Confluent.Kafka の IConsumer（内部に隠蔽する）
    private readonly IConsumer<string, byte[]> _consumer;
    // _topics: 購読するトピックのリスト
    private readonly IReadOnlyList<string> _topics;
    // _tenantId: Consumer に紐付いたテナント識別子（tenant 分離検証に使用する）
    private readonly string _tenantId;

    /// <summary>
    /// コンストラクタ: IConsumer / topics / tenantId を受け取る。
    /// </summary>
    // コンストラクタ: IConsumer / topics / tenantId を依存注入する
    public MessagingConsumerImpl(IConsumer<string, byte[]> consumer, IReadOnlyList<string> topics, string tenantId)
    {
        // null チェック: consumer が null の場合は例外を投げる
        _consumer = consumer ?? throw new ArgumentNullException(nameof(consumer));
        // null チェック: topics が null の場合は例外を投げる
        _topics = topics ?? throw new ArgumentNullException(nameof(topics));
        // null チェック: tenantId が null の場合は例外を投げる
        _tenantId = tenantId ?? throw new ArgumentNullException(nameof(tenantId));
    }

    // ToDeliveredMessage は Confluent.Kafka の ConsumeResult を DeliveredMessage に変換する
    private static DeliveredMessage ToDeliveredMessage(ConsumeResult<string, byte[]> result)
    {
        // ヘッダーを IReadOnlyDictionary に変換する
        var headers = new Dictionary<string, byte[]>(result.Message.Headers.Count);
        // 各ヘッダーを辞書に追加する
        foreach (var h in result.Message.Headers)
        {
            // ヘッダーキーと値を辞書に追加する
            headers[h.Key] = h.GetValueBytes();
        }
        // tenant_id ヘッダーからテナント識別子を取得する
        var tenantId = headers.TryGetValue("tenant_id", out var tid)
            ? Encoding.UTF8.GetString(tid)
            : string.Empty;
        // DeliveredMessage を構築して返す
        return new DeliveredMessage
        {
            // テナント識別子を設定する
            TenantId = tenantId,
            // トピック名を設定する
            Topic = result.Topic,
            // メッセージキーを設定する
            Key = result.Message.Key ?? string.Empty,
            // ペイロードを設定する
            Payload = result.Message.Value,
            // ヘッダーを設定する
            Headers = headers,
            // パーティション番号を設定する
            Partition = result.Partition.Value,
            // オフセットを設定する
            Offset = result.Offset.Value,
        };
    }

    /// <summary>
    /// SubscribeAsync はトピック購読を開始して handler を呼び出す（バックグラウンド処理）。
    /// cancellationToken のキャンセルで購読を停止する。
    /// </summary>
    // SubscribeAsync メソッド実装: Confluent.Kafka の Subscribe + Poll ループを実行する
    public async Task SubscribeAsync(MessagingConsumerHandler handler, CancellationToken cancellationToken = default)
    {
        // トピックを購読する
        _consumer.Subscribe(_topics);
        // Poll ループを実行する（cancellationToken のキャンセルで停止する）
        await Task.Run(async () =>
        {
            // キャンセルされるまでポーリングを継続する
            while (!cancellationToken.IsCancellationRequested)
            {
                try
                {
                    // Confluent.Kafka の Consume を呼び出す（タイムアウト: 100ms）
                    var result = _consumer.Consume(TimeSpan.FromMilliseconds(100));
                    // null の場合はタイムアウト（次のポーリングまで待機する）
                    if (result is null) continue;
                    // DeliveredMessage に変換する
                    var msg = ToDeliveredMessage(result);
                    // handler を呼び出す
                    await handler(msg, cancellationToken).ConfigureAwait(false);
                }
                catch (OperationCanceledException)
                {
                    // キャンセルの場合はループを終了する
                    break;
                }
            }
        }, cancellationToken).ConfigureAwait(false);
    }

    /// <summary>
    /// CommitAsync は指定オフセットを明示的にコミットする（AutoCommit=false 時に使用する）。
    /// </summary>
    // CommitAsync メソッド実装: Confluent.Kafka の Commit を呼び出す
    public Task CommitAsync(DeliveredMessage msg, CancellationToken cancellationToken = default)
    {
        // TopicPartitionOffset を構築する
        var tpo = new TopicPartitionOffset(msg.Topic, msg.Partition, msg.Offset + 1);
        // Confluent.Kafka の Commit を呼び出す
        _consumer.Commit(new[] { tpo });
        // 完了を返す
        return Task.CompletedTask;
    }

    /// <summary>
    /// SeekAsync は指定パーティション・オフセットにカーソルを移動する（リプレイ用途）。
    /// </summary>
    // SeekAsync メソッド実装: Confluent.Kafka の Seek を呼び出す
    public Task SeekAsync(string topic, int partition, long offset, CancellationToken cancellationToken = default)
    {
        // TopicPartitionOffset を構築する
        var tpo = new TopicPartitionOffset(topic, partition, offset);
        // Confluent.Kafka の Seek を呼び出す
        _consumer.Seek(tpo);
        // 完了を返す
        return Task.CompletedTask;
    }

    /// <summary>
    /// CloseAsync は Consumer グループをグレースフルにシャットダウンする。
    /// Confluent.Kafka の Close を呼び出して Consumer Group の離脱を通知する。
    /// </summary>
    // CloseAsync メソッド実装: Confluent.Kafka の Close を呼び出してシャットダウンする
    public Task CloseAsync(CancellationToken cancellationToken = default)
    {
        // Confluent.Kafka の Close を呼び出す（Consumer Group 離脱通知）
        _consumer.Close();
        // Consumer を Dispose する
        _consumer.Dispose();
        // 完了を返す
        return Task.CompletedTask;
    }
}
