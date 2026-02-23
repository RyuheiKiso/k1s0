enum IdempotencyStatus { pending, completed, failed }

class IdempotencyRecord {
  final String key;
  IdempotencyStatus status;
  String? responseBody;
  int? statusCode;
  final DateTime createdAt;
  final DateTime? expiresAt;
  DateTime? completedAt;

  IdempotencyRecord({
    required this.key,
    this.status = IdempotencyStatus.pending,
    this.responseBody,
    this.statusCode,
    DateTime? createdAt,
    this.expiresAt,
    this.completedAt,
  }) : createdAt = createdAt ?? DateTime.now();

  factory IdempotencyRecord.create(String key, {int? ttlSecs}) {
    final now = DateTime.now();
    return IdempotencyRecord(
      key: key,
      createdAt: now,
      expiresAt:
          ttlSecs != null ? now.add(Duration(seconds: ttlSecs)) : null,
    );
  }
}
