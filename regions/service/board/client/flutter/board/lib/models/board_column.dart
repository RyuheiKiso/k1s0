/// ボードサービスのデータモデル定義
/// freezed/json_serializableを使用せず、手書きのfromJson/toJsonで実装する
library;

/// BoardColumnモデル
/// KanbanボードのカラムをWIP制限付きで表す
class BoardColumn {
  final String id;
  final String projectId;
  final String statusCode;
  final int wipLimit;
  final int taskCount;
  final int version;
  final DateTime createdAt;
  final DateTime updatedAt;

  const BoardColumn({
    required this.id,
    required this.projectId,
    required this.statusCode,
    required this.wipLimit,
    required this.taskCount,
    required this.version,
    required this.createdAt,
    required this.updatedAt,
  });

  /// JSONマップからBoardColumnインスタンスを生成する
  factory BoardColumn.fromJson(Map<String, dynamic> json) {
    return BoardColumn(
      id: json['id'] as String,
      projectId: json['project_id'] as String,
      statusCode: json['status_code'] as String,
      wipLimit: json['wip_limit'] as int,
      taskCount: json['task_count'] as int,
      version: json['version'] as int,
      createdAt: DateTime.parse(json['created_at'] as String),
      updatedAt: DateTime.parse(json['updated_at'] as String),
    );
  }

  /// BoardColumnインスタンスをJSONマップに変換する
  Map<String, dynamic> toJson() {
    return {
      'id': id,
      'project_id': projectId,
      'status_code': statusCode,
      'wip_limit': wipLimit,
      'task_count': taskCount,
      'version': version,
      'created_at': createdAt.toIso8601String(),
      'updated_at': updatedAt.toIso8601String(),
    };
  }

  /// WIPゲージの使用率を0.0〜1.0で返す（WIP制限0は無制限扱いで0.0を返す）
  double get wipUsageRatio {
    if (wipLimit <= 0) return 0.0;
    return (taskCount / wipLimit).clamp(0.0, 1.0);
  }

  /// WIP制限を超過しているかどうかを返す
  bool get isOverWipLimit {
    if (wipLimit <= 0) return false;
    return taskCount >= wipLimit;
  }
}

/// タスク数インクリメント時の入力モデル
class IncrementColumnInput {
  final String projectId;
  final String statusCode;

  const IncrementColumnInput({
    required this.projectId,
    required this.statusCode,
  });

  /// 入力データをAPIリクエスト用のJSONマップに変換する
  Map<String, dynamic> toJson() {
    return {
      'project_id': projectId,
      'status_code': statusCode,
    };
  }
}

/// タスク数デクリメント時の入力モデル
class DecrementColumnInput {
  final String projectId;
  final String statusCode;

  const DecrementColumnInput({
    required this.projectId,
    required this.statusCode,
  });

  /// 入力データをAPIリクエスト用のJSONマップに変換する
  Map<String, dynamic> toJson() {
    return {
      'project_id': projectId,
      'status_code': statusCode,
    };
  }
}

/// WIP制限更新時の入力モデル
class UpdateWipLimitInput {
  final int wipLimit;

  const UpdateWipLimitInput({
    required this.wipLimit,
  });

  /// 更新データをAPIリクエスト用のJSONマップに変換する
  Map<String, dynamic> toJson() {
    return {
      'wip_limit': wipLimit,
    };
  }
}
