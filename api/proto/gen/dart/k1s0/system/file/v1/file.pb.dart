// This is a generated file - do not edit.
//
// Generated from k1s0/system/file/v1/file.proto.

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

class FileMetadata extends $pb.GeneratedMessage {
  factory FileMetadata({
    $core.String? id,
    $core.String? filename,
    $core.String? contentType,
    $fixnum.Int64? sizeBytes,
    $core.String? tenantId,
    $core.String? uploadedBy,
    $core.String? status,
    $core.String? createdAt,
    $core.String? updatedAt,
    $core.Iterable<$core.MapEntry<$core.String, $core.String>>? tags,
    $core.String? storageKey,
    $core.String? checksumSha256,
  }) {
    final result = create();
    if (id != null) result.id = id;
    if (filename != null) result.filename = filename;
    if (contentType != null) result.contentType = contentType;
    if (sizeBytes != null) result.sizeBytes = sizeBytes;
    if (tenantId != null) result.tenantId = tenantId;
    if (uploadedBy != null) result.uploadedBy = uploadedBy;
    if (status != null) result.status = status;
    if (createdAt != null) result.createdAt = createdAt;
    if (updatedAt != null) result.updatedAt = updatedAt;
    if (tags != null) result.tags.addEntries(tags);
    if (storageKey != null) result.storageKey = storageKey;
    if (checksumSha256 != null) result.checksumSha256 = checksumSha256;
    return result;
  }

  FileMetadata._();

