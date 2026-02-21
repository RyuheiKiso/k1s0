/// DLQ メッセージステータス。
enum DlqStatus {
  pending,
  retrying,
  resolved,
  dead;

  static DlqStatus fromString(String value) {
    return switch (value) {
      'PENDING' => DlqStatus.pending,
      'RETRYING' => DlqStatus.retrying,
      'RESOLVED' => DlqStatus.resolved,
      'DEAD' => DlqStatus.dead,
      _ => throw ArgumentError('Unknown DlqStatus: $value'),
    };
  }

  String toJson() => name.toUpperCase();
}

/// DLQ メッセージ。
class DlqMessage {
  final String id;
  final String originalTopic;
  final String errorMessage;
  final int retryCount;
  final int maxRetries;
  final dynamic payload;
  final DlqStatus status;
  final String createdAt;
  final String? lastRetryAt;

  const DlqMessage({
    required this.id,
    required this.originalTopic,
    required this.errorMessage,
    required this.retryCount,
    required this.maxRetries,
    required this.payload,
    required this.status,
    required this.createdAt,
    this.lastRetryAt,
  });

  factory DlqMessage.fromJson(Map<String, dynamic> json) => DlqMessage(
        id: json['id'] as String,
        originalTopic: json['original_topic'] as String,
        errorMessage: json['error_message'] as String,
        retryCount: json['retry_count'] as int,
        maxRetries: json['max_retries'] as int,
        payload: json['payload'],
        status: DlqStatus.fromString(json['status'] as String),
        createdAt: json['created_at'] as String,
        lastRetryAt: json['last_retry_at'] as String?,
      );
}

/// DLQ メッセージ一覧取得レスポンス。
class ListDlqMessagesResponse {
  final List<DlqMessage> messages;
  final int total;
  final int page;

  const ListDlqMessagesResponse({
    required this.messages,
    required this.total,
    required this.page,
  });

  factory ListDlqMessagesResponse.fromJson(Map<String, dynamic> json) =>
      ListDlqMessagesResponse(
        messages: (json['messages'] as List<dynamic>)
            .map((e) => DlqMessage.fromJson(e as Map<String, dynamic>))
            .toList(),
        total: json['total'] as int,
        page: json['page'] as int,
      );
}

/// DLQ メッセージ再処理レスポンス。
class RetryDlqMessageResponse {
  final String messageId;
  final DlqStatus status;

  const RetryDlqMessageResponse({
    required this.messageId,
    required this.status,
  });

  factory RetryDlqMessageResponse.fromJson(Map<String, dynamic> json) =>
      RetryDlqMessageResponse(
        messageId: json['message_id'] as String,
        status: DlqStatus.fromString(json['status'] as String),
      );
}
