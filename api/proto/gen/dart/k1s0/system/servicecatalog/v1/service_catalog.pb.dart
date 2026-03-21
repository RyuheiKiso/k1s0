// This is a generated file - do not edit.
//
// Generated from k1s0/system/servicecatalog/v1/service_catalog.proto.

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

export 'package:protobuf/protobuf.dart' show GeneratedMessageGenericExtensions;

/// ServiceInfo はカタログに登録されたサービスの情報。
class ServiceInfo extends $pb.GeneratedMessage {
  factory ServiceInfo({
    $core.String? id,
    $core.String? name,
    $core.String? displayName,
    $core.String? description,
    $core.String? tier,
    $core.String? version,
    $core.String? baseUrl,
    $core.String? grpcEndpoint,
    $core.String? healthUrl,
    $core.String? status,
    $core.Iterable<$core.MapEntry<$core.String, $core.String>>? metadata,
    $1.Timestamp? createdAt,
    $1.Timestamp? updatedAt,
  }) {
    final result = create();
    if (id != null) result.id = id;
    if (name != null) result.name = name;
    if (displayName != null) result.displayName = displayName;
    if (description != null) result.description = description;
    if (tier != null) result.tier = tier;
    if (version != null) result.version = version;
    if (baseUrl != null) result.baseUrl = baseUrl;
    if (grpcEndpoint != null) result.grpcEndpoint = grpcEndpoint;
    if (healthUrl != null) result.healthUrl = healthUrl;
    if (status != null) result.status = status;
    if (metadata != null) result.metadata.addEntries(metadata);
    if (createdAt != null) result.createdAt = createdAt;
    if (updatedAt != null) result.updatedAt = updatedAt;
    return result;
  }

  ServiceInfo._();

