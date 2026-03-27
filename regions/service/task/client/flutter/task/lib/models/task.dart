/// タスクサービスのデータモデル定義
/// freezed/json_serializableを使用せず、手書きのfromJson/toJsonで実装する
library;

/// タスクステータスの列挙型
/// タスクのライフサイクルにおける各状態を定義する
enum TaskStatus {
  /// オープン: タスクが作成された直後の状態
  open,

  /// 進行中: タスクが作業中の状態
  inProgress,

  /// レビュー中: タスクがレビュー待ちの状態
  review,

  /// 完了: タスクが完了した状態
  done,

  /// キャンセル: タスクがキャンセルされた状態
  cancelled;

  /// 文字列からTaskStatusに変換する
  static TaskStatus fromString(String value) {
    return switch (value) {
      'open' => TaskStatus.open,
      'in_progress' => TaskStatus.inProgress,
      'review' => TaskStatus.review,
      'done' => TaskStatus.done,
      'cancelled' => TaskStatus.cancelled,
      _ => TaskStatus.open,
    };
  }

  /// APIリクエスト用の文字列表現を返す
  String get apiValue {
    return switch (this) {
      TaskStatus.open => 'open',
      TaskStatus.inProgress => 'in_progress',
      TaskStatus.review => 'review',
      TaskStatus.done => 'done',
      TaskStatus.cancelled => 'cancelled',
    };
  }

  /// ステータスの日本語表示名を返す
  String get displayName {
    return switch (this) {
      TaskStatus.open => 'オープン',
      TaskStatus.inProgress => '進行中',
      TaskStatus.review => 'レビュー中',
      TaskStatus.done => '完了',
      TaskStatus.cancelled => 'キャンセル',
    };
  }
}

/// タスク優先度の列挙型
/// タスクの緊急度・重要度を表す
enum TaskPriority {
  /// 低優先度
  low,

  /// 中優先度
  medium,

  /// 高優先度
  high,

  /// 緊急: 最も高い優先度
  critical;

  /// 文字列からTaskPriorityに変換する
  static TaskPriority fromString(String value) {
    return TaskPriority.values.firstWhere(
      (e) => e.name == value,
      orElse: () => TaskPriority.medium,
    );
  }

  /// 優先度の日本語表示名を返す
  String get displayName {
    return switch (this) {
      TaskPriority.low => '低',
      TaskPriority.medium => '中',
      TaskPriority.high => '高',
      TaskPriority.critical => '緊急',
    };
  }
}

/// タスクモデル
/// タスク管理の中心エンティティを表す
class Task {
  final String id;
  final String projectId;
  final String title;
  final String? description;
  final TaskStatus status;
  final TaskPriority priority;
  final String? assigneeId;
  final String reporterId;
  final String? dueDate;
  final List<String> labels;
  final String createdBy;
  final String updatedBy;
  final int version;
  final DateTime createdAt;
  final DateTime updatedAt;

  const Task({
    required this.id,
    required this.projectId,
    required this.title,
    this.description,
    required this.status,
    required this.priority,
    this.assigneeId,
    required this.reporterId,
    this.dueDate,
    required this.labels,
    required this.createdBy,
    required this.updatedBy,
    required this.version,
    required this.createdAt,
    required this.updatedAt,
  });

  /// JSONマップからTaskインスタンスを生成する
  factory Task.fromJson(Map<String, dynamic> json) {
    return Task(
      id: json['id'] as String,
      projectId: json['project_id'] as String,
      title: json['title'] as String,
      description: json['description'] as String?,
      status: TaskStatus.fromString(json['status'] as String),
      priority: TaskPriority.fromString(json['priority'] as String),
      assigneeId: json['assignee_id'] as String?,
      reporterId: json['reporter_id'] as String,
      dueDate: json['due_date'] as String?,
      // labels フィールドが null の場合は空リストを使用し、NullPointerException を防ぐ
      labels: (json['labels'] as List<dynamic>?)?.cast<String>() ?? [],
      createdBy: json['created_by'] as String,
      updatedBy: json['updated_by'] as String,
      version: json['version'] as int,
      createdAt: DateTime.parse(json['created_at'] as String),
      updatedAt: DateTime.parse(json['updated_at'] as String),
    );
  }

  /// TaskインスタンスをJSONマップに変換する
  Map<String, dynamic> toJson() {
    return {
      'id': id,
      'project_id': projectId,
      'title': title,
      'description': description,
      'status': status.apiValue,
      'priority': priority.name,
      'assignee_id': assigneeId,
      'reporter_id': reporterId,
      'due_date': dueDate,
      'labels': labels,
      'created_by': createdBy,
      'updated_by': updatedBy,
      'version': version,
      'created_at': createdAt.toIso8601String(),
      'updated_at': updatedAt.toIso8601String(),
    };
  }
}

/// タスク作成時の入力モデル
/// 新規タスク作成に必要なフィールドのみを定義する
class CreateTaskInput {
  final String projectId;
  final String title;
  final String? description;
  final TaskPriority? priority;
  final String? assigneeId;
  final String? dueDate;
  final List<String>? labels;

  const CreateTaskInput({
    required this.projectId,
    required this.title,
    this.description,
    this.priority,
    this.assigneeId,
    this.dueDate,
    this.labels,
  });

  /// 入力データをAPIリクエスト用のJSONマップに変換する
  Map<String, dynamic> toJson() {
    return {
      'project_id': projectId,
      'title': title,
      if (description != null) 'description': description,
      if (priority != null) 'priority': priority!.name,
      if (assigneeId != null) 'assignee_id': assigneeId,
      if (dueDate != null) 'due_date': dueDate,
      if (labels != null) 'labels': labels,
    };
  }
}

/// タスクステータス更新時の入力モデル
/// 更新対象のステータスのみを定義する
class UpdateTaskStatusInput {
  final TaskStatus status;

  const UpdateTaskStatusInput({
    required this.status,
  });

  /// 更新データをAPIリクエスト用のJSONマップに変換する
  Map<String, dynamic> toJson() {
    return {
      'status': status.apiValue,
    };
  }
}
