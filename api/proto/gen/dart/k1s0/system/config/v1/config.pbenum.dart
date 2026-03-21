// This is a generated file - do not edit.
//
// Generated from k1s0/system/config/v1/config.proto.

// @dart = 3.3

// ignore_for_file: annotate_overrides, camel_case_types, comment_references
// ignore_for_file: constant_identifier_names
// ignore_for_file: curly_braces_in_flow_control_structures
// ignore_for_file: deprecated_member_use_from_same_package, library_prefixes
// ignore_for_file: non_constant_identifier_names, prefer_relative_imports

import 'dart:core' as $core;

import 'package:protobuf/protobuf.dart' as $pb;

/// ConfigFieldType は設定フィールドの型を表す。
class ConfigFieldType extends $pb.ProtobufEnum {
  static const ConfigFieldType CONFIG_FIELD_TYPE_UNSPECIFIED =
      ConfigFieldType._(
          0, _omitEnumNames ? '' : 'CONFIG_FIELD_TYPE_UNSPECIFIED');
  static const ConfigFieldType CONFIG_FIELD_TYPE_STRING =
      ConfigFieldType._(1, _omitEnumNames ? '' : 'CONFIG_FIELD_TYPE_STRING');
  static const ConfigFieldType CONFIG_FIELD_TYPE_INTEGER =
      ConfigFieldType._(2, _omitEnumNames ? '' : 'CONFIG_FIELD_TYPE_INTEGER');
  static const ConfigFieldType CONFIG_FIELD_TYPE_FLOAT =
      ConfigFieldType._(3, _omitEnumNames ? '' : 'CONFIG_FIELD_TYPE_FLOAT');
  static const ConfigFieldType CONFIG_FIELD_TYPE_BOOLEAN =
      ConfigFieldType._(4, _omitEnumNames ? '' : 'CONFIG_FIELD_TYPE_BOOLEAN');
  static const ConfigFieldType CONFIG_FIELD_TYPE_ENUM =
      ConfigFieldType._(5, _omitEnumNames ? '' : 'CONFIG_FIELD_TYPE_ENUM');
  static const ConfigFieldType CONFIG_FIELD_TYPE_OBJECT =
      ConfigFieldType._(6, _omitEnumNames ? '' : 'CONFIG_FIELD_TYPE_OBJECT');
  static const ConfigFieldType CONFIG_FIELD_TYPE_ARRAY =
      ConfigFieldType._(7, _omitEnumNames ? '' : 'CONFIG_FIELD_TYPE_ARRAY');

  static const $core.List<ConfigFieldType> values = <ConfigFieldType>[
    CONFIG_FIELD_TYPE_UNSPECIFIED,
    CONFIG_FIELD_TYPE_STRING,
    CONFIG_FIELD_TYPE_INTEGER,
    CONFIG_FIELD_TYPE_FLOAT,
    CONFIG_FIELD_TYPE_BOOLEAN,
    CONFIG_FIELD_TYPE_ENUM,
    CONFIG_FIELD_TYPE_OBJECT,
    CONFIG_FIELD_TYPE_ARRAY,
  ];

  static final $core.List<ConfigFieldType?> _byValue =
      $pb.ProtobufEnum.$_initByValueList(values, 7);
  static ConfigFieldType? valueOf($core.int value) =>
      value < 0 || value >= _byValue.length ? null : _byValue[value];

  const ConfigFieldType._(super.value, super.name);
}

const $core.bool _omitEnumNames =
    $core.bool.fromEnvironment('protobuf.omit_enum_names');
