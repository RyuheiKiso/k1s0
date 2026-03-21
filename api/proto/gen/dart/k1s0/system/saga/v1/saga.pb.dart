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
import 'package:protobuf/well_known_types/google/protobuf/struct.pb.dart' as $1;

import '../../common/v1/types.pb.dart' as $2;
import 'saga.pbenum.dart';

export 'package:protobuf/protobuf.dart' show GeneratedMessageGenericExtensions;

export 'saga.pbenum.dart';

/// SagaStateProto は Saga の状態情報。
class SagaStateProto extends $pb.GeneratedMessage {
  factory SagaStateProto({
    $core.String? id,
    $core.String? workflowName,
    $core.int? currentStep,
    $core.String? status,
    $1.Struct? payload,
    $core.String? correlationId,
    $core.String? initiatedBy,
    $core.String? errorMessage,
    $2.Timestamp? createdAt,
    $2.Timestamp? updatedAt,
    SagaStatus? statusEnum,
  }) {
    final result = create();
    if (id != null) result.id = id;
    if (workflowName != null) result.workflowName = workflowName;
    if (currentStep != null) result.currentStep = currentStep;
    if (status != null) result.status = status;
    if (payload != null) result.payload = payload;
    if (correlationId != null) result.correlationId = correlationId;
    if (initiatedBy != null) result.initiatedBy = initiatedBy;
    if (errorMessage != null) result.errorMessage = errorMessage;
    if (createdAt != null) result.createdAt = createdAt;
    if (updatedAt != null) result.updatedAt = updatedAt;
    if (statusEnum != null) result.statusEnum = statusEnum;
    return result;
  }

  SagaStateProto._();

