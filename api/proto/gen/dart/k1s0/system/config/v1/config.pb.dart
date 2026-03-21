// This is a generated file - do not edit.
//
// Generated from k1s0/system/config/v1/config.proto.

// @dart = 3.3

// ignore_for_file: annotate_overrides, camel_case_types, comment_references
// ignore_for_file: constant_identifier_names
// ignore_for_file: curly_braces_in_flow_control_structures
// ignore_for_file: deprecated_member_use_from_same_package, library_prefixes
// ignore_for_file: non_constant_identifier_names, prefer_relative_imports

import 'dart:core' as $core;

import 'package:fixnum/fixnum.dart' as $fixnum;
import 'package:protobuf/protobuf.dart' as $pb;

import '../../common/v1/types.pb.dart' as $1;
import 'config.pbenum.dart';

export 'package:protobuf/protobuf.dart' show GeneratedMessageGenericExtensions;

export 'config.pbenum.dart';

/// ConfigEntry は設定値エントリ。
class ConfigEntry extends $pb.GeneratedMessage {
  factory ConfigEntry({
    $core.String? id,
    $core.String? namespace,
    $core.String? key,
    $core.List<$core.int>? value,
    $core.int? version,
    $core.String? description,
    $core.String? createdBy,
    $core.String? updatedBy,
    $1.Timestamp? createdAt,
    $1.Timestamp? updatedAt,
  }) {
    final result = create();
    if (id != null) result.id = id;
    if (namespace != null) result.namespace = namespace;
    if (key != null) result.key = key;
    if (value != null) result.value = value;
    if (version != null) result.version = version;
    if (description != null) result.description = description;
    if (createdBy != null) result.createdBy = createdBy;
    if (updatedBy != null) result.updatedBy = updatedBy;
    if (createdAt != null) result.createdAt = createdAt;
    if (updatedAt != null) result.updatedAt = updatedAt;
    return result;
  }

  ConfigEntry._();

