// This is a generated file - do not edit.
//
// Generated from k1s0/system/mastermaintenance/v1/master_maintenance.proto.

// @dart = 3.3

// ignore_for_file: annotate_overrides, camel_case_types, comment_references
// ignore_for_file: constant_identifier_names
// ignore_for_file: curly_braces_in_flow_control_structures
// ignore_for_file: deprecated_member_use_from_same_package, library_prefixes
// ignore_for_file: non_constant_identifier_names, prefer_relative_imports

import 'dart:core' as $core;

import 'package:protobuf/protobuf.dart' as $pb;
import 'package:protobuf/well_known_types/google/protobuf/struct.pb.dart' as $1;

import '../../common/v1/types.pb.dart' as $2;

export 'package:protobuf/protobuf.dart' show GeneratedMessageGenericExtensions;

class CreateTableDefinitionRequest extends $pb.GeneratedMessage {
  factory CreateTableDefinitionRequest({
    $1.Struct? data,
  }) {
    final result = create();
    if (data != null) result.data = data;
    return result;
  }

  CreateTableDefinitionRequest._();

  factory CreateTableDefinitionRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory CreateTableDefinitionRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'CreateTableDefinitionRequest',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.mastermaintenance.v1'),
      createEmptyInstance: create)
    ..aOM<$1.Struct>(1, _omitFieldNames ? '' : 'data',
        subBuilder: $1.Struct.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  CreateTableDefinitionRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  CreateTableDefinitionRequest copyWith(
          void Function(CreateTableDefinitionRequest) updates) =>
      super.copyWith(
              (message) => updates(message as CreateTableDefinitionRequest))
          as CreateTableDefinitionRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static CreateTableDefinitionRequest create() =>
      CreateTableDefinitionRequest._();
  @$core.override
  CreateTableDefinitionRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static CreateTableDefinitionRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<CreateTableDefinitionRequest>(create);
  static CreateTableDefinitionRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $1.Struct get data => $_getN(0);
  @$pb.TagNumber(1)
  set data($1.Struct value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasData() => $_has(0);
  @$pb.TagNumber(1)
  void clearData() => $_clearField(1);
  @$pb.TagNumber(1)
  $1.Struct ensureData() => $_ensure(0);
}

class CreateTableDefinitionResponse extends $pb.GeneratedMessage {
  factory CreateTableDefinitionResponse({
    GetTableDefinitionResponse? table,
  }) {
    final result = create();
    if (table != null) result.table = table;
    return result;
  }

  CreateTableDefinitionResponse._();

  factory CreateTableDefinitionResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory CreateTableDefinitionResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'CreateTableDefinitionResponse',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.mastermaintenance.v1'),
      createEmptyInstance: create)
    ..aOM<GetTableDefinitionResponse>(1, _omitFieldNames ? '' : 'table',
        subBuilder: GetTableDefinitionResponse.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  CreateTableDefinitionResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  CreateTableDefinitionResponse copyWith(
          void Function(CreateTableDefinitionResponse) updates) =>
      super.copyWith(
              (message) => updates(message as CreateTableDefinitionResponse))
          as CreateTableDefinitionResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static CreateTableDefinitionResponse create() =>
      CreateTableDefinitionResponse._();
  @$core.override
  CreateTableDefinitionResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static CreateTableDefinitionResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<CreateTableDefinitionResponse>(create);
  static CreateTableDefinitionResponse? _defaultInstance;

  @$pb.TagNumber(1)
  GetTableDefinitionResponse get table => $_getN(0);
  @$pb.TagNumber(1)
  set table(GetTableDefinitionResponse value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasTable() => $_has(0);
  @$pb.TagNumber(1)
  void clearTable() => $_clearField(1);
  @$pb.TagNumber(1)
  GetTableDefinitionResponse ensureTable() => $_ensure(0);
}

class UpdateTableDefinitionRequest extends $pb.GeneratedMessage {
  factory UpdateTableDefinitionRequest({
    $core.String? tableName,
    $1.Struct? data,
    $core.String? domainScope,
  }) {
    final result = create();
    if (tableName != null) result.tableName = tableName;
    if (data != null) result.data = data;
    if (domainScope != null) result.domainScope = domainScope;
    return result;
  }

  UpdateTableDefinitionRequest._();

  factory UpdateTableDefinitionRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory UpdateTableDefinitionRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'UpdateTableDefinitionRequest',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.mastermaintenance.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'tableName')
    ..aOM<$1.Struct>(2, _omitFieldNames ? '' : 'data',
        subBuilder: $1.Struct.create)
    ..aOS(3, _omitFieldNames ? '' : 'domainScope')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  UpdateTableDefinitionRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  UpdateTableDefinitionRequest copyWith(
          void Function(UpdateTableDefinitionRequest) updates) =>
      super.copyWith(
              (message) => updates(message as UpdateTableDefinitionRequest))
          as UpdateTableDefinitionRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static UpdateTableDefinitionRequest create() =>
      UpdateTableDefinitionRequest._();
  @$core.override
  UpdateTableDefinitionRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static UpdateTableDefinitionRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<UpdateTableDefinitionRequest>(create);
  static UpdateTableDefinitionRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get tableName => $_getSZ(0);
  @$pb.TagNumber(1)
  set tableName($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasTableName() => $_has(0);
  @$pb.TagNumber(1)
  void clearTableName() => $_clearField(1);

  @$pb.TagNumber(2)
  $1.Struct get data => $_getN(1);
  @$pb.TagNumber(2)
  set data($1.Struct value) => $_setField(2, value);
  @$pb.TagNumber(2)
  $core.bool hasData() => $_has(1);
  @$pb.TagNumber(2)
  void clearData() => $_clearField(2);
  @$pb.TagNumber(2)
  $1.Struct ensureData() => $_ensure(1);

  @$pb.TagNumber(3)
  $core.String get domainScope => $_getSZ(2);
  @$pb.TagNumber(3)
  set domainScope($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasDomainScope() => $_has(2);
  @$pb.TagNumber(3)
  void clearDomainScope() => $_clearField(3);
}

class UpdateTableDefinitionResponse extends $pb.GeneratedMessage {
  factory UpdateTableDefinitionResponse({
    GetTableDefinitionResponse? table,
  }) {
    final result = create();
    if (table != null) result.table = table;
    return result;
  }

  UpdateTableDefinitionResponse._();

  factory UpdateTableDefinitionResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory UpdateTableDefinitionResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'UpdateTableDefinitionResponse',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.mastermaintenance.v1'),
      createEmptyInstance: create)
    ..aOM<GetTableDefinitionResponse>(1, _omitFieldNames ? '' : 'table',
        subBuilder: GetTableDefinitionResponse.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  UpdateTableDefinitionResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  UpdateTableDefinitionResponse copyWith(
          void Function(UpdateTableDefinitionResponse) updates) =>
      super.copyWith(
              (message) => updates(message as UpdateTableDefinitionResponse))
          as UpdateTableDefinitionResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static UpdateTableDefinitionResponse create() =>
      UpdateTableDefinitionResponse._();
  @$core.override
  UpdateTableDefinitionResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static UpdateTableDefinitionResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<UpdateTableDefinitionResponse>(create);
  static UpdateTableDefinitionResponse? _defaultInstance;

  @$pb.TagNumber(1)
  GetTableDefinitionResponse get table => $_getN(0);
  @$pb.TagNumber(1)
  set table(GetTableDefinitionResponse value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasTable() => $_has(0);
  @$pb.TagNumber(1)
  void clearTable() => $_clearField(1);
  @$pb.TagNumber(1)
  GetTableDefinitionResponse ensureTable() => $_ensure(0);
}

class DeleteTableDefinitionRequest extends $pb.GeneratedMessage {
  factory DeleteTableDefinitionRequest({
    $core.String? tableName,
    $core.String? domainScope,
  }) {
    final result = create();
    if (tableName != null) result.tableName = tableName;
    if (domainScope != null) result.domainScope = domainScope;
    return result;
  }

  DeleteTableDefinitionRequest._();

  factory DeleteTableDefinitionRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory DeleteTableDefinitionRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'DeleteTableDefinitionRequest',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.mastermaintenance.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'tableName')
    ..aOS(2, _omitFieldNames ? '' : 'domainScope')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  DeleteTableDefinitionRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  DeleteTableDefinitionRequest copyWith(
          void Function(DeleteTableDefinitionRequest) updates) =>
      super.copyWith(
              (message) => updates(message as DeleteTableDefinitionRequest))
          as DeleteTableDefinitionRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static DeleteTableDefinitionRequest create() =>
      DeleteTableDefinitionRequest._();
  @$core.override
  DeleteTableDefinitionRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static DeleteTableDefinitionRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<DeleteTableDefinitionRequest>(create);
  static DeleteTableDefinitionRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get tableName => $_getSZ(0);
  @$pb.TagNumber(1)
  set tableName($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasTableName() => $_has(0);
  @$pb.TagNumber(1)
  void clearTableName() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get domainScope => $_getSZ(1);
  @$pb.TagNumber(2)
  set domainScope($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasDomainScope() => $_has(1);
  @$pb.TagNumber(2)
  void clearDomainScope() => $_clearField(2);
}

class DeleteTableDefinitionResponse extends $pb.GeneratedMessage {
  factory DeleteTableDefinitionResponse({
    $core.bool? success,
  }) {
    final result = create();
    if (success != null) result.success = success;
    return result;
  }

  DeleteTableDefinitionResponse._();

  factory DeleteTableDefinitionResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory DeleteTableDefinitionResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'DeleteTableDefinitionResponse',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.mastermaintenance.v1'),
      createEmptyInstance: create)
    ..aOB(1, _omitFieldNames ? '' : 'success')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  DeleteTableDefinitionResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  DeleteTableDefinitionResponse copyWith(
          void Function(DeleteTableDefinitionResponse) updates) =>
      super.copyWith(
              (message) => updates(message as DeleteTableDefinitionResponse))
          as DeleteTableDefinitionResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static DeleteTableDefinitionResponse create() =>
      DeleteTableDefinitionResponse._();
  @$core.override
  DeleteTableDefinitionResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static DeleteTableDefinitionResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<DeleteTableDefinitionResponse>(create);
  static DeleteTableDefinitionResponse? _defaultInstance;

  @$pb.TagNumber(1)
  $core.bool get success => $_getBF(0);
  @$pb.TagNumber(1)
  set success($core.bool value) => $_setBool(0, value);
  @$pb.TagNumber(1)
  $core.bool hasSuccess() => $_has(0);
  @$pb.TagNumber(1)
  void clearSuccess() => $_clearField(1);
}

/// テーブル定義取得リクエスト
class GetTableDefinitionRequest extends $pb.GeneratedMessage {
  factory GetTableDefinitionRequest({
    $core.String? tableName,
    $core.String? domainScope,
  }) {
    final result = create();
    if (tableName != null) result.tableName = tableName;
    if (domainScope != null) result.domainScope = domainScope;
    return result;
  }

  GetTableDefinitionRequest._();

  factory GetTableDefinitionRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory GetTableDefinitionRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'GetTableDefinitionRequest',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.mastermaintenance.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'tableName')
    ..aOS(2, _omitFieldNames ? '' : 'domainScope')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetTableDefinitionRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetTableDefinitionRequest copyWith(
          void Function(GetTableDefinitionRequest) updates) =>
      super.copyWith((message) => updates(message as GetTableDefinitionRequest))
          as GetTableDefinitionRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static GetTableDefinitionRequest create() => GetTableDefinitionRequest._();
  @$core.override
  GetTableDefinitionRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static GetTableDefinitionRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<GetTableDefinitionRequest>(create);
  static GetTableDefinitionRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get tableName => $_getSZ(0);
  @$pb.TagNumber(1)
  set tableName($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasTableName() => $_has(0);
  @$pb.TagNumber(1)
  void clearTableName() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get domainScope => $_getSZ(1);
  @$pb.TagNumber(2)
  set domainScope($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasDomainScope() => $_has(1);
  @$pb.TagNumber(2)
  void clearDomainScope() => $_clearField(2);
}

/// テーブル定義レスポンス
class GetTableDefinitionResponse extends $pb.GeneratedMessage {
  factory GetTableDefinitionResponse({
    $core.String? id,
    $core.String? name,
    $core.String? schemaName,
    $core.String? displayName,
    $core.String? description,
    $core.bool? allowCreate,
    $core.bool? allowUpdate,
    $core.bool? allowDelete,
    $core.Iterable<ColumnDefinition>? columns,
    $core.Iterable<TableRelationship>? relationships,
    $core.String? databaseName,
    $core.String? category,
    $core.bool? isActive,
    $core.int? sortOrder,
    $core.String? createdBy,
    $2.Timestamp? createdAt,
    $2.Timestamp? updatedAt,
    $core.String? domainScope,
    $core.Iterable<$core.String>? readRoles,
    $core.Iterable<$core.String>? writeRoles,
    $core.Iterable<$core.String>? adminRoles,
  }) {
    final result = create();
    if (id != null) result.id = id;
    if (name != null) result.name = name;
    if (schemaName != null) result.schemaName = schemaName;
    if (displayName != null) result.displayName = displayName;
    if (description != null) result.description = description;
    if (allowCreate != null) result.allowCreate = allowCreate;
    if (allowUpdate != null) result.allowUpdate = allowUpdate;
    if (allowDelete != null) result.allowDelete = allowDelete;
    if (columns != null) result.columns.addAll(columns);
    if (relationships != null) result.relationships.addAll(relationships);
    if (databaseName != null) result.databaseName = databaseName;
    if (category != null) result.category = category;
    if (isActive != null) result.isActive = isActive;
    if (sortOrder != null) result.sortOrder = sortOrder;
    if (createdBy != null) result.createdBy = createdBy;
    if (createdAt != null) result.createdAt = createdAt;
    if (updatedAt != null) result.updatedAt = updatedAt;
    if (domainScope != null) result.domainScope = domainScope;
    if (readRoles != null) result.readRoles.addAll(readRoles);
    if (writeRoles != null) result.writeRoles.addAll(writeRoles);
    if (adminRoles != null) result.adminRoles.addAll(adminRoles);
    return result;
  }

  GetTableDefinitionResponse._();

  factory GetTableDefinitionResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory GetTableDefinitionResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'GetTableDefinitionResponse',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.mastermaintenance.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'id')
    ..aOS(2, _omitFieldNames ? '' : 'name')
    ..aOS(3, _omitFieldNames ? '' : 'schemaName')
    ..aOS(4, _omitFieldNames ? '' : 'displayName')
    ..aOS(5, _omitFieldNames ? '' : 'description')
    ..aOB(6, _omitFieldNames ? '' : 'allowCreate')
    ..aOB(7, _omitFieldNames ? '' : 'allowUpdate')
    ..aOB(8, _omitFieldNames ? '' : 'allowDelete')
    ..pPM<ColumnDefinition>(9, _omitFieldNames ? '' : 'columns',
        subBuilder: ColumnDefinition.create)
    ..pPM<TableRelationship>(10, _omitFieldNames ? '' : 'relationships',
        subBuilder: TableRelationship.create)
    ..aOS(11, _omitFieldNames ? '' : 'databaseName')
    ..aOS(12, _omitFieldNames ? '' : 'category')
    ..aOB(13, _omitFieldNames ? '' : 'isActive')
    ..aI(14, _omitFieldNames ? '' : 'sortOrder')
    ..aOS(15, _omitFieldNames ? '' : 'createdBy')
    ..aOM<$2.Timestamp>(16, _omitFieldNames ? '' : 'createdAt',
        subBuilder: $2.Timestamp.create)
    ..aOM<$2.Timestamp>(17, _omitFieldNames ? '' : 'updatedAt',
        subBuilder: $2.Timestamp.create)
    ..aOS(18, _omitFieldNames ? '' : 'domainScope')
    ..pPS(19, _omitFieldNames ? '' : 'readRoles')
    ..pPS(20, _omitFieldNames ? '' : 'writeRoles')
    ..pPS(21, _omitFieldNames ? '' : 'adminRoles')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetTableDefinitionResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetTableDefinitionResponse copyWith(
          void Function(GetTableDefinitionResponse) updates) =>
      super.copyWith(
              (message) => updates(message as GetTableDefinitionResponse))
          as GetTableDefinitionResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static GetTableDefinitionResponse create() => GetTableDefinitionResponse._();
  @$core.override
  GetTableDefinitionResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static GetTableDefinitionResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<GetTableDefinitionResponse>(create);
  static GetTableDefinitionResponse? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get id => $_getSZ(0);
  @$pb.TagNumber(1)
  set id($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasId() => $_has(0);
  @$pb.TagNumber(1)
  void clearId() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get name => $_getSZ(1);
  @$pb.TagNumber(2)
  set name($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasName() => $_has(1);
  @$pb.TagNumber(2)
  void clearName() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.String get schemaName => $_getSZ(2);
  @$pb.TagNumber(3)
  set schemaName($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasSchemaName() => $_has(2);
  @$pb.TagNumber(3)
  void clearSchemaName() => $_clearField(3);

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
  $core.bool get allowCreate => $_getBF(5);
  @$pb.TagNumber(6)
  set allowCreate($core.bool value) => $_setBool(5, value);
  @$pb.TagNumber(6)
  $core.bool hasAllowCreate() => $_has(5);
  @$pb.TagNumber(6)
  void clearAllowCreate() => $_clearField(6);

  @$pb.TagNumber(7)
  $core.bool get allowUpdate => $_getBF(6);
  @$pb.TagNumber(7)
  set allowUpdate($core.bool value) => $_setBool(6, value);
  @$pb.TagNumber(7)
  $core.bool hasAllowUpdate() => $_has(6);
  @$pb.TagNumber(7)
  void clearAllowUpdate() => $_clearField(7);

  @$pb.TagNumber(8)
  $core.bool get allowDelete => $_getBF(7);
  @$pb.TagNumber(8)
  set allowDelete($core.bool value) => $_setBool(7, value);
  @$pb.TagNumber(8)
  $core.bool hasAllowDelete() => $_has(7);
  @$pb.TagNumber(8)
  void clearAllowDelete() => $_clearField(8);

  @$pb.TagNumber(9)
  $pb.PbList<ColumnDefinition> get columns => $_getList(8);

  @$pb.TagNumber(10)
  $pb.PbList<TableRelationship> get relationships => $_getList(9);

  @$pb.TagNumber(11)
  $core.String get databaseName => $_getSZ(10);
  @$pb.TagNumber(11)
  set databaseName($core.String value) => $_setString(10, value);
  @$pb.TagNumber(11)
  $core.bool hasDatabaseName() => $_has(10);
  @$pb.TagNumber(11)
  void clearDatabaseName() => $_clearField(11);

  @$pb.TagNumber(12)
  $core.String get category => $_getSZ(11);
  @$pb.TagNumber(12)
  set category($core.String value) => $_setString(11, value);
  @$pb.TagNumber(12)
  $core.bool hasCategory() => $_has(11);
  @$pb.TagNumber(12)
  void clearCategory() => $_clearField(12);

  @$pb.TagNumber(13)
  $core.bool get isActive => $_getBF(12);
  @$pb.TagNumber(13)
  set isActive($core.bool value) => $_setBool(12, value);
  @$pb.TagNumber(13)
  $core.bool hasIsActive() => $_has(12);
  @$pb.TagNumber(13)
  void clearIsActive() => $_clearField(13);

  @$pb.TagNumber(14)
  $core.int get sortOrder => $_getIZ(13);
  @$pb.TagNumber(14)
  set sortOrder($core.int value) => $_setSignedInt32(13, value);
  @$pb.TagNumber(14)
  $core.bool hasSortOrder() => $_has(13);
  @$pb.TagNumber(14)
  void clearSortOrder() => $_clearField(14);

  @$pb.TagNumber(15)
  $core.String get createdBy => $_getSZ(14);
  @$pb.TagNumber(15)
  set createdBy($core.String value) => $_setString(14, value);
  @$pb.TagNumber(15)
  $core.bool hasCreatedBy() => $_has(14);
  @$pb.TagNumber(15)
  void clearCreatedBy() => $_clearField(15);

  @$pb.TagNumber(16)
  $2.Timestamp get createdAt => $_getN(15);
  @$pb.TagNumber(16)
  set createdAt($2.Timestamp value) => $_setField(16, value);
  @$pb.TagNumber(16)
  $core.bool hasCreatedAt() => $_has(15);
  @$pb.TagNumber(16)
  void clearCreatedAt() => $_clearField(16);
  @$pb.TagNumber(16)
  $2.Timestamp ensureCreatedAt() => $_ensure(15);

  @$pb.TagNumber(17)
  $2.Timestamp get updatedAt => $_getN(16);
  @$pb.TagNumber(17)
  set updatedAt($2.Timestamp value) => $_setField(17, value);
  @$pb.TagNumber(17)
  $core.bool hasUpdatedAt() => $_has(16);
  @$pb.TagNumber(17)
  void clearUpdatedAt() => $_clearField(17);
  @$pb.TagNumber(17)
  $2.Timestamp ensureUpdatedAt() => $_ensure(16);

  @$pb.TagNumber(18)
  $core.String get domainScope => $_getSZ(17);
  @$pb.TagNumber(18)
  set domainScope($core.String value) => $_setString(17, value);
  @$pb.TagNumber(18)
  $core.bool hasDomainScope() => $_has(17);
  @$pb.TagNumber(18)
  void clearDomainScope() => $_clearField(18);

  @$pb.TagNumber(19)
  $pb.PbList<$core.String> get readRoles => $_getList(18);

  @$pb.TagNumber(20)
  $pb.PbList<$core.String> get writeRoles => $_getList(19);

  @$pb.TagNumber(21)
  $pb.PbList<$core.String> get adminRoles => $_getList(20);
}

/// カラム定義
class ColumnDefinition extends $pb.GeneratedMessage {
  factory ColumnDefinition({
    $core.String? columnName,
    $core.String? displayName,
    $core.String? dataType,
    $core.bool? isPrimaryKey,
    $core.bool? isNullable,
    $core.bool? isSearchable,
    $core.bool? isSortable,
    $core.bool? isFilterable,
    $core.bool? isVisibleInList,
    $core.bool? isVisibleInForm,
    $core.bool? isReadonly,
    $core.String? inputType,
    $core.int? displayOrder,
    $core.bool? isUnique,
    $core.String? defaultValue,
    $core.int? maxLength,
    $core.double? minValue,
    $core.double? maxValue,
    $core.String? regexPattern,
    $core.String? selectOptionsJson,
  }) {
    final result = create();
    if (columnName != null) result.columnName = columnName;
    if (displayName != null) result.displayName = displayName;
    if (dataType != null) result.dataType = dataType;
    if (isPrimaryKey != null) result.isPrimaryKey = isPrimaryKey;
    if (isNullable != null) result.isNullable = isNullable;
    if (isSearchable != null) result.isSearchable = isSearchable;
    if (isSortable != null) result.isSortable = isSortable;
    if (isFilterable != null) result.isFilterable = isFilterable;
    if (isVisibleInList != null) result.isVisibleInList = isVisibleInList;
    if (isVisibleInForm != null) result.isVisibleInForm = isVisibleInForm;
    if (isReadonly != null) result.isReadonly = isReadonly;
    if (inputType != null) result.inputType = inputType;
    if (displayOrder != null) result.displayOrder = displayOrder;
    if (isUnique != null) result.isUnique = isUnique;
    if (defaultValue != null) result.defaultValue = defaultValue;
    if (maxLength != null) result.maxLength = maxLength;
    if (minValue != null) result.minValue = minValue;
    if (maxValue != null) result.maxValue = maxValue;
    if (regexPattern != null) result.regexPattern = regexPattern;
    if (selectOptionsJson != null) result.selectOptionsJson = selectOptionsJson;
    return result;
  }

  ColumnDefinition._();

  factory ColumnDefinition.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory ColumnDefinition.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ColumnDefinition',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.mastermaintenance.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'columnName')
    ..aOS(2, _omitFieldNames ? '' : 'displayName')
    ..aOS(3, _omitFieldNames ? '' : 'dataType')
    ..aOB(4, _omitFieldNames ? '' : 'isPrimaryKey')
    ..aOB(5, _omitFieldNames ? '' : 'isNullable')
    ..aOB(6, _omitFieldNames ? '' : 'isSearchable')
    ..aOB(7, _omitFieldNames ? '' : 'isSortable')
    ..aOB(8, _omitFieldNames ? '' : 'isFilterable')
    ..aOB(9, _omitFieldNames ? '' : 'isVisibleInList')
    ..aOB(10, _omitFieldNames ? '' : 'isVisibleInForm')
    ..aOB(11, _omitFieldNames ? '' : 'isReadonly')
    ..aOS(12, _omitFieldNames ? '' : 'inputType')
    ..aI(13, _omitFieldNames ? '' : 'displayOrder')
    ..aOB(14, _omitFieldNames ? '' : 'isUnique')
    ..aOS(15, _omitFieldNames ? '' : 'defaultValue')
    ..aI(16, _omitFieldNames ? '' : 'maxLength')
    ..aD(17, _omitFieldNames ? '' : 'minValue')
    ..aD(18, _omitFieldNames ? '' : 'maxValue')
    ..aOS(19, _omitFieldNames ? '' : 'regexPattern')
    ..aOS(20, _omitFieldNames ? '' : 'selectOptionsJson')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ColumnDefinition clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ColumnDefinition copyWith(void Function(ColumnDefinition) updates) =>
      super.copyWith((message) => updates(message as ColumnDefinition))
          as ColumnDefinition;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static ColumnDefinition create() => ColumnDefinition._();
  @$core.override
  ColumnDefinition createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static ColumnDefinition getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ColumnDefinition>(create);
  static ColumnDefinition? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get columnName => $_getSZ(0);
  @$pb.TagNumber(1)
  set columnName($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasColumnName() => $_has(0);
  @$pb.TagNumber(1)
  void clearColumnName() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get displayName => $_getSZ(1);
  @$pb.TagNumber(2)
  set displayName($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasDisplayName() => $_has(1);
  @$pb.TagNumber(2)
  void clearDisplayName() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.String get dataType => $_getSZ(2);
  @$pb.TagNumber(3)
  set dataType($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasDataType() => $_has(2);
  @$pb.TagNumber(3)
  void clearDataType() => $_clearField(3);

  @$pb.TagNumber(4)
  $core.bool get isPrimaryKey => $_getBF(3);
  @$pb.TagNumber(4)
  set isPrimaryKey($core.bool value) => $_setBool(3, value);
  @$pb.TagNumber(4)
  $core.bool hasIsPrimaryKey() => $_has(3);
  @$pb.TagNumber(4)
  void clearIsPrimaryKey() => $_clearField(4);

  @$pb.TagNumber(5)
  $core.bool get isNullable => $_getBF(4);
  @$pb.TagNumber(5)
  set isNullable($core.bool value) => $_setBool(4, value);
  @$pb.TagNumber(5)
  $core.bool hasIsNullable() => $_has(4);
  @$pb.TagNumber(5)
  void clearIsNullable() => $_clearField(5);

  @$pb.TagNumber(6)
  $core.bool get isSearchable => $_getBF(5);
  @$pb.TagNumber(6)
  set isSearchable($core.bool value) => $_setBool(5, value);
  @$pb.TagNumber(6)
  $core.bool hasIsSearchable() => $_has(5);
  @$pb.TagNumber(6)
  void clearIsSearchable() => $_clearField(6);

  @$pb.TagNumber(7)
  $core.bool get isSortable => $_getBF(6);
  @$pb.TagNumber(7)
  set isSortable($core.bool value) => $_setBool(6, value);
  @$pb.TagNumber(7)
  $core.bool hasIsSortable() => $_has(6);
  @$pb.TagNumber(7)
  void clearIsSortable() => $_clearField(7);

  @$pb.TagNumber(8)
  $core.bool get isFilterable => $_getBF(7);
  @$pb.TagNumber(8)
  set isFilterable($core.bool value) => $_setBool(7, value);
  @$pb.TagNumber(8)
  $core.bool hasIsFilterable() => $_has(7);
  @$pb.TagNumber(8)
  void clearIsFilterable() => $_clearField(8);

  @$pb.TagNumber(9)
  $core.bool get isVisibleInList => $_getBF(8);
  @$pb.TagNumber(9)
  set isVisibleInList($core.bool value) => $_setBool(8, value);
  @$pb.TagNumber(9)
  $core.bool hasIsVisibleInList() => $_has(8);
  @$pb.TagNumber(9)
  void clearIsVisibleInList() => $_clearField(9);

  @$pb.TagNumber(10)
  $core.bool get isVisibleInForm => $_getBF(9);
  @$pb.TagNumber(10)
  set isVisibleInForm($core.bool value) => $_setBool(9, value);
  @$pb.TagNumber(10)
  $core.bool hasIsVisibleInForm() => $_has(9);
  @$pb.TagNumber(10)
  void clearIsVisibleInForm() => $_clearField(10);

  @$pb.TagNumber(11)
  $core.bool get isReadonly => $_getBF(10);
  @$pb.TagNumber(11)
  set isReadonly($core.bool value) => $_setBool(10, value);
  @$pb.TagNumber(11)
  $core.bool hasIsReadonly() => $_has(10);
  @$pb.TagNumber(11)
  void clearIsReadonly() => $_clearField(11);

  @$pb.TagNumber(12)
  $core.String get inputType => $_getSZ(11);
  @$pb.TagNumber(12)
  set inputType($core.String value) => $_setString(11, value);
  @$pb.TagNumber(12)
  $core.bool hasInputType() => $_has(11);
  @$pb.TagNumber(12)
  void clearInputType() => $_clearField(12);

  @$pb.TagNumber(13)
  $core.int get displayOrder => $_getIZ(12);
  @$pb.TagNumber(13)
  set displayOrder($core.int value) => $_setSignedInt32(12, value);
  @$pb.TagNumber(13)
  $core.bool hasDisplayOrder() => $_has(12);
  @$pb.TagNumber(13)
  void clearDisplayOrder() => $_clearField(13);

  @$pb.TagNumber(14)
  $core.bool get isUnique => $_getBF(13);
  @$pb.TagNumber(14)
  set isUnique($core.bool value) => $_setBool(13, value);
  @$pb.TagNumber(14)
  $core.bool hasIsUnique() => $_has(13);
  @$pb.TagNumber(14)
  void clearIsUnique() => $_clearField(14);

  @$pb.TagNumber(15)
  $core.String get defaultValue => $_getSZ(14);
  @$pb.TagNumber(15)
  set defaultValue($core.String value) => $_setString(14, value);
  @$pb.TagNumber(15)
  $core.bool hasDefaultValue() => $_has(14);
  @$pb.TagNumber(15)
  void clearDefaultValue() => $_clearField(15);

  @$pb.TagNumber(16)
  $core.int get maxLength => $_getIZ(15);
  @$pb.TagNumber(16)
  set maxLength($core.int value) => $_setSignedInt32(15, value);
  @$pb.TagNumber(16)
  $core.bool hasMaxLength() => $_has(15);
  @$pb.TagNumber(16)
  void clearMaxLength() => $_clearField(16);

  @$pb.TagNumber(17)
  $core.double get minValue => $_getN(16);
  @$pb.TagNumber(17)
  set minValue($core.double value) => $_setDouble(16, value);
  @$pb.TagNumber(17)
  $core.bool hasMinValue() => $_has(16);
  @$pb.TagNumber(17)
  void clearMinValue() => $_clearField(17);

  @$pb.TagNumber(18)
  $core.double get maxValue => $_getN(17);
  @$pb.TagNumber(18)
  set maxValue($core.double value) => $_setDouble(17, value);
  @$pb.TagNumber(18)
  $core.bool hasMaxValue() => $_has(17);
  @$pb.TagNumber(18)
  void clearMaxValue() => $_clearField(18);

  @$pb.TagNumber(19)
  $core.String get regexPattern => $_getSZ(18);
  @$pb.TagNumber(19)
  set regexPattern($core.String value) => $_setString(18, value);
  @$pb.TagNumber(19)
  $core.bool hasRegexPattern() => $_has(18);
  @$pb.TagNumber(19)
  void clearRegexPattern() => $_clearField(19);

  @$pb.TagNumber(20)
  $core.String get selectOptionsJson => $_getSZ(19);
  @$pb.TagNumber(20)
  set selectOptionsJson($core.String value) => $_setString(19, value);
  @$pb.TagNumber(20)
  $core.bool hasSelectOptionsJson() => $_has(19);
  @$pb.TagNumber(20)
  void clearSelectOptionsJson() => $_clearField(20);
}

/// テーブル間関係
class TableRelationship extends $pb.GeneratedMessage {
  factory TableRelationship({
    $core.String? sourceColumn,
    $core.String? targetTable,
    $core.String? targetColumn,
    $core.String? relationshipType,
    $core.String? displayName,
    $core.String? id,
    $core.bool? isCascadeDelete,
    $core.String? createdAt,
  }) {
    final result = create();
    if (sourceColumn != null) result.sourceColumn = sourceColumn;
    if (targetTable != null) result.targetTable = targetTable;
    if (targetColumn != null) result.targetColumn = targetColumn;
    if (relationshipType != null) result.relationshipType = relationshipType;
    if (displayName != null) result.displayName = displayName;
    if (id != null) result.id = id;
    if (isCascadeDelete != null) result.isCascadeDelete = isCascadeDelete;
    if (createdAt != null) result.createdAt = createdAt;
    return result;
  }

  TableRelationship._();

  factory TableRelationship.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory TableRelationship.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'TableRelationship',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.mastermaintenance.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'sourceColumn')
    ..aOS(2, _omitFieldNames ? '' : 'targetTable')
    ..aOS(3, _omitFieldNames ? '' : 'targetColumn')
    ..aOS(4, _omitFieldNames ? '' : 'relationshipType')
    ..aOS(5, _omitFieldNames ? '' : 'displayName')
    ..aOS(6, _omitFieldNames ? '' : 'id')
    ..aOB(7, _omitFieldNames ? '' : 'isCascadeDelete')
    ..aOS(8, _omitFieldNames ? '' : 'createdAt')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  TableRelationship clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  TableRelationship copyWith(void Function(TableRelationship) updates) =>
      super.copyWith((message) => updates(message as TableRelationship))
          as TableRelationship;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static TableRelationship create() => TableRelationship._();
  @$core.override
  TableRelationship createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static TableRelationship getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<TableRelationship>(create);
  static TableRelationship? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get sourceColumn => $_getSZ(0);
  @$pb.TagNumber(1)
  set sourceColumn($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasSourceColumn() => $_has(0);
  @$pb.TagNumber(1)
  void clearSourceColumn() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get targetTable => $_getSZ(1);
  @$pb.TagNumber(2)
  set targetTable($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasTargetTable() => $_has(1);
  @$pb.TagNumber(2)
  void clearTargetTable() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.String get targetColumn => $_getSZ(2);
  @$pb.TagNumber(3)
  set targetColumn($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasTargetColumn() => $_has(2);
  @$pb.TagNumber(3)
  void clearTargetColumn() => $_clearField(3);

  @$pb.TagNumber(4)
  $core.String get relationshipType => $_getSZ(3);
  @$pb.TagNumber(4)
  set relationshipType($core.String value) => $_setString(3, value);
  @$pb.TagNumber(4)
  $core.bool hasRelationshipType() => $_has(3);
  @$pb.TagNumber(4)
  void clearRelationshipType() => $_clearField(4);

  @$pb.TagNumber(5)
  $core.String get displayName => $_getSZ(4);
  @$pb.TagNumber(5)
  set displayName($core.String value) => $_setString(4, value);
  @$pb.TagNumber(5)
  $core.bool hasDisplayName() => $_has(4);
  @$pb.TagNumber(5)
  void clearDisplayName() => $_clearField(5);

  @$pb.TagNumber(6)
  $core.String get id => $_getSZ(5);
  @$pb.TagNumber(6)
  set id($core.String value) => $_setString(5, value);
  @$pb.TagNumber(6)
  $core.bool hasId() => $_has(5);
  @$pb.TagNumber(6)
  void clearId() => $_clearField(6);

  @$pb.TagNumber(7)
  $core.bool get isCascadeDelete => $_getBF(6);
  @$pb.TagNumber(7)
  set isCascadeDelete($core.bool value) => $_setBool(6, value);
  @$pb.TagNumber(7)
  $core.bool hasIsCascadeDelete() => $_has(6);
  @$pb.TagNumber(7)
  void clearIsCascadeDelete() => $_clearField(7);

  @$pb.TagNumber(8)
  $core.String get createdAt => $_getSZ(7);
  @$pb.TagNumber(8)
  set createdAt($core.String value) => $_setString(7, value);
  @$pb.TagNumber(8)
  $core.bool hasCreatedAt() => $_has(7);
  @$pb.TagNumber(8)
  void clearCreatedAt() => $_clearField(8);
}

/// テーブル定義一覧リクエスト
class ListTableDefinitionsRequest extends $pb.GeneratedMessage {
  factory ListTableDefinitionsRequest({
    $core.String? category,
    $core.bool? activeOnly,
    $2.Pagination? pagination,
    $core.String? domainScope,
  }) {
    final result = create();
    if (category != null) result.category = category;
    if (activeOnly != null) result.activeOnly = activeOnly;
    if (pagination != null) result.pagination = pagination;
    if (domainScope != null) result.domainScope = domainScope;
    return result;
  }

  ListTableDefinitionsRequest._();

  factory ListTableDefinitionsRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory ListTableDefinitionsRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ListTableDefinitionsRequest',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.mastermaintenance.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'category')
    ..aOB(2, _omitFieldNames ? '' : 'activeOnly')
    ..aOM<$2.Pagination>(3, _omitFieldNames ? '' : 'pagination',
        subBuilder: $2.Pagination.create)
    ..aOS(4, _omitFieldNames ? '' : 'domainScope')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListTableDefinitionsRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListTableDefinitionsRequest copyWith(
          void Function(ListTableDefinitionsRequest) updates) =>
      super.copyWith(
              (message) => updates(message as ListTableDefinitionsRequest))
          as ListTableDefinitionsRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static ListTableDefinitionsRequest create() =>
      ListTableDefinitionsRequest._();
  @$core.override
  ListTableDefinitionsRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static ListTableDefinitionsRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ListTableDefinitionsRequest>(create);
  static ListTableDefinitionsRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get category => $_getSZ(0);
  @$pb.TagNumber(1)
  set category($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasCategory() => $_has(0);
  @$pb.TagNumber(1)
  void clearCategory() => $_clearField(1);

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

  @$pb.TagNumber(4)
  $core.String get domainScope => $_getSZ(3);
  @$pb.TagNumber(4)
  set domainScope($core.String value) => $_setString(3, value);
  @$pb.TagNumber(4)
  $core.bool hasDomainScope() => $_has(3);
  @$pb.TagNumber(4)
  void clearDomainScope() => $_clearField(4);
}

/// テーブル定義一覧レスポンス
class ListTableDefinitionsResponse extends $pb.GeneratedMessage {
  factory ListTableDefinitionsResponse({
    $core.Iterable<GetTableDefinitionResponse>? tables,
    $2.PaginationResult? pagination,
  }) {
    final result = create();
    if (tables != null) result.tables.addAll(tables);
    if (pagination != null) result.pagination = pagination;
    return result;
  }

  ListTableDefinitionsResponse._();

  factory ListTableDefinitionsResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory ListTableDefinitionsResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ListTableDefinitionsResponse',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.mastermaintenance.v1'),
      createEmptyInstance: create)
    ..pPM<GetTableDefinitionResponse>(1, _omitFieldNames ? '' : 'tables',
        subBuilder: GetTableDefinitionResponse.create)
    ..aOM<$2.PaginationResult>(2, _omitFieldNames ? '' : 'pagination',
        subBuilder: $2.PaginationResult.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListTableDefinitionsResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListTableDefinitionsResponse copyWith(
          void Function(ListTableDefinitionsResponse) updates) =>
      super.copyWith(
              (message) => updates(message as ListTableDefinitionsResponse))
          as ListTableDefinitionsResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static ListTableDefinitionsResponse create() =>
      ListTableDefinitionsResponse._();
  @$core.override
  ListTableDefinitionsResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static ListTableDefinitionsResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ListTableDefinitionsResponse>(create);
  static ListTableDefinitionsResponse? _defaultInstance;

  @$pb.TagNumber(1)
  $pb.PbList<GetTableDefinitionResponse> get tables => $_getList(0);

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

/// レコード取得リクエスト
class GetRecordRequest extends $pb.GeneratedMessage {
  factory GetRecordRequest({
    $core.String? tableName,
    $core.String? recordId,
    $core.String? domainScope,
  }) {
    final result = create();
    if (tableName != null) result.tableName = tableName;
    if (recordId != null) result.recordId = recordId;
    if (domainScope != null) result.domainScope = domainScope;
    return result;
  }

  GetRecordRequest._();

  factory GetRecordRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory GetRecordRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'GetRecordRequest',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.mastermaintenance.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'tableName')
    ..aOS(2, _omitFieldNames ? '' : 'recordId')
    ..aOS(3, _omitFieldNames ? '' : 'domainScope')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetRecordRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetRecordRequest copyWith(void Function(GetRecordRequest) updates) =>
      super.copyWith((message) => updates(message as GetRecordRequest))
          as GetRecordRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static GetRecordRequest create() => GetRecordRequest._();
  @$core.override
  GetRecordRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static GetRecordRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<GetRecordRequest>(create);
  static GetRecordRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get tableName => $_getSZ(0);
  @$pb.TagNumber(1)
  set tableName($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasTableName() => $_has(0);
  @$pb.TagNumber(1)
  void clearTableName() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get recordId => $_getSZ(1);
  @$pb.TagNumber(2)
  set recordId($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasRecordId() => $_has(1);
  @$pb.TagNumber(2)
  void clearRecordId() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.String get domainScope => $_getSZ(2);
  @$pb.TagNumber(3)
  set domainScope($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasDomainScope() => $_has(2);
  @$pb.TagNumber(3)
  void clearDomainScope() => $_clearField(3);
}

/// レコードレスポンス
class GetRecordResponse extends $pb.GeneratedMessage {
  factory GetRecordResponse({
    $1.Struct? data,
    $core.Iterable<ValidationWarning>? warnings,
  }) {
    final result = create();
    if (data != null) result.data = data;
    if (warnings != null) result.warnings.addAll(warnings);
    return result;
  }

  GetRecordResponse._();

  factory GetRecordResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory GetRecordResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'GetRecordResponse',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.mastermaintenance.v1'),
      createEmptyInstance: create)
    ..aOM<$1.Struct>(1, _omitFieldNames ? '' : 'data',
        subBuilder: $1.Struct.create)
    ..pPM<ValidationWarning>(2, _omitFieldNames ? '' : 'warnings',
        subBuilder: ValidationWarning.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetRecordResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetRecordResponse copyWith(void Function(GetRecordResponse) updates) =>
      super.copyWith((message) => updates(message as GetRecordResponse))
          as GetRecordResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static GetRecordResponse create() => GetRecordResponse._();
  @$core.override
  GetRecordResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static GetRecordResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<GetRecordResponse>(create);
  static GetRecordResponse? _defaultInstance;

  @$pb.TagNumber(1)
  $1.Struct get data => $_getN(0);
  @$pb.TagNumber(1)
  set data($1.Struct value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasData() => $_has(0);
  @$pb.TagNumber(1)
  void clearData() => $_clearField(1);
  @$pb.TagNumber(1)
  $1.Struct ensureData() => $_ensure(0);

  @$pb.TagNumber(2)
  $pb.PbList<ValidationWarning> get warnings => $_getList(1);
}

class CreateRecordResponse extends $pb.GeneratedMessage {
  factory CreateRecordResponse({
    $1.Struct? data,
    $core.Iterable<ValidationWarning>? warnings,
  }) {
    final result = create();
    if (data != null) result.data = data;
    if (warnings != null) result.warnings.addAll(warnings);
    return result;
  }

  CreateRecordResponse._();

  factory CreateRecordResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory CreateRecordResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'CreateRecordResponse',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.mastermaintenance.v1'),
      createEmptyInstance: create)
    ..aOM<$1.Struct>(1, _omitFieldNames ? '' : 'data',
        subBuilder: $1.Struct.create)
    ..pPM<ValidationWarning>(2, _omitFieldNames ? '' : 'warnings',
        subBuilder: ValidationWarning.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  CreateRecordResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  CreateRecordResponse copyWith(void Function(CreateRecordResponse) updates) =>
      super.copyWith((message) => updates(message as CreateRecordResponse))
          as CreateRecordResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static CreateRecordResponse create() => CreateRecordResponse._();
  @$core.override
  CreateRecordResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static CreateRecordResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<CreateRecordResponse>(create);
  static CreateRecordResponse? _defaultInstance;

  @$pb.TagNumber(1)
  $1.Struct get data => $_getN(0);
  @$pb.TagNumber(1)
  set data($1.Struct value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasData() => $_has(0);
  @$pb.TagNumber(1)
  void clearData() => $_clearField(1);
  @$pb.TagNumber(1)
  $1.Struct ensureData() => $_ensure(0);

  @$pb.TagNumber(2)
  $pb.PbList<ValidationWarning> get warnings => $_getList(1);
}

class UpdateRecordResponse extends $pb.GeneratedMessage {
  factory UpdateRecordResponse({
    $1.Struct? data,
    $core.Iterable<ValidationWarning>? warnings,
  }) {
    final result = create();
    if (data != null) result.data = data;
    if (warnings != null) result.warnings.addAll(warnings);
    return result;
  }

  UpdateRecordResponse._();

  factory UpdateRecordResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory UpdateRecordResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'UpdateRecordResponse',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.mastermaintenance.v1'),
      createEmptyInstance: create)
    ..aOM<$1.Struct>(1, _omitFieldNames ? '' : 'data',
        subBuilder: $1.Struct.create)
    ..pPM<ValidationWarning>(2, _omitFieldNames ? '' : 'warnings',
        subBuilder: ValidationWarning.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  UpdateRecordResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  UpdateRecordResponse copyWith(void Function(UpdateRecordResponse) updates) =>
      super.copyWith((message) => updates(message as UpdateRecordResponse))
          as UpdateRecordResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static UpdateRecordResponse create() => UpdateRecordResponse._();
  @$core.override
  UpdateRecordResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static UpdateRecordResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<UpdateRecordResponse>(create);
  static UpdateRecordResponse? _defaultInstance;

  @$pb.TagNumber(1)
  $1.Struct get data => $_getN(0);
  @$pb.TagNumber(1)
  set data($1.Struct value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasData() => $_has(0);
  @$pb.TagNumber(1)
  void clearData() => $_clearField(1);
  @$pb.TagNumber(1)
  $1.Struct ensureData() => $_ensure(0);

  @$pb.TagNumber(2)
  $pb.PbList<ValidationWarning> get warnings => $_getList(1);
}

/// レコード一覧リクエスト
class ListRecordsRequest extends $pb.GeneratedMessage {
  factory ListRecordsRequest({
    $core.String? tableName,
    $2.Pagination? pagination,
    $core.String? sort,
    $core.String? filter,
    $core.String? search,
    $core.String? domainScope,
  }) {
    final result = create();
    if (tableName != null) result.tableName = tableName;
    if (pagination != null) result.pagination = pagination;
    if (sort != null) result.sort = sort;
    if (filter != null) result.filter = filter;
    if (search != null) result.search = search;
    if (domainScope != null) result.domainScope = domainScope;
    return result;
  }

  ListRecordsRequest._();

  factory ListRecordsRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory ListRecordsRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ListRecordsRequest',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.mastermaintenance.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'tableName')
    ..aOM<$2.Pagination>(2, _omitFieldNames ? '' : 'pagination',
        subBuilder: $2.Pagination.create)
    ..aOS(3, _omitFieldNames ? '' : 'sort')
    ..aOS(4, _omitFieldNames ? '' : 'filter')
    ..aOS(5, _omitFieldNames ? '' : 'search')
    ..aOS(6, _omitFieldNames ? '' : 'domainScope')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListRecordsRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListRecordsRequest copyWith(void Function(ListRecordsRequest) updates) =>
      super.copyWith((message) => updates(message as ListRecordsRequest))
          as ListRecordsRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static ListRecordsRequest create() => ListRecordsRequest._();
  @$core.override
  ListRecordsRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static ListRecordsRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ListRecordsRequest>(create);
  static ListRecordsRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get tableName => $_getSZ(0);
  @$pb.TagNumber(1)
  set tableName($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasTableName() => $_has(0);
  @$pb.TagNumber(1)
  void clearTableName() => $_clearField(1);

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

  @$pb.TagNumber(3)
  $core.String get sort => $_getSZ(2);
  @$pb.TagNumber(3)
  set sort($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasSort() => $_has(2);
  @$pb.TagNumber(3)
  void clearSort() => $_clearField(3);

  @$pb.TagNumber(4)
  $core.String get filter => $_getSZ(3);
  @$pb.TagNumber(4)
  set filter($core.String value) => $_setString(3, value);
  @$pb.TagNumber(4)
  $core.bool hasFilter() => $_has(3);
  @$pb.TagNumber(4)
  void clearFilter() => $_clearField(4);

  @$pb.TagNumber(5)
  $core.String get search => $_getSZ(4);
  @$pb.TagNumber(5)
  set search($core.String value) => $_setString(4, value);
  @$pb.TagNumber(5)
  $core.bool hasSearch() => $_has(4);
  @$pb.TagNumber(5)
  void clearSearch() => $_clearField(5);

  @$pb.TagNumber(6)
  $core.String get domainScope => $_getSZ(5);
  @$pb.TagNumber(6)
  set domainScope($core.String value) => $_setString(5, value);
  @$pb.TagNumber(6)
  $core.bool hasDomainScope() => $_has(5);
  @$pb.TagNumber(6)
  void clearDomainScope() => $_clearField(6);
}

/// レコード一覧レスポンス
class ListRecordsResponse extends $pb.GeneratedMessage {
  factory ListRecordsResponse({
    $core.Iterable<$1.Struct>? records,
    $2.PaginationResult? pagination,
  }) {
    final result = create();
    if (records != null) result.records.addAll(records);
    if (pagination != null) result.pagination = pagination;
    return result;
  }

  ListRecordsResponse._();

  factory ListRecordsResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory ListRecordsResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ListRecordsResponse',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.mastermaintenance.v1'),
      createEmptyInstance: create)
    ..pPM<$1.Struct>(1, _omitFieldNames ? '' : 'records',
        subBuilder: $1.Struct.create)
    ..aOM<$2.PaginationResult>(2, _omitFieldNames ? '' : 'pagination',
        subBuilder: $2.PaginationResult.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListRecordsResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListRecordsResponse copyWith(void Function(ListRecordsResponse) updates) =>
      super.copyWith((message) => updates(message as ListRecordsResponse))
          as ListRecordsResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static ListRecordsResponse create() => ListRecordsResponse._();
  @$core.override
  ListRecordsResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static ListRecordsResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ListRecordsResponse>(create);
  static ListRecordsResponse? _defaultInstance;

  @$pb.TagNumber(1)
  $pb.PbList<$1.Struct> get records => $_getList(0);

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

/// レコード作成リクエスト
class CreateRecordRequest extends $pb.GeneratedMessage {
  factory CreateRecordRequest({
    $core.String? tableName,
    $1.Struct? data,
    $core.String? domainScope,
  }) {
    final result = create();
    if (tableName != null) result.tableName = tableName;
    if (data != null) result.data = data;
    if (domainScope != null) result.domainScope = domainScope;
    return result;
  }

  CreateRecordRequest._();

  factory CreateRecordRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory CreateRecordRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'CreateRecordRequest',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.mastermaintenance.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'tableName')
    ..aOM<$1.Struct>(2, _omitFieldNames ? '' : 'data',
        subBuilder: $1.Struct.create)
    ..aOS(3, _omitFieldNames ? '' : 'domainScope')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  CreateRecordRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  CreateRecordRequest copyWith(void Function(CreateRecordRequest) updates) =>
      super.copyWith((message) => updates(message as CreateRecordRequest))
          as CreateRecordRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static CreateRecordRequest create() => CreateRecordRequest._();
  @$core.override
  CreateRecordRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static CreateRecordRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<CreateRecordRequest>(create);
  static CreateRecordRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get tableName => $_getSZ(0);
  @$pb.TagNumber(1)
  set tableName($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasTableName() => $_has(0);
  @$pb.TagNumber(1)
  void clearTableName() => $_clearField(1);

  @$pb.TagNumber(2)
  $1.Struct get data => $_getN(1);
  @$pb.TagNumber(2)
  set data($1.Struct value) => $_setField(2, value);
  @$pb.TagNumber(2)
  $core.bool hasData() => $_has(1);
  @$pb.TagNumber(2)
  void clearData() => $_clearField(2);
  @$pb.TagNumber(2)
  $1.Struct ensureData() => $_ensure(1);

  @$pb.TagNumber(3)
  $core.String get domainScope => $_getSZ(2);
  @$pb.TagNumber(3)
  set domainScope($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasDomainScope() => $_has(2);
  @$pb.TagNumber(3)
  void clearDomainScope() => $_clearField(3);
}

/// レコード更新リクエスト
class UpdateRecordRequest extends $pb.GeneratedMessage {
  factory UpdateRecordRequest({
    $core.String? tableName,
    $core.String? recordId,
    $1.Struct? data,
    $core.String? domainScope,
  }) {
    final result = create();
    if (tableName != null) result.tableName = tableName;
    if (recordId != null) result.recordId = recordId;
    if (data != null) result.data = data;
    if (domainScope != null) result.domainScope = domainScope;
    return result;
  }

  UpdateRecordRequest._();

  factory UpdateRecordRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory UpdateRecordRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'UpdateRecordRequest',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.mastermaintenance.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'tableName')
    ..aOS(2, _omitFieldNames ? '' : 'recordId')
    ..aOM<$1.Struct>(3, _omitFieldNames ? '' : 'data',
        subBuilder: $1.Struct.create)
    ..aOS(4, _omitFieldNames ? '' : 'domainScope')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  UpdateRecordRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  UpdateRecordRequest copyWith(void Function(UpdateRecordRequest) updates) =>
      super.copyWith((message) => updates(message as UpdateRecordRequest))
          as UpdateRecordRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static UpdateRecordRequest create() => UpdateRecordRequest._();
  @$core.override
  UpdateRecordRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static UpdateRecordRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<UpdateRecordRequest>(create);
  static UpdateRecordRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get tableName => $_getSZ(0);
  @$pb.TagNumber(1)
  set tableName($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasTableName() => $_has(0);
  @$pb.TagNumber(1)
  void clearTableName() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get recordId => $_getSZ(1);
  @$pb.TagNumber(2)
  set recordId($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasRecordId() => $_has(1);
  @$pb.TagNumber(2)
  void clearRecordId() => $_clearField(2);

  @$pb.TagNumber(3)
  $1.Struct get data => $_getN(2);
  @$pb.TagNumber(3)
  set data($1.Struct value) => $_setField(3, value);
  @$pb.TagNumber(3)
  $core.bool hasData() => $_has(2);
  @$pb.TagNumber(3)
  void clearData() => $_clearField(3);
  @$pb.TagNumber(3)
  $1.Struct ensureData() => $_ensure(2);

  @$pb.TagNumber(4)
  $core.String get domainScope => $_getSZ(3);
  @$pb.TagNumber(4)
  set domainScope($core.String value) => $_setString(3, value);
  @$pb.TagNumber(4)
  $core.bool hasDomainScope() => $_has(3);
  @$pb.TagNumber(4)
  void clearDomainScope() => $_clearField(4);
}

/// レコード削除リクエスト
class DeleteRecordRequest extends $pb.GeneratedMessage {
  factory DeleteRecordRequest({
    $core.String? tableName,
    $core.String? recordId,
    $core.String? domainScope,
  }) {
    final result = create();
    if (tableName != null) result.tableName = tableName;
    if (recordId != null) result.recordId = recordId;
    if (domainScope != null) result.domainScope = domainScope;
    return result;
  }

  DeleteRecordRequest._();

  factory DeleteRecordRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory DeleteRecordRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'DeleteRecordRequest',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.mastermaintenance.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'tableName')
    ..aOS(2, _omitFieldNames ? '' : 'recordId')
    ..aOS(3, _omitFieldNames ? '' : 'domainScope')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  DeleteRecordRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  DeleteRecordRequest copyWith(void Function(DeleteRecordRequest) updates) =>
      super.copyWith((message) => updates(message as DeleteRecordRequest))
          as DeleteRecordRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static DeleteRecordRequest create() => DeleteRecordRequest._();
  @$core.override
  DeleteRecordRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static DeleteRecordRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<DeleteRecordRequest>(create);
  static DeleteRecordRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get tableName => $_getSZ(0);
  @$pb.TagNumber(1)
  set tableName($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasTableName() => $_has(0);
  @$pb.TagNumber(1)
  void clearTableName() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get recordId => $_getSZ(1);
  @$pb.TagNumber(2)
  set recordId($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasRecordId() => $_has(1);
  @$pb.TagNumber(2)
  void clearRecordId() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.String get domainScope => $_getSZ(2);
  @$pb.TagNumber(3)
  set domainScope($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasDomainScope() => $_has(2);
  @$pb.TagNumber(3)
  void clearDomainScope() => $_clearField(3);
}

/// レコード削除レスポンス
class DeleteRecordResponse extends $pb.GeneratedMessage {
  factory DeleteRecordResponse({
    $core.bool? success,
  }) {
    final result = create();
    if (success != null) result.success = success;
    return result;
  }

  DeleteRecordResponse._();

  factory DeleteRecordResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory DeleteRecordResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'DeleteRecordResponse',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.mastermaintenance.v1'),
      createEmptyInstance: create)
    ..aOB(1, _omitFieldNames ? '' : 'success')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  DeleteRecordResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  DeleteRecordResponse copyWith(void Function(DeleteRecordResponse) updates) =>
      super.copyWith((message) => updates(message as DeleteRecordResponse))
          as DeleteRecordResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static DeleteRecordResponse create() => DeleteRecordResponse._();
  @$core.override
  DeleteRecordResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static DeleteRecordResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<DeleteRecordResponse>(create);
  static DeleteRecordResponse? _defaultInstance;

  @$pb.TagNumber(1)
  $core.bool get success => $_getBF(0);
  @$pb.TagNumber(1)
  set success($core.bool value) => $_setBool(0, value);
  @$pb.TagNumber(1)
  $core.bool hasSuccess() => $_has(0);
  @$pb.TagNumber(1)
  void clearSuccess() => $_clearField(1);
}

/// 整合性チェックリクエスト
class CheckConsistencyRequest extends $pb.GeneratedMessage {
  factory CheckConsistencyRequest({
    $core.String? tableName,
    $core.Iterable<$core.String>? ruleIds,
    $core.String? domainScope,
  }) {
    final result = create();
    if (tableName != null) result.tableName = tableName;
    if (ruleIds != null) result.ruleIds.addAll(ruleIds);
    if (domainScope != null) result.domainScope = domainScope;
    return result;
  }

  CheckConsistencyRequest._();

  factory CheckConsistencyRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory CheckConsistencyRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'CheckConsistencyRequest',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.mastermaintenance.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'tableName')
    ..pPS(2, _omitFieldNames ? '' : 'ruleIds')
    ..aOS(3, _omitFieldNames ? '' : 'domainScope')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  CheckConsistencyRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  CheckConsistencyRequest copyWith(
          void Function(CheckConsistencyRequest) updates) =>
      super.copyWith((message) => updates(message as CheckConsistencyRequest))
          as CheckConsistencyRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static CheckConsistencyRequest create() => CheckConsistencyRequest._();
  @$core.override
  CheckConsistencyRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static CheckConsistencyRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<CheckConsistencyRequest>(create);
  static CheckConsistencyRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get tableName => $_getSZ(0);
  @$pb.TagNumber(1)
  set tableName($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasTableName() => $_has(0);
  @$pb.TagNumber(1)
  void clearTableName() => $_clearField(1);

  @$pb.TagNumber(2)
  $pb.PbList<$core.String> get ruleIds => $_getList(1);

  @$pb.TagNumber(3)
  $core.String get domainScope => $_getSZ(2);
  @$pb.TagNumber(3)
  set domainScope($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasDomainScope() => $_has(2);
  @$pb.TagNumber(3)
  void clearDomainScope() => $_clearField(3);
}

/// 整合性チェックレスポンス
class CheckConsistencyResponse extends $pb.GeneratedMessage {
  factory CheckConsistencyResponse({
    $core.Iterable<ConsistencyResult>? results,
    $core.int? totalChecked,
    $core.int? errorCount,
    $core.int? warningCount,
  }) {
    final result = create();
    if (results != null) result.results.addAll(results);
    if (totalChecked != null) result.totalChecked = totalChecked;
    if (errorCount != null) result.errorCount = errorCount;
    if (warningCount != null) result.warningCount = warningCount;
    return result;
  }

  CheckConsistencyResponse._();

  factory CheckConsistencyResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory CheckConsistencyResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'CheckConsistencyResponse',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.mastermaintenance.v1'),
      createEmptyInstance: create)
    ..pPM<ConsistencyResult>(1, _omitFieldNames ? '' : 'results',
        subBuilder: ConsistencyResult.create)
    ..aI(2, _omitFieldNames ? '' : 'totalChecked')
    ..aI(3, _omitFieldNames ? '' : 'errorCount')
    ..aI(4, _omitFieldNames ? '' : 'warningCount')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  CheckConsistencyResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  CheckConsistencyResponse copyWith(
          void Function(CheckConsistencyResponse) updates) =>
      super.copyWith((message) => updates(message as CheckConsistencyResponse))
          as CheckConsistencyResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static CheckConsistencyResponse create() => CheckConsistencyResponse._();
  @$core.override
  CheckConsistencyResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static CheckConsistencyResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<CheckConsistencyResponse>(create);
  static CheckConsistencyResponse? _defaultInstance;

  @$pb.TagNumber(1)
  $pb.PbList<ConsistencyResult> get results => $_getList(0);

  @$pb.TagNumber(2)
  $core.int get totalChecked => $_getIZ(1);
  @$pb.TagNumber(2)
  set totalChecked($core.int value) => $_setSignedInt32(1, value);
  @$pb.TagNumber(2)
  $core.bool hasTotalChecked() => $_has(1);
  @$pb.TagNumber(2)
  void clearTotalChecked() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.int get errorCount => $_getIZ(2);
  @$pb.TagNumber(3)
  set errorCount($core.int value) => $_setSignedInt32(2, value);
  @$pb.TagNumber(3)
  $core.bool hasErrorCount() => $_has(2);
  @$pb.TagNumber(3)
  void clearErrorCount() => $_clearField(3);

  @$pb.TagNumber(4)
  $core.int get warningCount => $_getIZ(3);
  @$pb.TagNumber(4)
  set warningCount($core.int value) => $_setSignedInt32(3, value);
  @$pb.TagNumber(4)
  $core.bool hasWarningCount() => $_has(3);
  @$pb.TagNumber(4)
  void clearWarningCount() => $_clearField(4);
}

class CreateRuleRequest extends $pb.GeneratedMessage {
  factory CreateRuleRequest({
    $1.Struct? data,
  }) {
    final result = create();
    if (data != null) result.data = data;
    return result;
  }

  CreateRuleRequest._();

  factory CreateRuleRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory CreateRuleRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'CreateRuleRequest',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.mastermaintenance.v1'),
      createEmptyInstance: create)
    ..aOM<$1.Struct>(1, _omitFieldNames ? '' : 'data',
        subBuilder: $1.Struct.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  CreateRuleRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  CreateRuleRequest copyWith(void Function(CreateRuleRequest) updates) =>
      super.copyWith((message) => updates(message as CreateRuleRequest))
          as CreateRuleRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static CreateRuleRequest create() => CreateRuleRequest._();
  @$core.override
  CreateRuleRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static CreateRuleRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<CreateRuleRequest>(create);
  static CreateRuleRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $1.Struct get data => $_getN(0);
  @$pb.TagNumber(1)
  set data($1.Struct value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasData() => $_has(0);
  @$pb.TagNumber(1)
  void clearData() => $_clearField(1);
  @$pb.TagNumber(1)
  $1.Struct ensureData() => $_ensure(0);
}

class CreateRuleResponse extends $pb.GeneratedMessage {
  factory CreateRuleResponse({
    ConsistencyRule? rule,
  }) {
    final result = create();
    if (rule != null) result.rule = rule;
    return result;
  }

  CreateRuleResponse._();

  factory CreateRuleResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory CreateRuleResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'CreateRuleResponse',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.mastermaintenance.v1'),
      createEmptyInstance: create)
    ..aOM<ConsistencyRule>(1, _omitFieldNames ? '' : 'rule',
        subBuilder: ConsistencyRule.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  CreateRuleResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  CreateRuleResponse copyWith(void Function(CreateRuleResponse) updates) =>
      super.copyWith((message) => updates(message as CreateRuleResponse))
          as CreateRuleResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static CreateRuleResponse create() => CreateRuleResponse._();
  @$core.override
  CreateRuleResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static CreateRuleResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<CreateRuleResponse>(create);
  static CreateRuleResponse? _defaultInstance;

  @$pb.TagNumber(1)
  ConsistencyRule get rule => $_getN(0);
  @$pb.TagNumber(1)
  set rule(ConsistencyRule value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasRule() => $_has(0);
  @$pb.TagNumber(1)
  void clearRule() => $_clearField(1);
  @$pb.TagNumber(1)
  ConsistencyRule ensureRule() => $_ensure(0);
}

class GetRuleRequest extends $pb.GeneratedMessage {
  factory GetRuleRequest({
    $core.String? ruleId,
  }) {
    final result = create();
    if (ruleId != null) result.ruleId = ruleId;
    return result;
  }

  GetRuleRequest._();

  factory GetRuleRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory GetRuleRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'GetRuleRequest',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.mastermaintenance.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'ruleId')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetRuleRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetRuleRequest copyWith(void Function(GetRuleRequest) updates) =>
      super.copyWith((message) => updates(message as GetRuleRequest))
          as GetRuleRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static GetRuleRequest create() => GetRuleRequest._();
  @$core.override
  GetRuleRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static GetRuleRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<GetRuleRequest>(create);
  static GetRuleRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get ruleId => $_getSZ(0);
  @$pb.TagNumber(1)
  set ruleId($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasRuleId() => $_has(0);
  @$pb.TagNumber(1)
  void clearRuleId() => $_clearField(1);
}

class GetRuleResponse extends $pb.GeneratedMessage {
  factory GetRuleResponse({
    ConsistencyRule? rule,
  }) {
    final result = create();
    if (rule != null) result.rule = rule;
    return result;
  }

  GetRuleResponse._();

  factory GetRuleResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory GetRuleResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'GetRuleResponse',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.mastermaintenance.v1'),
      createEmptyInstance: create)
    ..aOM<ConsistencyRule>(1, _omitFieldNames ? '' : 'rule',
        subBuilder: ConsistencyRule.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetRuleResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetRuleResponse copyWith(void Function(GetRuleResponse) updates) =>
      super.copyWith((message) => updates(message as GetRuleResponse))
          as GetRuleResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static GetRuleResponse create() => GetRuleResponse._();
  @$core.override
  GetRuleResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static GetRuleResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<GetRuleResponse>(create);
  static GetRuleResponse? _defaultInstance;

  @$pb.TagNumber(1)
  ConsistencyRule get rule => $_getN(0);
  @$pb.TagNumber(1)
  set rule(ConsistencyRule value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasRule() => $_has(0);
  @$pb.TagNumber(1)
  void clearRule() => $_clearField(1);
  @$pb.TagNumber(1)
  ConsistencyRule ensureRule() => $_ensure(0);
}

class UpdateRuleRequest extends $pb.GeneratedMessage {
  factory UpdateRuleRequest({
    $core.String? ruleId,
    $1.Struct? data,
  }) {
    final result = create();
    if (ruleId != null) result.ruleId = ruleId;
    if (data != null) result.data = data;
    return result;
  }

  UpdateRuleRequest._();

  factory UpdateRuleRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory UpdateRuleRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'UpdateRuleRequest',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.mastermaintenance.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'ruleId')
    ..aOM<$1.Struct>(2, _omitFieldNames ? '' : 'data',
        subBuilder: $1.Struct.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  UpdateRuleRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  UpdateRuleRequest copyWith(void Function(UpdateRuleRequest) updates) =>
      super.copyWith((message) => updates(message as UpdateRuleRequest))
          as UpdateRuleRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static UpdateRuleRequest create() => UpdateRuleRequest._();
  @$core.override
  UpdateRuleRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static UpdateRuleRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<UpdateRuleRequest>(create);
  static UpdateRuleRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get ruleId => $_getSZ(0);
  @$pb.TagNumber(1)
  set ruleId($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasRuleId() => $_has(0);
  @$pb.TagNumber(1)
  void clearRuleId() => $_clearField(1);

  @$pb.TagNumber(2)
  $1.Struct get data => $_getN(1);
  @$pb.TagNumber(2)
  set data($1.Struct value) => $_setField(2, value);
  @$pb.TagNumber(2)
  $core.bool hasData() => $_has(1);
  @$pb.TagNumber(2)
  void clearData() => $_clearField(2);
  @$pb.TagNumber(2)
  $1.Struct ensureData() => $_ensure(1);
}

class UpdateRuleResponse extends $pb.GeneratedMessage {
  factory UpdateRuleResponse({
    ConsistencyRule? rule,
  }) {
    final result = create();
    if (rule != null) result.rule = rule;
    return result;
  }

  UpdateRuleResponse._();

  factory UpdateRuleResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory UpdateRuleResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'UpdateRuleResponse',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.mastermaintenance.v1'),
      createEmptyInstance: create)
    ..aOM<ConsistencyRule>(1, _omitFieldNames ? '' : 'rule',
        subBuilder: ConsistencyRule.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  UpdateRuleResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  UpdateRuleResponse copyWith(void Function(UpdateRuleResponse) updates) =>
      super.copyWith((message) => updates(message as UpdateRuleResponse))
          as UpdateRuleResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static UpdateRuleResponse create() => UpdateRuleResponse._();
  @$core.override
  UpdateRuleResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static UpdateRuleResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<UpdateRuleResponse>(create);
  static UpdateRuleResponse? _defaultInstance;

  @$pb.TagNumber(1)
  ConsistencyRule get rule => $_getN(0);
  @$pb.TagNumber(1)
  set rule(ConsistencyRule value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasRule() => $_has(0);
  @$pb.TagNumber(1)
  void clearRule() => $_clearField(1);
  @$pb.TagNumber(1)
  ConsistencyRule ensureRule() => $_ensure(0);
}

class DeleteRuleRequest extends $pb.GeneratedMessage {
  factory DeleteRuleRequest({
    $core.String? ruleId,
  }) {
    final result = create();
    if (ruleId != null) result.ruleId = ruleId;
    return result;
  }

  DeleteRuleRequest._();

  factory DeleteRuleRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory DeleteRuleRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'DeleteRuleRequest',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.mastermaintenance.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'ruleId')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  DeleteRuleRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  DeleteRuleRequest copyWith(void Function(DeleteRuleRequest) updates) =>
      super.copyWith((message) => updates(message as DeleteRuleRequest))
          as DeleteRuleRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static DeleteRuleRequest create() => DeleteRuleRequest._();
  @$core.override
  DeleteRuleRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static DeleteRuleRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<DeleteRuleRequest>(create);
  static DeleteRuleRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get ruleId => $_getSZ(0);
  @$pb.TagNumber(1)
  set ruleId($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasRuleId() => $_has(0);
  @$pb.TagNumber(1)
  void clearRuleId() => $_clearField(1);
}

class DeleteRuleResponse extends $pb.GeneratedMessage {
  factory DeleteRuleResponse({
    $core.bool? success,
  }) {
    final result = create();
    if (success != null) result.success = success;
    return result;
  }

  DeleteRuleResponse._();

  factory DeleteRuleResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory DeleteRuleResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'DeleteRuleResponse',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.mastermaintenance.v1'),
      createEmptyInstance: create)
    ..aOB(1, _omitFieldNames ? '' : 'success')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  DeleteRuleResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  DeleteRuleResponse copyWith(void Function(DeleteRuleResponse) updates) =>
      super.copyWith((message) => updates(message as DeleteRuleResponse))
          as DeleteRuleResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static DeleteRuleResponse create() => DeleteRuleResponse._();
  @$core.override
  DeleteRuleResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static DeleteRuleResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<DeleteRuleResponse>(create);
  static DeleteRuleResponse? _defaultInstance;

  @$pb.TagNumber(1)
  $core.bool get success => $_getBF(0);
  @$pb.TagNumber(1)
  set success($core.bool value) => $_setBool(0, value);
  @$pb.TagNumber(1)
  $core.bool hasSuccess() => $_has(0);
  @$pb.TagNumber(1)
  void clearSuccess() => $_clearField(1);
}

class ListRulesRequest extends $pb.GeneratedMessage {
  factory ListRulesRequest({
    $core.String? tableName,
    $core.String? ruleType,
    $core.String? severity,
    $2.Pagination? pagination,
  }) {
    final result = create();
    if (tableName != null) result.tableName = tableName;
    if (ruleType != null) result.ruleType = ruleType;
    if (severity != null) result.severity = severity;
    if (pagination != null) result.pagination = pagination;
    return result;
  }

  ListRulesRequest._();

  factory ListRulesRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory ListRulesRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ListRulesRequest',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.mastermaintenance.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'tableName')
    ..aOS(2, _omitFieldNames ? '' : 'ruleType')
    ..aOS(3, _omitFieldNames ? '' : 'severity')
    ..aOM<$2.Pagination>(4, _omitFieldNames ? '' : 'pagination',
        subBuilder: $2.Pagination.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListRulesRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListRulesRequest copyWith(void Function(ListRulesRequest) updates) =>
      super.copyWith((message) => updates(message as ListRulesRequest))
          as ListRulesRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static ListRulesRequest create() => ListRulesRequest._();
  @$core.override
  ListRulesRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static ListRulesRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ListRulesRequest>(create);
  static ListRulesRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get tableName => $_getSZ(0);
  @$pb.TagNumber(1)
  set tableName($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasTableName() => $_has(0);
  @$pb.TagNumber(1)
  void clearTableName() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get ruleType => $_getSZ(1);
  @$pb.TagNumber(2)
  set ruleType($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasRuleType() => $_has(1);
  @$pb.TagNumber(2)
  void clearRuleType() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.String get severity => $_getSZ(2);
  @$pb.TagNumber(3)
  set severity($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasSeverity() => $_has(2);
  @$pb.TagNumber(3)
  void clearSeverity() => $_clearField(3);

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

class ListRulesResponse extends $pb.GeneratedMessage {
  factory ListRulesResponse({
    $core.Iterable<ConsistencyRule>? rules,
    $2.PaginationResult? pagination,
  }) {
    final result = create();
    if (rules != null) result.rules.addAll(rules);
    if (pagination != null) result.pagination = pagination;
    return result;
  }

  ListRulesResponse._();

  factory ListRulesResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory ListRulesResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ListRulesResponse',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.mastermaintenance.v1'),
      createEmptyInstance: create)
    ..pPM<ConsistencyRule>(1, _omitFieldNames ? '' : 'rules',
        subBuilder: ConsistencyRule.create)
    ..aOM<$2.PaginationResult>(2, _omitFieldNames ? '' : 'pagination',
        subBuilder: $2.PaginationResult.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListRulesResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListRulesResponse copyWith(void Function(ListRulesResponse) updates) =>
      super.copyWith((message) => updates(message as ListRulesResponse))
          as ListRulesResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static ListRulesResponse create() => ListRulesResponse._();
  @$core.override
  ListRulesResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static ListRulesResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ListRulesResponse>(create);
  static ListRulesResponse? _defaultInstance;

  @$pb.TagNumber(1)
  $pb.PbList<ConsistencyRule> get rules => $_getList(0);

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

class ExecuteRuleRequest extends $pb.GeneratedMessage {
  factory ExecuteRuleRequest({
    $core.String? ruleId,
  }) {
    final result = create();
    if (ruleId != null) result.ruleId = ruleId;
    return result;
  }

  ExecuteRuleRequest._();

  factory ExecuteRuleRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory ExecuteRuleRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ExecuteRuleRequest',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.mastermaintenance.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'ruleId')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ExecuteRuleRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ExecuteRuleRequest copyWith(void Function(ExecuteRuleRequest) updates) =>
      super.copyWith((message) => updates(message as ExecuteRuleRequest))
          as ExecuteRuleRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static ExecuteRuleRequest create() => ExecuteRuleRequest._();
  @$core.override
  ExecuteRuleRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static ExecuteRuleRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ExecuteRuleRequest>(create);
  static ExecuteRuleRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get ruleId => $_getSZ(0);
  @$pb.TagNumber(1)
  set ruleId($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasRuleId() => $_has(0);
  @$pb.TagNumber(1)
  void clearRuleId() => $_clearField(1);
}

class ExecuteRuleResponse extends $pb.GeneratedMessage {
  factory ExecuteRuleResponse({
    $core.Iterable<ConsistencyResult>? results,
    $core.int? totalChecked,
    $core.int? errorCount,
    $core.int? warningCount,
  }) {
    final result = create();
    if (results != null) result.results.addAll(results);
    if (totalChecked != null) result.totalChecked = totalChecked;
    if (errorCount != null) result.errorCount = errorCount;
    if (warningCount != null) result.warningCount = warningCount;
    return result;
  }

  ExecuteRuleResponse._();

  factory ExecuteRuleResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory ExecuteRuleResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ExecuteRuleResponse',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.mastermaintenance.v1'),
      createEmptyInstance: create)
    ..pPM<ConsistencyResult>(1, _omitFieldNames ? '' : 'results',
        subBuilder: ConsistencyResult.create)
    ..aI(2, _omitFieldNames ? '' : 'totalChecked')
    ..aI(3, _omitFieldNames ? '' : 'errorCount')
    ..aI(4, _omitFieldNames ? '' : 'warningCount')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ExecuteRuleResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ExecuteRuleResponse copyWith(void Function(ExecuteRuleResponse) updates) =>
      super.copyWith((message) => updates(message as ExecuteRuleResponse))
          as ExecuteRuleResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static ExecuteRuleResponse create() => ExecuteRuleResponse._();
  @$core.override
  ExecuteRuleResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static ExecuteRuleResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ExecuteRuleResponse>(create);
  static ExecuteRuleResponse? _defaultInstance;

  @$pb.TagNumber(1)
  $pb.PbList<ConsistencyResult> get results => $_getList(0);

  @$pb.TagNumber(2)
  $core.int get totalChecked => $_getIZ(1);
  @$pb.TagNumber(2)
  set totalChecked($core.int value) => $_setSignedInt32(1, value);
  @$pb.TagNumber(2)
  $core.bool hasTotalChecked() => $_has(1);
  @$pb.TagNumber(2)
  void clearTotalChecked() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.int get errorCount => $_getIZ(2);
  @$pb.TagNumber(3)
  set errorCount($core.int value) => $_setSignedInt32(2, value);
  @$pb.TagNumber(3)
  $core.bool hasErrorCount() => $_has(2);
  @$pb.TagNumber(3)
  void clearErrorCount() => $_clearField(3);

  @$pb.TagNumber(4)
  $core.int get warningCount => $_getIZ(3);
  @$pb.TagNumber(4)
  set warningCount($core.int value) => $_setSignedInt32(3, value);
  @$pb.TagNumber(4)
  $core.bool hasWarningCount() => $_has(3);
  @$pb.TagNumber(4)
  void clearWarningCount() => $_clearField(4);
}

/// 整合性チェック結果
class ConsistencyResult extends $pb.GeneratedMessage {
  factory ConsistencyResult({
    $core.String? ruleId,
    $core.String? ruleName,
    $core.String? severity,
    $core.bool? passed,
    $core.String? message,
    $core.Iterable<$core.String>? affectedRecordIds,
  }) {
    final result = create();
    if (ruleId != null) result.ruleId = ruleId;
    if (ruleName != null) result.ruleName = ruleName;
    if (severity != null) result.severity = severity;
    if (passed != null) result.passed = passed;
    if (message != null) result.message = message;
    if (affectedRecordIds != null)
      result.affectedRecordIds.addAll(affectedRecordIds);
    return result;
  }

  ConsistencyResult._();

  factory ConsistencyResult.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory ConsistencyResult.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ConsistencyResult',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.mastermaintenance.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'ruleId')
    ..aOS(2, _omitFieldNames ? '' : 'ruleName')
    ..aOS(3, _omitFieldNames ? '' : 'severity')
    ..aOB(4, _omitFieldNames ? '' : 'passed')
    ..aOS(5, _omitFieldNames ? '' : 'message')
    ..pPS(6, _omitFieldNames ? '' : 'affectedRecordIds')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ConsistencyResult clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ConsistencyResult copyWith(void Function(ConsistencyResult) updates) =>
      super.copyWith((message) => updates(message as ConsistencyResult))
          as ConsistencyResult;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static ConsistencyResult create() => ConsistencyResult._();
  @$core.override
  ConsistencyResult createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static ConsistencyResult getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ConsistencyResult>(create);
  static ConsistencyResult? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get ruleId => $_getSZ(0);
  @$pb.TagNumber(1)
  set ruleId($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasRuleId() => $_has(0);
  @$pb.TagNumber(1)
  void clearRuleId() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get ruleName => $_getSZ(1);
  @$pb.TagNumber(2)
  set ruleName($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasRuleName() => $_has(1);
  @$pb.TagNumber(2)
  void clearRuleName() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.String get severity => $_getSZ(2);
  @$pb.TagNumber(3)
  set severity($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasSeverity() => $_has(2);
  @$pb.TagNumber(3)
  void clearSeverity() => $_clearField(3);

  @$pb.TagNumber(4)
  $core.bool get passed => $_getBF(3);
  @$pb.TagNumber(4)
  set passed($core.bool value) => $_setBool(3, value);
  @$pb.TagNumber(4)
  $core.bool hasPassed() => $_has(3);
  @$pb.TagNumber(4)
  void clearPassed() => $_clearField(4);

  @$pb.TagNumber(5)
  $core.String get message => $_getSZ(4);
  @$pb.TagNumber(5)
  set message($core.String value) => $_setString(4, value);
  @$pb.TagNumber(5)
  $core.bool hasMessage() => $_has(4);
  @$pb.TagNumber(5)
  void clearMessage() => $_clearField(5);

  @$pb.TagNumber(6)
  $pb.PbList<$core.String> get affectedRecordIds => $_getList(5);
}

class ConsistencyRule extends $pb.GeneratedMessage {
  factory ConsistencyRule({
    $core.String? id,
    $core.String? name,
    $core.String? description,
    $core.String? ruleType,
    $core.String? severity,
    $core.bool? isActive,
    $core.String? sourceTableId,
    $core.String? evaluationTiming,
    $core.String? errorMessageTemplate,
    $core.String? zenRuleJson,
    $core.String? createdBy,
    $2.Timestamp? createdAt,
    $2.Timestamp? updatedAt,
  }) {
    final result = create();
    if (id != null) result.id = id;
    if (name != null) result.name = name;
    if (description != null) result.description = description;
    if (ruleType != null) result.ruleType = ruleType;
    if (severity != null) result.severity = severity;
    if (isActive != null) result.isActive = isActive;
    if (sourceTableId != null) result.sourceTableId = sourceTableId;
    if (evaluationTiming != null) result.evaluationTiming = evaluationTiming;
    if (errorMessageTemplate != null)
      result.errorMessageTemplate = errorMessageTemplate;
    if (zenRuleJson != null) result.zenRuleJson = zenRuleJson;
    if (createdBy != null) result.createdBy = createdBy;
    if (createdAt != null) result.createdAt = createdAt;
    if (updatedAt != null) result.updatedAt = updatedAt;
    return result;
  }

  ConsistencyRule._();

  factory ConsistencyRule.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory ConsistencyRule.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ConsistencyRule',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.mastermaintenance.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'id')
    ..aOS(2, _omitFieldNames ? '' : 'name')
    ..aOS(3, _omitFieldNames ? '' : 'description')
    ..aOS(4, _omitFieldNames ? '' : 'ruleType')
    ..aOS(5, _omitFieldNames ? '' : 'severity')
    ..aOB(6, _omitFieldNames ? '' : 'isActive')
    ..aOS(7, _omitFieldNames ? '' : 'sourceTableId')
    ..aOS(8, _omitFieldNames ? '' : 'evaluationTiming')
    ..aOS(9, _omitFieldNames ? '' : 'errorMessageTemplate')
    ..aOS(10, _omitFieldNames ? '' : 'zenRuleJson')
    ..aOS(11, _omitFieldNames ? '' : 'createdBy')
    ..aOM<$2.Timestamp>(12, _omitFieldNames ? '' : 'createdAt',
        subBuilder: $2.Timestamp.create)
    ..aOM<$2.Timestamp>(13, _omitFieldNames ? '' : 'updatedAt',
        subBuilder: $2.Timestamp.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ConsistencyRule clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ConsistencyRule copyWith(void Function(ConsistencyRule) updates) =>
      super.copyWith((message) => updates(message as ConsistencyRule))
          as ConsistencyRule;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static ConsistencyRule create() => ConsistencyRule._();
  @$core.override
  ConsistencyRule createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static ConsistencyRule getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ConsistencyRule>(create);
  static ConsistencyRule? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get id => $_getSZ(0);
  @$pb.TagNumber(1)
  set id($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasId() => $_has(0);
  @$pb.TagNumber(1)
  void clearId() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get name => $_getSZ(1);
  @$pb.TagNumber(2)
  set name($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasName() => $_has(1);
  @$pb.TagNumber(2)
  void clearName() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.String get description => $_getSZ(2);
  @$pb.TagNumber(3)
  set description($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasDescription() => $_has(2);
  @$pb.TagNumber(3)
  void clearDescription() => $_clearField(3);

  @$pb.TagNumber(4)
  $core.String get ruleType => $_getSZ(3);
  @$pb.TagNumber(4)
  set ruleType($core.String value) => $_setString(3, value);
  @$pb.TagNumber(4)
  $core.bool hasRuleType() => $_has(3);
  @$pb.TagNumber(4)
  void clearRuleType() => $_clearField(4);

  @$pb.TagNumber(5)
  $core.String get severity => $_getSZ(4);
  @$pb.TagNumber(5)
  set severity($core.String value) => $_setString(4, value);
  @$pb.TagNumber(5)
  $core.bool hasSeverity() => $_has(4);
  @$pb.TagNumber(5)
  void clearSeverity() => $_clearField(5);

  @$pb.TagNumber(6)
  $core.bool get isActive => $_getBF(5);
  @$pb.TagNumber(6)
  set isActive($core.bool value) => $_setBool(5, value);
  @$pb.TagNumber(6)
  $core.bool hasIsActive() => $_has(5);
  @$pb.TagNumber(6)
  void clearIsActive() => $_clearField(6);

  @$pb.TagNumber(7)
  $core.String get sourceTableId => $_getSZ(6);
  @$pb.TagNumber(7)
  set sourceTableId($core.String value) => $_setString(6, value);
  @$pb.TagNumber(7)
  $core.bool hasSourceTableId() => $_has(6);
  @$pb.TagNumber(7)
  void clearSourceTableId() => $_clearField(7);

  @$pb.TagNumber(8)
  $core.String get evaluationTiming => $_getSZ(7);
  @$pb.TagNumber(8)
  set evaluationTiming($core.String value) => $_setString(7, value);
  @$pb.TagNumber(8)
  $core.bool hasEvaluationTiming() => $_has(7);
  @$pb.TagNumber(8)
  void clearEvaluationTiming() => $_clearField(8);

  @$pb.TagNumber(9)
  $core.String get errorMessageTemplate => $_getSZ(8);
  @$pb.TagNumber(9)
  set errorMessageTemplate($core.String value) => $_setString(8, value);
  @$pb.TagNumber(9)
  $core.bool hasErrorMessageTemplate() => $_has(8);
  @$pb.TagNumber(9)
  void clearErrorMessageTemplate() => $_clearField(9);

  @$pb.TagNumber(10)
  $core.String get zenRuleJson => $_getSZ(9);
  @$pb.TagNumber(10)
  set zenRuleJson($core.String value) => $_setString(9, value);
  @$pb.TagNumber(10)
  $core.bool hasZenRuleJson() => $_has(9);
  @$pb.TagNumber(10)
  void clearZenRuleJson() => $_clearField(10);

  @$pb.TagNumber(11)
  $core.String get createdBy => $_getSZ(10);
  @$pb.TagNumber(11)
  set createdBy($core.String value) => $_setString(10, value);
  @$pb.TagNumber(11)
  $core.bool hasCreatedBy() => $_has(10);
  @$pb.TagNumber(11)
  void clearCreatedBy() => $_clearField(11);

  @$pb.TagNumber(12)
  $2.Timestamp get createdAt => $_getN(11);
  @$pb.TagNumber(12)
  set createdAt($2.Timestamp value) => $_setField(12, value);
  @$pb.TagNumber(12)
  $core.bool hasCreatedAt() => $_has(11);
  @$pb.TagNumber(12)
  void clearCreatedAt() => $_clearField(12);
  @$pb.TagNumber(12)
  $2.Timestamp ensureCreatedAt() => $_ensure(11);

  @$pb.TagNumber(13)
  $2.Timestamp get updatedAt => $_getN(12);
  @$pb.TagNumber(13)
  set updatedAt($2.Timestamp value) => $_setField(13, value);
  @$pb.TagNumber(13)
  $core.bool hasUpdatedAt() => $_has(12);
  @$pb.TagNumber(13)
  void clearUpdatedAt() => $_clearField(13);
  @$pb.TagNumber(13)
  $2.Timestamp ensureUpdatedAt() => $_ensure(12);
}

/// バリデーション警告
class ValidationWarning extends $pb.GeneratedMessage {
  factory ValidationWarning({
    $core.String? ruleName,
    $core.String? message,
    $core.String? severity,
  }) {
    final result = create();
    if (ruleName != null) result.ruleName = ruleName;
    if (message != null) result.message = message;
    if (severity != null) result.severity = severity;
    return result;
  }

  ValidationWarning._();

  factory ValidationWarning.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory ValidationWarning.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ValidationWarning',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.mastermaintenance.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'ruleName')
    ..aOS(2, _omitFieldNames ? '' : 'message')
    ..aOS(3, _omitFieldNames ? '' : 'severity')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ValidationWarning clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ValidationWarning copyWith(void Function(ValidationWarning) updates) =>
      super.copyWith((message) => updates(message as ValidationWarning))
          as ValidationWarning;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static ValidationWarning create() => ValidationWarning._();
  @$core.override
  ValidationWarning createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static ValidationWarning getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ValidationWarning>(create);
  static ValidationWarning? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get ruleName => $_getSZ(0);
  @$pb.TagNumber(1)
  set ruleName($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasRuleName() => $_has(0);
  @$pb.TagNumber(1)
  void clearRuleName() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get message => $_getSZ(1);
  @$pb.TagNumber(2)
  set message($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasMessage() => $_has(1);
  @$pb.TagNumber(2)
  void clearMessage() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.String get severity => $_getSZ(2);
  @$pb.TagNumber(3)
  set severity($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasSeverity() => $_has(2);
  @$pb.TagNumber(3)
  void clearSeverity() => $_clearField(3);
}

/// テーブルスキーマ取得リクエスト
class GetTableSchemaRequest extends $pb.GeneratedMessage {
  factory GetTableSchemaRequest({
    $core.String? tableName,
  }) {
    final result = create();
    if (tableName != null) result.tableName = tableName;
    return result;
  }

  GetTableSchemaRequest._();

  factory GetTableSchemaRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory GetTableSchemaRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'GetTableSchemaRequest',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.mastermaintenance.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'tableName')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetTableSchemaRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetTableSchemaRequest copyWith(
          void Function(GetTableSchemaRequest) updates) =>
      super.copyWith((message) => updates(message as GetTableSchemaRequest))
          as GetTableSchemaRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static GetTableSchemaRequest create() => GetTableSchemaRequest._();
  @$core.override
  GetTableSchemaRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static GetTableSchemaRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<GetTableSchemaRequest>(create);
  static GetTableSchemaRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get tableName => $_getSZ(0);
  @$pb.TagNumber(1)
  set tableName($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasTableName() => $_has(0);
  @$pb.TagNumber(1)
  void clearTableName() => $_clearField(1);
}

/// テーブルスキーマレスポンス
class GetTableSchemaResponse extends $pb.GeneratedMessage {
  factory GetTableSchemaResponse({
    $core.String? jsonSchema,
  }) {
    final result = create();
    if (jsonSchema != null) result.jsonSchema = jsonSchema;
    return result;
  }

  GetTableSchemaResponse._();

  factory GetTableSchemaResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory GetTableSchemaResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'GetTableSchemaResponse',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.mastermaintenance.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'jsonSchema')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetTableSchemaResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetTableSchemaResponse copyWith(
          void Function(GetTableSchemaResponse) updates) =>
      super.copyWith((message) => updates(message as GetTableSchemaResponse))
          as GetTableSchemaResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static GetTableSchemaResponse create() => GetTableSchemaResponse._();
  @$core.override
  GetTableSchemaResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static GetTableSchemaResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<GetTableSchemaResponse>(create);
  static GetTableSchemaResponse? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get jsonSchema => $_getSZ(0);
  @$pb.TagNumber(1)
  set jsonSchema($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasJsonSchema() => $_has(0);
  @$pb.TagNumber(1)
  void clearJsonSchema() => $_clearField(1);
}

class ListColumnsRequest extends $pb.GeneratedMessage {
  factory ListColumnsRequest({
    $core.String? tableName,
  }) {
    final result = create();
    if (tableName != null) result.tableName = tableName;
    return result;
  }

  ListColumnsRequest._();

  factory ListColumnsRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory ListColumnsRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ListColumnsRequest',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.mastermaintenance.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'tableName')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListColumnsRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListColumnsRequest copyWith(void Function(ListColumnsRequest) updates) =>
      super.copyWith((message) => updates(message as ListColumnsRequest))
          as ListColumnsRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static ListColumnsRequest create() => ListColumnsRequest._();
  @$core.override
  ListColumnsRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static ListColumnsRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ListColumnsRequest>(create);
  static ListColumnsRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get tableName => $_getSZ(0);
  @$pb.TagNumber(1)
  set tableName($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasTableName() => $_has(0);
  @$pb.TagNumber(1)
  void clearTableName() => $_clearField(1);
}

class ListColumnsResponse extends $pb.GeneratedMessage {
  factory ListColumnsResponse({
    $core.Iterable<ColumnDefinition>? columns,
  }) {
    final result = create();
    if (columns != null) result.columns.addAll(columns);
    return result;
  }

  ListColumnsResponse._();

  factory ListColumnsResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory ListColumnsResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ListColumnsResponse',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.mastermaintenance.v1'),
      createEmptyInstance: create)
    ..pPM<ColumnDefinition>(1, _omitFieldNames ? '' : 'columns',
        subBuilder: ColumnDefinition.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListColumnsResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListColumnsResponse copyWith(void Function(ListColumnsResponse) updates) =>
      super.copyWith((message) => updates(message as ListColumnsResponse))
          as ListColumnsResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static ListColumnsResponse create() => ListColumnsResponse._();
  @$core.override
  ListColumnsResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static ListColumnsResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ListColumnsResponse>(create);
  static ListColumnsResponse? _defaultInstance;

  @$pb.TagNumber(1)
  $pb.PbList<ColumnDefinition> get columns => $_getList(0);
}

class CreateColumnsRequest extends $pb.GeneratedMessage {
  factory CreateColumnsRequest({
    $core.String? tableName,
    $core.Iterable<$1.Struct>? columns,
  }) {
    final result = create();
    if (tableName != null) result.tableName = tableName;
    if (columns != null) result.columns.addAll(columns);
    return result;
  }

  CreateColumnsRequest._();

  factory CreateColumnsRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory CreateColumnsRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'CreateColumnsRequest',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.mastermaintenance.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'tableName')
    ..pPM<$1.Struct>(2, _omitFieldNames ? '' : 'columns',
        subBuilder: $1.Struct.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  CreateColumnsRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  CreateColumnsRequest copyWith(void Function(CreateColumnsRequest) updates) =>
      super.copyWith((message) => updates(message as CreateColumnsRequest))
          as CreateColumnsRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static CreateColumnsRequest create() => CreateColumnsRequest._();
  @$core.override
  CreateColumnsRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static CreateColumnsRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<CreateColumnsRequest>(create);
  static CreateColumnsRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get tableName => $_getSZ(0);
  @$pb.TagNumber(1)
  set tableName($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasTableName() => $_has(0);
  @$pb.TagNumber(1)
  void clearTableName() => $_clearField(1);

  @$pb.TagNumber(2)
  $pb.PbList<$1.Struct> get columns => $_getList(1);
}

class CreateColumnsResponse extends $pb.GeneratedMessage {
  factory CreateColumnsResponse({
    $core.Iterable<ColumnDefinition>? columns,
  }) {
    final result = create();
    if (columns != null) result.columns.addAll(columns);
    return result;
  }

  CreateColumnsResponse._();

  factory CreateColumnsResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory CreateColumnsResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'CreateColumnsResponse',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.mastermaintenance.v1'),
      createEmptyInstance: create)
    ..pPM<ColumnDefinition>(1, _omitFieldNames ? '' : 'columns',
        subBuilder: ColumnDefinition.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  CreateColumnsResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  CreateColumnsResponse copyWith(
          void Function(CreateColumnsResponse) updates) =>
      super.copyWith((message) => updates(message as CreateColumnsResponse))
          as CreateColumnsResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static CreateColumnsResponse create() => CreateColumnsResponse._();
  @$core.override
  CreateColumnsResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static CreateColumnsResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<CreateColumnsResponse>(create);
  static CreateColumnsResponse? _defaultInstance;

  @$pb.TagNumber(1)
  $pb.PbList<ColumnDefinition> get columns => $_getList(0);
}

class UpdateColumnRequest extends $pb.GeneratedMessage {
  factory UpdateColumnRequest({
    $core.String? tableName,
    $core.String? columnName,
    $1.Struct? data,
  }) {
    final result = create();
    if (tableName != null) result.tableName = tableName;
    if (columnName != null) result.columnName = columnName;
    if (data != null) result.data = data;
    return result;
  }

  UpdateColumnRequest._();

  factory UpdateColumnRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory UpdateColumnRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'UpdateColumnRequest',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.mastermaintenance.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'tableName')
    ..aOS(2, _omitFieldNames ? '' : 'columnName')
    ..aOM<$1.Struct>(3, _omitFieldNames ? '' : 'data',
        subBuilder: $1.Struct.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  UpdateColumnRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  UpdateColumnRequest copyWith(void Function(UpdateColumnRequest) updates) =>
      super.copyWith((message) => updates(message as UpdateColumnRequest))
          as UpdateColumnRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static UpdateColumnRequest create() => UpdateColumnRequest._();
  @$core.override
  UpdateColumnRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static UpdateColumnRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<UpdateColumnRequest>(create);
  static UpdateColumnRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get tableName => $_getSZ(0);
  @$pb.TagNumber(1)
  set tableName($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasTableName() => $_has(0);
  @$pb.TagNumber(1)
  void clearTableName() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get columnName => $_getSZ(1);
  @$pb.TagNumber(2)
  set columnName($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasColumnName() => $_has(1);
  @$pb.TagNumber(2)
  void clearColumnName() => $_clearField(2);

  @$pb.TagNumber(3)
  $1.Struct get data => $_getN(2);
  @$pb.TagNumber(3)
  set data($1.Struct value) => $_setField(3, value);
  @$pb.TagNumber(3)
  $core.bool hasData() => $_has(2);
  @$pb.TagNumber(3)
  void clearData() => $_clearField(3);
  @$pb.TagNumber(3)
  $1.Struct ensureData() => $_ensure(2);
}

class UpdateColumnResponse extends $pb.GeneratedMessage {
  factory UpdateColumnResponse({
    ColumnDefinition? column,
  }) {
    final result = create();
    if (column != null) result.column = column;
    return result;
  }

  UpdateColumnResponse._();

  factory UpdateColumnResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory UpdateColumnResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'UpdateColumnResponse',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.mastermaintenance.v1'),
      createEmptyInstance: create)
    ..aOM<ColumnDefinition>(1, _omitFieldNames ? '' : 'column',
        subBuilder: ColumnDefinition.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  UpdateColumnResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  UpdateColumnResponse copyWith(void Function(UpdateColumnResponse) updates) =>
      super.copyWith((message) => updates(message as UpdateColumnResponse))
          as UpdateColumnResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static UpdateColumnResponse create() => UpdateColumnResponse._();
  @$core.override
  UpdateColumnResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static UpdateColumnResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<UpdateColumnResponse>(create);
  static UpdateColumnResponse? _defaultInstance;

  @$pb.TagNumber(1)
  ColumnDefinition get column => $_getN(0);
  @$pb.TagNumber(1)
  set column(ColumnDefinition value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasColumn() => $_has(0);
  @$pb.TagNumber(1)
  void clearColumn() => $_clearField(1);
  @$pb.TagNumber(1)
  ColumnDefinition ensureColumn() => $_ensure(0);
}

class DeleteColumnRequest extends $pb.GeneratedMessage {
  factory DeleteColumnRequest({
    $core.String? tableName,
    $core.String? columnName,
  }) {
    final result = create();
    if (tableName != null) result.tableName = tableName;
    if (columnName != null) result.columnName = columnName;
    return result;
  }

  DeleteColumnRequest._();

  factory DeleteColumnRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory DeleteColumnRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'DeleteColumnRequest',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.mastermaintenance.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'tableName')
    ..aOS(2, _omitFieldNames ? '' : 'columnName')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  DeleteColumnRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  DeleteColumnRequest copyWith(void Function(DeleteColumnRequest) updates) =>
      super.copyWith((message) => updates(message as DeleteColumnRequest))
          as DeleteColumnRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static DeleteColumnRequest create() => DeleteColumnRequest._();
  @$core.override
  DeleteColumnRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static DeleteColumnRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<DeleteColumnRequest>(create);
  static DeleteColumnRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get tableName => $_getSZ(0);
  @$pb.TagNumber(1)
  set tableName($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasTableName() => $_has(0);
  @$pb.TagNumber(1)
  void clearTableName() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get columnName => $_getSZ(1);
  @$pb.TagNumber(2)
  set columnName($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasColumnName() => $_has(1);
  @$pb.TagNumber(2)
  void clearColumnName() => $_clearField(2);
}

class DeleteColumnResponse extends $pb.GeneratedMessage {
  factory DeleteColumnResponse({
    $core.bool? success,
  }) {
    final result = create();
    if (success != null) result.success = success;
    return result;
  }

  DeleteColumnResponse._();

  factory DeleteColumnResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory DeleteColumnResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'DeleteColumnResponse',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.mastermaintenance.v1'),
      createEmptyInstance: create)
    ..aOB(1, _omitFieldNames ? '' : 'success')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  DeleteColumnResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  DeleteColumnResponse copyWith(void Function(DeleteColumnResponse) updates) =>
      super.copyWith((message) => updates(message as DeleteColumnResponse))
          as DeleteColumnResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static DeleteColumnResponse create() => DeleteColumnResponse._();
  @$core.override
  DeleteColumnResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static DeleteColumnResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<DeleteColumnResponse>(create);
  static DeleteColumnResponse? _defaultInstance;

  @$pb.TagNumber(1)
  $core.bool get success => $_getBF(0);
  @$pb.TagNumber(1)
  set success($core.bool value) => $_setBool(0, value);
  @$pb.TagNumber(1)
  $core.bool hasSuccess() => $_has(0);
  @$pb.TagNumber(1)
  void clearSuccess() => $_clearField(1);
}

class ListRelationshipsRequest extends $pb.GeneratedMessage {
  factory ListRelationshipsRequest() => create();

  ListRelationshipsRequest._();

  factory ListRelationshipsRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory ListRelationshipsRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ListRelationshipsRequest',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.mastermaintenance.v1'),
      createEmptyInstance: create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListRelationshipsRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListRelationshipsRequest copyWith(
          void Function(ListRelationshipsRequest) updates) =>
      super.copyWith((message) => updates(message as ListRelationshipsRequest))
          as ListRelationshipsRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static ListRelationshipsRequest create() => ListRelationshipsRequest._();
  @$core.override
  ListRelationshipsRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static ListRelationshipsRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ListRelationshipsRequest>(create);
  static ListRelationshipsRequest? _defaultInstance;
}

class ListRelationshipsResponse extends $pb.GeneratedMessage {
  factory ListRelationshipsResponse({
    $core.Iterable<TableRelationship>? relationships,
  }) {
    final result = create();
    if (relationships != null) result.relationships.addAll(relationships);
    return result;
  }

  ListRelationshipsResponse._();

  factory ListRelationshipsResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory ListRelationshipsResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ListRelationshipsResponse',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.mastermaintenance.v1'),
      createEmptyInstance: create)
    ..pPM<TableRelationship>(1, _omitFieldNames ? '' : 'relationships',
        subBuilder: TableRelationship.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListRelationshipsResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListRelationshipsResponse copyWith(
          void Function(ListRelationshipsResponse) updates) =>
      super.copyWith((message) => updates(message as ListRelationshipsResponse))
          as ListRelationshipsResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static ListRelationshipsResponse create() => ListRelationshipsResponse._();
  @$core.override
  ListRelationshipsResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static ListRelationshipsResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ListRelationshipsResponse>(create);
  static ListRelationshipsResponse? _defaultInstance;

  @$pb.TagNumber(1)
  $pb.PbList<TableRelationship> get relationships => $_getList(0);
}

class CreateRelationshipRequest extends $pb.GeneratedMessage {
  factory CreateRelationshipRequest({
    $1.Struct? data,
  }) {
    final result = create();
    if (data != null) result.data = data;
    return result;
  }

  CreateRelationshipRequest._();

  factory CreateRelationshipRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory CreateRelationshipRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'CreateRelationshipRequest',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.mastermaintenance.v1'),
      createEmptyInstance: create)
    ..aOM<$1.Struct>(1, _omitFieldNames ? '' : 'data',
        subBuilder: $1.Struct.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  CreateRelationshipRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  CreateRelationshipRequest copyWith(
          void Function(CreateRelationshipRequest) updates) =>
      super.copyWith((message) => updates(message as CreateRelationshipRequest))
          as CreateRelationshipRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static CreateRelationshipRequest create() => CreateRelationshipRequest._();
  @$core.override
  CreateRelationshipRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static CreateRelationshipRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<CreateRelationshipRequest>(create);
  static CreateRelationshipRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $1.Struct get data => $_getN(0);
  @$pb.TagNumber(1)
  set data($1.Struct value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasData() => $_has(0);
  @$pb.TagNumber(1)
  void clearData() => $_clearField(1);
  @$pb.TagNumber(1)
  $1.Struct ensureData() => $_ensure(0);
}

class CreateRelationshipResponse extends $pb.GeneratedMessage {
  factory CreateRelationshipResponse({
    TableRelationship? relationship,
  }) {
    final result = create();
    if (relationship != null) result.relationship = relationship;
    return result;
  }

  CreateRelationshipResponse._();

  factory CreateRelationshipResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory CreateRelationshipResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'CreateRelationshipResponse',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.mastermaintenance.v1'),
      createEmptyInstance: create)
    ..aOM<TableRelationship>(1, _omitFieldNames ? '' : 'relationship',
        subBuilder: TableRelationship.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  CreateRelationshipResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  CreateRelationshipResponse copyWith(
          void Function(CreateRelationshipResponse) updates) =>
      super.copyWith(
              (message) => updates(message as CreateRelationshipResponse))
          as CreateRelationshipResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static CreateRelationshipResponse create() => CreateRelationshipResponse._();
  @$core.override
  CreateRelationshipResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static CreateRelationshipResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<CreateRelationshipResponse>(create);
  static CreateRelationshipResponse? _defaultInstance;

  @$pb.TagNumber(1)
  TableRelationship get relationship => $_getN(0);
  @$pb.TagNumber(1)
  set relationship(TableRelationship value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasRelationship() => $_has(0);
  @$pb.TagNumber(1)
  void clearRelationship() => $_clearField(1);
  @$pb.TagNumber(1)
  TableRelationship ensureRelationship() => $_ensure(0);
}

class UpdateRelationshipRequest extends $pb.GeneratedMessage {
  factory UpdateRelationshipRequest({
    $core.String? relationshipId,
    $1.Struct? data,
  }) {
    final result = create();
    if (relationshipId != null) result.relationshipId = relationshipId;
    if (data != null) result.data = data;
    return result;
  }

  UpdateRelationshipRequest._();

  factory UpdateRelationshipRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory UpdateRelationshipRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'UpdateRelationshipRequest',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.mastermaintenance.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'relationshipId')
    ..aOM<$1.Struct>(2, _omitFieldNames ? '' : 'data',
        subBuilder: $1.Struct.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  UpdateRelationshipRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  UpdateRelationshipRequest copyWith(
          void Function(UpdateRelationshipRequest) updates) =>
      super.copyWith((message) => updates(message as UpdateRelationshipRequest))
          as UpdateRelationshipRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static UpdateRelationshipRequest create() => UpdateRelationshipRequest._();
  @$core.override
  UpdateRelationshipRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static UpdateRelationshipRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<UpdateRelationshipRequest>(create);
  static UpdateRelationshipRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get relationshipId => $_getSZ(0);
  @$pb.TagNumber(1)
  set relationshipId($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasRelationshipId() => $_has(0);
  @$pb.TagNumber(1)
  void clearRelationshipId() => $_clearField(1);

  @$pb.TagNumber(2)
  $1.Struct get data => $_getN(1);
  @$pb.TagNumber(2)
  set data($1.Struct value) => $_setField(2, value);
  @$pb.TagNumber(2)
  $core.bool hasData() => $_has(1);
  @$pb.TagNumber(2)
  void clearData() => $_clearField(2);
  @$pb.TagNumber(2)
  $1.Struct ensureData() => $_ensure(1);
}

class UpdateRelationshipResponse extends $pb.GeneratedMessage {
  factory UpdateRelationshipResponse({
    TableRelationship? relationship,
  }) {
    final result = create();
    if (relationship != null) result.relationship = relationship;
    return result;
  }

  UpdateRelationshipResponse._();

  factory UpdateRelationshipResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory UpdateRelationshipResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'UpdateRelationshipResponse',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.mastermaintenance.v1'),
      createEmptyInstance: create)
    ..aOM<TableRelationship>(1, _omitFieldNames ? '' : 'relationship',
        subBuilder: TableRelationship.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  UpdateRelationshipResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  UpdateRelationshipResponse copyWith(
          void Function(UpdateRelationshipResponse) updates) =>
      super.copyWith(
              (message) => updates(message as UpdateRelationshipResponse))
          as UpdateRelationshipResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static UpdateRelationshipResponse create() => UpdateRelationshipResponse._();
  @$core.override
  UpdateRelationshipResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static UpdateRelationshipResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<UpdateRelationshipResponse>(create);
  static UpdateRelationshipResponse? _defaultInstance;

  @$pb.TagNumber(1)
  TableRelationship get relationship => $_getN(0);
  @$pb.TagNumber(1)
  set relationship(TableRelationship value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasRelationship() => $_has(0);
  @$pb.TagNumber(1)
  void clearRelationship() => $_clearField(1);
  @$pb.TagNumber(1)
  TableRelationship ensureRelationship() => $_ensure(0);
}

class DeleteRelationshipRequest extends $pb.GeneratedMessage {
  factory DeleteRelationshipRequest({
    $core.String? relationshipId,
  }) {
    final result = create();
    if (relationshipId != null) result.relationshipId = relationshipId;
    return result;
  }

  DeleteRelationshipRequest._();

  factory DeleteRelationshipRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory DeleteRelationshipRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'DeleteRelationshipRequest',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.mastermaintenance.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'relationshipId')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  DeleteRelationshipRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  DeleteRelationshipRequest copyWith(
          void Function(DeleteRelationshipRequest) updates) =>
      super.copyWith((message) => updates(message as DeleteRelationshipRequest))
          as DeleteRelationshipRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static DeleteRelationshipRequest create() => DeleteRelationshipRequest._();
  @$core.override
  DeleteRelationshipRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static DeleteRelationshipRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<DeleteRelationshipRequest>(create);
  static DeleteRelationshipRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get relationshipId => $_getSZ(0);
  @$pb.TagNumber(1)
  set relationshipId($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasRelationshipId() => $_has(0);
  @$pb.TagNumber(1)
  void clearRelationshipId() => $_clearField(1);
}

class DeleteRelationshipResponse extends $pb.GeneratedMessage {
  factory DeleteRelationshipResponse({
    $core.bool? success,
  }) {
    final result = create();
    if (success != null) result.success = success;
    return result;
  }

  DeleteRelationshipResponse._();

  factory DeleteRelationshipResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory DeleteRelationshipResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'DeleteRelationshipResponse',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.mastermaintenance.v1'),
      createEmptyInstance: create)
    ..aOB(1, _omitFieldNames ? '' : 'success')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  DeleteRelationshipResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  DeleteRelationshipResponse copyWith(
          void Function(DeleteRelationshipResponse) updates) =>
      super.copyWith(
              (message) => updates(message as DeleteRelationshipResponse))
          as DeleteRelationshipResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static DeleteRelationshipResponse create() => DeleteRelationshipResponse._();
  @$core.override
  DeleteRelationshipResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static DeleteRelationshipResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<DeleteRelationshipResponse>(create);
  static DeleteRelationshipResponse? _defaultInstance;

  @$pb.TagNumber(1)
  $core.bool get success => $_getBF(0);
  @$pb.TagNumber(1)
  set success($core.bool value) => $_setBool(0, value);
  @$pb.TagNumber(1)
  $core.bool hasSuccess() => $_has(0);
  @$pb.TagNumber(1)
  void clearSuccess() => $_clearField(1);
}

class ImportRecordsRequest extends $pb.GeneratedMessage {
  factory ImportRecordsRequest({
    $core.String? tableName,
    $1.Struct? data,
  }) {
    final result = create();
    if (tableName != null) result.tableName = tableName;
    if (data != null) result.data = data;
    return result;
  }

  ImportRecordsRequest._();

  factory ImportRecordsRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory ImportRecordsRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ImportRecordsRequest',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.mastermaintenance.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'tableName')
    ..aOM<$1.Struct>(2, _omitFieldNames ? '' : 'data',
        subBuilder: $1.Struct.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ImportRecordsRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ImportRecordsRequest copyWith(void Function(ImportRecordsRequest) updates) =>
      super.copyWith((message) => updates(message as ImportRecordsRequest))
          as ImportRecordsRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static ImportRecordsRequest create() => ImportRecordsRequest._();
  @$core.override
  ImportRecordsRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static ImportRecordsRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ImportRecordsRequest>(create);
  static ImportRecordsRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get tableName => $_getSZ(0);
  @$pb.TagNumber(1)
  set tableName($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasTableName() => $_has(0);
  @$pb.TagNumber(1)
  void clearTableName() => $_clearField(1);

  @$pb.TagNumber(2)
  $1.Struct get data => $_getN(1);
  @$pb.TagNumber(2)
  set data($1.Struct value) => $_setField(2, value);
  @$pb.TagNumber(2)
  $core.bool hasData() => $_has(1);
  @$pb.TagNumber(2)
  void clearData() => $_clearField(2);
  @$pb.TagNumber(2)
  $1.Struct ensureData() => $_ensure(1);
}

class ImportRecordsResponse extends $pb.GeneratedMessage {
  factory ImportRecordsResponse({
    ImportJob? importJob,
  }) {
    final result = create();
    if (importJob != null) result.importJob = importJob;
    return result;
  }

  ImportRecordsResponse._();

  factory ImportRecordsResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory ImportRecordsResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ImportRecordsResponse',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.mastermaintenance.v1'),
      createEmptyInstance: create)
    ..aOM<ImportJob>(1, _omitFieldNames ? '' : 'importJob',
        subBuilder: ImportJob.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ImportRecordsResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ImportRecordsResponse copyWith(
          void Function(ImportRecordsResponse) updates) =>
      super.copyWith((message) => updates(message as ImportRecordsResponse))
          as ImportRecordsResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static ImportRecordsResponse create() => ImportRecordsResponse._();
  @$core.override
  ImportRecordsResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static ImportRecordsResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ImportRecordsResponse>(create);
  static ImportRecordsResponse? _defaultInstance;

  @$pb.TagNumber(1)
  ImportJob get importJob => $_getN(0);
  @$pb.TagNumber(1)
  set importJob(ImportJob value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasImportJob() => $_has(0);
  @$pb.TagNumber(1)
  void clearImportJob() => $_clearField(1);
  @$pb.TagNumber(1)
  ImportJob ensureImportJob() => $_ensure(0);
}

class ExportRecordsRequest extends $pb.GeneratedMessage {
  factory ExportRecordsRequest({
    $core.String? tableName,
  }) {
    final result = create();
    if (tableName != null) result.tableName = tableName;
    return result;
  }

  ExportRecordsRequest._();

  factory ExportRecordsRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory ExportRecordsRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ExportRecordsRequest',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.mastermaintenance.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'tableName')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ExportRecordsRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ExportRecordsRequest copyWith(void Function(ExportRecordsRequest) updates) =>
      super.copyWith((message) => updates(message as ExportRecordsRequest))
          as ExportRecordsRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static ExportRecordsRequest create() => ExportRecordsRequest._();
  @$core.override
  ExportRecordsRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static ExportRecordsRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ExportRecordsRequest>(create);
  static ExportRecordsRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get tableName => $_getSZ(0);
  @$pb.TagNumber(1)
  set tableName($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasTableName() => $_has(0);
  @$pb.TagNumber(1)
  void clearTableName() => $_clearField(1);
}

class ExportRecordsResponse extends $pb.GeneratedMessage {
  factory ExportRecordsResponse({
    $1.Struct? data,
  }) {
    final result = create();
    if (data != null) result.data = data;
    return result;
  }

  ExportRecordsResponse._();

  factory ExportRecordsResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory ExportRecordsResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ExportRecordsResponse',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.mastermaintenance.v1'),
      createEmptyInstance: create)
    ..aOM<$1.Struct>(1, _omitFieldNames ? '' : 'data',
        subBuilder: $1.Struct.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ExportRecordsResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ExportRecordsResponse copyWith(
          void Function(ExportRecordsResponse) updates) =>
      super.copyWith((message) => updates(message as ExportRecordsResponse))
          as ExportRecordsResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static ExportRecordsResponse create() => ExportRecordsResponse._();
  @$core.override
  ExportRecordsResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static ExportRecordsResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ExportRecordsResponse>(create);
  static ExportRecordsResponse? _defaultInstance;

  @$pb.TagNumber(1)
  $1.Struct get data => $_getN(0);
  @$pb.TagNumber(1)
  set data($1.Struct value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasData() => $_has(0);
  @$pb.TagNumber(1)
  void clearData() => $_clearField(1);
  @$pb.TagNumber(1)
  $1.Struct ensureData() => $_ensure(0);
}

class GetImportJobRequest extends $pb.GeneratedMessage {
  factory GetImportJobRequest({
    $core.String? importJobId,
  }) {
    final result = create();
    if (importJobId != null) result.importJobId = importJobId;
    return result;
  }

  GetImportJobRequest._();

  factory GetImportJobRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory GetImportJobRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'GetImportJobRequest',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.mastermaintenance.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'importJobId')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetImportJobRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetImportJobRequest copyWith(void Function(GetImportJobRequest) updates) =>
      super.copyWith((message) => updates(message as GetImportJobRequest))
          as GetImportJobRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static GetImportJobRequest create() => GetImportJobRequest._();
  @$core.override
  GetImportJobRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static GetImportJobRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<GetImportJobRequest>(create);
  static GetImportJobRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get importJobId => $_getSZ(0);
  @$pb.TagNumber(1)
  set importJobId($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasImportJobId() => $_has(0);
  @$pb.TagNumber(1)
  void clearImportJobId() => $_clearField(1);
}

class GetImportJobResponse extends $pb.GeneratedMessage {
  factory GetImportJobResponse({
    ImportJob? importJob,
  }) {
    final result = create();
    if (importJob != null) result.importJob = importJob;
    return result;
  }

  GetImportJobResponse._();

  factory GetImportJobResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory GetImportJobResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'GetImportJobResponse',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.mastermaintenance.v1'),
      createEmptyInstance: create)
    ..aOM<ImportJob>(1, _omitFieldNames ? '' : 'importJob',
        subBuilder: ImportJob.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetImportJobResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetImportJobResponse copyWith(void Function(GetImportJobResponse) updates) =>
      super.copyWith((message) => updates(message as GetImportJobResponse))
          as GetImportJobResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static GetImportJobResponse create() => GetImportJobResponse._();
  @$core.override
  GetImportJobResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static GetImportJobResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<GetImportJobResponse>(create);
  static GetImportJobResponse? _defaultInstance;

  @$pb.TagNumber(1)
  ImportJob get importJob => $_getN(0);
  @$pb.TagNumber(1)
  set importJob(ImportJob value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasImportJob() => $_has(0);
  @$pb.TagNumber(1)
  void clearImportJob() => $_clearField(1);
  @$pb.TagNumber(1)
  ImportJob ensureImportJob() => $_ensure(0);
}

class ImportJob extends $pb.GeneratedMessage {
  factory ImportJob({
    $core.String? id,
    $core.String? tableId,
    $core.String? fileName,
    $core.String? status,
    $core.int? totalRows,
    $core.int? processedRows,
    $core.int? errorRows,
    $core.String? errorDetailsJson,
    $core.String? startedBy,
    $core.String? startedAt,
    $core.String? completedAt,
  }) {
    final result = create();
    if (id != null) result.id = id;
    if (tableId != null) result.tableId = tableId;
    if (fileName != null) result.fileName = fileName;
    if (status != null) result.status = status;
    if (totalRows != null) result.totalRows = totalRows;
    if (processedRows != null) result.processedRows = processedRows;
    if (errorRows != null) result.errorRows = errorRows;
    if (errorDetailsJson != null) result.errorDetailsJson = errorDetailsJson;
    if (startedBy != null) result.startedBy = startedBy;
    if (startedAt != null) result.startedAt = startedAt;
    if (completedAt != null) result.completedAt = completedAt;
    return result;
  }

  ImportJob._();

  factory ImportJob.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory ImportJob.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ImportJob',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.mastermaintenance.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'id')
    ..aOS(2, _omitFieldNames ? '' : 'tableId')
    ..aOS(3, _omitFieldNames ? '' : 'fileName')
    ..aOS(4, _omitFieldNames ? '' : 'status')
    ..aI(5, _omitFieldNames ? '' : 'totalRows')
    ..aI(6, _omitFieldNames ? '' : 'processedRows')
    ..aI(7, _omitFieldNames ? '' : 'errorRows')
    ..aOS(8, _omitFieldNames ? '' : 'errorDetailsJson')
    ..aOS(9, _omitFieldNames ? '' : 'startedBy')
    ..aOS(10, _omitFieldNames ? '' : 'startedAt')
    ..aOS(11, _omitFieldNames ? '' : 'completedAt')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ImportJob clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ImportJob copyWith(void Function(ImportJob) updates) =>
      super.copyWith((message) => updates(message as ImportJob)) as ImportJob;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static ImportJob create() => ImportJob._();
  @$core.override
  ImportJob createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static ImportJob getDefault() =>
      _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<ImportJob>(create);
  static ImportJob? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get id => $_getSZ(0);
  @$pb.TagNumber(1)
  set id($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasId() => $_has(0);
  @$pb.TagNumber(1)
  void clearId() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get tableId => $_getSZ(1);
  @$pb.TagNumber(2)
  set tableId($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasTableId() => $_has(1);
  @$pb.TagNumber(2)
  void clearTableId() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.String get fileName => $_getSZ(2);
  @$pb.TagNumber(3)
  set fileName($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasFileName() => $_has(2);
  @$pb.TagNumber(3)
  void clearFileName() => $_clearField(3);

  @$pb.TagNumber(4)
  $core.String get status => $_getSZ(3);
  @$pb.TagNumber(4)
  set status($core.String value) => $_setString(3, value);
  @$pb.TagNumber(4)
  $core.bool hasStatus() => $_has(3);
  @$pb.TagNumber(4)
  void clearStatus() => $_clearField(4);

  @$pb.TagNumber(5)
  $core.int get totalRows => $_getIZ(4);
  @$pb.TagNumber(5)
  set totalRows($core.int value) => $_setSignedInt32(4, value);
  @$pb.TagNumber(5)
  $core.bool hasTotalRows() => $_has(4);
  @$pb.TagNumber(5)
  void clearTotalRows() => $_clearField(5);

  @$pb.TagNumber(6)
  $core.int get processedRows => $_getIZ(5);
  @$pb.TagNumber(6)
  set processedRows($core.int value) => $_setSignedInt32(5, value);
  @$pb.TagNumber(6)
  $core.bool hasProcessedRows() => $_has(5);
  @$pb.TagNumber(6)
  void clearProcessedRows() => $_clearField(6);

  @$pb.TagNumber(7)
  $core.int get errorRows => $_getIZ(6);
  @$pb.TagNumber(7)
  set errorRows($core.int value) => $_setSignedInt32(6, value);
  @$pb.TagNumber(7)
  $core.bool hasErrorRows() => $_has(6);
  @$pb.TagNumber(7)
  void clearErrorRows() => $_clearField(7);

  @$pb.TagNumber(8)
  $core.String get errorDetailsJson => $_getSZ(7);
  @$pb.TagNumber(8)
  set errorDetailsJson($core.String value) => $_setString(7, value);
  @$pb.TagNumber(8)
  $core.bool hasErrorDetailsJson() => $_has(7);
  @$pb.TagNumber(8)
  void clearErrorDetailsJson() => $_clearField(8);

  @$pb.TagNumber(9)
  $core.String get startedBy => $_getSZ(8);
  @$pb.TagNumber(9)
  set startedBy($core.String value) => $_setString(8, value);
  @$pb.TagNumber(9)
  $core.bool hasStartedBy() => $_has(8);
  @$pb.TagNumber(9)
  void clearStartedBy() => $_clearField(9);

  @$pb.TagNumber(10)
  $core.String get startedAt => $_getSZ(9);
  @$pb.TagNumber(10)
  set startedAt($core.String value) => $_setString(9, value);
  @$pb.TagNumber(10)
  $core.bool hasStartedAt() => $_has(9);
  @$pb.TagNumber(10)
  void clearStartedAt() => $_clearField(10);

  @$pb.TagNumber(11)
  $core.String get completedAt => $_getSZ(10);
  @$pb.TagNumber(11)
  set completedAt($core.String value) => $_setString(10, value);
  @$pb.TagNumber(11)
  $core.bool hasCompletedAt() => $_has(10);
  @$pb.TagNumber(11)
  void clearCompletedAt() => $_clearField(11);
}

class ListDisplayConfigsRequest extends $pb.GeneratedMessage {
  factory ListDisplayConfigsRequest({
    $core.String? tableName,
  }) {
    final result = create();
    if (tableName != null) result.tableName = tableName;
    return result;
  }

  ListDisplayConfigsRequest._();

  factory ListDisplayConfigsRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory ListDisplayConfigsRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ListDisplayConfigsRequest',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.mastermaintenance.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'tableName')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListDisplayConfigsRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListDisplayConfigsRequest copyWith(
          void Function(ListDisplayConfigsRequest) updates) =>
      super.copyWith((message) => updates(message as ListDisplayConfigsRequest))
          as ListDisplayConfigsRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static ListDisplayConfigsRequest create() => ListDisplayConfigsRequest._();
  @$core.override
  ListDisplayConfigsRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static ListDisplayConfigsRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ListDisplayConfigsRequest>(create);
  static ListDisplayConfigsRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get tableName => $_getSZ(0);
  @$pb.TagNumber(1)
  set tableName($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasTableName() => $_has(0);
  @$pb.TagNumber(1)
  void clearTableName() => $_clearField(1);
}

class ListDisplayConfigsResponse extends $pb.GeneratedMessage {
  factory ListDisplayConfigsResponse({
    $core.Iterable<DisplayConfig>? displayConfigs,
  }) {
    final result = create();
    if (displayConfigs != null) result.displayConfigs.addAll(displayConfigs);
    return result;
  }

  ListDisplayConfigsResponse._();

  factory ListDisplayConfigsResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory ListDisplayConfigsResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ListDisplayConfigsResponse',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.mastermaintenance.v1'),
      createEmptyInstance: create)
    ..pPM<DisplayConfig>(1, _omitFieldNames ? '' : 'displayConfigs',
        subBuilder: DisplayConfig.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListDisplayConfigsResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListDisplayConfigsResponse copyWith(
          void Function(ListDisplayConfigsResponse) updates) =>
      super.copyWith(
              (message) => updates(message as ListDisplayConfigsResponse))
          as ListDisplayConfigsResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static ListDisplayConfigsResponse create() => ListDisplayConfigsResponse._();
  @$core.override
  ListDisplayConfigsResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static ListDisplayConfigsResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ListDisplayConfigsResponse>(create);
  static ListDisplayConfigsResponse? _defaultInstance;

  @$pb.TagNumber(1)
  $pb.PbList<DisplayConfig> get displayConfigs => $_getList(0);
}

class GetDisplayConfigRequest extends $pb.GeneratedMessage {
  factory GetDisplayConfigRequest({
    $core.String? tableName,
    $core.String? displayConfigId,
  }) {
    final result = create();
    if (tableName != null) result.tableName = tableName;
    if (displayConfigId != null) result.displayConfigId = displayConfigId;
    return result;
  }

  GetDisplayConfigRequest._();

  factory GetDisplayConfigRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory GetDisplayConfigRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'GetDisplayConfigRequest',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.mastermaintenance.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'tableName')
    ..aOS(2, _omitFieldNames ? '' : 'displayConfigId')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetDisplayConfigRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetDisplayConfigRequest copyWith(
          void Function(GetDisplayConfigRequest) updates) =>
      super.copyWith((message) => updates(message as GetDisplayConfigRequest))
          as GetDisplayConfigRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static GetDisplayConfigRequest create() => GetDisplayConfigRequest._();
  @$core.override
  GetDisplayConfigRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static GetDisplayConfigRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<GetDisplayConfigRequest>(create);
  static GetDisplayConfigRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get tableName => $_getSZ(0);
  @$pb.TagNumber(1)
  set tableName($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasTableName() => $_has(0);
  @$pb.TagNumber(1)
  void clearTableName() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get displayConfigId => $_getSZ(1);
  @$pb.TagNumber(2)
  set displayConfigId($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasDisplayConfigId() => $_has(1);
  @$pb.TagNumber(2)
  void clearDisplayConfigId() => $_clearField(2);
}

class GetDisplayConfigResponse extends $pb.GeneratedMessage {
  factory GetDisplayConfigResponse({
    DisplayConfig? displayConfig,
  }) {
    final result = create();
    if (displayConfig != null) result.displayConfig = displayConfig;
    return result;
  }

  GetDisplayConfigResponse._();

  factory GetDisplayConfigResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory GetDisplayConfigResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'GetDisplayConfigResponse',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.mastermaintenance.v1'),
      createEmptyInstance: create)
    ..aOM<DisplayConfig>(1, _omitFieldNames ? '' : 'displayConfig',
        subBuilder: DisplayConfig.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetDisplayConfigResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetDisplayConfigResponse copyWith(
          void Function(GetDisplayConfigResponse) updates) =>
      super.copyWith((message) => updates(message as GetDisplayConfigResponse))
          as GetDisplayConfigResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static GetDisplayConfigResponse create() => GetDisplayConfigResponse._();
  @$core.override
  GetDisplayConfigResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static GetDisplayConfigResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<GetDisplayConfigResponse>(create);
  static GetDisplayConfigResponse? _defaultInstance;

  @$pb.TagNumber(1)
  DisplayConfig get displayConfig => $_getN(0);
  @$pb.TagNumber(1)
  set displayConfig(DisplayConfig value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasDisplayConfig() => $_has(0);
  @$pb.TagNumber(1)
  void clearDisplayConfig() => $_clearField(1);
  @$pb.TagNumber(1)
  DisplayConfig ensureDisplayConfig() => $_ensure(0);
}

class CreateDisplayConfigRequest extends $pb.GeneratedMessage {
  factory CreateDisplayConfigRequest({
    $core.String? tableName,
    $1.Struct? data,
  }) {
    final result = create();
    if (tableName != null) result.tableName = tableName;
    if (data != null) result.data = data;
    return result;
  }

  CreateDisplayConfigRequest._();

  factory CreateDisplayConfigRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory CreateDisplayConfigRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'CreateDisplayConfigRequest',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.mastermaintenance.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'tableName')
    ..aOM<$1.Struct>(2, _omitFieldNames ? '' : 'data',
        subBuilder: $1.Struct.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  CreateDisplayConfigRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  CreateDisplayConfigRequest copyWith(
          void Function(CreateDisplayConfigRequest) updates) =>
      super.copyWith(
              (message) => updates(message as CreateDisplayConfigRequest))
          as CreateDisplayConfigRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static CreateDisplayConfigRequest create() => CreateDisplayConfigRequest._();
  @$core.override
  CreateDisplayConfigRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static CreateDisplayConfigRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<CreateDisplayConfigRequest>(create);
  static CreateDisplayConfigRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get tableName => $_getSZ(0);
  @$pb.TagNumber(1)
  set tableName($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasTableName() => $_has(0);
  @$pb.TagNumber(1)
  void clearTableName() => $_clearField(1);

  @$pb.TagNumber(2)
  $1.Struct get data => $_getN(1);
  @$pb.TagNumber(2)
  set data($1.Struct value) => $_setField(2, value);
  @$pb.TagNumber(2)
  $core.bool hasData() => $_has(1);
  @$pb.TagNumber(2)
  void clearData() => $_clearField(2);
  @$pb.TagNumber(2)
  $1.Struct ensureData() => $_ensure(1);
}

class CreateDisplayConfigResponse extends $pb.GeneratedMessage {
  factory CreateDisplayConfigResponse({
    DisplayConfig? displayConfig,
  }) {
    final result = create();
    if (displayConfig != null) result.displayConfig = displayConfig;
    return result;
  }

  CreateDisplayConfigResponse._();

  factory CreateDisplayConfigResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory CreateDisplayConfigResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'CreateDisplayConfigResponse',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.mastermaintenance.v1'),
      createEmptyInstance: create)
    ..aOM<DisplayConfig>(1, _omitFieldNames ? '' : 'displayConfig',
        subBuilder: DisplayConfig.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  CreateDisplayConfigResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  CreateDisplayConfigResponse copyWith(
          void Function(CreateDisplayConfigResponse) updates) =>
      super.copyWith(
              (message) => updates(message as CreateDisplayConfigResponse))
          as CreateDisplayConfigResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static CreateDisplayConfigResponse create() =>
      CreateDisplayConfigResponse._();
  @$core.override
  CreateDisplayConfigResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static CreateDisplayConfigResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<CreateDisplayConfigResponse>(create);
  static CreateDisplayConfigResponse? _defaultInstance;

  @$pb.TagNumber(1)
  DisplayConfig get displayConfig => $_getN(0);
  @$pb.TagNumber(1)
  set displayConfig(DisplayConfig value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasDisplayConfig() => $_has(0);
  @$pb.TagNumber(1)
  void clearDisplayConfig() => $_clearField(1);
  @$pb.TagNumber(1)
  DisplayConfig ensureDisplayConfig() => $_ensure(0);
}

class UpdateDisplayConfigRequest extends $pb.GeneratedMessage {
  factory UpdateDisplayConfigRequest({
    $core.String? tableName,
    $core.String? displayConfigId,
    $1.Struct? data,
  }) {
    final result = create();
    if (tableName != null) result.tableName = tableName;
    if (displayConfigId != null) result.displayConfigId = displayConfigId;
    if (data != null) result.data = data;
    return result;
  }

  UpdateDisplayConfigRequest._();

  factory UpdateDisplayConfigRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory UpdateDisplayConfigRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'UpdateDisplayConfigRequest',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.mastermaintenance.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'tableName')
    ..aOS(2, _omitFieldNames ? '' : 'displayConfigId')
    ..aOM<$1.Struct>(3, _omitFieldNames ? '' : 'data',
        subBuilder: $1.Struct.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  UpdateDisplayConfigRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  UpdateDisplayConfigRequest copyWith(
          void Function(UpdateDisplayConfigRequest) updates) =>
      super.copyWith(
              (message) => updates(message as UpdateDisplayConfigRequest))
          as UpdateDisplayConfigRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static UpdateDisplayConfigRequest create() => UpdateDisplayConfigRequest._();
  @$core.override
  UpdateDisplayConfigRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static UpdateDisplayConfigRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<UpdateDisplayConfigRequest>(create);
  static UpdateDisplayConfigRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get tableName => $_getSZ(0);
  @$pb.TagNumber(1)
  set tableName($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasTableName() => $_has(0);
  @$pb.TagNumber(1)
  void clearTableName() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get displayConfigId => $_getSZ(1);
  @$pb.TagNumber(2)
  set displayConfigId($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasDisplayConfigId() => $_has(1);
  @$pb.TagNumber(2)
  void clearDisplayConfigId() => $_clearField(2);

  @$pb.TagNumber(3)
  $1.Struct get data => $_getN(2);
  @$pb.TagNumber(3)
  set data($1.Struct value) => $_setField(3, value);
  @$pb.TagNumber(3)
  $core.bool hasData() => $_has(2);
  @$pb.TagNumber(3)
  void clearData() => $_clearField(3);
  @$pb.TagNumber(3)
  $1.Struct ensureData() => $_ensure(2);
}

class UpdateDisplayConfigResponse extends $pb.GeneratedMessage {
  factory UpdateDisplayConfigResponse({
    DisplayConfig? displayConfig,
  }) {
    final result = create();
    if (displayConfig != null) result.displayConfig = displayConfig;
    return result;
  }

  UpdateDisplayConfigResponse._();

  factory UpdateDisplayConfigResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory UpdateDisplayConfigResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'UpdateDisplayConfigResponse',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.mastermaintenance.v1'),
      createEmptyInstance: create)
    ..aOM<DisplayConfig>(1, _omitFieldNames ? '' : 'displayConfig',
        subBuilder: DisplayConfig.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  UpdateDisplayConfigResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  UpdateDisplayConfigResponse copyWith(
          void Function(UpdateDisplayConfigResponse) updates) =>
      super.copyWith(
              (message) => updates(message as UpdateDisplayConfigResponse))
          as UpdateDisplayConfigResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static UpdateDisplayConfigResponse create() =>
      UpdateDisplayConfigResponse._();
  @$core.override
  UpdateDisplayConfigResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static UpdateDisplayConfigResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<UpdateDisplayConfigResponse>(create);
  static UpdateDisplayConfigResponse? _defaultInstance;

  @$pb.TagNumber(1)
  DisplayConfig get displayConfig => $_getN(0);
  @$pb.TagNumber(1)
  set displayConfig(DisplayConfig value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasDisplayConfig() => $_has(0);
  @$pb.TagNumber(1)
  void clearDisplayConfig() => $_clearField(1);
  @$pb.TagNumber(1)
  DisplayConfig ensureDisplayConfig() => $_ensure(0);
}

class DeleteDisplayConfigRequest extends $pb.GeneratedMessage {
  factory DeleteDisplayConfigRequest({
    $core.String? tableName,
    $core.String? displayConfigId,
  }) {
    final result = create();
    if (tableName != null) result.tableName = tableName;
    if (displayConfigId != null) result.displayConfigId = displayConfigId;
    return result;
  }

  DeleteDisplayConfigRequest._();

  factory DeleteDisplayConfigRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory DeleteDisplayConfigRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'DeleteDisplayConfigRequest',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.mastermaintenance.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'tableName')
    ..aOS(2, _omitFieldNames ? '' : 'displayConfigId')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  DeleteDisplayConfigRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  DeleteDisplayConfigRequest copyWith(
          void Function(DeleteDisplayConfigRequest) updates) =>
      super.copyWith(
              (message) => updates(message as DeleteDisplayConfigRequest))
          as DeleteDisplayConfigRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static DeleteDisplayConfigRequest create() => DeleteDisplayConfigRequest._();
  @$core.override
  DeleteDisplayConfigRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static DeleteDisplayConfigRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<DeleteDisplayConfigRequest>(create);
  static DeleteDisplayConfigRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get tableName => $_getSZ(0);
  @$pb.TagNumber(1)
  set tableName($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasTableName() => $_has(0);
  @$pb.TagNumber(1)
  void clearTableName() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get displayConfigId => $_getSZ(1);
  @$pb.TagNumber(2)
  set displayConfigId($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasDisplayConfigId() => $_has(1);
  @$pb.TagNumber(2)
  void clearDisplayConfigId() => $_clearField(2);
}

class DeleteDisplayConfigResponse extends $pb.GeneratedMessage {
  factory DeleteDisplayConfigResponse({
    $core.bool? success,
  }) {
    final result = create();
    if (success != null) result.success = success;
    return result;
  }

  DeleteDisplayConfigResponse._();

  factory DeleteDisplayConfigResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory DeleteDisplayConfigResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'DeleteDisplayConfigResponse',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.mastermaintenance.v1'),
      createEmptyInstance: create)
    ..aOB(1, _omitFieldNames ? '' : 'success')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  DeleteDisplayConfigResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  DeleteDisplayConfigResponse copyWith(
          void Function(DeleteDisplayConfigResponse) updates) =>
      super.copyWith(
              (message) => updates(message as DeleteDisplayConfigResponse))
          as DeleteDisplayConfigResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static DeleteDisplayConfigResponse create() =>
      DeleteDisplayConfigResponse._();
  @$core.override
  DeleteDisplayConfigResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static DeleteDisplayConfigResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<DeleteDisplayConfigResponse>(create);
  static DeleteDisplayConfigResponse? _defaultInstance;

  @$pb.TagNumber(1)
  $core.bool get success => $_getBF(0);
  @$pb.TagNumber(1)
  set success($core.bool value) => $_setBool(0, value);
  @$pb.TagNumber(1)
  $core.bool hasSuccess() => $_has(0);
  @$pb.TagNumber(1)
  void clearSuccess() => $_clearField(1);
}

class DisplayConfig extends $pb.GeneratedMessage {
  factory DisplayConfig({
    $core.String? id,
    $core.String? tableId,
    $core.String? configType,
    $core.String? configJson,
    $core.bool? isDefault,
    $core.String? createdBy,
    $core.String? createdAt,
    $core.String? updatedAt,
  }) {
    final result = create();
    if (id != null) result.id = id;
    if (tableId != null) result.tableId = tableId;
    if (configType != null) result.configType = configType;
    if (configJson != null) result.configJson = configJson;
    if (isDefault != null) result.isDefault = isDefault;
    if (createdBy != null) result.createdBy = createdBy;
    if (createdAt != null) result.createdAt = createdAt;
    if (updatedAt != null) result.updatedAt = updatedAt;
    return result;
  }

  DisplayConfig._();

  factory DisplayConfig.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory DisplayConfig.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'DisplayConfig',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.mastermaintenance.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'id')
    ..aOS(2, _omitFieldNames ? '' : 'tableId')
    ..aOS(3, _omitFieldNames ? '' : 'configType')
    ..aOS(4, _omitFieldNames ? '' : 'configJson')
    ..aOB(5, _omitFieldNames ? '' : 'isDefault')
    ..aOS(6, _omitFieldNames ? '' : 'createdBy')
    ..aOS(7, _omitFieldNames ? '' : 'createdAt')
    ..aOS(8, _omitFieldNames ? '' : 'updatedAt')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  DisplayConfig clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  DisplayConfig copyWith(void Function(DisplayConfig) updates) =>
      super.copyWith((message) => updates(message as DisplayConfig))
          as DisplayConfig;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static DisplayConfig create() => DisplayConfig._();
  @$core.override
  DisplayConfig createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static DisplayConfig getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<DisplayConfig>(create);
  static DisplayConfig? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get id => $_getSZ(0);
  @$pb.TagNumber(1)
  set id($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasId() => $_has(0);
  @$pb.TagNumber(1)
  void clearId() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get tableId => $_getSZ(1);
  @$pb.TagNumber(2)
  set tableId($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasTableId() => $_has(1);
  @$pb.TagNumber(2)
  void clearTableId() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.String get configType => $_getSZ(2);
  @$pb.TagNumber(3)
  set configType($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasConfigType() => $_has(2);
  @$pb.TagNumber(3)
  void clearConfigType() => $_clearField(3);

  @$pb.TagNumber(4)
  $core.String get configJson => $_getSZ(3);
  @$pb.TagNumber(4)
  set configJson($core.String value) => $_setString(3, value);
  @$pb.TagNumber(4)
  $core.bool hasConfigJson() => $_has(3);
  @$pb.TagNumber(4)
  void clearConfigJson() => $_clearField(4);

  @$pb.TagNumber(5)
  $core.bool get isDefault => $_getBF(4);
  @$pb.TagNumber(5)
  set isDefault($core.bool value) => $_setBool(4, value);
  @$pb.TagNumber(5)
  $core.bool hasIsDefault() => $_has(4);
  @$pb.TagNumber(5)
  void clearIsDefault() => $_clearField(5);

  @$pb.TagNumber(6)
  $core.String get createdBy => $_getSZ(5);
  @$pb.TagNumber(6)
  set createdBy($core.String value) => $_setString(5, value);
  @$pb.TagNumber(6)
  $core.bool hasCreatedBy() => $_has(5);
  @$pb.TagNumber(6)
  void clearCreatedBy() => $_clearField(6);

  @$pb.TagNumber(7)
  $core.String get createdAt => $_getSZ(6);
  @$pb.TagNumber(7)
  set createdAt($core.String value) => $_setString(6, value);
  @$pb.TagNumber(7)
  $core.bool hasCreatedAt() => $_has(6);
  @$pb.TagNumber(7)
  void clearCreatedAt() => $_clearField(7);

  @$pb.TagNumber(8)
  $core.String get updatedAt => $_getSZ(7);
  @$pb.TagNumber(8)
  set updatedAt($core.String value) => $_setString(7, value);
  @$pb.TagNumber(8)
  $core.bool hasUpdatedAt() => $_has(7);
  @$pb.TagNumber(8)
  void clearUpdatedAt() => $_clearField(8);
}

class ListTableAuditLogsRequest extends $pb.GeneratedMessage {
  factory ListTableAuditLogsRequest({
    $core.String? tableName,
    $2.Pagination? pagination,
    $core.String? domainScope,
  }) {
    final result = create();
    if (tableName != null) result.tableName = tableName;
    if (pagination != null) result.pagination = pagination;
    if (domainScope != null) result.domainScope = domainScope;
    return result;
  }

  ListTableAuditLogsRequest._();

  factory ListTableAuditLogsRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory ListTableAuditLogsRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ListTableAuditLogsRequest',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.mastermaintenance.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'tableName')
    ..aOM<$2.Pagination>(2, _omitFieldNames ? '' : 'pagination',
        subBuilder: $2.Pagination.create)
    ..aOS(3, _omitFieldNames ? '' : 'domainScope')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListTableAuditLogsRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListTableAuditLogsRequest copyWith(
          void Function(ListTableAuditLogsRequest) updates) =>
      super.copyWith((message) => updates(message as ListTableAuditLogsRequest))
          as ListTableAuditLogsRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static ListTableAuditLogsRequest create() => ListTableAuditLogsRequest._();
  @$core.override
  ListTableAuditLogsRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static ListTableAuditLogsRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ListTableAuditLogsRequest>(create);
  static ListTableAuditLogsRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get tableName => $_getSZ(0);
  @$pb.TagNumber(1)
  set tableName($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasTableName() => $_has(0);
  @$pb.TagNumber(1)
  void clearTableName() => $_clearField(1);

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

  @$pb.TagNumber(3)
  $core.String get domainScope => $_getSZ(2);
  @$pb.TagNumber(3)
  set domainScope($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasDomainScope() => $_has(2);
  @$pb.TagNumber(3)
  void clearDomainScope() => $_clearField(3);
}

class ListTableAuditLogsResponse extends $pb.GeneratedMessage {
  factory ListTableAuditLogsResponse({
    $core.Iterable<AuditLogEntry>? logs,
    $2.PaginationResult? pagination,
  }) {
    final result = create();
    if (logs != null) result.logs.addAll(logs);
    if (pagination != null) result.pagination = pagination;
    return result;
  }

  ListTableAuditLogsResponse._();

  factory ListTableAuditLogsResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory ListTableAuditLogsResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ListTableAuditLogsResponse',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.mastermaintenance.v1'),
      createEmptyInstance: create)
    ..pPM<AuditLogEntry>(1, _omitFieldNames ? '' : 'logs',
        subBuilder: AuditLogEntry.create)
    ..aOM<$2.PaginationResult>(2, _omitFieldNames ? '' : 'pagination',
        subBuilder: $2.PaginationResult.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListTableAuditLogsResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListTableAuditLogsResponse copyWith(
          void Function(ListTableAuditLogsResponse) updates) =>
      super.copyWith(
              (message) => updates(message as ListTableAuditLogsResponse))
          as ListTableAuditLogsResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static ListTableAuditLogsResponse create() => ListTableAuditLogsResponse._();
  @$core.override
  ListTableAuditLogsResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static ListTableAuditLogsResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ListTableAuditLogsResponse>(create);
  static ListTableAuditLogsResponse? _defaultInstance;

  @$pb.TagNumber(1)
  $pb.PbList<AuditLogEntry> get logs => $_getList(0);

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

class ListRecordAuditLogsRequest extends $pb.GeneratedMessage {
  factory ListRecordAuditLogsRequest({
    $core.String? tableName,
    $core.String? recordId,
    $2.Pagination? pagination,
    $core.String? domainScope,
  }) {
    final result = create();
    if (tableName != null) result.tableName = tableName;
    if (recordId != null) result.recordId = recordId;
    if (pagination != null) result.pagination = pagination;
    if (domainScope != null) result.domainScope = domainScope;
    return result;
  }

  ListRecordAuditLogsRequest._();

  factory ListRecordAuditLogsRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory ListRecordAuditLogsRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ListRecordAuditLogsRequest',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.mastermaintenance.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'tableName')
    ..aOS(2, _omitFieldNames ? '' : 'recordId')
    ..aOM<$2.Pagination>(3, _omitFieldNames ? '' : 'pagination',
        subBuilder: $2.Pagination.create)
    ..aOS(4, _omitFieldNames ? '' : 'domainScope')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListRecordAuditLogsRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListRecordAuditLogsRequest copyWith(
          void Function(ListRecordAuditLogsRequest) updates) =>
      super.copyWith(
              (message) => updates(message as ListRecordAuditLogsRequest))
          as ListRecordAuditLogsRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static ListRecordAuditLogsRequest create() => ListRecordAuditLogsRequest._();
  @$core.override
  ListRecordAuditLogsRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static ListRecordAuditLogsRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ListRecordAuditLogsRequest>(create);
  static ListRecordAuditLogsRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get tableName => $_getSZ(0);
  @$pb.TagNumber(1)
  set tableName($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasTableName() => $_has(0);
  @$pb.TagNumber(1)
  void clearTableName() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get recordId => $_getSZ(1);
  @$pb.TagNumber(2)
  set recordId($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasRecordId() => $_has(1);
  @$pb.TagNumber(2)
  void clearRecordId() => $_clearField(2);

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

  @$pb.TagNumber(4)
  $core.String get domainScope => $_getSZ(3);
  @$pb.TagNumber(4)
  set domainScope($core.String value) => $_setString(3, value);
  @$pb.TagNumber(4)
  $core.bool hasDomainScope() => $_has(3);
  @$pb.TagNumber(4)
  void clearDomainScope() => $_clearField(4);
}

class ListRecordAuditLogsResponse extends $pb.GeneratedMessage {
  factory ListRecordAuditLogsResponse({
    $core.Iterable<AuditLogEntry>? logs,
    $2.PaginationResult? pagination,
  }) {
    final result = create();
    if (logs != null) result.logs.addAll(logs);
    if (pagination != null) result.pagination = pagination;
    return result;
  }

  ListRecordAuditLogsResponse._();

  factory ListRecordAuditLogsResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory ListRecordAuditLogsResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ListRecordAuditLogsResponse',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.mastermaintenance.v1'),
      createEmptyInstance: create)
    ..pPM<AuditLogEntry>(1, _omitFieldNames ? '' : 'logs',
        subBuilder: AuditLogEntry.create)
    ..aOM<$2.PaginationResult>(2, _omitFieldNames ? '' : 'pagination',
        subBuilder: $2.PaginationResult.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListRecordAuditLogsResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListRecordAuditLogsResponse copyWith(
          void Function(ListRecordAuditLogsResponse) updates) =>
      super.copyWith(
              (message) => updates(message as ListRecordAuditLogsResponse))
          as ListRecordAuditLogsResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static ListRecordAuditLogsResponse create() =>
      ListRecordAuditLogsResponse._();
  @$core.override
  ListRecordAuditLogsResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static ListRecordAuditLogsResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ListRecordAuditLogsResponse>(create);
  static ListRecordAuditLogsResponse? _defaultInstance;

  @$pb.TagNumber(1)
  $pb.PbList<AuditLogEntry> get logs => $_getList(0);

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

class AuditLogEntry extends $pb.GeneratedMessage {
  factory AuditLogEntry({
    $core.String? id,
    $core.String? targetTable,
    $core.String? targetRecordId,
    $core.String? operation,
    $core.String? beforeDataJson,
    $core.String? afterDataJson,
    $core.Iterable<$core.String>? changedColumns,
    $core.String? changedBy,
    $core.String? changeReason,
    $core.String? traceId,
    $core.String? createdAt,
    $core.String? domainScope,
  }) {
    final result = create();
    if (id != null) result.id = id;
    if (targetTable != null) result.targetTable = targetTable;
    if (targetRecordId != null) result.targetRecordId = targetRecordId;
    if (operation != null) result.operation = operation;
    if (beforeDataJson != null) result.beforeDataJson = beforeDataJson;
    if (afterDataJson != null) result.afterDataJson = afterDataJson;
    if (changedColumns != null) result.changedColumns.addAll(changedColumns);
    if (changedBy != null) result.changedBy = changedBy;
    if (changeReason != null) result.changeReason = changeReason;
    if (traceId != null) result.traceId = traceId;
    if (createdAt != null) result.createdAt = createdAt;
    if (domainScope != null) result.domainScope = domainScope;
    return result;
  }

  AuditLogEntry._();

  factory AuditLogEntry.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory AuditLogEntry.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'AuditLogEntry',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.mastermaintenance.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'id')
    ..aOS(2, _omitFieldNames ? '' : 'targetTable')
    ..aOS(3, _omitFieldNames ? '' : 'targetRecordId')
    ..aOS(4, _omitFieldNames ? '' : 'operation')
    ..aOS(5, _omitFieldNames ? '' : 'beforeDataJson')
    ..aOS(6, _omitFieldNames ? '' : 'afterDataJson')
    ..pPS(7, _omitFieldNames ? '' : 'changedColumns')
    ..aOS(8, _omitFieldNames ? '' : 'changedBy')
    ..aOS(9, _omitFieldNames ? '' : 'changeReason')
    ..aOS(10, _omitFieldNames ? '' : 'traceId')
    ..aOS(11, _omitFieldNames ? '' : 'createdAt')
    ..aOS(12, _omitFieldNames ? '' : 'domainScope')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  AuditLogEntry clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  AuditLogEntry copyWith(void Function(AuditLogEntry) updates) =>
      super.copyWith((message) => updates(message as AuditLogEntry))
          as AuditLogEntry;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static AuditLogEntry create() => AuditLogEntry._();
  @$core.override
  AuditLogEntry createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static AuditLogEntry getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<AuditLogEntry>(create);
  static AuditLogEntry? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get id => $_getSZ(0);
  @$pb.TagNumber(1)
  set id($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasId() => $_has(0);
  @$pb.TagNumber(1)
  void clearId() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get targetTable => $_getSZ(1);
  @$pb.TagNumber(2)
  set targetTable($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasTargetTable() => $_has(1);
  @$pb.TagNumber(2)
  void clearTargetTable() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.String get targetRecordId => $_getSZ(2);
  @$pb.TagNumber(3)
  set targetRecordId($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasTargetRecordId() => $_has(2);
  @$pb.TagNumber(3)
  void clearTargetRecordId() => $_clearField(3);

  @$pb.TagNumber(4)
  $core.String get operation => $_getSZ(3);
  @$pb.TagNumber(4)
  set operation($core.String value) => $_setString(3, value);
  @$pb.TagNumber(4)
  $core.bool hasOperation() => $_has(3);
  @$pb.TagNumber(4)
  void clearOperation() => $_clearField(4);

  @$pb.TagNumber(5)
  $core.String get beforeDataJson => $_getSZ(4);
  @$pb.TagNumber(5)
  set beforeDataJson($core.String value) => $_setString(4, value);
  @$pb.TagNumber(5)
  $core.bool hasBeforeDataJson() => $_has(4);
  @$pb.TagNumber(5)
  void clearBeforeDataJson() => $_clearField(5);

  @$pb.TagNumber(6)
  $core.String get afterDataJson => $_getSZ(5);
  @$pb.TagNumber(6)
  set afterDataJson($core.String value) => $_setString(5, value);
  @$pb.TagNumber(6)
  $core.bool hasAfterDataJson() => $_has(5);
  @$pb.TagNumber(6)
  void clearAfterDataJson() => $_clearField(6);

  @$pb.TagNumber(7)
  $pb.PbList<$core.String> get changedColumns => $_getList(6);

  @$pb.TagNumber(8)
  $core.String get changedBy => $_getSZ(7);
  @$pb.TagNumber(8)
  set changedBy($core.String value) => $_setString(7, value);
  @$pb.TagNumber(8)
  $core.bool hasChangedBy() => $_has(7);
  @$pb.TagNumber(8)
  void clearChangedBy() => $_clearField(8);

  @$pb.TagNumber(9)
  $core.String get changeReason => $_getSZ(8);
  @$pb.TagNumber(9)
  set changeReason($core.String value) => $_setString(8, value);
  @$pb.TagNumber(9)
  $core.bool hasChangeReason() => $_has(8);
  @$pb.TagNumber(9)
  void clearChangeReason() => $_clearField(9);

  @$pb.TagNumber(10)
  $core.String get traceId => $_getSZ(9);
  @$pb.TagNumber(10)
  set traceId($core.String value) => $_setString(9, value);
  @$pb.TagNumber(10)
  $core.bool hasTraceId() => $_has(9);
  @$pb.TagNumber(10)
  void clearTraceId() => $_clearField(10);

  @$pb.TagNumber(11)
  $core.String get createdAt => $_getSZ(10);
  @$pb.TagNumber(11)
  set createdAt($core.String value) => $_setString(10, value);
  @$pb.TagNumber(11)
  $core.bool hasCreatedAt() => $_has(10);
  @$pb.TagNumber(11)
  void clearCreatedAt() => $_clearField(11);

  @$pb.TagNumber(12)
  $core.String get domainScope => $_getSZ(11);
  @$pb.TagNumber(12)
  set domainScope($core.String value) => $_setString(11, value);
  @$pb.TagNumber(12)
  $core.bool hasDomainScope() => $_has(11);
  @$pb.TagNumber(12)
  void clearDomainScope() => $_clearField(12);
}

/// ドメイン一覧リクエスト
class ListDomainsRequest extends $pb.GeneratedMessage {
  factory ListDomainsRequest() => create();

  ListDomainsRequest._();

  factory ListDomainsRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory ListDomainsRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ListDomainsRequest',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.mastermaintenance.v1'),
      createEmptyInstance: create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListDomainsRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListDomainsRequest copyWith(void Function(ListDomainsRequest) updates) =>
      super.copyWith((message) => updates(message as ListDomainsRequest))
          as ListDomainsRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static ListDomainsRequest create() => ListDomainsRequest._();
  @$core.override
  ListDomainsRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static ListDomainsRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ListDomainsRequest>(create);
  static ListDomainsRequest? _defaultInstance;
}

/// ドメイン一覧レスポンス
class ListDomainsResponse extends $pb.GeneratedMessage {
  factory ListDomainsResponse({
    $core.Iterable<DomainInfo>? domains,
  }) {
    final result = create();
    if (domains != null) result.domains.addAll(domains);
    return result;
  }

  ListDomainsResponse._();

  factory ListDomainsResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory ListDomainsResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ListDomainsResponse',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.mastermaintenance.v1'),
      createEmptyInstance: create)
    ..pPM<DomainInfo>(1, _omitFieldNames ? '' : 'domains',
        subBuilder: DomainInfo.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListDomainsResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListDomainsResponse copyWith(void Function(ListDomainsResponse) updates) =>
      super.copyWith((message) => updates(message as ListDomainsResponse))
          as ListDomainsResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static ListDomainsResponse create() => ListDomainsResponse._();
  @$core.override
  ListDomainsResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static ListDomainsResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ListDomainsResponse>(create);
  static ListDomainsResponse? _defaultInstance;

  @$pb.TagNumber(1)
  $pb.PbList<DomainInfo> get domains => $_getList(0);
}

/// ドメイン情報
class DomainInfo extends $pb.GeneratedMessage {
  factory DomainInfo({
    $core.String? domainScope,
    $core.int? tableCount,
  }) {
    final result = create();
    if (domainScope != null) result.domainScope = domainScope;
    if (tableCount != null) result.tableCount = tableCount;
    return result;
  }

  DomainInfo._();

  factory DomainInfo.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory DomainInfo.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'DomainInfo',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.mastermaintenance.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'domainScope')
    ..aI(2, _omitFieldNames ? '' : 'tableCount')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  DomainInfo clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  DomainInfo copyWith(void Function(DomainInfo) updates) =>
      super.copyWith((message) => updates(message as DomainInfo)) as DomainInfo;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static DomainInfo create() => DomainInfo._();
  @$core.override
  DomainInfo createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static DomainInfo getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<DomainInfo>(create);
  static DomainInfo? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get domainScope => $_getSZ(0);
  @$pb.TagNumber(1)
  set domainScope($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasDomainScope() => $_has(0);
  @$pb.TagNumber(1)
  void clearDomainScope() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.int get tableCount => $_getIZ(1);
  @$pb.TagNumber(2)
  set tableCount($core.int value) => $_setSignedInt32(1, value);
  @$pb.TagNumber(2)
  $core.bool hasTableCount() => $_has(1);
  @$pb.TagNumber(2)
  void clearTableCount() => $_clearField(2);
}

const $core.bool _omitFieldNames =
    $core.bool.fromEnvironment('protobuf.omit_field_names');
const $core.bool _omitMessageNames =
    $core.bool.fromEnvironment('protobuf.omit_message_names');