  factory SagaStateProto.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory SagaStateProto.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'SagaStateProto',
      package:
          const $pb.PackageName(_omitMessageNames ? '' : 'k1s0.system.saga.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'id')
    ..aOS(2, _omitFieldNames ? '' : 'workflowName')
    ..aI(3, _omitFieldNames ? '' : 'currentStep')
    ..aOS(4, _omitFieldNames ? '' : 'status')
    ..aOM<$1.Struct>(5, _omitFieldNames ? '' : 'payload',
        subBuilder: $1.Struct.create)
    ..aOS(6, _omitFieldNames ? '' : 'correlationId')
    ..aOS(7, _omitFieldNames ? '' : 'initiatedBy')
    ..aOS(8, _omitFieldNames ? '' : 'errorMessage')
    ..aOM<$2.Timestamp>(9, _omitFieldNames ? '' : 'createdAt',
        subBuilder: $2.Timestamp.create)
    ..aOM<$2.Timestamp>(10, _omitFieldNames ? '' : 'updatedAt',
        subBuilder: $2.Timestamp.create)
    ..aE<SagaStatus>(11, _omitFieldNames ? '' : 'statusEnum',
        enumValues: SagaStatus.values)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  SagaStateProto clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  SagaStateProto copyWith(void Function(SagaStateProto) updates) =>
      super.copyWith((message) => updates(message as SagaStateProto))
          as SagaStateProto;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static SagaStateProto create() => SagaStateProto._();
  @$core.override
  SagaStateProto createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static SagaStateProto getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<SagaStateProto>(create);
  static SagaStateProto? _defaultInstance;

  /// Saga UUID
  @$pb.TagNumber(1)
  $core.String get id => $_getSZ(0);
  @$pb.TagNumber(1)
  set id($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasId() => $_has(0);
  @$pb.TagNumber(1)
  void clearId() => $_clearField(1);

  /// ワークフロー名
  @$pb.TagNumber(2)
  $core.String get workflowName => $_getSZ(1);
  @$pb.TagNumber(2)
  set workflowName($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasWorkflowName() => $_has(1);
  @$pb.TagNumber(2)
  void clearWorkflowName() => $_clearField(2);

  /// 現在のステップインデックス
  @$pb.TagNumber(3)
  $core.int get currentStep => $_getIZ(2);
  @$pb.TagNumber(3)
  set currentStep($core.int value) => $_setSignedInt32(2, value);
  @$pb.TagNumber(3)
  $core.bool hasCurrentStep() => $_has(2);
  @$pb.TagNumber(3)
  void clearCurrentStep() => $_clearField(3);

  /// Deprecated: use status_enum instead.
  /// ステータス: STARTED, RUNNING, COMPLETED, COMPENSATING, FAILED, CANCELLED
  @$pb.TagNumber(4)
  $core.String get status => $_getSZ(3);
  @$pb.TagNumber(4)
  set status($core.String value) => $_setString(3, value);
  @$pb.TagNumber(4)
  $core.bool hasStatus() => $_has(3);
  @$pb.TagNumber(4)
  void clearStatus() => $_clearField(4);

  /// 各ステップに渡す JSON ペイロード
  @$pb.TagNumber(5)
  $1.Struct get payload => $_getN(4);
  @$pb.TagNumber(5)
  set payload($1.Struct value) => $_setField(5, value);
  @$pb.TagNumber(5)
  $core.bool hasPayload() => $_has(4);
  @$pb.TagNumber(5)
  void clearPayload() => $_clearField(5);
  @$pb.TagNumber(5)
  $1.Struct ensurePayload() => $_ensure(4);

  /// 業務相関 ID
  @$pb.TagNumber(6)
  $core.String get correlationId => $_getSZ(5);
  @$pb.TagNumber(6)
  set correlationId($core.String value) => $_setString(5, value);
  @$pb.TagNumber(6)
  $core.bool hasCorrelationId() => $_has(5);
  @$pb.TagNumber(6)
  void clearCorrelationId() => $_clearField(6);

  /// 呼び出し元サービス名
  @$pb.TagNumber(7)
  $core.String get initiatedBy => $_getSZ(6);
  @$pb.TagNumber(7)
  set initiatedBy($core.String value) => $_setString(6, value);
  @$pb.TagNumber(7)
  $core.bool hasInitiatedBy() => $_has(6);
  @$pb.TagNumber(7)
  void clearInitiatedBy() => $_clearField(7);

  /// エラーメッセージ（失敗時）
  @$pb.TagNumber(8)
  $core.String get errorMessage => $_getSZ(7);
  @$pb.TagNumber(8)
  set errorMessage($core.String value) => $_setString(7, value);
  @$pb.TagNumber(8)
  $core.bool hasErrorMessage() => $_has(7);
  @$pb.TagNumber(8)
  void clearErrorMessage() => $_clearField(8);

  @$pb.TagNumber(9)
  $2.Timestamp get createdAt => $_getN(8);
  @$pb.TagNumber(9)
  set createdAt($2.Timestamp value) => $_setField(9, value);
  @$pb.TagNumber(9)
  $core.bool hasCreatedAt() => $_has(8);
  @$pb.TagNumber(9)
  void clearCreatedAt() => $_clearField(9);
  @$pb.TagNumber(9)
  $2.Timestamp ensureCreatedAt() => $_ensure(8);

  @$pb.TagNumber(10)
  $2.Timestamp get updatedAt => $_getN(9);
  @$pb.TagNumber(10)
  set updatedAt($2.Timestamp value) => $_setField(10, value);
  @$pb.TagNumber(10)
  $core.bool hasUpdatedAt() => $_has(9);
  @$pb.TagNumber(10)
  void clearUpdatedAt() => $_clearField(10);
  @$pb.TagNumber(10)
  $2.Timestamp ensureUpdatedAt() => $_ensure(9);

  /// Saga ステータスの enum 版（status の型付き版）。
  @$pb.TagNumber(11)
  SagaStatus get statusEnum => $_getN(10);
  @$pb.TagNumber(11)
  set statusEnum(SagaStatus value) => $_setField(11, value);
  @$pb.TagNumber(11)
  $core.bool hasStatusEnum() => $_has(10);
  @$pb.TagNumber(11)
  void clearStatusEnum() => $_clearField(11);
}

/// SagaStepLogProto は Saga の各ステップ実行ログ。
class SagaStepLogProto extends $pb.GeneratedMessage {
  factory SagaStepLogProto({
    $core.String? id,
    $core.String? sagaId,
    $core.int? stepIndex,
    $core.String? stepName,
    $core.String? action,
    $core.String? status,
    $1.Struct? requestPayload,
    $1.Struct? responsePayload,
    $core.String? errorMessage,
    $2.Timestamp? startedAt,
    $2.Timestamp? completedAt,
  }) {
    final result = create();
    if (id != null) result.id = id;
    if (sagaId != null) result.sagaId = sagaId;
    if (stepIndex != null) result.stepIndex = stepIndex;
    if (stepName != null) result.stepName = stepName;
    if (action != null) result.action = action;
    if (status != null) result.status = status;
    if (requestPayload != null) result.requestPayload = requestPayload;
    if (responsePayload != null) result.responsePayload = responsePayload;
    if (errorMessage != null) result.errorMessage = errorMessage;
    if (startedAt != null) result.startedAt = startedAt;
    if (completedAt != null) result.completedAt = completedAt;
    return result;
  }

  SagaStepLogProto._();

  factory SagaStepLogProto.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory SagaStepLogProto.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'SagaStepLogProto',
      package:
          const $pb.PackageName(_omitMessageNames ? '' : 'k1s0.system.saga.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'id')
    ..aOS(2, _omitFieldNames ? '' : 'sagaId')
    ..aI(3, _omitFieldNames ? '' : 'stepIndex')
    ..aOS(4, _omitFieldNames ? '' : 'stepName')
    ..aOS(5, _omitFieldNames ? '' : 'action')
    ..aOS(6, _omitFieldNames ? '' : 'status')
    ..aOM<$1.Struct>(7, _omitFieldNames ? '' : 'requestPayload',
        subBuilder: $1.Struct.create)
    ..aOM<$1.Struct>(8, _omitFieldNames ? '' : 'responsePayload',
        subBuilder: $1.Struct.create)
    ..aOS(9, _omitFieldNames ? '' : 'errorMessage')
    ..aOM<$2.Timestamp>(10, _omitFieldNames ? '' : 'startedAt',
        subBuilder: $2.Timestamp.create)
    ..aOM<$2.Timestamp>(11, _omitFieldNames ? '' : 'completedAt',
        subBuilder: $2.Timestamp.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  SagaStepLogProto clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  SagaStepLogProto copyWith(void Function(SagaStepLogProto) updates) =>
      super.copyWith((message) => updates(message as SagaStepLogProto))
          as SagaStepLogProto;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static SagaStepLogProto create() => SagaStepLogProto._();
  @$core.override
  SagaStepLogProto createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static SagaStepLogProto getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<SagaStepLogProto>(create);
  static SagaStepLogProto? _defaultInstance;

  /// ステップログ UUID
  @$pb.TagNumber(1)
  $core.String get id => $_getSZ(0);
  @$pb.TagNumber(1)
  set id($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasId() => $_has(0);
  @$pb.TagNumber(1)
  void clearId() => $_clearField(1);

  /// 親 Saga UUID
  @$pb.TagNumber(2)
  $core.String get sagaId => $_getSZ(1);
  @$pb.TagNumber(2)
  set sagaId($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasSagaId() => $_has(1);
  @$pb.TagNumber(2)
  void clearSagaId() => $_clearField(2);

  /// ステップインデックス（0 始まり）
  @$pb.TagNumber(3)
  $core.int get stepIndex => $_getIZ(2);
  @$pb.TagNumber(3)
  set stepIndex($core.int value) => $_setSignedInt32(2, value);
  @$pb.TagNumber(3)
  $core.bool hasStepIndex() => $_has(2);
  @$pb.TagNumber(3)
  void clearStepIndex() => $_clearField(3);

  /// ステップ名
  @$pb.TagNumber(4)
  $core.String get stepName => $_getSZ(3);
  @$pb.TagNumber(4)
  set stepName($core.String value) => $_setString(3, value);
  @$pb.TagNumber(4)
  $core.bool hasStepName() => $_has(3);
  @$pb.TagNumber(4)
  void clearStepName() => $_clearField(4);

  /// アクション種別: EXECUTE, COMPENSATE
  @$pb.TagNumber(5)
  $core.String get action => $_getSZ(4);
  @$pb.TagNumber(5)
  set action($core.String value) => $_setString(4, value);
  @$pb.TagNumber(5)
  $core.bool hasAction() => $_has(4);
  @$pb.TagNumber(5)
  void clearAction() => $_clearField(5);

  /// 実行結果: SUCCESS, FAILED, TIMEOUT, SKIPPED
  @$pb.TagNumber(6)
  $core.String get status => $_getSZ(5);
  @$pb.TagNumber(6)
  set status($core.String value) => $_setString(5, value);
  @$pb.TagNumber(6)
  $core.bool hasStatus() => $_has(5);
  @$pb.TagNumber(6)
  void clearStatus() => $_clearField(6);

  /// リクエストペイロード
  @$pb.TagNumber(7)
  $1.Struct get requestPayload => $_getN(6);
  @$pb.TagNumber(7)
  set requestPayload($1.Struct value) => $_setField(7, value);
  @$pb.TagNumber(7)
  $core.bool hasRequestPayload() => $_has(6);
  @$pb.TagNumber(7)
  void clearRequestPayload() => $_clearField(7);
  @$pb.TagNumber(7)
  $1.Struct ensureRequestPayload() => $_ensure(6);

  /// レスポンスペイロード
  @$pb.TagNumber(8)
  $1.Struct get responsePayload => $_getN(7);
  @$pb.TagNumber(8)
  set responsePayload($1.Struct value) => $_setField(8, value);
  @$pb.TagNumber(8)
  $core.bool hasResponsePayload() => $_has(7);
  @$pb.TagNumber(8)
  void clearResponsePayload() => $_clearField(8);
  @$pb.TagNumber(8)
  $1.Struct ensureResponsePayload() => $_ensure(7);

  /// エラーメッセージ（失敗時）
  @$pb.TagNumber(9)
  $core.String get errorMessage => $_getSZ(8);
  @$pb.TagNumber(9)
  set errorMessage($core.String value) => $_setString(8, value);
  @$pb.TagNumber(9)
  $core.bool hasErrorMessage() => $_has(8);
  @$pb.TagNumber(9)
  void clearErrorMessage() => $_clearField(9);

  @$pb.TagNumber(10)
  $2.Timestamp get startedAt => $_getN(9);
  @$pb.TagNumber(10)
  set startedAt($2.Timestamp value) => $_setField(10, value);
  @$pb.TagNumber(10)
  $core.bool hasStartedAt() => $_has(9);
  @$pb.TagNumber(10)
  void clearStartedAt() => $_clearField(10);
  @$pb.TagNumber(10)
  $2.Timestamp ensureStartedAt() => $_ensure(9);

  @$pb.TagNumber(11)
  $2.Timestamp get completedAt => $_getN(10);
  @$pb.TagNumber(11)
  set completedAt($2.Timestamp value) => $_setField(11, value);
  @$pb.TagNumber(11)
  $core.bool hasCompletedAt() => $_has(10);
  @$pb.TagNumber(11)
  void clearCompletedAt() => $_clearField(11);
  @$pb.TagNumber(11)
  $2.Timestamp ensureCompletedAt() => $_ensure(10);
}

/// WorkflowSummary はワークフローの概要情報。
class WorkflowSummary extends $pb.GeneratedMessage {
  factory WorkflowSummary({
    $core.String? name,
    $core.int? stepCount,
    $core.Iterable<$core.String>? stepNames,
  }) {
    final result = create();
    if (name != null) result.name = name;
    if (stepCount != null) result.stepCount = stepCount;
    if (stepNames != null) result.stepNames.addAll(stepNames);
    return result;
  }

  WorkflowSummary._();

  factory WorkflowSummary.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory WorkflowSummary.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'WorkflowSummary',
      package:
          const $pb.PackageName(_omitMessageNames ? '' : 'k1s0.system.saga.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'name')
    ..aI(2, _omitFieldNames ? '' : 'stepCount')
    ..pPS(3, _omitFieldNames ? '' : 'stepNames')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  WorkflowSummary clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  WorkflowSummary copyWith(void Function(WorkflowSummary) updates) =>
      super.copyWith((message) => updates(message as WorkflowSummary))
          as WorkflowSummary;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static WorkflowSummary create() => WorkflowSummary._();
  @$core.override
  WorkflowSummary createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static WorkflowSummary getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<WorkflowSummary>(create);
  static WorkflowSummary? _defaultInstance;

  /// ワークフロー名
  @$pb.TagNumber(1)
  $core.String get name => $_getSZ(0);
  @$pb.TagNumber(1)
  set name($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasName() => $_has(0);
  @$pb.TagNumber(1)
  void clearName() => $_clearField(1);

  /// ステップ数
  @$pb.TagNumber(2)
  $core.int get stepCount => $_getIZ(1);
  @$pb.TagNumber(2)
  set stepCount($core.int value) => $_setSignedInt32(1, value);
  @$pb.TagNumber(2)
  $core.bool hasStepCount() => $_has(1);
  @$pb.TagNumber(2)
  void clearStepCount() => $_clearField(2);

  /// ステップ名一覧
  @$pb.TagNumber(3)
  $pb.PbList<$core.String> get stepNames => $_getList(2);
}

/// StartSagaRequest は Saga 開始リクエスト。
class StartSagaRequest extends $pb.GeneratedMessage {
  factory StartSagaRequest({
    $core.String? workflowName,
    $1.Struct? payload,
    $core.String? correlationId,
    $core.String? initiatedBy,
  }) {
    final result = create();
    if (workflowName != null) result.workflowName = workflowName;
    if (payload != null) result.payload = payload;
    if (correlationId != null) result.correlationId = correlationId;
    if (initiatedBy != null) result.initiatedBy = initiatedBy;
    return result;
  }

  StartSagaRequest._();

  factory StartSagaRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory StartSagaRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'StartSagaRequest',
      package:
          const $pb.PackageName(_omitMessageNames ? '' : 'k1s0.system.saga.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'workflowName')
    ..aOM<$1.Struct>(2, _omitFieldNames ? '' : 'payload',
        subBuilder: $1.Struct.create)
    ..aOS(3, _omitFieldNames ? '' : 'correlationId')
    ..aOS(4, _omitFieldNames ? '' : 'initiatedBy')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  StartSagaRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  StartSagaRequest copyWith(void Function(StartSagaRequest) updates) =>
      super.copyWith((message) => updates(message as StartSagaRequest))
          as StartSagaRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static StartSagaRequest create() => StartSagaRequest._();
  @$core.override
  StartSagaRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static StartSagaRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<StartSagaRequest>(create);
  static StartSagaRequest? _defaultInstance;

  /// 実行するワークフロー名
  @$pb.TagNumber(1)
  $core.String get workflowName => $_getSZ(0);
  @$pb.TagNumber(1)
  set workflowName($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasWorkflowName() => $_has(0);
  @$pb.TagNumber(1)
  void clearWorkflowName() => $_clearField(1);

  /// 各ステップに渡す JSON ペイロード
  @$pb.TagNumber(2)
  $1.Struct get payload => $_getN(1);
  @$pb.TagNumber(2)
  set payload($1.Struct value) => $_setField(2, value);
  @$pb.TagNumber(2)
  $core.bool hasPayload() => $_has(1);
  @$pb.TagNumber(2)
  void clearPayload() => $_clearField(2);
  @$pb.TagNumber(2)
  $1.Struct ensurePayload() => $_ensure(1);

  /// 業務相関 ID（任意）
  @$pb.TagNumber(3)
  $core.String get correlationId => $_getSZ(2);
  @$pb.TagNumber(3)
  set correlationId($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasCorrelationId() => $_has(2);
  @$pb.TagNumber(3)
  void clearCorrelationId() => $_clearField(3);

  /// 呼び出し元サービス名（任意）
  @$pb.TagNumber(4)
  $core.String get initiatedBy => $_getSZ(3);
  @$pb.TagNumber(4)
  set initiatedBy($core.String value) => $_setString(3, value);
  @$pb.TagNumber(4)
  $core.bool hasInitiatedBy() => $_has(3);
  @$pb.TagNumber(4)
  void clearInitiatedBy() => $_clearField(4);
}

/// StartSagaResponse は Saga 開始レスポンス。
class StartSagaResponse extends $pb.GeneratedMessage {
  factory StartSagaResponse({
    $core.String? sagaId,
    $core.String? status,
  }) {
    final result = create();
    if (sagaId != null) result.sagaId = sagaId;
    if (status != null) result.status = status;
    return result;
  }

  StartSagaResponse._();

  factory StartSagaResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory StartSagaResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'StartSagaResponse',
      package:
          const $pb.PackageName(_omitMessageNames ? '' : 'k1s0.system.saga.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'sagaId')
    ..aOS(2, _omitFieldNames ? '' : 'status')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  StartSagaResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  StartSagaResponse copyWith(void Function(StartSagaResponse) updates) =>
      super.copyWith((message) => updates(message as StartSagaResponse))
          as StartSagaResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static StartSagaResponse create() => StartSagaResponse._();
  @$core.override
  StartSagaResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static StartSagaResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<StartSagaResponse>(create);
  static StartSagaResponse? _defaultInstance;

  /// 発行された Saga UUID
  @$pb.TagNumber(1)
  $core.String get sagaId => $_getSZ(0);
  @$pb.TagNumber(1)
  set sagaId($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasSagaId() => $_has(0);
  @$pb.TagNumber(1)
  void clearSagaId() => $_clearField(1);

  /// 初期ステータス（常に "STARTED"）
  @$pb.TagNumber(2)
  $core.String get status => $_getSZ(1);
  @$pb.TagNumber(2)
  set status($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasStatus() => $_has(1);
  @$pb.TagNumber(2)
  void clearStatus() => $_clearField(2);
}

/// GetSagaRequest は Saga 詳細取得リクエスト。
class GetSagaRequest extends $pb.GeneratedMessage {
  factory GetSagaRequest({
    $core.String? sagaId,
  }) {
    final result = create();
    if (sagaId != null) result.sagaId = sagaId;
    return result;
  }

  GetSagaRequest._();

  factory GetSagaRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory GetSagaRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'GetSagaRequest',
      package:
          const $pb.PackageName(_omitMessageNames ? '' : 'k1s0.system.saga.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'sagaId')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetSagaRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetSagaRequest copyWith(void Function(GetSagaRequest) updates) =>
      super.copyWith((message) => updates(message as GetSagaRequest))
          as GetSagaRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static GetSagaRequest create() => GetSagaRequest._();
  @$core.override
  GetSagaRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static GetSagaRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<GetSagaRequest>(create);
  static GetSagaRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get sagaId => $_getSZ(0);
  @$pb.TagNumber(1)
  set sagaId($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasSagaId() => $_has(0);
  @$pb.TagNumber(1)
  void clearSagaId() => $_clearField(1);
}

/// GetSagaResponse は Saga 詳細取得レスポンス。
class GetSagaResponse extends $pb.GeneratedMessage {
  factory GetSagaResponse({
    SagaStateProto? saga,
    $core.Iterable<SagaStepLogProto>? stepLogs,
  }) {
    final result = create();
    if (saga != null) result.saga = saga;
    if (stepLogs != null) result.stepLogs.addAll(stepLogs);
    return result;
  }

  GetSagaResponse._();

  factory GetSagaResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory GetSagaResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'GetSagaResponse',
      package:
          const $pb.PackageName(_omitMessageNames ? '' : 'k1s0.system.saga.v1'),
      createEmptyInstance: create)
    ..aOM<SagaStateProto>(1, _omitFieldNames ? '' : 'saga',
        subBuilder: SagaStateProto.create)
    ..pPM<SagaStepLogProto>(2, _omitFieldNames ? '' : 'stepLogs',
        subBuilder: SagaStepLogProto.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetSagaResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetSagaResponse copyWith(void Function(GetSagaResponse) updates) =>
      super.copyWith((message) => updates(message as GetSagaResponse))
          as GetSagaResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static GetSagaResponse create() => GetSagaResponse._();
  @$core.override
  GetSagaResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static GetSagaResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<GetSagaResponse>(create);
  static GetSagaResponse? _defaultInstance;

  @$pb.TagNumber(1)
  SagaStateProto get saga => $_getN(0);
  @$pb.TagNumber(1)
  set saga(SagaStateProto value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasSaga() => $_has(0);
  @$pb.TagNumber(1)
  void clearSaga() => $_clearField(1);
  @$pb.TagNumber(1)
  SagaStateProto ensureSaga() => $_ensure(0);

  @$pb.TagNumber(2)
  $pb.PbList<SagaStepLogProto> get stepLogs => $_getList(1);
}

/// ListSagasRequest は Saga 一覧取得リクエスト。
class ListSagasRequest extends $pb.GeneratedMessage {
  factory ListSagasRequest({
    $2.Pagination? pagination,
    $core.String? workflowName,
    $core.String? status,
    $core.String? correlationId,
  }) {
    final result = create();
    if (pagination != null) result.pagination = pagination;
    if (workflowName != null) result.workflowName = workflowName;
    if (status != null) result.status = status;
    if (correlationId != null) result.correlationId = correlationId;
    return result;
  }

  ListSagasRequest._();

  factory ListSagasRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory ListSagasRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ListSagasRequest',
      package:
          const $pb.PackageName(_omitMessageNames ? '' : 'k1s0.system.saga.v1'),
      createEmptyInstance: create)
    ..aOM<$2.Pagination>(1, _omitFieldNames ? '' : 'pagination',
        subBuilder: $2.Pagination.create)
    ..aOS(2, _omitFieldNames ? '' : 'workflowName')
    ..aOS(3, _omitFieldNames ? '' : 'status')
    ..aOS(4, _omitFieldNames ? '' : 'correlationId')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListSagasRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListSagasRequest copyWith(void Function(ListSagasRequest) updates) =>
      super.copyWith((message) => updates(message as ListSagasRequest))
          as ListSagasRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static ListSagasRequest create() => ListSagasRequest._();
  @$core.override
  ListSagasRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static ListSagasRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ListSagasRequest>(create);
  static ListSagasRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $2.Pagination get pagination => $_getN(0);
  @$pb.TagNumber(1)
  set pagination($2.Pagination value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasPagination() => $_has(0);
  @$pb.TagNumber(1)
  void clearPagination() => $_clearField(1);
  @$pb.TagNumber(1)
  $2.Pagination ensurePagination() => $_ensure(0);

  /// ワークフロー名フィルタ（任意）
  @$pb.TagNumber(2)
  $core.String get workflowName => $_getSZ(1);
  @$pb.TagNumber(2)
  set workflowName($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasWorkflowName() => $_has(1);
  @$pb.TagNumber(2)
  void clearWorkflowName() => $_clearField(2);

  /// ステータスフィルタ（任意）
  @$pb.TagNumber(3)
  $core.String get status => $_getSZ(2);
  @$pb.TagNumber(3)
  set status($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasStatus() => $_has(2);
  @$pb.TagNumber(3)
  void clearStatus() => $_clearField(3);

  /// 相関 ID フィルタ（任意）
  @$pb.TagNumber(4)
  $core.String get correlationId => $_getSZ(3);
  @$pb.TagNumber(4)
  set correlationId($core.String value) => $_setString(3, value);
  @$pb.TagNumber(4)
  $core.bool hasCorrelationId() => $_has(3);
  @$pb.TagNumber(4)
  void clearCorrelationId() => $_clearField(4);
}

/// ListSagasResponse は Saga 一覧取得レスポンス。
class ListSagasResponse extends $pb.GeneratedMessage {
  factory ListSagasResponse({
    $core.Iterable<SagaStateProto>? sagas,
    $2.PaginationResult? pagination,
  }) {
    final result = create();
    if (sagas != null) result.sagas.addAll(sagas);
    if (pagination != null) result.pagination = pagination;
    return result;
  }

  ListSagasResponse._();

  factory ListSagasResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory ListSagasResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ListSagasResponse',
      package:
          const $pb.PackageName(_omitMessageNames ? '' : 'k1s0.system.saga.v1'),
      createEmptyInstance: create)
    ..pPM<SagaStateProto>(1, _omitFieldNames ? '' : 'sagas',
        subBuilder: SagaStateProto.create)
    ..aOM<$2.PaginationResult>(2, _omitFieldNames ? '' : 'pagination',
        subBuilder: $2.PaginationResult.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListSagasResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListSagasResponse copyWith(void Function(ListSagasResponse) updates) =>
      super.copyWith((message) => updates(message as ListSagasResponse))
          as ListSagasResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static ListSagasResponse create() => ListSagasResponse._();
  @$core.override
  ListSagasResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static ListSagasResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ListSagasResponse>(create);
  static ListSagasResponse? _defaultInstance;

  @$pb.TagNumber(1)
  $pb.PbList<SagaStateProto> get sagas => $_getList(0);

  @$pb.TagNumber(2)
  $2.PaginationResult get pagination => $_getN(1);
  @$pb.TagNumber(2)
  set pagination($2.PaginationResult value) => $_setField(2, value);
  @$pb.TagNumber(2)
  $core.bool hasPagination() => $_has(1);
  @$pb.TagNumber(2)
  void clearPagination() => $_clearField(2);
  @$pb.TagNumber(2)
  $2.PaginationResult ensurePagination() => $_ensure(1);
}

/// CancelSagaRequest は Saga キャンセルリクエスト。
class CancelSagaRequest extends $pb.GeneratedMessage {
  factory CancelSagaRequest({
    $core.String? sagaId,
  }) {
    final result = create();
    if (sagaId != null) result.sagaId = sagaId;
    return result;
  }

  CancelSagaRequest._();

  factory CancelSagaRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory CancelSagaRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'CancelSagaRequest',
      package:
          const $pb.PackageName(_omitMessageNames ? '' : 'k1s0.system.saga.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'sagaId')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  CancelSagaRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  CancelSagaRequest copyWith(void Function(CancelSagaRequest) updates) =>
      super.copyWith((message) => updates(message as CancelSagaRequest))
          as CancelSagaRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static CancelSagaRequest create() => CancelSagaRequest._();
  @$core.override
  CancelSagaRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static CancelSagaRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<CancelSagaRequest>(create);
  static CancelSagaRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get sagaId => $_getSZ(0);
  @$pb.TagNumber(1)
  set sagaId($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasSagaId() => $_has(0);
  @$pb.TagNumber(1)
  void clearSagaId() => $_clearField(1);
}

/// CancelSagaResponse は Saga キャンセルレスポンス。
class CancelSagaResponse extends $pb.GeneratedMessage {
  factory CancelSagaResponse({
    $core.bool? success,
    $core.String? message,
  }) {
    final result = create();
    if (success != null) result.success = success;
    if (message != null) result.message = message;
    return result;
  }

  CancelSagaResponse._();

  factory CancelSagaResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory CancelSagaResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'CancelSagaResponse',
      package:
          const $pb.PackageName(_omitMessageNames ? '' : 'k1s0.system.saga.v1'),
      createEmptyInstance: create)
    ..aOB(1, _omitFieldNames ? '' : 'success')
    ..aOS(2, _omitFieldNames ? '' : 'message')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  CancelSagaResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  CancelSagaResponse copyWith(void Function(CancelSagaResponse) updates) =>
      super.copyWith((message) => updates(message as CancelSagaResponse))
          as CancelSagaResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static CancelSagaResponse create() => CancelSagaResponse._();
  @$core.override
  CancelSagaResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static CancelSagaResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<CancelSagaResponse>(create);
  static CancelSagaResponse? _defaultInstance;

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

/// CompensateSagaRequest は Saga 補償実行リクエスト。
class CompensateSagaRequest extends $pb.GeneratedMessage {
  factory CompensateSagaRequest({
    $core.String? sagaId,
  }) {
    final result = create();
    if (sagaId != null) result.sagaId = sagaId;
    return result;
  }

  CompensateSagaRequest._();

  factory CompensateSagaRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory CompensateSagaRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'CompensateSagaRequest',
      package:
          const $pb.PackageName(_omitMessageNames ? '' : 'k1s0.system.saga.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'sagaId')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  CompensateSagaRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  CompensateSagaRequest copyWith(
          void Function(CompensateSagaRequest) updates) =>
      super.copyWith((message) => updates(message as CompensateSagaRequest))
          as CompensateSagaRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static CompensateSagaRequest create() => CompensateSagaRequest._();
  @$core.override
  CompensateSagaRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static CompensateSagaRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<CompensateSagaRequest>(create);
  static CompensateSagaRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get sagaId => $_getSZ(0);
  @$pb.TagNumber(1)
  set sagaId($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasSagaId() => $_has(0);
  @$pb.TagNumber(1)
  void clearSagaId() => $_clearField(1);
}

/// CompensateSagaResponse は Saga 補償実行レスポンス。
class CompensateSagaResponse extends $pb.GeneratedMessage {
  factory CompensateSagaResponse({
    $core.bool? success,
    $core.String? status,
    $core.String? message,
    $core.String? sagaId,
  }) {
    final result = create();
    if (success != null) result.success = success;
    if (status != null) result.status = status;
    if (message != null) result.message = message;
    if (sagaId != null) result.sagaId = sagaId;
    return result;
  }

  CompensateSagaResponse._();

  factory CompensateSagaResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory CompensateSagaResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'CompensateSagaResponse',
      package:
          const $pb.PackageName(_omitMessageNames ? '' : 'k1s0.system.saga.v1'),
      createEmptyInstance: create)
    ..aOB(1, _omitFieldNames ? '' : 'success')
    ..aOS(2, _omitFieldNames ? '' : 'status')
    ..aOS(3, _omitFieldNames ? '' : 'message')
    ..aOS(4, _omitFieldNames ? '' : 'sagaId')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  CompensateSagaResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  CompensateSagaResponse copyWith(
          void Function(CompensateSagaResponse) updates) =>
      super.copyWith((message) => updates(message as CompensateSagaResponse))
          as CompensateSagaResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static CompensateSagaResponse create() => CompensateSagaResponse._();
  @$core.override
  CompensateSagaResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static CompensateSagaResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<CompensateSagaResponse>(create);
  static CompensateSagaResponse? _defaultInstance;

  @$pb.TagNumber(1)
  $core.bool get success => $_getBF(0);
  @$pb.TagNumber(1)
  set success($core.bool value) => $_setBool(0, value);
  @$pb.TagNumber(1)
  $core.bool hasSuccess() => $_has(0);
  @$pb.TagNumber(1)
  void clearSuccess() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get status => $_getSZ(1);
  @$pb.TagNumber(2)
  set status($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasStatus() => $_has(1);
  @$pb.TagNumber(2)
  void clearStatus() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.String get message => $_getSZ(2);
  @$pb.TagNumber(3)
  set message($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasMessage() => $_has(2);
  @$pb.TagNumber(3)
  void clearMessage() => $_clearField(3);

  @$pb.TagNumber(4)
  $core.String get sagaId => $_getSZ(3);
  @$pb.TagNumber(4)
  set sagaId($core.String value) => $_setString(3, value);
  @$pb.TagNumber(4)
  $core.bool hasSagaId() => $_has(3);
  @$pb.TagNumber(4)
  void clearSagaId() => $_clearField(4);
}

/// RegisterWorkflowRequest はワークフロー登録リクエスト。
class RegisterWorkflowRequest extends $pb.GeneratedMessage {
  factory RegisterWorkflowRequest({
    $core.String? workflowYaml,
  }) {
    final result = create();
    if (workflowYaml != null) result.workflowYaml = workflowYaml;
    return result;
  }

  RegisterWorkflowRequest._();

  factory RegisterWorkflowRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory RegisterWorkflowRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'RegisterWorkflowRequest',
      package:
          const $pb.PackageName(_omitMessageNames ? '' : 'k1s0.system.saga.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'workflowYaml')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  RegisterWorkflowRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  RegisterWorkflowRequest copyWith(
          void Function(RegisterWorkflowRequest) updates) =>
      super.copyWith((message) => updates(message as RegisterWorkflowRequest))
          as RegisterWorkflowRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static RegisterWorkflowRequest create() => RegisterWorkflowRequest._();
  @$core.override
  RegisterWorkflowRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static RegisterWorkflowRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<RegisterWorkflowRequest>(create);
  static RegisterWorkflowRequest? _defaultInstance;

  /// YAML 形式のワークフロー定義文字列
  @$pb.TagNumber(1)
  $core.String get workflowYaml => $_getSZ(0);
  @$pb.TagNumber(1)
  set workflowYaml($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasWorkflowYaml() => $_has(0);
  @$pb.TagNumber(1)
  void clearWorkflowYaml() => $_clearField(1);
}

/// RegisterWorkflowResponse はワークフロー登録レスポンス。
class RegisterWorkflowResponse extends $pb.GeneratedMessage {
  factory RegisterWorkflowResponse({
    $core.String? name,
    $core.int? stepCount,
  }) {
    final result = create();
    if (name != null) result.name = name;
    if (stepCount != null) result.stepCount = stepCount;
    return result;
  }

  RegisterWorkflowResponse._();

  factory RegisterWorkflowResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory RegisterWorkflowResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'RegisterWorkflowResponse',
      package:
          const $pb.PackageName(_omitMessageNames ? '' : 'k1s0.system.saga.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'name')
    ..aI(2, _omitFieldNames ? '' : 'stepCount')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  RegisterWorkflowResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  RegisterWorkflowResponse copyWith(
          void Function(RegisterWorkflowResponse) updates) =>
      super.copyWith((message) => updates(message as RegisterWorkflowResponse))
          as RegisterWorkflowResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static RegisterWorkflowResponse create() => RegisterWorkflowResponse._();
  @$core.override
  RegisterWorkflowResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static RegisterWorkflowResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<RegisterWorkflowResponse>(create);
  static RegisterWorkflowResponse? _defaultInstance;

  /// 登録されたワークフロー名
  @$pb.TagNumber(1)
  $core.String get name => $_getSZ(0);
  @$pb.TagNumber(1)
  set name($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasName() => $_has(0);
  @$pb.TagNumber(1)
  void clearName() => $_clearField(1);

  /// ステップ数
  @$pb.TagNumber(2)
  $core.int get stepCount => $_getIZ(1);
  @$pb.TagNumber(2)
  set stepCount($core.int value) => $_setSignedInt32(1, value);
  @$pb.TagNumber(2)
  $core.bool hasStepCount() => $_has(1);
  @$pb.TagNumber(2)
  void clearStepCount() => $_clearField(2);
}

/// ListWorkflowsRequest はワークフロー一覧取得リクエスト（フィールドなし）。
class ListWorkflowsRequest extends $pb.GeneratedMessage {
  factory ListWorkflowsRequest() => create();

  ListWorkflowsRequest._();

  factory ListWorkflowsRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory ListWorkflowsRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ListWorkflowsRequest',
      package:
          const $pb.PackageName(_omitMessageNames ? '' : 'k1s0.system.saga.v1'),
      createEmptyInstance: create)
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
}

/// ListWorkflowsResponse はワークフロー一覧取得レスポンス。
class ListWorkflowsResponse extends $pb.GeneratedMessage {
  factory ListWorkflowsResponse({
    $core.Iterable<WorkflowSummary>? workflows,
  }) {
    final result = create();
    if (workflows != null) result.workflows.addAll(workflows);
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
      package:
          const $pb.PackageName(_omitMessageNames ? '' : 'k1s0.system.saga.v1'),
      createEmptyInstance: create)
    ..pPM<WorkflowSummary>(1, _omitFieldNames ? '' : 'workflows',
        subBuilder: WorkflowSummary.create)
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
  $pb.PbList<WorkflowSummary> get workflows => $_getList(0);
}

const $core.bool _omitFieldNames =
    $core.bool.fromEnvironment('protobuf.omit_field_names');
const $core.bool _omitMessageNames =
    $core.bool.fromEnvironment('protobuf.omit_message_names');
