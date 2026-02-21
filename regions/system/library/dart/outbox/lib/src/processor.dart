import 'dart:async';

import 'store.dart';
import 'message.dart';
import 'error.dart';

class OutboxProcessor {
  final OutboxStore store;
  final OutboxPublisher publisher;
  final int batchSize;

  OutboxProcessor(this.store, this.publisher, {int batchSize = 100})
      : batchSize = batchSize <= 0 ? 100 : batchSize;

  Future<int> processBatch() async {
    final List<OutboxMessage> messages;
    try {
      messages = await store.getPendingMessages(batchSize);
    } on Exception catch (e) {
      throw OutboxError('GetPendingMessages', cause: e);
    }

    if (messages.isEmpty) return 0;

    var processed = 0;
    for (final msg in messages) {
      await store.updateStatus(msg.id, OutboxStatus.processing);

      try {
        await publisher.publish(msg);
        await store.updateStatus(msg.id, OutboxStatus.delivered);
        processed++;
      } on Exception catch (_) {
        final nextRetry = msg.retryCount + 1;
        final scheduledAt = nextScheduledAt(nextRetry);
        await store.updateStatusWithRetry(
          msg.id,
          OutboxStatus.failed,
          nextRetry,
          scheduledAt,
        );
      }
    }

    return processed;
  }

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
