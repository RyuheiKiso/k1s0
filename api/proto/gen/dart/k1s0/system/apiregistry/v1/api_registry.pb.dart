// This is a generated file - do not edit.
//
// Generated from k1s0/system/apiregistry/v1/api_registry.proto.

// @dart = 3.3

// ignore_for_file: annotate_overrides, camel_case_types, comment_references
// ignore_for_file: constant_identifier_names
// ignore_for_file: curly_braces_in_flow_control_structures
// ignore_for_file: deprecated_member_use_from_same_package, library_prefixes
// ignore_for_file: non_constant_identifier_names, prefer_relative_imports

import 'dart:core' as $core;

import 'package:protobuf/protobuf.dart' as $pb;

import '../../common/v1/types.pb.dart' as $1;

export 'package:protobuf/protobuf.dart' show GeneratedMessageGenericExtensions;

class GetSchemaRequest extends $pb.GeneratedMessage {
  factory GetSchemaRequest({
    $core.String? name,
  }) {
    final result = create();
    if (name != null) result.name = name;
    return result;
  }

  GetSchemaRequest._();

  factory GetSchemaRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory GetSchemaRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'GetSchemaRequest',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.apiregistry.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'name')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetSchemaRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetSchemaRequest copyWith(void Function(GetSchemaRequest) updates) =>
      super.copyWith((message) => updates(message as GetSchemaRequest))
          as GetSchemaRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static GetSchemaRequest create() => GetSchemaRequest._();
  @$core.override
  GetSchemaRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static GetSchemaRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<GetSchemaRequest>(create);
  static GetSchemaRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get name => $_getSZ(0);
  @$pb.TagNumber(1)
  set name($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasName() => $_has(0);
  @$pb.TagNumber(1)
  void clearName() => $_clearField(1);
}

class ListSchemasRequest extends $pb.GeneratedMessage {
  factory ListSchemasRequest({
    $core.String? schemaType,
    $1.Pagination? pagination,
  }) {
    final result = create();
    if (schemaType != null) result.schemaType = schemaType;
    if (pagination != null) result.pagination = pagination;
    return result;
  }

  ListSchemasRequest._();

