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

import '../../common/v1/types.pb.dart' as $1;
import 'workflow.pbenum.dart';

export 'package:protobuf/protobuf.dart' show GeneratedMessageGenericExtensions;

export 'workflow.pbenum.dart';

class WorkflowStep extends $pb.GeneratedMessage {
  factory WorkflowStep({
    $core.String? stepId,
    $core.String? name,
    $core.String? stepType,
    $core.String? assigneeRole,
    $core.int? timeoutHours,
    $core.String? onApprove,
    $core.String? onReject,
    WorkflowStepType? stepTypeEnum,
  }) {
    final result = create();
    if (stepId != null) result.stepId = stepId;
    if (name != null) result.name = name;
    if (stepType != null) result.stepType = stepType;
    if (assigneeRole != null) result.assigneeRole = assigneeRole;
    if (timeoutHours != null) result.timeoutHours = timeoutHours;
    if (onApprove != null) result.onApprove = onApprove;
    if (onReject != null) result.onReject = onReject;
    if (stepTypeEnum != null) result.stepTypeEnum = stepTypeEnum;
    return result;
  }

  WorkflowStep._();

  factory WorkflowStep.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory WorkflowStep.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'WorkflowStep',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.workflow.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'stepId')
    ..aOS(2, _omitFieldNames ? '' : 'name')
    ..aOS(3, _omitFieldNames ? '' : 'stepType')
    ..aOS(4, _omitFieldNames ? '' : 'assigneeRole')
    ..aI(5, _omitFieldNames ? '' : 'timeoutHours',
        fieldType: $pb.PbFieldType.OU3)
    ..aOS(6, _omitFieldNames ? '' : 'onApprove')
    ..aOS(7, _omitFieldNames ? '' : 'onReject')
    ..aE<WorkflowStepType>(8, _omitFieldNames ? '' : 'stepTypeEnum',
        enumValues: WorkflowStepType.values)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  WorkflowStep clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  WorkflowStep copyWith(void Function(WorkflowStep) updates) =>
      super.copyWith((message) => updates(message as WorkflowStep))
          as WorkflowStep;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static WorkflowStep create() => WorkflowStep._();
  @$core.override
  WorkflowStep createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static WorkflowStep getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<WorkflowStep>(create);
  static WorkflowStep? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get stepId => $_getSZ(0);
  @$pb.TagNumber(1)
  set stepId($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasStepId() => $_has(0);
  @$pb.TagNumber(1)
  void clearStepId() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get name => $_getSZ(1);
  @$pb.TagNumber(2)
  set name($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasName() => $_has(1);
  @$pb.TagNumber(2)
  void clearName() => $_clearField(2);

  /// Deprecated: use step_type_enum instead.
  @$pb.TagNumber(3)
  $core.String get stepType => $_getSZ(2);
  @$pb.TagNumber(3)
  set stepType($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasStepType() => $_has(2);
  @$pb.TagNumber(3)
  void clearStepType() => $_clearField(3);

  @$pb.TagNumber(4)
  $core.String get assigneeRole => $_getSZ(3);
  @$pb.TagNumber(4)
  set assigneeRole($core.String value) => $_setString(3, value);
  @$pb.TagNumber(4)
  $core.bool hasAssigneeRole() => $_has(3);
  @$pb.TagNumber(4)
  void clearAssigneeRole() => $_clearField(4);

  @$pb.TagNumber(5)
  $core.int get timeoutHours => $_getIZ(4);
  @$pb.TagNumber(5)
  set timeoutHours($core.int value) => $_setUnsignedInt32(4, value);
  @$pb.TagNumber(5)
  $core.bool hasTimeoutHours() => $_has(4);
  @$pb.TagNumber(5)
  void clearTimeoutHours() => $_clearField(5);

  @$pb.TagNumber(6)
  $core.String get onApprove => $_getSZ(5);
  @$pb.TagNumber(6)
  set onApprove($core.String value) => $_setString(5, value);
  @$pb.TagNumber(6)
  $core.bool hasOnApprove() => $_has(5);
  @$pb.TagNumber(6)
  void clearOnApprove() => $_clearField(6);

  @$pb.TagNumber(7)
  $core.String get onReject => $_getSZ(6);
  @$pb.TagNumber(7)
  set onReject($core.String value) => $_setString(6, value);
  @$pb.TagNumber(7)
  $core.bool hasOnReject() => $_has(6);
  @$pb.TagNumber(7)
  void clearOnReject() => $_clearField(7);

  /// ステップ種別の enum 版（step_type の型付き版）。
  @$pb.TagNumber(8)
  WorkflowStepType get stepTypeEnum => $_getN(7);
  @$pb.TagNumber(8)
  set stepTypeEnum(WorkflowStepType value) => $_setField(8, value);
  @$pb.TagNumber(8)
  $core.bool hasStepTypeEnum() => $_has(7);
  @$pb.TagNumber(8)
  void clearStepTypeEnum() => $_clearField(8);
}

class WorkflowDefinition extends $pb.GeneratedMessage {
  factory WorkflowDefinition({
    $core.String? id,
    $core.String? name,
    $core.String? description,
    $core.int? version,
    $core.bool? enabled,
    $core.Iterable<WorkflowStep>? steps,
    $1.Timestamp? createdAt,
    $1.Timestamp? updatedAt,
  }) {
    final result = create();
    if (id != null) result.id = id;
    if (name != null) result.name = name;
    if (description != null) result.description = description;
    if (version != null) result.version = version;
    if (enabled != null) result.enabled = enabled;
    if (steps != null) result.steps.addAll(steps);
    if (createdAt != null) result.createdAt = createdAt;
    if (updatedAt != null) result.updatedAt = updatedAt;
    return result;
  }

  WorkflowDefinition._();

  factory WorkflowDefinition.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory WorkflowDefinition.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'WorkflowDefinition',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.workflow.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'id')
    ..aOS(2, _omitFieldNames ? '' : 'name')
    ..aOS(3, _omitFieldNames ? '' : 'description')
    ..aI(4, _omitFieldNames ? '' : 'version', fieldType: $pb.PbFieldType.OU3)
    ..aOB(5, _omitFieldNames ? '' : 'enabled')
    ..pPM<WorkflowStep>(6, _omitFieldNames ? '' : 'steps',
        subBuilder: WorkflowStep.create)
    ..aOM<$1.Timestamp>(7, _omitFieldNames ? '' : 'createdAt',
        subBuilder: $1.Timestamp.create)
    ..aOM<$1.Timestamp>(8, _omitFieldNames ? '' : 'updatedAt',
        subBuilder: $1.Timestamp.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  WorkflowDefinition clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  WorkflowDefinition copyWith(void Function(WorkflowDefinition) updates) =>
      super.copyWith((message) => updates(message as WorkflowDefinition))
          as WorkflowDefinition;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static WorkflowDefinition create() => WorkflowDefinition._();
  @$core.override
  WorkflowDefinition createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static WorkflowDefinition getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<WorkflowDefinition>(create);
  static WorkflowDefinition? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get id => $_getSZ(0);
  @$pb.TagNumber(1)
  set id($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasId() => $_has(0);
  @$pb.TagNumber(1)
  void clearId() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get name => $_getSZ(1);
  @$pb.TagNumber(2)
  set name($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasName() => $_has(1);
  @$pb.TagNumber(2)
  void clearName() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.String get description => $_getSZ(2);
  @$pb.TagNumber(3)
  set description($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasDescription() => $_has(2);
  @$pb.TagNumber(3)
  void clearDescription() => $_clearField(3);

  @$pb.TagNumber(4)
  $core.int get version => $_getIZ(3);
  @$pb.TagNumber(4)
  set version($core.int value) => $_setUnsignedInt32(3, value);
  @$pb.TagNumber(4)
  $core.bool hasVersion() => $_has(3);
  @$pb.TagNumber(4)
  void clearVersion() => $_clearField(4);

  @$pb.TagNumber(5)
  $core.bool get enabled => $_getBF(4);
  @$pb.TagNumber(5)
  set enabled($core.bool value) => $_setBool(4, value);
  @$pb.TagNumber(5)
  $core.bool hasEnabled() => $_has(4);
  @$pb.TagNumber(5)
  void clearEnabled() => $_clearField(5);

  @$pb.TagNumber(6)
  $pb.PbList<WorkflowStep> get steps => $_getList(5);

  @$pb.TagNumber(7)
  $1.Timestamp get createdAt => $_getN(6);
  @$pb.TagNumber(7)
  set createdAt($1.Timestamp value) => $_setField(7, value);
  @$pb.TagNumber(7)
  $core.bool hasCreatedAt() => $_has(6);
  @$pb.TagNumber(7)
  void clearCreatedAt() => $_clearField(7);
  @$pb.TagNumber(7)
  $1.Timestamp ensureCreatedAt() => $_ensure(6);

  @$pb.TagNumber(8)
  $1.Timestamp get updatedAt => $_getN(7);
  @$pb.TagNumber(8)
  set updatedAt($1.Timestamp value) => $_setField(8, value);
  @$pb.TagNumber(8)
  $core.bool hasUpdatedAt() => $_has(7);
  @$pb.TagNumber(8)
  void clearUpdatedAt() => $_clearField(8);
  @$pb.TagNumber(8)
  $1.Timestamp ensureUpdatedAt() => $_ensure(7);
}

class ListWorkflowsRequest extends $pb.GeneratedMessage {
  factory ListWorkflowsRequest({
    $core.bool? enabledOnly,
    $1.Pagination? pagination,
  }) {
    final result = create();
    if (enabledOnly != null) result.enabledOnly = enabledOnly;
    if (pagination != null) result.pagination = pagination;
    return result;
  }

  ListWorkflowsRequest._();

  factory ListWorkflowsRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory ListWorkflowsRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ListWorkflowsRequest',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.workflow.v1'),
      createEmptyInstance: create)
    ..aOB(1, _omitFieldNames ? '' : 'enabledOnly')
    ..aOM<$1.Pagination>(2, _omitFieldNames ? '' : 'pagination',
        subBuilder: $1.Pagination.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListWorkflowsRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListWorkflowsRequest copyWith(void Function(ListWorkflowsRequest) updates) =>
      super.copyWith((message) => updates(message as ListWorkflowsRequest))
          as ListWorkflowsRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static ListWorkflowsRequest create() => ListWorkflowsRequest._();
  @$core.override
  ListWorkflowsRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static ListWorkflowsRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ListWorkflowsRequest>(create);
  static ListWorkflowsRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.bool get enabledOnly => $_getBF(0);
  @$pb.TagNumber(1)
  set enabledOnly($core.bool value) => $_setBool(0, value);
  @$pb.TagNumber(1)
  $core.bool hasEnabledOnly() => $_has(0);
  @$pb.TagNumber(1)
  void clearEnabledOnly() => $_clearField(1);

  @$pb.TagNumber(2)
  $1.Pagination get pagination => $_getN(1);
  @$pb.TagNumber(2)
  set pagination($1.Pagination value) => $_setField(2, value);
  @$pb.TagNumber(2)
  $core.bool hasPagination() => $_has(1);
  @$pb.TagNumber(2)
  void clearPagination() => $_clearField(2);
  @$pb.TagNumber(2)
  $1.Pagination ensurePagination() => $_ensure(1);
}

class ListWorkflowsResponse extends $pb.GeneratedMessage {
  factory ListWorkflowsResponse({
    $core.Iterable<WorkflowDefinition>? workflows,
    $1.PaginationResult? pagination,
  }) {
    final result = create();
    if (workflows != null) result.workflows.addAll(workflows);
    if (pagination != null) result.pagination = pagination;
    return result;
  }

  ListWorkflowsResponse._();

  factory ListWorkflowsResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory ListWorkflowsResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ListWorkflowsResponse',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.workflow.v1'),
      createEmptyInstance: create)
    ..pPM<WorkflowDefinition>(1, _omitFieldNames ? '' : 'workflows',
        subBuilder: WorkflowDefinition.create)
    ..aOM<$1.PaginationResult>(2, _omitFieldNames ? '' : 'pagination',
        subBuilder: $1.PaginationResult.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListWorkflowsResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListWorkflowsResponse copyWith(
          void Function(ListWorkflowsResponse) updates) =>
      super.copyWith((message) => updates(message as ListWorkflowsResponse))
          as ListWorkflowsResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static ListWorkflowsResponse create() => ListWorkflowsResponse._();
  @$core.override
  ListWorkflowsResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static ListWorkflowsResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ListWorkflowsResponse>(create);
  static ListWorkflowsResponse? _defaultInstance;

  @$pb.TagNumber(1)
  $pb.PbList<WorkflowDefinition> get workflows => $_getList(0);

  @$pb.TagNumber(2)
  $1.PaginationResult get pagination => $_getN(1);
  @$pb.TagNumber(2)
  set pagination($1.PaginationResult value) => $_setField(2, value);
  @$pb.TagNumber(2)
  $core.bool hasPagination() => $_has(1);
  @$pb.TagNumber(2)
  void clearPagination() => $_clearField(2);
  @$pb.TagNumber(2)
  $1.PaginationResult ensurePagination() => $_ensure(1);
}

class CreateWorkflowRequest extends $pb.GeneratedMessage {
  factory CreateWorkflowRequest({
    $core.String? name,
    $core.String? description,
    $core.bool? enabled,
    $core.Iterable<WorkflowStep>? steps,
  }) {
    final result = create();
    if (name != null) result.name = name;
    if (description != null) result.description = description;
    if (enabled != null) result.enabled = enabled;
    if (steps != null) result.steps.addAll(steps);
    return result;
  }

  CreateWorkflowRequest._();

  factory CreateWorkflowRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory CreateWorkflowRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'CreateWorkflowRequest',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.workflow.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'name')
    ..aOS(2, _omitFieldNames ? '' : 'description')
    ..aOB(3, _omitFieldNames ? '' : 'enabled')
    ..pPM<WorkflowStep>(4, _omitFieldNames ? '' : 'steps',
        subBuilder: WorkflowStep.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  CreateWorkflowRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  CreateWorkflowRequest copyWith(
          void Function(CreateWorkflowRequest) updates) =>
      super.copyWith((message) => updates(message as CreateWorkflowRequest))
          as CreateWorkflowRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static CreateWorkflowRequest create() => CreateWorkflowRequest._();
  @$core.override
  CreateWorkflowRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static CreateWorkflowRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<CreateWorkflowRequest>(create);
  static CreateWorkflowRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get name => $_getSZ(0);
  @$pb.TagNumber(1)
  set name($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasName() => $_has(0);
  @$pb.TagNumber(1)
  void clearName() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get description => $_getSZ(1);
  @$pb.TagNumber(2)
  set description($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasDescription() => $_has(1);
  @$pb.TagNumber(2)
  void clearDescription() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.bool get enabled => $_getBF(2);
  @$pb.TagNumber(3)
  set enabled($core.bool value) => $_setBool(2, value);
  @$pb.TagNumber(3)
  $core.bool hasEnabled() => $_has(2);
  @$pb.TagNumber(3)
  void clearEnabled() => $_clearField(3);

  @$pb.TagNumber(4)
  $pb.PbList<WorkflowStep> get steps => $_getList(3);
}

class CreateWorkflowResponse extends $pb.GeneratedMessage {
  factory CreateWorkflowResponse({
    WorkflowDefinition? workflow,
  }) {
    final result = create();
    if (workflow != null) result.workflow = workflow;
    return result;
  }

  CreateWorkflowResponse._();

  factory CreateWorkflowResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory CreateWorkflowResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'CreateWorkflowResponse',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.workflow.v1'),
      createEmptyInstance: create)
    ..aOM<WorkflowDefinition>(1, _omitFieldNames ? '' : 'workflow',
        subBuilder: WorkflowDefinition.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  CreateWorkflowResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  CreateWorkflowResponse copyWith(
          void Function(CreateWorkflowResponse) updates) =>
      super.copyWith((message) => updates(message as CreateWorkflowResponse))
          as CreateWorkflowResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static CreateWorkflowResponse create() => CreateWorkflowResponse._();
  @$core.override
  CreateWorkflowResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static CreateWorkflowResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<CreateWorkflowResponse>(create);
  static CreateWorkflowResponse? _defaultInstance;

  @$pb.TagNumber(1)
  WorkflowDefinition get workflow => $_getN(0);
  @$pb.TagNumber(1)
  set workflow(WorkflowDefinition value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasWorkflow() => $_has(0);
  @$pb.TagNumber(1)
  void clearWorkflow() => $_clearField(1);
  @$pb.TagNumber(1)
  WorkflowDefinition ensureWorkflow() => $_ensure(0);
}

class GetWorkflowRequest extends $pb.GeneratedMessage {
  factory GetWorkflowRequest({
    $core.String? workflowId,
  }) {
    final result = create();
    if (workflowId != null) result.workflowId = workflowId;
    return result;
  }

  GetWorkflowRequest._();

  factory GetWorkflowRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory GetWorkflowRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'GetWorkflowRequest',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.workflow.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'workflowId')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetWorkflowRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetWorkflowRequest copyWith(void Function(GetWorkflowRequest) updates) =>
      super.copyWith((message) => updates(message as GetWorkflowRequest))
          as GetWorkflowRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static GetWorkflowRequest create() => GetWorkflowRequest._();
  @$core.override
  GetWorkflowRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static GetWorkflowRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<GetWorkflowRequest>(create);
  static GetWorkflowRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get workflowId => $_getSZ(0);
  @$pb.TagNumber(1)
  set workflowId($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasWorkflowId() => $_has(0);
  @$pb.TagNumber(1)
  void clearWorkflowId() => $_clearField(1);
}

class GetWorkflowResponse extends $pb.GeneratedMessage {
  factory GetWorkflowResponse({
    WorkflowDefinition? workflow,
  }) {
    final result = create();
    if (workflow != null) result.workflow = workflow;
    return result;
  }

  GetWorkflowResponse._();

  factory GetWorkflowResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory GetWorkflowResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'GetWorkflowResponse',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.workflow.v1'),
      createEmptyInstance: create)
    ..aOM<WorkflowDefinition>(1, _omitFieldNames ? '' : 'workflow',
        subBuilder: WorkflowDefinition.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetWorkflowResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetWorkflowResponse copyWith(void Function(GetWorkflowResponse) updates) =>
      super.copyWith((message) => updates(message as GetWorkflowResponse))
          as GetWorkflowResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static GetWorkflowResponse create() => GetWorkflowResponse._();
  @$core.override
  GetWorkflowResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static GetWorkflowResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<GetWorkflowResponse>(create);
  static GetWorkflowResponse? _defaultInstance;

  @$pb.TagNumber(1)
  WorkflowDefinition get workflow => $_getN(0);
  @$pb.TagNumber(1)
  set workflow(WorkflowDefinition value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasWorkflow() => $_has(0);
  @$pb.TagNumber(1)
  void clearWorkflow() => $_clearField(1);
  @$pb.TagNumber(1)
  WorkflowDefinition ensureWorkflow() => $_ensure(0);
}

class WorkflowSteps extends $pb.GeneratedMessage {
  factory WorkflowSteps({
    $core.Iterable<WorkflowStep>? items,
  }) {
    final result = create();
    if (items != null) result.items.addAll(items);
    return result;
  }

  WorkflowSteps._();

  factory WorkflowSteps.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory WorkflowSteps.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'WorkflowSteps',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.workflow.v1'),
      createEmptyInstance: create)
    ..pPM<WorkflowStep>(1, _omitFieldNames ? '' : 'items',
        subBuilder: WorkflowStep.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  WorkflowSteps clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  WorkflowSteps copyWith(void Function(WorkflowSteps) updates) =>
      super.copyWith((message) => updates(message as WorkflowSteps))
          as WorkflowSteps;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static WorkflowSteps create() => WorkflowSteps._();
  @$core.override
  WorkflowSteps createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static WorkflowSteps getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<WorkflowSteps>(create);
  static WorkflowSteps? _defaultInstance;

  @$pb.TagNumber(1)
  $pb.PbList<WorkflowStep> get items => $_getList(0);
}

class UpdateWorkflowRequest extends $pb.GeneratedMessage {
  factory UpdateWorkflowRequest({
    $core.String? workflowId,
    $core.String? name,
    $core.String? description,
    $core.bool? enabled,
    WorkflowSteps? steps,
  }) {
    final result = create();
    if (workflowId != null) result.workflowId = workflowId;
    if (name != null) result.name = name;
    if (description != null) result.description = description;
    if (enabled != null) result.enabled = enabled;
    if (steps != null) result.steps = steps;
    return result;
  }

  UpdateWorkflowRequest._();

  factory UpdateWorkflowRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory UpdateWorkflowRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'UpdateWorkflowRequest',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.workflow.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'workflowId')
    ..aOS(2, _omitFieldNames ? '' : 'name')
    ..aOS(3, _omitFieldNames ? '' : 'description')
    ..aOB(4, _omitFieldNames ? '' : 'enabled')
    ..aOM<WorkflowSteps>(5, _omitFieldNames ? '' : 'steps',
        subBuilder: WorkflowSteps.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  UpdateWorkflowRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  UpdateWorkflowRequest copyWith(
          void Function(UpdateWorkflowRequest) updates) =>
      super.copyWith((message) => updates(message as UpdateWorkflowRequest))
          as UpdateWorkflowRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static UpdateWorkflowRequest create() => UpdateWorkflowRequest._();
  @$core.override
  UpdateWorkflowRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static UpdateWorkflowRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<UpdateWorkflowRequest>(create);
  static UpdateWorkflowRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get workflowId => $_getSZ(0);
  @$pb.TagNumber(1)
  set workflowId($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasWorkflowId() => $_has(0);
  @$pb.TagNumber(1)
  void clearWorkflowId() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get name => $_getSZ(1);
  @$pb.TagNumber(2)
  set name($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasName() => $_has(1);
  @$pb.TagNumber(2)
  void clearName() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.String get description => $_getSZ(2);
  @$pb.TagNumber(3)
  set description($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasDescription() => $_has(2);
  @$pb.TagNumber(3)
  void clearDescription() => $_clearField(3);

  @$pb.TagNumber(4)
  $core.bool get enabled => $_getBF(3);
  @$pb.TagNumber(4)
  set enabled($core.bool value) => $_setBool(3, value);
  @$pb.TagNumber(4)
  $core.bool hasEnabled() => $_has(3);
  @$pb.TagNumber(4)
  void clearEnabled() => $_clearField(4);

  @$pb.TagNumber(5)
  WorkflowSteps get steps => $_getN(4);
  @$pb.TagNumber(5)
  set steps(WorkflowSteps value) => $_setField(5, value);
  @$pb.TagNumber(5)
  $core.bool hasSteps() => $_has(4);
  @$pb.TagNumber(5)
  void clearSteps() => $_clearField(5);
  @$pb.TagNumber(5)
  WorkflowSteps ensureSteps() => $_ensure(4);
}

class UpdateWorkflowResponse extends $pb.GeneratedMessage {
  factory UpdateWorkflowResponse({
    WorkflowDefinition? workflow,
  }) {
    final result = create();
    if (workflow != null) result.workflow = workflow;
    return result;
  }

  UpdateWorkflowResponse._();

  factory UpdateWorkflowResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory UpdateWorkflowResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'UpdateWorkflowResponse',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.workflow.v1'),
      createEmptyInstance: create)
    ..aOM<WorkflowDefinition>(1, _omitFieldNames ? '' : 'workflow',
        subBuilder: WorkflowDefinition.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  UpdateWorkflowResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  UpdateWorkflowResponse copyWith(
          void Function(UpdateWorkflowResponse) updates) =>
      super.copyWith((message) => updates(message as UpdateWorkflowResponse))
          as UpdateWorkflowResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static UpdateWorkflowResponse create() => UpdateWorkflowResponse._();
  @$core.override
  UpdateWorkflowResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static UpdateWorkflowResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<UpdateWorkflowResponse>(create);
  static UpdateWorkflowResponse? _defaultInstance;

  @$pb.TagNumber(1)
  WorkflowDefinition get workflow => $_getN(0);
  @$pb.TagNumber(1)
  set workflow(WorkflowDefinition value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasWorkflow() => $_has(0);
  @$pb.TagNumber(1)
  void clearWorkflow() => $_clearField(1);
  @$pb.TagNumber(1)
  WorkflowDefinition ensureWorkflow() => $_ensure(0);
}

class DeleteWorkflowRequest extends $pb.GeneratedMessage {
  factory DeleteWorkflowRequest({
    $core.String? workflowId,
  }) {
    final result = create();
    if (workflowId != null) result.workflowId = workflowId;
    return result;
  }

  DeleteWorkflowRequest._();

  factory DeleteWorkflowRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory DeleteWorkflowRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'DeleteWorkflowRequest',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.workflow.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'workflowId')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  DeleteWorkflowRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  DeleteWorkflowRequest copyWith(
          void Function(DeleteWorkflowRequest) updates) =>
      super.copyWith((message) => updates(message as DeleteWorkflowRequest))
          as DeleteWorkflowRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static DeleteWorkflowRequest create() => DeleteWorkflowRequest._();
  @$core.override
  DeleteWorkflowRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static DeleteWorkflowRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<DeleteWorkflowRequest>(create);
  static DeleteWorkflowRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get workflowId => $_getSZ(0);
  @$pb.TagNumber(1)
  set workflowId($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasWorkflowId() => $_has(0);
  @$pb.TagNumber(1)
  void clearWorkflowId() => $_clearField(1);
}

class DeleteWorkflowResponse extends $pb.GeneratedMessage {
  factory DeleteWorkflowResponse({
    $core.bool? success,
    $core.String? message,
  }) {
    final result = create();
    if (success != null) result.success = success;
    if (message != null) result.message = message;
    return result;
  }

  DeleteWorkflowResponse._();

  factory DeleteWorkflowResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory DeleteWorkflowResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'DeleteWorkflowResponse',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.workflow.v1'),
      createEmptyInstance: create)
    ..aOB(1, _omitFieldNames ? '' : 'success')
    ..aOS(2, _omitFieldNames ? '' : 'message')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  DeleteWorkflowResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  DeleteWorkflowResponse copyWith(
          void Function(DeleteWorkflowResponse) updates) =>
      super.copyWith((message) => updates(message as DeleteWorkflowResponse))
          as DeleteWorkflowResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static DeleteWorkflowResponse create() => DeleteWorkflowResponse._();
  @$core.override
  DeleteWorkflowResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static DeleteWorkflowResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<DeleteWorkflowResponse>(create);
  static DeleteWorkflowResponse? _defaultInstance;

  @$pb.TagNumber(1)
  $core.bool get success => $_getBF(0);
  @$pb.TagNumber(1)
  set success($core.bool value) => $_setBool(0, value);
  @$pb.TagNumber(1)
  $core.bool hasSuccess() => $_has(0);
  @$pb.TagNumber(1)
  void clearSuccess() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get message => $_getSZ(1);
  @$pb.TagNumber(2)
  set message($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasMessage() => $_has(1);
  @$pb.TagNumber(2)
  void clearMessage() => $_clearField(2);
}

class StartInstanceRequest extends $pb.GeneratedMessage {
  factory StartInstanceRequest({
    $core.String? workflowId,
    $core.String? title,
    $core.String? initiatorId,
    $core.List<$core.int>? contextJson,
  }) {
    final result = create();
    if (workflowId != null) result.workflowId = workflowId;
    if (title != null) result.title = title;
    if (initiatorId != null) result.initiatorId = initiatorId;
    if (contextJson != null) result.contextJson = contextJson;
    return result;
  }

  StartInstanceRequest._();

  factory StartInstanceRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory StartInstanceRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'StartInstanceRequest',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.workflow.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'workflowId')
    ..aOS(2, _omitFieldNames ? '' : 'title')
    ..aOS(3, _omitFieldNames ? '' : 'initiatorId')
    ..a<$core.List<$core.int>>(
        4, _omitFieldNames ? '' : 'contextJson', $pb.PbFieldType.OY)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  StartInstanceRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  StartInstanceRequest copyWith(void Function(StartInstanceRequest) updates) =>
      super.copyWith((message) => updates(message as StartInstanceRequest))
          as StartInstanceRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static StartInstanceRequest create() => StartInstanceRequest._();
  @$core.override
  StartInstanceRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static StartInstanceRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<StartInstanceRequest>(create);
  static StartInstanceRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get workflowId => $_getSZ(0);
  @$pb.TagNumber(1)
  set workflowId($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasWorkflowId() => $_has(0);
  @$pb.TagNumber(1)
  void clearWorkflowId() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get title => $_getSZ(1);
  @$pb.TagNumber(2)
  set title($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasTitle() => $_has(1);
  @$pb.TagNumber(2)
  void clearTitle() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.String get initiatorId => $_getSZ(2);
  @$pb.TagNumber(3)
  set initiatorId($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasInitiatorId() => $_has(2);
  @$pb.TagNumber(3)
  void clearInitiatorId() => $_clearField(3);

  /// ワークフロー実行コンテキスト（JSON バイト列）
  @$pb.TagNumber(4)
  $core.List<$core.int> get contextJson => $_getN(3);
  @$pb.TagNumber(4)
  set contextJson($core.List<$core.int> value) => $_setBytes(3, value);
  @$pb.TagNumber(4)
  $core.bool hasContextJson() => $_has(3);
  @$pb.TagNumber(4)
  void clearContextJson() => $_clearField(4);
}

class StartInstanceResponse extends $pb.GeneratedMessage {
  factory StartInstanceResponse({
    $core.String? instanceId,
    $core.String? status,
    $core.String? currentStepId,
    $1.Timestamp? startedAt,
    $core.String? workflowId,
    $core.String? workflowName,
    $core.String? title,
    $core.String? initiatorId,
    $core.List<$core.int>? contextJson,
  }) {
    final result = create();
    if (instanceId != null) result.instanceId = instanceId;
    if (status != null) result.status = status;
    if (currentStepId != null) result.currentStepId = currentStepId;
    if (startedAt != null) result.startedAt = startedAt;
    if (workflowId != null) result.workflowId = workflowId;
    if (workflowName != null) result.workflowName = workflowName;
    if (title != null) result.title = title;
    if (initiatorId != null) result.initiatorId = initiatorId;
    if (contextJson != null) result.contextJson = contextJson;
    return result;
  }

  StartInstanceResponse._();

  factory StartInstanceResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory StartInstanceResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'StartInstanceResponse',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.workflow.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'instanceId')
    ..aOS(2, _omitFieldNames ? '' : 'status')
    ..aOS(3, _omitFieldNames ? '' : 'currentStepId')
    ..aOM<$1.Timestamp>(4, _omitFieldNames ? '' : 'startedAt',
        subBuilder: $1.Timestamp.create)
    ..aOS(5, _omitFieldNames ? '' : 'workflowId')
    ..aOS(6, _omitFieldNames ? '' : 'workflowName')
    ..aOS(7, _omitFieldNames ? '' : 'title')
    ..aOS(8, _omitFieldNames ? '' : 'initiatorId')
    ..a<$core.List<$core.int>>(
        9, _omitFieldNames ? '' : 'contextJson', $pb.PbFieldType.OY)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  StartInstanceResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  StartInstanceResponse copyWith(
          void Function(StartInstanceResponse) updates) =>
      super.copyWith((message) => updates(message as StartInstanceResponse))
          as StartInstanceResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static StartInstanceResponse create() => StartInstanceResponse._();
  @$core.override
  StartInstanceResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static StartInstanceResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<StartInstanceResponse>(create);
  static StartInstanceResponse? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get instanceId => $_getSZ(0);
  @$pb.TagNumber(1)
  set instanceId($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasInstanceId() => $_has(0);
  @$pb.TagNumber(1)
  void clearInstanceId() => $_clearField(1);

  /// インスタンス状態（pending / running / completed / cancelled / failed）
  @$pb.TagNumber(2)
  $core.String get status => $_getSZ(1);
  @$pb.TagNumber(2)
  set status($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasStatus() => $_has(1);
  @$pb.TagNumber(2)
  void clearStatus() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.String get currentStepId => $_getSZ(2);
  @$pb.TagNumber(3)
  set currentStepId($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasCurrentStepId() => $_has(2);
  @$pb.TagNumber(3)
  void clearCurrentStepId() => $_clearField(3);

  @$pb.TagNumber(4)
  $1.Timestamp get startedAt => $_getN(3);
  @$pb.TagNumber(4)
  set startedAt($1.Timestamp value) => $_setField(4, value);
  @$pb.TagNumber(4)
  $core.bool hasStartedAt() => $_has(3);
  @$pb.TagNumber(4)
  void clearStartedAt() => $_clearField(4);
  @$pb.TagNumber(4)
  $1.Timestamp ensureStartedAt() => $_ensure(3);

  @$pb.TagNumber(5)
  $core.String get workflowId => $_getSZ(4);
  @$pb.TagNumber(5)
  set workflowId($core.String value) => $_setString(4, value);
  @$pb.TagNumber(5)
  $core.bool hasWorkflowId() => $_has(4);
  @$pb.TagNumber(5)
  void clearWorkflowId() => $_clearField(5);

  @$pb.TagNumber(6)
  $core.String get workflowName => $_getSZ(5);
  @$pb.TagNumber(6)
  set workflowName($core.String value) => $_setString(5, value);
  @$pb.TagNumber(6)
  $core.bool hasWorkflowName() => $_has(5);
  @$pb.TagNumber(6)
  void clearWorkflowName() => $_clearField(6);

  @$pb.TagNumber(7)
  $core.String get title => $_getSZ(6);
  @$pb.TagNumber(7)
  set title($core.String value) => $_setString(6, value);
  @$pb.TagNumber(7)
  $core.bool hasTitle() => $_has(6);
  @$pb.TagNumber(7)
  void clearTitle() => $_clearField(7);

  @$pb.TagNumber(8)
  $core.String get initiatorId => $_getSZ(7);
  @$pb.TagNumber(8)
  set initiatorId($core.String value) => $_setString(7, value);
  @$pb.TagNumber(8)
  $core.bool hasInitiatorId() => $_has(7);
  @$pb.TagNumber(8)
  void clearInitiatorId() => $_clearField(8);

  @$pb.TagNumber(9)
  $core.List<$core.int> get contextJson => $_getN(8);
  @$pb.TagNumber(9)
  set contextJson($core.List<$core.int> value) => $_setBytes(8, value);
  @$pb.TagNumber(9)
  $core.bool hasContextJson() => $_has(8);
  @$pb.TagNumber(9)
  void clearContextJson() => $_clearField(9);
}

class GetInstanceRequest extends $pb.GeneratedMessage {
  factory GetInstanceRequest({
    $core.String? instanceId,
  }) {
    final result = create();
    if (instanceId != null) result.instanceId = instanceId;
    return result;
  }

  GetInstanceRequest._();

  factory GetInstanceRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory GetInstanceRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'GetInstanceRequest',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.workflow.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'instanceId')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetInstanceRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetInstanceRequest copyWith(void Function(GetInstanceRequest) updates) =>
      super.copyWith((message) => updates(message as GetInstanceRequest))
          as GetInstanceRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static GetInstanceRequest create() => GetInstanceRequest._();
  @$core.override
  GetInstanceRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static GetInstanceRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<GetInstanceRequest>(create);
  static GetInstanceRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get instanceId => $_getSZ(0);
  @$pb.TagNumber(1)
  set instanceId($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasInstanceId() => $_has(0);
  @$pb.TagNumber(1)
  void clearInstanceId() => $_clearField(1);
}

class GetInstanceResponse extends $pb.GeneratedMessage {
  factory GetInstanceResponse({
    WorkflowInstance? instance,
  }) {
    final result = create();
    if (instance != null) result.instance = instance;
    return result;
  }

  GetInstanceResponse._();

  factory GetInstanceResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory GetInstanceResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'GetInstanceResponse',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.workflow.v1'),
      createEmptyInstance: create)
    ..aOM<WorkflowInstance>(1, _omitFieldNames ? '' : 'instance',
        subBuilder: WorkflowInstance.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetInstanceResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetInstanceResponse copyWith(void Function(GetInstanceResponse) updates) =>
      super.copyWith((message) => updates(message as GetInstanceResponse))
          as GetInstanceResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static GetInstanceResponse create() => GetInstanceResponse._();
  @$core.override
  GetInstanceResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static GetInstanceResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<GetInstanceResponse>(create);
  static GetInstanceResponse? _defaultInstance;

  @$pb.TagNumber(1)
  WorkflowInstance get instance => $_getN(0);
  @$pb.TagNumber(1)
  set instance(WorkflowInstance value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasInstance() => $_has(0);
  @$pb.TagNumber(1)
  void clearInstance() => $_clearField(1);
  @$pb.TagNumber(1)
  WorkflowInstance ensureInstance() => $_ensure(0);
}

class ListInstancesRequest extends $pb.GeneratedMessage {
  factory ListInstancesRequest({
    $core.String? status,
    $core.String? workflowId,
    $core.String? initiatorId,
    $1.Pagination? pagination,
  }) {
    final result = create();
    if (status != null) result.status = status;
    if (workflowId != null) result.workflowId = workflowId;
    if (initiatorId != null) result.initiatorId = initiatorId;
    if (pagination != null) result.pagination = pagination;
    return result;
  }

  ListInstancesRequest._();

  factory ListInstancesRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory ListInstancesRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ListInstancesRequest',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.workflow.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'status')
    ..aOS(2, _omitFieldNames ? '' : 'workflowId')
    ..aOS(3, _omitFieldNames ? '' : 'initiatorId')
    ..aOM<$1.Pagination>(4, _omitFieldNames ? '' : 'pagination',
        subBuilder: $1.Pagination.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListInstancesRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListInstancesRequest copyWith(void Function(ListInstancesRequest) updates) =>
      super.copyWith((message) => updates(message as ListInstancesRequest))
          as ListInstancesRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static ListInstancesRequest create() => ListInstancesRequest._();
  @$core.override
  ListInstancesRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static ListInstancesRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ListInstancesRequest>(create);
  static ListInstancesRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get status => $_getSZ(0);
  @$pb.TagNumber(1)
  set status($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasStatus() => $_has(0);
  @$pb.TagNumber(1)
  void clearStatus() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get workflowId => $_getSZ(1);
  @$pb.TagNumber(2)
  set workflowId($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasWorkflowId() => $_has(1);
  @$pb.TagNumber(2)
  void clearWorkflowId() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.String get initiatorId => $_getSZ(2);
  @$pb.TagNumber(3)
  set initiatorId($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasInitiatorId() => $_has(2);
  @$pb.TagNumber(3)
  void clearInitiatorId() => $_clearField(3);

  @$pb.TagNumber(4)
  $1.Pagination get pagination => $_getN(3);
  @$pb.TagNumber(4)
  set pagination($1.Pagination value) => $_setField(4, value);
  @$pb.TagNumber(4)
  $core.bool hasPagination() => $_has(3);
  @$pb.TagNumber(4)
  void clearPagination() => $_clearField(4);
  @$pb.TagNumber(4)
  $1.Pagination ensurePagination() => $_ensure(3);
}

class ListInstancesResponse extends $pb.GeneratedMessage {
  factory ListInstancesResponse({
    $core.Iterable<WorkflowInstance>? instances,
    $1.PaginationResult? pagination,
  }) {
    final result = create();
    if (instances != null) result.instances.addAll(instances);
    if (pagination != null) result.pagination = pagination;
    return result;
  }

  ListInstancesResponse._();

  factory ListInstancesResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory ListInstancesResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ListInstancesResponse',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.workflow.v1'),
      createEmptyInstance: create)
    ..pPM<WorkflowInstance>(1, _omitFieldNames ? '' : 'instances',
        subBuilder: WorkflowInstance.create)
    ..aOM<$1.PaginationResult>(2, _omitFieldNames ? '' : 'pagination',
        subBuilder: $1.PaginationResult.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListInstancesResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListInstancesResponse copyWith(
          void Function(ListInstancesResponse) updates) =>
      super.copyWith((message) => updates(message as ListInstancesResponse))
          as ListInstancesResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static ListInstancesResponse create() => ListInstancesResponse._();
  @$core.override
  ListInstancesResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static ListInstancesResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ListInstancesResponse>(create);
  static ListInstancesResponse? _defaultInstance;

  @$pb.TagNumber(1)
  $pb.PbList<WorkflowInstance> get instances => $_getList(0);

  @$pb.TagNumber(2)
  $1.PaginationResult get pagination => $_getN(1);
  @$pb.TagNumber(2)
  set pagination($1.PaginationResult value) => $_setField(2, value);
  @$pb.TagNumber(2)
  $core.bool hasPagination() => $_has(1);
  @$pb.TagNumber(2)
  void clearPagination() => $_clearField(2);
  @$pb.TagNumber(2)
  $1.PaginationResult ensurePagination() => $_ensure(1);
}

class CancelInstanceRequest extends $pb.GeneratedMessage {
  factory CancelInstanceRequest({
    $core.String? instanceId,
    $core.String? reason,
  }) {
    final result = create();
    if (instanceId != null) result.instanceId = instanceId;
    if (reason != null) result.reason = reason;
    return result;
  }

  CancelInstanceRequest._();

  factory CancelInstanceRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory CancelInstanceRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'CancelInstanceRequest',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.workflow.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'instanceId')
    ..aOS(2, _omitFieldNames ? '' : 'reason')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  CancelInstanceRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  CancelInstanceRequest copyWith(
          void Function(CancelInstanceRequest) updates) =>
      super.copyWith((message) => updates(message as CancelInstanceRequest))
          as CancelInstanceRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static CancelInstanceRequest create() => CancelInstanceRequest._();
  @$core.override
  CancelInstanceRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static CancelInstanceRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<CancelInstanceRequest>(create);
  static CancelInstanceRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get instanceId => $_getSZ(0);
  @$pb.TagNumber(1)
  set instanceId($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasInstanceId() => $_has(0);
  @$pb.TagNumber(1)
  void clearInstanceId() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get reason => $_getSZ(1);
  @$pb.TagNumber(2)
  set reason($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasReason() => $_has(1);
  @$pb.TagNumber(2)
  void clearReason() => $_clearField(2);
}

class CancelInstanceResponse extends $pb.GeneratedMessage {
  factory CancelInstanceResponse({
    WorkflowInstance? instance,
  }) {
    final result = create();
    if (instance != null) result.instance = instance;
    return result;
  }

  CancelInstanceResponse._();

  factory CancelInstanceResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory CancelInstanceResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'CancelInstanceResponse',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.workflow.v1'),
      createEmptyInstance: create)
    ..aOM<WorkflowInstance>(1, _omitFieldNames ? '' : 'instance',
        subBuilder: WorkflowInstance.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  CancelInstanceResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  CancelInstanceResponse copyWith(
          void Function(CancelInstanceResponse) updates) =>
      super.copyWith((message) => updates(message as CancelInstanceResponse))
          as CancelInstanceResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static CancelInstanceResponse create() => CancelInstanceResponse._();
  @$core.override
  CancelInstanceResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static CancelInstanceResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<CancelInstanceResponse>(create);
  static CancelInstanceResponse? _defaultInstance;

  @$pb.TagNumber(1)
  WorkflowInstance get instance => $_getN(0);
  @$pb.TagNumber(1)
  set instance(WorkflowInstance value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasInstance() => $_has(0);
  @$pb.TagNumber(1)
  void clearInstance() => $_clearField(1);
  @$pb.TagNumber(1)
  WorkflowInstance ensureInstance() => $_ensure(0);
}

class WorkflowTask extends $pb.GeneratedMessage {
  factory WorkflowTask({
    $core.String? id,
    $core.String? instanceId,
    $core.String? stepId,
    $core.String? stepName,
    $core.String? assigneeId,
    $core.String? status,
    $1.Timestamp? dueAt,
    $core.String? comment,
    $core.String? actorId,
    $1.Timestamp? decidedAt,
    $1.Timestamp? createdAt,
    $1.Timestamp? updatedAt,
  }) {
    final result = create();
    if (id != null) result.id = id;
    if (instanceId != null) result.instanceId = instanceId;
    if (stepId != null) result.stepId = stepId;
    if (stepName != null) result.stepName = stepName;
    if (assigneeId != null) result.assigneeId = assigneeId;
    if (status != null) result.status = status;
    if (dueAt != null) result.dueAt = dueAt;
    if (comment != null) result.comment = comment;
    if (actorId != null) result.actorId = actorId;
    if (decidedAt != null) result.decidedAt = decidedAt;
    if (createdAt != null) result.createdAt = createdAt;
    if (updatedAt != null) result.updatedAt = updatedAt;
    return result;
  }

  WorkflowTask._();

  factory WorkflowTask.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory WorkflowTask.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'WorkflowTask',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.workflow.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'id')
    ..aOS(2, _omitFieldNames ? '' : 'instanceId')
    ..aOS(3, _omitFieldNames ? '' : 'stepId')
    ..aOS(4, _omitFieldNames ? '' : 'stepName')
    ..aOS(5, _omitFieldNames ? '' : 'assigneeId')
    ..aOS(6, _omitFieldNames ? '' : 'status')
    ..aOM<$1.Timestamp>(7, _omitFieldNames ? '' : 'dueAt',
        subBuilder: $1.Timestamp.create)
    ..aOS(8, _omitFieldNames ? '' : 'comment')
    ..aOS(9, _omitFieldNames ? '' : 'actorId')
    ..aOM<$1.Timestamp>(10, _omitFieldNames ? '' : 'decidedAt',
        subBuilder: $1.Timestamp.create)
    ..aOM<$1.Timestamp>(11, _omitFieldNames ? '' : 'createdAt',
        subBuilder: $1.Timestamp.create)
    ..aOM<$1.Timestamp>(12, _omitFieldNames ? '' : 'updatedAt',
        subBuilder: $1.Timestamp.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  WorkflowTask clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  WorkflowTask copyWith(void Function(WorkflowTask) updates) =>
      super.copyWith((message) => updates(message as WorkflowTask))
          as WorkflowTask;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static WorkflowTask create() => WorkflowTask._();
  @$core.override
  WorkflowTask createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static WorkflowTask getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<WorkflowTask>(create);
  static WorkflowTask? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get id => $_getSZ(0);
  @$pb.TagNumber(1)
  set id($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasId() => $_has(0);
  @$pb.TagNumber(1)
  void clearId() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get instanceId => $_getSZ(1);
  @$pb.TagNumber(2)
  set instanceId($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasInstanceId() => $_has(1);
  @$pb.TagNumber(2)
  void clearInstanceId() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.String get stepId => $_getSZ(2);
  @$pb.TagNumber(3)
  set stepId($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasStepId() => $_has(2);
  @$pb.TagNumber(3)
  void clearStepId() => $_clearField(3);

  @$pb.TagNumber(4)
  $core.String get stepName => $_getSZ(3);
  @$pb.TagNumber(4)
  set stepName($core.String value) => $_setString(3, value);
  @$pb.TagNumber(4)
  $core.bool hasStepName() => $_has(3);
  @$pb.TagNumber(4)
  void clearStepName() => $_clearField(4);

  @$pb.TagNumber(5)
  $core.String get assigneeId => $_getSZ(4);
  @$pb.TagNumber(5)
  set assigneeId($core.String value) => $_setString(4, value);
  @$pb.TagNumber(5)
  $core.bool hasAssigneeId() => $_has(4);
  @$pb.TagNumber(5)
  void clearAssigneeId() => $_clearField(5);

  @$pb.TagNumber(6)
  $core.String get status => $_getSZ(5);
  @$pb.TagNumber(6)
  set status($core.String value) => $_setString(5, value);
  @$pb.TagNumber(6)
  $core.bool hasStatus() => $_has(5);
  @$pb.TagNumber(6)
  void clearStatus() => $_clearField(6);

  @$pb.TagNumber(7)
  $1.Timestamp get dueAt => $_getN(6);
  @$pb.TagNumber(7)
  set dueAt($1.Timestamp value) => $_setField(7, value);
  @$pb.TagNumber(7)
  $core.bool hasDueAt() => $_has(6);
  @$pb.TagNumber(7)
  void clearDueAt() => $_clearField(7);
  @$pb.TagNumber(7)
  $1.Timestamp ensureDueAt() => $_ensure(6);

  @$pb.TagNumber(8)
  $core.String get comment => $_getSZ(7);
  @$pb.TagNumber(8)
  set comment($core.String value) => $_setString(7, value);
  @$pb.TagNumber(8)
  $core.bool hasComment() => $_has(7);
  @$pb.TagNumber(8)
  void clearComment() => $_clearField(8);

  @$pb.TagNumber(9)
  $core.String get actorId => $_getSZ(8);
  @$pb.TagNumber(9)
  set actorId($core.String value) => $_setString(8, value);
  @$pb.TagNumber(9)
  $core.bool hasActorId() => $_has(8);
  @$pb.TagNumber(9)
  void clearActorId() => $_clearField(9);

  @$pb.TagNumber(10)
  $1.Timestamp get decidedAt => $_getN(9);
  @$pb.TagNumber(10)
  set decidedAt($1.Timestamp value) => $_setField(10, value);
  @$pb.TagNumber(10)
  $core.bool hasDecidedAt() => $_has(9);
  @$pb.TagNumber(10)
  void clearDecidedAt() => $_clearField(10);
  @$pb.TagNumber(10)
  $1.Timestamp ensureDecidedAt() => $_ensure(9);

  @$pb.TagNumber(11)
  $1.Timestamp get createdAt => $_getN(10);
  @$pb.TagNumber(11)
  set createdAt($1.Timestamp value) => $_setField(11, value);
  @$pb.TagNumber(11)
  $core.bool hasCreatedAt() => $_has(10);
  @$pb.TagNumber(11)
  void clearCreatedAt() => $_clearField(11);
  @$pb.TagNumber(11)
  $1.Timestamp ensureCreatedAt() => $_ensure(10);

  @$pb.TagNumber(12)
  $1.Timestamp get updatedAt => $_getN(11);
  @$pb.TagNumber(12)
  set updatedAt($1.Timestamp value) => $_setField(12, value);
  @$pb.TagNumber(12)
  $core.bool hasUpdatedAt() => $_has(11);
  @$pb.TagNumber(12)
  void clearUpdatedAt() => $_clearField(12);
  @$pb.TagNumber(12)
  $1.Timestamp ensureUpdatedAt() => $_ensure(11);
}

class ListTasksRequest extends $pb.GeneratedMessage {
  factory ListTasksRequest({
    $core.String? assigneeId,
    $core.String? status,
    $core.String? instanceId,
    $core.bool? overdueOnly,
    $1.Pagination? pagination,
  }) {
    final result = create();
    if (assigneeId != null) result.assigneeId = assigneeId;
    if (status != null) result.status = status;
    if (instanceId != null) result.instanceId = instanceId;
    if (overdueOnly != null) result.overdueOnly = overdueOnly;
    if (pagination != null) result.pagination = pagination;
    return result;
  }

  ListTasksRequest._();

  factory ListTasksRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory ListTasksRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ListTasksRequest',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.workflow.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'assigneeId')
    ..aOS(2, _omitFieldNames ? '' : 'status')
    ..aOS(3, _omitFieldNames ? '' : 'instanceId')
    ..aOB(4, _omitFieldNames ? '' : 'overdueOnly')
    ..aOM<$1.Pagination>(5, _omitFieldNames ? '' : 'pagination',
        subBuilder: $1.Pagination.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListTasksRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListTasksRequest copyWith(void Function(ListTasksRequest) updates) =>
      super.copyWith((message) => updates(message as ListTasksRequest))
          as ListTasksRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static ListTasksRequest create() => ListTasksRequest._();
  @$core.override
  ListTasksRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static ListTasksRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ListTasksRequest>(create);
  static ListTasksRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get assigneeId => $_getSZ(0);
  @$pb.TagNumber(1)
  set assigneeId($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasAssigneeId() => $_has(0);
  @$pb.TagNumber(1)
  void clearAssigneeId() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get status => $_getSZ(1);
  @$pb.TagNumber(2)
  set status($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasStatus() => $_has(1);
  @$pb.TagNumber(2)
  void clearStatus() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.String get instanceId => $_getSZ(2);
  @$pb.TagNumber(3)
  set instanceId($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasInstanceId() => $_has(2);
  @$pb.TagNumber(3)
  void clearInstanceId() => $_clearField(3);

  @$pb.TagNumber(4)
  $core.bool get overdueOnly => $_getBF(3);
  @$pb.TagNumber(4)
  set overdueOnly($core.bool value) => $_setBool(3, value);
  @$pb.TagNumber(4)
  $core.bool hasOverdueOnly() => $_has(3);
  @$pb.TagNumber(4)
  void clearOverdueOnly() => $_clearField(4);

  @$pb.TagNumber(5)
  $1.Pagination get pagination => $_getN(4);
  @$pb.TagNumber(5)
  set pagination($1.Pagination value) => $_setField(5, value);
  @$pb.TagNumber(5)
  $core.bool hasPagination() => $_has(4);
  @$pb.TagNumber(5)
  void clearPagination() => $_clearField(5);
  @$pb.TagNumber(5)
  $1.Pagination ensurePagination() => $_ensure(4);
}

class ListTasksResponse extends $pb.GeneratedMessage {
  factory ListTasksResponse({
    $core.Iterable<WorkflowTask>? tasks,
    $1.PaginationResult? pagination,
  }) {
    final result = create();
    if (tasks != null) result.tasks.addAll(tasks);
    if (pagination != null) result.pagination = pagination;
    return result;
  }

  ListTasksResponse._();

  factory ListTasksResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory ListTasksResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ListTasksResponse',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.workflow.v1'),
      createEmptyInstance: create)
    ..pPM<WorkflowTask>(1, _omitFieldNames ? '' : 'tasks',
        subBuilder: WorkflowTask.create)
    ..aOM<$1.PaginationResult>(2, _omitFieldNames ? '' : 'pagination',
        subBuilder: $1.PaginationResult.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListTasksResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListTasksResponse copyWith(void Function(ListTasksResponse) updates) =>
      super.copyWith((message) => updates(message as ListTasksResponse))
          as ListTasksResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static ListTasksResponse create() => ListTasksResponse._();
  @$core.override
  ListTasksResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static ListTasksResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ListTasksResponse>(create);
  static ListTasksResponse? _defaultInstance;

  @$pb.TagNumber(1)
  $pb.PbList<WorkflowTask> get tasks => $_getList(0);

  @$pb.TagNumber(2)
  $1.PaginationResult get pagination => $_getN(1);
  @$pb.TagNumber(2)
  set pagination($1.PaginationResult value) => $_setField(2, value);
  @$pb.TagNumber(2)
  $core.bool hasPagination() => $_has(1);
  @$pb.TagNumber(2)
  void clearPagination() => $_clearField(2);
  @$pb.TagNumber(2)
  $1.PaginationResult ensurePagination() => $_ensure(1);
}

class ReassignTaskRequest extends $pb.GeneratedMessage {
  factory ReassignTaskRequest({
    $core.String? taskId,
    $core.String? newAssigneeId,
    $core.String? reason,
    $core.String? actorId,
  }) {
    final result = create();
    if (taskId != null) result.taskId = taskId;
    if (newAssigneeId != null) result.newAssigneeId = newAssigneeId;
    if (reason != null) result.reason = reason;
    if (actorId != null) result.actorId = actorId;
    return result;
  }

  ReassignTaskRequest._();

  factory ReassignTaskRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory ReassignTaskRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ReassignTaskRequest',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.workflow.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'taskId')
    ..aOS(2, _omitFieldNames ? '' : 'newAssigneeId')
    ..aOS(3, _omitFieldNames ? '' : 'reason')
    ..aOS(4, _omitFieldNames ? '' : 'actorId')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ReassignTaskRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ReassignTaskRequest copyWith(void Function(ReassignTaskRequest) updates) =>
      super.copyWith((message) => updates(message as ReassignTaskRequest))
          as ReassignTaskRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static ReassignTaskRequest create() => ReassignTaskRequest._();
  @$core.override
  ReassignTaskRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static ReassignTaskRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ReassignTaskRequest>(create);
  static ReassignTaskRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get taskId => $_getSZ(0);
  @$pb.TagNumber(1)
  set taskId($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasTaskId() => $_has(0);
  @$pb.TagNumber(1)
  void clearTaskId() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get newAssigneeId => $_getSZ(1);
  @$pb.TagNumber(2)
  set newAssigneeId($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasNewAssigneeId() => $_has(1);
  @$pb.TagNumber(2)
  void clearNewAssigneeId() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.String get reason => $_getSZ(2);
  @$pb.TagNumber(3)
  set reason($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasReason() => $_has(2);
  @$pb.TagNumber(3)
  void clearReason() => $_clearField(3);

  @$pb.TagNumber(4)
  $core.String get actorId => $_getSZ(3);
  @$pb.TagNumber(4)
  set actorId($core.String value) => $_setString(3, value);
  @$pb.TagNumber(4)
  $core.bool hasActorId() => $_has(3);
  @$pb.TagNumber(4)
  void clearActorId() => $_clearField(4);
}

class ReassignTaskResponse extends $pb.GeneratedMessage {
  factory ReassignTaskResponse({
    WorkflowTask? task,
    $core.String? previousAssigneeId,
  }) {
    final result = create();
    if (task != null) result.task = task;
    if (previousAssigneeId != null)
      result.previousAssigneeId = previousAssigneeId;
    return result;
  }

  ReassignTaskResponse._();

  factory ReassignTaskResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory ReassignTaskResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ReassignTaskResponse',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.workflow.v1'),
      createEmptyInstance: create)
    ..aOM<WorkflowTask>(1, _omitFieldNames ? '' : 'task',
        subBuilder: WorkflowTask.create)
    ..aOS(2, _omitFieldNames ? '' : 'previousAssigneeId')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ReassignTaskResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ReassignTaskResponse copyWith(void Function(ReassignTaskResponse) updates) =>
      super.copyWith((message) => updates(message as ReassignTaskResponse))
          as ReassignTaskResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static ReassignTaskResponse create() => ReassignTaskResponse._();
  @$core.override
  ReassignTaskResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static ReassignTaskResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ReassignTaskResponse>(create);
  static ReassignTaskResponse? _defaultInstance;

  @$pb.TagNumber(1)
  WorkflowTask get task => $_getN(0);
  @$pb.TagNumber(1)
  set task(WorkflowTask value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasTask() => $_has(0);
  @$pb.TagNumber(1)
  void clearTask() => $_clearField(1);
  @$pb.TagNumber(1)
  WorkflowTask ensureTask() => $_ensure(0);

  @$pb.TagNumber(2)
  $core.String get previousAssigneeId => $_getSZ(1);
  @$pb.TagNumber(2)
  set previousAssigneeId($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasPreviousAssigneeId() => $_has(1);
  @$pb.TagNumber(2)
  void clearPreviousAssigneeId() => $_clearField(2);
}

class WorkflowInstance extends $pb.GeneratedMessage {
  factory WorkflowInstance({
    $core.String? id,
    $core.String? workflowId,
    $core.String? workflowName,
    $core.String? title,
    $core.String? initiatorId,
    $core.String? currentStepId,
    $core.String? status,
    $core.List<$core.int>? contextJson,
    $1.Timestamp? startedAt,
    $1.Timestamp? completedAt,
    $1.Timestamp? createdAt,
  }) {
    final result = create();
    if (id != null) result.id = id;
    if (workflowId != null) result.workflowId = workflowId;
    if (workflowName != null) result.workflowName = workflowName;
    if (title != null) result.title = title;
    if (initiatorId != null) result.initiatorId = initiatorId;
    if (currentStepId != null) result.currentStepId = currentStepId;
    if (status != null) result.status = status;
    if (contextJson != null) result.contextJson = contextJson;
    if (startedAt != null) result.startedAt = startedAt;
    if (completedAt != null) result.completedAt = completedAt;
    if (createdAt != null) result.createdAt = createdAt;
    return result;
  }

  WorkflowInstance._();

  factory WorkflowInstance.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory WorkflowInstance.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'WorkflowInstance',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.workflow.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'id')
    ..aOS(2, _omitFieldNames ? '' : 'workflowId')
    ..aOS(3, _omitFieldNames ? '' : 'workflowName')
    ..aOS(4, _omitFieldNames ? '' : 'title')
    ..aOS(5, _omitFieldNames ? '' : 'initiatorId')
    ..aOS(6, _omitFieldNames ? '' : 'currentStepId')
    ..aOS(7, _omitFieldNames ? '' : 'status')
    ..a<$core.List<$core.int>>(
        8, _omitFieldNames ? '' : 'contextJson', $pb.PbFieldType.OY)
    ..aOM<$1.Timestamp>(9, _omitFieldNames ? '' : 'startedAt',
        subBuilder: $1.Timestamp.create)
    ..aOM<$1.Timestamp>(10, _omitFieldNames ? '' : 'completedAt',
        subBuilder: $1.Timestamp.create)
    ..aOM<$1.Timestamp>(11, _omitFieldNames ? '' : 'createdAt',
        subBuilder: $1.Timestamp.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  WorkflowInstance clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  WorkflowInstance copyWith(void Function(WorkflowInstance) updates) =>
      super.copyWith((message) => updates(message as WorkflowInstance))
          as WorkflowInstance;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static WorkflowInstance create() => WorkflowInstance._();
  @$core.override
  WorkflowInstance createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static WorkflowInstance getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<WorkflowInstance>(create);
  static WorkflowInstance? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get id => $_getSZ(0);
  @$pb.TagNumber(1)
  set id($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasId() => $_has(0);
  @$pb.TagNumber(1)
  void clearId() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get workflowId => $_getSZ(1);
  @$pb.TagNumber(2)
  set workflowId($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasWorkflowId() => $_has(1);
  @$pb.TagNumber(2)
  void clearWorkflowId() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.String get workflowName => $_getSZ(2);
  @$pb.TagNumber(3)
  set workflowName($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasWorkflowName() => $_has(2);
  @$pb.TagNumber(3)
  void clearWorkflowName() => $_clearField(3);

  @$pb.TagNumber(4)
  $core.String get title => $_getSZ(3);
  @$pb.TagNumber(4)
  set title($core.String value) => $_setString(3, value);
  @$pb.TagNumber(4)
  $core.bool hasTitle() => $_has(3);
  @$pb.TagNumber(4)
  void clearTitle() => $_clearField(4);

  @$pb.TagNumber(5)
  $core.String get initiatorId => $_getSZ(4);
  @$pb.TagNumber(5)
  set initiatorId($core.String value) => $_setString(4, value);
  @$pb.TagNumber(5)
  $core.bool hasInitiatorId() => $_has(4);
  @$pb.TagNumber(5)
  void clearInitiatorId() => $_clearField(5);

  @$pb.TagNumber(6)
  $core.String get currentStepId => $_getSZ(5);
  @$pb.TagNumber(6)
  set currentStepId($core.String value) => $_setString(5, value);
  @$pb.TagNumber(6)
  $core.bool hasCurrentStepId() => $_has(5);
  @$pb.TagNumber(6)
  void clearCurrentStepId() => $_clearField(6);

  /// インスタンス状態（pending / running / completed / cancelled / failed）
  @$pb.TagNumber(7)
  $core.String get status => $_getSZ(6);
  @$pb.TagNumber(7)
  set status($core.String value) => $_setString(6, value);
  @$pb.TagNumber(7)
  $core.bool hasStatus() => $_has(6);
  @$pb.TagNumber(7)
  void clearStatus() => $_clearField(7);

  /// ワークフロー実行コンテキスト（JSON バイト列）
  @$pb.TagNumber(8)
  $core.List<$core.int> get contextJson => $_getN(7);
  @$pb.TagNumber(8)
  set contextJson($core.List<$core.int> value) => $_setBytes(7, value);
  @$pb.TagNumber(8)
  $core.bool hasContextJson() => $_has(7);
  @$pb.TagNumber(8)
  void clearContextJson() => $_clearField(8);

  @$pb.TagNumber(9)
  $1.Timestamp get startedAt => $_getN(8);
  @$pb.TagNumber(9)
  set startedAt($1.Timestamp value) => $_setField(9, value);
  @$pb.TagNumber(9)
  $core.bool hasStartedAt() => $_has(8);
  @$pb.TagNumber(9)
  void clearStartedAt() => $_clearField(9);
  @$pb.TagNumber(9)
  $1.Timestamp ensureStartedAt() => $_ensure(8);

  @$pb.TagNumber(10)
  $1.Timestamp get completedAt => $_getN(9);
  @$pb.TagNumber(10)
  set completedAt($1.Timestamp value) => $_setField(10, value);
  @$pb.TagNumber(10)
  $core.bool hasCompletedAt() => $_has(9);
  @$pb.TagNumber(10)
  void clearCompletedAt() => $_clearField(10);
  @$pb.TagNumber(10)
  $1.Timestamp ensureCompletedAt() => $_ensure(9);

  @$pb.TagNumber(11)
  $1.Timestamp get createdAt => $_getN(10);
  @$pb.TagNumber(11)
  set createdAt($1.Timestamp value) => $_setField(11, value);
  @$pb.TagNumber(11)
  $core.bool hasCreatedAt() => $_has(10);
  @$pb.TagNumber(11)
  void clearCreatedAt() => $_clearField(11);
  @$pb.TagNumber(11)
  $1.Timestamp ensureCreatedAt() => $_ensure(10);
}

class ApproveTaskRequest extends $pb.GeneratedMessage {
  factory ApproveTaskRequest({
    $core.String? taskId,
    $core.String? actorId,
    $core.String? comment,
  }) {
    final result = create();
    if (taskId != null) result.taskId = taskId;
    if (actorId != null) result.actorId = actorId;
    if (comment != null) result.comment = comment;
    return result;
  }

  ApproveTaskRequest._();

  factory ApproveTaskRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory ApproveTaskRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ApproveTaskRequest',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.workflow.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'taskId')
    ..aOS(2, _omitFieldNames ? '' : 'actorId')
    ..aOS(3, _omitFieldNames ? '' : 'comment')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ApproveTaskRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ApproveTaskRequest copyWith(void Function(ApproveTaskRequest) updates) =>
      super.copyWith((message) => updates(message as ApproveTaskRequest))
          as ApproveTaskRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static ApproveTaskRequest create() => ApproveTaskRequest._();
  @$core.override
  ApproveTaskRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static ApproveTaskRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ApproveTaskRequest>(create);
  static ApproveTaskRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get taskId => $_getSZ(0);
  @$pb.TagNumber(1)
  set taskId($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasTaskId() => $_has(0);
  @$pb.TagNumber(1)
  void clearTaskId() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get actorId => $_getSZ(1);
  @$pb.TagNumber(2)
  set actorId($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasActorId() => $_has(1);
  @$pb.TagNumber(2)
  void clearActorId() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.String get comment => $_getSZ(2);
  @$pb.TagNumber(3)
  set comment($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasComment() => $_has(2);
  @$pb.TagNumber(3)
  void clearComment() => $_clearField(3);
}

class ApproveTaskResponse extends $pb.GeneratedMessage {
  factory ApproveTaskResponse({
    $core.String? taskId,
    $core.String? status,
    $core.String? nextTaskId,
    $core.String? instanceStatus,
  }) {
    final result = create();
    if (taskId != null) result.taskId = taskId;
    if (status != null) result.status = status;
    if (nextTaskId != null) result.nextTaskId = nextTaskId;
    if (instanceStatus != null) result.instanceStatus = instanceStatus;
    return result;
  }

  ApproveTaskResponse._();

  factory ApproveTaskResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory ApproveTaskResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ApproveTaskResponse',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.workflow.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'taskId')
    ..aOS(2, _omitFieldNames ? '' : 'status')
    ..aOS(3, _omitFieldNames ? '' : 'nextTaskId')
    ..aOS(4, _omitFieldNames ? '' : 'instanceStatus')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ApproveTaskResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ApproveTaskResponse copyWith(void Function(ApproveTaskResponse) updates) =>
      super.copyWith((message) => updates(message as ApproveTaskResponse))
          as ApproveTaskResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static ApproveTaskResponse create() => ApproveTaskResponse._();
  @$core.override
  ApproveTaskResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static ApproveTaskResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ApproveTaskResponse>(create);
  static ApproveTaskResponse? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get taskId => $_getSZ(0);
  @$pb.TagNumber(1)
  set taskId($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasTaskId() => $_has(0);
  @$pb.TagNumber(1)
  void clearTaskId() => $_clearField(1);

  /// タスク状態（approved）
  @$pb.TagNumber(2)
  $core.String get status => $_getSZ(1);
  @$pb.TagNumber(2)
  set status($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasStatus() => $_has(1);
  @$pb.TagNumber(2)
  void clearStatus() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.String get nextTaskId => $_getSZ(2);
  @$pb.TagNumber(3)
  set nextTaskId($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasNextTaskId() => $_has(2);
  @$pb.TagNumber(3)
  void clearNextTaskId() => $_clearField(3);

  /// インスタンス状態（running / completed）
  @$pb.TagNumber(4)
  $core.String get instanceStatus => $_getSZ(3);
  @$pb.TagNumber(4)
  set instanceStatus($core.String value) => $_setString(3, value);
  @$pb.TagNumber(4)
  $core.bool hasInstanceStatus() => $_has(3);
  @$pb.TagNumber(4)
  void clearInstanceStatus() => $_clearField(4);
}

class RejectTaskRequest extends $pb.GeneratedMessage {
  factory RejectTaskRequest({
    $core.String? taskId,
    $core.String? actorId,
    $core.String? comment,
  }) {
    final result = create();
    if (taskId != null) result.taskId = taskId;
    if (actorId != null) result.actorId = actorId;
    if (comment != null) result.comment = comment;
    return result;
  }

  RejectTaskRequest._();

  factory RejectTaskRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory RejectTaskRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'RejectTaskRequest',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.workflow.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'taskId')
    ..aOS(2, _omitFieldNames ? '' : 'actorId')
    ..aOS(3, _omitFieldNames ? '' : 'comment')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  RejectTaskRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  RejectTaskRequest copyWith(void Function(RejectTaskRequest) updates) =>
      super.copyWith((message) => updates(message as RejectTaskRequest))
          as RejectTaskRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static RejectTaskRequest create() => RejectTaskRequest._();
  @$core.override
  RejectTaskRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static RejectTaskRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<RejectTaskRequest>(create);
  static RejectTaskRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get taskId => $_getSZ(0);
  @$pb.TagNumber(1)
  set taskId($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasTaskId() => $_has(0);
  @$pb.TagNumber(1)
  void clearTaskId() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get actorId => $_getSZ(1);
  @$pb.TagNumber(2)
  set actorId($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasActorId() => $_has(1);
  @$pb.TagNumber(2)
  void clearActorId() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.String get comment => $_getSZ(2);
  @$pb.TagNumber(3)
  set comment($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasComment() => $_has(2);
  @$pb.TagNumber(3)
  void clearComment() => $_clearField(3);
}

class RejectTaskResponse extends $pb.GeneratedMessage {
  factory RejectTaskResponse({
    $core.String? taskId,
    $core.String? status,
    $core.String? nextTaskId,
    $core.String? instanceStatus,
  }) {
    final result = create();
    if (taskId != null) result.taskId = taskId;
    if (status != null) result.status = status;
    if (nextTaskId != null) result.nextTaskId = nextTaskId;
    if (instanceStatus != null) result.instanceStatus = instanceStatus;
    return result;
  }

  RejectTaskResponse._();

  factory RejectTaskResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory RejectTaskResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'RejectTaskResponse',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.workflow.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'taskId')
    ..aOS(2, _omitFieldNames ? '' : 'status')
    ..aOS(3, _omitFieldNames ? '' : 'nextTaskId')
    ..aOS(4, _omitFieldNames ? '' : 'instanceStatus')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  RejectTaskResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  RejectTaskResponse copyWith(void Function(RejectTaskResponse) updates) =>
      super.copyWith((message) => updates(message as RejectTaskResponse))
          as RejectTaskResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static RejectTaskResponse create() => RejectTaskResponse._();
  @$core.override
  RejectTaskResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static RejectTaskResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<RejectTaskResponse>(create);
  static RejectTaskResponse? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get taskId => $_getSZ(0);
  @$pb.TagNumber(1)
  set taskId($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasTaskId() => $_has(0);
  @$pb.TagNumber(1)
  void clearTaskId() => $_clearField(1);

  /// タスク状態（rejected）
  @$pb.TagNumber(2)
  $core.String get status => $_getSZ(1);
  @$pb.TagNumber(2)
  set status($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasStatus() => $_has(1);
  @$pb.TagNumber(2)
  void clearStatus() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.String get nextTaskId => $_getSZ(2);
  @$pb.TagNumber(3)
  set nextTaskId($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasNextTaskId() => $_has(2);
  @$pb.TagNumber(3)
  void clearNextTaskId() => $_clearField(3);

  /// インスタンス状態（running / failed）
  @$pb.TagNumber(4)
  $core.String get instanceStatus => $_getSZ(3);
  @$pb.TagNumber(4)
  set instanceStatus($core.String value) => $_setString(3, value);
  @$pb.TagNumber(4)
  $core.bool hasInstanceStatus() => $_has(3);
  @$pb.TagNumber(4)
  void clearInstanceStatus() => $_clearField(4);
}

const $core.bool _omitFieldNames =
    $core.bool.fromEnvironment('protobuf.omit_field_names');
const $core.bool _omitMessageNames =
    $core.bool.fromEnvironment('protobuf.omit_message_names');
