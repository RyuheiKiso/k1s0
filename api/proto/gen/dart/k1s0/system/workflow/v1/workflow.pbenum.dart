// This is a generated file - do not edit.
//
// Generated from k1s0/system/workflow/v1/workflow.proto.

// @dart = 3.3

// ignore_for_file: annotate_overrides, camel_case_types, comment_references
// ignore_for_file: constant_identifier_names
// ignore_for_file: curly_braces_in_flow_control_structures
// ignore_for_file: deprecated_member_use_from_same_package, library_prefixes
// ignore_for_file: non_constant_identifier_names, prefer_relative_imports

import 'dart:core' as $core;

import 'package:protobuf/protobuf.dart' as $pb;

/// WorkflowStepType はワークフローステップの種別。
class WorkflowStepType extends $pb.ProtobufEnum {
  /// WORKFLOW_STEP_TYPE_UNSPECIFIED は未指定（デフォルト値）。
  static const WorkflowStepType WORKFLOW_STEP_TYPE_UNSPECIFIED =
      WorkflowStepType._(
          0, _omitEnumNames ? '' : 'WORKFLOW_STEP_TYPE_UNSPECIFIED');

  /// WORKFLOW_STEP_TYPE_APPROVAL は人間による承認タスク。
  static const WorkflowStepType WORKFLOW_STEP_TYPE_APPROVAL =
      WorkflowStepType._(
          1, _omitEnumNames ? '' : 'WORKFLOW_STEP_TYPE_APPROVAL');

  /// WORKFLOW_STEP_TYPE_AUTOMATED は自動実行タスク。
  static const WorkflowStepType WORKFLOW_STEP_TYPE_AUTOMATED =
      WorkflowStepType._(
          2, _omitEnumNames ? '' : 'WORKFLOW_STEP_TYPE_AUTOMATED');

  /// WORKFLOW_STEP_TYPE_NOTIFICATION は通知のみのタスク。
  static const WorkflowStepType WORKFLOW_STEP_TYPE_NOTIFICATION =
      WorkflowStepType._(
          3, _omitEnumNames ? '' : 'WORKFLOW_STEP_TYPE_NOTIFICATION');

  static const $core.List<WorkflowStepType> values = <WorkflowStepType>[
    WORKFLOW_STEP_TYPE_UNSPECIFIED,
    WORKFLOW_STEP_TYPE_APPROVAL,
    WORKFLOW_STEP_TYPE_AUTOMATED,
    WORKFLOW_STEP_TYPE_NOTIFICATION,
  ];

  static final $core.List<WorkflowStepType?> _byValue =
      $pb.ProtobufEnum.$_initByValueList(values, 3);
  static WorkflowStepType? valueOf($core.int value) =>
      value < 0 || value >= _byValue.length ? null : _byValue[value];

  const WorkflowStepType._(super.value, super.name);
}

const $core.bool _omitEnumNames =
    $core.bool.fromEnvironment('protobuf.omit_enum_names');
