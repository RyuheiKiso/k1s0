// This is a generated file - do not edit.
//
// Generated from k1s0/event/system/config/v1/config_events.proto.

// @dart = 3.3

// ignore_for_file: annotate_overrides, camel_case_types, comment_references
// ignore_for_file: constant_identifier_names
// ignore_for_file: curly_braces_in_flow_control_structures
// ignore_for_file: deprecated_member_use_from_same_package, library_prefixes
// ignore_for_file: non_constant_identifier_names, prefer_relative_imports

import 'dart:core' as $core;

import 'package:protobuf/protobuf.dart' as $pb;

import '../../../../system/common/v1/event_metadata.pb.dart' as $0;

export 'package:protobuf/protobuf.dart' show GeneratedMessageGenericExtensions;

/// ConfigChangedEvent は設定値変更イベント。
/// Kafka トピック: k1s0.system.config.changed.v1
/// パーティションキー: namespace
class ConfigChangedEvent extends $pb.GeneratedMessage {
  factory ConfigChangedEvent({
    $0.EventMetadata? metadata,
    $core.String? namespace,
    $core.String? key,
    $core.String? oldValue,
    $core.String? newValue,
    $core.int? oldVersion,
    $core.int? newVersion,
    $core.String? changeType,
    $core.String? changedBy,
  }) {
    final result = create();
    if (metadata != null) result.metadata = metadata;
    if (namespace != null) result.namespace = namespace;
    if (key != null) result.key = key;
    if (oldValue != null) result.oldValue = oldValue;
    if (newValue != null) result.newValue = newValue;
    if (oldVersion != null) result.oldVersion = oldVersion;
    if (newVersion != null) result.newVersion = newVersion;
    if (changeType != null) result.changeType = changeType;
    if (changedBy != null) result.changedBy = changedBy;
    return result;
  }

  ConfigChangedEvent._();

  factory ConfigChangedEvent.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory ConfigChangedEvent.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ConfigChangedEvent',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.event.system.config.v1'),
      createEmptyInstance: create)
    ..aOM<$0.EventMetadata>(1, _omitFieldNames ? '' : 'metadata',
        subBuilder: $0.EventMetadata.create)
    ..aOS(2, _omitFieldNames ? '' : 'namespace')
    ..aOS(3, _omitFieldNames ? '' : 'key')
    ..aOS(4, _omitFieldNames ? '' : 'oldValue')
    ..aOS(5, _omitFieldNames ? '' : 'newValue')
    ..aI(6, _omitFieldNames ? '' : 'oldVersion')
    ..aI(7, _omitFieldNames ? '' : 'newVersion')
    ..aOS(8, _omitFieldNames ? '' : 'changeType')
    ..aOS(9, _omitFieldNames ? '' : 'changedBy')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ConfigChangedEvent clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ConfigChangedEvent copyWith(void Function(ConfigChangedEvent) updates) =>
      super.copyWith((message) => updates(message as ConfigChangedEvent))
          as ConfigChangedEvent;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static ConfigChangedEvent create() => ConfigChangedEvent._();
  @$core.override
  ConfigChangedEvent createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static ConfigChangedEvent getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ConfigChangedEvent>(create);
  static ConfigChangedEvent? _defaultInstance;

  @$pb.TagNumber(1)
  $0.EventMetadata get metadata => $_getN(0);
  @$pb.TagNumber(1)
  set metadata($0.EventMetadata value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasMetadata() => $_has(0);
  @$pb.TagNumber(1)
  void clearMetadata() => $_clearField(1);
  @$pb.TagNumber(1)
  $0.EventMetadata ensureMetadata() => $_ensure(0);

  @$pb.TagNumber(2)
  $core.String get namespace => $_getSZ(1);
  @$pb.TagNumber(2)
  set namespace($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasNamespace() => $_has(1);
  @$pb.TagNumber(2)
  void clearNamespace() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.String get key => $_getSZ(2);
  @$pb.TagNumber(3)
  set key($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasKey() => $_has(2);
  @$pb.TagNumber(3)
  void clearKey() => $_clearField(3);

  /// JSON 文字列（変更前。新規作成時は空）
  @$pb.TagNumber(4)
  $core.String get oldValue => $_getSZ(3);
  @$pb.TagNumber(4)
  set oldValue($core.String value) => $_setString(3, value);
  @$pb.TagNumber(4)
  $core.bool hasOldValue() => $_has(3);
  @$pb.TagNumber(4)
  void clearOldValue() => $_clearField(4);

  /// JSON 文字列（変更後。削除時は空）
  @$pb.TagNumber(5)
  $core.String get newValue => $_getSZ(4);
  @$pb.TagNumber(5)
  set newValue($core.String value) => $_setString(4, value);
  @$pb.TagNumber(5)
  $core.bool hasNewValue() => $_has(4);
  @$pb.TagNumber(5)
  void clearNewValue() => $_clearField(5);

  @$pb.TagNumber(6)
  $core.int get oldVersion => $_getIZ(5);
  @$pb.TagNumber(6)
  set oldVersion($core.int value) => $_setSignedInt32(5, value);
  @$pb.TagNumber(6)
  $core.bool hasOldVersion() => $_has(5);
  @$pb.TagNumber(6)
  void clearOldVersion() => $_clearField(6);

  @$pb.TagNumber(7)
  $core.int get newVersion => $_getIZ(6);
  @$pb.TagNumber(7)
  set newVersion($core.int value) => $_setSignedInt32(6, value);
  @$pb.TagNumber(7)
  $core.bool hasNewVersion() => $_has(6);
  @$pb.TagNumber(7)
  void clearNewVersion() => $_clearField(7);

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
  $core.String get changedBy => $_getSZ(8);
  @$pb.TagNumber(9)
  set changedBy($core.String value) => $_setString(8, value);
  @$pb.TagNumber(9)
  $core.bool hasChangedBy() => $_has(8);
  @$pb.TagNumber(9)
  void clearChangedBy() => $_clearField(9);
}

const $core.bool _omitFieldNames =
    $core.bool.fromEnvironment('protobuf.omit_field_names');
const $core.bool _omitMessageNames =
    $core.bool.fromEnvironment('protobuf.omit_message_names');