  factory ConfigEntry.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory ConfigEntry.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ConfigEntry',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.config.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'id')
    ..aOS(2, _omitFieldNames ? '' : 'namespace')
    ..aOS(3, _omitFieldNames ? '' : 'key')
    ..a<$core.List<$core.int>>(
        4, _omitFieldNames ? '' : 'value', $pb.PbFieldType.OY)
    ..aI(5, _omitFieldNames ? '' : 'version')
    ..aOS(6, _omitFieldNames ? '' : 'description')
    ..aOS(7, _omitFieldNames ? '' : 'createdBy')
    ..aOS(8, _omitFieldNames ? '' : 'updatedBy')
    ..aOM<$1.Timestamp>(9, _omitFieldNames ? '' : 'createdAt',
        subBuilder: $1.Timestamp.create)
    ..aOM<$1.Timestamp>(10, _omitFieldNames ? '' : 'updatedAt',
        subBuilder: $1.Timestamp.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ConfigEntry clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ConfigEntry copyWith(void Function(ConfigEntry) updates) =>
      super.copyWith((message) => updates(message as ConfigEntry))
          as ConfigEntry;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static ConfigEntry create() => ConfigEntry._();
  @$core.override
  ConfigEntry createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static ConfigEntry getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ConfigEntry>(create);
  static ConfigEntry? _defaultInstance;

  /// UUID
  @$pb.TagNumber(1)
  $core.String get id => $_getSZ(0);
  @$pb.TagNumber(1)
  set id($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasId() => $_has(0);
  @$pb.TagNumber(1)
  void clearId() => $_clearField(1);

  /// 名前空間（例: "system.auth.database"）
  @$pb.TagNumber(2)
  $core.String get namespace => $_getSZ(1);
  @$pb.TagNumber(2)
  set namespace($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasNamespace() => $_has(1);
  @$pb.TagNumber(2)
  void clearNamespace() => $_clearField(2);

  /// キー名（例: "max_connections"）
  @$pb.TagNumber(3)
  $core.String get key => $_getSZ(2);
  @$pb.TagNumber(3)
  set key($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasKey() => $_has(2);
  @$pb.TagNumber(3)
  void clearKey() => $_clearField(3);

  /// JSON エンコード済みの値
  @$pb.TagNumber(4)
  $core.List<$core.int> get value => $_getN(3);
  @$pb.TagNumber(4)
  set value($core.List<$core.int> value) => $_setBytes(3, value);
  @$pb.TagNumber(4)
  $core.bool hasValue() => $_has(3);
  @$pb.TagNumber(4)
  void clearValue() => $_clearField(4);

  /// 楽観的排他制御用バージョン
  @$pb.TagNumber(5)
  $core.int get version => $_getIZ(4);
  @$pb.TagNumber(5)
  set version($core.int value) => $_setSignedInt32(4, value);
  @$pb.TagNumber(5)
  $core.bool hasVersion() => $_has(4);
  @$pb.TagNumber(5)
  void clearVersion() => $_clearField(5);

  @$pb.TagNumber(6)
  $core.String get description => $_getSZ(5);
  @$pb.TagNumber(6)
  set description($core.String value) => $_setString(5, value);
  @$pb.TagNumber(6)
  $core.bool hasDescription() => $_has(5);
  @$pb.TagNumber(6)
  void clearDescription() => $_clearField(6);

  @$pb.TagNumber(7)
  $core.String get createdBy => $_getSZ(6);
  @$pb.TagNumber(7)
  set createdBy($core.String value) => $_setString(6, value);
  @$pb.TagNumber(7)
  $core.bool hasCreatedBy() => $_has(6);
  @$pb.TagNumber(7)
  void clearCreatedBy() => $_clearField(7);

  @$pb.TagNumber(8)
  $core.String get updatedBy => $_getSZ(7);
  @$pb.TagNumber(8)
  set updatedBy($core.String value) => $_setString(7, value);
  @$pb.TagNumber(8)
  $core.bool hasUpdatedBy() => $_has(7);
  @$pb.TagNumber(8)
  void clearUpdatedBy() => $_clearField(8);

  @$pb.TagNumber(9)
  $1.Timestamp get createdAt => $_getN(8);
  @$pb.TagNumber(9)
  set createdAt($1.Timestamp value) => $_setField(9, value);
  @$pb.TagNumber(9)
  $core.bool hasCreatedAt() => $_has(8);
  @$pb.TagNumber(9)
  void clearCreatedAt() => $_clearField(9);
  @$pb.TagNumber(9)
  $1.Timestamp ensureCreatedAt() => $_ensure(8);

  @$pb.TagNumber(10)
  $1.Timestamp get updatedAt => $_getN(9);
  @$pb.TagNumber(10)
  set updatedAt($1.Timestamp value) => $_setField(10, value);
  @$pb.TagNumber(10)
  $core.bool hasUpdatedAt() => $_has(9);
  @$pb.TagNumber(10)
  void clearUpdatedAt() => $_clearField(10);
  @$pb.TagNumber(10)
  $1.Timestamp ensureUpdatedAt() => $_ensure(9);
}

/// GetConfigRequest は設定値取得リクエスト。
class GetConfigRequest extends $pb.GeneratedMessage {
  factory GetConfigRequest({
    $core.String? namespace,
    $core.String? key,
  }) {
    final result = create();
    if (namespace != null) result.namespace = namespace;
    if (key != null) result.key = key;
    return result;
  }

  GetConfigRequest._();

  factory GetConfigRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory GetConfigRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'GetConfigRequest',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.config.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'namespace')
    ..aOS(2, _omitFieldNames ? '' : 'key')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetConfigRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetConfigRequest copyWith(void Function(GetConfigRequest) updates) =>
      super.copyWith((message) => updates(message as GetConfigRequest))
          as GetConfigRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static GetConfigRequest create() => GetConfigRequest._();
  @$core.override
  GetConfigRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static GetConfigRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<GetConfigRequest>(create);
  static GetConfigRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get namespace => $_getSZ(0);
  @$pb.TagNumber(1)
  set namespace($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasNamespace() => $_has(0);
  @$pb.TagNumber(1)
  void clearNamespace() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get key => $_getSZ(1);
  @$pb.TagNumber(2)
  set key($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasKey() => $_has(1);
  @$pb.TagNumber(2)
  void clearKey() => $_clearField(2);
}

/// GetConfigResponse は設定値取得レスポンス。
class GetConfigResponse extends $pb.GeneratedMessage {
  factory GetConfigResponse({
    ConfigEntry? entry,
  }) {
    final result = create();
    if (entry != null) result.entry = entry;
    return result;
  }

  GetConfigResponse._();

  factory GetConfigResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory GetConfigResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'GetConfigResponse',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.config.v1'),
      createEmptyInstance: create)
    ..aOM<ConfigEntry>(1, _omitFieldNames ? '' : 'entry',
        subBuilder: ConfigEntry.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetConfigResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetConfigResponse copyWith(void Function(GetConfigResponse) updates) =>
      super.copyWith((message) => updates(message as GetConfigResponse))
          as GetConfigResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static GetConfigResponse create() => GetConfigResponse._();
  @$core.override
  GetConfigResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static GetConfigResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<GetConfigResponse>(create);
  static GetConfigResponse? _defaultInstance;

  @$pb.TagNumber(1)
  ConfigEntry get entry => $_getN(0);
  @$pb.TagNumber(1)
  set entry(ConfigEntry value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasEntry() => $_has(0);
  @$pb.TagNumber(1)
  void clearEntry() => $_clearField(1);
  @$pb.TagNumber(1)
  ConfigEntry ensureEntry() => $_ensure(0);
}

/// ListConfigsRequest は設定値一覧取得リクエスト。
class ListConfigsRequest extends $pb.GeneratedMessage {
  factory ListConfigsRequest({
    $core.String? namespace,
    $1.Pagination? pagination,
    $core.String? search,
  }) {
    final result = create();
    if (namespace != null) result.namespace = namespace;
    if (pagination != null) result.pagination = pagination;
    if (search != null) result.search = search;
    return result;
  }

  ListConfigsRequest._();

  factory ListConfigsRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory ListConfigsRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ListConfigsRequest',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.config.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'namespace')
    ..aOM<$1.Pagination>(2, _omitFieldNames ? '' : 'pagination',
        subBuilder: $1.Pagination.create)
    ..aOS(3, _omitFieldNames ? '' : 'search')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListConfigsRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListConfigsRequest copyWith(void Function(ListConfigsRequest) updates) =>
      super.copyWith((message) => updates(message as ListConfigsRequest))
          as ListConfigsRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static ListConfigsRequest create() => ListConfigsRequest._();
  @$core.override
  ListConfigsRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static ListConfigsRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ListConfigsRequest>(create);
  static ListConfigsRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get namespace => $_getSZ(0);
  @$pb.TagNumber(1)
  set namespace($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasNamespace() => $_has(0);
  @$pb.TagNumber(1)
  void clearNamespace() => $_clearField(1);

  @$pb.TagNumber(2)
  $1.Pagination get pagination => $_getN(1);
  @$pb.TagNumber(2)
  set pagination($1.Pagination value) => $_setField(2, value);
  @$pb.TagNumber(2)
  $core.bool hasPagination() => $_has(1);
  @$pb.TagNumber(2)
  void clearPagination() => $_clearField(2);
  @$pb.TagNumber(2)
  $1.Pagination ensurePagination() => $_ensure(1);

  /// キー名の部分一致検索
  @$pb.TagNumber(3)
  $core.String get search => $_getSZ(2);
  @$pb.TagNumber(3)
  set search($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasSearch() => $_has(2);
  @$pb.TagNumber(3)
  void clearSearch() => $_clearField(3);
}

/// ListConfigsResponse は設定値一覧取得レスポンス。
class ListConfigsResponse extends $pb.GeneratedMessage {
  factory ListConfigsResponse({
    $core.Iterable<ConfigEntry>? entries,
    $1.PaginationResult? pagination,
  }) {
    final result = create();
    if (entries != null) result.entries.addAll(entries);
    if (pagination != null) result.pagination = pagination;
    return result;
  }

  ListConfigsResponse._();

  factory ListConfigsResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory ListConfigsResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ListConfigsResponse',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.config.v1'),
      createEmptyInstance: create)
    ..pPM<ConfigEntry>(1, _omitFieldNames ? '' : 'entries',
        subBuilder: ConfigEntry.create)
    ..aOM<$1.PaginationResult>(2, _omitFieldNames ? '' : 'pagination',
        subBuilder: $1.PaginationResult.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListConfigsResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListConfigsResponse copyWith(void Function(ListConfigsResponse) updates) =>
      super.copyWith((message) => updates(message as ListConfigsResponse))
          as ListConfigsResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static ListConfigsResponse create() => ListConfigsResponse._();
  @$core.override
  ListConfigsResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static ListConfigsResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ListConfigsResponse>(create);
  static ListConfigsResponse? _defaultInstance;

  @$pb.TagNumber(1)
  $pb.PbList<ConfigEntry> get entries => $_getList(0);

  @$pb.TagNumber(2)
  $1.PaginationResult get pagination => $_getN(1);
  @$pb.TagNumber(2)
  set pagination($1.PaginationResult value) => $_setField(2, value);
  @$pb.TagNumber(2)
  $core.bool hasPagination() => $_has(1);
  @$pb.TagNumber(2)
  void clearPagination() => $_clearField(2);
  @$pb.TagNumber(2)
  $1.PaginationResult ensurePagination() => $_ensure(1);
}

/// UpdateConfigRequest は設定値更新リクエスト。
class UpdateConfigRequest extends $pb.GeneratedMessage {
  factory UpdateConfigRequest({
    $core.String? namespace,
    $core.String? key,
    $core.List<$core.int>? value,
    $core.int? version,
    $core.String? description,
    $core.String? updatedBy,
  }) {
    final result = create();
    if (namespace != null) result.namespace = namespace;
    if (key != null) result.key = key;
    if (value != null) result.value = value;
    if (version != null) result.version = version;
    if (description != null) result.description = description;
    if (updatedBy != null) result.updatedBy = updatedBy;
    return result;
  }

  UpdateConfigRequest._();

  factory UpdateConfigRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory UpdateConfigRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'UpdateConfigRequest',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.config.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'namespace')
    ..aOS(2, _omitFieldNames ? '' : 'key')
    ..a<$core.List<$core.int>>(
        3, _omitFieldNames ? '' : 'value', $pb.PbFieldType.OY)
    ..aI(4, _omitFieldNames ? '' : 'version')
    ..aOS(5, _omitFieldNames ? '' : 'description')
    ..aOS(6, _omitFieldNames ? '' : 'updatedBy')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  UpdateConfigRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  UpdateConfigRequest copyWith(void Function(UpdateConfigRequest) updates) =>
      super.copyWith((message) => updates(message as UpdateConfigRequest))
          as UpdateConfigRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static UpdateConfigRequest create() => UpdateConfigRequest._();
  @$core.override
  UpdateConfigRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static UpdateConfigRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<UpdateConfigRequest>(create);
  static UpdateConfigRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get namespace => $_getSZ(0);
  @$pb.TagNumber(1)
  set namespace($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasNamespace() => $_has(0);
  @$pb.TagNumber(1)
  void clearNamespace() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get key => $_getSZ(1);
  @$pb.TagNumber(2)
  set key($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasKey() => $_has(1);
  @$pb.TagNumber(2)
  void clearKey() => $_clearField(2);

  /// JSON エンコード済みの値
  @$pb.TagNumber(3)
  $core.List<$core.int> get value => $_getN(2);
  @$pb.TagNumber(3)
  set value($core.List<$core.int> value) => $_setBytes(2, value);
  @$pb.TagNumber(3)
  $core.bool hasValue() => $_has(2);
  @$pb.TagNumber(3)
  void clearValue() => $_clearField(3);

  /// 楽観的排他制御用（現在のバージョン番号）
  @$pb.TagNumber(4)
  $core.int get version => $_getIZ(3);
  @$pb.TagNumber(4)
  set version($core.int value) => $_setSignedInt32(3, value);
  @$pb.TagNumber(4)
  $core.bool hasVersion() => $_has(3);
  @$pb.TagNumber(4)
  void clearVersion() => $_clearField(4);

  @$pb.TagNumber(5)
  $core.String get description => $_getSZ(4);
  @$pb.TagNumber(5)
  set description($core.String value) => $_setString(4, value);
  @$pb.TagNumber(5)
  $core.bool hasDescription() => $_has(4);
  @$pb.TagNumber(5)
  void clearDescription() => $_clearField(5);

  @$pb.TagNumber(6)
  $core.String get updatedBy => $_getSZ(5);
  @$pb.TagNumber(6)
  set updatedBy($core.String value) => $_setString(5, value);
  @$pb.TagNumber(6)
  $core.bool hasUpdatedBy() => $_has(5);
  @$pb.TagNumber(6)
  void clearUpdatedBy() => $_clearField(6);
}

/// UpdateConfigResponse は設定値更新レスポンス。
class UpdateConfigResponse extends $pb.GeneratedMessage {
  factory UpdateConfigResponse({
    ConfigEntry? entry,
  }) {
    final result = create();
    if (entry != null) result.entry = entry;
    return result;
  }

  UpdateConfigResponse._();

  factory UpdateConfigResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory UpdateConfigResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'UpdateConfigResponse',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.config.v1'),
      createEmptyInstance: create)
    ..aOM<ConfigEntry>(1, _omitFieldNames ? '' : 'entry',
        subBuilder: ConfigEntry.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  UpdateConfigResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  UpdateConfigResponse copyWith(void Function(UpdateConfigResponse) updates) =>
      super.copyWith((message) => updates(message as UpdateConfigResponse))
          as UpdateConfigResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static UpdateConfigResponse create() => UpdateConfigResponse._();
  @$core.override
  UpdateConfigResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static UpdateConfigResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<UpdateConfigResponse>(create);
  static UpdateConfigResponse? _defaultInstance;

  @$pb.TagNumber(1)
  ConfigEntry get entry => $_getN(0);
  @$pb.TagNumber(1)
  set entry(ConfigEntry value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasEntry() => $_has(0);
  @$pb.TagNumber(1)
  void clearEntry() => $_clearField(1);
  @$pb.TagNumber(1)
  ConfigEntry ensureEntry() => $_ensure(0);
}

/// DeleteConfigRequest は設定値削除リクエスト。
class DeleteConfigRequest extends $pb.GeneratedMessage {
  factory DeleteConfigRequest({
    $core.String? namespace,
    $core.String? key,
    $core.String? deletedBy,
  }) {
    final result = create();
    if (namespace != null) result.namespace = namespace;
    if (key != null) result.key = key;
    if (deletedBy != null) result.deletedBy = deletedBy;
    return result;
  }

  DeleteConfigRequest._();

  factory DeleteConfigRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory DeleteConfigRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'DeleteConfigRequest',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.config.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'namespace')
    ..aOS(2, _omitFieldNames ? '' : 'key')
    ..aOS(3, _omitFieldNames ? '' : 'deletedBy')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  DeleteConfigRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  DeleteConfigRequest copyWith(void Function(DeleteConfigRequest) updates) =>
      super.copyWith((message) => updates(message as DeleteConfigRequest))
          as DeleteConfigRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static DeleteConfigRequest create() => DeleteConfigRequest._();
  @$core.override
  DeleteConfigRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static DeleteConfigRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<DeleteConfigRequest>(create);
  static DeleteConfigRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get namespace => $_getSZ(0);
  @$pb.TagNumber(1)
  set namespace($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasNamespace() => $_has(0);
  @$pb.TagNumber(1)
  void clearNamespace() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get key => $_getSZ(1);
  @$pb.TagNumber(2)
  set key($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasKey() => $_has(1);
  @$pb.TagNumber(2)
  void clearKey() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.String get deletedBy => $_getSZ(2);
  @$pb.TagNumber(3)
  set deletedBy($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasDeletedBy() => $_has(2);
  @$pb.TagNumber(3)
  void clearDeletedBy() => $_clearField(3);
}

/// DeleteConfigResponse は設定値削除レスポンス。
class DeleteConfigResponse extends $pb.GeneratedMessage {
  factory DeleteConfigResponse({
    $core.bool? success,
  }) {
    final result = create();
    if (success != null) result.success = success;
    return result;
  }

  DeleteConfigResponse._();

  factory DeleteConfigResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory DeleteConfigResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'DeleteConfigResponse',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.config.v1'),
      createEmptyInstance: create)
    ..aOB(1, _omitFieldNames ? '' : 'success')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  DeleteConfigResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  DeleteConfigResponse copyWith(void Function(DeleteConfigResponse) updates) =>
      super.copyWith((message) => updates(message as DeleteConfigResponse))
          as DeleteConfigResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static DeleteConfigResponse create() => DeleteConfigResponse._();
  @$core.override
  DeleteConfigResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static DeleteConfigResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<DeleteConfigResponse>(create);
  static DeleteConfigResponse? _defaultInstance;

  @$pb.TagNumber(1)
  $core.bool get success => $_getBF(0);
  @$pb.TagNumber(1)
  set success($core.bool value) => $_setBool(0, value);
  @$pb.TagNumber(1)
  $core.bool hasSuccess() => $_has(0);
  @$pb.TagNumber(1)
  void clearSuccess() => $_clearField(1);
}

/// GetServiceConfigRequest はサービス向け設定一括取得リクエスト。
class GetServiceConfigRequest extends $pb.GeneratedMessage {
  factory GetServiceConfigRequest({
    $core.String? serviceName,
    $core.String? environment,
  }) {
    final result = create();
    if (serviceName != null) result.serviceName = serviceName;
    if (environment != null) result.environment = environment;
    return result;
  }

  GetServiceConfigRequest._();

  factory GetServiceConfigRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory GetServiceConfigRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'GetServiceConfigRequest',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.config.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'serviceName')
    ..aOS(2, _omitFieldNames ? '' : 'environment')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetServiceConfigRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetServiceConfigRequest copyWith(
          void Function(GetServiceConfigRequest) updates) =>
      super.copyWith((message) => updates(message as GetServiceConfigRequest))
          as GetServiceConfigRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static GetServiceConfigRequest create() => GetServiceConfigRequest._();
  @$core.override
  GetServiceConfigRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static GetServiceConfigRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<GetServiceConfigRequest>(create);
  static GetServiceConfigRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get serviceName => $_getSZ(0);
  @$pb.TagNumber(1)
  set serviceName($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasServiceName() => $_has(0);
  @$pb.TagNumber(1)
  void clearServiceName() => $_clearField(1);

  /// dev | staging | prod
  @$pb.TagNumber(2)
  $core.String get environment => $_getSZ(1);
  @$pb.TagNumber(2)
  set environment($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasEnvironment() => $_has(1);
  @$pb.TagNumber(2)
  void clearEnvironment() => $_clearField(2);
}

/// GetServiceConfigResponse はサービス向け設定一括取得レスポンス。
class ServiceConfigEntry extends $pb.GeneratedMessage {
  factory ServiceConfigEntry({
    $core.String? namespace,
    $core.String? key,
    $core.String? value,
    $core.int? version,
  }) {
    final result = create();
    if (namespace != null) result.namespace = namespace;
    if (key != null) result.key = key;
    if (value != null) result.value = value;
    if (version != null) result.version = version;
    return result;
  }

  ServiceConfigEntry._();

  factory ServiceConfigEntry.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory ServiceConfigEntry.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ServiceConfigEntry',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.config.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'namespace')
    ..aOS(2, _omitFieldNames ? '' : 'key')
    ..aOS(3, _omitFieldNames ? '' : 'value')
    ..aI(4, _omitFieldNames ? '' : 'version')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ServiceConfigEntry clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ServiceConfigEntry copyWith(void Function(ServiceConfigEntry) updates) =>
      super.copyWith((message) => updates(message as ServiceConfigEntry))
          as ServiceConfigEntry;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static ServiceConfigEntry create() => ServiceConfigEntry._();
  @$core.override
  ServiceConfigEntry createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static ServiceConfigEntry getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ServiceConfigEntry>(create);
  static ServiceConfigEntry? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get namespace => $_getSZ(0);
  @$pb.TagNumber(1)
  set namespace($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasNamespace() => $_has(0);
  @$pb.TagNumber(1)
  void clearNamespace() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get key => $_getSZ(1);
  @$pb.TagNumber(2)
  set key($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasKey() => $_has(1);
  @$pb.TagNumber(2)
  void clearKey() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.String get value => $_getSZ(2);
  @$pb.TagNumber(3)
  set value($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasValue() => $_has(2);
  @$pb.TagNumber(3)
  void clearValue() => $_clearField(3);

  @$pb.TagNumber(4)
  $core.int get version => $_getIZ(3);
  @$pb.TagNumber(4)
  set version($core.int value) => $_setSignedInt32(3, value);
  @$pb.TagNumber(4)
  $core.bool hasVersion() => $_has(3);
  @$pb.TagNumber(4)
  void clearVersion() => $_clearField(4);
}

/// GetServiceConfigResponse はサービス向け設定一括取得レスポンス。
class GetServiceConfigResponse extends $pb.GeneratedMessage {
  factory GetServiceConfigResponse({
    $core.Iterable<ServiceConfigEntry>? entries,
  }) {
    final result = create();
    if (entries != null) result.entries.addAll(entries);
    return result;
  }

  GetServiceConfigResponse._();

  factory GetServiceConfigResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory GetServiceConfigResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'GetServiceConfigResponse',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.config.v1'),
      createEmptyInstance: create)
    ..pPM<ServiceConfigEntry>(1, _omitFieldNames ? '' : 'entries',
        subBuilder: ServiceConfigEntry.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetServiceConfigResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetServiceConfigResponse copyWith(
          void Function(GetServiceConfigResponse) updates) =>
      super.copyWith((message) => updates(message as GetServiceConfigResponse))
          as GetServiceConfigResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static GetServiceConfigResponse create() => GetServiceConfigResponse._();
  @$core.override
  GetServiceConfigResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static GetServiceConfigResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<GetServiceConfigResponse>(create);
  static GetServiceConfigResponse? _defaultInstance;

  @$pb.TagNumber(1)
  $pb.PbList<ServiceConfigEntry> get entries => $_getList(0);
}

/// WatchConfigRequest は設定変更監視リクエスト。
class WatchConfigRequest extends $pb.GeneratedMessage {
  factory WatchConfigRequest({
    $core.Iterable<$core.String>? namespaces,
  }) {
    final result = create();
    if (namespaces != null) result.namespaces.addAll(namespaces);
    return result;
  }

  WatchConfigRequest._();

  factory WatchConfigRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory WatchConfigRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'WatchConfigRequest',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.config.v1'),
      createEmptyInstance: create)
    ..pPS(1, _omitFieldNames ? '' : 'namespaces')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  WatchConfigRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  WatchConfigRequest copyWith(void Function(WatchConfigRequest) updates) =>
      super.copyWith((message) => updates(message as WatchConfigRequest))
          as WatchConfigRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static WatchConfigRequest create() => WatchConfigRequest._();
  @$core.override
  WatchConfigRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static WatchConfigRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<WatchConfigRequest>(create);
  static WatchConfigRequest? _defaultInstance;

  /// 監視対象 namespace（空の場合は全件）
  @$pb.TagNumber(1)
  $pb.PbList<$core.String> get namespaces => $_getList(0);
}

/// WatchConfigResponse は設定変更の監視レスポンス（ストリーミング）。
class WatchConfigResponse extends $pb.GeneratedMessage {
  factory WatchConfigResponse({
    $core.String? namespace,
    $core.String? key,
    $core.List<$core.int>? oldValue,
    $core.List<$core.int>? newValue,
    $core.int? oldVersion,
    $core.int? newVersion,
    $core.String? changedBy,
    $core.String? changeType,
    $1.Timestamp? changedAt,
    $1.ChangeType? changeTypeEnum,
  }) {
    final result = create();
    if (namespace != null) result.namespace = namespace;
    if (key != null) result.key = key;
    if (oldValue != null) result.oldValue = oldValue;
    if (newValue != null) result.newValue = newValue;
    if (oldVersion != null) result.oldVersion = oldVersion;
    if (newVersion != null) result.newVersion = newVersion;
    if (changedBy != null) result.changedBy = changedBy;
    if (changeType != null) result.changeType = changeType;
    if (changedAt != null) result.changedAt = changedAt;
    if (changeTypeEnum != null) result.changeTypeEnum = changeTypeEnum;
    return result;
  }

  WatchConfigResponse._();

  factory WatchConfigResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory WatchConfigResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'WatchConfigResponse',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.config.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'namespace')
    ..aOS(2, _omitFieldNames ? '' : 'key')
    ..a<$core.List<$core.int>>(
        3, _omitFieldNames ? '' : 'oldValue', $pb.PbFieldType.OY)
    ..a<$core.List<$core.int>>(
        4, _omitFieldNames ? '' : 'newValue', $pb.PbFieldType.OY)
    ..aI(5, _omitFieldNames ? '' : 'oldVersion')
    ..aI(6, _omitFieldNames ? '' : 'newVersion')
    ..aOS(7, _omitFieldNames ? '' : 'changedBy')
    ..aOS(8, _omitFieldNames ? '' : 'changeType')
    ..aOM<$1.Timestamp>(9, _omitFieldNames ? '' : 'changedAt',
        subBuilder: $1.Timestamp.create)
    ..aE<$1.ChangeType>(10, _omitFieldNames ? '' : 'changeTypeEnum',
        enumValues: $1.ChangeType.values)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  WatchConfigResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  WatchConfigResponse copyWith(void Function(WatchConfigResponse) updates) =>
      super.copyWith((message) => updates(message as WatchConfigResponse))
          as WatchConfigResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static WatchConfigResponse create() => WatchConfigResponse._();
  @$core.override
  WatchConfigResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static WatchConfigResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<WatchConfigResponse>(create);
  static WatchConfigResponse? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get namespace => $_getSZ(0);
  @$pb.TagNumber(1)
  set namespace($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasNamespace() => $_has(0);
  @$pb.TagNumber(1)
  void clearNamespace() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get key => $_getSZ(1);
  @$pb.TagNumber(2)
  set key($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasKey() => $_has(1);
  @$pb.TagNumber(2)
  void clearKey() => $_clearField(2);

  /// 変更前の値（JSON エンコード済み）
  @$pb.TagNumber(3)
  $core.List<$core.int> get oldValue => $_getN(2);
  @$pb.TagNumber(3)
  set oldValue($core.List<$core.int> value) => $_setBytes(2, value);
  @$pb.TagNumber(3)
  $core.bool hasOldValue() => $_has(2);
  @$pb.TagNumber(3)
  void clearOldValue() => $_clearField(3);

  /// 変更後の値（JSON エンコード済み）
  @$pb.TagNumber(4)
  $core.List<$core.int> get newValue => $_getN(3);
  @$pb.TagNumber(4)
  set newValue($core.List<$core.int> value) => $_setBytes(3, value);
  @$pb.TagNumber(4)
  $core.bool hasNewValue() => $_has(3);
  @$pb.TagNumber(4)
  void clearNewValue() => $_clearField(4);

  @$pb.TagNumber(5)
  $core.int get oldVersion => $_getIZ(4);
  @$pb.TagNumber(5)
  set oldVersion($core.int value) => $_setSignedInt32(4, value);
  @$pb.TagNumber(5)
  $core.bool hasOldVersion() => $_has(4);
  @$pb.TagNumber(5)
  void clearOldVersion() => $_clearField(5);

  @$pb.TagNumber(6)
  $core.int get newVersion => $_getIZ(5);
  @$pb.TagNumber(6)
  set newVersion($core.int value) => $_setSignedInt32(5, value);
  @$pb.TagNumber(6)
  $core.bool hasNewVersion() => $_has(5);
  @$pb.TagNumber(6)
  void clearNewVersion() => $_clearField(6);

  @$pb.TagNumber(7)
  $core.String get changedBy => $_getSZ(6);
  @$pb.TagNumber(7)
  set changedBy($core.String value) => $_setString(6, value);
  @$pb.TagNumber(7)
  $core.bool hasChangedBy() => $_has(6);
  @$pb.TagNumber(7)
  void clearChangedBy() => $_clearField(7);

  /// Deprecated: use change_type_enum instead.
  /// CREATED, UPDATED, DELETED
  @$pb.TagNumber(8)
  $core.String get changeType => $_getSZ(7);
  @$pb.TagNumber(8)
  set changeType($core.String value) => $_setString(7, value);
  @$pb.TagNumber(8)
  $core.bool hasChangeType() => $_has(7);
  @$pb.TagNumber(8)
  void clearChangeType() => $_clearField(8);

  @$pb.TagNumber(9)
  $1.Timestamp get changedAt => $_getN(8);
  @$pb.TagNumber(9)
  set changedAt($1.Timestamp value) => $_setField(9, value);
  @$pb.TagNumber(9)
  $core.bool hasChangedAt() => $_has(8);
  @$pb.TagNumber(9)
  void clearChangedAt() => $_clearField(9);
  @$pb.TagNumber(9)
  $1.Timestamp ensureChangedAt() => $_ensure(8);

  /// 変更操作の種別（change_type の enum 版）。
  @$pb.TagNumber(10)
  $1.ChangeType get changeTypeEnum => $_getN(9);
  @$pb.TagNumber(10)
  set changeTypeEnum($1.ChangeType value) => $_setField(10, value);
  @$pb.TagNumber(10)
  $core.bool hasChangeTypeEnum() => $_has(9);
  @$pb.TagNumber(10)
  void clearChangeTypeEnum() => $_clearField(10);
}

/// ConfigFieldSchema は設定フィールドのスキーマ定義。
class ConfigFieldSchema extends $pb.GeneratedMessage {
  factory ConfigFieldSchema({
    $core.String? key,
    $core.String? label,
    $core.String? description,
    ConfigFieldType? type,
    $fixnum.Int64? min,
    $fixnum.Int64? max,
    $core.Iterable<$core.String>? options,
    $core.String? pattern,
    $core.String? unit,
    $core.List<$core.int>? defaultValue,
  }) {
    final result = create();
    if (key != null) result.key = key;
    if (label != null) result.label = label;
    if (description != null) result.description = description;
    if (type != null) result.type = type;
    if (min != null) result.min = min;
    if (max != null) result.max = max;
    if (options != null) result.options.addAll(options);
    if (pattern != null) result.pattern = pattern;
    if (unit != null) result.unit = unit;
    if (defaultValue != null) result.defaultValue = defaultValue;
    return result;
  }

  ConfigFieldSchema._();

  factory ConfigFieldSchema.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory ConfigFieldSchema.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ConfigFieldSchema',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.config.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'key')
    ..aOS(2, _omitFieldNames ? '' : 'label')
    ..aOS(3, _omitFieldNames ? '' : 'description')
    ..aE<ConfigFieldType>(4, _omitFieldNames ? '' : 'type',
        enumValues: ConfigFieldType.values)
    ..aInt64(5, _omitFieldNames ? '' : 'min')
    ..aInt64(6, _omitFieldNames ? '' : 'max')
    ..pPS(7, _omitFieldNames ? '' : 'options')
    ..aOS(8, _omitFieldNames ? '' : 'pattern')
    ..aOS(9, _omitFieldNames ? '' : 'unit')
    ..a<$core.List<$core.int>>(
        10, _omitFieldNames ? '' : 'defaultValue', $pb.PbFieldType.OY)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ConfigFieldSchema clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ConfigFieldSchema copyWith(void Function(ConfigFieldSchema) updates) =>
      super.copyWith((message) => updates(message as ConfigFieldSchema))
          as ConfigFieldSchema;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static ConfigFieldSchema create() => ConfigFieldSchema._();
  @$core.override
  ConfigFieldSchema createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static ConfigFieldSchema getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ConfigFieldSchema>(create);
  static ConfigFieldSchema? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get key => $_getSZ(0);
  @$pb.TagNumber(1)
  set key($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasKey() => $_has(0);
  @$pb.TagNumber(1)
  void clearKey() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get label => $_getSZ(1);
  @$pb.TagNumber(2)
  set label($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasLabel() => $_has(1);
  @$pb.TagNumber(2)
  void clearLabel() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.String get description => $_getSZ(2);
  @$pb.TagNumber(3)
  set description($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasDescription() => $_has(2);
  @$pb.TagNumber(3)
  void clearDescription() => $_clearField(3);

  @$pb.TagNumber(4)
  ConfigFieldType get type => $_getN(3);
  @$pb.TagNumber(4)
  set type(ConfigFieldType value) => $_setField(4, value);
  @$pb.TagNumber(4)
  $core.bool hasType() => $_has(3);
  @$pb.TagNumber(4)
  void clearType() => $_clearField(4);

  @$pb.TagNumber(5)
  $fixnum.Int64 get min => $_getI64(4);
  @$pb.TagNumber(5)
  set min($fixnum.Int64 value) => $_setInt64(4, value);
  @$pb.TagNumber(5)
  $core.bool hasMin() => $_has(4);
  @$pb.TagNumber(5)
  void clearMin() => $_clearField(5);

  @$pb.TagNumber(6)
  $fixnum.Int64 get max => $_getI64(5);
  @$pb.TagNumber(6)
  set max($fixnum.Int64 value) => $_setInt64(5, value);
  @$pb.TagNumber(6)
  $core.bool hasMax() => $_has(5);
  @$pb.TagNumber(6)
  void clearMax() => $_clearField(6);

  @$pb.TagNumber(7)
  $pb.PbList<$core.String> get options => $_getList(6);

  @$pb.TagNumber(8)
  $core.String get pattern => $_getSZ(7);
  @$pb.TagNumber(8)
  set pattern($core.String value) => $_setString(7, value);
  @$pb.TagNumber(8)
  $core.bool hasPattern() => $_has(7);
  @$pb.TagNumber(8)
  void clearPattern() => $_clearField(8);

  @$pb.TagNumber(9)
  $core.String get unit => $_getSZ(8);
  @$pb.TagNumber(9)
  set unit($core.String value) => $_setString(8, value);
  @$pb.TagNumber(9)
  $core.bool hasUnit() => $_has(8);
  @$pb.TagNumber(9)
  void clearUnit() => $_clearField(9);

  @$pb.TagNumber(10)
  $core.List<$core.int> get defaultValue => $_getN(9);
  @$pb.TagNumber(10)
  set defaultValue($core.List<$core.int> value) => $_setBytes(9, value);
  @$pb.TagNumber(10)
  $core.bool hasDefaultValue() => $_has(9);
  @$pb.TagNumber(10)
  void clearDefaultValue() => $_clearField(10);
}

/// ConfigCategorySchema はカテゴリ単位のスキーマ定義。
class ConfigCategorySchema extends $pb.GeneratedMessage {
  factory ConfigCategorySchema({
    $core.String? id,
    $core.String? label,
    $core.String? icon,
    $core.Iterable<$core.String>? namespaces,
    $core.Iterable<ConfigFieldSchema>? fields,
  }) {
    final result = create();
    if (id != null) result.id = id;
    if (label != null) result.label = label;
    if (icon != null) result.icon = icon;
    if (namespaces != null) result.namespaces.addAll(namespaces);
    if (fields != null) result.fields.addAll(fields);
    return result;
  }

  ConfigCategorySchema._();

  factory ConfigCategorySchema.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory ConfigCategorySchema.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ConfigCategorySchema',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.config.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'id')
    ..aOS(2, _omitFieldNames ? '' : 'label')
    ..aOS(3, _omitFieldNames ? '' : 'icon')
    ..pPS(4, _omitFieldNames ? '' : 'namespaces')
    ..pPM<ConfigFieldSchema>(5, _omitFieldNames ? '' : 'fields',
        subBuilder: ConfigFieldSchema.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ConfigCategorySchema clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ConfigCategorySchema copyWith(void Function(ConfigCategorySchema) updates) =>
      super.copyWith((message) => updates(message as ConfigCategorySchema))
          as ConfigCategorySchema;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static ConfigCategorySchema create() => ConfigCategorySchema._();
  @$core.override
  ConfigCategorySchema createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static ConfigCategorySchema getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ConfigCategorySchema>(create);
  static ConfigCategorySchema? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get id => $_getSZ(0);
  @$pb.TagNumber(1)
  set id($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasId() => $_has(0);
  @$pb.TagNumber(1)
  void clearId() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get label => $_getSZ(1);
  @$pb.TagNumber(2)
  set label($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasLabel() => $_has(1);
  @$pb.TagNumber(2)
  void clearLabel() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.String get icon => $_getSZ(2);
  @$pb.TagNumber(3)
  set icon($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasIcon() => $_has(2);
  @$pb.TagNumber(3)
  void clearIcon() => $_clearField(3);

  @$pb.TagNumber(4)
  $pb.PbList<$core.String> get namespaces => $_getList(3);

  @$pb.TagNumber(5)
  $pb.PbList<ConfigFieldSchema> get fields => $_getList(4);
}

/// ConfigEditorSchema はサービスの設定エディタスキーマ全体を表す。
class ConfigEditorSchema extends $pb.GeneratedMessage {
  factory ConfigEditorSchema({
    $core.String? serviceName,
    $core.String? namespacePrefix,
    $core.Iterable<ConfigCategorySchema>? categories,
    $1.Timestamp? updatedAt,
  }) {
    final result = create();
    if (serviceName != null) result.serviceName = serviceName;
    if (namespacePrefix != null) result.namespacePrefix = namespacePrefix;
    if (categories != null) result.categories.addAll(categories);
    if (updatedAt != null) result.updatedAt = updatedAt;
    return result;
  }

  ConfigEditorSchema._();

  factory ConfigEditorSchema.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory ConfigEditorSchema.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ConfigEditorSchema',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.config.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'serviceName')
    ..aOS(2, _omitFieldNames ? '' : 'namespacePrefix')
    ..pPM<ConfigCategorySchema>(3, _omitFieldNames ? '' : 'categories',
        subBuilder: ConfigCategorySchema.create)
    ..aOM<$1.Timestamp>(4, _omitFieldNames ? '' : 'updatedAt',
        subBuilder: $1.Timestamp.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ConfigEditorSchema clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ConfigEditorSchema copyWith(void Function(ConfigEditorSchema) updates) =>
      super.copyWith((message) => updates(message as ConfigEditorSchema))
          as ConfigEditorSchema;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static ConfigEditorSchema create() => ConfigEditorSchema._();
  @$core.override
  ConfigEditorSchema createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static ConfigEditorSchema getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ConfigEditorSchema>(create);
  static ConfigEditorSchema? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get serviceName => $_getSZ(0);
  @$pb.TagNumber(1)
  set serviceName($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasServiceName() => $_has(0);
  @$pb.TagNumber(1)
  void clearServiceName() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get namespacePrefix => $_getSZ(1);
  @$pb.TagNumber(2)
  set namespacePrefix($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasNamespacePrefix() => $_has(1);
  @$pb.TagNumber(2)
  void clearNamespacePrefix() => $_clearField(2);

  @$pb.TagNumber(3)
  $pb.PbList<ConfigCategorySchema> get categories => $_getList(2);

  @$pb.TagNumber(4)
  $1.Timestamp get updatedAt => $_getN(3);
  @$pb.TagNumber(4)
  set updatedAt($1.Timestamp value) => $_setField(4, value);
  @$pb.TagNumber(4)
  $core.bool hasUpdatedAt() => $_has(3);
  @$pb.TagNumber(4)
  void clearUpdatedAt() => $_clearField(4);
  @$pb.TagNumber(4)
  $1.Timestamp ensureUpdatedAt() => $_ensure(3);
}

/// GetConfigSchemaRequest は設定スキーマ取得リクエスト。
class GetConfigSchemaRequest extends $pb.GeneratedMessage {
  factory GetConfigSchemaRequest({
    $core.String? serviceName,
  }) {
    final result = create();
    if (serviceName != null) result.serviceName = serviceName;
    return result;
  }

  GetConfigSchemaRequest._();

  factory GetConfigSchemaRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory GetConfigSchemaRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'GetConfigSchemaRequest',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.config.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'serviceName')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetConfigSchemaRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetConfigSchemaRequest copyWith(
          void Function(GetConfigSchemaRequest) updates) =>
      super.copyWith((message) => updates(message as GetConfigSchemaRequest))
          as GetConfigSchemaRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static GetConfigSchemaRequest create() => GetConfigSchemaRequest._();
  @$core.override
  GetConfigSchemaRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static GetConfigSchemaRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<GetConfigSchemaRequest>(create);
  static GetConfigSchemaRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get serviceName => $_getSZ(0);
  @$pb.TagNumber(1)
  set serviceName($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasServiceName() => $_has(0);
  @$pb.TagNumber(1)
  void clearServiceName() => $_clearField(1);
}

/// GetConfigSchemaResponse は設定スキーマ取得レスポンス。
class GetConfigSchemaResponse extends $pb.GeneratedMessage {
  factory GetConfigSchemaResponse({
    ConfigEditorSchema? schema,
  }) {
    final result = create();
    if (schema != null) result.schema = schema;
    return result;
  }

  GetConfigSchemaResponse._();

  factory GetConfigSchemaResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory GetConfigSchemaResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'GetConfigSchemaResponse',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.config.v1'),
      createEmptyInstance: create)
    ..aOM<ConfigEditorSchema>(1, _omitFieldNames ? '' : 'schema',
        subBuilder: ConfigEditorSchema.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetConfigSchemaResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetConfigSchemaResponse copyWith(
          void Function(GetConfigSchemaResponse) updates) =>
      super.copyWith((message) => updates(message as GetConfigSchemaResponse))
          as GetConfigSchemaResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static GetConfigSchemaResponse create() => GetConfigSchemaResponse._();
  @$core.override
  GetConfigSchemaResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static GetConfigSchemaResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<GetConfigSchemaResponse>(create);
  static GetConfigSchemaResponse? _defaultInstance;

  @$pb.TagNumber(1)
  ConfigEditorSchema get schema => $_getN(0);
  @$pb.TagNumber(1)
  set schema(ConfigEditorSchema value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasSchema() => $_has(0);
  @$pb.TagNumber(1)
  void clearSchema() => $_clearField(1);
  @$pb.TagNumber(1)
  ConfigEditorSchema ensureSchema() => $_ensure(0);
}

/// UpsertConfigSchemaRequest は設定スキーマ作成・更新リクエスト。
class UpsertConfigSchemaRequest extends $pb.GeneratedMessage {
  factory UpsertConfigSchemaRequest({
    ConfigEditorSchema? schema,
    $core.String? updatedBy,
  }) {
    final result = create();
    if (schema != null) result.schema = schema;
    if (updatedBy != null) result.updatedBy = updatedBy;
    return result;
  }

  UpsertConfigSchemaRequest._();

  factory UpsertConfigSchemaRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory UpsertConfigSchemaRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'UpsertConfigSchemaRequest',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.config.v1'),
      createEmptyInstance: create)
    ..aOM<ConfigEditorSchema>(1, _omitFieldNames ? '' : 'schema',
        subBuilder: ConfigEditorSchema.create)
    ..aOS(2, _omitFieldNames ? '' : 'updatedBy')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  UpsertConfigSchemaRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  UpsertConfigSchemaRequest copyWith(
          void Function(UpsertConfigSchemaRequest) updates) =>
      super.copyWith((message) => updates(message as UpsertConfigSchemaRequest))
          as UpsertConfigSchemaRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static UpsertConfigSchemaRequest create() => UpsertConfigSchemaRequest._();
  @$core.override
  UpsertConfigSchemaRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static UpsertConfigSchemaRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<UpsertConfigSchemaRequest>(create);
  static UpsertConfigSchemaRequest? _defaultInstance;

  @$pb.TagNumber(1)
  ConfigEditorSchema get schema => $_getN(0);
  @$pb.TagNumber(1)
  set schema(ConfigEditorSchema value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasSchema() => $_has(0);
  @$pb.TagNumber(1)
  void clearSchema() => $_clearField(1);
  @$pb.TagNumber(1)
  ConfigEditorSchema ensureSchema() => $_ensure(0);

  @$pb.TagNumber(2)
  $core.String get updatedBy => $_getSZ(1);
  @$pb.TagNumber(2)
  set updatedBy($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasUpdatedBy() => $_has(1);
  @$pb.TagNumber(2)
  void clearUpdatedBy() => $_clearField(2);
}

/// UpsertConfigSchemaResponse は設定スキーマ作成・更新レスポンス。
class UpsertConfigSchemaResponse extends $pb.GeneratedMessage {
  factory UpsertConfigSchemaResponse({
    ConfigEditorSchema? schema,
  }) {
    final result = create();
    if (schema != null) result.schema = schema;
    return result;
  }

  UpsertConfigSchemaResponse._();

  factory UpsertConfigSchemaResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory UpsertConfigSchemaResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'UpsertConfigSchemaResponse',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.config.v1'),
      createEmptyInstance: create)
    ..aOM<ConfigEditorSchema>(1, _omitFieldNames ? '' : 'schema',
        subBuilder: ConfigEditorSchema.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  UpsertConfigSchemaResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  UpsertConfigSchemaResponse copyWith(
          void Function(UpsertConfigSchemaResponse) updates) =>
      super.copyWith(
              (message) => updates(message as UpsertConfigSchemaResponse))
          as UpsertConfigSchemaResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static UpsertConfigSchemaResponse create() => UpsertConfigSchemaResponse._();
  @$core.override
  UpsertConfigSchemaResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static UpsertConfigSchemaResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<UpsertConfigSchemaResponse>(create);
  static UpsertConfigSchemaResponse? _defaultInstance;

  @$pb.TagNumber(1)
  ConfigEditorSchema get schema => $_getN(0);
  @$pb.TagNumber(1)
  set schema(ConfigEditorSchema value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasSchema() => $_has(0);
  @$pb.TagNumber(1)
  void clearSchema() => $_clearField(1);
  @$pb.TagNumber(1)
  ConfigEditorSchema ensureSchema() => $_ensure(0);
}

/// ListConfigSchemasRequest は設定スキーマ一覧取得リクエスト。
class ListConfigSchemasRequest extends $pb.GeneratedMessage {
  factory ListConfigSchemasRequest() => create();

  ListConfigSchemasRequest._();

  factory ListConfigSchemasRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory ListConfigSchemasRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ListConfigSchemasRequest',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.config.v1'),
      createEmptyInstance: create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListConfigSchemasRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListConfigSchemasRequest copyWith(
          void Function(ListConfigSchemasRequest) updates) =>
      super.copyWith((message) => updates(message as ListConfigSchemasRequest))
          as ListConfigSchemasRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static ListConfigSchemasRequest create() => ListConfigSchemasRequest._();
  @$core.override
  ListConfigSchemasRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static ListConfigSchemasRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ListConfigSchemasRequest>(create);
  static ListConfigSchemasRequest? _defaultInstance;
}

/// ListConfigSchemasResponse は設定スキーマ一覧取得レスポンス。
class ListConfigSchemasResponse extends $pb.GeneratedMessage {
  factory ListConfigSchemasResponse({
    $core.Iterable<ConfigEditorSchema>? schemas,
  }) {
    final result = create();
    if (schemas != null) result.schemas.addAll(schemas);
    return result;
  }

  ListConfigSchemasResponse._();

  factory ListConfigSchemasResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory ListConfigSchemasResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ListConfigSchemasResponse',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.config.v1'),
      createEmptyInstance: create)
    ..pPM<ConfigEditorSchema>(1, _omitFieldNames ? '' : 'schemas',
        subBuilder: ConfigEditorSchema.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListConfigSchemasResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListConfigSchemasResponse copyWith(
          void Function(ListConfigSchemasResponse) updates) =>
      super.copyWith((message) => updates(message as ListConfigSchemasResponse))
          as ListConfigSchemasResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static ListConfigSchemasResponse create() => ListConfigSchemasResponse._();
  @$core.override
  ListConfigSchemasResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static ListConfigSchemasResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ListConfigSchemasResponse>(create);
  static ListConfigSchemasResponse? _defaultInstance;

  @$pb.TagNumber(1)
  $pb.PbList<ConfigEditorSchema> get schemas => $_getList(0);
}

const $core.bool _omitFieldNames =
    $core.bool.fromEnvironment('protobuf.omit_field_names');
const $core.bool _omitMessageNames =
    $core.bool.fromEnvironment('protobuf.omit_message_names');
