import 'message.dart';

abstract class OutboxStore {
  Future<void> saveMessage(OutboxMessage msg);
  Future<List<OutboxMessage>> getPendingMessages(int limit);
  Future<void> updateStatus(String id, OutboxStatus status);
  Future<void> updateStatusWithRetry(
    String id,
    OutboxStatus status,
    int retryCount,
    DateTime scheduledAt,
  );
}

abstract class OutboxPublisher {
  Future<void> publish(OutboxMessage msg);
}
