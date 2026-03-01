import 'message.dart';

/// アウトボックスメッセージの永続化インターフェース。
abstract class OutboxStore {
  Future<void> save(OutboxMessage msg);
  Future<List<OutboxMessage>> fetchPending(int limit);
  Future<void> update(OutboxMessage msg);
  Future<int> deleteDelivered(int olderThanDays);
}

/// メッセージを外部に送信するインターフェース。
abstract class OutboxPublisher {
  Future<void> publish(OutboxMessage msg);
}
