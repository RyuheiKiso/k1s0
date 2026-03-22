/// テナントプロジェクト拡張モデル定義
/// テナント固有のプロジェクトマスタカスタマイズを管理する
library;

/// テナントプロジェクト拡張モデル
/// テナント固有のステータス定義カスタマイズを管理する
class TenantProjectExtension {
  final String id;
  final String tenantId;
  final String statusDefinitionId;
  final String? displayNameOverride;
  final Map<String, dynamic>? attributesOverride;
  final bool isEnabled;
  final DateTime createdAt;
  final DateTime updatedAt;

  const TenantProjectExtension({
    required this.id,
    required this.tenantId,
    required this.statusDefinitionId,
    this.displayNameOverride,
    this.attributesOverride,
    required this.isEnabled,
    required this.createdAt,
    required this.updatedAt,
  });

  /// JSONマップからTenantProjectExtensionインスタンスを生成する
  factory TenantProjectExtension.fromJson(Map<String, dynamic> json) {
    return TenantProjectExtension(
      id: json['id'] as String,
      tenantId: json['tenant_id'] as String,
      statusDefinitionId: json['status_definition_id'] as String,
      displayNameOverride: json['display_name_override'] as String?,
      attributesOverride:
          json['attributes_override'] as Map<String, dynamic>?,
      isEnabled: json['is_enabled'] as bool,
      createdAt: DateTime.parse(json['created_at'] as String),
      updatedAt: DateTime.parse(json['updated_at'] as String),
    );
  }

  /// TenantProjectExtensionインスタンスをJSONマップに変換する
  Map<String, dynamic> toJson() {
    return {
      'id': id,
      'tenant_id': tenantId,
      'status_definition_id': statusDefinitionId,
      'display_name_override': displayNameOverride,
      'attributes_override': attributesOverride,
      'is_enabled': isEnabled,
      'created_at': createdAt.toIso8601String(),
      'updated_at': updatedAt.toIso8601String(),
    };
  }
}

/// テナント拡張更新時の入力モデル
class UpdateTenantExtensionInput {
  final String? displayNameOverride;
  final Map<String, dynamic>? attributesOverride;
  final bool? isEnabled;

  const UpdateTenantExtensionInput({
    this.displayNameOverride,
    this.attributesOverride,
    this.isEnabled,
  });

  /// 更新データをAPIリクエスト用のJSONマップに変換する
  Map<String, dynamic> toJson() {
    final json = <String, dynamic>{};
    if (displayNameOverride != null) {
      json['display_name_override'] = displayNameOverride;
    }
    if (attributesOverride != null) {
      json['attributes_override'] = attributesOverride;
    }
    if (isEnabled != null) {
      json['is_enabled'] = isEnabled;
    }
    return json;
  }
}
