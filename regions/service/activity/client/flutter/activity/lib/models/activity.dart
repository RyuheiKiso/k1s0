/// アクティビティサービスのデータモデル定義
/// freezed/json_serializableを使用せず、手書きのfromJson/toJsonで実装する
library;

/// アクティビティステータスの列挙型
/// 承認フローにおける各状態を定義する
enum ActivityStatus {
  /// アクティブ: 作成直後の通常状態
  active,
  /// 申請中: 承認フローに投入された状態
  submitted,
  /// 承認済み: 承認者が承認した状態
  approved,
  /// 却下済み: 承認者が却下した状態
  rejected;

  /// 文字列からActivityStatusに変換する
  static ActivityStatus fromString(String value) {
    return ActivityStatus.values.firstWhere(
      (e) => e.name == value,
      orElse: () => ActivityStatus.active,
    );
  }

  /// ステータスの日本語表示名を返す
  String get displayName {
    return switch (this) {
      ActivityStatus.active => 'アクティブ',
      ActivityStatus.submitted => '申請中',
      ActivityStatus.approved => '承認済み',
      ActivityStatus.rejected => '却下済み',
    };
  }
}

/// アクティビティ種別の列挙型
/// アクティビティの内容を分類する
enum ActivityType {
  /// コメント: タスクへのテキストコメント
  comment,
  /// 作業時間記録: 作業に費やした時間の記録
  // ignore: constant_identifier_names
  time_entry,
  /// ステータス変更: タスクのステータス変更の記録
  // ignore: constant_identifier_names
  status_change,
  /// 担当割当: タスクの担当者変更の記録
  assignment;

  /// 文字列からActivityTypeに変換する
  static ActivityType fromString(String value) {
    return ActivityType.values.firstWhere(
      (e) => e.name == value,
      orElse: () => ActivityType.comment,
    );
  }

  /// 種別の日本語表示名を返す
  String get displayName {
    return switch (this) {
      ActivityType.comment => 'コメント',
      ActivityType.time_entry => '作業時間',
      ActivityType.status_change => 'ステータス変更',
      ActivityType.assignment => '担当割当',
    };
  }
}

/// アクティビティモデル
/// タスクへのコメント・作業時間・ステータス変更を記録するエンティティ
class Activity {
  final String id;
  final String taskId;
  final String actorId;
  final ActivityType activityType;
  final String? content;
  final int? durationMinutes;
  final ActivityStatus status;
  final Map<String, dynamic>? metadata;
  final String? idempotencyKey;
  final int version;
  final DateTime createdAt;
  final DateTime updatedAt;

  const Activity({
    required this.id,
    required this.taskId,
    required this.actorId,
    required this.activityType,
    this.content,
    this.durationMinutes,
    required this.status,
    this.metadata,
    this.idempotencyKey,
    required this.version,
    required this.createdAt,
    required this.updatedAt,
  });

  /// JSONマップからActivityインスタンスを生成する
  factory Activity.fromJson(Map<String, dynamic> json) {
    return Activity(
      id: json['id'] as String,
      taskId: json['task_id'] as String,
      actorId: json['actor_id'] as String,
      activityType: ActivityType.fromString(json['activity_type'] as String),
      content: json['content'] as String?,
      durationMinutes: json['duration_minutes'] as int?,
      status: ActivityStatus.fromString(json['status'] as String),
      metadata: json['metadata'] as Map<String, dynamic>?,
      idempotencyKey: json['idempotency_key'] as String?,
      version: json['version'] as int,
      createdAt: DateTime.parse(json['created_at'] as String),
      updatedAt: DateTime.parse(json['updated_at'] as String),
    );
  }

  /// ActivityインスタンスをJSONマップに変換する
  Map<String, dynamic> toJson() {
    return {
      'id': id,
      'task_id': taskId,
      'actor_id': actorId,
      'activity_type': activityType.name,
      'content': content,
      'duration_minutes': durationMinutes,
      'status': status.name,
      'metadata': metadata,
      'idempotency_key': idempotencyKey,
      'version': version,
      'created_at': createdAt.toIso8601String(),
      'updated_at': updatedAt.toIso8601String(),
    };
  }
}

/// アクティビティ作成時の入力モデル
/// 新規アクティビティ作成に必要なフィールドのみを定義する
class CreateActivityInput {
  final String taskId;
  final String actorId;
  final ActivityType activityType;
  final String? content;
  final int? durationMinutes;
  final Map<String, dynamic>? metadata;
  final String? idempotencyKey;

  const CreateActivityInput({
    required this.taskId,
    required this.actorId,
    required this.activityType,
    this.content,
    this.durationMinutes,
    this.metadata,
    this.idempotencyKey,
  });

  /// 入力データをAPIリクエスト用のJSONマップに変換する
  Map<String, dynamic> toJson() {
    return {
      'task_id': taskId,
      'actor_id': actorId,
      'activity_type': activityType.name,
      if (content != null) 'content': content,
      if (durationMinutes != null) 'duration_minutes': durationMinutes,
      if (metadata != null) 'metadata': metadata,
      if (idempotencyKey != null) 'idempotency_key': idempotencyKey,
    };
  }
}

/// アクティビティ却下時の入力モデル
/// 却下理由は任意フィールドとして定義する
class RejectActivityInput {
  final String? reason;

  const RejectActivityInput({this.reason});

  /// 入力データをAPIリクエスト用のJSONマップに変換する
  Map<String, dynamic> toJson() {
    return {
      if (reason != null) 'reason': reason,
    };
  }
}
