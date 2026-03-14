/// ドメインマスタのデータモデル定義
/// freezed/json_serializableを使用せず、手書きのfromJson/toJsonで実装する

/// マスタカテゴリモデル
/// マスタデータの分類単位を表す（例：勘定科目、部門、通貨など）
class MasterCategory {
  final String id;
  final String code;
  final String displayName;
  final String? description;
  final Map<String, dynamic>? validationSchema;
  final bool isActive;
  final int sortOrder;
  final String createdBy;
  final DateTime createdAt;
  final DateTime updatedAt;

  const MasterCategory({
    required this.id,
    required this.code,
    required this.displayName,
    this.description,
    this.validationSchema,
    required this.isActive,
    required this.sortOrder,
    required this.createdBy,
    required this.createdAt,
    required this.updatedAt,
  });

  /// JSONマップからMasterCategoryインスタンスを生成する
  factory MasterCategory.fromJson(Map<String, dynamic> json) {
    return MasterCategory(
      id: json['id'] as String,
      code: json['code'] as String,
      displayName: json['display_name'] as String,
      description: json['description'] as String?,
      validationSchema: json['validation_schema'] as Map<String, dynamic>?,
      isActive: json['is_active'] as bool,
      sortOrder: json['sort_order'] as int,
      createdBy: json['created_by'] as String,
      createdAt: DateTime.parse(json['created_at'] as String),
      updatedAt: DateTime.parse(json['updated_at'] as String),
    );
  }

  /// MasterCategoryインスタンスをJSONマップに変換する
  Map<String, dynamic> toJson() {
    return {
      'id': id,
      'code': code,
      'display_name': displayName,
      'description': description,
      'validation_schema': validationSchema,
      'is_active': isActive,
      'sort_order': sortOrder,
      'created_by': createdBy,
      'created_at': createdAt.toIso8601String(),
      'updated_at': updatedAt.toIso8601String(),
    };
  }
}

/// カテゴリ作成時の入力モデル
/// 新規カテゴリ作成に必要なフィールドのみを定義する
class CreateCategoryInput {
  final String code;
  final String displayName;
  final String? description;
  final Map<String, dynamic>? validationSchema;
  final bool isActive;
  final int sortOrder;

  const CreateCategoryInput({
    required this.code,
    required this.displayName,
    this.description,
    this.validationSchema,
    this.isActive = true,
    this.sortOrder = 0,
  });

  /// 入力データをAPIリクエスト用のJSONマップに変換する
  Map<String, dynamic> toJson() {
    return {
      'code': code,
      'display_name': displayName,
      'description': description,
      'validation_schema': validationSchema,
      'is_active': isActive,
      'sort_order': sortOrder,
    };
  }
}

/// カテゴリ更新時の入力モデル
/// 更新可能なフィールドのみを定義する
class UpdateCategoryInput {
  final String? displayName;
  final String? description;
  final Map<String, dynamic>? validationSchema;
  final bool? isActive;
  final int? sortOrder;

  const UpdateCategoryInput({
    this.displayName,
    this.description,
    this.validationSchema,
    this.isActive,
    this.sortOrder,
  });

  /// 更新データをAPIリクエスト用のJSONマップに変換する
  /// nullのフィールドは除外する
  Map<String, dynamic> toJson() {
    final json = <String, dynamic>{};
    if (displayName != null) json['display_name'] = displayName;
    if (description != null) json['description'] = description;
    if (validationSchema != null) json['validation_schema'] = validationSchema;
    if (isActive != null) json['is_active'] = isActive;
    if (sortOrder != null) json['sort_order'] = sortOrder;
    return json;
  }
}

/// マスタアイテムモデル
/// カテゴリに属する個別のマスタデータ項目を表す
class MasterItem {
  final String id;
  final String categoryId;
  final String code;
  final String displayName;
  final String? description;
  final Map<String, dynamic>? attributes;
  final String? parentItemId;
  final DateTime? effectiveFrom;
  final DateTime? effectiveUntil;
  final bool isActive;
  final int sortOrder;
  final String createdBy;
  final DateTime createdAt;
  final DateTime updatedAt;

