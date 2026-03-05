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
  final String id;
  final String sagaId;
  final int stepIndex;
  final String stepName;
  final String action;
  final String status;
  final Map<String, dynamic>? requestPayload;
  final Map<String, dynamic>? responsePayload;
  final String? errorMessage;
  final String startedAt;
  final String? completedAt;

  const SagaStepLog({
    required this.id,
    required this.sagaId,
    required this.stepIndex,
    required this.stepName,
    required this.action,
    required this.status,
    this.requestPayload,
    this.responsePayload,
    this.errorMessage,
    required this.startedAt,
    this.completedAt,
  });

  factory SagaStepLog.fromJson(Map<String, dynamic> json) => SagaStepLog(
        id: json['id'] as String? ?? '',
        sagaId: json['saga_id'] as String? ?? '',
        stepIndex: json['step_index'] as int? ?? 0,
        stepName: json['step_name'] as String,
        action: json['action'] as String? ?? '',
        status: json['status'] as String,
        requestPayload: json['request_payload'] == null
            ? null
            : Map<String, dynamic>.from(json['request_payload'] as Map),
        responsePayload: json['response_payload'] == null
            ? null
            : Map<String, dynamic>.from(json['response_payload'] as Map),
        errorMessage: json['error_message'] as String?,
        startedAt: json['started_at'] as String? ?? '',
        completedAt: json['completed_at'] as String?,
      );
}

/// Saga の現在状態。
class SagaState {
  final String sagaId;
  final String workflowName;
  final int currentStep;
  final SagaStatus status;
  final Map<String, dynamic> payload;
  final String? correlationId;
  final String? initiatedBy;
  final String? errorMessage;
  final List<SagaStepLog> stepLogs;
  final String createdAt;
  final String updatedAt;

  const SagaState({
    required this.sagaId,
    required this.workflowName,
    required this.currentStep,
    required this.status,
    required this.payload,
    this.correlationId,
    this.initiatedBy,
    this.errorMessage,
    required this.stepLogs,
    required this.createdAt,
    required this.updatedAt,
  });

  factory SagaState.fromJson(Map<String, dynamic> json) => SagaState(
        sagaId: json['saga_id'] as String,
        workflowName: json['workflow_name'] as String,
        currentStep: json['current_step'] as int? ?? 0,
        status: SagaStatus.fromString(json['status'] as String),
        payload: json['payload'] == null
            ? const {}
            : Map<String, dynamic>.from(json['payload'] as Map),
        correlationId: json['correlation_id'] as String?,
        initiatedBy: json['initiated_by'] as String?,
        errorMessage: json['error_message'] as String?,
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
