// This is a generated file - do not edit.
//
// Generated from k1s0/system/auth/v1/auth.proto.

// @dart = 3.3

// ignore_for_file: annotate_overrides, camel_case_types, comment_references
// ignore_for_file: constant_identifier_names
// ignore_for_file: curly_braces_in_flow_control_structures
// ignore_for_file: deprecated_member_use_from_same_package, library_prefixes
// ignore_for_file: non_constant_identifier_names, prefer_relative_imports

import 'dart:core' as $core;

import 'package:protobuf/protobuf.dart' as $pb;

/// AuditEventType は監査イベントの種別。
class AuditEventType extends $pb.ProtobufEnum {
  static const AuditEventType AUDIT_EVENT_TYPE_UNSPECIFIED =
      AuditEventType._(0, _omitEnumNames ? '' : 'AUDIT_EVENT_TYPE_UNSPECIFIED');
  static const AuditEventType AUDIT_EVENT_TYPE_LOGIN =
      AuditEventType._(1, _omitEnumNames ? '' : 'AUDIT_EVENT_TYPE_LOGIN');
  static const AuditEventType AUDIT_EVENT_TYPE_LOGOUT =
      AuditEventType._(2, _omitEnumNames ? '' : 'AUDIT_EVENT_TYPE_LOGOUT');
  static const AuditEventType AUDIT_EVENT_TYPE_TOKEN_REFRESH = AuditEventType._(
      3, _omitEnumNames ? '' : 'AUDIT_EVENT_TYPE_TOKEN_REFRESH');
  static const AuditEventType AUDIT_EVENT_TYPE_PERMISSION_CHECK =
      AuditEventType._(
          4, _omitEnumNames ? '' : 'AUDIT_EVENT_TYPE_PERMISSION_CHECK');
  static const AuditEventType AUDIT_EVENT_TYPE_API_KEY_CREATED =
      AuditEventType._(
          5, _omitEnumNames ? '' : 'AUDIT_EVENT_TYPE_API_KEY_CREATED');
  static const AuditEventType AUDIT_EVENT_TYPE_API_KEY_REVOKED =
      AuditEventType._(
          6, _omitEnumNames ? '' : 'AUDIT_EVENT_TYPE_API_KEY_REVOKED');

  static const $core.List<AuditEventType> values = <AuditEventType>[
    AUDIT_EVENT_TYPE_UNSPECIFIED,
    AUDIT_EVENT_TYPE_LOGIN,
    AUDIT_EVENT_TYPE_LOGOUT,
    AUDIT_EVENT_TYPE_TOKEN_REFRESH,
    AUDIT_EVENT_TYPE_PERMISSION_CHECK,
    AUDIT_EVENT_TYPE_API_KEY_CREATED,
    AUDIT_EVENT_TYPE_API_KEY_REVOKED,
  ];

  static final $core.List<AuditEventType?> _byValue =
      $pb.ProtobufEnum.$_initByValueList(values, 6);
  static AuditEventType? valueOf($core.int value) =>
      value < 0 || value >= _byValue.length ? null : _byValue[value];

  const AuditEventType._(super.value, super.name);
}

/// AuditResult は監査イベントの結果。
class AuditResult extends $pb.ProtobufEnum {
  static const AuditResult AUDIT_RESULT_UNSPECIFIED =
      AuditResult._(0, _omitEnumNames ? '' : 'AUDIT_RESULT_UNSPECIFIED');
  static const AuditResult AUDIT_RESULT_SUCCESS =
      AuditResult._(1, _omitEnumNames ? '' : 'AUDIT_RESULT_SUCCESS');
  static const AuditResult AUDIT_RESULT_FAILURE =
      AuditResult._(2, _omitEnumNames ? '' : 'AUDIT_RESULT_FAILURE');

  static const $core.List<AuditResult> values = <AuditResult>[
    AUDIT_RESULT_UNSPECIFIED,
    AUDIT_RESULT_SUCCESS,
    AUDIT_RESULT_FAILURE,
  ];

  static final $core.List<AuditResult?> _byValue =
      $pb.ProtobufEnum.$_initByValueList(values, 2);
  static AuditResult? valueOf($core.int value) =>
      value < 0 || value >= _byValue.length ? null : _byValue[value];

  const AuditResult._(super.value, super.name);
}

const $core.bool _omitEnumNames =
    $core.bool.fromEnvironment('protobuf.omit_enum_names');
