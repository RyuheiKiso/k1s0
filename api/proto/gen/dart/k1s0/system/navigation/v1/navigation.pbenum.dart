// This is a generated file - do not edit.
//
// Generated from k1s0/system/navigation/v1/navigation.proto.

// @dart = 3.3

// ignore_for_file: annotate_overrides, camel_case_types, comment_references
// ignore_for_file: constant_identifier_names
// ignore_for_file: curly_braces_in_flow_control_structures
// ignore_for_file: deprecated_member_use_from_same_package, library_prefixes
// ignore_for_file: non_constant_identifier_names, prefer_relative_imports

import 'dart:core' as $core;

import 'package:protobuf/protobuf.dart' as $pb;

/// GuardType はガードの種別を表す。
class GuardType extends $pb.ProtobufEnum {
  static const GuardType GUARD_TYPE_UNSPECIFIED =
      GuardType._(0, _omitEnumNames ? '' : 'GUARD_TYPE_UNSPECIFIED');
  static const GuardType GUARD_TYPE_AUTH_REQUIRED =
      GuardType._(1, _omitEnumNames ? '' : 'GUARD_TYPE_AUTH_REQUIRED');
  static const GuardType GUARD_TYPE_ROLE_REQUIRED =
      GuardType._(2, _omitEnumNames ? '' : 'GUARD_TYPE_ROLE_REQUIRED');
  static const GuardType GUARD_TYPE_REDIRECT_IF_AUTHENTICATED = GuardType._(
      3, _omitEnumNames ? '' : 'GUARD_TYPE_REDIRECT_IF_AUTHENTICATED');

  static const $core.List<GuardType> values = <GuardType>[
    GUARD_TYPE_UNSPECIFIED,
    GUARD_TYPE_AUTH_REQUIRED,
    GUARD_TYPE_ROLE_REQUIRED,
    GUARD_TYPE_REDIRECT_IF_AUTHENTICATED,
  ];

  static final $core.List<GuardType?> _byValue =
      $pb.ProtobufEnum.$_initByValueList(values, 3);
  static GuardType? valueOf($core.int value) =>
      value < 0 || value >= _byValue.length ? null : _byValue[value];

  const GuardType._(super.value, super.name);
}

/// TransitionType はページ遷移アニメーションの種別を表す。
class TransitionType extends $pb.ProtobufEnum {
  static const TransitionType TRANSITION_TYPE_UNSPECIFIED =
      TransitionType._(0, _omitEnumNames ? '' : 'TRANSITION_TYPE_UNSPECIFIED');
  static const TransitionType TRANSITION_TYPE_FADE =
      TransitionType._(1, _omitEnumNames ? '' : 'TRANSITION_TYPE_FADE');
  static const TransitionType TRANSITION_TYPE_SLIDE =
      TransitionType._(2, _omitEnumNames ? '' : 'TRANSITION_TYPE_SLIDE');
  static const TransitionType TRANSITION_TYPE_MODAL =
      TransitionType._(3, _omitEnumNames ? '' : 'TRANSITION_TYPE_MODAL');

  static const $core.List<TransitionType> values = <TransitionType>[
    TRANSITION_TYPE_UNSPECIFIED,
    TRANSITION_TYPE_FADE,
    TRANSITION_TYPE_SLIDE,
    TRANSITION_TYPE_MODAL,
  ];

  static final $core.List<TransitionType?> _byValue =
      $pb.ProtobufEnum.$_initByValueList(values, 3);
  static TransitionType? valueOf($core.int value) =>
      value < 0 || value >= _byValue.length ? null : _byValue[value];

  const TransitionType._(super.value, super.name);
}

/// ParamType はルートパラメータの型を表す。
class ParamType extends $pb.ProtobufEnum {
  static const ParamType PARAM_TYPE_UNSPECIFIED =
      ParamType._(0, _omitEnumNames ? '' : 'PARAM_TYPE_UNSPECIFIED');
  static const ParamType PARAM_TYPE_STRING =
      ParamType._(1, _omitEnumNames ? '' : 'PARAM_TYPE_STRING');
  static const ParamType PARAM_TYPE_INT =
      ParamType._(2, _omitEnumNames ? '' : 'PARAM_TYPE_INT');
  static const ParamType PARAM_TYPE_UUID =
      ParamType._(3, _omitEnumNames ? '' : 'PARAM_TYPE_UUID');

  static const $core.List<ParamType> values = <ParamType>[
    PARAM_TYPE_UNSPECIFIED,
    PARAM_TYPE_STRING,
    PARAM_TYPE_INT,
    PARAM_TYPE_UUID,
  ];

  static final $core.List<ParamType?> _byValue =
      $pb.ProtobufEnum.$_initByValueList(values, 3);
  static ParamType? valueOf($core.int value) =>
      value < 0 || value >= _byValue.length ? null : _byValue[value];

  const ParamType._(super.value, super.name);
}

const $core.bool _omitEnumNames =
    $core.bool.fromEnvironment('protobuf.omit_enum_names');
