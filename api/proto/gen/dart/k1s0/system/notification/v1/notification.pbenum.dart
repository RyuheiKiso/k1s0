// This is a generated file - do not edit.
//
// Generated from k1s0/system/notification/v1/notification.proto.

// @dart = 3.3

// ignore_for_file: annotate_overrides, camel_case_types, comment_references
// ignore_for_file: constant_identifier_names
// ignore_for_file: curly_braces_in_flow_control_structures
// ignore_for_file: deprecated_member_use_from_same_package, library_prefixes
// ignore_for_file: non_constant_identifier_names, prefer_relative_imports

import 'dart:core' as $core;

import 'package:protobuf/protobuf.dart' as $pb;

/// NotificationStatus は通知配信のステータス。
class NotificationStatus extends $pb.ProtobufEnum {
  /// NOTIFICATION_STATUS_UNSPECIFIED は未指定（デフォルト値）。
  static const NotificationStatus NOTIFICATION_STATUS_UNSPECIFIED =
      NotificationStatus._(
          0, _omitEnumNames ? '' : 'NOTIFICATION_STATUS_UNSPECIFIED');

  /// NOTIFICATION_STATUS_PENDING は送信待ち。
  static const NotificationStatus NOTIFICATION_STATUS_PENDING =
      NotificationStatus._(
          1, _omitEnumNames ? '' : 'NOTIFICATION_STATUS_PENDING');

  /// NOTIFICATION_STATUS_SENT は送信済み。
  static const NotificationStatus NOTIFICATION_STATUS_SENT =
      NotificationStatus._(2, _omitEnumNames ? '' : 'NOTIFICATION_STATUS_SENT');

  /// NOTIFICATION_STATUS_FAILED は送信失敗。
  static const NotificationStatus NOTIFICATION_STATUS_FAILED =
      NotificationStatus._(
          3, _omitEnumNames ? '' : 'NOTIFICATION_STATUS_FAILED');

  /// NOTIFICATION_STATUS_RETRYING はリトライ中。
  static const NotificationStatus NOTIFICATION_STATUS_RETRYING =
      NotificationStatus._(
          4, _omitEnumNames ? '' : 'NOTIFICATION_STATUS_RETRYING');

  static const $core.List<NotificationStatus> values = <NotificationStatus>[
    NOTIFICATION_STATUS_UNSPECIFIED,
    NOTIFICATION_STATUS_PENDING,
    NOTIFICATION_STATUS_SENT,
    NOTIFICATION_STATUS_FAILED,
    NOTIFICATION_STATUS_RETRYING,
  ];

  static final $core.List<NotificationStatus?> _byValue =
      $pb.ProtobufEnum.$_initByValueList(values, 4);
  static NotificationStatus? valueOf($core.int value) =>
      value < 0 || value >= _byValue.length ? null : _byValue[value];

  const NotificationStatus._(super.value, super.name);
}

const $core.bool _omitEnumNames =
    $core.bool.fromEnvironment('protobuf.omit_enum_names');
