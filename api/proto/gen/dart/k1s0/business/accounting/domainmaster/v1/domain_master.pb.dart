// This is a generated file - do not edit.
//
// Generated from k1s0/business/accounting/domainmaster/v1/domain_master.proto.

// @dart = 3.3

// ignore_for_file: annotate_overrides, camel_case_types, comment_references
// ignore_for_file: constant_identifier_names
// ignore_for_file: curly_braces_in_flow_control_structures
// ignore_for_file: deprecated_member_use_from_same_package, library_prefixes
// ignore_for_file: non_constant_identifier_names, prefer_relative_imports

import 'dart:core' as $core;

import 'package:protobuf/protobuf.dart' as $pb;
import 'package:protobuf/well_known_types/google/protobuf/struct.pb.dart' as $1;

import '../../../../system/common/v1/types.pb.dart' as $2;

export 'package:protobuf/protobuf.dart' show GeneratedMessageGenericExtensions;

/// マスタカテゴリ
class MasterCategory extends $pb.GeneratedMessage {
  factory MasterCategory({
    $core.String? id,
    $core.String? code,
    $core.String? displayName,
    $core.String? description,
    $1.Struct? validationSchema,
    $core.bool? isActive,
    $core.int? sortOrder,
    $core.String? createdBy,
    $2.Timestamp? createdAt,
    $2.Timestamp? updatedAt,
  }) {
    final result = create();
    if (id != null) result.id = id;
    if (code != null) result.code = code;
    if (displayName != null) result.displayName = displayName;
    if (description != null) result.description = description;
    if (validationSchema != null) result.validationSchema = validationSchema;
    if (isActive != null) result.isActive = isActive;
    if (sortOrder != null) result.sortOrder = sortOrder;
    if (createdBy != null) result.createdBy = createdBy;
    if (createdAt != null) result.createdAt = createdAt;
    if (updatedAt != null) result.updatedAt = updatedAt;
    return result;
  }

  MasterCategory._();