  factory ListSchemasRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory ListSchemasRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ListSchemasRequest',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.apiregistry.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'schemaType')
    ..aOM<$1.Pagination>(2, _omitFieldNames ? '' : 'pagination',
        subBuilder: $1.Pagination.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListSchemasRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListSchemasRequest copyWith(void Function(ListSchemasRequest) updates) =>
      super.copyWith((message) => updates(message as ListSchemasRequest))
          as ListSchemasRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static ListSchemasRequest create() => ListSchemasRequest._();
  @$core.override
  ListSchemasRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static ListSchemasRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ListSchemasRequest>(create);
  static ListSchemasRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get schemaType => $_getSZ(0);
  @$pb.TagNumber(1)
  set schemaType($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasSchemaType() => $_has(0);
  @$pb.TagNumber(1)
  void clearSchemaType() => $_clearField(1);

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
}

class ListSchemasResponse extends $pb.GeneratedMessage {
  factory ListSchemasResponse({
    $core.Iterable<ApiSchemaProto>? schemas,
    $1.PaginationResult? pagination,
  }) {
    final result = create();
    if (schemas != null) result.schemas.addAll(schemas);
    if (pagination != null) result.pagination = pagination;
    return result;
  }

  ListSchemasResponse._();

  factory ListSchemasResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory ListSchemasResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ListSchemasResponse',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.apiregistry.v1'),
      createEmptyInstance: create)
    ..pPM<ApiSchemaProto>(1, _omitFieldNames ? '' : 'schemas',
        subBuilder: ApiSchemaProto.create)
    ..aOM<$1.PaginationResult>(2, _omitFieldNames ? '' : 'pagination',
        subBuilder: $1.PaginationResult.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListSchemasResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListSchemasResponse copyWith(void Function(ListSchemasResponse) updates) =>
      super.copyWith((message) => updates(message as ListSchemasResponse))
          as ListSchemasResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static ListSchemasResponse create() => ListSchemasResponse._();
  @$core.override
  ListSchemasResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static ListSchemasResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ListSchemasResponse>(create);
  static ListSchemasResponse? _defaultInstance;

  @$pb.TagNumber(1)
  $pb.PbList<ApiSchemaProto> get schemas => $_getList(0);

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

class RegisterSchemaRequest extends $pb.GeneratedMessage {
  factory RegisterSchemaRequest({
    $core.String? name,
    $core.String? description,
    $core.String? schemaType,
    $core.String? content,
    $core.String? registeredBy,
  }) {
    final result = create();
    if (name != null) result.name = name;
    if (description != null) result.description = description;
    if (schemaType != null) result.schemaType = schemaType;
    if (content != null) result.content = content;
    if (registeredBy != null) result.registeredBy = registeredBy;
    return result;
  }

  RegisterSchemaRequest._();

  factory RegisterSchemaRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory RegisterSchemaRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'RegisterSchemaRequest',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.apiregistry.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'name')
    ..aOS(2, _omitFieldNames ? '' : 'description')
    ..aOS(3, _omitFieldNames ? '' : 'schemaType')
    ..aOS(4, _omitFieldNames ? '' : 'content')
    ..aOS(5, _omitFieldNames ? '' : 'registeredBy')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  RegisterSchemaRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  RegisterSchemaRequest copyWith(
          void Function(RegisterSchemaRequest) updates) =>
      super.copyWith((message) => updates(message as RegisterSchemaRequest))
          as RegisterSchemaRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static RegisterSchemaRequest create() => RegisterSchemaRequest._();
  @$core.override
  RegisterSchemaRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static RegisterSchemaRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<RegisterSchemaRequest>(create);
  static RegisterSchemaRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get name => $_getSZ(0);
  @$pb.TagNumber(1)
  set name($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasName() => $_has(0);
  @$pb.TagNumber(1)
  void clearName() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get description => $_getSZ(1);
  @$pb.TagNumber(2)
  set description($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasDescription() => $_has(1);
  @$pb.TagNumber(2)
  void clearDescription() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.String get schemaType => $_getSZ(2);
  @$pb.TagNumber(3)
  set schemaType($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasSchemaType() => $_has(2);
  @$pb.TagNumber(3)
  void clearSchemaType() => $_clearField(3);

  @$pb.TagNumber(4)
  $core.String get content => $_getSZ(3);
  @$pb.TagNumber(4)
  set content($core.String value) => $_setString(3, value);
  @$pb.TagNumber(4)
  $core.bool hasContent() => $_has(3);
  @$pb.TagNumber(4)
  void clearContent() => $_clearField(4);

  @$pb.TagNumber(5)
  $core.String get registeredBy => $_getSZ(4);
  @$pb.TagNumber(5)
  set registeredBy($core.String value) => $_setString(4, value);
  @$pb.TagNumber(5)
  $core.bool hasRegisteredBy() => $_has(4);
  @$pb.TagNumber(5)
  void clearRegisteredBy() => $_clearField(5);
}

class RegisterSchemaResponse extends $pb.GeneratedMessage {
  factory RegisterSchemaResponse({
    ApiSchemaVersionProto? version,
  }) {
    final result = create();
    if (version != null) result.version = version;
    return result;
  }

  RegisterSchemaResponse._();

  factory RegisterSchemaResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory RegisterSchemaResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'RegisterSchemaResponse',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.apiregistry.v1'),
      createEmptyInstance: create)
    ..aOM<ApiSchemaVersionProto>(1, _omitFieldNames ? '' : 'version',
        subBuilder: ApiSchemaVersionProto.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  RegisterSchemaResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  RegisterSchemaResponse copyWith(
          void Function(RegisterSchemaResponse) updates) =>
      super.copyWith((message) => updates(message as RegisterSchemaResponse))
          as RegisterSchemaResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static RegisterSchemaResponse create() => RegisterSchemaResponse._();
  @$core.override
  RegisterSchemaResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static RegisterSchemaResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<RegisterSchemaResponse>(create);
  static RegisterSchemaResponse? _defaultInstance;

  @$pb.TagNumber(1)
  ApiSchemaVersionProto get version => $_getN(0);
  @$pb.TagNumber(1)
  set version(ApiSchemaVersionProto value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasVersion() => $_has(0);
  @$pb.TagNumber(1)
  void clearVersion() => $_clearField(1);
  @$pb.TagNumber(1)
  ApiSchemaVersionProto ensureVersion() => $_ensure(0);
}

class GetSchemaResponse extends $pb.GeneratedMessage {
  factory GetSchemaResponse({
    ApiSchemaProto? schema,
    $core.String? latestContent,
  }) {
    final result = create();
    if (schema != null) result.schema = schema;
    if (latestContent != null) result.latestContent = latestContent;
    return result;
  }

  GetSchemaResponse._();

  factory GetSchemaResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory GetSchemaResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'GetSchemaResponse',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.apiregistry.v1'),
      createEmptyInstance: create)
    ..aOM<ApiSchemaProto>(1, _omitFieldNames ? '' : 'schema',
        subBuilder: ApiSchemaProto.create)
    ..aOS(2, _omitFieldNames ? '' : 'latestContent')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetSchemaResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetSchemaResponse copyWith(void Function(GetSchemaResponse) updates) =>
      super.copyWith((message) => updates(message as GetSchemaResponse))
          as GetSchemaResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static GetSchemaResponse create() => GetSchemaResponse._();
  @$core.override
  GetSchemaResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static GetSchemaResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<GetSchemaResponse>(create);
  static GetSchemaResponse? _defaultInstance;

  @$pb.TagNumber(1)
  ApiSchemaProto get schema => $_getN(0);
  @$pb.TagNumber(1)
  set schema(ApiSchemaProto value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasSchema() => $_has(0);
  @$pb.TagNumber(1)
  void clearSchema() => $_clearField(1);
  @$pb.TagNumber(1)
  ApiSchemaProto ensureSchema() => $_ensure(0);

  @$pb.TagNumber(2)
  $core.String get latestContent => $_getSZ(1);
  @$pb.TagNumber(2)
  set latestContent($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasLatestContent() => $_has(1);
  @$pb.TagNumber(2)
  void clearLatestContent() => $_clearField(2);
}

class ListVersionsRequest extends $pb.GeneratedMessage {
  factory ListVersionsRequest({
    $core.String? name,
    $1.Pagination? pagination,
  }) {
    final result = create();
    if (name != null) result.name = name;
    if (pagination != null) result.pagination = pagination;
    return result;
  }

  ListVersionsRequest._();

  factory ListVersionsRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory ListVersionsRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ListVersionsRequest',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.apiregistry.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'name')
    ..aOM<$1.Pagination>(2, _omitFieldNames ? '' : 'pagination',
        subBuilder: $1.Pagination.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListVersionsRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListVersionsRequest copyWith(void Function(ListVersionsRequest) updates) =>
      super.copyWith((message) => updates(message as ListVersionsRequest))
          as ListVersionsRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static ListVersionsRequest create() => ListVersionsRequest._();
  @$core.override
  ListVersionsRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static ListVersionsRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ListVersionsRequest>(create);
  static ListVersionsRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get name => $_getSZ(0);
  @$pb.TagNumber(1)
  set name($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasName() => $_has(0);
  @$pb.TagNumber(1)
  void clearName() => $_clearField(1);

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
}

class ListVersionsResponse extends $pb.GeneratedMessage {
  factory ListVersionsResponse({
    $core.String? name,
    $core.Iterable<ApiSchemaVersionProto>? versions,
    $1.PaginationResult? pagination,
  }) {
    final result = create();
    if (name != null) result.name = name;
    if (versions != null) result.versions.addAll(versions);
    if (pagination != null) result.pagination = pagination;
    return result;
  }

  ListVersionsResponse._();

  factory ListVersionsResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory ListVersionsResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ListVersionsResponse',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.apiregistry.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'name')
    ..pPM<ApiSchemaVersionProto>(2, _omitFieldNames ? '' : 'versions',
        subBuilder: ApiSchemaVersionProto.create)
    ..aOM<$1.PaginationResult>(3, _omitFieldNames ? '' : 'pagination',
        subBuilder: $1.PaginationResult.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListVersionsResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListVersionsResponse copyWith(void Function(ListVersionsResponse) updates) =>
      super.copyWith((message) => updates(message as ListVersionsResponse))
          as ListVersionsResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static ListVersionsResponse create() => ListVersionsResponse._();
  @$core.override
  ListVersionsResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static ListVersionsResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ListVersionsResponse>(create);
  static ListVersionsResponse? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get name => $_getSZ(0);
  @$pb.TagNumber(1)
  set name($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasName() => $_has(0);
  @$pb.TagNumber(1)
  void clearName() => $_clearField(1);

  @$pb.TagNumber(2)
  $pb.PbList<ApiSchemaVersionProto> get versions => $_getList(1);

  @$pb.TagNumber(3)
  $1.PaginationResult get pagination => $_getN(2);
  @$pb.TagNumber(3)
  set pagination($1.PaginationResult value) => $_setField(3, value);
  @$pb.TagNumber(3)
  $core.bool hasPagination() => $_has(2);
  @$pb.TagNumber(3)
  void clearPagination() => $_clearField(3);
  @$pb.TagNumber(3)
  $1.PaginationResult ensurePagination() => $_ensure(2);
}

class RegisterVersionRequest extends $pb.GeneratedMessage {
  factory RegisterVersionRequest({
    $core.String? name,
    $core.String? content,
    $core.String? registeredBy,
  }) {
    final result = create();
    if (name != null) result.name = name;
    if (content != null) result.content = content;
    if (registeredBy != null) result.registeredBy = registeredBy;
    return result;
  }

  RegisterVersionRequest._();

  factory RegisterVersionRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory RegisterVersionRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'RegisterVersionRequest',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.apiregistry.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'name')
    ..aOS(2, _omitFieldNames ? '' : 'content')
    ..aOS(3, _omitFieldNames ? '' : 'registeredBy')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  RegisterVersionRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  RegisterVersionRequest copyWith(
          void Function(RegisterVersionRequest) updates) =>
      super.copyWith((message) => updates(message as RegisterVersionRequest))
          as RegisterVersionRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static RegisterVersionRequest create() => RegisterVersionRequest._();
  @$core.override
  RegisterVersionRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static RegisterVersionRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<RegisterVersionRequest>(create);
  static RegisterVersionRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get name => $_getSZ(0);
  @$pb.TagNumber(1)
  set name($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasName() => $_has(0);
  @$pb.TagNumber(1)
  void clearName() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get content => $_getSZ(1);
  @$pb.TagNumber(2)
  set content($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasContent() => $_has(1);
  @$pb.TagNumber(2)
  void clearContent() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.String get registeredBy => $_getSZ(2);
  @$pb.TagNumber(3)
  set registeredBy($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasRegisteredBy() => $_has(2);
  @$pb.TagNumber(3)
  void clearRegisteredBy() => $_clearField(3);
}

class RegisterVersionResponse extends $pb.GeneratedMessage {
  factory RegisterVersionResponse({
    ApiSchemaVersionProto? version,
  }) {
    final result = create();
    if (version != null) result.version = version;
    return result;
  }

  RegisterVersionResponse._();

  factory RegisterVersionResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory RegisterVersionResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'RegisterVersionResponse',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.apiregistry.v1'),
      createEmptyInstance: create)
    ..aOM<ApiSchemaVersionProto>(1, _omitFieldNames ? '' : 'version',
        subBuilder: ApiSchemaVersionProto.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  RegisterVersionResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  RegisterVersionResponse copyWith(
          void Function(RegisterVersionResponse) updates) =>
      super.copyWith((message) => updates(message as RegisterVersionResponse))
          as RegisterVersionResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static RegisterVersionResponse create() => RegisterVersionResponse._();
  @$core.override
  RegisterVersionResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static RegisterVersionResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<RegisterVersionResponse>(create);
  static RegisterVersionResponse? _defaultInstance;

  @$pb.TagNumber(1)
  ApiSchemaVersionProto get version => $_getN(0);
  @$pb.TagNumber(1)
  set version(ApiSchemaVersionProto value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasVersion() => $_has(0);
  @$pb.TagNumber(1)
  void clearVersion() => $_clearField(1);
  @$pb.TagNumber(1)
  ApiSchemaVersionProto ensureVersion() => $_ensure(0);
}

class GetSchemaVersionRequest extends $pb.GeneratedMessage {
  factory GetSchemaVersionRequest({
    $core.String? name,
    $core.int? version,
  }) {
    final result = create();
    if (name != null) result.name = name;
    if (version != null) result.version = version;
    return result;
  }

  GetSchemaVersionRequest._();

  factory GetSchemaVersionRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory GetSchemaVersionRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'GetSchemaVersionRequest',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.apiregistry.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'name')
    ..aI(2, _omitFieldNames ? '' : 'version', fieldType: $pb.PbFieldType.OU3)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetSchemaVersionRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetSchemaVersionRequest copyWith(
          void Function(GetSchemaVersionRequest) updates) =>
      super.copyWith((message) => updates(message as GetSchemaVersionRequest))
          as GetSchemaVersionRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static GetSchemaVersionRequest create() => GetSchemaVersionRequest._();
  @$core.override
  GetSchemaVersionRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static GetSchemaVersionRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<GetSchemaVersionRequest>(create);
  static GetSchemaVersionRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get name => $_getSZ(0);
  @$pb.TagNumber(1)
  set name($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasName() => $_has(0);
  @$pb.TagNumber(1)
  void clearName() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.int get version => $_getIZ(1);
  @$pb.TagNumber(2)
  set version($core.int value) => $_setUnsignedInt32(1, value);
  @$pb.TagNumber(2)
  $core.bool hasVersion() => $_has(1);
  @$pb.TagNumber(2)
  void clearVersion() => $_clearField(2);
}

class GetSchemaVersionResponse extends $pb.GeneratedMessage {
  factory GetSchemaVersionResponse({
    ApiSchemaVersionProto? version,
  }) {
    final result = create();
    if (version != null) result.version = version;
    return result;
  }

  GetSchemaVersionResponse._();

  factory GetSchemaVersionResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory GetSchemaVersionResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'GetSchemaVersionResponse',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.apiregistry.v1'),
      createEmptyInstance: create)
    ..aOM<ApiSchemaVersionProto>(1, _omitFieldNames ? '' : 'version',
        subBuilder: ApiSchemaVersionProto.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetSchemaVersionResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetSchemaVersionResponse copyWith(
          void Function(GetSchemaVersionResponse) updates) =>
      super.copyWith((message) => updates(message as GetSchemaVersionResponse))
          as GetSchemaVersionResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static GetSchemaVersionResponse create() => GetSchemaVersionResponse._();
  @$core.override
  GetSchemaVersionResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static GetSchemaVersionResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<GetSchemaVersionResponse>(create);
  static GetSchemaVersionResponse? _defaultInstance;

  @$pb.TagNumber(1)
  ApiSchemaVersionProto get version => $_getN(0);
  @$pb.TagNumber(1)
  set version(ApiSchemaVersionProto value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasVersion() => $_has(0);
  @$pb.TagNumber(1)
  void clearVersion() => $_clearField(1);
  @$pb.TagNumber(1)
  ApiSchemaVersionProto ensureVersion() => $_ensure(0);
}

class DeleteVersionRequest extends $pb.GeneratedMessage {
  factory DeleteVersionRequest({
    $core.String? name,
    $core.int? version,
  }) {
    final result = create();
    if (name != null) result.name = name;
    if (version != null) result.version = version;
    return result;
  }

  DeleteVersionRequest._();

  factory DeleteVersionRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory DeleteVersionRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'DeleteVersionRequest',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.apiregistry.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'name')
    ..aI(2, _omitFieldNames ? '' : 'version', fieldType: $pb.PbFieldType.OU3)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  DeleteVersionRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  DeleteVersionRequest copyWith(void Function(DeleteVersionRequest) updates) =>
      super.copyWith((message) => updates(message as DeleteVersionRequest))
          as DeleteVersionRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static DeleteVersionRequest create() => DeleteVersionRequest._();
  @$core.override
  DeleteVersionRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static DeleteVersionRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<DeleteVersionRequest>(create);
  static DeleteVersionRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get name => $_getSZ(0);
  @$pb.TagNumber(1)
  set name($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasName() => $_has(0);
  @$pb.TagNumber(1)
  void clearName() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.int get version => $_getIZ(1);
  @$pb.TagNumber(2)
  set version($core.int value) => $_setUnsignedInt32(1, value);
  @$pb.TagNumber(2)
  $core.bool hasVersion() => $_has(1);
  @$pb.TagNumber(2)
  void clearVersion() => $_clearField(2);
}

class DeleteVersionResponse extends $pb.GeneratedMessage {
  factory DeleteVersionResponse({
    $core.bool? success,
    $core.String? message,
  }) {
    final result = create();
    if (success != null) result.success = success;
    if (message != null) result.message = message;
    return result;
  }

  DeleteVersionResponse._();

  factory DeleteVersionResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory DeleteVersionResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'DeleteVersionResponse',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.apiregistry.v1'),
      createEmptyInstance: create)
    ..aOB(1, _omitFieldNames ? '' : 'success')
    ..aOS(2, _omitFieldNames ? '' : 'message')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  DeleteVersionResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  DeleteVersionResponse copyWith(
          void Function(DeleteVersionResponse) updates) =>
      super.copyWith((message) => updates(message as DeleteVersionResponse))
          as DeleteVersionResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static DeleteVersionResponse create() => DeleteVersionResponse._();
  @$core.override
  DeleteVersionResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static DeleteVersionResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<DeleteVersionResponse>(create);
  static DeleteVersionResponse? _defaultInstance;

  @$pb.TagNumber(1)
  $core.bool get success => $_getBF(0);
  @$pb.TagNumber(1)
  set success($core.bool value) => $_setBool(0, value);
  @$pb.TagNumber(1)
  $core.bool hasSuccess() => $_has(0);
  @$pb.TagNumber(1)
  void clearSuccess() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get message => $_getSZ(1);
  @$pb.TagNumber(2)
  set message($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasMessage() => $_has(1);
  @$pb.TagNumber(2)
  void clearMessage() => $_clearField(2);
}

class CheckCompatibilityRequest extends $pb.GeneratedMessage {
  factory CheckCompatibilityRequest({
    $core.String? name,
    $core.String? content,
    $core.int? baseVersion,
  }) {
    final result = create();
    if (name != null) result.name = name;
    if (content != null) result.content = content;
    if (baseVersion != null) result.baseVersion = baseVersion;
    return result;
  }

  CheckCompatibilityRequest._();

  factory CheckCompatibilityRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory CheckCompatibilityRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'CheckCompatibilityRequest',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.apiregistry.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'name')
    ..aOS(2, _omitFieldNames ? '' : 'content')
    ..aI(3, _omitFieldNames ? '' : 'baseVersion',
        fieldType: $pb.PbFieldType.OU3)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  CheckCompatibilityRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  CheckCompatibilityRequest copyWith(
          void Function(CheckCompatibilityRequest) updates) =>
      super.copyWith((message) => updates(message as CheckCompatibilityRequest))
          as CheckCompatibilityRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static CheckCompatibilityRequest create() => CheckCompatibilityRequest._();
  @$core.override
  CheckCompatibilityRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static CheckCompatibilityRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<CheckCompatibilityRequest>(create);
  static CheckCompatibilityRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get name => $_getSZ(0);
  @$pb.TagNumber(1)
  set name($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasName() => $_has(0);
  @$pb.TagNumber(1)
  void clearName() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get content => $_getSZ(1);
  @$pb.TagNumber(2)
  set content($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasContent() => $_has(1);
  @$pb.TagNumber(2)
  void clearContent() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.int get baseVersion => $_getIZ(2);
  @$pb.TagNumber(3)
  set baseVersion($core.int value) => $_setUnsignedInt32(2, value);
  @$pb.TagNumber(3)
  $core.bool hasBaseVersion() => $_has(2);
  @$pb.TagNumber(3)
  void clearBaseVersion() => $_clearField(3);
}

class CheckCompatibilityResponse extends $pb.GeneratedMessage {
  factory CheckCompatibilityResponse({
    $core.String? name,
    $core.int? baseVersion,
    CompatibilityResultProto? result,
  }) {
    final result$ = create();
    if (name != null) result$.name = name;
    if (baseVersion != null) result$.baseVersion = baseVersion;
    if (result != null) result$.result = result;
    return result$;
  }

  CheckCompatibilityResponse._();

  factory CheckCompatibilityResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory CheckCompatibilityResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'CheckCompatibilityResponse',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.apiregistry.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'name')
    ..aI(2, _omitFieldNames ? '' : 'baseVersion',
        fieldType: $pb.PbFieldType.OU3)
    ..aOM<CompatibilityResultProto>(3, _omitFieldNames ? '' : 'result',
        subBuilder: CompatibilityResultProto.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  CheckCompatibilityResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  CheckCompatibilityResponse copyWith(
          void Function(CheckCompatibilityResponse) updates) =>
      super.copyWith(
              (message) => updates(message as CheckCompatibilityResponse))
          as CheckCompatibilityResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static CheckCompatibilityResponse create() => CheckCompatibilityResponse._();
  @$core.override
  CheckCompatibilityResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static CheckCompatibilityResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<CheckCompatibilityResponse>(create);
  static CheckCompatibilityResponse? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get name => $_getSZ(0);
  @$pb.TagNumber(1)
  set name($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasName() => $_has(0);
  @$pb.TagNumber(1)
  void clearName() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.int get baseVersion => $_getIZ(1);
  @$pb.TagNumber(2)
  set baseVersion($core.int value) => $_setUnsignedInt32(1, value);
  @$pb.TagNumber(2)
  $core.bool hasBaseVersion() => $_has(1);
  @$pb.TagNumber(2)
  void clearBaseVersion() => $_clearField(2);

  @$pb.TagNumber(3)
  CompatibilityResultProto get result => $_getN(2);
  @$pb.TagNumber(3)
  set result(CompatibilityResultProto value) => $_setField(3, value);
  @$pb.TagNumber(3)
  $core.bool hasResult() => $_has(2);
  @$pb.TagNumber(3)
  void clearResult() => $_clearField(3);
  @$pb.TagNumber(3)
  CompatibilityResultProto ensureResult() => $_ensure(2);
}

class GetDiffRequest extends $pb.GeneratedMessage {
  factory GetDiffRequest({
    $core.String? name,
    $core.int? fromVersion,
    $core.int? toVersion,
  }) {
    final result = create();
    if (name != null) result.name = name;
    if (fromVersion != null) result.fromVersion = fromVersion;
    if (toVersion != null) result.toVersion = toVersion;
    return result;
  }

  GetDiffRequest._();

  factory GetDiffRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory GetDiffRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'GetDiffRequest',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.apiregistry.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'name')
    ..aI(2, _omitFieldNames ? '' : 'fromVersion',
        fieldType: $pb.PbFieldType.OU3)
    ..aI(3, _omitFieldNames ? '' : 'toVersion', fieldType: $pb.PbFieldType.OU3)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetDiffRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetDiffRequest copyWith(void Function(GetDiffRequest) updates) =>
      super.copyWith((message) => updates(message as GetDiffRequest))
          as GetDiffRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static GetDiffRequest create() => GetDiffRequest._();
  @$core.override
  GetDiffRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static GetDiffRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<GetDiffRequest>(create);
  static GetDiffRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get name => $_getSZ(0);
  @$pb.TagNumber(1)
  set name($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasName() => $_has(0);
  @$pb.TagNumber(1)
  void clearName() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.int get fromVersion => $_getIZ(1);
  @$pb.TagNumber(2)
  set fromVersion($core.int value) => $_setUnsignedInt32(1, value);
  @$pb.TagNumber(2)
  $core.bool hasFromVersion() => $_has(1);
  @$pb.TagNumber(2)
  void clearFromVersion() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.int get toVersion => $_getIZ(2);
  @$pb.TagNumber(3)
  set toVersion($core.int value) => $_setUnsignedInt32(2, value);
  @$pb.TagNumber(3)
  $core.bool hasToVersion() => $_has(2);
  @$pb.TagNumber(3)
  void clearToVersion() => $_clearField(3);
}

class GetDiffResponse extends $pb.GeneratedMessage {
  factory GetDiffResponse({
    $core.String? name,
    $core.int? fromVersion,
    $core.int? toVersion,
    $core.bool? breakingChanges,
    SchemaDiffProto? diff,
  }) {
    final result = create();
    if (name != null) result.name = name;
    if (fromVersion != null) result.fromVersion = fromVersion;
    if (toVersion != null) result.toVersion = toVersion;
    if (breakingChanges != null) result.breakingChanges = breakingChanges;
    if (diff != null) result.diff = diff;
    return result;
  }

  GetDiffResponse._();

  factory GetDiffResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory GetDiffResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'GetDiffResponse',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.apiregistry.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'name')
    ..aI(2, _omitFieldNames ? '' : 'fromVersion',
        fieldType: $pb.PbFieldType.OU3)
    ..aI(3, _omitFieldNames ? '' : 'toVersion', fieldType: $pb.PbFieldType.OU3)
    ..aOB(4, _omitFieldNames ? '' : 'breakingChanges')
    ..aOM<SchemaDiffProto>(5, _omitFieldNames ? '' : 'diff',
        subBuilder: SchemaDiffProto.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetDiffResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetDiffResponse copyWith(void Function(GetDiffResponse) updates) =>
      super.copyWith((message) => updates(message as GetDiffResponse))
          as GetDiffResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static GetDiffResponse create() => GetDiffResponse._();
  @$core.override
  GetDiffResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static GetDiffResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<GetDiffResponse>(create);
  static GetDiffResponse? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get name => $_getSZ(0);
  @$pb.TagNumber(1)
  set name($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasName() => $_has(0);
  @$pb.TagNumber(1)
  void clearName() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.int get fromVersion => $_getIZ(1);
  @$pb.TagNumber(2)
  set fromVersion($core.int value) => $_setUnsignedInt32(1, value);
  @$pb.TagNumber(2)
  $core.bool hasFromVersion() => $_has(1);
  @$pb.TagNumber(2)
  void clearFromVersion() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.int get toVersion => $_getIZ(2);
  @$pb.TagNumber(3)
  set toVersion($core.int value) => $_setUnsignedInt32(2, value);
  @$pb.TagNumber(3)
  $core.bool hasToVersion() => $_has(2);
  @$pb.TagNumber(3)
  void clearToVersion() => $_clearField(3);

  @$pb.TagNumber(4)
  $core.bool get breakingChanges => $_getBF(3);
  @$pb.TagNumber(4)
  set breakingChanges($core.bool value) => $_setBool(3, value);
  @$pb.TagNumber(4)
  $core.bool hasBreakingChanges() => $_has(3);
  @$pb.TagNumber(4)
  void clearBreakingChanges() => $_clearField(4);

  @$pb.TagNumber(5)
  SchemaDiffProto get diff => $_getN(4);
  @$pb.TagNumber(5)
  set diff(SchemaDiffProto value) => $_setField(5, value);
  @$pb.TagNumber(5)
  $core.bool hasDiff() => $_has(4);
  @$pb.TagNumber(5)
  void clearDiff() => $_clearField(5);
  @$pb.TagNumber(5)
  SchemaDiffProto ensureDiff() => $_ensure(4);
}

class ApiSchemaProto extends $pb.GeneratedMessage {
  factory ApiSchemaProto({
    $core.String? name,
    $core.String? description,
    $core.String? schemaType,
    $core.int? latestVersion,
    $core.int? versionCount,
    $1.Timestamp? createdAt,
    $1.Timestamp? updatedAt,
  }) {
    final result = create();
    if (name != null) result.name = name;
    if (description != null) result.description = description;
    if (schemaType != null) result.schemaType = schemaType;
    if (latestVersion != null) result.latestVersion = latestVersion;
    if (versionCount != null) result.versionCount = versionCount;
    if (createdAt != null) result.createdAt = createdAt;
    if (updatedAt != null) result.updatedAt = updatedAt;
    return result;
  }

  ApiSchemaProto._();

  factory ApiSchemaProto.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory ApiSchemaProto.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ApiSchemaProto',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.apiregistry.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'name')
    ..aOS(2, _omitFieldNames ? '' : 'description')
    ..aOS(3, _omitFieldNames ? '' : 'schemaType')
    ..aI(4, _omitFieldNames ? '' : 'latestVersion',
        fieldType: $pb.PbFieldType.OU3)
    ..aI(5, _omitFieldNames ? '' : 'versionCount',
        fieldType: $pb.PbFieldType.OU3)
    ..aOM<$1.Timestamp>(6, _omitFieldNames ? '' : 'createdAt',
        subBuilder: $1.Timestamp.create)
    ..aOM<$1.Timestamp>(7, _omitFieldNames ? '' : 'updatedAt',
        subBuilder: $1.Timestamp.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ApiSchemaProto clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ApiSchemaProto copyWith(void Function(ApiSchemaProto) updates) =>
      super.copyWith((message) => updates(message as ApiSchemaProto))
          as ApiSchemaProto;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static ApiSchemaProto create() => ApiSchemaProto._();
  @$core.override
  ApiSchemaProto createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static ApiSchemaProto getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ApiSchemaProto>(create);
  static ApiSchemaProto? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get name => $_getSZ(0);
  @$pb.TagNumber(1)
  set name($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasName() => $_has(0);
  @$pb.TagNumber(1)
  void clearName() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get description => $_getSZ(1);
  @$pb.TagNumber(2)
  set description($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasDescription() => $_has(1);
  @$pb.TagNumber(2)
  void clearDescription() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.String get schemaType => $_getSZ(2);
  @$pb.TagNumber(3)
  set schemaType($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasSchemaType() => $_has(2);
  @$pb.TagNumber(3)
  void clearSchemaType() => $_clearField(3);

  @$pb.TagNumber(4)
  $core.int get latestVersion => $_getIZ(3);
  @$pb.TagNumber(4)
  set latestVersion($core.int value) => $_setUnsignedInt32(3, value);
  @$pb.TagNumber(4)
  $core.bool hasLatestVersion() => $_has(3);
  @$pb.TagNumber(4)
  void clearLatestVersion() => $_clearField(4);

  @$pb.TagNumber(5)
  $core.int get versionCount => $_getIZ(4);
  @$pb.TagNumber(5)
  set versionCount($core.int value) => $_setUnsignedInt32(4, value);
  @$pb.TagNumber(5)
  $core.bool hasVersionCount() => $_has(4);
  @$pb.TagNumber(5)
  void clearVersionCount() => $_clearField(5);

  @$pb.TagNumber(6)
  $1.Timestamp get createdAt => $_getN(5);
  @$pb.TagNumber(6)
  set createdAt($1.Timestamp value) => $_setField(6, value);
  @$pb.TagNumber(6)
  $core.bool hasCreatedAt() => $_has(5);
  @$pb.TagNumber(6)
  void clearCreatedAt() => $_clearField(6);
  @$pb.TagNumber(6)
  $1.Timestamp ensureCreatedAt() => $_ensure(5);

  @$pb.TagNumber(7)
  $1.Timestamp get updatedAt => $_getN(6);
  @$pb.TagNumber(7)
  set updatedAt($1.Timestamp value) => $_setField(7, value);
  @$pb.TagNumber(7)
  $core.bool hasUpdatedAt() => $_has(6);
  @$pb.TagNumber(7)
  void clearUpdatedAt() => $_clearField(7);
  @$pb.TagNumber(7)
  $1.Timestamp ensureUpdatedAt() => $_ensure(6);
}

class ApiSchemaVersionProto extends $pb.GeneratedMessage {
  factory ApiSchemaVersionProto({
    $core.String? name,
    $core.int? version,
    $core.String? schemaType,
    $core.String? content,
    $core.String? contentHash,
    $core.bool? breakingChanges,
    $core.String? registeredBy,
    $1.Timestamp? createdAt,
    $core.Iterable<SchemaChange>? breakingChangeDetails,
  }) {
    final result = create();
    if (name != null) result.name = name;
    if (version != null) result.version = version;
    if (schemaType != null) result.schemaType = schemaType;
    if (content != null) result.content = content;
    if (contentHash != null) result.contentHash = contentHash;
    if (breakingChanges != null) result.breakingChanges = breakingChanges;
    if (registeredBy != null) result.registeredBy = registeredBy;
    if (createdAt != null) result.createdAt = createdAt;
    if (breakingChangeDetails != null)
      result.breakingChangeDetails.addAll(breakingChangeDetails);
    return result;
  }

  ApiSchemaVersionProto._();

  factory ApiSchemaVersionProto.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory ApiSchemaVersionProto.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ApiSchemaVersionProto',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.apiregistry.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'name')
    ..aI(2, _omitFieldNames ? '' : 'version', fieldType: $pb.PbFieldType.OU3)
    ..aOS(3, _omitFieldNames ? '' : 'schemaType')
    ..aOS(4, _omitFieldNames ? '' : 'content')
    ..aOS(5, _omitFieldNames ? '' : 'contentHash')
    ..aOB(6, _omitFieldNames ? '' : 'breakingChanges')
    ..aOS(7, _omitFieldNames ? '' : 'registeredBy')
    ..aOM<$1.Timestamp>(8, _omitFieldNames ? '' : 'createdAt',
        subBuilder: $1.Timestamp.create)
    ..pPM<SchemaChange>(9, _omitFieldNames ? '' : 'breakingChangeDetails',
        subBuilder: SchemaChange.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ApiSchemaVersionProto clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ApiSchemaVersionProto copyWith(
          void Function(ApiSchemaVersionProto) updates) =>
      super.copyWith((message) => updates(message as ApiSchemaVersionProto))
          as ApiSchemaVersionProto;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static ApiSchemaVersionProto create() => ApiSchemaVersionProto._();
  @$core.override
  ApiSchemaVersionProto createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static ApiSchemaVersionProto getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ApiSchemaVersionProto>(create);
  static ApiSchemaVersionProto? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get name => $_getSZ(0);
  @$pb.TagNumber(1)
  set name($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasName() => $_has(0);
  @$pb.TagNumber(1)
  void clearName() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.int get version => $_getIZ(1);
  @$pb.TagNumber(2)
  set version($core.int value) => $_setUnsignedInt32(1, value);
  @$pb.TagNumber(2)
  $core.bool hasVersion() => $_has(1);
  @$pb.TagNumber(2)
  void clearVersion() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.String get schemaType => $_getSZ(2);
  @$pb.TagNumber(3)
  set schemaType($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasSchemaType() => $_has(2);
  @$pb.TagNumber(3)
  void clearSchemaType() => $_clearField(3);

  @$pb.TagNumber(4)
  $core.String get content => $_getSZ(3);
  @$pb.TagNumber(4)
  set content($core.String value) => $_setString(3, value);
  @$pb.TagNumber(4)
  $core.bool hasContent() => $_has(3);
  @$pb.TagNumber(4)
  void clearContent() => $_clearField(4);

  @$pb.TagNumber(5)
  $core.String get contentHash => $_getSZ(4);
  @$pb.TagNumber(5)
  set contentHash($core.String value) => $_setString(4, value);
  @$pb.TagNumber(5)
  $core.bool hasContentHash() => $_has(4);
  @$pb.TagNumber(5)
  void clearContentHash() => $_clearField(5);

  @$pb.TagNumber(6)
  $core.bool get breakingChanges => $_getBF(5);
  @$pb.TagNumber(6)
  set breakingChanges($core.bool value) => $_setBool(5, value);
  @$pb.TagNumber(6)
  $core.bool hasBreakingChanges() => $_has(5);
  @$pb.TagNumber(6)
  void clearBreakingChanges() => $_clearField(6);

  @$pb.TagNumber(7)
  $core.String get registeredBy => $_getSZ(6);
  @$pb.TagNumber(7)
  set registeredBy($core.String value) => $_setString(6, value);
  @$pb.TagNumber(7)
  $core.bool hasRegisteredBy() => $_has(6);
  @$pb.TagNumber(7)
  void clearRegisteredBy() => $_clearField(7);

  @$pb.TagNumber(8)
  $1.Timestamp get createdAt => $_getN(7);
  @$pb.TagNumber(8)
  set createdAt($1.Timestamp value) => $_setField(8, value);
  @$pb.TagNumber(8)
  $core.bool hasCreatedAt() => $_has(7);
  @$pb.TagNumber(8)
  void clearCreatedAt() => $_clearField(8);
  @$pb.TagNumber(8)
  $1.Timestamp ensureCreatedAt() => $_ensure(7);

  @$pb.TagNumber(9)
  $pb.PbList<SchemaChange> get breakingChangeDetails => $_getList(8);
}

class SchemaChange extends $pb.GeneratedMessage {
  factory SchemaChange({
    $core.String? changeType,
    $core.String? path,
    $core.String? description,
  }) {
    final result = create();
    if (changeType != null) result.changeType = changeType;
    if (path != null) result.path = path;
    if (description != null) result.description = description;
    return result;
  }

  SchemaChange._();

  factory SchemaChange.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory SchemaChange.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'SchemaChange',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.apiregistry.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'changeType')
    ..aOS(2, _omitFieldNames ? '' : 'path')
    ..aOS(3, _omitFieldNames ? '' : 'description')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  SchemaChange clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  SchemaChange copyWith(void Function(SchemaChange) updates) =>
      super.copyWith((message) => updates(message as SchemaChange))
          as SchemaChange;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static SchemaChange create() => SchemaChange._();
  @$core.override
  SchemaChange createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static SchemaChange getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<SchemaChange>(create);
  static SchemaChange? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get changeType => $_getSZ(0);
  @$pb.TagNumber(1)
  set changeType($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasChangeType() => $_has(0);
  @$pb.TagNumber(1)
  void clearChangeType() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get path => $_getSZ(1);
  @$pb.TagNumber(2)
  set path($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasPath() => $_has(1);
  @$pb.TagNumber(2)
  void clearPath() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.String get description => $_getSZ(2);
  @$pb.TagNumber(3)
  set description($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasDescription() => $_has(2);
  @$pb.TagNumber(3)
  void clearDescription() => $_clearField(3);
}

class CompatibilityResultProto extends $pb.GeneratedMessage {
  factory CompatibilityResultProto({
    $core.bool? compatible,
    $core.Iterable<SchemaChange>? breakingChanges,
    $core.Iterable<SchemaChange>? nonBreakingChanges,
  }) {
    final result = create();
    if (compatible != null) result.compatible = compatible;
    if (breakingChanges != null) result.breakingChanges.addAll(breakingChanges);
    if (nonBreakingChanges != null)
      result.nonBreakingChanges.addAll(nonBreakingChanges);
    return result;
  }

  CompatibilityResultProto._();

  factory CompatibilityResultProto.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory CompatibilityResultProto.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'CompatibilityResultProto',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.apiregistry.v1'),
      createEmptyInstance: create)
    ..aOB(1, _omitFieldNames ? '' : 'compatible')
    ..pPM<SchemaChange>(2, _omitFieldNames ? '' : 'breakingChanges',
        subBuilder: SchemaChange.create)
    ..pPM<SchemaChange>(3, _omitFieldNames ? '' : 'nonBreakingChanges',
        subBuilder: SchemaChange.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  CompatibilityResultProto clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  CompatibilityResultProto copyWith(
          void Function(CompatibilityResultProto) updates) =>
      super.copyWith((message) => updates(message as CompatibilityResultProto))
          as CompatibilityResultProto;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static CompatibilityResultProto create() => CompatibilityResultProto._();
  @$core.override
  CompatibilityResultProto createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static CompatibilityResultProto getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<CompatibilityResultProto>(create);
  static CompatibilityResultProto? _defaultInstance;

  @$pb.TagNumber(1)
  $core.bool get compatible => $_getBF(0);
  @$pb.TagNumber(1)
  set compatible($core.bool value) => $_setBool(0, value);
  @$pb.TagNumber(1)
  $core.bool hasCompatible() => $_has(0);
  @$pb.TagNumber(1)
  void clearCompatible() => $_clearField(1);

  @$pb.TagNumber(2)
  $pb.PbList<SchemaChange> get breakingChanges => $_getList(1);

  @$pb.TagNumber(3)
  $pb.PbList<SchemaChange> get nonBreakingChanges => $_getList(2);
}

class SchemaDiffProto extends $pb.GeneratedMessage {
  factory SchemaDiffProto({
    $core.Iterable<DiffEntryProto>? added,
    $core.Iterable<DiffModifiedEntryProto>? modified,
    $core.Iterable<DiffEntryProto>? removed,
  }) {
    final result = create();
    if (added != null) result.added.addAll(added);
    if (modified != null) result.modified.addAll(modified);
    if (removed != null) result.removed.addAll(removed);
    return result;
  }

  SchemaDiffProto._();

  factory SchemaDiffProto.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory SchemaDiffProto.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'SchemaDiffProto',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.apiregistry.v1'),
      createEmptyInstance: create)
    ..pPM<DiffEntryProto>(1, _omitFieldNames ? '' : 'added',
        subBuilder: DiffEntryProto.create)
    ..pPM<DiffModifiedEntryProto>(2, _omitFieldNames ? '' : 'modified',
        subBuilder: DiffModifiedEntryProto.create)
    ..pPM<DiffEntryProto>(3, _omitFieldNames ? '' : 'removed',
        subBuilder: DiffEntryProto.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  SchemaDiffProto clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  SchemaDiffProto copyWith(void Function(SchemaDiffProto) updates) =>
      super.copyWith((message) => updates(message as SchemaDiffProto))
          as SchemaDiffProto;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static SchemaDiffProto create() => SchemaDiffProto._();
  @$core.override
  SchemaDiffProto createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static SchemaDiffProto getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<SchemaDiffProto>(create);
  static SchemaDiffProto? _defaultInstance;

  @$pb.TagNumber(1)
  $pb.PbList<DiffEntryProto> get added => $_getList(0);

  @$pb.TagNumber(2)
  $pb.PbList<DiffModifiedEntryProto> get modified => $_getList(1);

  @$pb.TagNumber(3)
  $pb.PbList<DiffEntryProto> get removed => $_getList(2);
}

class DiffEntryProto extends $pb.GeneratedMessage {
  factory DiffEntryProto({
    $core.String? path,
    $core.String? type,
    $core.String? description,
  }) {
    final result = create();
    if (path != null) result.path = path;
    if (type != null) result.type = type;
    if (description != null) result.description = description;
    return result;
  }

  DiffEntryProto._();

  factory DiffEntryProto.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory DiffEntryProto.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'DiffEntryProto',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.apiregistry.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'path')
    ..aOS(2, _omitFieldNames ? '' : 'type')
    ..aOS(3, _omitFieldNames ? '' : 'description')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  DiffEntryProto clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  DiffEntryProto copyWith(void Function(DiffEntryProto) updates) =>
      super.copyWith((message) => updates(message as DiffEntryProto))
          as DiffEntryProto;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static DiffEntryProto create() => DiffEntryProto._();
  @$core.override
  DiffEntryProto createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static DiffEntryProto getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<DiffEntryProto>(create);
  static DiffEntryProto? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get path => $_getSZ(0);
  @$pb.TagNumber(1)
  set path($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasPath() => $_has(0);
  @$pb.TagNumber(1)
  void clearPath() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get type => $_getSZ(1);
  @$pb.TagNumber(2)
  set type($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasType() => $_has(1);
  @$pb.TagNumber(2)
  void clearType() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.String get description => $_getSZ(2);
  @$pb.TagNumber(3)
  set description($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasDescription() => $_has(2);
  @$pb.TagNumber(3)
  void clearDescription() => $_clearField(3);
}

class DiffModifiedEntryProto extends $pb.GeneratedMessage {
  factory DiffModifiedEntryProto({
    $core.String? path,
    $core.String? before,
    $core.String? after,
  }) {
    final result = create();
    if (path != null) result.path = path;
    if (before != null) result.before = before;
    if (after != null) result.after = after;
    return result;
  }

  DiffModifiedEntryProto._();

  factory DiffModifiedEntryProto.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory DiffModifiedEntryProto.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'DiffModifiedEntryProto',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.apiregistry.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'path')
    ..aOS(2, _omitFieldNames ? '' : 'before')
    ..aOS(3, _omitFieldNames ? '' : 'after')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  DiffModifiedEntryProto clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  DiffModifiedEntryProto copyWith(
          void Function(DiffModifiedEntryProto) updates) =>
      super.copyWith((message) => updates(message as DiffModifiedEntryProto))
          as DiffModifiedEntryProto;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static DiffModifiedEntryProto create() => DiffModifiedEntryProto._();
  @$core.override
  DiffModifiedEntryProto createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static DiffModifiedEntryProto getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<DiffModifiedEntryProto>(create);
  static DiffModifiedEntryProto? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get path => $_getSZ(0);
  @$pb.TagNumber(1)
  set path($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasPath() => $_has(0);
  @$pb.TagNumber(1)
  void clearPath() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get before => $_getSZ(1);
  @$pb.TagNumber(2)
  set before($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasBefore() => $_has(1);
  @$pb.TagNumber(2)
  void clearBefore() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.String get after => $_getSZ(2);
  @$pb.TagNumber(3)
  set after($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasAfter() => $_has(2);
  @$pb.TagNumber(3)
  void clearAfter() => $_clearField(3);
}

const $core.bool _omitFieldNames =
    $core.bool.fromEnvironment('protobuf.omit_field_names');
const $core.bool _omitMessageNames =
    $core.bool.fromEnvironment('protobuf.omit_message_names');
