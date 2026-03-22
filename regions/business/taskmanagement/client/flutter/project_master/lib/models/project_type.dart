/// プロジェクトタイプモデル定義
/// タスク管理プロジェクトマスタのプロジェクトタイプを表す
library;

/// プロジェクトタイプモデル
/// プロジェクトの種類を表す（例：ソフトウェア開発、インフラ構築など）
class ProjectType {
  final String id;
  final String code;
  final String displayName;
  final String? description;
  final String? defaultWorkflow;
  final bool isActive;
  final int sortOrder;
  final String createdBy;
  final DateTime createdAt;
  final DateTime updatedAt;

  const ProjectType({
    required this.id,
    required this.code,
    required this.displayName,
    this.description,
    this.defaultWorkflow,
    required this.isActive,
    required this.sortOrder,
    required this.createdBy,
    required this.createdAt,
    required this.updatedAt,
  });

  /// JSONマップからProjectTypeインスタンスを生成する
  factory ProjectType.fromJson(Map<String, dynamic> json) {
    return ProjectType(
      id: json['id'] as String,
      code: json['code'] as String,
      displayName: json['display_name'] as String,
      description: json['description'] as String?,
      defaultWorkflow: json['default_workflow'] as String?,
      isActive: json['is_active'] as bool,
      sortOrder: json['sort_order'] as int,
      createdBy: json['created_by'] as String,
      createdAt: DateTime.parse(json['created_at'] as String),
      updatedAt: DateTime.parse(json['updated_at'] as String),
    );
  }

  /// ProjectTypeインスタンスをJSONマップに変換する
  Map<String, dynamic> toJson() {
    return {
      'id': id,
      'code': code,
      'display_name': displayName,
      'description': description,
      'default_workflow': defaultWorkflow,
      'is_active': isActive,
      'sort_order': sortOrder,
      'created_by': createdBy,
      'created_at': createdAt.toIso8601String(),
      'updated_at': updatedAt.toIso8601String(),
    };
  }
}

/// プロジェクトタイプ作成時の入力モデル
/// 新規プロジェクトタイプ作成に必要なフィールドのみを定義する
class CreateProjectTypeInput {
  final String code;
  final String displayName;
  final String? description;
  final String? defaultWorkflow;
  final bool isActive;
  final int sortOrder;

  const CreateProjectTypeInput({
    required this.code,
    required this.displayName,
    this.description,
    this.defaultWorkflow,
    this.isActive = true,
    this.sortOrder = 0,
  });

  /// 入力データをAPIリクエスト用のJSONマップに変換する
  Map<String, dynamic> toJson() {
    return {
      'code': code,
      'display_name': displayName,
      'description': description,
      'default_workflow': defaultWorkflow,
      'is_active': isActive,
      'sort_order': sortOrder,
    };
  }
}

/// プロジェクトタイプ更新時の入力モデル
/// 更新可能なフィールドのみを定義する
class UpdateProjectTypeInput {
  final String? displayName;
  final String? description;
  final String? defaultWorkflow;
  final bool? isActive;
  final int? sortOrder;

  const UpdateProjectTypeInput({
    this.displayName,
    this.description,
    this.defaultWorkflow,
    this.isActive,
    this.sortOrder,
  });

  /// 更新データをAPIリクエスト用のJSONマップに変換する
  /// nullのフィールドは除外する
  Map<String, dynamic> toJson() {
    final json = <String, dynamic>{};
    if (displayName != null) json['display_name'] = displayName;
    if (description != null) json['description'] = description;
    if (defaultWorkflow != null) json['default_workflow'] = defaultWorkflow;
    if (isActive != null) json['is_active'] = isActive;
    if (sortOrder != null) json['sort_order'] = sortOrder;
    return json;
  }
}
