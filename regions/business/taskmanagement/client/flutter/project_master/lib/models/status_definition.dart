/// ステータス定義モデル定義
/// プロジェクトタイプに属するステータスとそのバージョン履歴を表す
library;

/// ステータス定義モデル
/// プロジェクトタイプに紐づくワークフローのステータスを定義する
class StatusDefinition {
  final String id;
  final String projectTypeId;
  final String code;
  final String displayName;
  final String? description;
  final String? color;
  final List<String>? allowedTransitions;
  final bool isInitial;
  final bool isTerminal;
  final int sortOrder;
  final String createdBy;
  final DateTime createdAt;
  final DateTime updatedAt;

  const StatusDefinition({
    required this.id,
    required this.projectTypeId,
    required this.code,
    required this.displayName,
    this.description,
    this.color,
    this.allowedTransitions,
    required this.isInitial,
    required this.isTerminal,
    required this.sortOrder,
    required this.createdBy,
    required this.createdAt,
    required this.updatedAt,
  });

  /// JSONマップからStatusDefinitionインスタンスを生成する
  factory StatusDefinition.fromJson(Map<String, dynamic> json) {
    return StatusDefinition(
      id: json['id'] as String,
      projectTypeId: json['project_type_id'] as String,
      code: json['code'] as String,
      displayName: json['display_name'] as String,
      description: json['description'] as String?,
      color: json['color'] as String?,
      allowedTransitions: json['allowed_transitions'] != null
          ? List<String>.from(json['allowed_transitions'] as List)
          : null,
      isInitial: json['is_initial'] as bool,
      isTerminal: json['is_terminal'] as bool,
      sortOrder: json['sort_order'] as int,
      createdBy: json['created_by'] as String,
      createdAt: DateTime.parse(json['created_at'] as String),
      updatedAt: DateTime.parse(json['updated_at'] as String),
    );
  }

  /// StatusDefinitionインスタンスをJSONマップに変換する
  Map<String, dynamic> toJson() {
    return {
      'id': id,
      'project_type_id': projectTypeId,
      'code': code,
      'display_name': displayName,
      'description': description,
      'color': color,
      'allowed_transitions': allowedTransitions,
      'is_initial': isInitial,
      'is_terminal': isTerminal,
      'sort_order': sortOrder,
      'created_by': createdBy,
      'created_at': createdAt.toIso8601String(),
      'updated_at': updatedAt.toIso8601String(),
    };
  }
}

/// ステータス定義作成時の入力モデル
class CreateStatusDefinitionInput {
  final String code;
  final String displayName;
  final String? description;
  final String? color;
  final List<String>? allowedTransitions;
  final bool isInitial;
  final bool isTerminal;
  final int sortOrder;

  const CreateStatusDefinitionInput({
    required this.code,
    required this.displayName,
    this.description,
    this.color,
    this.allowedTransitions,
    this.isInitial = false,
    this.isTerminal = false,
    this.sortOrder = 0,
  });

  /// 入力データをAPIリクエスト用のJSONマップに変換する
  Map<String, dynamic> toJson() {
    return {
      'code': code,
      'display_name': displayName,
      'description': description,
      'color': color,
      'allowed_transitions': allowedTransitions,
      'is_initial': isInitial,
      'is_terminal': isTerminal,
      'sort_order': sortOrder,
    };
  }
}

/// ステータス定義更新時の入力モデル
class UpdateStatusDefinitionInput {
  final String? displayName;
  final String? description;
  final String? color;
  final List<String>? allowedTransitions;
  final bool? isInitial;
  final bool? isTerminal;
  final int? sortOrder;

  const UpdateStatusDefinitionInput({
    this.displayName,
    this.description,
    this.color,
    this.allowedTransitions,
    this.isInitial,
    this.isTerminal,
    this.sortOrder,
  });

  /// 更新データをAPIリクエスト用のJSONマップに変換する
  /// nullのフィールドは除外する
  Map<String, dynamic> toJson() {
    final json = <String, dynamic>{};
    if (displayName != null) json['display_name'] = displayName;
    if (description != null) json['description'] = description;
    if (color != null) json['color'] = color;
    if (allowedTransitions != null) json['allowed_transitions'] = allowedTransitions;
    if (isInitial != null) json['is_initial'] = isInitial;
    if (isTerminal != null) json['is_terminal'] = isTerminal;
    if (sortOrder != null) json['sort_order'] = sortOrder;
    return json;
  }
}

/// ステータス定義のバージョン履歴モデル
/// ステータス定義の変更履歴を記録し、監査証跡として機能する
class StatusDefinitionVersion {
  final String id;
  final String statusDefinitionId;
  final int versionNumber;
  final Map<String, dynamic>? beforeData;
  final Map<String, dynamic> afterData;
  final String changedBy;
  final String? changeReason;
  final DateTime createdAt;

  const StatusDefinitionVersion({
    required this.id,
    required this.statusDefinitionId,
    required this.versionNumber,
    this.beforeData,
    required this.afterData,
    required this.changedBy,
    this.changeReason,
    required this.createdAt,
  });

  /// JSONマップからStatusDefinitionVersionインスタンスを生成する
  factory StatusDefinitionVersion.fromJson(Map<String, dynamic> json) {
    return StatusDefinitionVersion(
      id: json['id'] as String,
      statusDefinitionId: json['status_definition_id'] as String,
      versionNumber: json['version_number'] as int,
      beforeData: json['before_data'] as Map<String, dynamic>?,
      afterData: json['after_data'] as Map<String, dynamic>,
      changedBy: json['changed_by'] as String,
      changeReason: json['change_reason'] as String?,
      createdAt: DateTime.parse(json['created_at'] as String),
    );
  }

  /// StatusDefinitionVersionインスタンスをJSONマップに変換する
  Map<String, dynamic> toJson() {
    return {
      'id': id,
      'status_definition_id': statusDefinitionId,
      'version_number': versionNumber,
      'before_data': beforeData,
      'after_data': afterData,
      'changed_by': changedBy,
      'change_reason': changeReason,
      'created_at': createdAt.toIso8601String(),
    };
  }
}
