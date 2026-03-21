// This is a generated file - do not edit.
//
// Generated from k1s0/system/dlq/v1/dlq.proto.

// @dart = 3.3

// ignore_for_file: annotate_overrides, camel_case_types, comment_references
// ignore_for_file: constant_identifier_names
// ignore_for_file: curly_braces_in_flow_control_structures
// ignore_for_file: deprecated_member_use_from_same_package, library_prefixes
// ignore_for_file: non_constant_identifier_names, prefer_relative_imports

import 'dart:core' as $core;

import 'package:protobuf/protobuf.dart' as $pb;

/// DlqMessageStatus は DLQ メッセージのステータス。
class DlqMessageStatus extends $pb.ProtobufEnum {
  static const DlqMessageStatus DLQ_MESSAGE_STATUS_UNSPECIFIED =
      DlqMessageStatus._(
          0, _omitEnumNames ? '' : 'DLQ_MESSAGE_STATUS_UNSPECIFIED');
  static const DlqMessageStatus DLQ_MESSAGE_STATUS_PENDING =
      DlqMessageStatus._(1, _omitEnumNames ? '' : 'DLQ_MESSAGE_STATUS_PENDING');
  static const DlqMessageStatus DLQ_MESSAGE_STATUS_RETRYING =
      DlqMessageStatus._(
          2, _omitEnumNames ? '' : 'DLQ_MESSAGE_STATUS_RETRYING');
  static const DlqMessageStatus DLQ_MESSAGE_STATUS_SUCCEEDED =
      DlqMessageStatus._(
          3, _omitEnumNames ? '' : 'DLQ_MESSAGE_STATUS_SUCCEEDED');
  static const DlqMessageStatus DLQ_MESSAGE_STATUS_FAILED =
      DlqMessageStatus._(4, _omitEnumNames ? '' : 'DLQ_MESSAGE_STATUS_FAILED');

  static const $core.List<DlqMessageStatus> values = <DlqMessageStatus>[
    DLQ_MESSAGE_STATUS_UNSPECIFIED,
    DLQ_MESSAGE_STATUS_PENDING,
    DLQ_MESSAGE_STATUS_RETRYING,
    DLQ_MESSAGE_STATUS_SUCCEEDED,
    DLQ_MESSAGE_STATUS_FAILED,
  ];

  static final $core.List<DlqMessageStatus?> _byValue =
      $pb.ProtobufEnum.$_initByValueList(values, 4);
  static DlqMessageStatus? valueOf($core.int value) =>
      value < 0 || value >= _byValue.length ? null : _byValue[value];

  const DlqMessageStatus._(super.value, super.name);
}

const $core.bool _omitEnumNames =
    $core.bool.fromEnvironment('protobuf.omit_enum_names');
