// This is a generated file - do not edit.
//
// Generated from k1s0/system/common/v1/event_metadata.proto.

// @dart = 3.3

// ignore_for_file: annotate_overrides, camel_case_types, comment_references
// ignore_for_file: constant_identifier_names
// ignore_for_file: curly_braces_in_flow_control_structures
// ignore_for_file: deprecated_member_use_from_same_package, library_prefixes
// ignore_for_file: non_constant_identifier_names, prefer_relative_imports

import 'dart:core' as $core;

import 'package:fixnum/fixnum.dart' as $fixnum;
import 'package:protobuf/protobuf.dart' as $pb;

export 'package:protobuf/protobuf.dart' show GeneratedMessageGenericExtensions;

/// EventMetadata は全イベントに付与する共通メタデータ。
class EventMetadata extends $pb.GeneratedMessage {
  factory EventMetadata({
    $core.String? eventId,
    $core.String? eventType,
    $core.String? source,
    $fixnum.Int64? timestamp,
    $core.String? traceId,
    $core.String? correlationId,
    $core.int? schemaVersion,
    $core.String? causationId,
  }) {
    final result = create();
    if (eventId != null) result.eventId = eventId;
    if (eventType != null) result.eventType = eventType;
    if (source != null) result.source = source;
    if (timestamp != null) result.timestamp = timestamp;
    if (traceId != null) result.traceId = traceId;
    if (correlationId != null) result.correlationId = correlationId;
    if (schemaVersion != null) result.schemaVersion = schemaVersion;
    if (causationId != null) result.causationId = causationId;
    return result;
  }

  EventMetadata._();

  factory EventMetadata.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory EventMetadata.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'EventMetadata',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.common.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'eventId')
    ..aOS(2, _omitFieldNames ? '' : 'eventType')
    ..aOS(3, _omitFieldNames ? '' : 'source')
    ..aInt64(4, _omitFieldNames ? '' : 'timestamp')
    ..aOS(5, _omitFieldNames ? '' : 'traceId')
    ..aOS(6, _omitFieldNames ? '' : 'correlationId')
    ..aI(7, _omitFieldNames ? '' : 'schemaVersion')
    ..aOS(8, _omitFieldNames ? '' : 'causationId')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  EventMetadata clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  EventMetadata copyWith(void Function(EventMetadata) updates) =>
      super.copyWith((message) => updates(message as EventMetadata))
          as EventMetadata;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static EventMetadata create() => EventMetadata._();
  @$core.override
  EventMetadata createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static EventMetadata getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<EventMetadata>(create);
  static EventMetadata? _defaultInstance;

  /// UUID
  @$pb.TagNumber(1)
  $core.String get eventId => $_getSZ(0);
  @$pb.TagNumber(1)
  set eventId($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasEventId() => $_has(0);
  @$pb.TagNumber(1)
  void clearEventId() => $_clearField(1);

  /// イベント種別（例: "auth.audit.recorded"）
  @$pb.TagNumber(2)
  $core.String get eventType => $_getSZ(1);
  @$pb.TagNumber(2)
  set eventType($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasEventType() => $_has(1);
  @$pb.TagNumber(2)
  void clearEventType() => $_clearField(2);

  /// イベント発行元（例: "auth-server"）
  @$pb.TagNumber(3)
  $core.String get source => $_getSZ(2);
  @$pb.TagNumber(3)
  set source($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasSource() => $_has(2);
  @$pb.TagNumber(3)
  void clearSource() => $_clearField(3);

  /// Unix timestamp (ms)
  @$pb.TagNumber(4)
  $fixnum.Int64 get timestamp => $_getI64(3);
  @$pb.TagNumber(4)
  set timestamp($fixnum.Int64 value) => $_setInt64(3, value);
  @$pb.TagNumber(4)
  $core.bool hasTimestamp() => $_has(3);
  @$pb.TagNumber(4)
  void clearTimestamp() => $_clearField(4);

  /// 分散トレース ID
  @$pb.TagNumber(5)
  $core.String get traceId => $_getSZ(4);
  @$pb.TagNumber(5)
  set traceId($core.String value) => $_setString(4, value);
  @$pb.TagNumber(5)
  $core.bool hasTraceId() => $_has(4);
  @$pb.TagNumber(5)
  void clearTraceId() => $_clearField(5);

  /// 業務相関 ID
  @$pb.TagNumber(6)
  $core.String get correlationId => $_getSZ(5);
  @$pb.TagNumber(6)
  set correlationId($core.String value) => $_setString(5, value);
  @$pb.TagNumber(6)
  $core.bool hasCorrelationId() => $_has(5);
  @$pb.TagNumber(6)
  void clearCorrelationId() => $_clearField(6);

  /// スキーマバージョン
  @$pb.TagNumber(7)
  $core.int get schemaVersion => $_getIZ(6);
  @$pb.TagNumber(7)
  set schemaVersion($core.int value) => $_setSignedInt32(6, value);
  @$pb.TagNumber(7)
  $core.bool hasSchemaVersion() => $_has(6);
  @$pb.TagNumber(7)
  void clearSchemaVersion() => $_clearField(7);

  /// 因果関係追跡用 ID（このイベントを引き起こしたコマンド/イベントの ID）
  @$pb.TagNumber(8)
  $core.String get causationId => $_getSZ(7);
  @$pb.TagNumber(8)
  set causationId($core.String value) => $_setString(7, value);
  @$pb.TagNumber(8)
  $core.bool hasCausationId() => $_has(7);
  @$pb.TagNumber(8)
  void clearCausationId() => $_clearField(8);
}

const $core.bool _omitFieldNames =
    $core.bool.fromEnvironment('protobuf.omit_field_names');
const $core.bool _omitMessageNames =
    $core.bool.fromEnvironment('protobuf.omit_message_names');
