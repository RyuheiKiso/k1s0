/// Saga の実行ステータス。
enum SagaStatus {
  started,
  running,
  completed,
  compensating,
  failed,
  cancelled;

  static SagaStatus fromString(String value) {
    return switch (value) {
      'STARTED' => SagaStatus.started,
      'RUNNING' => SagaStatus.running,
      'COMPLETED' => SagaStatus.completed,
      'COMPENSATING' => SagaStatus.compensating,
      'FAILED' => SagaStatus.failed,
      'CANCELLED' => SagaStatus.cancelled,
      _ => throw ArgumentError('Unknown SagaStatus: $value'),
    };
  }

  String toJson() => name.toUpperCase();
}

/// Saga ステップのログ。
class SagaStepLog {
  final String stepName;
  final String status;
  final String message;
  final String createdAt;

  const SagaStepLog({
    required this.stepName,
    required this.status,
    required this.message,
    required this.createdAt,
  });

  factory SagaStepLog.fromJson(Map<String, dynamic> json) => SagaStepLog(
        stepName: json['step_name'] as String,
        status: json['status'] as String,
        message: json['message'] as String,
        createdAt: json['created_at'] as String,
      );
}

/// Saga の現在状態。
class SagaState {
  final String sagaId;
  final String workflowName;
  final SagaStatus status;
  final List<SagaStepLog> stepLogs;
  final String createdAt;
  final String updatedAt;

  const SagaState({
    required this.sagaId,
    required this.workflowName,
    required this.status,
    required this.stepLogs,
    required this.createdAt,
    required this.updatedAt,
  });

  factory SagaState.fromJson(Map<String, dynamic> json) => SagaState(
        sagaId: json['saga_id'] as String,
        workflowName: json['workflow_name'] as String,
        status: SagaStatus.fromString(json['status'] as String),
        stepLogs: (json['step_logs'] as List<dynamic>? ?? [])
            .map((e) => SagaStepLog.fromJson(e as Map<String, dynamic>))
            .toList(),
        createdAt: json['created_at'] as String,
        updatedAt: json['updated_at'] as String,
      );
}

/// Saga 開始リクエスト。
class StartSagaRequest {
  final String workflowName;
  final Map<String, dynamic> payload;
  final String? correlationId;
  final String? initiatedBy;

  const StartSagaRequest({
    required this.workflowName,
    required this.payload,
    this.correlationId,
    this.initiatedBy,
  });

  Map<String, dynamic> toJson() => {
        'workflow_name': workflowName,
        'payload': payload,
        if (correlationId != null) 'correlation_id': correlationId,
        if (initiatedBy != null) 'initiated_by': initiatedBy,
      };
}

/// Saga 開始レスポンス。
class StartSagaResponse {
  final String sagaId;

  const StartSagaResponse({required this.sagaId});

  factory StartSagaResponse.fromJson(Map<String, dynamic> json) =>
      StartSagaResponse(sagaId: json['saga_id'] as String);
}
