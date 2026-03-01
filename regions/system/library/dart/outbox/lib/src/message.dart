import 'package:uuid/uuid.dart';

const _uuid = Uuid();

enum OutboxStatus { pending, processing, delivered, failed, deadLetter }

class OutboxMessage {
  final String id;
  final String topic;
  final String partitionKey;
  final String payload;
  OutboxStatus status;
  int retryCount;
  int maxRetries;
  String? lastError;
  final DateTime createdAt;
  DateTime processAfter;

  OutboxMessage({
    required this.id,
    required this.topic,
    required this.partitionKey,
    required this.payload,
    required this.status,
    required this.retryCount,
    required this.maxRetries,
    this.lastError,
    required this.createdAt,
    required this.processAfter,
  });

  /// メッセージを処理中状態に遷移する。
  void markProcessing() {
    status = OutboxStatus.processing;
  }

  /// メッセージを配信完了状態に遷移する。
  void markDelivered() {
    status = OutboxStatus.delivered;
  }

  /// メッセージを失敗状態に遷移し、リトライ回数をインクリメントする。
  void markFailed(String error) {
    retryCount += 1;
    lastError = error;
    if (retryCount >= maxRetries) {
      status = OutboxStatus.deadLetter;
    } else {
      status = OutboxStatus.failed;
      // Exponential backoff: 2^retryCount 秒後に再処理
      final delaySecs = 1 << retryCount; // 2^retryCount
      processAfter = DateTime.now().toUtc().add(Duration(seconds: delaySecs));
    }
  }

  /// メッセージが処理可能かどうか判定する。
  bool get isProcessable {
    return (status == OutboxStatus.pending || status == OutboxStatus.failed) &&
        !processAfter.isAfter(DateTime.now().toUtc());
  }
}

/// 新しい OutboxMessage を生成する。
OutboxMessage createOutboxMessage(
  String topic,
  String partitionKey,
  String payload,
) {
  final now = DateTime.now().toUtc();
  return OutboxMessage(
    id: _uuid.v4(),
    topic: topic,
    partitionKey: partitionKey,
    payload: payload,
    status: OutboxStatus.pending,
    retryCount: 0,
    maxRetries: 3,
    lastError: null,
    createdAt: now,
    processAfter: now,
  );
}

/// 現在のステータスから目的のステータスへ遷移可能かを返す。
bool canTransitionTo(OutboxStatus from, OutboxStatus to) {
  switch (from) {
    case OutboxStatus.pending:
      return to == OutboxStatus.processing;
    case OutboxStatus.processing:
      return to == OutboxStatus.delivered ||
          to == OutboxStatus.failed ||
          to == OutboxStatus.deadLetter;
    case OutboxStatus.failed:
      return to == OutboxStatus.processing;
    case OutboxStatus.delivered:
      return false;
    case OutboxStatus.deadLetter:
      return false;
  }
}
