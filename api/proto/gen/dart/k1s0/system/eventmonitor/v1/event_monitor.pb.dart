// This is a generated file - do not edit.
//
// Generated from k1s0/system/eventmonitor/v1/event_monitor.proto.

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

class EventRecord extends $pb.GeneratedMessage {
  factory EventRecord({
    $core.String? id,
    $core.String? correlationId,
    $core.String? eventType,
    $core.String? source,
    $core.String? domain,
    $core.String? traceId,
    $1.Timestamp? timestamp,
    $core.String? flowId,
    $core.int? flowStepIndex,
    $core.String? status,
  }) {
    final result = create();
    if (id != null) result.id = id;
    if (correlationId != null) result.correlationId = correlationId;
    if (eventType != null) result.eventType = eventType;
    if (source != null) result.source = source;
    if (domain != null) result.domain = domain;
    if (traceId != null) result.traceId = traceId;
    if (timestamp != null) result.timestamp = timestamp;
    if (flowId != null) result.flowId = flowId;
    if (flowStepIndex != null) result.flowStepIndex = flowStepIndex;
    if (status != null) result.status = status;
    return result;
  }

  EventRecord._();

  factory EventRecord.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory EventRecord.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'EventRecord',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.eventmonitor.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'id')
    ..aOS(2, _omitFieldNames ? '' : 'correlationId')
    ..aOS(3, _omitFieldNames ? '' : 'eventType')
    ..aOS(4, _omitFieldNames ? '' : 'source')
    ..aOS(5, _omitFieldNames ? '' : 'domain')
    ..aOS(6, _omitFieldNames ? '' : 'traceId')
    ..aOM<$1.Timestamp>(7, _omitFieldNames ? '' : 'timestamp',
        subBuilder: $1.Timestamp.create)
    ..aOS(8, _omitFieldNames ? '' : 'flowId')
    ..aI(9, _omitFieldNames ? '' : 'flowStepIndex')
    ..aOS(10, _omitFieldNames ? '' : 'status')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  EventRecord clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  EventRecord copyWith(void Function(EventRecord) updates) =>
      super.copyWith((message) => updates(message as EventRecord))
          as EventRecord;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static EventRecord create() => EventRecord._();
  @$core.override
  EventRecord createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static EventRecord getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<EventRecord>(create);
  static EventRecord? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get id => $_getSZ(0);
  @$pb.TagNumber(1)
  set id($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasId() => $_has(0);
  @$pb.TagNumber(1)
  void clearId() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get correlationId => $_getSZ(1);
  @$pb.TagNumber(2)
  set correlationId($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasCorrelationId() => $_has(1);
  @$pb.TagNumber(2)
  void clearCorrelationId() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.String get eventType => $_getSZ(2);
  @$pb.TagNumber(3)
  set eventType($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasEventType() => $_has(2);
  @$pb.TagNumber(3)
  void clearEventType() => $_clearField(3);

  @$pb.TagNumber(4)
  $core.String get source => $_getSZ(3);
  @$pb.TagNumber(4)
  set source($core.String value) => $_setString(3, value);
  @$pb.TagNumber(4)
  $core.bool hasSource() => $_has(3);
  @$pb.TagNumber(4)
  void clearSource() => $_clearField(4);

  @$pb.TagNumber(5)
  $core.String get domain => $_getSZ(4);
  @$pb.TagNumber(5)
  set domain($core.String value) => $_setString(4, value);
  @$pb.TagNumber(5)
  $core.bool hasDomain() => $_has(4);
  @$pb.TagNumber(5)
  void clearDomain() => $_clearField(5);

  @$pb.TagNumber(6)
  $core.String get traceId => $_getSZ(5);
  @$pb.TagNumber(6)
  set traceId($core.String value) => $_setString(5, value);
  @$pb.TagNumber(6)
  $core.bool hasTraceId() => $_has(5);
  @$pb.TagNumber(6)
  void clearTraceId() => $_clearField(6);

  @$pb.TagNumber(7)
  $1.Timestamp get timestamp => $_getN(6);
  @$pb.TagNumber(7)
  set timestamp($1.Timestamp value) => $_setField(7, value);
  @$pb.TagNumber(7)
  $core.bool hasTimestamp() => $_has(6);
  @$pb.TagNumber(7)
  void clearTimestamp() => $_clearField(7);
  @$pb.TagNumber(7)
  $1.Timestamp ensureTimestamp() => $_ensure(6);

  @$pb.TagNumber(8)
  $core.String get flowId => $_getSZ(7);
  @$pb.TagNumber(8)
  set flowId($core.String value) => $_setString(7, value);
  @$pb.TagNumber(8)
  $core.bool hasFlowId() => $_has(7);
  @$pb.TagNumber(8)
  void clearFlowId() => $_clearField(8);

  @$pb.TagNumber(9)
  $core.int get flowStepIndex => $_getIZ(8);
  @$pb.TagNumber(9)
  set flowStepIndex($core.int value) => $_setSignedInt32(8, value);
  @$pb.TagNumber(9)
  $core.bool hasFlowStepIndex() => $_has(8);
  @$pb.TagNumber(9)
  void clearFlowStepIndex() => $_clearField(9);

  @$pb.TagNumber(10)
  $core.String get status => $_getSZ(9);
  @$pb.TagNumber(10)
  set status($core.String value) => $_setString(9, value);
  @$pb.TagNumber(10)
  $core.bool hasStatus() => $_has(9);
  @$pb.TagNumber(10)
  void clearStatus() => $_clearField(10);
}

class ListEventsRequest extends $pb.GeneratedMessage {
  factory ListEventsRequest({
    $1.Pagination? pagination,
    $core.String? domain,
    $core.String? eventType,
    $core.String? source,
    $1.Timestamp? from,
    $1.Timestamp? to,
    $core.String? status,
  }) {
    final result = create();
    if (pagination != null) result.pagination = pagination;
    if (domain != null) result.domain = domain;
    if (eventType != null) result.eventType = eventType;
    if (source != null) result.source = source;
    if (from != null) result.from = from;
    if (to != null) result.to = to;
    if (status != null) result.status = status;
    return result;
  }

  ListEventsRequest._();

  factory ListEventsRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory ListEventsRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ListEventsRequest',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.eventmonitor.v1'),
      createEmptyInstance: create)
    ..aOM<$1.Pagination>(1, _omitFieldNames ? '' : 'pagination',
        subBuilder: $1.Pagination.create)
    ..aOS(2, _omitFieldNames ? '' : 'domain')
    ..aOS(3, _omitFieldNames ? '' : 'eventType')
    ..aOS(4, _omitFieldNames ? '' : 'source')
    ..aOM<$1.Timestamp>(5, _omitFieldNames ? '' : 'from',
        subBuilder: $1.Timestamp.create)
    ..aOM<$1.Timestamp>(6, _omitFieldNames ? '' : 'to',
        subBuilder: $1.Timestamp.create)
    ..aOS(7, _omitFieldNames ? '' : 'status')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListEventsRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListEventsRequest copyWith(void Function(ListEventsRequest) updates) =>
      super.copyWith((message) => updates(message as ListEventsRequest))
          as ListEventsRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static ListEventsRequest create() => ListEventsRequest._();
  @$core.override
  ListEventsRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static ListEventsRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ListEventsRequest>(create);
  static ListEventsRequest? _defaultInstance;

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

  @$pb.TagNumber(2)
  $core.String get domain => $_getSZ(1);
  @$pb.TagNumber(2)
  set domain($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasDomain() => $_has(1);
  @$pb.TagNumber(2)
  void clearDomain() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.String get eventType => $_getSZ(2);
  @$pb.TagNumber(3)
  set eventType($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasEventType() => $_has(2);
  @$pb.TagNumber(3)
  void clearEventType() => $_clearField(3);

  @$pb.TagNumber(4)
  $core.String get source => $_getSZ(3);
  @$pb.TagNumber(4)
  set source($core.String value) => $_setString(3, value);
  @$pb.TagNumber(4)
  $core.bool hasSource() => $_has(3);
  @$pb.TagNumber(4)
  void clearSource() => $_clearField(4);

  @$pb.TagNumber(5)
  $1.Timestamp get from => $_getN(4);
  @$pb.TagNumber(5)
  set from($1.Timestamp value) => $_setField(5, value);
  @$pb.TagNumber(5)
  $core.bool hasFrom() => $_has(4);
  @$pb.TagNumber(5)
  void clearFrom() => $_clearField(5);
  @$pb.TagNumber(5)
  $1.Timestamp ensureFrom() => $_ensure(4);

  @$pb.TagNumber(6)
  $1.Timestamp get to => $_getN(5);
  @$pb.TagNumber(6)
  set to($1.Timestamp value) => $_setField(6, value);
  @$pb.TagNumber(6)
  $core.bool hasTo() => $_has(5);
  @$pb.TagNumber(6)
  void clearTo() => $_clearField(6);
  @$pb.TagNumber(6)
  $1.Timestamp ensureTo() => $_ensure(5);

  @$pb.TagNumber(7)
  $core.String get status => $_getSZ(6);
  @$pb.TagNumber(7)
  set status($core.String value) => $_setString(6, value);
  @$pb.TagNumber(7)
  $core.bool hasStatus() => $_has(6);
  @$pb.TagNumber(7)
  void clearStatus() => $_clearField(7);
}

class ListEventsResponse extends $pb.GeneratedMessage {
  factory ListEventsResponse({
    $core.Iterable<EventRecord>? events,
    $1.PaginationResult? pagination,
  }) {
    final result = create();
    if (events != null) result.events.addAll(events);
    if (pagination != null) result.pagination = pagination;
    return result;
  }

  ListEventsResponse._();

  factory ListEventsResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory ListEventsResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ListEventsResponse',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.eventmonitor.v1'),
      createEmptyInstance: create)
    ..pPM<EventRecord>(1, _omitFieldNames ? '' : 'events',
        subBuilder: EventRecord.create)
    ..aOM<$1.PaginationResult>(2, _omitFieldNames ? '' : 'pagination',
        subBuilder: $1.PaginationResult.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListEventsResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListEventsResponse copyWith(void Function(ListEventsResponse) updates) =>
      super.copyWith((message) => updates(message as ListEventsResponse))
          as ListEventsResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static ListEventsResponse create() => ListEventsResponse._();
  @$core.override
  ListEventsResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static ListEventsResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ListEventsResponse>(create);
  static ListEventsResponse? _defaultInstance;

  @$pb.TagNumber(1)
  $pb.PbList<EventRecord> get events => $_getList(0);

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

class TraceByCorrelationRequest extends $pb.GeneratedMessage {
  factory TraceByCorrelationRequest({
    $core.String? correlationId,
  }) {
    final result = create();
    if (correlationId != null) result.correlationId = correlationId;
    return result;
  }

  TraceByCorrelationRequest._();

  factory TraceByCorrelationRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory TraceByCorrelationRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'TraceByCorrelationRequest',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.eventmonitor.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'correlationId')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  TraceByCorrelationRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  TraceByCorrelationRequest copyWith(
          void Function(TraceByCorrelationRequest) updates) =>
      super.copyWith((message) => updates(message as TraceByCorrelationRequest))
          as TraceByCorrelationRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static TraceByCorrelationRequest create() => TraceByCorrelationRequest._();
  @$core.override
  TraceByCorrelationRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static TraceByCorrelationRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<TraceByCorrelationRequest>(create);
  static TraceByCorrelationRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get correlationId => $_getSZ(0);
  @$pb.TagNumber(1)
  set correlationId($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasCorrelationId() => $_has(0);
  @$pb.TagNumber(1)
  void clearCorrelationId() => $_clearField(1);
}

class TraceEvent extends $pb.GeneratedMessage {
  factory TraceEvent({
    $core.String? id,
    $core.String? eventType,
    $core.String? source,
    $1.Timestamp? timestamp,
    $core.int? stepIndex,
    $core.String? status,
    $fixnum.Int64? durationFromPreviousMs,
  }) {
    final result = create();
    if (id != null) result.id = id;
    if (eventType != null) result.eventType = eventType;
    if (source != null) result.source = source;
    if (timestamp != null) result.timestamp = timestamp;
    if (stepIndex != null) result.stepIndex = stepIndex;
    if (status != null) result.status = status;
    if (durationFromPreviousMs != null)
      result.durationFromPreviousMs = durationFromPreviousMs;
    return result;
  }

  TraceEvent._();

  factory TraceEvent.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory TraceEvent.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'TraceEvent',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.eventmonitor.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'id')
    ..aOS(2, _omitFieldNames ? '' : 'eventType')
    ..aOS(3, _omitFieldNames ? '' : 'source')
    ..aOM<$1.Timestamp>(4, _omitFieldNames ? '' : 'timestamp',
        subBuilder: $1.Timestamp.create)
    ..aI(5, _omitFieldNames ? '' : 'stepIndex')
    ..aOS(6, _omitFieldNames ? '' : 'status')
    ..aInt64(7, _omitFieldNames ? '' : 'durationFromPreviousMs')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  TraceEvent clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  TraceEvent copyWith(void Function(TraceEvent) updates) =>
      super.copyWith((message) => updates(message as TraceEvent)) as TraceEvent;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static TraceEvent create() => TraceEvent._();
  @$core.override
  TraceEvent createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static TraceEvent getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<TraceEvent>(create);
  static TraceEvent? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get id => $_getSZ(0);
  @$pb.TagNumber(1)
  set id($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasId() => $_has(0);
  @$pb.TagNumber(1)
  void clearId() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get eventType => $_getSZ(1);
  @$pb.TagNumber(2)
  set eventType($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasEventType() => $_has(1);
  @$pb.TagNumber(2)
  void clearEventType() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.String get source => $_getSZ(2);
  @$pb.TagNumber(3)
  set source($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasSource() => $_has(2);
  @$pb.TagNumber(3)
  void clearSource() => $_clearField(3);

  @$pb.TagNumber(4)
  $1.Timestamp get timestamp => $_getN(3);
  @$pb.TagNumber(4)
  set timestamp($1.Timestamp value) => $_setField(4, value);
  @$pb.TagNumber(4)
  $core.bool hasTimestamp() => $_has(3);
  @$pb.TagNumber(4)
  void clearTimestamp() => $_clearField(4);
  @$pb.TagNumber(4)
  $1.Timestamp ensureTimestamp() => $_ensure(3);

  @$pb.TagNumber(5)
  $core.int get stepIndex => $_getIZ(4);
  @$pb.TagNumber(5)
  set stepIndex($core.int value) => $_setSignedInt32(4, value);
  @$pb.TagNumber(5)
  $core.bool hasStepIndex() => $_has(4);
  @$pb.TagNumber(5)
  void clearStepIndex() => $_clearField(5);

  @$pb.TagNumber(6)
  $core.String get status => $_getSZ(5);
  @$pb.TagNumber(6)
  set status($core.String value) => $_setString(5, value);
  @$pb.TagNumber(6)
  $core.bool hasStatus() => $_has(5);
  @$pb.TagNumber(6)
  void clearStatus() => $_clearField(6);

  @$pb.TagNumber(7)
  $fixnum.Int64 get durationFromPreviousMs => $_getI64(6);
  @$pb.TagNumber(7)
  set durationFromPreviousMs($fixnum.Int64 value) => $_setInt64(6, value);
  @$pb.TagNumber(7)
  $core.bool hasDurationFromPreviousMs() => $_has(6);
  @$pb.TagNumber(7)
  void clearDurationFromPreviousMs() => $_clearField(7);
}

class PendingStep extends $pb.GeneratedMessage {
  factory PendingStep({
    $core.String? eventType,
    $core.String? source,
    $core.int? stepIndex,
    $core.int? timeoutSeconds,
    $fixnum.Int64? waitingSinceSeconds,
  }) {
    final result = create();
    if (eventType != null) result.eventType = eventType;
    if (source != null) result.source = source;
    if (stepIndex != null) result.stepIndex = stepIndex;
    if (timeoutSeconds != null) result.timeoutSeconds = timeoutSeconds;
    if (waitingSinceSeconds != null)
      result.waitingSinceSeconds = waitingSinceSeconds;
    return result;
  }

  PendingStep._();

  factory PendingStep.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory PendingStep.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'PendingStep',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.eventmonitor.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'eventType')
    ..aOS(2, _omitFieldNames ? '' : 'source')
    ..aI(3, _omitFieldNames ? '' : 'stepIndex')
    ..aI(4, _omitFieldNames ? '' : 'timeoutSeconds')
    ..aInt64(5, _omitFieldNames ? '' : 'waitingSinceSeconds')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  PendingStep clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  PendingStep copyWith(void Function(PendingStep) updates) =>
      super.copyWith((message) => updates(message as PendingStep))
          as PendingStep;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static PendingStep create() => PendingStep._();
  @$core.override
  PendingStep createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static PendingStep getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<PendingStep>(create);
  static PendingStep? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get eventType => $_getSZ(0);
  @$pb.TagNumber(1)
  set eventType($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasEventType() => $_has(0);
  @$pb.TagNumber(1)
  void clearEventType() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get source => $_getSZ(1);
  @$pb.TagNumber(2)
  set source($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasSource() => $_has(1);
  @$pb.TagNumber(2)
  void clearSource() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.int get stepIndex => $_getIZ(2);
  @$pb.TagNumber(3)
  set stepIndex($core.int value) => $_setSignedInt32(2, value);
  @$pb.TagNumber(3)
  $core.bool hasStepIndex() => $_has(2);
  @$pb.TagNumber(3)
  void clearStepIndex() => $_clearField(3);

  @$pb.TagNumber(4)
  $core.int get timeoutSeconds => $_getIZ(3);
  @$pb.TagNumber(4)
  set timeoutSeconds($core.int value) => $_setSignedInt32(3, value);
  @$pb.TagNumber(4)
  $core.bool hasTimeoutSeconds() => $_has(3);
  @$pb.TagNumber(4)
  void clearTimeoutSeconds() => $_clearField(4);

  @$pb.TagNumber(5)
  $fixnum.Int64 get waitingSinceSeconds => $_getI64(4);
  @$pb.TagNumber(5)
  set waitingSinceSeconds($fixnum.Int64 value) => $_setInt64(4, value);
  @$pb.TagNumber(5)
  $core.bool hasWaitingSinceSeconds() => $_has(4);
  @$pb.TagNumber(5)
  void clearWaitingSinceSeconds() => $_clearField(5);
}

class FlowSummary extends $pb.GeneratedMessage {
  factory FlowSummary({
    $core.String? id,
    $core.String? name,
    $core.String? status,
    $1.Timestamp? startedAt,
    $fixnum.Int64? elapsedSeconds,
  }) {
    final result = create();
    if (id != null) result.id = id;
    if (name != null) result.name = name;
    if (status != null) result.status = status;
    if (startedAt != null) result.startedAt = startedAt;
    if (elapsedSeconds != null) result.elapsedSeconds = elapsedSeconds;
    return result;
  }

  FlowSummary._();

  factory FlowSummary.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory FlowSummary.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'FlowSummary',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.eventmonitor.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'id')
    ..aOS(2, _omitFieldNames ? '' : 'name')
    ..aOS(3, _omitFieldNames ? '' : 'status')
    ..aOM<$1.Timestamp>(4, _omitFieldNames ? '' : 'startedAt',
        subBuilder: $1.Timestamp.create)
    ..aInt64(5, _omitFieldNames ? '' : 'elapsedSeconds')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  FlowSummary clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  FlowSummary copyWith(void Function(FlowSummary) updates) =>
      super.copyWith((message) => updates(message as FlowSummary))
          as FlowSummary;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static FlowSummary create() => FlowSummary._();
  @$core.override
  FlowSummary createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static FlowSummary getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<FlowSummary>(create);
  static FlowSummary? _defaultInstance;

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
  $core.String get status => $_getSZ(2);
  @$pb.TagNumber(3)
  set status($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasStatus() => $_has(2);
  @$pb.TagNumber(3)
  void clearStatus() => $_clearField(3);

  @$pb.TagNumber(4)
  $1.Timestamp get startedAt => $_getN(3);
  @$pb.TagNumber(4)
  set startedAt($1.Timestamp value) => $_setField(4, value);
  @$pb.TagNumber(4)
  $core.bool hasStartedAt() => $_has(3);
  @$pb.TagNumber(4)
  void clearStartedAt() => $_clearField(4);
  @$pb.TagNumber(4)
  $1.Timestamp ensureStartedAt() => $_ensure(3);

  @$pb.TagNumber(5)
  $fixnum.Int64 get elapsedSeconds => $_getI64(4);
  @$pb.TagNumber(5)
  set elapsedSeconds($fixnum.Int64 value) => $_setInt64(4, value);
  @$pb.TagNumber(5)
  $core.bool hasElapsedSeconds() => $_has(4);
  @$pb.TagNumber(5)
  void clearElapsedSeconds() => $_clearField(5);
}

class TraceByCorrelationResponse extends $pb.GeneratedMessage {
  factory TraceByCorrelationResponse({
    $core.String? correlationId,
    FlowSummary? flow,
    $core.Iterable<TraceEvent>? events,
    $core.Iterable<PendingStep>? pendingSteps,
  }) {
    final result = create();
    if (correlationId != null) result.correlationId = correlationId;
    if (flow != null) result.flow = flow;
    if (events != null) result.events.addAll(events);
    if (pendingSteps != null) result.pendingSteps.addAll(pendingSteps);
    return result;
  }

  TraceByCorrelationResponse._();

  factory TraceByCorrelationResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory TraceByCorrelationResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'TraceByCorrelationResponse',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.eventmonitor.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'correlationId')
    ..aOM<FlowSummary>(2, _omitFieldNames ? '' : 'flow',
        subBuilder: FlowSummary.create)
    ..pPM<TraceEvent>(3, _omitFieldNames ? '' : 'events',
        subBuilder: TraceEvent.create)
    ..pPM<PendingStep>(4, _omitFieldNames ? '' : 'pendingSteps',
        subBuilder: PendingStep.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  TraceByCorrelationResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  TraceByCorrelationResponse copyWith(
          void Function(TraceByCorrelationResponse) updates) =>
      super.copyWith(
              (message) => updates(message as TraceByCorrelationResponse))
          as TraceByCorrelationResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static TraceByCorrelationResponse create() => TraceByCorrelationResponse._();
  @$core.override
  TraceByCorrelationResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static TraceByCorrelationResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<TraceByCorrelationResponse>(create);
  static TraceByCorrelationResponse? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get correlationId => $_getSZ(0);
  @$pb.TagNumber(1)
  set correlationId($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasCorrelationId() => $_has(0);
  @$pb.TagNumber(1)
  void clearCorrelationId() => $_clearField(1);

  @$pb.TagNumber(2)
  FlowSummary get flow => $_getN(1);
  @$pb.TagNumber(2)
  set flow(FlowSummary value) => $_setField(2, value);
  @$pb.TagNumber(2)
  $core.bool hasFlow() => $_has(1);
  @$pb.TagNumber(2)
  void clearFlow() => $_clearField(2);
  @$pb.TagNumber(2)
  FlowSummary ensureFlow() => $_ensure(1);

  @$pb.TagNumber(3)
  $pb.PbList<TraceEvent> get events => $_getList(2);

  @$pb.TagNumber(4)
  $pb.PbList<PendingStep> get pendingSteps => $_getList(3);
}

class FlowStep extends $pb.GeneratedMessage {
  factory FlowStep({
    $core.String? eventType,
    $core.String? source,
    $core.int? timeoutSeconds,
    $core.String? description,
  }) {
    final result = create();
    if (eventType != null) result.eventType = eventType;
    if (source != null) result.source = source;
    if (timeoutSeconds != null) result.timeoutSeconds = timeoutSeconds;
    if (description != null) result.description = description;
    return result;
  }

  FlowStep._();

  factory FlowStep.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory FlowStep.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'FlowStep',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.eventmonitor.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'eventType')
    ..aOS(2, _omitFieldNames ? '' : 'source')
    ..aI(3, _omitFieldNames ? '' : 'timeoutSeconds')
    ..aOS(4, _omitFieldNames ? '' : 'description')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  FlowStep clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  FlowStep copyWith(void Function(FlowStep) updates) =>
      super.copyWith((message) => updates(message as FlowStep)) as FlowStep;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static FlowStep create() => FlowStep._();
  @$core.override
  FlowStep createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static FlowStep getDefault() =>
      _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<FlowStep>(create);
  static FlowStep? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get eventType => $_getSZ(0);
  @$pb.TagNumber(1)
  set eventType($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasEventType() => $_has(0);
  @$pb.TagNumber(1)
  void clearEventType() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get source => $_getSZ(1);
  @$pb.TagNumber(2)
  set source($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasSource() => $_has(1);
  @$pb.TagNumber(2)
  void clearSource() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.int get timeoutSeconds => $_getIZ(2);
  @$pb.TagNumber(3)
  set timeoutSeconds($core.int value) => $_setSignedInt32(2, value);
  @$pb.TagNumber(3)
  $core.bool hasTimeoutSeconds() => $_has(2);
  @$pb.TagNumber(3)
  void clearTimeoutSeconds() => $_clearField(3);

  @$pb.TagNumber(4)
  $core.String get description => $_getSZ(3);
  @$pb.TagNumber(4)
  set description($core.String value) => $_setString(3, value);
  @$pb.TagNumber(4)
  $core.bool hasDescription() => $_has(3);
  @$pb.TagNumber(4)
  void clearDescription() => $_clearField(4);
}

class FlowSlo extends $pb.GeneratedMessage {
  factory FlowSlo({
    $core.int? targetCompletionSeconds,
    $core.double? targetSuccessRate,
    $core.bool? alertOnViolation,
  }) {
    final result = create();
    if (targetCompletionSeconds != null)
      result.targetCompletionSeconds = targetCompletionSeconds;
    if (targetSuccessRate != null) result.targetSuccessRate = targetSuccessRate;
    if (alertOnViolation != null) result.alertOnViolation = alertOnViolation;
    return result;
  }

  FlowSlo._();

  factory FlowSlo.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory FlowSlo.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'FlowSlo',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.eventmonitor.v1'),
      createEmptyInstance: create)
    ..aI(1, _omitFieldNames ? '' : 'targetCompletionSeconds')
    ..aD(2, _omitFieldNames ? '' : 'targetSuccessRate')
    ..aOB(3, _omitFieldNames ? '' : 'alertOnViolation')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  FlowSlo clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  FlowSlo copyWith(void Function(FlowSlo) updates) =>
      super.copyWith((message) => updates(message as FlowSlo)) as FlowSlo;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static FlowSlo create() => FlowSlo._();
  @$core.override
  FlowSlo createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static FlowSlo getDefault() =>
      _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<FlowSlo>(create);
  static FlowSlo? _defaultInstance;

  @$pb.TagNumber(1)
  $core.int get targetCompletionSeconds => $_getIZ(0);
  @$pb.TagNumber(1)
  set targetCompletionSeconds($core.int value) => $_setSignedInt32(0, value);
  @$pb.TagNumber(1)
  $core.bool hasTargetCompletionSeconds() => $_has(0);
  @$pb.TagNumber(1)
  void clearTargetCompletionSeconds() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.double get targetSuccessRate => $_getN(1);
  @$pb.TagNumber(2)
  set targetSuccessRate($core.double value) => $_setDouble(1, value);
  @$pb.TagNumber(2)
  $core.bool hasTargetSuccessRate() => $_has(1);
  @$pb.TagNumber(2)
  void clearTargetSuccessRate() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.bool get alertOnViolation => $_getBF(2);
  @$pb.TagNumber(3)
  set alertOnViolation($core.bool value) => $_setBool(2, value);
  @$pb.TagNumber(3)
  $core.bool hasAlertOnViolation() => $_has(2);
  @$pb.TagNumber(3)
  void clearAlertOnViolation() => $_clearField(3);
}

class FlowDefinition extends $pb.GeneratedMessage {
  factory FlowDefinition({
    $core.String? id,
    $core.String? name,
    $core.String? description,
    $core.String? domain,
    $core.Iterable<FlowStep>? steps,
    FlowSlo? slo,
    $core.bool? enabled,
    $1.Timestamp? createdAt,
    $1.Timestamp? updatedAt,
  }) {
    final result = create();
    if (id != null) result.id = id;
    if (name != null) result.name = name;
    if (description != null) result.description = description;
    if (domain != null) result.domain = domain;
    if (steps != null) result.steps.addAll(steps);
    if (slo != null) result.slo = slo;
    if (enabled != null) result.enabled = enabled;
    if (createdAt != null) result.createdAt = createdAt;
    if (updatedAt != null) result.updatedAt = updatedAt;
    return result;
  }

  FlowDefinition._();

  factory FlowDefinition.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory FlowDefinition.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'FlowDefinition',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.eventmonitor.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'id')
    ..aOS(2, _omitFieldNames ? '' : 'name')
    ..aOS(3, _omitFieldNames ? '' : 'description')
    ..aOS(4, _omitFieldNames ? '' : 'domain')
    ..pPM<FlowStep>(5, _omitFieldNames ? '' : 'steps',
        subBuilder: FlowStep.create)
    ..aOM<FlowSlo>(6, _omitFieldNames ? '' : 'slo', subBuilder: FlowSlo.create)
    ..aOB(7, _omitFieldNames ? '' : 'enabled')
    ..aOM<$1.Timestamp>(8, _omitFieldNames ? '' : 'createdAt',
        subBuilder: $1.Timestamp.create)
    ..aOM<$1.Timestamp>(9, _omitFieldNames ? '' : 'updatedAt',
        subBuilder: $1.Timestamp.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  FlowDefinition clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  FlowDefinition copyWith(void Function(FlowDefinition) updates) =>
      super.copyWith((message) => updates(message as FlowDefinition))
          as FlowDefinition;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static FlowDefinition create() => FlowDefinition._();
  @$core.override
  FlowDefinition createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static FlowDefinition getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<FlowDefinition>(create);
  static FlowDefinition? _defaultInstance;

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
  $core.String get domain => $_getSZ(3);
  @$pb.TagNumber(4)
  set domain($core.String value) => $_setString(3, value);
  @$pb.TagNumber(4)
  $core.bool hasDomain() => $_has(3);
  @$pb.TagNumber(4)
  void clearDomain() => $_clearField(4);

  @$pb.TagNumber(5)
  $pb.PbList<FlowStep> get steps => $_getList(4);

  @$pb.TagNumber(6)
  FlowSlo get slo => $_getN(5);
  @$pb.TagNumber(6)
  set slo(FlowSlo value) => $_setField(6, value);
  @$pb.TagNumber(6)
  $core.bool hasSlo() => $_has(5);
  @$pb.TagNumber(6)
  void clearSlo() => $_clearField(6);
  @$pb.TagNumber(6)
  FlowSlo ensureSlo() => $_ensure(5);

  @$pb.TagNumber(7)
  $core.bool get enabled => $_getBF(6);
  @$pb.TagNumber(7)
  set enabled($core.bool value) => $_setBool(6, value);
  @$pb.TagNumber(7)
  $core.bool hasEnabled() => $_has(6);
  @$pb.TagNumber(7)
  void clearEnabled() => $_clearField(7);

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
  $1.Timestamp get updatedAt => $_getN(8);
  @$pb.TagNumber(9)
  set updatedAt($1.Timestamp value) => $_setField(9, value);
  @$pb.TagNumber(9)
  $core.bool hasUpdatedAt() => $_has(8);
  @$pb.TagNumber(9)
  void clearUpdatedAt() => $_clearField(9);
  @$pb.TagNumber(9)
  $1.Timestamp ensureUpdatedAt() => $_ensure(8);
}

class ListFlowsRequest extends $pb.GeneratedMessage {
  factory ListFlowsRequest({
    $1.Pagination? pagination,
    $core.String? domain,
  }) {
    final result = create();
    if (pagination != null) result.pagination = pagination;
    if (domain != null) result.domain = domain;
    return result;
  }

  ListFlowsRequest._();

  factory ListFlowsRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory ListFlowsRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ListFlowsRequest',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.eventmonitor.v1'),
      createEmptyInstance: create)
    ..aOM<$1.Pagination>(1, _omitFieldNames ? '' : 'pagination',
        subBuilder: $1.Pagination.create)
    ..aOS(2, _omitFieldNames ? '' : 'domain')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListFlowsRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListFlowsRequest copyWith(void Function(ListFlowsRequest) updates) =>
      super.copyWith((message) => updates(message as ListFlowsRequest))
          as ListFlowsRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static ListFlowsRequest create() => ListFlowsRequest._();
  @$core.override
  ListFlowsRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static ListFlowsRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ListFlowsRequest>(create);
  static ListFlowsRequest? _defaultInstance;

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

  @$pb.TagNumber(2)
  $core.String get domain => $_getSZ(1);
  @$pb.TagNumber(2)
  set domain($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasDomain() => $_has(1);
  @$pb.TagNumber(2)
  void clearDomain() => $_clearField(2);
}

class ListFlowsResponse extends $pb.GeneratedMessage {
  factory ListFlowsResponse({
    $core.Iterable<FlowDefinition>? flows,
    $1.PaginationResult? pagination,
  }) {
    final result = create();
    if (flows != null) result.flows.addAll(flows);
    if (pagination != null) result.pagination = pagination;
    return result;
  }

  ListFlowsResponse._();

  factory ListFlowsResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory ListFlowsResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ListFlowsResponse',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.eventmonitor.v1'),
      createEmptyInstance: create)
    ..pPM<FlowDefinition>(1, _omitFieldNames ? '' : 'flows',
        subBuilder: FlowDefinition.create)
    ..aOM<$1.PaginationResult>(2, _omitFieldNames ? '' : 'pagination',
        subBuilder: $1.PaginationResult.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListFlowsResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListFlowsResponse copyWith(void Function(ListFlowsResponse) updates) =>
      super.copyWith((message) => updates(message as ListFlowsResponse))
          as ListFlowsResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static ListFlowsResponse create() => ListFlowsResponse._();
  @$core.override
  ListFlowsResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static ListFlowsResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ListFlowsResponse>(create);
  static ListFlowsResponse? _defaultInstance;

  @$pb.TagNumber(1)
  $pb.PbList<FlowDefinition> get flows => $_getList(0);

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

class GetFlowRequest extends $pb.GeneratedMessage {
  factory GetFlowRequest({
    $core.String? id,
  }) {
    final result = create();
    if (id != null) result.id = id;
    return result;
  }

  GetFlowRequest._();

  factory GetFlowRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory GetFlowRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'GetFlowRequest',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.eventmonitor.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'id')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetFlowRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetFlowRequest copyWith(void Function(GetFlowRequest) updates) =>
      super.copyWith((message) => updates(message as GetFlowRequest))
          as GetFlowRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static GetFlowRequest create() => GetFlowRequest._();
  @$core.override
  GetFlowRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static GetFlowRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<GetFlowRequest>(create);
  static GetFlowRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get id => $_getSZ(0);
  @$pb.TagNumber(1)
  set id($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasId() => $_has(0);
  @$pb.TagNumber(1)
  void clearId() => $_clearField(1);
}

class GetFlowResponse extends $pb.GeneratedMessage {
  factory GetFlowResponse({
    FlowDefinition? flow,
  }) {
    final result = create();
    if (flow != null) result.flow = flow;
    return result;
  }

  GetFlowResponse._();

  factory GetFlowResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory GetFlowResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'GetFlowResponse',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.eventmonitor.v1'),
      createEmptyInstance: create)
    ..aOM<FlowDefinition>(1, _omitFieldNames ? '' : 'flow',
        subBuilder: FlowDefinition.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetFlowResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetFlowResponse copyWith(void Function(GetFlowResponse) updates) =>
      super.copyWith((message) => updates(message as GetFlowResponse))
          as GetFlowResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static GetFlowResponse create() => GetFlowResponse._();
  @$core.override
  GetFlowResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static GetFlowResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<GetFlowResponse>(create);
  static GetFlowResponse? _defaultInstance;

  @$pb.TagNumber(1)
  FlowDefinition get flow => $_getN(0);
  @$pb.TagNumber(1)
  set flow(FlowDefinition value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasFlow() => $_has(0);
  @$pb.TagNumber(1)
  void clearFlow() => $_clearField(1);
  @$pb.TagNumber(1)
  FlowDefinition ensureFlow() => $_ensure(0);
}

class CreateFlowRequest extends $pb.GeneratedMessage {
  factory CreateFlowRequest({
    $core.String? name,
    $core.String? description,
    $core.String? domain,
    $core.Iterable<FlowStep>? steps,
    FlowSlo? slo,
  }) {
    final result = create();
    if (name != null) result.name = name;
    if (description != null) result.description = description;
    if (domain != null) result.domain = domain;
    if (steps != null) result.steps.addAll(steps);
    if (slo != null) result.slo = slo;
    return result;
  }

  CreateFlowRequest._();

  factory CreateFlowRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory CreateFlowRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'CreateFlowRequest',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.eventmonitor.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'name')
    ..aOS(2, _omitFieldNames ? '' : 'description')
    ..aOS(3, _omitFieldNames ? '' : 'domain')
    ..pPM<FlowStep>(4, _omitFieldNames ? '' : 'steps',
        subBuilder: FlowStep.create)
    ..aOM<FlowSlo>(5, _omitFieldNames ? '' : 'slo', subBuilder: FlowSlo.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  CreateFlowRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  CreateFlowRequest copyWith(void Function(CreateFlowRequest) updates) =>
      super.copyWith((message) => updates(message as CreateFlowRequest))
          as CreateFlowRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static CreateFlowRequest create() => CreateFlowRequest._();
  @$core.override
  CreateFlowRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static CreateFlowRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<CreateFlowRequest>(create);
  static CreateFlowRequest? _defaultInstance;

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
  $core.String get domain => $_getSZ(2);
  @$pb.TagNumber(3)
  set domain($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasDomain() => $_has(2);
  @$pb.TagNumber(3)
  void clearDomain() => $_clearField(3);

  @$pb.TagNumber(4)
  $pb.PbList<FlowStep> get steps => $_getList(3);

  @$pb.TagNumber(5)
  FlowSlo get slo => $_getN(4);
  @$pb.TagNumber(5)
  set slo(FlowSlo value) => $_setField(5, value);
  @$pb.TagNumber(5)
  $core.bool hasSlo() => $_has(4);
  @$pb.TagNumber(5)
  void clearSlo() => $_clearField(5);
  @$pb.TagNumber(5)
  FlowSlo ensureSlo() => $_ensure(4);
}

class CreateFlowResponse extends $pb.GeneratedMessage {
  factory CreateFlowResponse({
    FlowDefinition? flow,
  }) {
    final result = create();
    if (flow != null) result.flow = flow;
    return result;
  }

  CreateFlowResponse._();

  factory CreateFlowResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory CreateFlowResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'CreateFlowResponse',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.eventmonitor.v1'),
      createEmptyInstance: create)
    ..aOM<FlowDefinition>(1, _omitFieldNames ? '' : 'flow',
        subBuilder: FlowDefinition.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  CreateFlowResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  CreateFlowResponse copyWith(void Function(CreateFlowResponse) updates) =>
      super.copyWith((message) => updates(message as CreateFlowResponse))
          as CreateFlowResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static CreateFlowResponse create() => CreateFlowResponse._();
  @$core.override
  CreateFlowResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static CreateFlowResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<CreateFlowResponse>(create);
  static CreateFlowResponse? _defaultInstance;

  @$pb.TagNumber(1)
  FlowDefinition get flow => $_getN(0);
  @$pb.TagNumber(1)
  set flow(FlowDefinition value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasFlow() => $_has(0);
  @$pb.TagNumber(1)
  void clearFlow() => $_clearField(1);
  @$pb.TagNumber(1)
  FlowDefinition ensureFlow() => $_ensure(0);
}

class UpdateFlowRequest extends $pb.GeneratedMessage {
  factory UpdateFlowRequest({
    $core.String? id,
    $core.String? description,
    $core.Iterable<FlowStep>? steps,
    FlowSlo? slo,
    $core.bool? enabled,
  }) {
    final result = create();
    if (id != null) result.id = id;
    if (description != null) result.description = description;
    if (steps != null) result.steps.addAll(steps);
    if (slo != null) result.slo = slo;
    if (enabled != null) result.enabled = enabled;
    return result;
  }

  UpdateFlowRequest._();

  factory UpdateFlowRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory UpdateFlowRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'UpdateFlowRequest',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.eventmonitor.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'id')
    ..aOS(2, _omitFieldNames ? '' : 'description')
    ..pPM<FlowStep>(3, _omitFieldNames ? '' : 'steps',
        subBuilder: FlowStep.create)
    ..aOM<FlowSlo>(4, _omitFieldNames ? '' : 'slo', subBuilder: FlowSlo.create)
    ..aOB(5, _omitFieldNames ? '' : 'enabled')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  UpdateFlowRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  UpdateFlowRequest copyWith(void Function(UpdateFlowRequest) updates) =>
      super.copyWith((message) => updates(message as UpdateFlowRequest))
          as UpdateFlowRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static UpdateFlowRequest create() => UpdateFlowRequest._();
  @$core.override
  UpdateFlowRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static UpdateFlowRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<UpdateFlowRequest>(create);
  static UpdateFlowRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get id => $_getSZ(0);
  @$pb.TagNumber(1)
  set id($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasId() => $_has(0);
  @$pb.TagNumber(1)
  void clearId() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get description => $_getSZ(1);
  @$pb.TagNumber(2)
  set description($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasDescription() => $_has(1);
  @$pb.TagNumber(2)
  void clearDescription() => $_clearField(2);

  @$pb.TagNumber(3)
  $pb.PbList<FlowStep> get steps => $_getList(2);

  @$pb.TagNumber(4)
  FlowSlo get slo => $_getN(3);
  @$pb.TagNumber(4)
  set slo(FlowSlo value) => $_setField(4, value);
  @$pb.TagNumber(4)
  $core.bool hasSlo() => $_has(3);
  @$pb.TagNumber(4)
  void clearSlo() => $_clearField(4);
  @$pb.TagNumber(4)
  FlowSlo ensureSlo() => $_ensure(3);

  @$pb.TagNumber(5)
  $core.bool get enabled => $_getBF(4);
  @$pb.TagNumber(5)
  set enabled($core.bool value) => $_setBool(4, value);
  @$pb.TagNumber(5)
  $core.bool hasEnabled() => $_has(4);
  @$pb.TagNumber(5)
  void clearEnabled() => $_clearField(5);
}

class UpdateFlowResponse extends $pb.GeneratedMessage {
  factory UpdateFlowResponse({
    FlowDefinition? flow,
  }) {
    final result = create();
    if (flow != null) result.flow = flow;
    return result;
  }

  UpdateFlowResponse._();

  factory UpdateFlowResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory UpdateFlowResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'UpdateFlowResponse',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.eventmonitor.v1'),
      createEmptyInstance: create)
    ..aOM<FlowDefinition>(1, _omitFieldNames ? '' : 'flow',
        subBuilder: FlowDefinition.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  UpdateFlowResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  UpdateFlowResponse copyWith(void Function(UpdateFlowResponse) updates) =>
      super.copyWith((message) => updates(message as UpdateFlowResponse))
          as UpdateFlowResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static UpdateFlowResponse create() => UpdateFlowResponse._();
  @$core.override
  UpdateFlowResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static UpdateFlowResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<UpdateFlowResponse>(create);
  static UpdateFlowResponse? _defaultInstance;

  @$pb.TagNumber(1)
  FlowDefinition get flow => $_getN(0);
  @$pb.TagNumber(1)
  set flow(FlowDefinition value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasFlow() => $_has(0);
  @$pb.TagNumber(1)
  void clearFlow() => $_clearField(1);
  @$pb.TagNumber(1)
  FlowDefinition ensureFlow() => $_ensure(0);
}

class DeleteFlowRequest extends $pb.GeneratedMessage {
  factory DeleteFlowRequest({
    $core.String? id,
  }) {
    final result = create();
    if (id != null) result.id = id;
    return result;
  }

  DeleteFlowRequest._();

  factory DeleteFlowRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory DeleteFlowRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'DeleteFlowRequest',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.eventmonitor.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'id')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  DeleteFlowRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  DeleteFlowRequest copyWith(void Function(DeleteFlowRequest) updates) =>
      super.copyWith((message) => updates(message as DeleteFlowRequest))
          as DeleteFlowRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static DeleteFlowRequest create() => DeleteFlowRequest._();
  @$core.override
  DeleteFlowRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static DeleteFlowRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<DeleteFlowRequest>(create);
  static DeleteFlowRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get id => $_getSZ(0);
  @$pb.TagNumber(1)
  set id($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasId() => $_has(0);
  @$pb.TagNumber(1)
  void clearId() => $_clearField(1);
}

class DeleteFlowResponse extends $pb.GeneratedMessage {
  factory DeleteFlowResponse({
    $core.bool? success,
    $core.String? message,
  }) {
    final result = create();
    if (success != null) result.success = success;
    if (message != null) result.message = message;
    return result;
  }

  DeleteFlowResponse._();

  factory DeleteFlowResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory DeleteFlowResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'DeleteFlowResponse',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.eventmonitor.v1'),
      createEmptyInstance: create)
    ..aOB(1, _omitFieldNames ? '' : 'success')
    ..aOS(2, _omitFieldNames ? '' : 'message')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  DeleteFlowResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  DeleteFlowResponse copyWith(void Function(DeleteFlowResponse) updates) =>
      super.copyWith((message) => updates(message as DeleteFlowResponse))
          as DeleteFlowResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static DeleteFlowResponse create() => DeleteFlowResponse._();
  @$core.override
  DeleteFlowResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static DeleteFlowResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<DeleteFlowResponse>(create);
  static DeleteFlowResponse? _defaultInstance;

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

class BottleneckStep extends $pb.GeneratedMessage {
  factory BottleneckStep({
    $core.String? eventType,
    $core.int? stepIndex,
    $core.double? avgDurationSeconds,
    $core.double? timeoutRate,
  }) {
    final result = create();
    if (eventType != null) result.eventType = eventType;
    if (stepIndex != null) result.stepIndex = stepIndex;
    if (avgDurationSeconds != null)
      result.avgDurationSeconds = avgDurationSeconds;
    if (timeoutRate != null) result.timeoutRate = timeoutRate;
    return result;
  }

  BottleneckStep._();

  factory BottleneckStep.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory BottleneckStep.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'BottleneckStep',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.eventmonitor.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'eventType')
    ..aI(2, _omitFieldNames ? '' : 'stepIndex')
    ..aD(3, _omitFieldNames ? '' : 'avgDurationSeconds')
    ..aD(4, _omitFieldNames ? '' : 'timeoutRate')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  BottleneckStep clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  BottleneckStep copyWith(void Function(BottleneckStep) updates) =>
      super.copyWith((message) => updates(message as BottleneckStep))
          as BottleneckStep;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static BottleneckStep create() => BottleneckStep._();
  @$core.override
  BottleneckStep createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static BottleneckStep getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<BottleneckStep>(create);
  static BottleneckStep? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get eventType => $_getSZ(0);
  @$pb.TagNumber(1)
  set eventType($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasEventType() => $_has(0);
  @$pb.TagNumber(1)
  void clearEventType() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.int get stepIndex => $_getIZ(1);
  @$pb.TagNumber(2)
  set stepIndex($core.int value) => $_setSignedInt32(1, value);
  @$pb.TagNumber(2)
  $core.bool hasStepIndex() => $_has(1);
  @$pb.TagNumber(2)
  void clearStepIndex() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.double get avgDurationSeconds => $_getN(2);
  @$pb.TagNumber(3)
  set avgDurationSeconds($core.double value) => $_setDouble(2, value);
  @$pb.TagNumber(3)
  $core.bool hasAvgDurationSeconds() => $_has(2);
  @$pb.TagNumber(3)
  void clearAvgDurationSeconds() => $_clearField(3);

  @$pb.TagNumber(4)
  $core.double get timeoutRate => $_getN(3);
  @$pb.TagNumber(4)
  set timeoutRate($core.double value) => $_setDouble(3, value);
  @$pb.TagNumber(4)
  $core.bool hasTimeoutRate() => $_has(3);
  @$pb.TagNumber(4)
  void clearTimeoutRate() => $_clearField(4);
}

class SloStatus extends $pb.GeneratedMessage {
  factory SloStatus({
    $core.int? targetCompletionSeconds,
    $core.double? targetSuccessRate,
    $core.double? currentSuccessRate,
    $core.bool? isViolated,
    $core.double? burnRate,
    $core.double? estimatedBudgetExhaustionHours,
  }) {
    final result = create();
    if (targetCompletionSeconds != null)
      result.targetCompletionSeconds = targetCompletionSeconds;
    if (targetSuccessRate != null) result.targetSuccessRate = targetSuccessRate;
    if (currentSuccessRate != null)
      result.currentSuccessRate = currentSuccessRate;
    if (isViolated != null) result.isViolated = isViolated;
    if (burnRate != null) result.burnRate = burnRate;
    if (estimatedBudgetExhaustionHours != null)
      result.estimatedBudgetExhaustionHours = estimatedBudgetExhaustionHours;
    return result;
  }

  SloStatus._();

  factory SloStatus.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory SloStatus.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'SloStatus',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.eventmonitor.v1'),
      createEmptyInstance: create)
    ..aI(1, _omitFieldNames ? '' : 'targetCompletionSeconds')
    ..aD(2, _omitFieldNames ? '' : 'targetSuccessRate')
    ..aD(3, _omitFieldNames ? '' : 'currentSuccessRate')
    ..aOB(4, _omitFieldNames ? '' : 'isViolated')
    ..aD(5, _omitFieldNames ? '' : 'burnRate')
    ..aD(6, _omitFieldNames ? '' : 'estimatedBudgetExhaustionHours')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  SloStatus clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  SloStatus copyWith(void Function(SloStatus) updates) =>
      super.copyWith((message) => updates(message as SloStatus)) as SloStatus;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static SloStatus create() => SloStatus._();
  @$core.override
  SloStatus createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static SloStatus getDefault() =>
      _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<SloStatus>(create);
  static SloStatus? _defaultInstance;

  @$pb.TagNumber(1)
  $core.int get targetCompletionSeconds => $_getIZ(0);
  @$pb.TagNumber(1)
  set targetCompletionSeconds($core.int value) => $_setSignedInt32(0, value);
  @$pb.TagNumber(1)
  $core.bool hasTargetCompletionSeconds() => $_has(0);
  @$pb.TagNumber(1)
  void clearTargetCompletionSeconds() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.double get targetSuccessRate => $_getN(1);
  @$pb.TagNumber(2)
  set targetSuccessRate($core.double value) => $_setDouble(1, value);
  @$pb.TagNumber(2)
  $core.bool hasTargetSuccessRate() => $_has(1);
  @$pb.TagNumber(2)
  void clearTargetSuccessRate() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.double get currentSuccessRate => $_getN(2);
  @$pb.TagNumber(3)
  set currentSuccessRate($core.double value) => $_setDouble(2, value);
  @$pb.TagNumber(3)
  $core.bool hasCurrentSuccessRate() => $_has(2);
  @$pb.TagNumber(3)
  void clearCurrentSuccessRate() => $_clearField(3);

  @$pb.TagNumber(4)
  $core.bool get isViolated => $_getBF(3);
  @$pb.TagNumber(4)
  set isViolated($core.bool value) => $_setBool(3, value);
  @$pb.TagNumber(4)
  $core.bool hasIsViolated() => $_has(3);
  @$pb.TagNumber(4)
  void clearIsViolated() => $_clearField(4);

  @$pb.TagNumber(5)
  $core.double get burnRate => $_getN(4);
  @$pb.TagNumber(5)
  set burnRate($core.double value) => $_setDouble(4, value);
  @$pb.TagNumber(5)
  $core.bool hasBurnRate() => $_has(4);
  @$pb.TagNumber(5)
  void clearBurnRate() => $_clearField(5);

  @$pb.TagNumber(6)
  $core.double get estimatedBudgetExhaustionHours => $_getN(5);
  @$pb.TagNumber(6)
  set estimatedBudgetExhaustionHours($core.double value) =>
      $_setDouble(5, value);
  @$pb.TagNumber(6)
  $core.bool hasEstimatedBudgetExhaustionHours() => $_has(5);
  @$pb.TagNumber(6)
  void clearEstimatedBudgetExhaustionHours() => $_clearField(6);
}

class FlowKpi extends $pb.GeneratedMessage {
  factory FlowKpi({
    $fixnum.Int64? totalStarted,
    $fixnum.Int64? totalCompleted,
    $fixnum.Int64? totalFailed,
    $fixnum.Int64? totalInProgress,
    $core.double? completionRate,
    $core.double? avgDurationSeconds,
    $core.double? p50DurationSeconds,
    $core.double? p95DurationSeconds,
    $core.double? p99DurationSeconds,
    BottleneckStep? bottleneckStep,
  }) {
    final result = create();
    if (totalStarted != null) result.totalStarted = totalStarted;
    if (totalCompleted != null) result.totalCompleted = totalCompleted;
    if (totalFailed != null) result.totalFailed = totalFailed;
    if (totalInProgress != null) result.totalInProgress = totalInProgress;
    if (completionRate != null) result.completionRate = completionRate;
    if (avgDurationSeconds != null)
      result.avgDurationSeconds = avgDurationSeconds;
    if (p50DurationSeconds != null)
      result.p50DurationSeconds = p50DurationSeconds;
    if (p95DurationSeconds != null)
      result.p95DurationSeconds = p95DurationSeconds;
    if (p99DurationSeconds != null)
      result.p99DurationSeconds = p99DurationSeconds;
    if (bottleneckStep != null) result.bottleneckStep = bottleneckStep;
    return result;
  }

  FlowKpi._();

  factory FlowKpi.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory FlowKpi.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'FlowKpi',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.eventmonitor.v1'),
      createEmptyInstance: create)
    ..aInt64(1, _omitFieldNames ? '' : 'totalStarted')
    ..aInt64(2, _omitFieldNames ? '' : 'totalCompleted')
    ..aInt64(3, _omitFieldNames ? '' : 'totalFailed')
    ..aInt64(4, _omitFieldNames ? '' : 'totalInProgress')
    ..aD(5, _omitFieldNames ? '' : 'completionRate')
    ..aD(6, _omitFieldNames ? '' : 'avgDurationSeconds')
    ..aD(7, _omitFieldNames ? '' : 'p50DurationSeconds')
    ..aD(8, _omitFieldNames ? '' : 'p95DurationSeconds')
    ..aD(9, _omitFieldNames ? '' : 'p99DurationSeconds')
    ..aOM<BottleneckStep>(10, _omitFieldNames ? '' : 'bottleneckStep',
        subBuilder: BottleneckStep.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  FlowKpi clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  FlowKpi copyWith(void Function(FlowKpi) updates) =>
      super.copyWith((message) => updates(message as FlowKpi)) as FlowKpi;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static FlowKpi create() => FlowKpi._();
  @$core.override
  FlowKpi createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static FlowKpi getDefault() =>
      _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<FlowKpi>(create);
  static FlowKpi? _defaultInstance;

  @$pb.TagNumber(1)
  $fixnum.Int64 get totalStarted => $_getI64(0);
  @$pb.TagNumber(1)
  set totalStarted($fixnum.Int64 value) => $_setInt64(0, value);
  @$pb.TagNumber(1)
  $core.bool hasTotalStarted() => $_has(0);
  @$pb.TagNumber(1)
  void clearTotalStarted() => $_clearField(1);

  @$pb.TagNumber(2)
  $fixnum.Int64 get totalCompleted => $_getI64(1);
  @$pb.TagNumber(2)
  set totalCompleted($fixnum.Int64 value) => $_setInt64(1, value);
  @$pb.TagNumber(2)
  $core.bool hasTotalCompleted() => $_has(1);
  @$pb.TagNumber(2)
  void clearTotalCompleted() => $_clearField(2);

  @$pb.TagNumber(3)
  $fixnum.Int64 get totalFailed => $_getI64(2);
  @$pb.TagNumber(3)
  set totalFailed($fixnum.Int64 value) => $_setInt64(2, value);
  @$pb.TagNumber(3)
  $core.bool hasTotalFailed() => $_has(2);
  @$pb.TagNumber(3)
  void clearTotalFailed() => $_clearField(3);

  @$pb.TagNumber(4)
  $fixnum.Int64 get totalInProgress => $_getI64(3);
  @$pb.TagNumber(4)
  set totalInProgress($fixnum.Int64 value) => $_setInt64(3, value);
  @$pb.TagNumber(4)
  $core.bool hasTotalInProgress() => $_has(3);
  @$pb.TagNumber(4)
  void clearTotalInProgress() => $_clearField(4);

  @$pb.TagNumber(5)
  $core.double get completionRate => $_getN(4);
  @$pb.TagNumber(5)
  set completionRate($core.double value) => $_setDouble(4, value);
  @$pb.TagNumber(5)
  $core.bool hasCompletionRate() => $_has(4);
  @$pb.TagNumber(5)
  void clearCompletionRate() => $_clearField(5);

  @$pb.TagNumber(6)
  $core.double get avgDurationSeconds => $_getN(5);
  @$pb.TagNumber(6)
  set avgDurationSeconds($core.double value) => $_setDouble(5, value);
  @$pb.TagNumber(6)
  $core.bool hasAvgDurationSeconds() => $_has(5);
  @$pb.TagNumber(6)
  void clearAvgDurationSeconds() => $_clearField(6);

  @$pb.TagNumber(7)
  $core.double get p50DurationSeconds => $_getN(6);
  @$pb.TagNumber(7)
  set p50DurationSeconds($core.double value) => $_setDouble(6, value);
  @$pb.TagNumber(7)
  $core.bool hasP50DurationSeconds() => $_has(6);
  @$pb.TagNumber(7)
  void clearP50DurationSeconds() => $_clearField(7);

  @$pb.TagNumber(8)
  $core.double get p95DurationSeconds => $_getN(7);
  @$pb.TagNumber(8)
  set p95DurationSeconds($core.double value) => $_setDouble(7, value);
  @$pb.TagNumber(8)
  $core.bool hasP95DurationSeconds() => $_has(7);
  @$pb.TagNumber(8)
  void clearP95DurationSeconds() => $_clearField(8);

  @$pb.TagNumber(9)
  $core.double get p99DurationSeconds => $_getN(8);
  @$pb.TagNumber(9)
  set p99DurationSeconds($core.double value) => $_setDouble(8, value);
  @$pb.TagNumber(9)
  $core.bool hasP99DurationSeconds() => $_has(8);
  @$pb.TagNumber(9)
  void clearP99DurationSeconds() => $_clearField(9);

  @$pb.TagNumber(10)
  BottleneckStep get bottleneckStep => $_getN(9);
  @$pb.TagNumber(10)
  set bottleneckStep(BottleneckStep value) => $_setField(10, value);
  @$pb.TagNumber(10)
  $core.bool hasBottleneckStep() => $_has(9);
  @$pb.TagNumber(10)
  void clearBottleneckStep() => $_clearField(10);
  @$pb.TagNumber(10)
  BottleneckStep ensureBottleneckStep() => $_ensure(9);
}

class GetFlowKpiRequest extends $pb.GeneratedMessage {
  factory GetFlowKpiRequest({
    $core.String? flowId,
    $core.String? period,
  }) {
    final result = create();
    if (flowId != null) result.flowId = flowId;
    if (period != null) result.period = period;
    return result;
  }

  GetFlowKpiRequest._();

  factory GetFlowKpiRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory GetFlowKpiRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'GetFlowKpiRequest',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.eventmonitor.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'flowId')
    ..aOS(2, _omitFieldNames ? '' : 'period')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetFlowKpiRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetFlowKpiRequest copyWith(void Function(GetFlowKpiRequest) updates) =>
      super.copyWith((message) => updates(message as GetFlowKpiRequest))
          as GetFlowKpiRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static GetFlowKpiRequest create() => GetFlowKpiRequest._();
  @$core.override
  GetFlowKpiRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static GetFlowKpiRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<GetFlowKpiRequest>(create);
  static GetFlowKpiRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get flowId => $_getSZ(0);
  @$pb.TagNumber(1)
  set flowId($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasFlowId() => $_has(0);
  @$pb.TagNumber(1)
  void clearFlowId() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get period => $_getSZ(1);
  @$pb.TagNumber(2)
  set period($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasPeriod() => $_has(1);
  @$pb.TagNumber(2)
  void clearPeriod() => $_clearField(2);
}

class GetFlowKpiResponse extends $pb.GeneratedMessage {
  factory GetFlowKpiResponse({
    $core.String? flowId,
    $core.String? flowName,
    $core.String? period,
    FlowKpi? kpi,
    SloStatus? sloStatus,
  }) {
    final result = create();
    if (flowId != null) result.flowId = flowId;
    if (flowName != null) result.flowName = flowName;
    if (period != null) result.period = period;
    if (kpi != null) result.kpi = kpi;
    if (sloStatus != null) result.sloStatus = sloStatus;
    return result;
  }

  GetFlowKpiResponse._();

  factory GetFlowKpiResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory GetFlowKpiResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'GetFlowKpiResponse',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.eventmonitor.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'flowId')
    ..aOS(2, _omitFieldNames ? '' : 'flowName')
    ..aOS(3, _omitFieldNames ? '' : 'period')
    ..aOM<FlowKpi>(4, _omitFieldNames ? '' : 'kpi', subBuilder: FlowKpi.create)
    ..aOM<SloStatus>(5, _omitFieldNames ? '' : 'sloStatus',
        subBuilder: SloStatus.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetFlowKpiResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetFlowKpiResponse copyWith(void Function(GetFlowKpiResponse) updates) =>
      super.copyWith((message) => updates(message as GetFlowKpiResponse))
          as GetFlowKpiResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static GetFlowKpiResponse create() => GetFlowKpiResponse._();
  @$core.override
  GetFlowKpiResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static GetFlowKpiResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<GetFlowKpiResponse>(create);
  static GetFlowKpiResponse? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get flowId => $_getSZ(0);
  @$pb.TagNumber(1)
  set flowId($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasFlowId() => $_has(0);
  @$pb.TagNumber(1)
  void clearFlowId() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get flowName => $_getSZ(1);
  @$pb.TagNumber(2)
  set flowName($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasFlowName() => $_has(1);
  @$pb.TagNumber(2)
  void clearFlowName() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.String get period => $_getSZ(2);
  @$pb.TagNumber(3)
  set period($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasPeriod() => $_has(2);
  @$pb.TagNumber(3)
  void clearPeriod() => $_clearField(3);

  @$pb.TagNumber(4)
  FlowKpi get kpi => $_getN(3);
  @$pb.TagNumber(4)
  set kpi(FlowKpi value) => $_setField(4, value);
  @$pb.TagNumber(4)
  $core.bool hasKpi() => $_has(3);
  @$pb.TagNumber(4)
  void clearKpi() => $_clearField(4);
  @$pb.TagNumber(4)
  FlowKpi ensureKpi() => $_ensure(3);

  @$pb.TagNumber(5)
  SloStatus get sloStatus => $_getN(4);
  @$pb.TagNumber(5)
  set sloStatus(SloStatus value) => $_setField(5, value);
  @$pb.TagNumber(5)
  $core.bool hasSloStatus() => $_has(4);
  @$pb.TagNumber(5)
  void clearSloStatus() => $_clearField(5);
  @$pb.TagNumber(5)
  SloStatus ensureSloStatus() => $_ensure(4);
}

class FlowKpiSummary extends $pb.GeneratedMessage {
  factory FlowKpiSummary({
    $core.String? flowId,
    $core.String? flowName,
    $core.String? domain,
    $fixnum.Int64? totalStarted,
    $core.double? completionRate,
    $core.double? avgDurationSeconds,
    $core.bool? sloViolated,
  }) {
    final result = create();
    if (flowId != null) result.flowId = flowId;
    if (flowName != null) result.flowName = flowName;
    if (domain != null) result.domain = domain;
    if (totalStarted != null) result.totalStarted = totalStarted;
    if (completionRate != null) result.completionRate = completionRate;
    if (avgDurationSeconds != null)
      result.avgDurationSeconds = avgDurationSeconds;
    if (sloViolated != null) result.sloViolated = sloViolated;
    return result;
  }

  FlowKpiSummary._();

  factory FlowKpiSummary.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory FlowKpiSummary.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'FlowKpiSummary',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.eventmonitor.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'flowId')
    ..aOS(2, _omitFieldNames ? '' : 'flowName')
    ..aOS(3, _omitFieldNames ? '' : 'domain')
    ..aInt64(4, _omitFieldNames ? '' : 'totalStarted')
    ..aD(5, _omitFieldNames ? '' : 'completionRate')
    ..aD(6, _omitFieldNames ? '' : 'avgDurationSeconds')
    ..aOB(7, _omitFieldNames ? '' : 'sloViolated')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  FlowKpiSummary clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  FlowKpiSummary copyWith(void Function(FlowKpiSummary) updates) =>
      super.copyWith((message) => updates(message as FlowKpiSummary))
          as FlowKpiSummary;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static FlowKpiSummary create() => FlowKpiSummary._();
  @$core.override
  FlowKpiSummary createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static FlowKpiSummary getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<FlowKpiSummary>(create);
  static FlowKpiSummary? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get flowId => $_getSZ(0);
  @$pb.TagNumber(1)
  set flowId($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasFlowId() => $_has(0);
  @$pb.TagNumber(1)
  void clearFlowId() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get flowName => $_getSZ(1);
  @$pb.TagNumber(2)
  set flowName($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasFlowName() => $_has(1);
  @$pb.TagNumber(2)
  void clearFlowName() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.String get domain => $_getSZ(2);
  @$pb.TagNumber(3)
  set domain($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasDomain() => $_has(2);
  @$pb.TagNumber(3)
  void clearDomain() => $_clearField(3);

  @$pb.TagNumber(4)
  $fixnum.Int64 get totalStarted => $_getI64(3);
  @$pb.TagNumber(4)
  set totalStarted($fixnum.Int64 value) => $_setInt64(3, value);
  @$pb.TagNumber(4)
  $core.bool hasTotalStarted() => $_has(3);
  @$pb.TagNumber(4)
  void clearTotalStarted() => $_clearField(4);

  @$pb.TagNumber(5)
  $core.double get completionRate => $_getN(4);
  @$pb.TagNumber(5)
  set completionRate($core.double value) => $_setDouble(4, value);
  @$pb.TagNumber(5)
  $core.bool hasCompletionRate() => $_has(4);
  @$pb.TagNumber(5)
  void clearCompletionRate() => $_clearField(5);

  @$pb.TagNumber(6)
  $core.double get avgDurationSeconds => $_getN(5);
  @$pb.TagNumber(6)
  set avgDurationSeconds($core.double value) => $_setDouble(5, value);
  @$pb.TagNumber(6)
  $core.bool hasAvgDurationSeconds() => $_has(5);
  @$pb.TagNumber(6)
  void clearAvgDurationSeconds() => $_clearField(6);

  @$pb.TagNumber(7)
  $core.bool get sloViolated => $_getBF(6);
  @$pb.TagNumber(7)
  set sloViolated($core.bool value) => $_setBool(6, value);
  @$pb.TagNumber(7)
  $core.bool hasSloViolated() => $_has(6);
  @$pb.TagNumber(7)
  void clearSloViolated() => $_clearField(7);
}

class GetKpiSummaryRequest extends $pb.GeneratedMessage {
  factory GetKpiSummaryRequest({
    $core.String? period,
  }) {
    final result = create();
    if (period != null) result.period = period;
    return result;
  }

  GetKpiSummaryRequest._();

  factory GetKpiSummaryRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory GetKpiSummaryRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'GetKpiSummaryRequest',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.eventmonitor.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'period')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetKpiSummaryRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetKpiSummaryRequest copyWith(void Function(GetKpiSummaryRequest) updates) =>
      super.copyWith((message) => updates(message as GetKpiSummaryRequest))
          as GetKpiSummaryRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static GetKpiSummaryRequest create() => GetKpiSummaryRequest._();
  @$core.override
  GetKpiSummaryRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static GetKpiSummaryRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<GetKpiSummaryRequest>(create);
  static GetKpiSummaryRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get period => $_getSZ(0);
  @$pb.TagNumber(1)
  set period($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasPeriod() => $_has(0);
  @$pb.TagNumber(1)
  void clearPeriod() => $_clearField(1);
}

class GetKpiSummaryResponse extends $pb.GeneratedMessage {
  factory GetKpiSummaryResponse({
    $core.String? period,
    $core.Iterable<FlowKpiSummary>? flows,
    $core.int? totalFlows,
    $core.int? flowsWithSloViolation,
    $core.double? overallCompletionRate,
  }) {
    final result = create();
    if (period != null) result.period = period;
    if (flows != null) result.flows.addAll(flows);
    if (totalFlows != null) result.totalFlows = totalFlows;
    if (flowsWithSloViolation != null)
      result.flowsWithSloViolation = flowsWithSloViolation;
    if (overallCompletionRate != null)
      result.overallCompletionRate = overallCompletionRate;
    return result;
  }

  GetKpiSummaryResponse._();

  factory GetKpiSummaryResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory GetKpiSummaryResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'GetKpiSummaryResponse',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.eventmonitor.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'period')
    ..pPM<FlowKpiSummary>(2, _omitFieldNames ? '' : 'flows',
        subBuilder: FlowKpiSummary.create)
    ..aI(3, _omitFieldNames ? '' : 'totalFlows')
    ..aI(4, _omitFieldNames ? '' : 'flowsWithSloViolation')
    ..aD(5, _omitFieldNames ? '' : 'overallCompletionRate')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetKpiSummaryResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetKpiSummaryResponse copyWith(
          void Function(GetKpiSummaryResponse) updates) =>
      super.copyWith((message) => updates(message as GetKpiSummaryResponse))
          as GetKpiSummaryResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static GetKpiSummaryResponse create() => GetKpiSummaryResponse._();
  @$core.override
  GetKpiSummaryResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static GetKpiSummaryResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<GetKpiSummaryResponse>(create);
  static GetKpiSummaryResponse? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get period => $_getSZ(0);
  @$pb.TagNumber(1)
  set period($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasPeriod() => $_has(0);
  @$pb.TagNumber(1)
  void clearPeriod() => $_clearField(1);

  @$pb.TagNumber(2)
  $pb.PbList<FlowKpiSummary> get flows => $_getList(1);

  @$pb.TagNumber(3)
  $core.int get totalFlows => $_getIZ(2);
  @$pb.TagNumber(3)
  set totalFlows($core.int value) => $_setSignedInt32(2, value);
  @$pb.TagNumber(3)
  $core.bool hasTotalFlows() => $_has(2);
  @$pb.TagNumber(3)
  void clearTotalFlows() => $_clearField(3);

  @$pb.TagNumber(4)
  $core.int get flowsWithSloViolation => $_getIZ(3);
  @$pb.TagNumber(4)
  set flowsWithSloViolation($core.int value) => $_setSignedInt32(3, value);
  @$pb.TagNumber(4)
  $core.bool hasFlowsWithSloViolation() => $_has(3);
  @$pb.TagNumber(4)
  void clearFlowsWithSloViolation() => $_clearField(4);

  @$pb.TagNumber(5)
  $core.double get overallCompletionRate => $_getN(4);
  @$pb.TagNumber(5)
  set overallCompletionRate($core.double value) => $_setDouble(4, value);
  @$pb.TagNumber(5)
  $core.bool hasOverallCompletionRate() => $_has(4);
  @$pb.TagNumber(5)
  void clearOverallCompletionRate() => $_clearField(5);
}

class GetSloStatusRequest extends $pb.GeneratedMessage {
  factory GetSloStatusRequest() => create();

  GetSloStatusRequest._();

  factory GetSloStatusRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory GetSloStatusRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'GetSloStatusRequest',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.eventmonitor.v1'),
      createEmptyInstance: create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetSloStatusRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetSloStatusRequest copyWith(void Function(GetSloStatusRequest) updates) =>
      super.copyWith((message) => updates(message as GetSloStatusRequest))
          as GetSloStatusRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static GetSloStatusRequest create() => GetSloStatusRequest._();
  @$core.override
  GetSloStatusRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static GetSloStatusRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<GetSloStatusRequest>(create);
  static GetSloStatusRequest? _defaultInstance;
}

class SloFlowStatus extends $pb.GeneratedMessage {
  factory SloFlowStatus({
    $core.String? flowId,
    $core.String? flowName,
    $core.bool? isViolated,
    $core.double? burnRate,
    $core.double? errorBudgetRemaining,
  }) {
    final result = create();
    if (flowId != null) result.flowId = flowId;
    if (flowName != null) result.flowName = flowName;
    if (isViolated != null) result.isViolated = isViolated;
    if (burnRate != null) result.burnRate = burnRate;
    if (errorBudgetRemaining != null)
      result.errorBudgetRemaining = errorBudgetRemaining;
    return result;
  }

  SloFlowStatus._();

  factory SloFlowStatus.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory SloFlowStatus.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'SloFlowStatus',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.eventmonitor.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'flowId')
    ..aOS(2, _omitFieldNames ? '' : 'flowName')
    ..aOB(3, _omitFieldNames ? '' : 'isViolated')
    ..aD(4, _omitFieldNames ? '' : 'burnRate')
    ..aD(5, _omitFieldNames ? '' : 'errorBudgetRemaining')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  SloFlowStatus clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  SloFlowStatus copyWith(void Function(SloFlowStatus) updates) =>
      super.copyWith((message) => updates(message as SloFlowStatus))
          as SloFlowStatus;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static SloFlowStatus create() => SloFlowStatus._();
  @$core.override
  SloFlowStatus createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static SloFlowStatus getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<SloFlowStatus>(create);
  static SloFlowStatus? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get flowId => $_getSZ(0);
  @$pb.TagNumber(1)
  set flowId($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasFlowId() => $_has(0);
  @$pb.TagNumber(1)
  void clearFlowId() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get flowName => $_getSZ(1);
  @$pb.TagNumber(2)
  set flowName($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasFlowName() => $_has(1);
  @$pb.TagNumber(2)
  void clearFlowName() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.bool get isViolated => $_getBF(2);
  @$pb.TagNumber(3)
  set isViolated($core.bool value) => $_setBool(2, value);
  @$pb.TagNumber(3)
  $core.bool hasIsViolated() => $_has(2);
  @$pb.TagNumber(3)
  void clearIsViolated() => $_clearField(3);

  @$pb.TagNumber(4)
  $core.double get burnRate => $_getN(3);
  @$pb.TagNumber(4)
  set burnRate($core.double value) => $_setDouble(3, value);
  @$pb.TagNumber(4)
  $core.bool hasBurnRate() => $_has(3);
  @$pb.TagNumber(4)
  void clearBurnRate() => $_clearField(4);

  @$pb.TagNumber(5)
  $core.double get errorBudgetRemaining => $_getN(4);
  @$pb.TagNumber(5)
  set errorBudgetRemaining($core.double value) => $_setDouble(4, value);
  @$pb.TagNumber(5)
  $core.bool hasErrorBudgetRemaining() => $_has(4);
  @$pb.TagNumber(5)
  void clearErrorBudgetRemaining() => $_clearField(5);
}

class GetSloStatusResponse extends $pb.GeneratedMessage {
  factory GetSloStatusResponse({
    $core.Iterable<SloFlowStatus>? flows,
  }) {
    final result = create();
    if (flows != null) result.flows.addAll(flows);
    return result;
  }

  GetSloStatusResponse._();

  factory GetSloStatusResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory GetSloStatusResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'GetSloStatusResponse',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.eventmonitor.v1'),
      createEmptyInstance: create)
    ..pPM<SloFlowStatus>(1, _omitFieldNames ? '' : 'flows',
        subBuilder: SloFlowStatus.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetSloStatusResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetSloStatusResponse copyWith(void Function(GetSloStatusResponse) updates) =>
      super.copyWith((message) => updates(message as GetSloStatusResponse))
          as GetSloStatusResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static GetSloStatusResponse create() => GetSloStatusResponse._();
  @$core.override
  GetSloStatusResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static GetSloStatusResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<GetSloStatusResponse>(create);
  static GetSloStatusResponse? _defaultInstance;

  @$pb.TagNumber(1)
  $pb.PbList<SloFlowStatus> get flows => $_getList(0);
}

class BurnRateWindow extends $pb.GeneratedMessage {
  factory BurnRateWindow({
    $core.String? window,
    $core.double? burnRate,
    $core.double? errorBudgetRemaining,
  }) {
    final result = create();
    if (window != null) result.window = window;
    if (burnRate != null) result.burnRate = burnRate;
    if (errorBudgetRemaining != null)
      result.errorBudgetRemaining = errorBudgetRemaining;
    return result;
  }

  BurnRateWindow._();

  factory BurnRateWindow.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory BurnRateWindow.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'BurnRateWindow',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.eventmonitor.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'window')
    ..aD(2, _omitFieldNames ? '' : 'burnRate')
    ..aD(3, _omitFieldNames ? '' : 'errorBudgetRemaining')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  BurnRateWindow clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  BurnRateWindow copyWith(void Function(BurnRateWindow) updates) =>
      super.copyWith((message) => updates(message as BurnRateWindow))
          as BurnRateWindow;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static BurnRateWindow create() => BurnRateWindow._();
  @$core.override
  BurnRateWindow createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static BurnRateWindow getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<BurnRateWindow>(create);
  static BurnRateWindow? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get window => $_getSZ(0);
  @$pb.TagNumber(1)
  set window($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasWindow() => $_has(0);
  @$pb.TagNumber(1)
  void clearWindow() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.double get burnRate => $_getN(1);
  @$pb.TagNumber(2)
  set burnRate($core.double value) => $_setDouble(1, value);
  @$pb.TagNumber(2)
  $core.bool hasBurnRate() => $_has(1);
  @$pb.TagNumber(2)
  void clearBurnRate() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.double get errorBudgetRemaining => $_getN(2);
  @$pb.TagNumber(3)
  set errorBudgetRemaining($core.double value) => $_setDouble(2, value);
  @$pb.TagNumber(3)
  $core.bool hasErrorBudgetRemaining() => $_has(2);
  @$pb.TagNumber(3)
  void clearErrorBudgetRemaining() => $_clearField(3);
}

class GetSloBurnRateRequest extends $pb.GeneratedMessage {
  factory GetSloBurnRateRequest({
    $core.String? flowId,
  }) {
    final result = create();
    if (flowId != null) result.flowId = flowId;
    return result;
  }

  GetSloBurnRateRequest._();

  factory GetSloBurnRateRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory GetSloBurnRateRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'GetSloBurnRateRequest',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.eventmonitor.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'flowId')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetSloBurnRateRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetSloBurnRateRequest copyWith(
          void Function(GetSloBurnRateRequest) updates) =>
      super.copyWith((message) => updates(message as GetSloBurnRateRequest))
          as GetSloBurnRateRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static GetSloBurnRateRequest create() => GetSloBurnRateRequest._();
  @$core.override
  GetSloBurnRateRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static GetSloBurnRateRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<GetSloBurnRateRequest>(create);
  static GetSloBurnRateRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get flowId => $_getSZ(0);
  @$pb.TagNumber(1)
  set flowId($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasFlowId() => $_has(0);
  @$pb.TagNumber(1)
  void clearFlowId() => $_clearField(1);
}

class GetSloBurnRateResponse extends $pb.GeneratedMessage {
  factory GetSloBurnRateResponse({
    $core.String? flowId,
    $core.String? flowName,
    $core.Iterable<BurnRateWindow>? windows,
    $core.String? alertStatus,
    $1.Timestamp? alertFiredAt,
  }) {
    final result = create();
    if (flowId != null) result.flowId = flowId;
    if (flowName != null) result.flowName = flowName;
    if (windows != null) result.windows.addAll(windows);
    if (alertStatus != null) result.alertStatus = alertStatus;
    if (alertFiredAt != null) result.alertFiredAt = alertFiredAt;
    return result;
  }

  GetSloBurnRateResponse._();

  factory GetSloBurnRateResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory GetSloBurnRateResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'GetSloBurnRateResponse',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.eventmonitor.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'flowId')
    ..aOS(2, _omitFieldNames ? '' : 'flowName')
    ..pPM<BurnRateWindow>(3, _omitFieldNames ? '' : 'windows',
        subBuilder: BurnRateWindow.create)
    ..aOS(4, _omitFieldNames ? '' : 'alertStatus')
    ..aOM<$1.Timestamp>(5, _omitFieldNames ? '' : 'alertFiredAt',
        subBuilder: $1.Timestamp.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetSloBurnRateResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetSloBurnRateResponse copyWith(
          void Function(GetSloBurnRateResponse) updates) =>
      super.copyWith((message) => updates(message as GetSloBurnRateResponse))
          as GetSloBurnRateResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static GetSloBurnRateResponse create() => GetSloBurnRateResponse._();
  @$core.override
  GetSloBurnRateResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static GetSloBurnRateResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<GetSloBurnRateResponse>(create);
  static GetSloBurnRateResponse? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get flowId => $_getSZ(0);
  @$pb.TagNumber(1)
  set flowId($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasFlowId() => $_has(0);
  @$pb.TagNumber(1)
  void clearFlowId() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get flowName => $_getSZ(1);
  @$pb.TagNumber(2)
  set flowName($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasFlowName() => $_has(1);
  @$pb.TagNumber(2)
  void clearFlowName() => $_clearField(2);

  @$pb.TagNumber(3)
  $pb.PbList<BurnRateWindow> get windows => $_getList(2);

  @$pb.TagNumber(4)
  $core.String get alertStatus => $_getSZ(3);
  @$pb.TagNumber(4)
  set alertStatus($core.String value) => $_setString(3, value);
  @$pb.TagNumber(4)
  $core.bool hasAlertStatus() => $_has(3);
  @$pb.TagNumber(4)
  void clearAlertStatus() => $_clearField(4);

  @$pb.TagNumber(5)
  $1.Timestamp get alertFiredAt => $_getN(4);
  @$pb.TagNumber(5)
  set alertFiredAt($1.Timestamp value) => $_setField(5, value);
  @$pb.TagNumber(5)
  $core.bool hasAlertFiredAt() => $_has(4);
  @$pb.TagNumber(5)
  void clearAlertFiredAt() => $_clearField(5);
  @$pb.TagNumber(5)
  $1.Timestamp ensureAlertFiredAt() => $_ensure(4);
}

class ReplayFlowPreview extends $pb.GeneratedMessage {
  factory ReplayFlowPreview({
    $core.String? correlationId,
    $core.String? flowName,
    $core.int? replayFromStep,
    $core.int? eventsToReplay,
  }) {
    final result = create();
    if (correlationId != null) result.correlationId = correlationId;
    if (flowName != null) result.flowName = flowName;
    if (replayFromStep != null) result.replayFromStep = replayFromStep;
    if (eventsToReplay != null) result.eventsToReplay = eventsToReplay;
    return result;
  }

  ReplayFlowPreview._();

  factory ReplayFlowPreview.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory ReplayFlowPreview.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ReplayFlowPreview',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.eventmonitor.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'correlationId')
    ..aOS(2, _omitFieldNames ? '' : 'flowName')
    ..aI(3, _omitFieldNames ? '' : 'replayFromStep')
    ..aI(4, _omitFieldNames ? '' : 'eventsToReplay')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ReplayFlowPreview clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ReplayFlowPreview copyWith(void Function(ReplayFlowPreview) updates) =>
      super.copyWith((message) => updates(message as ReplayFlowPreview))
          as ReplayFlowPreview;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static ReplayFlowPreview create() => ReplayFlowPreview._();
  @$core.override
  ReplayFlowPreview createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static ReplayFlowPreview getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ReplayFlowPreview>(create);
  static ReplayFlowPreview? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get correlationId => $_getSZ(0);
  @$pb.TagNumber(1)
  set correlationId($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasCorrelationId() => $_has(0);
  @$pb.TagNumber(1)
  void clearCorrelationId() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get flowName => $_getSZ(1);
  @$pb.TagNumber(2)
  set flowName($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasFlowName() => $_has(1);
  @$pb.TagNumber(2)
  void clearFlowName() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.int get replayFromStep => $_getIZ(2);
  @$pb.TagNumber(3)
  set replayFromStep($core.int value) => $_setSignedInt32(2, value);
  @$pb.TagNumber(3)
  $core.bool hasReplayFromStep() => $_has(2);
  @$pb.TagNumber(3)
  void clearReplayFromStep() => $_clearField(3);

  @$pb.TagNumber(4)
  $core.int get eventsToReplay => $_getIZ(3);
  @$pb.TagNumber(4)
  set eventsToReplay($core.int value) => $_setSignedInt32(3, value);
  @$pb.TagNumber(4)
  $core.bool hasEventsToReplay() => $_has(3);
  @$pb.TagNumber(4)
  void clearEventsToReplay() => $_clearField(4);
}

class PreviewReplayRequest extends $pb.GeneratedMessage {
  factory PreviewReplayRequest({
    $core.Iterable<$core.String>? correlationIds,
    $core.int? fromStepIndex,
    $core.bool? includeDownstream,
  }) {
    final result = create();
    if (correlationIds != null) result.correlationIds.addAll(correlationIds);
    if (fromStepIndex != null) result.fromStepIndex = fromStepIndex;
    if (includeDownstream != null) result.includeDownstream = includeDownstream;
    return result;
  }

  PreviewReplayRequest._();

  factory PreviewReplayRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory PreviewReplayRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'PreviewReplayRequest',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.eventmonitor.v1'),
      createEmptyInstance: create)
    ..pPS(1, _omitFieldNames ? '' : 'correlationIds')
    ..aI(2, _omitFieldNames ? '' : 'fromStepIndex')
    ..aOB(3, _omitFieldNames ? '' : 'includeDownstream')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  PreviewReplayRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  PreviewReplayRequest copyWith(void Function(PreviewReplayRequest) updates) =>
      super.copyWith((message) => updates(message as PreviewReplayRequest))
          as PreviewReplayRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static PreviewReplayRequest create() => PreviewReplayRequest._();
  @$core.override
  PreviewReplayRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static PreviewReplayRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<PreviewReplayRequest>(create);
  static PreviewReplayRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $pb.PbList<$core.String> get correlationIds => $_getList(0);

  @$pb.TagNumber(2)
  $core.int get fromStepIndex => $_getIZ(1);
  @$pb.TagNumber(2)
  set fromStepIndex($core.int value) => $_setSignedInt32(1, value);
  @$pb.TagNumber(2)
  $core.bool hasFromStepIndex() => $_has(1);
  @$pb.TagNumber(2)
  void clearFromStepIndex() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.bool get includeDownstream => $_getBF(2);
  @$pb.TagNumber(3)
  set includeDownstream($core.bool value) => $_setBool(2, value);
  @$pb.TagNumber(3)
  $core.bool hasIncludeDownstream() => $_has(2);
  @$pb.TagNumber(3)
  void clearIncludeDownstream() => $_clearField(3);
}

class PreviewReplayResponse extends $pb.GeneratedMessage {
  factory PreviewReplayResponse({
    $core.int? totalEventsToReplay,
    $core.Iterable<$core.String>? affectedServices,
    $core.Iterable<ReplayFlowPreview>? affectedFlows,
    $core.int? dlqMessagesFound,
    $core.int? estimatedDurationSeconds,
  }) {
    final result = create();
    if (totalEventsToReplay != null)
      result.totalEventsToReplay = totalEventsToReplay;
    if (affectedServices != null)
      result.affectedServices.addAll(affectedServices);
    if (affectedFlows != null) result.affectedFlows.addAll(affectedFlows);
    if (dlqMessagesFound != null) result.dlqMessagesFound = dlqMessagesFound;
    if (estimatedDurationSeconds != null)
      result.estimatedDurationSeconds = estimatedDurationSeconds;
    return result;
  }

  PreviewReplayResponse._();

  factory PreviewReplayResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory PreviewReplayResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'PreviewReplayResponse',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.eventmonitor.v1'),
      createEmptyInstance: create)
    ..aI(1, _omitFieldNames ? '' : 'totalEventsToReplay')
    ..pPS(2, _omitFieldNames ? '' : 'affectedServices')
    ..pPM<ReplayFlowPreview>(3, _omitFieldNames ? '' : 'affectedFlows',
        subBuilder: ReplayFlowPreview.create)
    ..aI(4, _omitFieldNames ? '' : 'dlqMessagesFound')
    ..aI(5, _omitFieldNames ? '' : 'estimatedDurationSeconds')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  PreviewReplayResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  PreviewReplayResponse copyWith(
          void Function(PreviewReplayResponse) updates) =>
      super.copyWith((message) => updates(message as PreviewReplayResponse))
          as PreviewReplayResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static PreviewReplayResponse create() => PreviewReplayResponse._();
  @$core.override
  PreviewReplayResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static PreviewReplayResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<PreviewReplayResponse>(create);
  static PreviewReplayResponse? _defaultInstance;

  @$pb.TagNumber(1)
  $core.int get totalEventsToReplay => $_getIZ(0);
  @$pb.TagNumber(1)
  set totalEventsToReplay($core.int value) => $_setSignedInt32(0, value);
  @$pb.TagNumber(1)
  $core.bool hasTotalEventsToReplay() => $_has(0);
  @$pb.TagNumber(1)
  void clearTotalEventsToReplay() => $_clearField(1);

  @$pb.TagNumber(2)
  $pb.PbList<$core.String> get affectedServices => $_getList(1);

  @$pb.TagNumber(3)
  $pb.PbList<ReplayFlowPreview> get affectedFlows => $_getList(2);

  @$pb.TagNumber(4)
  $core.int get dlqMessagesFound => $_getIZ(3);
  @$pb.TagNumber(4)
  set dlqMessagesFound($core.int value) => $_setSignedInt32(3, value);
  @$pb.TagNumber(4)
  $core.bool hasDlqMessagesFound() => $_has(3);
  @$pb.TagNumber(4)
  void clearDlqMessagesFound() => $_clearField(4);

  @$pb.TagNumber(5)
  $core.int get estimatedDurationSeconds => $_getIZ(4);
  @$pb.TagNumber(5)
  set estimatedDurationSeconds($core.int value) => $_setSignedInt32(4, value);
  @$pb.TagNumber(5)
  $core.bool hasEstimatedDurationSeconds() => $_has(4);
  @$pb.TagNumber(5)
  void clearEstimatedDurationSeconds() => $_clearField(5);
}

class ExecuteReplayRequest extends $pb.GeneratedMessage {
  factory ExecuteReplayRequest({
    $core.Iterable<$core.String>? correlationIds,
    $core.int? fromStepIndex,
    $core.bool? includeDownstream,
    $core.bool? dryRun,
  }) {
    final result = create();
    if (correlationIds != null) result.correlationIds.addAll(correlationIds);
    if (fromStepIndex != null) result.fromStepIndex = fromStepIndex;
    if (includeDownstream != null) result.includeDownstream = includeDownstream;
    if (dryRun != null) result.dryRun = dryRun;
    return result;
  }

  ExecuteReplayRequest._();

  factory ExecuteReplayRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory ExecuteReplayRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ExecuteReplayRequest',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.eventmonitor.v1'),
      createEmptyInstance: create)
    ..pPS(1, _omitFieldNames ? '' : 'correlationIds')
    ..aI(2, _omitFieldNames ? '' : 'fromStepIndex')
    ..aOB(3, _omitFieldNames ? '' : 'includeDownstream')
    ..aOB(4, _omitFieldNames ? '' : 'dryRun')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ExecuteReplayRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ExecuteReplayRequest copyWith(void Function(ExecuteReplayRequest) updates) =>
      super.copyWith((message) => updates(message as ExecuteReplayRequest))
          as ExecuteReplayRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static ExecuteReplayRequest create() => ExecuteReplayRequest._();
  @$core.override
  ExecuteReplayRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static ExecuteReplayRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ExecuteReplayRequest>(create);
  static ExecuteReplayRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $pb.PbList<$core.String> get correlationIds => $_getList(0);

  @$pb.TagNumber(2)
  $core.int get fromStepIndex => $_getIZ(1);
  @$pb.TagNumber(2)
  set fromStepIndex($core.int value) => $_setSignedInt32(1, value);
  @$pb.TagNumber(2)
  $core.bool hasFromStepIndex() => $_has(1);
  @$pb.TagNumber(2)
  void clearFromStepIndex() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.bool get includeDownstream => $_getBF(2);
  @$pb.TagNumber(3)
  set includeDownstream($core.bool value) => $_setBool(2, value);
  @$pb.TagNumber(3)
  $core.bool hasIncludeDownstream() => $_has(2);
  @$pb.TagNumber(3)
  void clearIncludeDownstream() => $_clearField(3);

  @$pb.TagNumber(4)
  $core.bool get dryRun => $_getBF(3);
  @$pb.TagNumber(4)
  set dryRun($core.bool value) => $_setBool(3, value);
  @$pb.TagNumber(4)
  $core.bool hasDryRun() => $_has(3);
  @$pb.TagNumber(4)
  void clearDryRun() => $_clearField(4);
}

class ExecuteReplayResponse extends $pb.GeneratedMessage {
  factory ExecuteReplayResponse({
    $core.String? replayId,
    $core.String? status,
    $core.int? totalEvents,
    $core.int? replayedEvents,
    $1.Timestamp? startedAt,
  }) {
    final result = create();
    if (replayId != null) result.replayId = replayId;
    if (status != null) result.status = status;
    if (totalEvents != null) result.totalEvents = totalEvents;
    if (replayedEvents != null) result.replayedEvents = replayedEvents;
    if (startedAt != null) result.startedAt = startedAt;
    return result;
  }

  ExecuteReplayResponse._();

  factory ExecuteReplayResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory ExecuteReplayResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ExecuteReplayResponse',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.eventmonitor.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'replayId')
    ..aOS(2, _omitFieldNames ? '' : 'status')
    ..aI(3, _omitFieldNames ? '' : 'totalEvents')
    ..aI(4, _omitFieldNames ? '' : 'replayedEvents')
    ..aOM<$1.Timestamp>(5, _omitFieldNames ? '' : 'startedAt',
        subBuilder: $1.Timestamp.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ExecuteReplayResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ExecuteReplayResponse copyWith(
          void Function(ExecuteReplayResponse) updates) =>
      super.copyWith((message) => updates(message as ExecuteReplayResponse))
          as ExecuteReplayResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static ExecuteReplayResponse create() => ExecuteReplayResponse._();
  @$core.override
  ExecuteReplayResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static ExecuteReplayResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ExecuteReplayResponse>(create);
  static ExecuteReplayResponse? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get replayId => $_getSZ(0);
  @$pb.TagNumber(1)
  set replayId($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasReplayId() => $_has(0);
  @$pb.TagNumber(1)
  void clearReplayId() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get status => $_getSZ(1);
  @$pb.TagNumber(2)
  set status($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasStatus() => $_has(1);
  @$pb.TagNumber(2)
  void clearStatus() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.int get totalEvents => $_getIZ(2);
  @$pb.TagNumber(3)
  set totalEvents($core.int value) => $_setSignedInt32(2, value);
  @$pb.TagNumber(3)
  $core.bool hasTotalEvents() => $_has(2);
  @$pb.TagNumber(3)
  void clearTotalEvents() => $_clearField(3);

  @$pb.TagNumber(4)
  $core.int get replayedEvents => $_getIZ(3);
  @$pb.TagNumber(4)
  set replayedEvents($core.int value) => $_setSignedInt32(3, value);
  @$pb.TagNumber(4)
  $core.bool hasReplayedEvents() => $_has(3);
  @$pb.TagNumber(4)
  void clearReplayedEvents() => $_clearField(4);

  @$pb.TagNumber(5)
  $1.Timestamp get startedAt => $_getN(4);
  @$pb.TagNumber(5)
  set startedAt($1.Timestamp value) => $_setField(5, value);
  @$pb.TagNumber(5)
  $core.bool hasStartedAt() => $_has(4);
  @$pb.TagNumber(5)
  void clearStartedAt() => $_clearField(5);
  @$pb.TagNumber(5)
  $1.Timestamp ensureStartedAt() => $_ensure(4);
}

const $core.bool _omitFieldNames =
    $core.bool.fromEnvironment('protobuf.omit_field_names');
const $core.bool _omitMessageNames =
    $core.bool.fromEnvironment('protobuf.omit_message_names');