  const MasterItem({
    required this.id,
    required this.categoryId,
    required this.code,
    required this.displayName,
    this.description,
    this.attributes,
    this.parentItemId,
    this.effectiveFrom,
    this.effectiveUntil,
    required this.isActive,
    required this.sortOrder,
    required this.createdBy,
    required this.createdAt,
    required this.updatedAt,
  });

  /// JSONマップからMasterItemインスタンスを生成する
  factory MasterItem.fromJson(Map<String, dynamic> json) {
    return MasterItem(
      id: json['id'] as String,
      categoryId: json['category_id'] as String,
      code: json['code'] as String,
      displayName: json['display_name'] as String,
      description: json['description'] as String?,
      attributes: json['attributes'] as Map<String, dynamic>?,
      parentItemId: json['parent_item_id'] as String?,
      effectiveFrom: json['effective_from'] != null
          ? DateTime.parse(json['effective_from'] as String)
          : null,
      effectiveUntil: json['effective_until'] != null
          ? DateTime.parse(json['effective_until'] as String)
          : null,
      isActive: json['is_active'] as bool,
      sortOrder: json['sort_order'] as int,
      createdBy: json['created_by'] as String,
      createdAt: DateTime.parse(json['created_at'] as String),
      updatedAt: DateTime.parse(json['updated_at'] as String),
    );
  }

  /// MasterItemインスタンスをJSONマップに変換する
  Map<String, dynamic> toJson() {
    return {
      'id': id,
      'category_id': categoryId,
      'code': code,
      'display_name': displayName,
      'description': description,
      'attributes': attributes,
      'parent_item_id': parentItemId,
      'effective_from': effectiveFrom?.toIso8601String(),
      'effective_until': effectiveUntil?.toIso8601String(),
      'is_active': isActive,
      'sort_order': sortOrder,
      'created_by': createdBy,
      'created_at': createdAt.toIso8601String(),
      'updated_at': updatedAt.toIso8601String(),
    };
  }
}

/// アイテム作成時の入力モデル
class CreateItemInput {
  final String code;
  final String displayName;
  final String? description;
  final Map<String, dynamic>? attributes;
  final String? parentItemId;
  final DateTime? effectiveFrom;
  final DateTime? effectiveUntil;
  final bool isActive;
  final int sortOrder;

  const CreateItemInput({
    required this.code,
    required this.displayName,
    this.description,
    this.attributes,
    this.parentItemId,
    this.effectiveFrom,
    this.effectiveUntil,
    this.isActive = true,
    this.sortOrder = 0,
  });

  /// 入力データをAPIリクエスト用のJSONマップに変換する
  Map<String, dynamic> toJson() {
    return {
      'code': code,
      'display_name': displayName,
      'description': description,
      'attributes': attributes,
      'parent_item_id': parentItemId,
      'effective_from': effectiveFrom?.toIso8601String(),
      'effective_until': effectiveUntil?.toIso8601String(),
      'is_active': isActive,
      'sort_order': sortOrder,
    };
  }
}

/// アイテム更新時の入力モデル
class UpdateItemInput {
  final String? displayName;
  final String? description;
  final Map<String, dynamic>? attributes;
  final String? parentItemId;
  final DateTime? effectiveFrom;
  final DateTime? effectiveUntil;
  final bool? isActive;
  final int? sortOrder;

  const UpdateItemInput({
    this.displayName,
    this.description,
    this.attributes,
    this.parentItemId,
    this.effectiveFrom,
    this.effectiveUntil,
    this.isActive,
    this.sortOrder,
  });