  factory MasterCategory.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory MasterCategory.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'MasterCategory',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.business.accounting.domainmaster.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'id')
    ..aOS(2, _omitFieldNames ? '' : 'code')
    ..aOS(3, _omitFieldNames ? '' : 'displayName')
    ..aOS(4, _omitFieldNames ? '' : 'description')
    ..aOM<$1.Struct>(5, _omitFieldNames ? '' : 'validationSchema',
        subBuilder: $1.Struct.create)
    ..aOB(6, _omitFieldNames ? '' : 'isActive')
    ..aI(7, _omitFieldNames ? '' : 'sortOrder')
    ..aOS(8, _omitFieldNames ? '' : 'createdBy')
    ..aOM<$2.Timestamp>(9, _omitFieldNames ? '' : 'createdAt',
        subBuilder: $2.Timestamp.create)
    ..aOM<$2.Timestamp>(10, _omitFieldNames ? '' : 'updatedAt',
        subBuilder: $2.Timestamp.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  MasterCategory clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  MasterCategory copyWith(void Function(MasterCategory) updates) =>
      super.copyWith((message) => updates(message as MasterCategory))
          as MasterCategory;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static MasterCategory create() => MasterCategory._();
  @$core.override
  MasterCategory createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static MasterCategory getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<MasterCategory>(create);
  static MasterCategory? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get id => $_getSZ(0);
  @$pb.TagNumber(1)
  set id($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasId() => $_has(0);
  @$pb.TagNumber(1)
  void clearId() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get code => $_getSZ(1);
  @$pb.TagNumber(2)
  set code($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasCode() => $_has(1);
  @$pb.TagNumber(2)
  void clearCode() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.String get displayName => $_getSZ(2);
  @$pb.TagNumber(3)
  set displayName($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasDisplayName() => $_has(2);
  @$pb.TagNumber(3)
  void clearDisplayName() => $_clearField(3);

  @$pb.TagNumber(4)
  $core.String get description => $_getSZ(3);
  @$pb.TagNumber(4)
  set description($core.String value) => $_setString(3, value);
  @$pb.TagNumber(4)
  $core.bool hasDescription() => $_has(3);
  @$pb.TagNumber(4)
  void clearDescription() => $_clearField(4);

  @$pb.TagNumber(5)
  $1.Struct get validationSchema => $_getN(4);
  @$pb.TagNumber(5)
  set validationSchema($1.Struct value) => $_setField(5, value);
  @$pb.TagNumber(5)
  $core.bool hasValidationSchema() => $_has(4);
  @$pb.TagNumber(5)
  void clearValidationSchema() => $_clearField(5);
  @$pb.TagNumber(5)
  $1.Struct ensureValidationSchema() => $_ensure(4);

  @$pb.TagNumber(6)
  $core.bool get isActive => $_getBF(5);
  @$pb.TagNumber(6)
  set isActive($core.bool value) => $_setBool(5, value);
  @$pb.TagNumber(6)
  $core.bool hasIsActive() => $_has(5);
  @$pb.TagNumber(6)
  void clearIsActive() => $_clearField(6);

  @$pb.TagNumber(7)
  $core.int get sortOrder => $_getIZ(6);
  @$pb.TagNumber(7)
  set sortOrder($core.int value) => $_setSignedInt32(6, value);
  @$pb.TagNumber(7)
  $core.bool hasSortOrder() => $_has(6);
  @$pb.TagNumber(7)
  void clearSortOrder() => $_clearField(7);

  @$pb.TagNumber(8)
  $core.String get createdBy => $_getSZ(7);
  @$pb.TagNumber(8)
  set createdBy($core.String value) => $_setString(7, value);
  @$pb.TagNumber(8)
  $core.bool hasCreatedBy() => $_has(7);
  @$pb.TagNumber(8)
  void clearCreatedBy() => $_clearField(8);

  @$pb.TagNumber(9)
  $2.Timestamp get createdAt => $_getN(8);
  @$pb.TagNumber(9)
  set createdAt($2.Timestamp value) => $_setField(9, value);
  @$pb.TagNumber(9)
  $core.bool hasCreatedAt() => $_has(8);
  @$pb.TagNumber(9)
  void clearCreatedAt() => $_clearField(9);
  @$pb.TagNumber(9)
  $2.Timestamp ensureCreatedAt() => $_ensure(8);

  @$pb.TagNumber(10)
  $2.Timestamp get updatedAt => $_getN(9);
  @$pb.TagNumber(10)
  set updatedAt($2.Timestamp value) => $_setField(10, value);
  @$pb.TagNumber(10)
  $core.bool hasUpdatedAt() => $_has(9);
  @$pb.TagNumber(10)
  void clearUpdatedAt() => $_clearField(10);
  @$pb.TagNumber(10)
  $2.Timestamp ensureUpdatedAt() => $_ensure(9);
}

/// マスタ項目
class MasterItem extends $pb.GeneratedMessage {
  factory MasterItem({
    $core.String? id,
    $core.String? categoryId,
    $core.String? code,
    $core.String? displayName,
    $core.String? description,
    $1.Struct? attributes,
    $core.String? parentItemId,
    $2.Timestamp? effectiveFrom,
    $2.Timestamp? effectiveUntil,
    $core.bool? isActive,
    $core.int? sortOrder,
    $core.String? createdBy,
    $2.Timestamp? createdAt,
    $2.Timestamp? updatedAt,
  }) {
    final result = create();
    if (id != null) result.id = id;
    if (categoryId != null) result.categoryId = categoryId;
    if (code != null) result.code = code;
    if (displayName != null) result.displayName = displayName;
    if (description != null) result.description = description;
    if (attributes != null) result.attributes = attributes;
    if (parentItemId != null) result.parentItemId = parentItemId;
    if (effectiveFrom != null) result.effectiveFrom = effectiveFrom;
    if (effectiveUntil != null) result.effectiveUntil = effectiveUntil;
    if (isActive != null) result.isActive = isActive;
    if (sortOrder != null) result.sortOrder = sortOrder;
    if (createdBy != null) result.createdBy = createdBy;
    if (createdAt != null) result.createdAt = createdAt;
    if (updatedAt != null) result.updatedAt = updatedAt;
    return result;
  }

  MasterItem._();

  factory MasterItem.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory MasterItem.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'MasterItem',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.business.accounting.domainmaster.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'id')
    ..aOS(2, _omitFieldNames ? '' : 'categoryId')
    ..aOS(3, _omitFieldNames ? '' : 'code')
    ..aOS(4, _omitFieldNames ? '' : 'displayName')
    ..aOS(5, _omitFieldNames ? '' : 'description')
    ..aOM<$1.Struct>(6, _omitFieldNames ? '' : 'attributes',
        subBuilder: $1.Struct.create)
    ..aOS(7, _omitFieldNames ? '' : 'parentItemId')
    ..aOM<$2.Timestamp>(8, _omitFieldNames ? '' : 'effectiveFrom',
        subBuilder: $2.Timestamp.create)
    ..aOM<$2.Timestamp>(9, _omitFieldNames ? '' : 'effectiveUntil',
        subBuilder: $2.Timestamp.create)
    ..aOB(10, _omitFieldNames ? '' : 'isActive')
    ..aI(11, _omitFieldNames ? '' : 'sortOrder')
    ..aOS(12, _omitFieldNames ? '' : 'createdBy')
    ..aOM<$2.Timestamp>(13, _omitFieldNames ? '' : 'createdAt',
        subBuilder: $2.Timestamp.create)
    ..aOM<$2.Timestamp>(14, _omitFieldNames ? '' : 'updatedAt',
        subBuilder: $2.Timestamp.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  MasterItem clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  MasterItem copyWith(void Function(MasterItem) updates) =>
      super.copyWith((message) => updates(message as MasterItem)) as MasterItem;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static MasterItem create() => MasterItem._();
  @$core.override
  MasterItem createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static MasterItem getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<MasterItem>(create);
  static MasterItem? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get id => $_getSZ(0);
  @$pb.TagNumber(1)
  set id($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasId() => $_has(0);
  @$pb.TagNumber(1)
  void clearId() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get categoryId => $_getSZ(1);
  @$pb.TagNumber(2)
  set categoryId($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasCategoryId() => $_has(1);
  @$pb.TagNumber(2)
  void clearCategoryId() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.String get code => $_getSZ(2);
  @$pb.TagNumber(3)
  set code($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasCode() => $_has(2);
  @$pb.TagNumber(3)
  void clearCode() => $_clearField(3);

  @$pb.TagNumber(4)
  $core.String get displayName => $_getSZ(3);
  @$pb.TagNumber(4)
  set displayName($core.String value) => $_setString(3, value);
  @$pb.TagNumber(4)
  $core.bool hasDisplayName() => $_has(3);
  @$pb.TagNumber(4)
  void clearDisplayName() => $_clearField(4);

  @$pb.TagNumber(5)
  $core.String get description => $_getSZ(4);
  @$pb.TagNumber(5)
  set description($core.String value) => $_setString(4, value);
  @$pb.TagNumber(5)
  $core.bool hasDescription() => $_has(4);
  @$pb.TagNumber(5)
  void clearDescription() => $_clearField(5);

  @$pb.TagNumber(6)
  $1.Struct get attributes => $_getN(5);
  @$pb.TagNumber(6)
  set attributes($1.Struct value) => $_setField(6, value);
  @$pb.TagNumber(6)
  $core.bool hasAttributes() => $_has(5);
  @$pb.TagNumber(6)
  void clearAttributes() => $_clearField(6);
  @$pb.TagNumber(6)
  $1.Struct ensureAttributes() => $_ensure(5);

  @$pb.TagNumber(7)
  $core.String get parentItemId => $_getSZ(6);
  @$pb.TagNumber(7)
  set parentItemId($core.String value) => $_setString(6, value);
  @$pb.TagNumber(7)
  $core.bool hasParentItemId() => $_has(6);
  @$pb.TagNumber(7)
  void clearParentItemId() => $_clearField(7);

  @$pb.TagNumber(8)
  $2.Timestamp get effectiveFrom => $_getN(7);
  @$pb.TagNumber(8)
  set effectiveFrom($2.Timestamp value) => $_setField(8, value);
  @$pb.TagNumber(8)
  $core.bool hasEffectiveFrom() => $_has(7);
  @$pb.TagNumber(8)
  void clearEffectiveFrom() => $_clearField(8);
  @$pb.TagNumber(8)
  $2.Timestamp ensureEffectiveFrom() => $_ensure(7);

  @$pb.TagNumber(9)
  $2.Timestamp get effectiveUntil => $_getN(8);
  @$pb.TagNumber(9)
  set effectiveUntil($2.Timestamp value) => $_setField(9, value);
  @$pb.TagNumber(9)
  $core.bool hasEffectiveUntil() => $_has(8);
  @$pb.TagNumber(9)
  void clearEffectiveUntil() => $_clearField(9);
  @$pb.TagNumber(9)
  $2.Timestamp ensureEffectiveUntil() => $_ensure(8);

  @$pb.TagNumber(10)
  $core.bool get isActive => $_getBF(9);
  @$pb.TagNumber(10)
  set isActive($core.bool value) => $_setBool(9, value);
  @$pb.TagNumber(10)
  $core.bool hasIsActive() => $_has(9);
  @$pb.TagNumber(10)
  void clearIsActive() => $_clearField(10);

  @$pb.TagNumber(11)
  $core.int get sortOrder => $_getIZ(10);
  @$pb.TagNumber(11)
  set sortOrder($core.int value) => $_setSignedInt32(10, value);
  @$pb.TagNumber(11)
  $core.bool hasSortOrder() => $_has(10);
  @$pb.TagNumber(11)
  void clearSortOrder() => $_clearField(11);

  @$pb.TagNumber(12)
  $core.String get createdBy => $_getSZ(11);
  @$pb.TagNumber(12)
  set createdBy($core.String value) => $_setString(11, value);
  @$pb.TagNumber(12)
  $core.bool hasCreatedBy() => $_has(11);
  @$pb.TagNumber(12)
  void clearCreatedBy() => $_clearField(12);

  @$pb.TagNumber(13)
  $2.Timestamp get createdAt => $_getN(12);
  @$pb.TagNumber(13)
  set createdAt($2.Timestamp value) => $_setField(13, value);
  @$pb.TagNumber(13)
  $core.bool hasCreatedAt() => $_has(12);
  @$pb.TagNumber(13)
  void clearCreatedAt() => $_clearField(13);
  @$pb.TagNumber(13)
  $2.Timestamp ensureCreatedAt() => $_ensure(12);

  @$pb.TagNumber(14)
  $2.Timestamp get updatedAt => $_getN(13);
  @$pb.TagNumber(14)
  set updatedAt($2.Timestamp value) => $_setField(14, value);
  @$pb.TagNumber(14)
  $core.bool hasUpdatedAt() => $_has(13);
  @$pb.TagNumber(14)
  void clearUpdatedAt() => $_clearField(14);
  @$pb.TagNumber(14)
  $2.Timestamp ensureUpdatedAt() => $_ensure(13);
}

/// マスタ項目バージョン
class MasterItemVersion extends $pb.GeneratedMessage {
  factory MasterItemVersion({
    $core.String? id,
    $core.String? itemId,
    $core.int? versionNumber,
    $1.Struct? beforeData,
    $1.Struct? afterData,
    $core.String? changedBy,
    $core.String? changeReason,
    $2.Timestamp? createdAt,
  }) {
    final result = create();
    if (id != null) result.id = id;
    if (itemId != null) result.itemId = itemId;
    if (versionNumber != null) result.versionNumber = versionNumber;
    if (beforeData != null) result.beforeData = beforeData;
    if (afterData != null) result.afterData = afterData;
    if (changedBy != null) result.changedBy = changedBy;
    if (changeReason != null) result.changeReason = changeReason;
    if (createdAt != null) result.createdAt = createdAt;
    return result;
  }

  MasterItemVersion._();

  factory MasterItemVersion.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory MasterItemVersion.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'MasterItemVersion',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.business.accounting.domainmaster.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'id')
    ..aOS(2, _omitFieldNames ? '' : 'itemId')
    ..aI(3, _omitFieldNames ? '' : 'versionNumber')
    ..aOM<$1.Struct>(4, _omitFieldNames ? '' : 'beforeData',
        subBuilder: $1.Struct.create)
    ..aOM<$1.Struct>(5, _omitFieldNames ? '' : 'afterData',
        subBuilder: $1.Struct.create)
    ..aOS(6, _omitFieldNames ? '' : 'changedBy')
    ..aOS(7, _omitFieldNames ? '' : 'changeReason')
    ..aOM<$2.Timestamp>(8, _omitFieldNames ? '' : 'createdAt',
        subBuilder: $2.Timestamp.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  MasterItemVersion clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  MasterItemVersion copyWith(void Function(MasterItemVersion) updates) =>
      super.copyWith((message) => updates(message as MasterItemVersion))
          as MasterItemVersion;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static MasterItemVersion create() => MasterItemVersion._();
  @$core.override
  MasterItemVersion createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static MasterItemVersion getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<MasterItemVersion>(create);
  static MasterItemVersion? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get id => $_getSZ(0);
  @$pb.TagNumber(1)
  set id($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasId() => $_has(0);
  @$pb.TagNumber(1)
  void clearId() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get itemId => $_getSZ(1);
  @$pb.TagNumber(2)
  set itemId($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasItemId() => $_has(1);
  @$pb.TagNumber(2)
  void clearItemId() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.int get versionNumber => $_getIZ(2);
  @$pb.TagNumber(3)
  set versionNumber($core.int value) => $_setSignedInt32(2, value);
  @$pb.TagNumber(3)
  $core.bool hasVersionNumber() => $_has(2);
  @$pb.TagNumber(3)
  void clearVersionNumber() => $_clearField(3);

  @$pb.TagNumber(4)
  $1.Struct get beforeData => $_getN(3);
  @$pb.TagNumber(4)
  set beforeData($1.Struct value) => $_setField(4, value);
  @$pb.TagNumber(4)
  $core.bool hasBeforeData() => $_has(3);
  @$pb.TagNumber(4)
  void clearBeforeData() => $_clearField(4);
  @$pb.TagNumber(4)
  $1.Struct ensureBeforeData() => $_ensure(3);

  @$pb.TagNumber(5)
  $1.Struct get afterData => $_getN(4);
  @$pb.TagNumber(5)
  set afterData($1.Struct value) => $_setField(5, value);
  @$pb.TagNumber(5)
  $core.bool hasAfterData() => $_has(4);
  @$pb.TagNumber(5)
  void clearAfterData() => $_clearField(5);
  @$pb.TagNumber(5)
  $1.Struct ensureAfterData() => $_ensure(4);

  @$pb.TagNumber(6)
  $core.String get changedBy => $_getSZ(5);
  @$pb.TagNumber(6)
  set changedBy($core.String value) => $_setString(5, value);
  @$pb.TagNumber(6)
  $core.bool hasChangedBy() => $_has(5);
  @$pb.TagNumber(6)
  void clearChangedBy() => $_clearField(6);

  @$pb.TagNumber(7)
  $core.String get changeReason => $_getSZ(6);
  @$pb.TagNumber(7)
  set changeReason($core.String value) => $_setString(6, value);
  @$pb.TagNumber(7)
  $core.bool hasChangeReason() => $_has(6);
  @$pb.TagNumber(7)
  void clearChangeReason() => $_clearField(7);

  @$pb.TagNumber(8)
  $2.Timestamp get createdAt => $_getN(7);
  @$pb.TagNumber(8)
  set createdAt($2.Timestamp value) => $_setField(8, value);
  @$pb.TagNumber(8)
  $core.bool hasCreatedAt() => $_has(7);
  @$pb.TagNumber(8)
  void clearCreatedAt() => $_clearField(8);
  @$pb.TagNumber(8)
  $2.Timestamp ensureCreatedAt() => $_ensure(7);
}

/// テナントマスタ拡張
class TenantMasterExtension extends $pb.GeneratedMessage {
  factory TenantMasterExtension({
    $core.String? id,
    $core.String? tenantId,
    $core.String? itemId,
    $core.String? displayNameOverride,
    $1.Struct? attributesOverride,
    $core.bool? isEnabled,
    $2.Timestamp? createdAt,
    $2.Timestamp? updatedAt,
  }) {
    final result = create();
    if (id != null) result.id = id;
    if (tenantId != null) result.tenantId = tenantId;
    if (itemId != null) result.itemId = itemId;
    if (displayNameOverride != null)
      result.displayNameOverride = displayNameOverride;
    if (attributesOverride != null)
      result.attributesOverride = attributesOverride;
    if (isEnabled != null) result.isEnabled = isEnabled;
    if (createdAt != null) result.createdAt = createdAt;
    if (updatedAt != null) result.updatedAt = updatedAt;
    return result;
  }

  TenantMasterExtension._();

  factory TenantMasterExtension.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory TenantMasterExtension.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'TenantMasterExtension',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.business.accounting.domainmaster.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'id')
    ..aOS(2, _omitFieldNames ? '' : 'tenantId')
    ..aOS(3, _omitFieldNames ? '' : 'itemId')
    ..aOS(4, _omitFieldNames ? '' : 'displayNameOverride')
    ..aOM<$1.Struct>(5, _omitFieldNames ? '' : 'attributesOverride',
        subBuilder: $1.Struct.create)
    ..aOB(6, _omitFieldNames ? '' : 'isEnabled')
    ..aOM<$2.Timestamp>(7, _omitFieldNames ? '' : 'createdAt',
        subBuilder: $2.Timestamp.create)
    ..aOM<$2.Timestamp>(8, _omitFieldNames ? '' : 'updatedAt',
        subBuilder: $2.Timestamp.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  TenantMasterExtension clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  TenantMasterExtension copyWith(
          void Function(TenantMasterExtension) updates) =>
      super.copyWith((message) => updates(message as TenantMasterExtension))
          as TenantMasterExtension;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static TenantMasterExtension create() => TenantMasterExtension._();
  @$core.override
  TenantMasterExtension createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static TenantMasterExtension getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<TenantMasterExtension>(create);
  static TenantMasterExtension? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get id => $_getSZ(0);
  @$pb.TagNumber(1)
  set id($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasId() => $_has(0);
  @$pb.TagNumber(1)
  void clearId() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get tenantId => $_getSZ(1);
  @$pb.TagNumber(2)
  set tenantId($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasTenantId() => $_has(1);
  @$pb.TagNumber(2)
  void clearTenantId() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.String get itemId => $_getSZ(2);
  @$pb.TagNumber(3)
  set itemId($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasItemId() => $_has(2);
  @$pb.TagNumber(3)
  void clearItemId() => $_clearField(3);

  @$pb.TagNumber(4)
  $core.String get displayNameOverride => $_getSZ(3);
  @$pb.TagNumber(4)
  set displayNameOverride($core.String value) => $_setString(3, value);
  @$pb.TagNumber(4)
  $core.bool hasDisplayNameOverride() => $_has(3);
  @$pb.TagNumber(4)
  void clearDisplayNameOverride() => $_clearField(4);

  @$pb.TagNumber(5)
  $1.Struct get attributesOverride => $_getN(4);
  @$pb.TagNumber(5)
  set attributesOverride($1.Struct value) => $_setField(5, value);
  @$pb.TagNumber(5)
  $core.bool hasAttributesOverride() => $_has(4);
  @$pb.TagNumber(5)
  void clearAttributesOverride() => $_clearField(5);
  @$pb.TagNumber(5)
  $1.Struct ensureAttributesOverride() => $_ensure(4);

  @$pb.TagNumber(6)
  $core.bool get isEnabled => $_getBF(5);
  @$pb.TagNumber(6)
  set isEnabled($core.bool value) => $_setBool(5, value);
  @$pb.TagNumber(6)
  $core.bool hasIsEnabled() => $_has(5);
  @$pb.TagNumber(6)
  void clearIsEnabled() => $_clearField(6);

  @$pb.TagNumber(7)
  $2.Timestamp get createdAt => $_getN(6);
  @$pb.TagNumber(7)
  set createdAt($2.Timestamp value) => $_setField(7, value);
  @$pb.TagNumber(7)
  $core.bool hasCreatedAt() => $_has(6);
  @$pb.TagNumber(7)
  void clearCreatedAt() => $_clearField(7);
  @$pb.TagNumber(7)
  $2.Timestamp ensureCreatedAt() => $_ensure(6);

  @$pb.TagNumber(8)
  $2.Timestamp get updatedAt => $_getN(7);
  @$pb.TagNumber(8)
  set updatedAt($2.Timestamp value) => $_setField(8, value);
  @$pb.TagNumber(8)
  $core.bool hasUpdatedAt() => $_has(7);
  @$pb.TagNumber(8)
  void clearUpdatedAt() => $_clearField(8);
  @$pb.TagNumber(8)
  $2.Timestamp ensureUpdatedAt() => $_ensure(7);
}

/// テナントマージ項目（マージビュー用）
class TenantMergedItem extends $pb.GeneratedMessage {
  factory TenantMergedItem({
    MasterItem? baseItem,
    TenantMasterExtension? extension_2,
    $core.String? effectiveDisplayName,
    $1.Struct? effectiveAttributes,
  }) {
    final result = create();
    if (baseItem != null) result.baseItem = baseItem;
    if (extension_2 != null) result.extension_2 = extension_2;
    if (effectiveDisplayName != null)
      result.effectiveDisplayName = effectiveDisplayName;
    if (effectiveAttributes != null)
      result.effectiveAttributes = effectiveAttributes;
    return result;
  }

  TenantMergedItem._();

  factory TenantMergedItem.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory TenantMergedItem.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'TenantMergedItem',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.business.accounting.domainmaster.v1'),
      createEmptyInstance: create)
    ..aOM<MasterItem>(1, _omitFieldNames ? '' : 'baseItem',
        subBuilder: MasterItem.create)
    ..aOM<TenantMasterExtension>(2, _omitFieldNames ? '' : 'extension',
        subBuilder: TenantMasterExtension.create)
    ..aOS(3, _omitFieldNames ? '' : 'effectiveDisplayName')
    ..aOM<$1.Struct>(4, _omitFieldNames ? '' : 'effectiveAttributes',
        subBuilder: $1.Struct.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  TenantMergedItem clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  TenantMergedItem copyWith(void Function(TenantMergedItem) updates) =>
      super.copyWith((message) => updates(message as TenantMergedItem))
          as TenantMergedItem;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static TenantMergedItem create() => TenantMergedItem._();
  @$core.override
  TenantMergedItem createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static TenantMergedItem getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<TenantMergedItem>(create);
  static TenantMergedItem? _defaultInstance;

  @$pb.TagNumber(1)
  MasterItem get baseItem => $_getN(0);
  @$pb.TagNumber(1)
  set baseItem(MasterItem value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasBaseItem() => $_has(0);
  @$pb.TagNumber(1)
  void clearBaseItem() => $_clearField(1);
  @$pb.TagNumber(1)
  MasterItem ensureBaseItem() => $_ensure(0);

  @$pb.TagNumber(2)
  TenantMasterExtension get extension_2 => $_getN(1);
  @$pb.TagNumber(2)
  set extension_2(TenantMasterExtension value) => $_setField(2, value);
  @$pb.TagNumber(2)
  $core.bool hasExtension_2() => $_has(1);
  @$pb.TagNumber(2)
  void clearExtension_2() => $_clearField(2);
  @$pb.TagNumber(2)
  TenantMasterExtension ensureExtension_2() => $_ensure(1);

  @$pb.TagNumber(3)
  $core.String get effectiveDisplayName => $_getSZ(2);
  @$pb.TagNumber(3)
  set effectiveDisplayName($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasEffectiveDisplayName() => $_has(2);
  @$pb.TagNumber(3)
  void clearEffectiveDisplayName() => $_clearField(3);

  @$pb.TagNumber(4)
  $1.Struct get effectiveAttributes => $_getN(3);
  @$pb.TagNumber(4)
  set effectiveAttributes($1.Struct value) => $_setField(4, value);
  @$pb.TagNumber(4)
  $core.bool hasEffectiveAttributes() => $_has(3);
  @$pb.TagNumber(4)
  void clearEffectiveAttributes() => $_clearField(4);
  @$pb.TagNumber(4)
  $1.Struct ensureEffectiveAttributes() => $_ensure(3);
}

class ListCategoriesRequest extends $pb.GeneratedMessage {
  factory ListCategoriesRequest({
    $core.bool? activeOnly,
    $2.Pagination? pagination,
  }) {
    final result = create();
    if (activeOnly != null) result.activeOnly = activeOnly;
    if (pagination != null) result.pagination = pagination;
    return result;
  }

  ListCategoriesRequest._();

  factory ListCategoriesRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory ListCategoriesRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ListCategoriesRequest',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.business.accounting.domainmaster.v1'),
      createEmptyInstance: create)
    ..aOB(1, _omitFieldNames ? '' : 'activeOnly')
    ..aOM<$2.Pagination>(2, _omitFieldNames ? '' : 'pagination',
        subBuilder: $2.Pagination.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListCategoriesRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListCategoriesRequest copyWith(
          void Function(ListCategoriesRequest) updates) =>
      super.copyWith((message) => updates(message as ListCategoriesRequest))
          as ListCategoriesRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static ListCategoriesRequest create() => ListCategoriesRequest._();
  @$core.override
  ListCategoriesRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static ListCategoriesRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ListCategoriesRequest>(create);
  static ListCategoriesRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.bool get activeOnly => $_getBF(0);
  @$pb.TagNumber(1)
  set activeOnly($core.bool value) => $_setBool(0, value);
  @$pb.TagNumber(1)
  $core.bool hasActiveOnly() => $_has(0);
  @$pb.TagNumber(1)
  void clearActiveOnly() => $_clearField(1);

  @$pb.TagNumber(2)
  $2.Pagination get pagination => $_getN(1);
  @$pb.TagNumber(2)
  set pagination($2.Pagination value) => $_setField(2, value);
  @$pb.TagNumber(2)
  $core.bool hasPagination() => $_has(1);
  @$pb.TagNumber(2)
  void clearPagination() => $_clearField(2);
  @$pb.TagNumber(2)
  $2.Pagination ensurePagination() => $_ensure(1);
}

class ListCategoriesResponse extends $pb.GeneratedMessage {
  factory ListCategoriesResponse({
    $core.Iterable<MasterCategory>? categories,
    $2.PaginationResult? pagination,
  }) {
    final result = create();
    if (categories != null) result.categories.addAll(categories);
    if (pagination != null) result.pagination = pagination;
    return result;
  }

  ListCategoriesResponse._();

  factory ListCategoriesResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory ListCategoriesResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ListCategoriesResponse',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.business.accounting.domainmaster.v1'),
      createEmptyInstance: create)
    ..pPM<MasterCategory>(1, _omitFieldNames ? '' : 'categories',
        subBuilder: MasterCategory.create)
    ..aOM<$2.PaginationResult>(2, _omitFieldNames ? '' : 'pagination',
        subBuilder: $2.PaginationResult.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListCategoriesResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListCategoriesResponse copyWith(
          void Function(ListCategoriesResponse) updates) =>
      super.copyWith((message) => updates(message as ListCategoriesResponse))
          as ListCategoriesResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static ListCategoriesResponse create() => ListCategoriesResponse._();
  @$core.override
  ListCategoriesResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static ListCategoriesResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ListCategoriesResponse>(create);
  static ListCategoriesResponse? _defaultInstance;

  @$pb.TagNumber(1)
  $pb.PbList<MasterCategory> get categories => $_getList(0);

  @$pb.TagNumber(2)
  $2.PaginationResult get pagination => $_getN(1);
  @$pb.TagNumber(2)
  set pagination($2.PaginationResult value) => $_setField(2, value);
  @$pb.TagNumber(2)
  $core.bool hasPagination() => $_has(1);
  @$pb.TagNumber(2)
  void clearPagination() => $_clearField(2);
  @$pb.TagNumber(2)
  $2.PaginationResult ensurePagination() => $_ensure(1);
}

class GetCategoryRequest extends $pb.GeneratedMessage {
  factory GetCategoryRequest({
    $core.String? categoryId,
  }) {
    final result = create();
    if (categoryId != null) result.categoryId = categoryId;
    return result;
  }

  GetCategoryRequest._();

  factory GetCategoryRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory GetCategoryRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'GetCategoryRequest',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.business.accounting.domainmaster.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'categoryId')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetCategoryRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetCategoryRequest copyWith(void Function(GetCategoryRequest) updates) =>
      super.copyWith((message) => updates(message as GetCategoryRequest))
          as GetCategoryRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static GetCategoryRequest create() => GetCategoryRequest._();
  @$core.override
  GetCategoryRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static GetCategoryRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<GetCategoryRequest>(create);
  static GetCategoryRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get categoryId => $_getSZ(0);
  @$pb.TagNumber(1)
  set categoryId($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasCategoryId() => $_has(0);
  @$pb.TagNumber(1)
  void clearCategoryId() => $_clearField(1);
}

class GetCategoryResponse extends $pb.GeneratedMessage {
  factory GetCategoryResponse({
    MasterCategory? category,
  }) {
    final result = create();
    if (category != null) result.category = category;
    return result;
  }

  GetCategoryResponse._();

  factory GetCategoryResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory GetCategoryResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'GetCategoryResponse',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.business.accounting.domainmaster.v1'),
      createEmptyInstance: create)
    ..aOM<MasterCategory>(1, _omitFieldNames ? '' : 'category',
        subBuilder: MasterCategory.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetCategoryResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetCategoryResponse copyWith(void Function(GetCategoryResponse) updates) =>
      super.copyWith((message) => updates(message as GetCategoryResponse))
          as GetCategoryResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static GetCategoryResponse create() => GetCategoryResponse._();
  @$core.override
  GetCategoryResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static GetCategoryResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<GetCategoryResponse>(create);
  static GetCategoryResponse? _defaultInstance;

  @$pb.TagNumber(1)
  MasterCategory get category => $_getN(0);
  @$pb.TagNumber(1)
  set category(MasterCategory value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasCategory() => $_has(0);
  @$pb.TagNumber(1)
  void clearCategory() => $_clearField(1);
  @$pb.TagNumber(1)
  MasterCategory ensureCategory() => $_ensure(0);
}

class CreateCategoryRequest extends $pb.GeneratedMessage {
  factory CreateCategoryRequest({
    $core.String? code,
    $core.String? displayName,
    $core.String? description,
    $1.Struct? validationSchema,
    $core.bool? isActive,
    $core.int? sortOrder,
  }) {
    final result = create();
    if (code != null) result.code = code;
    if (displayName != null) result.displayName = displayName;
    if (description != null) result.description = description;
    if (validationSchema != null) result.validationSchema = validationSchema;
    if (isActive != null) result.isActive = isActive;
    if (sortOrder != null) result.sortOrder = sortOrder;
    return result;
  }

  CreateCategoryRequest._();

  factory CreateCategoryRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory CreateCategoryRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'CreateCategoryRequest',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.business.accounting.domainmaster.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'code')
    ..aOS(2, _omitFieldNames ? '' : 'displayName')
    ..aOS(3, _omitFieldNames ? '' : 'description')
    ..aOM<$1.Struct>(4, _omitFieldNames ? '' : 'validationSchema',
        subBuilder: $1.Struct.create)
    ..aOB(5, _omitFieldNames ? '' : 'isActive')
    ..aI(6, _omitFieldNames ? '' : 'sortOrder')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  CreateCategoryRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  CreateCategoryRequest copyWith(
          void Function(CreateCategoryRequest) updates) =>
      super.copyWith((message) => updates(message as CreateCategoryRequest))
          as CreateCategoryRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static CreateCategoryRequest create() => CreateCategoryRequest._();
  @$core.override
  CreateCategoryRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static CreateCategoryRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<CreateCategoryRequest>(create);
  static CreateCategoryRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get code => $_getSZ(0);
  @$pb.TagNumber(1)
  set code($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasCode() => $_has(0);
  @$pb.TagNumber(1)
  void clearCode() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get displayName => $_getSZ(1);
  @$pb.TagNumber(2)
  set displayName($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasDisplayName() => $_has(1);
  @$pb.TagNumber(2)
  void clearDisplayName() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.String get description => $_getSZ(2);
  @$pb.TagNumber(3)
  set description($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasDescription() => $_has(2);
  @$pb.TagNumber(3)
  void clearDescription() => $_clearField(3);

  @$pb.TagNumber(4)
  $1.Struct get validationSchema => $_getN(3);
  @$pb.TagNumber(4)
  set validationSchema($1.Struct value) => $_setField(4, value);
  @$pb.TagNumber(4)
  $core.bool hasValidationSchema() => $_has(3);
  @$pb.TagNumber(4)
  void clearValidationSchema() => $_clearField(4);
  @$pb.TagNumber(4)
  $1.Struct ensureValidationSchema() => $_ensure(3);

  @$pb.TagNumber(5)
  $core.bool get isActive => $_getBF(4);
  @$pb.TagNumber(5)
  set isActive($core.bool value) => $_setBool(4, value);
  @$pb.TagNumber(5)
  $core.bool hasIsActive() => $_has(4);
  @$pb.TagNumber(5)
  void clearIsActive() => $_clearField(5);

  @$pb.TagNumber(6)
  $core.int get sortOrder => $_getIZ(5);
  @$pb.TagNumber(6)
  set sortOrder($core.int value) => $_setSignedInt32(5, value);
  @$pb.TagNumber(6)
  $core.bool hasSortOrder() => $_has(5);
  @$pb.TagNumber(6)
  void clearSortOrder() => $_clearField(6);
}

class CreateCategoryResponse extends $pb.GeneratedMessage {
  factory CreateCategoryResponse({
    MasterCategory? category,
  }) {
    final result = create();
    if (category != null) result.category = category;
    return result;
  }

  CreateCategoryResponse._();

  factory CreateCategoryResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory CreateCategoryResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'CreateCategoryResponse',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.business.accounting.domainmaster.v1'),
      createEmptyInstance: create)
    ..aOM<MasterCategory>(1, _omitFieldNames ? '' : 'category',
        subBuilder: MasterCategory.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  CreateCategoryResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  CreateCategoryResponse copyWith(
          void Function(CreateCategoryResponse) updates) =>
      super.copyWith((message) => updates(message as CreateCategoryResponse))
          as CreateCategoryResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static CreateCategoryResponse create() => CreateCategoryResponse._();
  @$core.override
  CreateCategoryResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static CreateCategoryResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<CreateCategoryResponse>(create);
  static CreateCategoryResponse? _defaultInstance;

  @$pb.TagNumber(1)
  MasterCategory get category => $_getN(0);
  @$pb.TagNumber(1)
  set category(MasterCategory value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasCategory() => $_has(0);
  @$pb.TagNumber(1)
  void clearCategory() => $_clearField(1);
  @$pb.TagNumber(1)
  MasterCategory ensureCategory() => $_ensure(0);
}

class UpdateCategoryRequest extends $pb.GeneratedMessage {
  factory UpdateCategoryRequest({
    $core.String? categoryId,
    $core.String? displayName,
    $core.String? description,
    $1.Struct? validationSchema,
    $core.bool? isActive,
    $core.int? sortOrder,
  }) {
    final result = create();
    if (categoryId != null) result.categoryId = categoryId;
    if (displayName != null) result.displayName = displayName;
    if (description != null) result.description = description;
    if (validationSchema != null) result.validationSchema = validationSchema;
    if (isActive != null) result.isActive = isActive;
    if (sortOrder != null) result.sortOrder = sortOrder;
    return result;
  }

  UpdateCategoryRequest._();

  factory UpdateCategoryRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory UpdateCategoryRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'UpdateCategoryRequest',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.business.accounting.domainmaster.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'categoryId')
    ..aOS(2, _omitFieldNames ? '' : 'displayName')
    ..aOS(3, _omitFieldNames ? '' : 'description')
    ..aOM<$1.Struct>(4, _omitFieldNames ? '' : 'validationSchema',
        subBuilder: $1.Struct.create)
    ..aOB(5, _omitFieldNames ? '' : 'isActive')
    ..aI(6, _omitFieldNames ? '' : 'sortOrder')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  UpdateCategoryRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  UpdateCategoryRequest copyWith(
          void Function(UpdateCategoryRequest) updates) =>
      super.copyWith((message) => updates(message as UpdateCategoryRequest))
          as UpdateCategoryRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static UpdateCategoryRequest create() => UpdateCategoryRequest._();
  @$core.override
  UpdateCategoryRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static UpdateCategoryRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<UpdateCategoryRequest>(create);
  static UpdateCategoryRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get categoryId => $_getSZ(0);
  @$pb.TagNumber(1)
  set categoryId($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasCategoryId() => $_has(0);
  @$pb.TagNumber(1)
  void clearCategoryId() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get displayName => $_getSZ(1);
  @$pb.TagNumber(2)
  set displayName($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasDisplayName() => $_has(1);
  @$pb.TagNumber(2)
  void clearDisplayName() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.String get description => $_getSZ(2);
  @$pb.TagNumber(3)
  set description($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasDescription() => $_has(2);
  @$pb.TagNumber(3)
  void clearDescription() => $_clearField(3);

  @$pb.TagNumber(4)
  $1.Struct get validationSchema => $_getN(3);
  @$pb.TagNumber(4)
  set validationSchema($1.Struct value) => $_setField(4, value);
  @$pb.TagNumber(4)
  $core.bool hasValidationSchema() => $_has(3);
  @$pb.TagNumber(4)
  void clearValidationSchema() => $_clearField(4);
  @$pb.TagNumber(4)
  $1.Struct ensureValidationSchema() => $_ensure(3);

  @$pb.TagNumber(5)
  $core.bool get isActive => $_getBF(4);
  @$pb.TagNumber(5)
  set isActive($core.bool value) => $_setBool(4, value);
  @$pb.TagNumber(5)
  $core.bool hasIsActive() => $_has(4);
  @$pb.TagNumber(5)
  void clearIsActive() => $_clearField(5);

  @$pb.TagNumber(6)
  $core.int get sortOrder => $_getIZ(5);
  @$pb.TagNumber(6)
  set sortOrder($core.int value) => $_setSignedInt32(5, value);
  @$pb.TagNumber(6)
  $core.bool hasSortOrder() => $_has(5);
  @$pb.TagNumber(6)
  void clearSortOrder() => $_clearField(6);
}

class UpdateCategoryResponse extends $pb.GeneratedMessage {
  factory UpdateCategoryResponse({
    MasterCategory? category,
  }) {
    final result = create();
    if (category != null) result.category = category;
    return result;
  }

  UpdateCategoryResponse._();

  factory UpdateCategoryResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory UpdateCategoryResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'UpdateCategoryResponse',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.business.accounting.domainmaster.v1'),
      createEmptyInstance: create)
    ..aOM<MasterCategory>(1, _omitFieldNames ? '' : 'category',
        subBuilder: MasterCategory.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  UpdateCategoryResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  UpdateCategoryResponse copyWith(
          void Function(UpdateCategoryResponse) updates) =>
      super.copyWith((message) => updates(message as UpdateCategoryResponse))
          as UpdateCategoryResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static UpdateCategoryResponse create() => UpdateCategoryResponse._();
  @$core.override
  UpdateCategoryResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static UpdateCategoryResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<UpdateCategoryResponse>(create);
  static UpdateCategoryResponse? _defaultInstance;

  @$pb.TagNumber(1)
  MasterCategory get category => $_getN(0);
  @$pb.TagNumber(1)
  set category(MasterCategory value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasCategory() => $_has(0);
  @$pb.TagNumber(1)
  void clearCategory() => $_clearField(1);
  @$pb.TagNumber(1)
  MasterCategory ensureCategory() => $_ensure(0);
}

class DeleteCategoryRequest extends $pb.GeneratedMessage {
  factory DeleteCategoryRequest({
    $core.String? categoryId,
  }) {
    final result = create();
    if (categoryId != null) result.categoryId = categoryId;
    return result;
  }

  DeleteCategoryRequest._();

  factory DeleteCategoryRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory DeleteCategoryRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'DeleteCategoryRequest',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.business.accounting.domainmaster.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'categoryId')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  DeleteCategoryRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  DeleteCategoryRequest copyWith(
          void Function(DeleteCategoryRequest) updates) =>
      super.copyWith((message) => updates(message as DeleteCategoryRequest))
          as DeleteCategoryRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static DeleteCategoryRequest create() => DeleteCategoryRequest._();
  @$core.override
  DeleteCategoryRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static DeleteCategoryRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<DeleteCategoryRequest>(create);
  static DeleteCategoryRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get categoryId => $_getSZ(0);
  @$pb.TagNumber(1)
  set categoryId($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasCategoryId() => $_has(0);
  @$pb.TagNumber(1)
  void clearCategoryId() => $_clearField(1);
}

class DeleteCategoryResponse extends $pb.GeneratedMessage {
  factory DeleteCategoryResponse({
    $core.bool? success,
  }) {
    final result = create();
    if (success != null) result.success = success;
    return result;
  }

  DeleteCategoryResponse._();

  factory DeleteCategoryResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory DeleteCategoryResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'DeleteCategoryResponse',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.business.accounting.domainmaster.v1'),
      createEmptyInstance: create)
    ..aOB(1, _omitFieldNames ? '' : 'success')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  DeleteCategoryResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  DeleteCategoryResponse copyWith(
          void Function(DeleteCategoryResponse) updates) =>
      super.copyWith((message) => updates(message as DeleteCategoryResponse))
          as DeleteCategoryResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static DeleteCategoryResponse create() => DeleteCategoryResponse._();
  @$core.override
  DeleteCategoryResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static DeleteCategoryResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<DeleteCategoryResponse>(create);
  static DeleteCategoryResponse? _defaultInstance;

  @$pb.TagNumber(1)
  $core.bool get success => $_getBF(0);
  @$pb.TagNumber(1)
  set success($core.bool value) => $_setBool(0, value);
  @$pb.TagNumber(1)
  $core.bool hasSuccess() => $_has(0);
  @$pb.TagNumber(1)
  void clearSuccess() => $_clearField(1);
}

class ListItemsRequest extends $pb.GeneratedMessage {
  factory ListItemsRequest({
    $core.String? categoryId,
    $core.bool? activeOnly,
    $2.Pagination? pagination,
  }) {
    final result = create();
    if (categoryId != null) result.categoryId = categoryId;
    if (activeOnly != null) result.activeOnly = activeOnly;
    if (pagination != null) result.pagination = pagination;
    return result;
  }

  ListItemsRequest._();

  factory ListItemsRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory ListItemsRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ListItemsRequest',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.business.accounting.domainmaster.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'categoryId')
    ..aOB(2, _omitFieldNames ? '' : 'activeOnly')
    ..aOM<$2.Pagination>(3, _omitFieldNames ? '' : 'pagination',
        subBuilder: $2.Pagination.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListItemsRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListItemsRequest copyWith(void Function(ListItemsRequest) updates) =>
      super.copyWith((message) => updates(message as ListItemsRequest))
          as ListItemsRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static ListItemsRequest create() => ListItemsRequest._();
  @$core.override
  ListItemsRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static ListItemsRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ListItemsRequest>(create);
  static ListItemsRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get categoryId => $_getSZ(0);
  @$pb.TagNumber(1)
  set categoryId($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasCategoryId() => $_has(0);
  @$pb.TagNumber(1)
  void clearCategoryId() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.bool get activeOnly => $_getBF(1);
  @$pb.TagNumber(2)
  set activeOnly($core.bool value) => $_setBool(1, value);
  @$pb.TagNumber(2)
  $core.bool hasActiveOnly() => $_has(1);
  @$pb.TagNumber(2)
  void clearActiveOnly() => $_clearField(2);

  @$pb.TagNumber(3)
  $2.Pagination get pagination => $_getN(2);
  @$pb.TagNumber(3)
  set pagination($2.Pagination value) => $_setField(3, value);
  @$pb.TagNumber(3)
  $core.bool hasPagination() => $_has(2);
  @$pb.TagNumber(3)
  void clearPagination() => $_clearField(3);
  @$pb.TagNumber(3)
  $2.Pagination ensurePagination() => $_ensure(2);
}

class ListItemsResponse extends $pb.GeneratedMessage {
  factory ListItemsResponse({
    $core.Iterable<MasterItem>? items,
    $2.PaginationResult? pagination,
  }) {
    final result = create();
    if (items != null) result.items.addAll(items);
    if (pagination != null) result.pagination = pagination;
    return result;
  }

  ListItemsResponse._();

  factory ListItemsResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory ListItemsResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ListItemsResponse',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.business.accounting.domainmaster.v1'),
      createEmptyInstance: create)
    ..pPM<MasterItem>(1, _omitFieldNames ? '' : 'items',
        subBuilder: MasterItem.create)
    ..aOM<$2.PaginationResult>(2, _omitFieldNames ? '' : 'pagination',
        subBuilder: $2.PaginationResult.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListItemsResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListItemsResponse copyWith(void Function(ListItemsResponse) updates) =>
      super.copyWith((message) => updates(message as ListItemsResponse))
          as ListItemsResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static ListItemsResponse create() => ListItemsResponse._();
  @$core.override
  ListItemsResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static ListItemsResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ListItemsResponse>(create);
  static ListItemsResponse? _defaultInstance;

  @$pb.TagNumber(1)
  $pb.PbList<MasterItem> get items => $_getList(0);

  @$pb.TagNumber(2)
  $2.PaginationResult get pagination => $_getN(1);
  @$pb.TagNumber(2)
  set pagination($2.PaginationResult value) => $_setField(2, value);
  @$pb.TagNumber(2)
  $core.bool hasPagination() => $_has(1);
  @$pb.TagNumber(2)
  void clearPagination() => $_clearField(2);
  @$pb.TagNumber(2)
  $2.PaginationResult ensurePagination() => $_ensure(1);
}

class GetItemRequest extends $pb.GeneratedMessage {
  factory GetItemRequest({
    $core.String? itemId,
  }) {
    final result = create();
    if (itemId != null) result.itemId = itemId;
    return result;
  }

  GetItemRequest._();

  factory GetItemRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory GetItemRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'GetItemRequest',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.business.accounting.domainmaster.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'itemId')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetItemRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetItemRequest copyWith(void Function(GetItemRequest) updates) =>
      super.copyWith((message) => updates(message as GetItemRequest))
          as GetItemRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static GetItemRequest create() => GetItemRequest._();
  @$core.override
  GetItemRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static GetItemRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<GetItemRequest>(create);
  static GetItemRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get itemId => $_getSZ(0);
  @$pb.TagNumber(1)
  set itemId($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasItemId() => $_has(0);
  @$pb.TagNumber(1)
  void clearItemId() => $_clearField(1);
}

class GetItemResponse extends $pb.GeneratedMessage {
  factory GetItemResponse({
    MasterItem? item,
  }) {
    final result = create();
    if (item != null) result.item = item;
    return result;
  }

  GetItemResponse._();

  factory GetItemResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory GetItemResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'GetItemResponse',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.business.accounting.domainmaster.v1'),
      createEmptyInstance: create)
    ..aOM<MasterItem>(1, _omitFieldNames ? '' : 'item',
        subBuilder: MasterItem.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetItemResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetItemResponse copyWith(void Function(GetItemResponse) updates) =>
      super.copyWith((message) => updates(message as GetItemResponse))
          as GetItemResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static GetItemResponse create() => GetItemResponse._();
  @$core.override
  GetItemResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static GetItemResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<GetItemResponse>(create);
  static GetItemResponse? _defaultInstance;

  @$pb.TagNumber(1)
  MasterItem get item => $_getN(0);
  @$pb.TagNumber(1)
  set item(MasterItem value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasItem() => $_has(0);
  @$pb.TagNumber(1)
  void clearItem() => $_clearField(1);
  @$pb.TagNumber(1)
  MasterItem ensureItem() => $_ensure(0);
}

class CreateItemRequest extends $pb.GeneratedMessage {
  factory CreateItemRequest({
    $core.String? categoryId,
    $core.String? code,
    $core.String? displayName,
    $core.String? description,
    $1.Struct? attributes,
    $core.String? parentItemId,
    $2.Timestamp? effectiveFrom,
    $2.Timestamp? effectiveUntil,
    $core.bool? isActive,
    $core.int? sortOrder,
  }) {
    final result = create();
    if (categoryId != null) result.categoryId = categoryId;
    if (code != null) result.code = code;
    if (displayName != null) result.displayName = displayName;
    if (description != null) result.description = description;
    if (attributes != null) result.attributes = attributes;
    if (parentItemId != null) result.parentItemId = parentItemId;
    if (effectiveFrom != null) result.effectiveFrom = effectiveFrom;
    if (effectiveUntil != null) result.effectiveUntil = effectiveUntil;
    if (isActive != null) result.isActive = isActive;
    if (sortOrder != null) result.sortOrder = sortOrder;
    return result;
  }

  CreateItemRequest._();

  factory CreateItemRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory CreateItemRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'CreateItemRequest',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.business.accounting.domainmaster.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'categoryId')
    ..aOS(2, _omitFieldNames ? '' : 'code')
    ..aOS(3, _omitFieldNames ? '' : 'displayName')
    ..aOS(4, _omitFieldNames ? '' : 'description')
    ..aOM<$1.Struct>(5, _omitFieldNames ? '' : 'attributes',
        subBuilder: $1.Struct.create)
    ..aOS(6, _omitFieldNames ? '' : 'parentItemId')
    ..aOM<$2.Timestamp>(7, _omitFieldNames ? '' : 'effectiveFrom',
        subBuilder: $2.Timestamp.create)
    ..aOM<$2.Timestamp>(8, _omitFieldNames ? '' : 'effectiveUntil',
        subBuilder: $2.Timestamp.create)
    ..aOB(9, _omitFieldNames ? '' : 'isActive')
    ..aI(10, _omitFieldNames ? '' : 'sortOrder')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  CreateItemRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  CreateItemRequest copyWith(void Function(CreateItemRequest) updates) =>
      super.copyWith((message) => updates(message as CreateItemRequest))
          as CreateItemRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static CreateItemRequest create() => CreateItemRequest._();
  @$core.override
  CreateItemRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static CreateItemRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<CreateItemRequest>(create);
  static CreateItemRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get categoryId => $_getSZ(0);
  @$pb.TagNumber(1)
  set categoryId($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasCategoryId() => $_has(0);
  @$pb.TagNumber(1)
  void clearCategoryId() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get code => $_getSZ(1);
  @$pb.TagNumber(2)
  set code($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasCode() => $_has(1);
  @$pb.TagNumber(2)
  void clearCode() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.String get displayName => $_getSZ(2);
  @$pb.TagNumber(3)
  set displayName($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasDisplayName() => $_has(2);
  @$pb.TagNumber(3)
  void clearDisplayName() => $_clearField(3);

  @$pb.TagNumber(4)
  $core.String get description => $_getSZ(3);
  @$pb.TagNumber(4)
  set description($core.String value) => $_setString(3, value);
  @$pb.TagNumber(4)
  $core.bool hasDescription() => $_has(3);
  @$pb.TagNumber(4)
  void clearDescription() => $_clearField(4);

  @$pb.TagNumber(5)
  $1.Struct get attributes => $_getN(4);
  @$pb.TagNumber(5)
  set attributes($1.Struct value) => $_setField(5, value);
  @$pb.TagNumber(5)
  $core.bool hasAttributes() => $_has(4);
  @$pb.TagNumber(5)
  void clearAttributes() => $_clearField(5);
  @$pb.TagNumber(5)
  $1.Struct ensureAttributes() => $_ensure(4);

  @$pb.TagNumber(6)
  $core.String get parentItemId => $_getSZ(5);
  @$pb.TagNumber(6)
  set parentItemId($core.String value) => $_setString(5, value);
  @$pb.TagNumber(6)
  $core.bool hasParentItemId() => $_has(5);
  @$pb.TagNumber(6)
  void clearParentItemId() => $_clearField(6);

  @$pb.TagNumber(7)
  $2.Timestamp get effectiveFrom => $_getN(6);
  @$pb.TagNumber(7)
  set effectiveFrom($2.Timestamp value) => $_setField(7, value);
  @$pb.TagNumber(7)
  $core.bool hasEffectiveFrom() => $_has(6);
  @$pb.TagNumber(7)
  void clearEffectiveFrom() => $_clearField(7);
  @$pb.TagNumber(7)
  $2.Timestamp ensureEffectiveFrom() => $_ensure(6);

  @$pb.TagNumber(8)
  $2.Timestamp get effectiveUntil => $_getN(7);
  @$pb.TagNumber(8)
  set effectiveUntil($2.Timestamp value) => $_setField(8, value);
  @$pb.TagNumber(8)
  $core.bool hasEffectiveUntil() => $_has(7);
  @$pb.TagNumber(8)
  void clearEffectiveUntil() => $_clearField(8);
  @$pb.TagNumber(8)
  $2.Timestamp ensureEffectiveUntil() => $_ensure(7);

  @$pb.TagNumber(9)
  $core.bool get isActive => $_getBF(8);
  @$pb.TagNumber(9)
  set isActive($core.bool value) => $_setBool(8, value);
  @$pb.TagNumber(9)
  $core.bool hasIsActive() => $_has(8);
  @$pb.TagNumber(9)
  void clearIsActive() => $_clearField(9);

  @$pb.TagNumber(10)
  $core.int get sortOrder => $_getIZ(9);
  @$pb.TagNumber(10)
  set sortOrder($core.int value) => $_setSignedInt32(9, value);
  @$pb.TagNumber(10)
  $core.bool hasSortOrder() => $_has(9);
  @$pb.TagNumber(10)
  void clearSortOrder() => $_clearField(10);
}

class CreateItemResponse extends $pb.GeneratedMessage {
  factory CreateItemResponse({
    MasterItem? item,
  }) {
    final result = create();
    if (item != null) result.item = item;
    return result;
  }

  CreateItemResponse._();

  factory CreateItemResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory CreateItemResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'CreateItemResponse',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.business.accounting.domainmaster.v1'),
      createEmptyInstance: create)
    ..aOM<MasterItem>(1, _omitFieldNames ? '' : 'item',
        subBuilder: MasterItem.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  CreateItemResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  CreateItemResponse copyWith(void Function(CreateItemResponse) updates) =>
      super.copyWith((message) => updates(message as CreateItemResponse))
          as CreateItemResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static CreateItemResponse create() => CreateItemResponse._();
  @$core.override
  CreateItemResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static CreateItemResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<CreateItemResponse>(create);
  static CreateItemResponse? _defaultInstance;

  @$pb.TagNumber(1)
  MasterItem get item => $_getN(0);
  @$pb.TagNumber(1)
  set item(MasterItem value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasItem() => $_has(0);
  @$pb.TagNumber(1)
  void clearItem() => $_clearField(1);
  @$pb.TagNumber(1)
  MasterItem ensureItem() => $_ensure(0);
}

class UpdateItemRequest extends $pb.GeneratedMessage {
  factory UpdateItemRequest({
    $core.String? itemId,
    $core.String? displayName,
    $core.String? description,
    $1.Struct? attributes,
    $core.String? parentItemId,
    $2.Timestamp? effectiveFrom,
    $2.Timestamp? effectiveUntil,
    $core.bool? isActive,
    $core.int? sortOrder,
  }) {
    final result = create();
    if (itemId != null) result.itemId = itemId;
    if (displayName != null) result.displayName = displayName;
    if (description != null) result.description = description;
    if (attributes != null) result.attributes = attributes;
    if (parentItemId != null) result.parentItemId = parentItemId;
    if (effectiveFrom != null) result.effectiveFrom = effectiveFrom;
    if (effectiveUntil != null) result.effectiveUntil = effectiveUntil;
    if (isActive != null) result.isActive = isActive;
    if (sortOrder != null) result.sortOrder = sortOrder;
    return result;
  }

  UpdateItemRequest._();

  factory UpdateItemRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory UpdateItemRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'UpdateItemRequest',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.business.accounting.domainmaster.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'itemId')
    ..aOS(2, _omitFieldNames ? '' : 'displayName')
    ..aOS(3, _omitFieldNames ? '' : 'description')
    ..aOM<$1.Struct>(4, _omitFieldNames ? '' : 'attributes',
        subBuilder: $1.Struct.create)
    ..aOS(5, _omitFieldNames ? '' : 'parentItemId')
    ..aOM<$2.Timestamp>(6, _omitFieldNames ? '' : 'effectiveFrom',
        subBuilder: $2.Timestamp.create)
    ..aOM<$2.Timestamp>(7, _omitFieldNames ? '' : 'effectiveUntil',
        subBuilder: $2.Timestamp.create)
    ..aOB(8, _omitFieldNames ? '' : 'isActive')
    ..aI(9, _omitFieldNames ? '' : 'sortOrder')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  UpdateItemRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  UpdateItemRequest copyWith(void Function(UpdateItemRequest) updates) =>
      super.copyWith((message) => updates(message as UpdateItemRequest))
          as UpdateItemRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static UpdateItemRequest create() => UpdateItemRequest._();
  @$core.override
  UpdateItemRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static UpdateItemRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<UpdateItemRequest>(create);
  static UpdateItemRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get itemId => $_getSZ(0);
  @$pb.TagNumber(1)
  set itemId($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasItemId() => $_has(0);
  @$pb.TagNumber(1)
  void clearItemId() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get displayName => $_getSZ(1);
  @$pb.TagNumber(2)
  set displayName($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasDisplayName() => $_has(1);
  @$pb.TagNumber(2)
  void clearDisplayName() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.String get description => $_getSZ(2);
  @$pb.TagNumber(3)
  set description($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasDescription() => $_has(2);
  @$pb.TagNumber(3)
  void clearDescription() => $_clearField(3);

  @$pb.TagNumber(4)
  $1.Struct get attributes => $_getN(3);
  @$pb.TagNumber(4)
  set attributes($1.Struct value) => $_setField(4, value);
  @$pb.TagNumber(4)
  $core.bool hasAttributes() => $_has(3);
  @$pb.TagNumber(4)
  void clearAttributes() => $_clearField(4);
  @$pb.TagNumber(4)
  $1.Struct ensureAttributes() => $_ensure(3);

  @$pb.TagNumber(5)
  $core.String get parentItemId => $_getSZ(4);
  @$pb.TagNumber(5)
  set parentItemId($core.String value) => $_setString(4, value);
  @$pb.TagNumber(5)
  $core.bool hasParentItemId() => $_has(4);
  @$pb.TagNumber(5)
  void clearParentItemId() => $_clearField(5);

  @$pb.TagNumber(6)
  $2.Timestamp get effectiveFrom => $_getN(5);
  @$pb.TagNumber(6)
  set effectiveFrom($2.Timestamp value) => $_setField(6, value);
  @$pb.TagNumber(6)
  $core.bool hasEffectiveFrom() => $_has(5);
  @$pb.TagNumber(6)
  void clearEffectiveFrom() => $_clearField(6);
  @$pb.TagNumber(6)
  $2.Timestamp ensureEffectiveFrom() => $_ensure(5);

  @$pb.TagNumber(7)
  $2.Timestamp get effectiveUntil => $_getN(6);
  @$pb.TagNumber(7)
  set effectiveUntil($2.Timestamp value) => $_setField(7, value);
  @$pb.TagNumber(7)
  $core.bool hasEffectiveUntil() => $_has(6);
  @$pb.TagNumber(7)
  void clearEffectiveUntil() => $_clearField(7);
  @$pb.TagNumber(7)
  $2.Timestamp ensureEffectiveUntil() => $_ensure(6);

  @$pb.TagNumber(8)
  $core.bool get isActive => $_getBF(7);
  @$pb.TagNumber(8)
  set isActive($core.bool value) => $_setBool(7, value);
  @$pb.TagNumber(8)
  $core.bool hasIsActive() => $_has(7);
  @$pb.TagNumber(8)
  void clearIsActive() => $_clearField(8);

  @$pb.TagNumber(9)
  $core.int get sortOrder => $_getIZ(8);
  @$pb.TagNumber(9)
  set sortOrder($core.int value) => $_setSignedInt32(8, value);
  @$pb.TagNumber(9)
  $core.bool hasSortOrder() => $_has(8);
  @$pb.TagNumber(9)
  void clearSortOrder() => $_clearField(9);
}

class UpdateItemResponse extends $pb.GeneratedMessage {
  factory UpdateItemResponse({
    MasterItem? item,
  }) {
    final result = create();
    if (item != null) result.item = item;
    return result;
  }

  UpdateItemResponse._();

  factory UpdateItemResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory UpdateItemResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'UpdateItemResponse',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.business.accounting.domainmaster.v1'),
      createEmptyInstance: create)
    ..aOM<MasterItem>(1, _omitFieldNames ? '' : 'item',
        subBuilder: MasterItem.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  UpdateItemResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  UpdateItemResponse copyWith(void Function(UpdateItemResponse) updates) =>
      super.copyWith((message) => updates(message as UpdateItemResponse))
          as UpdateItemResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static UpdateItemResponse create() => UpdateItemResponse._();
  @$core.override
  UpdateItemResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static UpdateItemResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<UpdateItemResponse>(create);
  static UpdateItemResponse? _defaultInstance;

  @$pb.TagNumber(1)
  MasterItem get item => $_getN(0);
  @$pb.TagNumber(1)
  set item(MasterItem value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasItem() => $_has(0);
  @$pb.TagNumber(1)
  void clearItem() => $_clearField(1);
  @$pb.TagNumber(1)
  MasterItem ensureItem() => $_ensure(0);
}

class DeleteItemRequest extends $pb.GeneratedMessage {
  factory DeleteItemRequest({
    $core.String? itemId,
  }) {
    final result = create();
    if (itemId != null) result.itemId = itemId;
    return result;
  }

  DeleteItemRequest._();

  factory DeleteItemRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory DeleteItemRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'DeleteItemRequest',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.business.accounting.domainmaster.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'itemId')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  DeleteItemRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  DeleteItemRequest copyWith(void Function(DeleteItemRequest) updates) =>
      super.copyWith((message) => updates(message as DeleteItemRequest))
          as DeleteItemRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static DeleteItemRequest create() => DeleteItemRequest._();
  @$core.override
  DeleteItemRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static DeleteItemRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<DeleteItemRequest>(create);
  static DeleteItemRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get itemId => $_getSZ(0);
  @$pb.TagNumber(1)
  set itemId($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasItemId() => $_has(0);
  @$pb.TagNumber(1)
  void clearItemId() => $_clearField(1);
}

class DeleteItemResponse extends $pb.GeneratedMessage {
  factory DeleteItemResponse({
    $core.bool? success,
  }) {
    final result = create();
    if (success != null) result.success = success;
    return result;
  }

  DeleteItemResponse._();

  factory DeleteItemResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory DeleteItemResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'DeleteItemResponse',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.business.accounting.domainmaster.v1'),
      createEmptyInstance: create)
    ..aOB(1, _omitFieldNames ? '' : 'success')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  DeleteItemResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  DeleteItemResponse copyWith(void Function(DeleteItemResponse) updates) =>
      super.copyWith((message) => updates(message as DeleteItemResponse))
          as DeleteItemResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static DeleteItemResponse create() => DeleteItemResponse._();
  @$core.override
  DeleteItemResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static DeleteItemResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<DeleteItemResponse>(create);
  static DeleteItemResponse? _defaultInstance;

  @$pb.TagNumber(1)
  $core.bool get success => $_getBF(0);
  @$pb.TagNumber(1)
  set success($core.bool value) => $_setBool(0, value);
  @$pb.TagNumber(1)
  $core.bool hasSuccess() => $_has(0);
  @$pb.TagNumber(1)
  void clearSuccess() => $_clearField(1);
}

class ListItemVersionsRequest extends $pb.GeneratedMessage {
  factory ListItemVersionsRequest({
    $core.String? itemId,
    $2.Pagination? pagination,
  }) {
    final result = create();
    if (itemId != null) result.itemId = itemId;
    if (pagination != null) result.pagination = pagination;
    return result;
  }

  ListItemVersionsRequest._();

  factory ListItemVersionsRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory ListItemVersionsRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ListItemVersionsRequest',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.business.accounting.domainmaster.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'itemId')
    ..aOM<$2.Pagination>(2, _omitFieldNames ? '' : 'pagination',
        subBuilder: $2.Pagination.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListItemVersionsRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListItemVersionsRequest copyWith(
          void Function(ListItemVersionsRequest) updates) =>
      super.copyWith((message) => updates(message as ListItemVersionsRequest))
          as ListItemVersionsRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static ListItemVersionsRequest create() => ListItemVersionsRequest._();
  @$core.override
  ListItemVersionsRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static ListItemVersionsRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ListItemVersionsRequest>(create);
  static ListItemVersionsRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get itemId => $_getSZ(0);
  @$pb.TagNumber(1)
  set itemId($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasItemId() => $_has(0);
  @$pb.TagNumber(1)
  void clearItemId() => $_clearField(1);

  @$pb.TagNumber(2)
  $2.Pagination get pagination => $_getN(1);
  @$pb.TagNumber(2)
  set pagination($2.Pagination value) => $_setField(2, value);
  @$pb.TagNumber(2)
  $core.bool hasPagination() => $_has(1);
  @$pb.TagNumber(2)
  void clearPagination() => $_clearField(2);
  @$pb.TagNumber(2)
  $2.Pagination ensurePagination() => $_ensure(1);
}

class ListItemVersionsResponse extends $pb.GeneratedMessage {
  factory ListItemVersionsResponse({
    $core.Iterable<MasterItemVersion>? versions,
    $2.PaginationResult? pagination,
  }) {
    final result = create();
    if (versions != null) result.versions.addAll(versions);
    if (pagination != null) result.pagination = pagination;
    return result;
  }

  ListItemVersionsResponse._();

  factory ListItemVersionsResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory ListItemVersionsResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ListItemVersionsResponse',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.business.accounting.domainmaster.v1'),
      createEmptyInstance: create)
    ..pPM<MasterItemVersion>(1, _omitFieldNames ? '' : 'versions',
        subBuilder: MasterItemVersion.create)
    ..aOM<$2.PaginationResult>(2, _omitFieldNames ? '' : 'pagination',
        subBuilder: $2.PaginationResult.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListItemVersionsResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListItemVersionsResponse copyWith(
          void Function(ListItemVersionsResponse) updates) =>
      super.copyWith((message) => updates(message as ListItemVersionsResponse))
          as ListItemVersionsResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static ListItemVersionsResponse create() => ListItemVersionsResponse._();
  @$core.override
  ListItemVersionsResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static ListItemVersionsResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ListItemVersionsResponse>(create);
  static ListItemVersionsResponse? _defaultInstance;

  @$pb.TagNumber(1)
  $pb.PbList<MasterItemVersion> get versions => $_getList(0);

  @$pb.TagNumber(2)
  $2.PaginationResult get pagination => $_getN(1);
  @$pb.TagNumber(2)
  set pagination($2.PaginationResult value) => $_setField(2, value);
  @$pb.TagNumber(2)
  $core.bool hasPagination() => $_has(1);
  @$pb.TagNumber(2)
  void clearPagination() => $_clearField(2);
  @$pb.TagNumber(2)
  $2.PaginationResult ensurePagination() => $_ensure(1);
}

class GetTenantExtensionRequest extends $pb.GeneratedMessage {
  factory GetTenantExtensionRequest({
    $core.String? tenantId,
    $core.String? itemId,
  }) {
    final result = create();
    if (tenantId != null) result.tenantId = tenantId;
    if (itemId != null) result.itemId = itemId;
    return result;
  }

  GetTenantExtensionRequest._();

  factory GetTenantExtensionRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory GetTenantExtensionRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'GetTenantExtensionRequest',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.business.accounting.domainmaster.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'tenantId')
    ..aOS(2, _omitFieldNames ? '' : 'itemId')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetTenantExtensionRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetTenantExtensionRequest copyWith(
          void Function(GetTenantExtensionRequest) updates) =>
      super.copyWith((message) => updates(message as GetTenantExtensionRequest))
          as GetTenantExtensionRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static GetTenantExtensionRequest create() => GetTenantExtensionRequest._();
  @$core.override
  GetTenantExtensionRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static GetTenantExtensionRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<GetTenantExtensionRequest>(create);
  static GetTenantExtensionRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get tenantId => $_getSZ(0);
  @$pb.TagNumber(1)
  set tenantId($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasTenantId() => $_has(0);
  @$pb.TagNumber(1)
  void clearTenantId() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get itemId => $_getSZ(1);
  @$pb.TagNumber(2)
  set itemId($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasItemId() => $_has(1);
  @$pb.TagNumber(2)
  void clearItemId() => $_clearField(2);
}

class GetTenantExtensionResponse extends $pb.GeneratedMessage {
  factory GetTenantExtensionResponse({
    TenantMasterExtension? extension_1,
  }) {
    final result = create();
    if (extension_1 != null) result.extension_1 = extension_1;
    return result;
  }

  GetTenantExtensionResponse._();

  factory GetTenantExtensionResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory GetTenantExtensionResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'GetTenantExtensionResponse',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.business.accounting.domainmaster.v1'),
      createEmptyInstance: create)
    ..aOM<TenantMasterExtension>(1, _omitFieldNames ? '' : 'extension',
        subBuilder: TenantMasterExtension.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetTenantExtensionResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetTenantExtensionResponse copyWith(
          void Function(GetTenantExtensionResponse) updates) =>
      super.copyWith(
              (message) => updates(message as GetTenantExtensionResponse))
          as GetTenantExtensionResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static GetTenantExtensionResponse create() => GetTenantExtensionResponse._();
  @$core.override
  GetTenantExtensionResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static GetTenantExtensionResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<GetTenantExtensionResponse>(create);
  static GetTenantExtensionResponse? _defaultInstance;

  @$pb.TagNumber(1)
  TenantMasterExtension get extension_1 => $_getN(0);
  @$pb.TagNumber(1)
  set extension_1(TenantMasterExtension value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasExtension_1() => $_has(0);
  @$pb.TagNumber(1)
  void clearExtension_1() => $_clearField(1);
  @$pb.TagNumber(1)
  TenantMasterExtension ensureExtension_1() => $_ensure(0);
}

class UpsertTenantExtensionRequest extends $pb.GeneratedMessage {
  factory UpsertTenantExtensionRequest({
    $core.String? tenantId,
    $core.String? itemId,
    $core.String? displayNameOverride,
    $1.Struct? attributesOverride,
    $core.bool? isEnabled,
  }) {
    final result = create();
    if (tenantId != null) result.tenantId = tenantId;
    if (itemId != null) result.itemId = itemId;
    if (displayNameOverride != null)
      result.displayNameOverride = displayNameOverride;
    if (attributesOverride != null)
      result.attributesOverride = attributesOverride;
    if (isEnabled != null) result.isEnabled = isEnabled;
    return result;
  }

  UpsertTenantExtensionRequest._();

  factory UpsertTenantExtensionRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory UpsertTenantExtensionRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'UpsertTenantExtensionRequest',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.business.accounting.domainmaster.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'tenantId')
    ..aOS(2, _omitFieldNames ? '' : 'itemId')
    ..aOS(3, _omitFieldNames ? '' : 'displayNameOverride')
    ..aOM<$1.Struct>(4, _omitFieldNames ? '' : 'attributesOverride',
        subBuilder: $1.Struct.create)
    ..aOB(5, _omitFieldNames ? '' : 'isEnabled')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  UpsertTenantExtensionRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  UpsertTenantExtensionRequest copyWith(
          void Function(UpsertTenantExtensionRequest) updates) =>
      super.copyWith(
              (message) => updates(message as UpsertTenantExtensionRequest))
          as UpsertTenantExtensionRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static UpsertTenantExtensionRequest create() =>
      UpsertTenantExtensionRequest._();
  @$core.override
  UpsertTenantExtensionRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static UpsertTenantExtensionRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<UpsertTenantExtensionRequest>(create);
  static UpsertTenantExtensionRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get tenantId => $_getSZ(0);
  @$pb.TagNumber(1)
  set tenantId($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasTenantId() => $_has(0);
  @$pb.TagNumber(1)
  void clearTenantId() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get itemId => $_getSZ(1);
  @$pb.TagNumber(2)
  set itemId($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasItemId() => $_has(1);
  @$pb.TagNumber(2)
  void clearItemId() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.String get displayNameOverride => $_getSZ(2);
  @$pb.TagNumber(3)
  set displayNameOverride($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasDisplayNameOverride() => $_has(2);
  @$pb.TagNumber(3)
  void clearDisplayNameOverride() => $_clearField(3);

  @$pb.TagNumber(4)
  $1.Struct get attributesOverride => $_getN(3);
  @$pb.TagNumber(4)
  set attributesOverride($1.Struct value) => $_setField(4, value);
  @$pb.TagNumber(4)
  $core.bool hasAttributesOverride() => $_has(3);
  @$pb.TagNumber(4)
  void clearAttributesOverride() => $_clearField(4);
  @$pb.TagNumber(4)
  $1.Struct ensureAttributesOverride() => $_ensure(3);

  @$pb.TagNumber(5)
  $core.bool get isEnabled => $_getBF(4);
  @$pb.TagNumber(5)
  set isEnabled($core.bool value) => $_setBool(4, value);
  @$pb.TagNumber(5)
  $core.bool hasIsEnabled() => $_has(4);
  @$pb.TagNumber(5)
  void clearIsEnabled() => $_clearField(5);
}

class UpsertTenantExtensionResponse extends $pb.GeneratedMessage {
  factory UpsertTenantExtensionResponse({
    TenantMasterExtension? extension_1,
  }) {
    final result = create();
    if (extension_1 != null) result.extension_1 = extension_1;
    return result;
  }

  UpsertTenantExtensionResponse._();

  factory UpsertTenantExtensionResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory UpsertTenantExtensionResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'UpsertTenantExtensionResponse',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.business.accounting.domainmaster.v1'),
      createEmptyInstance: create)
    ..aOM<TenantMasterExtension>(1, _omitFieldNames ? '' : 'extension',
        subBuilder: TenantMasterExtension.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  UpsertTenantExtensionResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  UpsertTenantExtensionResponse copyWith(
          void Function(UpsertTenantExtensionResponse) updates) =>
      super.copyWith(
              (message) => updates(message as UpsertTenantExtensionResponse))
          as UpsertTenantExtensionResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static UpsertTenantExtensionResponse create() =>
      UpsertTenantExtensionResponse._();
  @$core.override
  UpsertTenantExtensionResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static UpsertTenantExtensionResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<UpsertTenantExtensionResponse>(create);
  static UpsertTenantExtensionResponse? _defaultInstance;

  @$pb.TagNumber(1)
  TenantMasterExtension get extension_1 => $_getN(0);
  @$pb.TagNumber(1)
  set extension_1(TenantMasterExtension value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasExtension_1() => $_has(0);
  @$pb.TagNumber(1)
  void clearExtension_1() => $_clearField(1);
  @$pb.TagNumber(1)
  TenantMasterExtension ensureExtension_1() => $_ensure(0);
}

class DeleteTenantExtensionRequest extends $pb.GeneratedMessage {
  factory DeleteTenantExtensionRequest({
    $core.String? tenantId,
    $core.String? itemId,
  }) {
    final result = create();
    if (tenantId != null) result.tenantId = tenantId;
    if (itemId != null) result.itemId = itemId;
    return result;
  }

  DeleteTenantExtensionRequest._();

  factory DeleteTenantExtensionRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory DeleteTenantExtensionRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'DeleteTenantExtensionRequest',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.business.accounting.domainmaster.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'tenantId')
    ..aOS(2, _omitFieldNames ? '' : 'itemId')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  DeleteTenantExtensionRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  DeleteTenantExtensionRequest copyWith(
          void Function(DeleteTenantExtensionRequest) updates) =>
      super.copyWith(
              (message) => updates(message as DeleteTenantExtensionRequest))
          as DeleteTenantExtensionRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static DeleteTenantExtensionRequest create() =>
      DeleteTenantExtensionRequest._();
  @$core.override
  DeleteTenantExtensionRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static DeleteTenantExtensionRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<DeleteTenantExtensionRequest>(create);
  static DeleteTenantExtensionRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get tenantId => $_getSZ(0);
  @$pb.TagNumber(1)
  set tenantId($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasTenantId() => $_has(0);
  @$pb.TagNumber(1)
  void clearTenantId() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get itemId => $_getSZ(1);
  @$pb.TagNumber(2)
  set itemId($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasItemId() => $_has(1);
  @$pb.TagNumber(2)
  void clearItemId() => $_clearField(2);
}

class DeleteTenantExtensionResponse extends $pb.GeneratedMessage {
  factory DeleteTenantExtensionResponse({
    $core.bool? success,
  }) {
    final result = create();
    if (success != null) result.success = success;
    return result;
  }

  DeleteTenantExtensionResponse._();

  factory DeleteTenantExtensionResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory DeleteTenantExtensionResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'DeleteTenantExtensionResponse',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.business.accounting.domainmaster.v1'),
      createEmptyInstance: create)
    ..aOB(1, _omitFieldNames ? '' : 'success')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  DeleteTenantExtensionResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  DeleteTenantExtensionResponse copyWith(
          void Function(DeleteTenantExtensionResponse) updates) =>
      super.copyWith(
              (message) => updates(message as DeleteTenantExtensionResponse))
          as DeleteTenantExtensionResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static DeleteTenantExtensionResponse create() =>
      DeleteTenantExtensionResponse._();
  @$core.override
  DeleteTenantExtensionResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static DeleteTenantExtensionResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<DeleteTenantExtensionResponse>(create);
  static DeleteTenantExtensionResponse? _defaultInstance;

  @$pb.TagNumber(1)
  $core.bool get success => $_getBF(0);
  @$pb.TagNumber(1)
  set success($core.bool value) => $_setBool(0, value);
  @$pb.TagNumber(1)
  $core.bool hasSuccess() => $_has(0);
  @$pb.TagNumber(1)
  void clearSuccess() => $_clearField(1);
}

class ListTenantItemsRequest extends $pb.GeneratedMessage {
  factory ListTenantItemsRequest({
    $core.String? tenantId,
    $core.String? categoryId,
    $core.bool? activeOnly,
    $2.Pagination? pagination,
  }) {
    final result = create();
    if (tenantId != null) result.tenantId = tenantId;
    if (categoryId != null) result.categoryId = categoryId;
    if (activeOnly != null) result.activeOnly = activeOnly;
    if (pagination != null) result.pagination = pagination;
    return result;
  }

  ListTenantItemsRequest._();

  factory ListTenantItemsRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory ListTenantItemsRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ListTenantItemsRequest',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.business.accounting.domainmaster.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'tenantId')
    ..aOS(2, _omitFieldNames ? '' : 'categoryId')
    ..aOB(3, _omitFieldNames ? '' : 'activeOnly')
    ..aOM<$2.Pagination>(4, _omitFieldNames ? '' : 'pagination',
        subBuilder: $2.Pagination.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListTenantItemsRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListTenantItemsRequest copyWith(
          void Function(ListTenantItemsRequest) updates) =>
      super.copyWith((message) => updates(message as ListTenantItemsRequest))
          as ListTenantItemsRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static ListTenantItemsRequest create() => ListTenantItemsRequest._();
  @$core.override
  ListTenantItemsRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static ListTenantItemsRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ListTenantItemsRequest>(create);
  static ListTenantItemsRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get tenantId => $_getSZ(0);
  @$pb.TagNumber(1)
  set tenantId($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasTenantId() => $_has(0);
  @$pb.TagNumber(1)
  void clearTenantId() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get categoryId => $_getSZ(1);
  @$pb.TagNumber(2)
  set categoryId($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasCategoryId() => $_has(1);
  @$pb.TagNumber(2)
  void clearCategoryId() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.bool get activeOnly => $_getBF(2);
  @$pb.TagNumber(3)
  set activeOnly($core.bool value) => $_setBool(2, value);
  @$pb.TagNumber(3)
  $core.bool hasActiveOnly() => $_has(2);
  @$pb.TagNumber(3)
  void clearActiveOnly() => $_clearField(3);

  @$pb.TagNumber(4)
  $2.Pagination get pagination => $_getN(3);
  @$pb.TagNumber(4)
  set pagination($2.Pagination value) => $_setField(4, value);
  @$pb.TagNumber(4)
  $core.bool hasPagination() => $_has(3);
  @$pb.TagNumber(4)
  void clearPagination() => $_clearField(4);
  @$pb.TagNumber(4)
  $2.Pagination ensurePagination() => $_ensure(3);
}

class ListTenantItemsResponse extends $pb.GeneratedMessage {
  factory ListTenantItemsResponse({
    $core.Iterable<TenantMergedItem>? items,
    $2.PaginationResult? pagination,
  }) {
    final result = create();
    if (items != null) result.items.addAll(items);
    if (pagination != null) result.pagination = pagination;
    return result;
  }

  ListTenantItemsResponse._();

  factory ListTenantItemsResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory ListTenantItemsResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ListTenantItemsResponse',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.business.accounting.domainmaster.v1'),
      createEmptyInstance: create)
    ..pPM<TenantMergedItem>(1, _omitFieldNames ? '' : 'items',
        subBuilder: TenantMergedItem.create)
    ..aOM<$2.PaginationResult>(2, _omitFieldNames ? '' : 'pagination',
        subBuilder: $2.PaginationResult.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListTenantItemsResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListTenantItemsResponse copyWith(
          void Function(ListTenantItemsResponse) updates) =>
      super.copyWith((message) => updates(message as ListTenantItemsResponse))
          as ListTenantItemsResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static ListTenantItemsResponse create() => ListTenantItemsResponse._();
  @$core.override
  ListTenantItemsResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static ListTenantItemsResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ListTenantItemsResponse>(create);
  static ListTenantItemsResponse? _defaultInstance;

  @$pb.TagNumber(1)
  $pb.PbList<TenantMergedItem> get items => $_getList(0);

  @$pb.TagNumber(2)
  $2.PaginationResult get pagination => $_getN(1);
  @$pb.TagNumber(2)
  set pagination($2.PaginationResult value) => $_setField(2, value);
  @$pb.TagNumber(2)
  $core.bool hasPagination() => $_has(1);
  @$pb.TagNumber(2)
  void clearPagination() => $_clearField(2);
  @$pb.TagNumber(2)
  $2.PaginationResult ensurePagination() => $_ensure(1);
}

const $core.bool _omitFieldNames =
    $core.bool.fromEnvironment('protobuf.omit_field_names');
const $core.bool _omitMessageNames =
    $core.bool.fromEnvironment('protobuf.omit_message_names');
