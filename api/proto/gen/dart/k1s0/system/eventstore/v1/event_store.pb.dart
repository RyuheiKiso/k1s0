// This is a generated file - do not edit.
//
// Generated from k1s0/system/eventstore/v1/event_store.proto.

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

class ListStreamsRequest extends $pb.GeneratedMessage {
  factory ListStreamsRequest({
    $1.Pagination? pagination,
  }) {
    final result = create();
    if (pagination != null) result.pagination = pagination;
    return result;
  }

  ListStreamsRequest._();

  factory ListStreamsRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory ListStreamsRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ListStreamsRequest',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.eventstore.v1'),
      createEmptyInstance: create)
    ..aOM<$1.Pagination>(1, _omitFieldNames ? '' : 'pagination',
        subBuilder: $1.Pagination.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListStreamsRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListStreamsRequest copyWith(void Function(ListStreamsRequest) updates) =>
      super.copyWith((message) => updates(message as ListStreamsRequest))
          as ListStreamsRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static ListStreamsRequest create() => ListStreamsRequest._();
  @$core.override
  ListStreamsRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static ListStreamsRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ListStreamsRequest>(create);
  static ListStreamsRequest? _defaultInstance;

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
}

class ListStreamsResponse extends $pb.GeneratedMessage {
  factory ListStreamsResponse({
    $core.Iterable<StreamInfo>? streams,
    $1.PaginationResult? pagination,
  }) {
    final result = create();
    if (streams != null) result.streams.addAll(streams);
    if (pagination != null) result.pagination = pagination;
    return result;
  }

  ListStreamsResponse._();

