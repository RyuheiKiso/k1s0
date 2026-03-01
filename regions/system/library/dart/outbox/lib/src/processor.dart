import 'dart:async';

import 'store.dart';
import 'message.dart';

/// アウトボックスメッセージを定期的に処理する。
class OutboxProcessor {
  final OutboxStore store;
  final OutboxPublisher publisher;
  final int batchSize;

  OutboxProcessor(this.store, this.publisher, {int batchSize = 100})
      : batchSize = batchSize <= 0 ? 100 : batchSize;

  /// 保留中のメッセージを一括処理する。
  Future<int> processBatch() async {
    final messages = await store.fetchPending(batchSize);
    if (messages.isEmpty) return 0;

    var processed = 0;
    for (final msg in messages) {
      msg.markProcessing();
      await store.update(msg);

      try {
        await publisher.publish(msg);
        msg.markDelivered();
        await store.update(msg);
        processed++;
      } on Exception catch (e) {
        msg.markFailed(e.toString());
        await store.update(msg);
      }
    }

    return processed;
  }

  /// interval 間隔でバッチ処理を継続実行する。stopSignal でキャンセル可能。
  Future<void> run(Duration interval, {Future<void>? stopSignal}) async {
    var stopped = false;
    stopSignal?.then((_) => stopped = true);

    while (!stopped) {
      try {
        await processBatch();
      } on Exception catch (_) {
        // log error in production; continue loop
      }
      await Future.any([
        Future<void>.delayed(interval),
        if (stopSignal != null) stopSignal,
      ]);
    }
  }
}