  factory ServiceInfo.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory ServiceInfo.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ServiceInfo',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.servicecatalog.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'id')
    ..aOS(2, _omitFieldNames ? '' : 'name')
    ..aOS(3, _omitFieldNames ? '' : 'displayName')
    ..aOS(4, _omitFieldNames ? '' : 'description')
    ..aOS(5, _omitFieldNames ? '' : 'tier')
    ..aOS(6, _omitFieldNames ? '' : 'version')
    ..aOS(7, _omitFieldNames ? '' : 'baseUrl')
    ..aOS(8, _omitFieldNames ? '' : 'grpcEndpoint')
    ..aOS(9, _omitFieldNames ? '' : 'healthUrl')
    ..aOS(10, _omitFieldNames ? '' : 'status')
    ..m<$core.String, $core.String>(11, _omitFieldNames ? '' : 'metadata',
        entryClassName: 'ServiceInfo.MetadataEntry',
        keyFieldType: $pb.PbFieldType.OS,
        valueFieldType: $pb.PbFieldType.OS,
        packageName: const $pb.PackageName('k1s0.system.servicecatalog.v1'))
    ..aOM<$1.Timestamp>(12, _omitFieldNames ? '' : 'createdAt',
        subBuilder: $1.Timestamp.create)
    ..aOM<$1.Timestamp>(13, _omitFieldNames ? '' : 'updatedAt',
        subBuilder: $1.Timestamp.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ServiceInfo clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ServiceInfo copyWith(void Function(ServiceInfo) updates) =>
      super.copyWith((message) => updates(message as ServiceInfo))
          as ServiceInfo;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static ServiceInfo create() => ServiceInfo._();
  @$core.override
  ServiceInfo createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static ServiceInfo getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ServiceInfo>(create);
  static ServiceInfo? _defaultInstance;

  /// サービス UUID
  @$pb.TagNumber(1)
  $core.String get id => $_getSZ(0);
  @$pb.TagNumber(1)
  set id($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasId() => $_has(0);
  @$pb.TagNumber(1)
  void clearId() => $_clearField(1);

  /// サービス名（例: auth, tenant, order）
  @$pb.TagNumber(2)
  $core.String get name => $_getSZ(1);
  @$pb.TagNumber(2)
  set name($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasName() => $_has(1);
  @$pb.TagNumber(2)
  void clearName() => $_clearField(2);

  /// 表示名
  @$pb.TagNumber(3)
  $core.String get displayName => $_getSZ(2);
  @$pb.TagNumber(3)
  set displayName($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasDisplayName() => $_has(2);
  @$pb.TagNumber(3)
  void clearDisplayName() => $_clearField(3);

  /// サービスの説明
  @$pb.TagNumber(4)
  $core.String get description => $_getSZ(3);
  @$pb.TagNumber(4)
  set description($core.String value) => $_setString(3, value);
  @$pb.TagNumber(4)
  $core.bool hasDescription() => $_has(3);
  @$pb.TagNumber(4)
  void clearDescription() => $_clearField(4);

  /// Tier（system, business, service）
  @$pb.TagNumber(5)
  $core.String get tier => $_getSZ(4);
  @$pb.TagNumber(5)
  set tier($core.String value) => $_setString(4, value);
  @$pb.TagNumber(5)
  $core.bool hasTier() => $_has(4);
  @$pb.TagNumber(5)
  void clearTier() => $_clearField(5);

  /// バージョン（例: v1.2.3）
  @$pb.TagNumber(6)
  $core.String get version => $_getSZ(5);
  @$pb.TagNumber(6)
  set version($core.String value) => $_setString(5, value);
  @$pb.TagNumber(6)
  $core.bool hasVersion() => $_has(5);
  @$pb.TagNumber(6)
  void clearVersion() => $_clearField(6);

  /// ベース URL（例: http://auth-rust:8080）
  @$pb.TagNumber(7)
  $core.String get baseUrl => $_getSZ(6);
  @$pb.TagNumber(7)
  set baseUrl($core.String value) => $_setString(6, value);
  @$pb.TagNumber(7)
  $core.bool hasBaseUrl() => $_has(6);
  @$pb.TagNumber(7)
  void clearBaseUrl() => $_clearField(7);

  /// gRPC エンドポイント（例: auth-rust:50051）
  @$pb.TagNumber(8)
  $core.String get grpcEndpoint => $_getSZ(7);
  @$pb.TagNumber(8)
  set grpcEndpoint($core.String value) => $_setString(7, value);
  @$pb.TagNumber(8)
  $core.bool hasGrpcEndpoint() => $_has(7);
  @$pb.TagNumber(8)
  void clearGrpcEndpoint() => $_clearField(8);

  /// ヘルスチェック URL（例: http://auth-rust:8080/healthz）
  @$pb.TagNumber(9)
  $core.String get healthUrl => $_getSZ(8);
  @$pb.TagNumber(9)
  set healthUrl($core.String value) => $_setString(8, value);
  @$pb.TagNumber(9)
  $core.bool hasHealthUrl() => $_has(8);
  @$pb.TagNumber(9)
  void clearHealthUrl() => $_clearField(9);

  /// サービスの現在のステータス（HEALTHY, UNHEALTHY, UNKNOWN）
  @$pb.TagNumber(10)
  $core.String get status => $_getSZ(9);
  @$pb.TagNumber(10)
  set status($core.String value) => $_setString(9, value);
  @$pb.TagNumber(10)
  $core.bool hasStatus() => $_has(9);
  @$pb.TagNumber(10)
  void clearStatus() => $_clearField(10);

  /// メタデータ（任意の key-value）
  @$pb.TagNumber(11)
  $pb.PbMap<$core.String, $core.String> get metadata => $_getMap(10);

  /// 登録日時
  @$pb.TagNumber(12)
  $1.Timestamp get createdAt => $_getN(11);
  @$pb.TagNumber(12)
  set createdAt($1.Timestamp value) => $_setField(12, value);
  @$pb.TagNumber(12)
  $core.bool hasCreatedAt() => $_has(11);
  @$pb.TagNumber(12)
  void clearCreatedAt() => $_clearField(12);
  @$pb.TagNumber(12)
  $1.Timestamp ensureCreatedAt() => $_ensure(11);

  /// 更新日時
  @$pb.TagNumber(13)
  $1.Timestamp get updatedAt => $_getN(12);
  @$pb.TagNumber(13)
  set updatedAt($1.Timestamp value) => $_setField(13, value);
  @$pb.TagNumber(13)
  $core.bool hasUpdatedAt() => $_has(12);
  @$pb.TagNumber(13)
  void clearUpdatedAt() => $_clearField(13);
  @$pb.TagNumber(13)
  $1.Timestamp ensureUpdatedAt() => $_ensure(12);
}

/// RegisterServiceRequest はサービス登録リクエスト。
class RegisterServiceRequest extends $pb.GeneratedMessage {
  factory RegisterServiceRequest({
    $core.String? name,
    $core.String? displayName,
    $core.String? description,
    $core.String? tier,
    $core.String? version,
    $core.String? baseUrl,
    $core.String? grpcEndpoint,
    $core.String? healthUrl,
    $core.Iterable<$core.MapEntry<$core.String, $core.String>>? metadata,
  }) {
    final result = create();
    if (name != null) result.name = name;
    if (displayName != null) result.displayName = displayName;
    if (description != null) result.description = description;
    if (tier != null) result.tier = tier;
    if (version != null) result.version = version;
    if (baseUrl != null) result.baseUrl = baseUrl;
    if (grpcEndpoint != null) result.grpcEndpoint = grpcEndpoint;
    if (healthUrl != null) result.healthUrl = healthUrl;
    if (metadata != null) result.metadata.addEntries(metadata);
    return result;
  }

  RegisterServiceRequest._();

  factory RegisterServiceRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory RegisterServiceRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'RegisterServiceRequest',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.servicecatalog.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'name')
    ..aOS(2, _omitFieldNames ? '' : 'displayName')
    ..aOS(3, _omitFieldNames ? '' : 'description')
    ..aOS(4, _omitFieldNames ? '' : 'tier')
    ..aOS(5, _omitFieldNames ? '' : 'version')
    ..aOS(6, _omitFieldNames ? '' : 'baseUrl')
    ..aOS(7, _omitFieldNames ? '' : 'grpcEndpoint')
    ..aOS(8, _omitFieldNames ? '' : 'healthUrl')
    ..m<$core.String, $core.String>(9, _omitFieldNames ? '' : 'metadata',
        entryClassName: 'RegisterServiceRequest.MetadataEntry',
        keyFieldType: $pb.PbFieldType.OS,
        valueFieldType: $pb.PbFieldType.OS,
        packageName: const $pb.PackageName('k1s0.system.servicecatalog.v1'))
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  RegisterServiceRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  RegisterServiceRequest copyWith(
          void Function(RegisterServiceRequest) updates) =>
      super.copyWith((message) => updates(message as RegisterServiceRequest))
          as RegisterServiceRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static RegisterServiceRequest create() => RegisterServiceRequest._();
  @$core.override
  RegisterServiceRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static RegisterServiceRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<RegisterServiceRequest>(create);
  static RegisterServiceRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get name => $_getSZ(0);
  @$pb.TagNumber(1)
  set name($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasName() => $_has(0);
  @$pb.TagNumber(1)
  void clearName() => $_clearField(1);

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
  $core.String get tier => $_getSZ(3);
  @$pb.TagNumber(4)
  set tier($core.String value) => $_setString(3, value);
  @$pb.TagNumber(4)
  $core.bool hasTier() => $_has(3);
  @$pb.TagNumber(4)
  void clearTier() => $_clearField(4);

  @$pb.TagNumber(5)
  $core.String get version => $_getSZ(4);
  @$pb.TagNumber(5)
  set version($core.String value) => $_setString(4, value);
  @$pb.TagNumber(5)
  $core.bool hasVersion() => $_has(4);
  @$pb.TagNumber(5)
  void clearVersion() => $_clearField(5);

  @$pb.TagNumber(6)
  $core.String get baseUrl => $_getSZ(5);
  @$pb.TagNumber(6)
  set baseUrl($core.String value) => $_setString(5, value);
  @$pb.TagNumber(6)
  $core.bool hasBaseUrl() => $_has(5);
  @$pb.TagNumber(6)
  void clearBaseUrl() => $_clearField(6);

  @$pb.TagNumber(7)
  $core.String get grpcEndpoint => $_getSZ(6);
  @$pb.TagNumber(7)
  set grpcEndpoint($core.String value) => $_setString(6, value);
  @$pb.TagNumber(7)
  $core.bool hasGrpcEndpoint() => $_has(6);
  @$pb.TagNumber(7)
  void clearGrpcEndpoint() => $_clearField(7);

  @$pb.TagNumber(8)
  $core.String get healthUrl => $_getSZ(7);
  @$pb.TagNumber(8)
  set healthUrl($core.String value) => $_setString(7, value);
  @$pb.TagNumber(8)
  $core.bool hasHealthUrl() => $_has(7);
  @$pb.TagNumber(8)
  void clearHealthUrl() => $_clearField(8);

  @$pb.TagNumber(9)
  $pb.PbMap<$core.String, $core.String> get metadata => $_getMap(8);
}

/// RegisterServiceResponse はサービス登録レスポンス。
class RegisterServiceResponse extends $pb.GeneratedMessage {
  factory RegisterServiceResponse({
    ServiceInfo? service,
  }) {
    final result = create();
    if (service != null) result.service = service;
    return result;
  }

  RegisterServiceResponse._();

  factory RegisterServiceResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory RegisterServiceResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'RegisterServiceResponse',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.servicecatalog.v1'),
      createEmptyInstance: create)
    ..aOM<ServiceInfo>(1, _omitFieldNames ? '' : 'service',
        subBuilder: ServiceInfo.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  RegisterServiceResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  RegisterServiceResponse copyWith(
          void Function(RegisterServiceResponse) updates) =>
      super.copyWith((message) => updates(message as RegisterServiceResponse))
          as RegisterServiceResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static RegisterServiceResponse create() => RegisterServiceResponse._();
  @$core.override
  RegisterServiceResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static RegisterServiceResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<RegisterServiceResponse>(create);
  static RegisterServiceResponse? _defaultInstance;

  @$pb.TagNumber(1)
  ServiceInfo get service => $_getN(0);
  @$pb.TagNumber(1)
  set service(ServiceInfo value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasService() => $_has(0);
  @$pb.TagNumber(1)
  void clearService() => $_clearField(1);
  @$pb.TagNumber(1)
  ServiceInfo ensureService() => $_ensure(0);
}

/// GetServiceRequest はサービス取得リクエスト。
class GetServiceRequest extends $pb.GeneratedMessage {
  factory GetServiceRequest({
    $core.String? serviceId,
  }) {
    final result = create();
    if (serviceId != null) result.serviceId = serviceId;
    return result;
  }

  GetServiceRequest._();

  factory GetServiceRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory GetServiceRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'GetServiceRequest',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.servicecatalog.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'serviceId')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetServiceRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetServiceRequest copyWith(void Function(GetServiceRequest) updates) =>
      super.copyWith((message) => updates(message as GetServiceRequest))
          as GetServiceRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static GetServiceRequest create() => GetServiceRequest._();
  @$core.override
  GetServiceRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static GetServiceRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<GetServiceRequest>(create);
  static GetServiceRequest? _defaultInstance;

  /// サービス UUID または名前
  @$pb.TagNumber(1)
  $core.String get serviceId => $_getSZ(0);
  @$pb.TagNumber(1)
  set serviceId($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasServiceId() => $_has(0);
  @$pb.TagNumber(1)
  void clearServiceId() => $_clearField(1);
}

/// GetServiceResponse はサービス取得レスポンス。
class GetServiceResponse extends $pb.GeneratedMessage {
  factory GetServiceResponse({
    ServiceInfo? service,
  }) {
    final result = create();
    if (service != null) result.service = service;
    return result;
  }

  GetServiceResponse._();

  factory GetServiceResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory GetServiceResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'GetServiceResponse',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.servicecatalog.v1'),
      createEmptyInstance: create)
    ..aOM<ServiceInfo>(1, _omitFieldNames ? '' : 'service',
        subBuilder: ServiceInfo.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetServiceResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetServiceResponse copyWith(void Function(GetServiceResponse) updates) =>
      super.copyWith((message) => updates(message as GetServiceResponse))
          as GetServiceResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static GetServiceResponse create() => GetServiceResponse._();
  @$core.override
  GetServiceResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static GetServiceResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<GetServiceResponse>(create);
  static GetServiceResponse? _defaultInstance;

  @$pb.TagNumber(1)
  ServiceInfo get service => $_getN(0);
  @$pb.TagNumber(1)
  set service(ServiceInfo value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasService() => $_has(0);
  @$pb.TagNumber(1)
  void clearService() => $_clearField(1);
  @$pb.TagNumber(1)
  ServiceInfo ensureService() => $_ensure(0);
}

/// ListServicesRequest はサービス一覧取得リクエスト。
class ListServicesRequest extends $pb.GeneratedMessage {
  factory ListServicesRequest({
    $1.Pagination? pagination,
    $core.String? tier,
    $core.String? status,
    $core.String? search,
  }) {
    final result = create();
    if (pagination != null) result.pagination = pagination;
    if (tier != null) result.tier = tier;
    if (status != null) result.status = status;
    if (search != null) result.search = search;
    return result;
  }

  ListServicesRequest._();

  factory ListServicesRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory ListServicesRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ListServicesRequest',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.servicecatalog.v1'),
      createEmptyInstance: create)
    ..aOM<$1.Pagination>(1, _omitFieldNames ? '' : 'pagination',
        subBuilder: $1.Pagination.create)
    ..aOS(2, _omitFieldNames ? '' : 'tier')
    ..aOS(3, _omitFieldNames ? '' : 'status')
    ..aOS(4, _omitFieldNames ? '' : 'search')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListServicesRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListServicesRequest copyWith(void Function(ListServicesRequest) updates) =>
      super.copyWith((message) => updates(message as ListServicesRequest))
          as ListServicesRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static ListServicesRequest create() => ListServicesRequest._();
  @$core.override
  ListServicesRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static ListServicesRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ListServicesRequest>(create);
  static ListServicesRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $1.Pagination get pagination => $_getN(0);
  @$pb.TagNumber(1)
  set pagination($1.Pagination value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasPagination() => $_has(0);
  @$pb.TagNumber(1)
  void clearPagination() => $_clearField(1);
  @$pb.TagNumber(1)
  $1.Pagination ensurePagination() => $_ensure(0);

  /// Tier フィルタ（system, business, service）
  @$pb.TagNumber(2)
  $core.String get tier => $_getSZ(1);
  @$pb.TagNumber(2)
  set tier($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasTier() => $_has(1);
  @$pb.TagNumber(2)
  void clearTier() => $_clearField(2);

  /// ステータスフィルタ（HEALTHY, UNHEALTHY, UNKNOWN）
  @$pb.TagNumber(3)
  $core.String get status => $_getSZ(2);
  @$pb.TagNumber(3)
  set status($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasStatus() => $_has(2);
  @$pb.TagNumber(3)
  void clearStatus() => $_clearField(3);

  /// 名前・表示名で部分一致検索
  @$pb.TagNumber(4)
  $core.String get search => $_getSZ(3);
  @$pb.TagNumber(4)
  set search($core.String value) => $_setString(3, value);
  @$pb.TagNumber(4)
  $core.bool hasSearch() => $_has(3);
  @$pb.TagNumber(4)
  void clearSearch() => $_clearField(4);
}

/// ListServicesResponse はサービス一覧取得レスポンス。
class ListServicesResponse extends $pb.GeneratedMessage {
  factory ListServicesResponse({
    $core.Iterable<ServiceInfo>? services,
    $1.PaginationResult? pagination,
  }) {
    final result = create();
    if (services != null) result.services.addAll(services);
    if (pagination != null) result.pagination = pagination;
    return result;
  }

  ListServicesResponse._();

  factory ListServicesResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory ListServicesResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ListServicesResponse',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.servicecatalog.v1'),
      createEmptyInstance: create)
    ..pPM<ServiceInfo>(1, _omitFieldNames ? '' : 'services',
        subBuilder: ServiceInfo.create)
    ..aOM<$1.PaginationResult>(2, _omitFieldNames ? '' : 'pagination',
        subBuilder: $1.PaginationResult.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListServicesResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListServicesResponse copyWith(void Function(ListServicesResponse) updates) =>
      super.copyWith((message) => updates(message as ListServicesResponse))
          as ListServicesResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static ListServicesResponse create() => ListServicesResponse._();
  @$core.override
  ListServicesResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static ListServicesResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ListServicesResponse>(create);
  static ListServicesResponse? _defaultInstance;

  @$pb.TagNumber(1)
  $pb.PbList<ServiceInfo> get services => $_getList(0);

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

/// UpdateServiceRequest はサービス更新リクエスト。
class UpdateServiceRequest extends $pb.GeneratedMessage {
  factory UpdateServiceRequest({
    $core.String? serviceId,
    $core.String? displayName,
    $core.String? description,
    $core.String? version,
    $core.String? baseUrl,
    $core.String? grpcEndpoint,
    $core.String? healthUrl,
    $core.Iterable<$core.MapEntry<$core.String, $core.String>>? metadata,
  }) {
    final result = create();
    if (serviceId != null) result.serviceId = serviceId;
    if (displayName != null) result.displayName = displayName;
    if (description != null) result.description = description;
    if (version != null) result.version = version;
    if (baseUrl != null) result.baseUrl = baseUrl;
    if (grpcEndpoint != null) result.grpcEndpoint = grpcEndpoint;
    if (healthUrl != null) result.healthUrl = healthUrl;
    if (metadata != null) result.metadata.addEntries(metadata);
    return result;
  }

  UpdateServiceRequest._();

  factory UpdateServiceRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory UpdateServiceRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'UpdateServiceRequest',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.servicecatalog.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'serviceId')
    ..aOS(2, _omitFieldNames ? '' : 'displayName')
    ..aOS(3, _omitFieldNames ? '' : 'description')
    ..aOS(4, _omitFieldNames ? '' : 'version')
    ..aOS(5, _omitFieldNames ? '' : 'baseUrl')
    ..aOS(6, _omitFieldNames ? '' : 'grpcEndpoint')
    ..aOS(7, _omitFieldNames ? '' : 'healthUrl')
    ..m<$core.String, $core.String>(8, _omitFieldNames ? '' : 'metadata',
        entryClassName: 'UpdateServiceRequest.MetadataEntry',
        keyFieldType: $pb.PbFieldType.OS,
        valueFieldType: $pb.PbFieldType.OS,
        packageName: const $pb.PackageName('k1s0.system.servicecatalog.v1'))
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  UpdateServiceRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  UpdateServiceRequest copyWith(void Function(UpdateServiceRequest) updates) =>
      super.copyWith((message) => updates(message as UpdateServiceRequest))
          as UpdateServiceRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static UpdateServiceRequest create() => UpdateServiceRequest._();
  @$core.override
  UpdateServiceRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static UpdateServiceRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<UpdateServiceRequest>(create);
  static UpdateServiceRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get serviceId => $_getSZ(0);
  @$pb.TagNumber(1)
  set serviceId($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasServiceId() => $_has(0);
  @$pb.TagNumber(1)
  void clearServiceId() => $_clearField(1);

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
  $core.String get version => $_getSZ(3);
  @$pb.TagNumber(4)
  set version($core.String value) => $_setString(3, value);
  @$pb.TagNumber(4)
  $core.bool hasVersion() => $_has(3);
  @$pb.TagNumber(4)
  void clearVersion() => $_clearField(4);

  @$pb.TagNumber(5)
  $core.String get baseUrl => $_getSZ(4);
  @$pb.TagNumber(5)
  set baseUrl($core.String value) => $_setString(4, value);
  @$pb.TagNumber(5)
  $core.bool hasBaseUrl() => $_has(4);
  @$pb.TagNumber(5)
  void clearBaseUrl() => $_clearField(5);

  @$pb.TagNumber(6)
  $core.String get grpcEndpoint => $_getSZ(5);
  @$pb.TagNumber(6)
  set grpcEndpoint($core.String value) => $_setString(5, value);
  @$pb.TagNumber(6)
  $core.bool hasGrpcEndpoint() => $_has(5);
  @$pb.TagNumber(6)
  void clearGrpcEndpoint() => $_clearField(6);

  @$pb.TagNumber(7)
  $core.String get healthUrl => $_getSZ(6);
  @$pb.TagNumber(7)
  set healthUrl($core.String value) => $_setString(6, value);
  @$pb.TagNumber(7)
  $core.bool hasHealthUrl() => $_has(6);
  @$pb.TagNumber(7)
  void clearHealthUrl() => $_clearField(7);

  @$pb.TagNumber(8)
  $pb.PbMap<$core.String, $core.String> get metadata => $_getMap(7);
}

/// UpdateServiceResponse はサービス更新レスポンス。
class UpdateServiceResponse extends $pb.GeneratedMessage {
  factory UpdateServiceResponse({
    ServiceInfo? service,
  }) {
    final result = create();
    if (service != null) result.service = service;
    return result;
  }

  UpdateServiceResponse._();

  factory UpdateServiceResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory UpdateServiceResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'UpdateServiceResponse',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.servicecatalog.v1'),
      createEmptyInstance: create)
    ..aOM<ServiceInfo>(1, _omitFieldNames ? '' : 'service',
        subBuilder: ServiceInfo.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  UpdateServiceResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  UpdateServiceResponse copyWith(
          void Function(UpdateServiceResponse) updates) =>
      super.copyWith((message) => updates(message as UpdateServiceResponse))
          as UpdateServiceResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static UpdateServiceResponse create() => UpdateServiceResponse._();
  @$core.override
  UpdateServiceResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static UpdateServiceResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<UpdateServiceResponse>(create);
  static UpdateServiceResponse? _defaultInstance;

  @$pb.TagNumber(1)
  ServiceInfo get service => $_getN(0);
  @$pb.TagNumber(1)
  set service(ServiceInfo value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasService() => $_has(0);
  @$pb.TagNumber(1)
  void clearService() => $_clearField(1);
  @$pb.TagNumber(1)
  ServiceInfo ensureService() => $_ensure(0);
}

/// DeleteServiceRequest はサービス削除リクエスト。
class DeleteServiceRequest extends $pb.GeneratedMessage {
  factory DeleteServiceRequest({
    $core.String? serviceId,
  }) {
    final result = create();
    if (serviceId != null) result.serviceId = serviceId;
    return result;
  }

  DeleteServiceRequest._();

  factory DeleteServiceRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory DeleteServiceRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'DeleteServiceRequest',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.servicecatalog.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'serviceId')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  DeleteServiceRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  DeleteServiceRequest copyWith(void Function(DeleteServiceRequest) updates) =>
      super.copyWith((message) => updates(message as DeleteServiceRequest))
          as DeleteServiceRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static DeleteServiceRequest create() => DeleteServiceRequest._();
  @$core.override
  DeleteServiceRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static DeleteServiceRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<DeleteServiceRequest>(create);
  static DeleteServiceRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get serviceId => $_getSZ(0);
  @$pb.TagNumber(1)
  set serviceId($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasServiceId() => $_has(0);
  @$pb.TagNumber(1)
  void clearServiceId() => $_clearField(1);
}

/// DeleteServiceResponse はサービス削除レスポンス。
class DeleteServiceResponse extends $pb.GeneratedMessage {
  factory DeleteServiceResponse({
    $core.bool? success,
  }) {
    final result = create();
    if (success != null) result.success = success;
    return result;
  }

  DeleteServiceResponse._();

  factory DeleteServiceResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory DeleteServiceResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'DeleteServiceResponse',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.servicecatalog.v1'),
      createEmptyInstance: create)
    ..aOB(1, _omitFieldNames ? '' : 'success')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  DeleteServiceResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  DeleteServiceResponse copyWith(
          void Function(DeleteServiceResponse) updates) =>
      super.copyWith((message) => updates(message as DeleteServiceResponse))
          as DeleteServiceResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static DeleteServiceResponse create() => DeleteServiceResponse._();
  @$core.override
  DeleteServiceResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static DeleteServiceResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<DeleteServiceResponse>(create);
  static DeleteServiceResponse? _defaultInstance;

  /// 削除成功
  @$pb.TagNumber(1)
  $core.bool get success => $_getBF(0);
  @$pb.TagNumber(1)
  set success($core.bool value) => $_setBool(0, value);
  @$pb.TagNumber(1)
  $core.bool hasSuccess() => $_has(0);
  @$pb.TagNumber(1)
  void clearSuccess() => $_clearField(1);
}

/// HealthCheckRequest はヘルスチェックリクエスト。
class HealthCheckRequest extends $pb.GeneratedMessage {
  factory HealthCheckRequest({
    $core.String? serviceId,
  }) {
    final result = create();
    if (serviceId != null) result.serviceId = serviceId;
    return result;
  }

  HealthCheckRequest._();

  factory HealthCheckRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory HealthCheckRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'HealthCheckRequest',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.servicecatalog.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'serviceId')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  HealthCheckRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  HealthCheckRequest copyWith(void Function(HealthCheckRequest) updates) =>
      super.copyWith((message) => updates(message as HealthCheckRequest))
          as HealthCheckRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static HealthCheckRequest create() => HealthCheckRequest._();
  @$core.override
  HealthCheckRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static HealthCheckRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<HealthCheckRequest>(create);
  static HealthCheckRequest? _defaultInstance;

  /// 空の場合は全サービスのヘルスを返す。指定した場合はそのサービスのみ。
  @$pb.TagNumber(1)
  $core.String get serviceId => $_getSZ(0);
  @$pb.TagNumber(1)
  set serviceId($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasServiceId() => $_has(0);
  @$pb.TagNumber(1)
  void clearServiceId() => $_clearField(1);
}

/// HealthCheckResponse はヘルスチェックレスポンス。
class HealthCheckResponse extends $pb.GeneratedMessage {
  factory HealthCheckResponse({
    $core.Iterable<ServiceHealth>? services,
  }) {
    final result = create();
    if (services != null) result.services.addAll(services);
    return result;
  }

  HealthCheckResponse._();

  factory HealthCheckResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory HealthCheckResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'HealthCheckResponse',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.servicecatalog.v1'),
      createEmptyInstance: create)
    ..pPM<ServiceHealth>(1, _omitFieldNames ? '' : 'services',
        subBuilder: ServiceHealth.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  HealthCheckResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  HealthCheckResponse copyWith(void Function(HealthCheckResponse) updates) =>
      super.copyWith((message) => updates(message as HealthCheckResponse))
          as HealthCheckResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static HealthCheckResponse create() => HealthCheckResponse._();
  @$core.override
  HealthCheckResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static HealthCheckResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<HealthCheckResponse>(create);
  static HealthCheckResponse? _defaultInstance;

  @$pb.TagNumber(1)
  $pb.PbList<ServiceHealth> get services => $_getList(0);
}

/// ServiceHealth は個別サービスのヘルスチェック結果。
class ServiceHealth extends $pb.GeneratedMessage {
  factory ServiceHealth({
    $core.String? serviceId,
    $core.String? serviceName,
    $core.String? status,
    $fixnum.Int64? responseTimeMs,
    $core.String? errorMessage,
    $1.Timestamp? checkedAt,
  }) {
    final result = create();
    if (serviceId != null) result.serviceId = serviceId;
    if (serviceName != null) result.serviceName = serviceName;
    if (status != null) result.status = status;
    if (responseTimeMs != null) result.responseTimeMs = responseTimeMs;
    if (errorMessage != null) result.errorMessage = errorMessage;
    if (checkedAt != null) result.checkedAt = checkedAt;
    return result;
  }

  ServiceHealth._();

  factory ServiceHealth.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory ServiceHealth.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ServiceHealth',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.servicecatalog.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'serviceId')
    ..aOS(2, _omitFieldNames ? '' : 'serviceName')
    ..aOS(3, _omitFieldNames ? '' : 'status')
    ..aInt64(4, _omitFieldNames ? '' : 'responseTimeMs')
    ..aOS(5, _omitFieldNames ? '' : 'errorMessage')
    ..aOM<$1.Timestamp>(6, _omitFieldNames ? '' : 'checkedAt',
        subBuilder: $1.Timestamp.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ServiceHealth clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ServiceHealth copyWith(void Function(ServiceHealth) updates) =>
      super.copyWith((message) => updates(message as ServiceHealth))
          as ServiceHealth;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static ServiceHealth create() => ServiceHealth._();
  @$core.override
  ServiceHealth createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static ServiceHealth getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ServiceHealth>(create);
  static ServiceHealth? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get serviceId => $_getSZ(0);
  @$pb.TagNumber(1)
  set serviceId($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasServiceId() => $_has(0);
  @$pb.TagNumber(1)
  void clearServiceId() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get serviceName => $_getSZ(1);
  @$pb.TagNumber(2)
  set serviceName($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasServiceName() => $_has(1);
  @$pb.TagNumber(2)
  void clearServiceName() => $_clearField(2);

  /// HEALTHY, UNHEALTHY, UNKNOWN
  @$pb.TagNumber(3)
  $core.String get status => $_getSZ(2);
  @$pb.TagNumber(3)
  set status($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasStatus() => $_has(2);
  @$pb.TagNumber(3)
  void clearStatus() => $_clearField(3);

  /// レスポンスタイム（ミリ秒）
  @$pb.TagNumber(4)
  $fixnum.Int64 get responseTimeMs => $_getI64(3);
  @$pb.TagNumber(4)
  set responseTimeMs($fixnum.Int64 value) => $_setInt64(3, value);
  @$pb.TagNumber(4)
  $core.bool hasResponseTimeMs() => $_has(3);
  @$pb.TagNumber(4)
  void clearResponseTimeMs() => $_clearField(4);

  /// エラーメッセージ（UNHEALTHY の場合）
  @$pb.TagNumber(5)
  $core.String get errorMessage => $_getSZ(4);
  @$pb.TagNumber(5)
  set errorMessage($core.String value) => $_setString(4, value);
  @$pb.TagNumber(5)
  $core.bool hasErrorMessage() => $_has(4);
  @$pb.TagNumber(5)
  void clearErrorMessage() => $_clearField(5);

  /// チェック実行日時
  @$pb.TagNumber(6)
  $1.Timestamp get checkedAt => $_getN(5);
  @$pb.TagNumber(6)
  set checkedAt($1.Timestamp value) => $_setField(6, value);
  @$pb.TagNumber(6)
  $core.bool hasCheckedAt() => $_has(5);
  @$pb.TagNumber(6)
  void clearCheckedAt() => $_clearField(6);
  @$pb.TagNumber(6)
  $1.Timestamp ensureCheckedAt() => $_ensure(5);
}

const $core.bool _omitFieldNames =
    $core.bool.fromEnvironment('protobuf.omit_field_names');
const $core.bool _omitMessageNames =
    $core.bool.fromEnvironment('protobuf.omit_message_names');