  factory ListStreamsResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory ListStreamsResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ListStreamsResponse',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.eventstore.v1'),
      createEmptyInstance: create)
    ..pPM<StreamInfo>(1, _omitFieldNames ? '' : 'streams',
        subBuilder: StreamInfo.create)
    ..aOM<$1.PaginationResult>(2, _omitFieldNames ? '' : 'pagination',
        subBuilder: $1.PaginationResult.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListStreamsResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListStreamsResponse copyWith(void Function(ListStreamsResponse) updates) =>
      super.copyWith((message) => updates(message as ListStreamsResponse))
          as ListStreamsResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static ListStreamsResponse create() => ListStreamsResponse._();
  @$core.override
  ListStreamsResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static ListStreamsResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ListStreamsResponse>(create);
  static ListStreamsResponse? _defaultInstance;

  @$pb.TagNumber(1)
  $pb.PbList<StreamInfo> get streams => $_getList(0);

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

class StreamInfo extends $pb.GeneratedMessage {
  factory StreamInfo({
    $core.String? id,
    $core.String? aggregateType,
    $fixnum.Int64? currentVersion,
    $1.Timestamp? createdAt,
    $1.Timestamp? updatedAt,
  }) {
    final result = create();
    if (id != null) result.id = id;
    if (aggregateType != null) result.aggregateType = aggregateType;
    if (currentVersion != null) result.currentVersion = currentVersion;
    if (createdAt != null) result.createdAt = createdAt;
    if (updatedAt != null) result.updatedAt = updatedAt;
    return result;
  }

  StreamInfo._();

  factory StreamInfo.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory StreamInfo.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'StreamInfo',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.eventstore.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'id')
    ..aOS(2, _omitFieldNames ? '' : 'aggregateType')
    ..aInt64(3, _omitFieldNames ? '' : 'currentVersion')
    ..aOM<$1.Timestamp>(4, _omitFieldNames ? '' : 'createdAt',
        subBuilder: $1.Timestamp.create)
    ..aOM<$1.Timestamp>(5, _omitFieldNames ? '' : 'updatedAt',
        subBuilder: $1.Timestamp.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  StreamInfo clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  StreamInfo copyWith(void Function(StreamInfo) updates) =>
      super.copyWith((message) => updates(message as StreamInfo)) as StreamInfo;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static StreamInfo create() => StreamInfo._();
  @$core.override
  StreamInfo createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static StreamInfo getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<StreamInfo>(create);
  static StreamInfo? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get id => $_getSZ(0);
  @$pb.TagNumber(1)
  set id($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasId() => $_has(0);
  @$pb.TagNumber(1)
  void clearId() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get aggregateType => $_getSZ(1);
  @$pb.TagNumber(2)
  set aggregateType($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasAggregateType() => $_has(1);
  @$pb.TagNumber(2)
  void clearAggregateType() => $_clearField(2);

  @$pb.TagNumber(3)
  $fixnum.Int64 get currentVersion => $_getI64(2);
  @$pb.TagNumber(3)
  set currentVersion($fixnum.Int64 value) => $_setInt64(2, value);
  @$pb.TagNumber(3)
  $core.bool hasCurrentVersion() => $_has(2);
  @$pb.TagNumber(3)
  void clearCurrentVersion() => $_clearField(3);

  /// タイムスタンプ型を共通型に統一（string → Timestamp）
  @$pb.TagNumber(4)
  $1.Timestamp get createdAt => $_getN(3);
  @$pb.TagNumber(4)
  set createdAt($1.Timestamp value) => $_setField(4, value);
  @$pb.TagNumber(4)
  $core.bool hasCreatedAt() => $_has(3);
  @$pb.TagNumber(4)
  void clearCreatedAt() => $_clearField(4);
  @$pb.TagNumber(4)
  $1.Timestamp ensureCreatedAt() => $_ensure(3);

  /// タイムスタンプ型を共通型に統一（string → Timestamp）
  @$pb.TagNumber(5)
  $1.Timestamp get updatedAt => $_getN(4);
  @$pb.TagNumber(5)
  set updatedAt($1.Timestamp value) => $_setField(5, value);
  @$pb.TagNumber(5)
  $core.bool hasUpdatedAt() => $_has(4);
  @$pb.TagNumber(5)
  void clearUpdatedAt() => $_clearField(5);
  @$pb.TagNumber(5)
  $1.Timestamp ensureUpdatedAt() => $_ensure(4);
}

class AppendEventsRequest extends $pb.GeneratedMessage {
  factory AppendEventsRequest({
    $core.String? streamId,
    $core.Iterable<EventData>? events,
    $fixnum.Int64? expectedVersion,
    $core.String? aggregateType,
  }) {
    final result = create();
    if (streamId != null) result.streamId = streamId;
    if (events != null) result.events.addAll(events);
    if (expectedVersion != null) result.expectedVersion = expectedVersion;
    if (aggregateType != null) result.aggregateType = aggregateType;
    return result;
  }

  AppendEventsRequest._();

  factory AppendEventsRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory AppendEventsRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'AppendEventsRequest',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.eventstore.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'streamId')
    ..pPM<EventData>(2, _omitFieldNames ? '' : 'events',
        subBuilder: EventData.create)
    ..aInt64(3, _omitFieldNames ? '' : 'expectedVersion')
    ..aOS(4, _omitFieldNames ? '' : 'aggregateType')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  AppendEventsRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  AppendEventsRequest copyWith(void Function(AppendEventsRequest) updates) =>
      super.copyWith((message) => updates(message as AppendEventsRequest))
          as AppendEventsRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static AppendEventsRequest create() => AppendEventsRequest._();
  @$core.override
  AppendEventsRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static AppendEventsRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<AppendEventsRequest>(create);
  static AppendEventsRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get streamId => $_getSZ(0);
  @$pb.TagNumber(1)
  set streamId($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasStreamId() => $_has(0);
  @$pb.TagNumber(1)
  void clearStreamId() => $_clearField(1);

  @$pb.TagNumber(2)
  $pb.PbList<EventData> get events => $_getList(1);

  @$pb.TagNumber(3)
  $fixnum.Int64 get expectedVersion => $_getI64(2);
  @$pb.TagNumber(3)
  set expectedVersion($fixnum.Int64 value) => $_setInt64(2, value);
  @$pb.TagNumber(3)
  $core.bool hasExpectedVersion() => $_has(2);
  @$pb.TagNumber(3)
  void clearExpectedVersion() => $_clearField(3);

  @$pb.TagNumber(4)
  $core.String get aggregateType => $_getSZ(3);
  @$pb.TagNumber(4)
  set aggregateType($core.String value) => $_setString(3, value);
  @$pb.TagNumber(4)
  $core.bool hasAggregateType() => $_has(3);
  @$pb.TagNumber(4)
  void clearAggregateType() => $_clearField(4);
}

class AppendEventsResponse extends $pb.GeneratedMessage {
  factory AppendEventsResponse({
    $core.String? streamId,
    $core.Iterable<StoredEvent>? events,
    $fixnum.Int64? currentVersion,
  }) {
    final result = create();
    if (streamId != null) result.streamId = streamId;
    if (events != null) result.events.addAll(events);
    if (currentVersion != null) result.currentVersion = currentVersion;
    return result;
  }

  AppendEventsResponse._();

  factory AppendEventsResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory AppendEventsResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'AppendEventsResponse',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.eventstore.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'streamId')
    ..pPM<StoredEvent>(2, _omitFieldNames ? '' : 'events',
        subBuilder: StoredEvent.create)
    ..aInt64(3, _omitFieldNames ? '' : 'currentVersion')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  AppendEventsResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  AppendEventsResponse copyWith(void Function(AppendEventsResponse) updates) =>
      super.copyWith((message) => updates(message as AppendEventsResponse))
          as AppendEventsResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static AppendEventsResponse create() => AppendEventsResponse._();
  @$core.override
  AppendEventsResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static AppendEventsResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<AppendEventsResponse>(create);
  static AppendEventsResponse? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get streamId => $_getSZ(0);
  @$pb.TagNumber(1)
  set streamId($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasStreamId() => $_has(0);
  @$pb.TagNumber(1)
  void clearStreamId() => $_clearField(1);

  @$pb.TagNumber(2)
  $pb.PbList<StoredEvent> get events => $_getList(1);

  @$pb.TagNumber(3)
  $fixnum.Int64 get currentVersion => $_getI64(2);
  @$pb.TagNumber(3)
  set currentVersion($fixnum.Int64 value) => $_setInt64(2, value);
  @$pb.TagNumber(3)
  $core.bool hasCurrentVersion() => $_has(2);
  @$pb.TagNumber(3)
  void clearCurrentVersion() => $_clearField(3);
}

class ReadEventsRequest extends $pb.GeneratedMessage {
  factory ReadEventsRequest({
    $core.String? streamId,
    $fixnum.Int64? fromVersion,
    $fixnum.Int64? toVersion,
    $1.Pagination? pagination,
    $core.String? eventType,
  }) {
    final result = create();
    if (streamId != null) result.streamId = streamId;
    if (fromVersion != null) result.fromVersion = fromVersion;
    if (toVersion != null) result.toVersion = toVersion;
    if (pagination != null) result.pagination = pagination;
    if (eventType != null) result.eventType = eventType;
    return result;
  }

  ReadEventsRequest._();

  factory ReadEventsRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory ReadEventsRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ReadEventsRequest',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.eventstore.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'streamId')
    ..aInt64(2, _omitFieldNames ? '' : 'fromVersion')
    ..aInt64(3, _omitFieldNames ? '' : 'toVersion')
    ..aOM<$1.Pagination>(4, _omitFieldNames ? '' : 'pagination',
        subBuilder: $1.Pagination.create)
    ..aOS(6, _omitFieldNames ? '' : 'eventType')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ReadEventsRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ReadEventsRequest copyWith(void Function(ReadEventsRequest) updates) =>
      super.copyWith((message) => updates(message as ReadEventsRequest))
          as ReadEventsRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static ReadEventsRequest create() => ReadEventsRequest._();
  @$core.override
  ReadEventsRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static ReadEventsRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ReadEventsRequest>(create);
  static ReadEventsRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get streamId => $_getSZ(0);
  @$pb.TagNumber(1)
  set streamId($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasStreamId() => $_has(0);
  @$pb.TagNumber(1)
  void clearStreamId() => $_clearField(1);

  @$pb.TagNumber(2)
  $fixnum.Int64 get fromVersion => $_getI64(1);
  @$pb.TagNumber(2)
  set fromVersion($fixnum.Int64 value) => $_setInt64(1, value);
  @$pb.TagNumber(2)
  $core.bool hasFromVersion() => $_has(1);
  @$pb.TagNumber(2)
  void clearFromVersion() => $_clearField(2);

  @$pb.TagNumber(3)
  $fixnum.Int64 get toVersion => $_getI64(2);
  @$pb.TagNumber(3)
  set toVersion($fixnum.Int64 value) => $_setInt64(2, value);
  @$pb.TagNumber(3)
  $core.bool hasToVersion() => $_has(2);
  @$pb.TagNumber(3)
  void clearToVersion() => $_clearField(3);

  /// ページネーションパラメータを共通型に統一
  @$pb.TagNumber(4)
  $1.Pagination get pagination => $_getN(3);
  @$pb.TagNumber(4)
  set pagination($1.Pagination value) => $_setField(4, value);
  @$pb.TagNumber(4)
  $core.bool hasPagination() => $_has(3);
  @$pb.TagNumber(4)
  void clearPagination() => $_clearField(4);
  @$pb.TagNumber(4)
  $1.Pagination ensurePagination() => $_ensure(3);

  @$pb.TagNumber(6)
  $core.String get eventType => $_getSZ(4);
  @$pb.TagNumber(6)
  set eventType($core.String value) => $_setString(4, value);
  @$pb.TagNumber(6)
  $core.bool hasEventType() => $_has(4);
  @$pb.TagNumber(6)
  void clearEventType() => $_clearField(6);
}

class ReadEventsResponse extends $pb.GeneratedMessage {
  factory ReadEventsResponse({
    $core.String? streamId,
    $core.Iterable<StoredEvent>? events,
    $fixnum.Int64? currentVersion,
    $1.PaginationResult? pagination,
  }) {
    final result = create();
    if (streamId != null) result.streamId = streamId;
    if (events != null) result.events.addAll(events);
    if (currentVersion != null) result.currentVersion = currentVersion;
    if (pagination != null) result.pagination = pagination;
    return result;
  }

  ReadEventsResponse._();

  factory ReadEventsResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory ReadEventsResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ReadEventsResponse',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.eventstore.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'streamId')
    ..pPM<StoredEvent>(2, _omitFieldNames ? '' : 'events',
        subBuilder: StoredEvent.create)
    ..aInt64(3, _omitFieldNames ? '' : 'currentVersion')
    ..aOM<$1.PaginationResult>(4, _omitFieldNames ? '' : 'pagination',
        subBuilder: $1.PaginationResult.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ReadEventsResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ReadEventsResponse copyWith(void Function(ReadEventsResponse) updates) =>
      super.copyWith((message) => updates(message as ReadEventsResponse))
          as ReadEventsResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static ReadEventsResponse create() => ReadEventsResponse._();
  @$core.override
  ReadEventsResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static ReadEventsResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ReadEventsResponse>(create);
  static ReadEventsResponse? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get streamId => $_getSZ(0);
  @$pb.TagNumber(1)
  set streamId($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasStreamId() => $_has(0);
  @$pb.TagNumber(1)
  void clearStreamId() => $_clearField(1);

  @$pb.TagNumber(2)
  $pb.PbList<StoredEvent> get events => $_getList(1);

  @$pb.TagNumber(3)
  $fixnum.Int64 get currentVersion => $_getI64(2);
  @$pb.TagNumber(3)
  set currentVersion($fixnum.Int64 value) => $_setInt64(2, value);
  @$pb.TagNumber(3)
  $core.bool hasCurrentVersion() => $_has(2);
  @$pb.TagNumber(3)
  void clearCurrentVersion() => $_clearField(3);

  @$pb.TagNumber(4)
  $1.PaginationResult get pagination => $_getN(3);
  @$pb.TagNumber(4)
  set pagination($1.PaginationResult value) => $_setField(4, value);
  @$pb.TagNumber(4)
  $core.bool hasPagination() => $_has(3);
  @$pb.TagNumber(4)
  void clearPagination() => $_clearField(4);
  @$pb.TagNumber(4)
  $1.PaginationResult ensurePagination() => $_ensure(3);
}

class ReadEventBySequenceRequest extends $pb.GeneratedMessage {
  factory ReadEventBySequenceRequest({
    $core.String? streamId,
    $fixnum.Int64? sequence,
  }) {
    final result = create();
    if (streamId != null) result.streamId = streamId;
    if (sequence != null) result.sequence = sequence;
    return result;
  }

  ReadEventBySequenceRequest._();

  factory ReadEventBySequenceRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory ReadEventBySequenceRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ReadEventBySequenceRequest',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.eventstore.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'streamId')
    ..a<$fixnum.Int64>(
        2, _omitFieldNames ? '' : 'sequence', $pb.PbFieldType.OU6,
        defaultOrMaker: $fixnum.Int64.ZERO)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ReadEventBySequenceRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ReadEventBySequenceRequest copyWith(
          void Function(ReadEventBySequenceRequest) updates) =>
      super.copyWith(
              (message) => updates(message as ReadEventBySequenceRequest))
          as ReadEventBySequenceRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static ReadEventBySequenceRequest create() => ReadEventBySequenceRequest._();
  @$core.override
  ReadEventBySequenceRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static ReadEventBySequenceRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ReadEventBySequenceRequest>(create);
  static ReadEventBySequenceRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get streamId => $_getSZ(0);
  @$pb.TagNumber(1)
  set streamId($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasStreamId() => $_has(0);
  @$pb.TagNumber(1)
  void clearStreamId() => $_clearField(1);

  @$pb.TagNumber(2)
  $fixnum.Int64 get sequence => $_getI64(1);
  @$pb.TagNumber(2)
  set sequence($fixnum.Int64 value) => $_setInt64(1, value);
  @$pb.TagNumber(2)
  $core.bool hasSequence() => $_has(1);
  @$pb.TagNumber(2)
  void clearSequence() => $_clearField(2);
}

class ReadEventBySequenceResponse extends $pb.GeneratedMessage {
  factory ReadEventBySequenceResponse({
    StoredEvent? event,
  }) {
    final result = create();
    if (event != null) result.event = event;
    return result;
  }

  ReadEventBySequenceResponse._();

  factory ReadEventBySequenceResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory ReadEventBySequenceResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ReadEventBySequenceResponse',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.eventstore.v1'),
      createEmptyInstance: create)
    ..aOM<StoredEvent>(1, _omitFieldNames ? '' : 'event',
        subBuilder: StoredEvent.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ReadEventBySequenceResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ReadEventBySequenceResponse copyWith(
          void Function(ReadEventBySequenceResponse) updates) =>
      super.copyWith(
              (message) => updates(message as ReadEventBySequenceResponse))
          as ReadEventBySequenceResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static ReadEventBySequenceResponse create() =>
      ReadEventBySequenceResponse._();
  @$core.override
  ReadEventBySequenceResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static ReadEventBySequenceResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ReadEventBySequenceResponse>(create);
  static ReadEventBySequenceResponse? _defaultInstance;

  @$pb.TagNumber(1)
  StoredEvent get event => $_getN(0);
  @$pb.TagNumber(1)
  set event(StoredEvent value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasEvent() => $_has(0);
  @$pb.TagNumber(1)
  void clearEvent() => $_clearField(1);
  @$pb.TagNumber(1)
  StoredEvent ensureEvent() => $_ensure(0);
}

class CreateSnapshotRequest extends $pb.GeneratedMessage {
  factory CreateSnapshotRequest({
    $core.String? streamId,
    $fixnum.Int64? snapshotVersion,
    $core.String? aggregateType,
    $core.List<$core.int>? state,
  }) {
    final result = create();
    if (streamId != null) result.streamId = streamId;
    if (snapshotVersion != null) result.snapshotVersion = snapshotVersion;
    if (aggregateType != null) result.aggregateType = aggregateType;
    if (state != null) result.state = state;
    return result;
  }

  CreateSnapshotRequest._();

  factory CreateSnapshotRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory CreateSnapshotRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'CreateSnapshotRequest',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.eventstore.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'streamId')
    ..aInt64(2, _omitFieldNames ? '' : 'snapshotVersion')
    ..aOS(3, _omitFieldNames ? '' : 'aggregateType')
    ..a<$core.List<$core.int>>(
        4, _omitFieldNames ? '' : 'state', $pb.PbFieldType.OY)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  CreateSnapshotRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  CreateSnapshotRequest copyWith(
          void Function(CreateSnapshotRequest) updates) =>
      super.copyWith((message) => updates(message as CreateSnapshotRequest))
          as CreateSnapshotRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static CreateSnapshotRequest create() => CreateSnapshotRequest._();
  @$core.override
  CreateSnapshotRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static CreateSnapshotRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<CreateSnapshotRequest>(create);
  static CreateSnapshotRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get streamId => $_getSZ(0);
  @$pb.TagNumber(1)
  set streamId($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasStreamId() => $_has(0);
  @$pb.TagNumber(1)
  void clearStreamId() => $_clearField(1);

  @$pb.TagNumber(2)
  $fixnum.Int64 get snapshotVersion => $_getI64(1);
  @$pb.TagNumber(2)
  set snapshotVersion($fixnum.Int64 value) => $_setInt64(1, value);
  @$pb.TagNumber(2)
  $core.bool hasSnapshotVersion() => $_has(1);
  @$pb.TagNumber(2)
  void clearSnapshotVersion() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.String get aggregateType => $_getSZ(2);
  @$pb.TagNumber(3)
  set aggregateType($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasAggregateType() => $_has(2);
  @$pb.TagNumber(3)
  void clearAggregateType() => $_clearField(3);

  @$pb.TagNumber(4)
  $core.List<$core.int> get state => $_getN(3);
  @$pb.TagNumber(4)
  set state($core.List<$core.int> value) => $_setBytes(3, value);
  @$pb.TagNumber(4)
  $core.bool hasState() => $_has(3);
  @$pb.TagNumber(4)
  void clearState() => $_clearField(4);
}

class CreateSnapshotResponse extends $pb.GeneratedMessage {
  factory CreateSnapshotResponse({
    $core.String? id,
    $core.String? streamId,
    $fixnum.Int64? snapshotVersion,
    $1.Timestamp? createdAt,
    $core.String? aggregateType,
  }) {
    final result = create();
    if (id != null) result.id = id;
    if (streamId != null) result.streamId = streamId;
    if (snapshotVersion != null) result.snapshotVersion = snapshotVersion;
    if (createdAt != null) result.createdAt = createdAt;
    if (aggregateType != null) result.aggregateType = aggregateType;
    return result;
  }

  CreateSnapshotResponse._();

  factory CreateSnapshotResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory CreateSnapshotResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'CreateSnapshotResponse',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.eventstore.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'id')
    ..aOS(2, _omitFieldNames ? '' : 'streamId')
    ..aInt64(3, _omitFieldNames ? '' : 'snapshotVersion')
    ..aOM<$1.Timestamp>(4, _omitFieldNames ? '' : 'createdAt',
        subBuilder: $1.Timestamp.create)
    ..aOS(5, _omitFieldNames ? '' : 'aggregateType')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  CreateSnapshotResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  CreateSnapshotResponse copyWith(
          void Function(CreateSnapshotResponse) updates) =>
      super.copyWith((message) => updates(message as CreateSnapshotResponse))
          as CreateSnapshotResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static CreateSnapshotResponse create() => CreateSnapshotResponse._();
  @$core.override
  CreateSnapshotResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static CreateSnapshotResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<CreateSnapshotResponse>(create);
  static CreateSnapshotResponse? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get id => $_getSZ(0);
  @$pb.TagNumber(1)
  set id($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasId() => $_has(0);
  @$pb.TagNumber(1)
  void clearId() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get streamId => $_getSZ(1);
  @$pb.TagNumber(2)
  set streamId($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasStreamId() => $_has(1);
  @$pb.TagNumber(2)
  void clearStreamId() => $_clearField(2);

  @$pb.TagNumber(3)
  $fixnum.Int64 get snapshotVersion => $_getI64(2);
  @$pb.TagNumber(3)
  set snapshotVersion($fixnum.Int64 value) => $_setInt64(2, value);
  @$pb.TagNumber(3)
  $core.bool hasSnapshotVersion() => $_has(2);
  @$pb.TagNumber(3)
  void clearSnapshotVersion() => $_clearField(3);

  /// タイムスタンプ型を共通型に統一（string → Timestamp）
  @$pb.TagNumber(4)
  $1.Timestamp get createdAt => $_getN(3);
  @$pb.TagNumber(4)
  set createdAt($1.Timestamp value) => $_setField(4, value);
  @$pb.TagNumber(4)
  $core.bool hasCreatedAt() => $_has(3);
  @$pb.TagNumber(4)
  void clearCreatedAt() => $_clearField(4);
  @$pb.TagNumber(4)
  $1.Timestamp ensureCreatedAt() => $_ensure(3);

  @$pb.TagNumber(5)
  $core.String get aggregateType => $_getSZ(4);
  @$pb.TagNumber(5)
  set aggregateType($core.String value) => $_setString(4, value);
  @$pb.TagNumber(5)
  $core.bool hasAggregateType() => $_has(4);
  @$pb.TagNumber(5)
  void clearAggregateType() => $_clearField(5);
}

class GetLatestSnapshotRequest extends $pb.GeneratedMessage {
  factory GetLatestSnapshotRequest({
    $core.String? streamId,
  }) {
    final result = create();
    if (streamId != null) result.streamId = streamId;
    return result;
  }

  GetLatestSnapshotRequest._();

  factory GetLatestSnapshotRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory GetLatestSnapshotRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'GetLatestSnapshotRequest',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.eventstore.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'streamId')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetLatestSnapshotRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetLatestSnapshotRequest copyWith(
          void Function(GetLatestSnapshotRequest) updates) =>
      super.copyWith((message) => updates(message as GetLatestSnapshotRequest))
          as GetLatestSnapshotRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static GetLatestSnapshotRequest create() => GetLatestSnapshotRequest._();
  @$core.override
  GetLatestSnapshotRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static GetLatestSnapshotRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<GetLatestSnapshotRequest>(create);
  static GetLatestSnapshotRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get streamId => $_getSZ(0);
  @$pb.TagNumber(1)
  set streamId($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasStreamId() => $_has(0);
  @$pb.TagNumber(1)
  void clearStreamId() => $_clearField(1);
}

class GetLatestSnapshotResponse extends $pb.GeneratedMessage {
  factory GetLatestSnapshotResponse({
    Snapshot? snapshot,
  }) {
    final result = create();
    if (snapshot != null) result.snapshot = snapshot;
    return result;
  }

  GetLatestSnapshotResponse._();

  factory GetLatestSnapshotResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory GetLatestSnapshotResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'GetLatestSnapshotResponse',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.eventstore.v1'),
      createEmptyInstance: create)
    ..aOM<Snapshot>(1, _omitFieldNames ? '' : 'snapshot',
        subBuilder: Snapshot.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetLatestSnapshotResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetLatestSnapshotResponse copyWith(
          void Function(GetLatestSnapshotResponse) updates) =>
      super.copyWith((message) => updates(message as GetLatestSnapshotResponse))
          as GetLatestSnapshotResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static GetLatestSnapshotResponse create() => GetLatestSnapshotResponse._();
  @$core.override
  GetLatestSnapshotResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static GetLatestSnapshotResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<GetLatestSnapshotResponse>(create);
  static GetLatestSnapshotResponse? _defaultInstance;

  @$pb.TagNumber(1)
  Snapshot get snapshot => $_getN(0);
  @$pb.TagNumber(1)
  set snapshot(Snapshot value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasSnapshot() => $_has(0);
  @$pb.TagNumber(1)
  void clearSnapshot() => $_clearField(1);
  @$pb.TagNumber(1)
  Snapshot ensureSnapshot() => $_ensure(0);
}

class DeleteStreamRequest extends $pb.GeneratedMessage {
  factory DeleteStreamRequest({
    $core.String? streamId,
  }) {
    final result = create();
    if (streamId != null) result.streamId = streamId;
    return result;
  }

  DeleteStreamRequest._();

  factory DeleteStreamRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory DeleteStreamRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'DeleteStreamRequest',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.eventstore.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'streamId')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  DeleteStreamRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  DeleteStreamRequest copyWith(void Function(DeleteStreamRequest) updates) =>
      super.copyWith((message) => updates(message as DeleteStreamRequest))
          as DeleteStreamRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static DeleteStreamRequest create() => DeleteStreamRequest._();
  @$core.override
  DeleteStreamRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static DeleteStreamRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<DeleteStreamRequest>(create);
  static DeleteStreamRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get streamId => $_getSZ(0);
  @$pb.TagNumber(1)
  set streamId($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasStreamId() => $_has(0);
  @$pb.TagNumber(1)
  void clearStreamId() => $_clearField(1);
}

class DeleteStreamResponse extends $pb.GeneratedMessage {
  factory DeleteStreamResponse({
    $core.bool? success,
    $core.String? message,
  }) {
    final result = create();
    if (success != null) result.success = success;
    if (message != null) result.message = message;
    return result;
  }

  DeleteStreamResponse._();

  factory DeleteStreamResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory DeleteStreamResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'DeleteStreamResponse',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.eventstore.v1'),
      createEmptyInstance: create)
    ..aOB(1, _omitFieldNames ? '' : 'success')
    ..aOS(2, _omitFieldNames ? '' : 'message')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  DeleteStreamResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  DeleteStreamResponse copyWith(void Function(DeleteStreamResponse) updates) =>
      super.copyWith((message) => updates(message as DeleteStreamResponse))
          as DeleteStreamResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static DeleteStreamResponse create() => DeleteStreamResponse._();
  @$core.override
  DeleteStreamResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static DeleteStreamResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<DeleteStreamResponse>(create);
  static DeleteStreamResponse? _defaultInstance;

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

class EventData extends $pb.GeneratedMessage {
  factory EventData({
    $core.String? eventType,
    $core.List<$core.int>? payload,
    EventStoreMetadata? metadata,
  }) {
    final result = create();
    if (eventType != null) result.eventType = eventType;
    if (payload != null) result.payload = payload;
    if (metadata != null) result.metadata = metadata;
    return result;
  }

  EventData._();

  factory EventData.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory EventData.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'EventData',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.eventstore.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'eventType')
    ..a<$core.List<$core.int>>(
        2, _omitFieldNames ? '' : 'payload', $pb.PbFieldType.OY)
    ..aOM<EventStoreMetadata>(3, _omitFieldNames ? '' : 'metadata',
        subBuilder: EventStoreMetadata.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  EventData clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  EventData copyWith(void Function(EventData) updates) =>
      super.copyWith((message) => updates(message as EventData)) as EventData;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static EventData create() => EventData._();
  @$core.override
  EventData createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static EventData getDefault() =>
      _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<EventData>(create);
  static EventData? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get eventType => $_getSZ(0);
  @$pb.TagNumber(1)
  set eventType($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasEventType() => $_has(0);
  @$pb.TagNumber(1)
  void clearEventType() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.List<$core.int> get payload => $_getN(1);
  @$pb.TagNumber(2)
  set payload($core.List<$core.int> value) => $_setBytes(1, value);
  @$pb.TagNumber(2)
  $core.bool hasPayload() => $_has(1);
  @$pb.TagNumber(2)
  void clearPayload() => $_clearField(2);

  @$pb.TagNumber(3)
  EventStoreMetadata get metadata => $_getN(2);
  @$pb.TagNumber(3)
  set metadata(EventStoreMetadata value) => $_setField(3, value);
  @$pb.TagNumber(3)
  $core.bool hasMetadata() => $_has(2);
  @$pb.TagNumber(3)
  void clearMetadata() => $_clearField(3);
  @$pb.TagNumber(3)
  EventStoreMetadata ensureMetadata() => $_ensure(2);
}

class StoredEvent extends $pb.GeneratedMessage {
  factory StoredEvent({
    $core.String? streamId,
    $fixnum.Int64? sequence,
    $core.String? eventType,
    $fixnum.Int64? version,
    $core.List<$core.int>? payload,
    EventStoreMetadata? metadata,
    $1.Timestamp? occurredAt,
    $1.Timestamp? storedAt,
  }) {
    final result = create();
    if (streamId != null) result.streamId = streamId;
    if (sequence != null) result.sequence = sequence;
    if (eventType != null) result.eventType = eventType;
    if (version != null) result.version = version;
    if (payload != null) result.payload = payload;
    if (metadata != null) result.metadata = metadata;
    if (occurredAt != null) result.occurredAt = occurredAt;
    if (storedAt != null) result.storedAt = storedAt;
    return result;
  }

  StoredEvent._();

  factory StoredEvent.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory StoredEvent.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'StoredEvent',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.eventstore.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'streamId')
    ..a<$fixnum.Int64>(
        2, _omitFieldNames ? '' : 'sequence', $pb.PbFieldType.OU6,
        defaultOrMaker: $fixnum.Int64.ZERO)
    ..aOS(3, _omitFieldNames ? '' : 'eventType')
    ..aInt64(4, _omitFieldNames ? '' : 'version')
    ..a<$core.List<$core.int>>(
        5, _omitFieldNames ? '' : 'payload', $pb.PbFieldType.OY)
    ..aOM<EventStoreMetadata>(6, _omitFieldNames ? '' : 'metadata',
        subBuilder: EventStoreMetadata.create)
    ..aOM<$1.Timestamp>(7, _omitFieldNames ? '' : 'occurredAt',
        subBuilder: $1.Timestamp.create)
    ..aOM<$1.Timestamp>(8, _omitFieldNames ? '' : 'storedAt',
        subBuilder: $1.Timestamp.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  StoredEvent clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  StoredEvent copyWith(void Function(StoredEvent) updates) =>
      super.copyWith((message) => updates(message as StoredEvent))
          as StoredEvent;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static StoredEvent create() => StoredEvent._();
  @$core.override
  StoredEvent createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static StoredEvent getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<StoredEvent>(create);
  static StoredEvent? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get streamId => $_getSZ(0);
  @$pb.TagNumber(1)
  set streamId($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasStreamId() => $_has(0);
  @$pb.TagNumber(1)
  void clearStreamId() => $_clearField(1);

  @$pb.TagNumber(2)
  $fixnum.Int64 get sequence => $_getI64(1);
  @$pb.TagNumber(2)
  set sequence($fixnum.Int64 value) => $_setInt64(1, value);
  @$pb.TagNumber(2)
  $core.bool hasSequence() => $_has(1);
  @$pb.TagNumber(2)
  void clearSequence() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.String get eventType => $_getSZ(2);
  @$pb.TagNumber(3)
  set eventType($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasEventType() => $_has(2);
  @$pb.TagNumber(3)
  void clearEventType() => $_clearField(3);

  @$pb.TagNumber(4)
  $fixnum.Int64 get version => $_getI64(3);
  @$pb.TagNumber(4)
  set version($fixnum.Int64 value) => $_setInt64(3, value);
  @$pb.TagNumber(4)
  $core.bool hasVersion() => $_has(3);
  @$pb.TagNumber(4)
  void clearVersion() => $_clearField(4);

  @$pb.TagNumber(5)
  $core.List<$core.int> get payload => $_getN(4);
  @$pb.TagNumber(5)
  set payload($core.List<$core.int> value) => $_setBytes(4, value);
  @$pb.TagNumber(5)
  $core.bool hasPayload() => $_has(4);
  @$pb.TagNumber(5)
  void clearPayload() => $_clearField(5);

  @$pb.TagNumber(6)
  EventStoreMetadata get metadata => $_getN(5);
  @$pb.TagNumber(6)
  set metadata(EventStoreMetadata value) => $_setField(6, value);
  @$pb.TagNumber(6)
  $core.bool hasMetadata() => $_has(5);
  @$pb.TagNumber(6)
  void clearMetadata() => $_clearField(6);
  @$pb.TagNumber(6)
  EventStoreMetadata ensureMetadata() => $_ensure(5);

  /// タイムスタンプ型を共通型に統一（string → Timestamp）
  @$pb.TagNumber(7)
  $1.Timestamp get occurredAt => $_getN(6);
  @$pb.TagNumber(7)
  set occurredAt($1.Timestamp value) => $_setField(7, value);
  @$pb.TagNumber(7)
  $core.bool hasOccurredAt() => $_has(6);
  @$pb.TagNumber(7)
  void clearOccurredAt() => $_clearField(7);
  @$pb.TagNumber(7)
  $1.Timestamp ensureOccurredAt() => $_ensure(6);

  /// タイムスタンプ型を共通型に統一（string → Timestamp）
  @$pb.TagNumber(8)
  $1.Timestamp get storedAt => $_getN(7);
  @$pb.TagNumber(8)
  set storedAt($1.Timestamp value) => $_setField(8, value);
  @$pb.TagNumber(8)
  $core.bool hasStoredAt() => $_has(7);
  @$pb.TagNumber(8)
  void clearStoredAt() => $_clearField(8);
  @$pb.TagNumber(8)
  $1.Timestamp ensureStoredAt() => $_ensure(7);
}

class EventStoreMetadata extends $pb.GeneratedMessage {
  factory EventStoreMetadata({
    $core.String? actorId,
    $core.String? correlationId,
    $core.String? causationId,
  }) {
    final result = create();
    if (actorId != null) result.actorId = actorId;
    if (correlationId != null) result.correlationId = correlationId;
    if (causationId != null) result.causationId = causationId;
    return result;
  }

  EventStoreMetadata._();

  factory EventStoreMetadata.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory EventStoreMetadata.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'EventStoreMetadata',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.eventstore.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'actorId')
    ..aOS(2, _omitFieldNames ? '' : 'correlationId')
    ..aOS(3, _omitFieldNames ? '' : 'causationId')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  EventStoreMetadata clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  EventStoreMetadata copyWith(void Function(EventStoreMetadata) updates) =>
      super.copyWith((message) => updates(message as EventStoreMetadata))
          as EventStoreMetadata;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static EventStoreMetadata create() => EventStoreMetadata._();
  @$core.override
  EventStoreMetadata createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static EventStoreMetadata getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<EventStoreMetadata>(create);
  static EventStoreMetadata? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get actorId => $_getSZ(0);
  @$pb.TagNumber(1)
  set actorId($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasActorId() => $_has(0);
  @$pb.TagNumber(1)
  void clearActorId() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get correlationId => $_getSZ(1);
  @$pb.TagNumber(2)
  set correlationId($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasCorrelationId() => $_has(1);
  @$pb.TagNumber(2)
  void clearCorrelationId() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.String get causationId => $_getSZ(2);
  @$pb.TagNumber(3)
  set causationId($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasCausationId() => $_has(2);
  @$pb.TagNumber(3)
  void clearCausationId() => $_clearField(3);
}

class Snapshot extends $pb.GeneratedMessage {
  factory Snapshot({
    $core.String? id,
    $core.String? streamId,
    $fixnum.Int64? snapshotVersion,
    $core.String? aggregateType,
    $core.List<$core.int>? state,
    $1.Timestamp? createdAt,
  }) {
    final result = create();
    if (id != null) result.id = id;
    if (streamId != null) result.streamId = streamId;
    if (snapshotVersion != null) result.snapshotVersion = snapshotVersion;
    if (aggregateType != null) result.aggregateType = aggregateType;
    if (state != null) result.state = state;
    if (createdAt != null) result.createdAt = createdAt;
    return result;
  }

  Snapshot._();

  factory Snapshot.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory Snapshot.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'Snapshot',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.eventstore.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'id')
    ..aOS(2, _omitFieldNames ? '' : 'streamId')
    ..aInt64(3, _omitFieldNames ? '' : 'snapshotVersion')
    ..aOS(4, _omitFieldNames ? '' : 'aggregateType')
    ..a<$core.List<$core.int>>(
        5, _omitFieldNames ? '' : 'state', $pb.PbFieldType.OY)
    ..aOM<$1.Timestamp>(6, _omitFieldNames ? '' : 'createdAt',
        subBuilder: $1.Timestamp.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  Snapshot clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  Snapshot copyWith(void Function(Snapshot) updates) =>
      super.copyWith((message) => updates(message as Snapshot)) as Snapshot;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static Snapshot create() => Snapshot._();
  @$core.override
  Snapshot createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static Snapshot getDefault() =>
      _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<Snapshot>(create);
  static Snapshot? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get id => $_getSZ(0);
  @$pb.TagNumber(1)
  set id($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasId() => $_has(0);
  @$pb.TagNumber(1)
  void clearId() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get streamId => $_getSZ(1);
  @$pb.TagNumber(2)
  set streamId($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasStreamId() => $_has(1);
  @$pb.TagNumber(2)
  void clearStreamId() => $_clearField(2);

  @$pb.TagNumber(3)
  $fixnum.Int64 get snapshotVersion => $_getI64(2);
  @$pb.TagNumber(3)
  set snapshotVersion($fixnum.Int64 value) => $_setInt64(2, value);
  @$pb.TagNumber(3)
  $core.bool hasSnapshotVersion() => $_has(2);
  @$pb.TagNumber(3)
  void clearSnapshotVersion() => $_clearField(3);

  @$pb.TagNumber(4)
  $core.String get aggregateType => $_getSZ(3);
  @$pb.TagNumber(4)
  set aggregateType($core.String value) => $_setString(3, value);
  @$pb.TagNumber(4)
  $core.bool hasAggregateType() => $_has(3);
  @$pb.TagNumber(4)
  void clearAggregateType() => $_clearField(4);

  @$pb.TagNumber(5)
  $core.List<$core.int> get state => $_getN(4);
  @$pb.TagNumber(5)
  set state($core.List<$core.int> value) => $_setBytes(4, value);
  @$pb.TagNumber(5)
  $core.bool hasState() => $_has(4);
  @$pb.TagNumber(5)
  void clearState() => $_clearField(5);

  /// タイムスタンプ型を共通型に統一（string → Timestamp）
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
}

const $core.bool _omitFieldNames =
    $core.bool.fromEnvironment('protobuf.omit_field_names');
const $core.bool _omitMessageNames =
    $core.bool.fromEnvironment('protobuf.omit_message_names');
