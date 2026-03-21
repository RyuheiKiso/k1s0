// This is a generated file - do not edit.
//
// Generated from k1s0/system/saga/v1/saga.proto.

// @dart = 3.3

// ignore_for_file: annotate_overrides, camel_case_types, comment_references
// ignore_for_file: constant_identifier_names
// ignore_for_file: curly_braces_in_flow_control_structures
// ignore_for_file: deprecated_member_use_from_same_package, library_prefixes
// ignore_for_file: non_constant_identifier_names, prefer_relative_imports

import 'dart:core' as $core;

import 'package:protobuf/protobuf.dart' as $pb;

/// SagaStatus は Saga の実行ステータス。
class SagaStatus extends $pb.ProtobufEnum {
  static const SagaStatus SAGA_STATUS_UNSPECIFIED =
      SagaStatus._(0, _omitEnumNames ? '' : 'SAGA_STATUS_UNSPECIFIED');
  static const SagaStatus SAGA_STATUS_RUNNING =
      SagaStatus._(1, _omitEnumNames ? '' : 'SAGA_STATUS_RUNNING');
  static const SagaStatus SAGA_STATUS_COMPLETED =
      SagaStatus._(2, _omitEnumNames ? '' : 'SAGA_STATUS_COMPLETED');
  static const SagaStatus SAGA_STATUS_FAILED =
      SagaStatus._(3, _omitEnumNames ? '' : 'SAGA_STATUS_FAILED');
  static const SagaStatus SAGA_STATUS_COMPENSATING =
      SagaStatus._(4, _omitEnumNames ? '' : 'SAGA_STATUS_COMPENSATING');
  static const SagaStatus SAGA_STATUS_COMPENSATED =
      SagaStatus._(5, _omitEnumNames ? '' : 'SAGA_STATUS_COMPENSATED');

  static const $core.List<SagaStatus> values = <SagaStatus>[
    SAGA_STATUS_UNSPECIFIED,
    SAGA_STATUS_RUNNING,
    SAGA_STATUS_COMPLETED,
    SAGA_STATUS_FAILED,
    SAGA_STATUS_COMPENSATING,
    SAGA_STATUS_COMPENSATED,
  ];

  static final $core.List<SagaStatus?> _byValue =
      $pb.ProtobufEnum.$_initByValueList(values, 5);
  static SagaStatus? valueOf($core.int value) =>
      value < 0 || value >= _byValue.length ? null : _byValue[value];

  const SagaStatus._(super.value, super.name);
}

const $core.bool _omitEnumNames =
    $core.bool.fromEnvironment('protobuf.omit_enum_names');