  /// 更新データをAPIリクエスト用のJSONマップに変換する
  Map<String, dynamic> toJson() {
    final json = <String, dynamic>{};
    if (displayName != null) json['display_name'] = displayName;
    if (description != null) json['description'] = description;
    if (attributes != null) json['attributes'] = attributes;
    if (parentItemId != null) json['parent_item_id'] = parentItemId;
    if (effectiveFrom != null) {
      json['effective_from'] = effectiveFrom!.toIso8601String();
    }
    if (effectiveUntil != null) {
      json['effective_until'] = effectiveUntil!.toIso8601String();
    }
    if (isActive != null) json['is_active'] = isActive;
    if (sortOrder != null) json['sort_order'] = sortOrder;
    return json;
  }
}

/// マスタアイテムのバージョン履歴モデル
/// アイテムの変更履歴を記録し、監査証跡として機能する
class MasterItemVersion {
  final String id;
  final String itemId;
  final int versionNumber;
  final Map<String, dynamic>? beforeData;
  final Map<String, dynamic> afterData;
  final String changedBy;
  final String? changeReason;
  final DateTime createdAt;

  const MasterItemVersion({
    required this.id,
    required this.itemId,
    required this.versionNumber,
    this.beforeData,
    required this.afterData,
    required this.changedBy,
    this.changeReason,
    required this.createdAt,
  });

  /// JSONマップからMasterItemVersionインスタンスを生成する
  factory MasterItemVersion.fromJson(Map<String, dynamic> json) {
    return MasterItemVersion(
      id: json['id'] as String,
      itemId: json['item_id'] as String,
      versionNumber: json['version_number'] as int,
      beforeData: json['before_data'] as Map<String, dynamic>?,
      afterData: json['after_data'] as Map<String, dynamic>,
      changedBy: json['changed_by'] as String,
      changeReason: json['change_reason'] as String?,
      createdAt: DateTime.parse(json['created_at'] as String),
    );
  }

  /// MasterItemVersionインスタンスをJSONマップに変換する
  Map<String, dynamic> toJson() {
    return {
      'id': id,
      'item_id': itemId,
      'version_number': versionNumber,
      'before_data': beforeData,
      'after_data': afterData,
      'changed_by': changedBy,
      'change_reason': changeReason,
      'created_at': createdAt.toIso8601String(),
    };
  }
}

/// テナントマスタ拡張モデル
/// テナント固有のマスタデータカスタマイズを管理する
class TenantMasterExtension {
  final String id;
  final String tenantId;
  final String itemId;
  final String? displayNameOverride;
  final Map<String, dynamic>? attributesOverride;
  final DateTime createdAt;
  final DateTime updatedAt;

  const TenantMasterExtension({
    required this.id,
    required this.tenantId,
    required this.itemId,
    this.displayNameOverride,
    this.attributesOverride,
    required this.createdAt,
    required this.updatedAt,
  });

  /// JSONマップからTenantMasterExtensionインスタンスを生成する
  factory TenantMasterExtension.fromJson(Map<String, dynamic> json) {
    return TenantMasterExtension(
      id: json['id'] as String,
      tenantId: json['tenant_id'] as String,
      itemId: json['item_id'] as String,
      displayNameOverride: json['display_name_override'] as String?,
      attributesOverride:
          json['attributes_override'] as Map<String, dynamic>?,
      createdAt: DateTime.parse(json['created_at'] as String),
      updatedAt: DateTime.parse(json['updated_at'] as String),
    );
  }

  /// TenantMasterExtensionインスタンスをJSONマップに変換する
  Map<String, dynamic> toJson() {
    return {
      'id': id,
      'tenant_id': tenantId,
      'item_id': itemId,
      'display_name_override': displayNameOverride,
      'attributes_override': attributesOverride,
      'created_at': createdAt.toIso8601String(),
      'updated_at': updatedAt.toIso8601String(),
    };
  }
}

/// テナント拡張更新時の入力モデル
class UpdateTenantExtensionInput {
  final String? displayNameOverride;
  final Map<String, dynamic>? attributesOverride;

  const UpdateTenantExtensionInput({
    this.displayNameOverride,
    this.attributesOverride,
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
    return json;
  }
}
