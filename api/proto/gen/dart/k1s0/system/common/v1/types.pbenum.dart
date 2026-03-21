// This is a generated file - do not edit.
//
// Generated from k1s0/system/common/v1/types.proto.

// @dart = 3.3

// ignore_for_file: annotate_overrides, camel_case_types, comment_references
// ignore_for_file: constant_identifier_names
// ignore_for_file: curly_braces_in_flow_control_structures
// ignore_for_file: deprecated_member_use_from_same_package, library_prefixes
// ignore_for_file: non_constant_identifier_names, prefer_relative_imports

import 'dart:core' as $core;

import 'package:protobuf/protobuf.dart' as $pb;

/// ChangeType は設定・フラグ変更操作の種別。
class ChangeType extends $pb.ProtobufEnum {
  /// CHANGE_TYPE_UNSPECIFIED は未指定（デフォルト値）。
  static const ChangeType CHANGE_TYPE_UNSPECIFIED =
      ChangeType._(0, _omitEnumNames ? '' : 'CHANGE_TYPE_UNSPECIFIED');

  /// CHANGE_TYPE_CREATED は新規作成。
  static const ChangeType CHANGE_TYPE_CREATED =
      ChangeType._(1, _omitEnumNames ? '' : 'CHANGE_TYPE_CREATED');

  /// CHANGE_TYPE_UPDATED は更新。
  static const ChangeType CHANGE_TYPE_UPDATED =
      ChangeType._(2, _omitEnumNames ? '' : 'CHANGE_TYPE_UPDATED');

  /// CHANGE_TYPE_DELETED は削除。
  static const ChangeType CHANGE_TYPE_DELETED =
      ChangeType._(3, _omitEnumNames ? '' : 'CHANGE_TYPE_DELETED');

  static const $core.List<ChangeType> values = <ChangeType>[
    CHANGE_TYPE_UNSPECIFIED,
    CHANGE_TYPE_CREATED,
    CHANGE_TYPE_UPDATED,
    CHANGE_TYPE_DELETED,
  ];

  static final $core.List<ChangeType?> _byValue =
      $pb.ProtobufEnum.$_initByValueList(values, 3);
  static ChangeType? valueOf($core.int value) =>
      value < 0 || value >= _byValue.length ? null : _byValue[value];

  const ChangeType._(super.value, super.name);
}

const $core.bool _omitEnumNames =
    $core.bool.fromEnvironment('protobuf.omit_enum_names');