  factory FileMetadata.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory FileMetadata.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'FileMetadata',
      package:
          const $pb.PackageName(_omitMessageNames ? '' : 'k1s0.system.file.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'id')
    ..aOS(2, _omitFieldNames ? '' : 'filename')
    ..aOS(3, _omitFieldNames ? '' : 'contentType')
    ..aInt64(4, _omitFieldNames ? '' : 'sizeBytes')
    ..aOS(5, _omitFieldNames ? '' : 'tenantId')
    ..aOS(6, _omitFieldNames ? '' : 'uploadedBy')
    ..aOS(7, _omitFieldNames ? '' : 'status')
    ..aOS(8, _omitFieldNames ? '' : 'createdAt')
    ..aOS(9, _omitFieldNames ? '' : 'updatedAt')
    ..m<$core.String, $core.String>(10, _omitFieldNames ? '' : 'tags',
        entryClassName: 'FileMetadata.TagsEntry',
        keyFieldType: $pb.PbFieldType.OS,
        valueFieldType: $pb.PbFieldType.OS,
        packageName: const $pb.PackageName('k1s0.system.file.v1'))
    ..aOS(11, _omitFieldNames ? '' : 'storageKey')
    ..aOS(12, _omitFieldNames ? '' : 'checksumSha256')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  FileMetadata clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  FileMetadata copyWith(void Function(FileMetadata) updates) =>
      super.copyWith((message) => updates(message as FileMetadata))
          as FileMetadata;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static FileMetadata create() => FileMetadata._();
  @$core.override
  FileMetadata createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static FileMetadata getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<FileMetadata>(create);
  static FileMetadata? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get id => $_getSZ(0);
  @$pb.TagNumber(1)
  set id($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasId() => $_has(0);
  @$pb.TagNumber(1)
  void clearId() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get filename => $_getSZ(1);
  @$pb.TagNumber(2)
  set filename($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasFilename() => $_has(1);
  @$pb.TagNumber(2)
  void clearFilename() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.String get contentType => $_getSZ(2);
  @$pb.TagNumber(3)
  set contentType($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasContentType() => $_has(2);
  @$pb.TagNumber(3)
  void clearContentType() => $_clearField(3);

  @$pb.TagNumber(4)
  $fixnum.Int64 get sizeBytes => $_getI64(3);
  @$pb.TagNumber(4)
  set sizeBytes($fixnum.Int64 value) => $_setInt64(3, value);
  @$pb.TagNumber(4)
  $core.bool hasSizeBytes() => $_has(3);
  @$pb.TagNumber(4)
  void clearSizeBytes() => $_clearField(4);

  @$pb.TagNumber(5)
  $core.String get tenantId => $_getSZ(4);
  @$pb.TagNumber(5)
  set tenantId($core.String value) => $_setString(4, value);
  @$pb.TagNumber(5)
  $core.bool hasTenantId() => $_has(4);
  @$pb.TagNumber(5)
  void clearTenantId() => $_clearField(5);

  @$pb.TagNumber(6)
  $core.String get uploadedBy => $_getSZ(5);
  @$pb.TagNumber(6)
  set uploadedBy($core.String value) => $_setString(5, value);
  @$pb.TagNumber(6)
  $core.bool hasUploadedBy() => $_has(5);
  @$pb.TagNumber(6)
  void clearUploadedBy() => $_clearField(6);

  @$pb.TagNumber(7)
  $core.String get status => $_getSZ(6);
  @$pb.TagNumber(7)
  set status($core.String value) => $_setString(6, value);
  @$pb.TagNumber(7)
  $core.bool hasStatus() => $_has(6);
  @$pb.TagNumber(7)
  void clearStatus() => $_clearField(7);

  @$pb.TagNumber(8)
  $core.String get createdAt => $_getSZ(7);
  @$pb.TagNumber(8)
  set createdAt($core.String value) => $_setString(7, value);
  @$pb.TagNumber(8)
  $core.bool hasCreatedAt() => $_has(7);
  @$pb.TagNumber(8)
  void clearCreatedAt() => $_clearField(8);

  @$pb.TagNumber(9)
  $core.String get updatedAt => $_getSZ(8);
  @$pb.TagNumber(9)
  set updatedAt($core.String value) => $_setString(8, value);
  @$pb.TagNumber(9)
  $core.bool hasUpdatedAt() => $_has(8);
  @$pb.TagNumber(9)
  void clearUpdatedAt() => $_clearField(9);

  @$pb.TagNumber(10)
  $pb.PbMap<$core.String, $core.String> get tags => $_getMap(9);

  @$pb.TagNumber(11)
  $core.String get storageKey => $_getSZ(10);
  @$pb.TagNumber(11)
  set storageKey($core.String value) => $_setString(10, value);
  @$pb.TagNumber(11)
  $core.bool hasStorageKey() => $_has(10);
  @$pb.TagNumber(11)
  void clearStorageKey() => $_clearField(11);

  @$pb.TagNumber(12)
  $core.String get checksumSha256 => $_getSZ(11);
  @$pb.TagNumber(12)
  set checksumSha256($core.String value) => $_setString(11, value);
  @$pb.TagNumber(12)
  $core.bool hasChecksumSha256() => $_has(11);
  @$pb.TagNumber(12)
  void clearChecksumSha256() => $_clearField(12);
}

class GetFileMetadataRequest extends $pb.GeneratedMessage {
  factory GetFileMetadataRequest({
    $core.String? id,
  }) {
    final result = create();
    if (id != null) result.id = id;
    return result;
  }

  GetFileMetadataRequest._();

  factory GetFileMetadataRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory GetFileMetadataRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'GetFileMetadataRequest',
      package:
          const $pb.PackageName(_omitMessageNames ? '' : 'k1s0.system.file.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'id')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetFileMetadataRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetFileMetadataRequest copyWith(
          void Function(GetFileMetadataRequest) updates) =>
      super.copyWith((message) => updates(message as GetFileMetadataRequest))
          as GetFileMetadataRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static GetFileMetadataRequest create() => GetFileMetadataRequest._();
  @$core.override
  GetFileMetadataRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static GetFileMetadataRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<GetFileMetadataRequest>(create);
  static GetFileMetadataRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get id => $_getSZ(0);
  @$pb.TagNumber(1)
  set id($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasId() => $_has(0);
  @$pb.TagNumber(1)
  void clearId() => $_clearField(1);
}

class GetFileMetadataResponse extends $pb.GeneratedMessage {
  factory GetFileMetadataResponse({
    FileMetadata? metadata,
  }) {
    final result = create();
    if (metadata != null) result.metadata = metadata;
    return result;
  }

  GetFileMetadataResponse._();

  factory GetFileMetadataResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory GetFileMetadataResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'GetFileMetadataResponse',
      package:
          const $pb.PackageName(_omitMessageNames ? '' : 'k1s0.system.file.v1'),
      createEmptyInstance: create)
    ..aOM<FileMetadata>(1, _omitFieldNames ? '' : 'metadata',
        subBuilder: FileMetadata.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetFileMetadataResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetFileMetadataResponse copyWith(
          void Function(GetFileMetadataResponse) updates) =>
      super.copyWith((message) => updates(message as GetFileMetadataResponse))
          as GetFileMetadataResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static GetFileMetadataResponse create() => GetFileMetadataResponse._();
  @$core.override
  GetFileMetadataResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static GetFileMetadataResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<GetFileMetadataResponse>(create);
  static GetFileMetadataResponse? _defaultInstance;

  @$pb.TagNumber(1)
  FileMetadata get metadata => $_getN(0);
  @$pb.TagNumber(1)
  set metadata(FileMetadata value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasMetadata() => $_has(0);
  @$pb.TagNumber(1)
  void clearMetadata() => $_clearField(1);
  @$pb.TagNumber(1)
  FileMetadata ensureMetadata() => $_ensure(0);
}

class ListFilesRequest extends $pb.GeneratedMessage {
  factory ListFilesRequest({
    $core.String? tenantId,
    $1.Pagination? pagination,
    $core.String? uploadedBy,
    $core.String? mimeType,
    $core.String? tag,
  }) {
    final result = create();
    if (tenantId != null) result.tenantId = tenantId;
    if (pagination != null) result.pagination = pagination;
    if (uploadedBy != null) result.uploadedBy = uploadedBy;
    if (mimeType != null) result.mimeType = mimeType;
    if (tag != null) result.tag = tag;
    return result;
  }

  ListFilesRequest._();

  factory ListFilesRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory ListFilesRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ListFilesRequest',
      package:
          const $pb.PackageName(_omitMessageNames ? '' : 'k1s0.system.file.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'tenantId')
    ..aOM<$1.Pagination>(2, _omitFieldNames ? '' : 'pagination',
        subBuilder: $1.Pagination.create)
    ..aOS(4, _omitFieldNames ? '' : 'uploadedBy')
    ..aOS(5, _omitFieldNames ? '' : 'mimeType')
    ..aOS(6, _omitFieldNames ? '' : 'tag')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListFilesRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListFilesRequest copyWith(void Function(ListFilesRequest) updates) =>
      super.copyWith((message) => updates(message as ListFilesRequest))
          as ListFilesRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static ListFilesRequest create() => ListFilesRequest._();
  @$core.override
  ListFilesRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static ListFilesRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ListFilesRequest>(create);
  static ListFilesRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get tenantId => $_getSZ(0);
  @$pb.TagNumber(1)
  set tenantId($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasTenantId() => $_has(0);
  @$pb.TagNumber(1)
  void clearTenantId() => $_clearField(1);

  /// ページネーションパラメータを共通型に統一
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

  @$pb.TagNumber(4)
  $core.String get uploadedBy => $_getSZ(2);
  @$pb.TagNumber(4)
  set uploadedBy($core.String value) => $_setString(2, value);
  @$pb.TagNumber(4)
  $core.bool hasUploadedBy() => $_has(2);
  @$pb.TagNumber(4)
  void clearUploadedBy() => $_clearField(4);

  @$pb.TagNumber(5)
  $core.String get mimeType => $_getSZ(3);
  @$pb.TagNumber(5)
  set mimeType($core.String value) => $_setString(3, value);
  @$pb.TagNumber(5)
  $core.bool hasMimeType() => $_has(3);
  @$pb.TagNumber(5)
  void clearMimeType() => $_clearField(5);

  @$pb.TagNumber(6)
  $core.String get tag => $_getSZ(4);
  @$pb.TagNumber(6)
  set tag($core.String value) => $_setString(4, value);
  @$pb.TagNumber(6)
  $core.bool hasTag() => $_has(4);
  @$pb.TagNumber(6)
  void clearTag() => $_clearField(6);
}

class ListFilesResponse extends $pb.GeneratedMessage {
  factory ListFilesResponse({
    $core.Iterable<FileMetadata>? files,
    $1.PaginationResult? pagination,
  }) {
    final result = create();
    if (files != null) result.files.addAll(files);
    if (pagination != null) result.pagination = pagination;
    return result;
  }

  ListFilesResponse._();

  factory ListFilesResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory ListFilesResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ListFilesResponse',
      package:
          const $pb.PackageName(_omitMessageNames ? '' : 'k1s0.system.file.v1'),
      createEmptyInstance: create)
    ..pPM<FileMetadata>(1, _omitFieldNames ? '' : 'files',
        subBuilder: FileMetadata.create)
    ..aOM<$1.PaginationResult>(2, _omitFieldNames ? '' : 'pagination',
        subBuilder: $1.PaginationResult.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListFilesResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListFilesResponse copyWith(void Function(ListFilesResponse) updates) =>
      super.copyWith((message) => updates(message as ListFilesResponse))
          as ListFilesResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static ListFilesResponse create() => ListFilesResponse._();
  @$core.override
  ListFilesResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static ListFilesResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ListFilesResponse>(create);
  static ListFilesResponse? _defaultInstance;

  @$pb.TagNumber(1)
  $pb.PbList<FileMetadata> get files => $_getList(0);

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

class GenerateUploadUrlRequest extends $pb.GeneratedMessage {
  factory GenerateUploadUrlRequest({
    $core.String? filename,
    $core.String? contentType,
    $core.String? tenantId,
    $core.String? uploadedBy,
    $core.Iterable<$core.MapEntry<$core.String, $core.String>>? tags,
    $core.int? expiresInSeconds,
    $fixnum.Int64? sizeBytes,
  }) {
    final result = create();
    if (filename != null) result.filename = filename;
    if (contentType != null) result.contentType = contentType;
    if (tenantId != null) result.tenantId = tenantId;
    if (uploadedBy != null) result.uploadedBy = uploadedBy;
    if (tags != null) result.tags.addEntries(tags);
    if (expiresInSeconds != null) result.expiresInSeconds = expiresInSeconds;
    if (sizeBytes != null) result.sizeBytes = sizeBytes;
    return result;
  }

  GenerateUploadUrlRequest._();

  factory GenerateUploadUrlRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory GenerateUploadUrlRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'GenerateUploadUrlRequest',
      package:
          const $pb.PackageName(_omitMessageNames ? '' : 'k1s0.system.file.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'filename')
    ..aOS(2, _omitFieldNames ? '' : 'contentType')
    ..aOS(3, _omitFieldNames ? '' : 'tenantId')
    ..aOS(4, _omitFieldNames ? '' : 'uploadedBy')
    ..m<$core.String, $core.String>(5, _omitFieldNames ? '' : 'tags',
        entryClassName: 'GenerateUploadUrlRequest.TagsEntry',
        keyFieldType: $pb.PbFieldType.OS,
        valueFieldType: $pb.PbFieldType.OS,
        packageName: const $pb.PackageName('k1s0.system.file.v1'))
    ..aI(6, _omitFieldNames ? '' : 'expiresInSeconds',
        fieldType: $pb.PbFieldType.OU3)
    ..aInt64(7, _omitFieldNames ? '' : 'sizeBytes')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GenerateUploadUrlRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GenerateUploadUrlRequest copyWith(
          void Function(GenerateUploadUrlRequest) updates) =>
      super.copyWith((message) => updates(message as GenerateUploadUrlRequest))
          as GenerateUploadUrlRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static GenerateUploadUrlRequest create() => GenerateUploadUrlRequest._();
  @$core.override
  GenerateUploadUrlRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static GenerateUploadUrlRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<GenerateUploadUrlRequest>(create);
  static GenerateUploadUrlRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get filename => $_getSZ(0);
  @$pb.TagNumber(1)
  set filename($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasFilename() => $_has(0);
  @$pb.TagNumber(1)
  void clearFilename() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get contentType => $_getSZ(1);
  @$pb.TagNumber(2)
  set contentType($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasContentType() => $_has(1);
  @$pb.TagNumber(2)
  void clearContentType() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.String get tenantId => $_getSZ(2);
  @$pb.TagNumber(3)
  set tenantId($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasTenantId() => $_has(2);
  @$pb.TagNumber(3)
  void clearTenantId() => $_clearField(3);

  @$pb.TagNumber(4)
  $core.String get uploadedBy => $_getSZ(3);
  @$pb.TagNumber(4)
  set uploadedBy($core.String value) => $_setString(3, value);
  @$pb.TagNumber(4)
  $core.bool hasUploadedBy() => $_has(3);
  @$pb.TagNumber(4)
  void clearUploadedBy() => $_clearField(4);

  @$pb.TagNumber(5)
  $pb.PbMap<$core.String, $core.String> get tags => $_getMap(4);

  @$pb.TagNumber(6)
  $core.int get expiresInSeconds => $_getIZ(5);
  @$pb.TagNumber(6)
  set expiresInSeconds($core.int value) => $_setUnsignedInt32(5, value);
  @$pb.TagNumber(6)
  $core.bool hasExpiresInSeconds() => $_has(5);
  @$pb.TagNumber(6)
  void clearExpiresInSeconds() => $_clearField(6);

  @$pb.TagNumber(7)
  $fixnum.Int64 get sizeBytes => $_getI64(6);
  @$pb.TagNumber(7)
  set sizeBytes($fixnum.Int64 value) => $_setInt64(6, value);
  @$pb.TagNumber(7)
  $core.bool hasSizeBytes() => $_has(6);
  @$pb.TagNumber(7)
  void clearSizeBytes() => $_clearField(7);
}

class GenerateUploadUrlResponse extends $pb.GeneratedMessage {
  factory GenerateUploadUrlResponse({
    $core.String? fileId,
    $core.String? uploadUrl,
    $core.int? expiresInSeconds,
  }) {
    final result = create();
    if (fileId != null) result.fileId = fileId;
    if (uploadUrl != null) result.uploadUrl = uploadUrl;
    if (expiresInSeconds != null) result.expiresInSeconds = expiresInSeconds;
    return result;
  }

  GenerateUploadUrlResponse._();

  factory GenerateUploadUrlResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory GenerateUploadUrlResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'GenerateUploadUrlResponse',
      package:
          const $pb.PackageName(_omitMessageNames ? '' : 'k1s0.system.file.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'fileId')
    ..aOS(2, _omitFieldNames ? '' : 'uploadUrl')
    ..aI(3, _omitFieldNames ? '' : 'expiresInSeconds',
        fieldType: $pb.PbFieldType.OU3)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GenerateUploadUrlResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GenerateUploadUrlResponse copyWith(
          void Function(GenerateUploadUrlResponse) updates) =>
      super.copyWith((message) => updates(message as GenerateUploadUrlResponse))
          as GenerateUploadUrlResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static GenerateUploadUrlResponse create() => GenerateUploadUrlResponse._();
  @$core.override
  GenerateUploadUrlResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static GenerateUploadUrlResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<GenerateUploadUrlResponse>(create);
  static GenerateUploadUrlResponse? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get fileId => $_getSZ(0);
  @$pb.TagNumber(1)
  set fileId($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasFileId() => $_has(0);
  @$pb.TagNumber(1)
  void clearFileId() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get uploadUrl => $_getSZ(1);
  @$pb.TagNumber(2)
  set uploadUrl($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasUploadUrl() => $_has(1);
  @$pb.TagNumber(2)
  void clearUploadUrl() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.int get expiresInSeconds => $_getIZ(2);
  @$pb.TagNumber(3)
  set expiresInSeconds($core.int value) => $_setUnsignedInt32(2, value);
  @$pb.TagNumber(3)
  $core.bool hasExpiresInSeconds() => $_has(2);
  @$pb.TagNumber(3)
  void clearExpiresInSeconds() => $_clearField(3);
}

class CompleteUploadRequest extends $pb.GeneratedMessage {
  factory CompleteUploadRequest({
    $core.String? fileId,
    $core.String? checksumSha256,
  }) {
    final result = create();
    if (fileId != null) result.fileId = fileId;
    if (checksumSha256 != null) result.checksumSha256 = checksumSha256;
    return result;
  }

  CompleteUploadRequest._();

  factory CompleteUploadRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory CompleteUploadRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'CompleteUploadRequest',
      package:
          const $pb.PackageName(_omitMessageNames ? '' : 'k1s0.system.file.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'fileId')
    ..aOS(3, _omitFieldNames ? '' : 'checksumSha256')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  CompleteUploadRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  CompleteUploadRequest copyWith(
          void Function(CompleteUploadRequest) updates) =>
      super.copyWith((message) => updates(message as CompleteUploadRequest))
          as CompleteUploadRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static CompleteUploadRequest create() => CompleteUploadRequest._();
  @$core.override
  CompleteUploadRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static CompleteUploadRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<CompleteUploadRequest>(create);
  static CompleteUploadRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get fileId => $_getSZ(0);
  @$pb.TagNumber(1)
  set fileId($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasFileId() => $_has(0);
  @$pb.TagNumber(1)
  void clearFileId() => $_clearField(1);

  @$pb.TagNumber(3)
  $core.String get checksumSha256 => $_getSZ(1);
  @$pb.TagNumber(3)
  set checksumSha256($core.String value) => $_setString(1, value);
  @$pb.TagNumber(3)
  $core.bool hasChecksumSha256() => $_has(1);
  @$pb.TagNumber(3)
  void clearChecksumSha256() => $_clearField(3);
}

class CompleteUploadResponse extends $pb.GeneratedMessage {
  factory CompleteUploadResponse({
    FileMetadata? metadata,
  }) {
    final result = create();
    if (metadata != null) result.metadata = metadata;
    return result;
  }

  CompleteUploadResponse._();

  factory CompleteUploadResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory CompleteUploadResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'CompleteUploadResponse',
      package:
          const $pb.PackageName(_omitMessageNames ? '' : 'k1s0.system.file.v1'),
      createEmptyInstance: create)
    ..aOM<FileMetadata>(1, _omitFieldNames ? '' : 'metadata',
        subBuilder: FileMetadata.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  CompleteUploadResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  CompleteUploadResponse copyWith(
          void Function(CompleteUploadResponse) updates) =>
      super.copyWith((message) => updates(message as CompleteUploadResponse))
          as CompleteUploadResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static CompleteUploadResponse create() => CompleteUploadResponse._();
  @$core.override
  CompleteUploadResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static CompleteUploadResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<CompleteUploadResponse>(create);
  static CompleteUploadResponse? _defaultInstance;

  @$pb.TagNumber(1)
  FileMetadata get metadata => $_getN(0);
  @$pb.TagNumber(1)
  set metadata(FileMetadata value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasMetadata() => $_has(0);
  @$pb.TagNumber(1)
  void clearMetadata() => $_clearField(1);
  @$pb.TagNumber(1)
  FileMetadata ensureMetadata() => $_ensure(0);
}

class GenerateDownloadUrlRequest extends $pb.GeneratedMessage {
  factory GenerateDownloadUrlRequest({
    $core.String? id,
  }) {
    final result = create();
    if (id != null) result.id = id;
    return result;
  }

  GenerateDownloadUrlRequest._();

  factory GenerateDownloadUrlRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory GenerateDownloadUrlRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'GenerateDownloadUrlRequest',
      package:
          const $pb.PackageName(_omitMessageNames ? '' : 'k1s0.system.file.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'id')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GenerateDownloadUrlRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GenerateDownloadUrlRequest copyWith(
          void Function(GenerateDownloadUrlRequest) updates) =>
      super.copyWith(
              (message) => updates(message as GenerateDownloadUrlRequest))
          as GenerateDownloadUrlRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static GenerateDownloadUrlRequest create() => GenerateDownloadUrlRequest._();
  @$core.override
  GenerateDownloadUrlRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static GenerateDownloadUrlRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<GenerateDownloadUrlRequest>(create);
  static GenerateDownloadUrlRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get id => $_getSZ(0);
  @$pb.TagNumber(1)
  set id($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasId() => $_has(0);
  @$pb.TagNumber(1)
  void clearId() => $_clearField(1);
}

class GenerateDownloadUrlResponse extends $pb.GeneratedMessage {
  factory GenerateDownloadUrlResponse({
    $core.String? downloadUrl,
    $core.int? expiresInSeconds,
  }) {
    final result = create();
    if (downloadUrl != null) result.downloadUrl = downloadUrl;
    if (expiresInSeconds != null) result.expiresInSeconds = expiresInSeconds;
    return result;
  }

  GenerateDownloadUrlResponse._();

  factory GenerateDownloadUrlResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory GenerateDownloadUrlResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'GenerateDownloadUrlResponse',
      package:
          const $pb.PackageName(_omitMessageNames ? '' : 'k1s0.system.file.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'downloadUrl')
    ..aI(2, _omitFieldNames ? '' : 'expiresInSeconds')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GenerateDownloadUrlResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GenerateDownloadUrlResponse copyWith(
          void Function(GenerateDownloadUrlResponse) updates) =>
      super.copyWith(
              (message) => updates(message as GenerateDownloadUrlResponse))
          as GenerateDownloadUrlResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static GenerateDownloadUrlResponse create() =>
      GenerateDownloadUrlResponse._();
  @$core.override
  GenerateDownloadUrlResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static GenerateDownloadUrlResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<GenerateDownloadUrlResponse>(create);
  static GenerateDownloadUrlResponse? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get downloadUrl => $_getSZ(0);
  @$pb.TagNumber(1)
  set downloadUrl($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasDownloadUrl() => $_has(0);
  @$pb.TagNumber(1)
  void clearDownloadUrl() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.int get expiresInSeconds => $_getIZ(1);
  @$pb.TagNumber(2)
  set expiresInSeconds($core.int value) => $_setSignedInt32(1, value);
  @$pb.TagNumber(2)
  $core.bool hasExpiresInSeconds() => $_has(1);
  @$pb.TagNumber(2)
  void clearExpiresInSeconds() => $_clearField(2);
}

class UpdateFileTagsRequest extends $pb.GeneratedMessage {
  factory UpdateFileTagsRequest({
    $core.String? id,
    $core.Iterable<$core.MapEntry<$core.String, $core.String>>? tags,
  }) {
    final result = create();
    if (id != null) result.id = id;
    if (tags != null) result.tags.addEntries(tags);
    return result;
  }

  UpdateFileTagsRequest._();

  factory UpdateFileTagsRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory UpdateFileTagsRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'UpdateFileTagsRequest',
      package:
          const $pb.PackageName(_omitMessageNames ? '' : 'k1s0.system.file.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'id')
    ..m<$core.String, $core.String>(2, _omitFieldNames ? '' : 'tags',
        entryClassName: 'UpdateFileTagsRequest.TagsEntry',
        keyFieldType: $pb.PbFieldType.OS,
        valueFieldType: $pb.PbFieldType.OS,
        packageName: const $pb.PackageName('k1s0.system.file.v1'))
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  UpdateFileTagsRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  UpdateFileTagsRequest copyWith(
          void Function(UpdateFileTagsRequest) updates) =>
      super.copyWith((message) => updates(message as UpdateFileTagsRequest))
          as UpdateFileTagsRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static UpdateFileTagsRequest create() => UpdateFileTagsRequest._();
  @$core.override
  UpdateFileTagsRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static UpdateFileTagsRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<UpdateFileTagsRequest>(create);
  static UpdateFileTagsRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get id => $_getSZ(0);
  @$pb.TagNumber(1)
  set id($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasId() => $_has(0);
  @$pb.TagNumber(1)
  void clearId() => $_clearField(1);

  @$pb.TagNumber(2)
  $pb.PbMap<$core.String, $core.String> get tags => $_getMap(1);
}

class UpdateFileTagsResponse extends $pb.GeneratedMessage {
  factory UpdateFileTagsResponse({
    FileMetadata? metadata,
  }) {
    final result = create();
    if (metadata != null) result.metadata = metadata;
    return result;
  }

  UpdateFileTagsResponse._();

  factory UpdateFileTagsResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory UpdateFileTagsResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'UpdateFileTagsResponse',
      package:
          const $pb.PackageName(_omitMessageNames ? '' : 'k1s0.system.file.v1'),
      createEmptyInstance: create)
    ..aOM<FileMetadata>(1, _omitFieldNames ? '' : 'metadata',
        subBuilder: FileMetadata.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  UpdateFileTagsResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  UpdateFileTagsResponse copyWith(
          void Function(UpdateFileTagsResponse) updates) =>
      super.copyWith((message) => updates(message as UpdateFileTagsResponse))
          as UpdateFileTagsResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static UpdateFileTagsResponse create() => UpdateFileTagsResponse._();
  @$core.override
  UpdateFileTagsResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static UpdateFileTagsResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<UpdateFileTagsResponse>(create);
  static UpdateFileTagsResponse? _defaultInstance;

  @$pb.TagNumber(1)
  FileMetadata get metadata => $_getN(0);
  @$pb.TagNumber(1)
  set metadata(FileMetadata value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasMetadata() => $_has(0);
  @$pb.TagNumber(1)
  void clearMetadata() => $_clearField(1);
  @$pb.TagNumber(1)
  FileMetadata ensureMetadata() => $_ensure(0);
}

class DeleteFileRequest extends $pb.GeneratedMessage {
  factory DeleteFileRequest({
    $core.String? id,
  }) {
    final result = create();
    if (id != null) result.id = id;
    return result;
  }

  DeleteFileRequest._();

  factory DeleteFileRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory DeleteFileRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'DeleteFileRequest',
      package:
          const $pb.PackageName(_omitMessageNames ? '' : 'k1s0.system.file.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'id')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  DeleteFileRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  DeleteFileRequest copyWith(void Function(DeleteFileRequest) updates) =>
      super.copyWith((message) => updates(message as DeleteFileRequest))
          as DeleteFileRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static DeleteFileRequest create() => DeleteFileRequest._();
  @$core.override
  DeleteFileRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static DeleteFileRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<DeleteFileRequest>(create);
  static DeleteFileRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get id => $_getSZ(0);
  @$pb.TagNumber(1)
  set id($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasId() => $_has(0);
  @$pb.TagNumber(1)
  void clearId() => $_clearField(1);
}

class DeleteFileResponse extends $pb.GeneratedMessage {
  factory DeleteFileResponse() => create();

  DeleteFileResponse._();

  factory DeleteFileResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory DeleteFileResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'DeleteFileResponse',
      package:
          const $pb.PackageName(_omitMessageNames ? '' : 'k1s0.system.file.v1'),
      createEmptyInstance: create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  DeleteFileResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  DeleteFileResponse copyWith(void Function(DeleteFileResponse) updates) =>
      super.copyWith((message) => updates(message as DeleteFileResponse))
          as DeleteFileResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static DeleteFileResponse create() => DeleteFileResponse._();
  @$core.override
  DeleteFileResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static DeleteFileResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<DeleteFileResponse>(create);
  static DeleteFileResponse? _defaultInstance;
}

const $core.bool _omitFieldNames =
    $core.bool.fromEnvironment('protobuf.omit_field_names');
const $core.bool _omitMessageNames =
    $core.bool.fromEnvironment('protobuf.omit_message_names');
