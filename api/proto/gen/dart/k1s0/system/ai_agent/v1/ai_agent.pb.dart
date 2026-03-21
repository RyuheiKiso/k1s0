// This is a generated file - do not edit.
//
// Generated from k1s0/system/ai_agent/v1/ai_agent.proto.

// @dart = 3.3

// ignore_for_file: annotate_overrides, camel_case_types, comment_references
// ignore_for_file: constant_identifier_names
// ignore_for_file: curly_braces_in_flow_control_structures
// ignore_for_file: deprecated_member_use_from_same_package, library_prefixes
// ignore_for_file: non_constant_identifier_names, prefer_relative_imports

import 'dart:core' as $core;

import 'package:protobuf/protobuf.dart' as $pb;

export 'package:protobuf/protobuf.dart' show GeneratedMessageGenericExtensions;

/// エージェント実行リクエスト: エージェントへの入力とコンテキストを含む
class ExecuteRequest extends $pb.GeneratedMessage {
  factory ExecuteRequest({
    $core.String? agentId,
    $core.String? input,
    $core.String? sessionId,
    $core.String? tenantId,
    $core.Iterable<$core.MapEntry<$core.String, $core.String>>? context,
  }) {
    final result = create();
    if (agentId != null) result.agentId = agentId;
    if (input != null) result.input = input;
    if (sessionId != null) result.sessionId = sessionId;
    if (tenantId != null) result.tenantId = tenantId;
    if (context != null) result.context.addEntries(context);
    return result;
  }

  ExecuteRequest._();

  factory ExecuteRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory ExecuteRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ExecuteRequest',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.aiagent.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'agentId')
    ..aOS(2, _omitFieldNames ? '' : 'input')
    ..aOS(3, _omitFieldNames ? '' : 'sessionId')
    ..aOS(4, _omitFieldNames ? '' : 'tenantId')
    ..m<$core.String, $core.String>(5, _omitFieldNames ? '' : 'context',
        entryClassName: 'ExecuteRequest.ContextEntry',
        keyFieldType: $pb.PbFieldType.OS,
        valueFieldType: $pb.PbFieldType.OS,
        packageName: const $pb.PackageName('k1s0.system.aiagent.v1'))
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ExecuteRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ExecuteRequest copyWith(void Function(ExecuteRequest) updates) =>
      super.copyWith((message) => updates(message as ExecuteRequest))
          as ExecuteRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static ExecuteRequest create() => ExecuteRequest._();
  @$core.override
  ExecuteRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static ExecuteRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ExecuteRequest>(create);
  static ExecuteRequest? _defaultInstance;

  /// 実行するエージェントの識別子
  @$pb.TagNumber(1)
  $core.String get agentId => $_getSZ(0);
  @$pb.TagNumber(1)
  set agentId($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasAgentId() => $_has(0);
  @$pb.TagNumber(1)
  void clearAgentId() => $_clearField(1);

  /// エージェントへの入力テキスト
  @$pb.TagNumber(2)
  $core.String get input => $_getSZ(1);
  @$pb.TagNumber(2)
  set input($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasInput() => $_has(1);
  @$pb.TagNumber(2)
  void clearInput() => $_clearField(2);

  /// セッション識別子（会話の継続に使用）
  @$pb.TagNumber(3)
  $core.String get sessionId => $_getSZ(2);
  @$pb.TagNumber(3)
  set sessionId($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasSessionId() => $_has(2);
  @$pb.TagNumber(3)
  void clearSessionId() => $_clearField(3);

  /// リクエスト元のテナント識別子
  @$pb.TagNumber(4)
  $core.String get tenantId => $_getSZ(3);
  @$pb.TagNumber(4)
  set tenantId($core.String value) => $_setString(3, value);
  @$pb.TagNumber(4)
  $core.bool hasTenantId() => $_has(3);
  @$pb.TagNumber(4)
  void clearTenantId() => $_clearField(4);

  /// エージェント実行に渡す追加コンテキスト
  @$pb.TagNumber(5)
  $pb.PbMap<$core.String, $core.String> get context => $_getMap(4);
}

/// エージェント実行レスポンス: 実行結果とステップ情報を含む
class ExecuteResponse extends $pb.GeneratedMessage {
  factory ExecuteResponse({
    $core.String? executionId,
    $core.String? status,
    $core.String? output,
    $core.Iterable<ExecutionStep>? steps,
  }) {
    final result = create();
    if (executionId != null) result.executionId = executionId;
    if (status != null) result.status = status;
    if (output != null) result.output = output;
    if (steps != null) result.steps.addAll(steps);
    return result;
  }

  ExecuteResponse._();

  factory ExecuteResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory ExecuteResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ExecuteResponse',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.aiagent.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'executionId')
    ..aOS(2, _omitFieldNames ? '' : 'status')
    ..aOS(3, _omitFieldNames ? '' : 'output')
    ..pPM<ExecutionStep>(4, _omitFieldNames ? '' : 'steps',
        subBuilder: ExecutionStep.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ExecuteResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ExecuteResponse copyWith(void Function(ExecuteResponse) updates) =>
      super.copyWith((message) => updates(message as ExecuteResponse))
          as ExecuteResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static ExecuteResponse create() => ExecuteResponse._();
  @$core.override
  ExecuteResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static ExecuteResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ExecuteResponse>(create);
  static ExecuteResponse? _defaultInstance;

  /// 実行の一意識別子
  @$pb.TagNumber(1)
  $core.String get executionId => $_getSZ(0);
  @$pb.TagNumber(1)
  set executionId($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasExecutionId() => $_has(0);
  @$pb.TagNumber(1)
  void clearExecutionId() => $_clearField(1);

  /// 実行状態（running / completed / failed / cancelled）
  @$pb.TagNumber(2)
  $core.String get status => $_getSZ(1);
  @$pb.TagNumber(2)
  set status($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasStatus() => $_has(1);
  @$pb.TagNumber(2)
  void clearStatus() => $_clearField(2);

  /// エージェントの最終出力
  @$pb.TagNumber(3)
  $core.String get output => $_getSZ(2);
  @$pb.TagNumber(3)
  set output($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasOutput() => $_has(2);
  @$pb.TagNumber(3)
  void clearOutput() => $_clearField(3);

  /// 実行されたステップのリスト
  @$pb.TagNumber(4)
  $pb.PbList<ExecutionStep> get steps => $_getList(3);
}

/// ストリーミング実行リクエスト: ExecuteStream RPC 専用のリクエストメッセージ
class ExecuteStreamRequest extends $pb.GeneratedMessage {
  factory ExecuteStreamRequest({
    $core.String? agentId,
    $core.String? input,
    $core.String? sessionId,
    $core.String? tenantId,
    $core.Iterable<$core.MapEntry<$core.String, $core.String>>? context,
  }) {
    final result = create();
    if (agentId != null) result.agentId = agentId;
    if (input != null) result.input = input;
    if (sessionId != null) result.sessionId = sessionId;
    if (tenantId != null) result.tenantId = tenantId;
    if (context != null) result.context.addEntries(context);
    return result;
  }

  ExecuteStreamRequest._();

  factory ExecuteStreamRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory ExecuteStreamRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ExecuteStreamRequest',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.aiagent.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'agentId')
    ..aOS(2, _omitFieldNames ? '' : 'input')
    ..aOS(3, _omitFieldNames ? '' : 'sessionId')
    ..aOS(4, _omitFieldNames ? '' : 'tenantId')
    ..m<$core.String, $core.String>(5, _omitFieldNames ? '' : 'context',
        entryClassName: 'ExecuteStreamRequest.ContextEntry',
        keyFieldType: $pb.PbFieldType.OS,
        valueFieldType: $pb.PbFieldType.OS,
        packageName: const $pb.PackageName('k1s0.system.aiagent.v1'))
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ExecuteStreamRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ExecuteStreamRequest copyWith(void Function(ExecuteStreamRequest) updates) =>
      super.copyWith((message) => updates(message as ExecuteStreamRequest))
          as ExecuteStreamRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static ExecuteStreamRequest create() => ExecuteStreamRequest._();
  @$core.override
  ExecuteStreamRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static ExecuteStreamRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ExecuteStreamRequest>(create);
  static ExecuteStreamRequest? _defaultInstance;

  /// 実行するエージェントの識別子
  @$pb.TagNumber(1)
  $core.String get agentId => $_getSZ(0);
  @$pb.TagNumber(1)
  set agentId($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasAgentId() => $_has(0);
  @$pb.TagNumber(1)
  void clearAgentId() => $_clearField(1);

  /// エージェントへの入力テキスト
  @$pb.TagNumber(2)
  $core.String get input => $_getSZ(1);
  @$pb.TagNumber(2)
  set input($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasInput() => $_has(1);
  @$pb.TagNumber(2)
  void clearInput() => $_clearField(2);

  /// セッション識別子（会話の継続に使用）
  @$pb.TagNumber(3)
  $core.String get sessionId => $_getSZ(2);
  @$pb.TagNumber(3)
  set sessionId($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasSessionId() => $_has(2);
  @$pb.TagNumber(3)
  void clearSessionId() => $_clearField(3);

  /// リクエスト元のテナント識別子
  @$pb.TagNumber(4)
  $core.String get tenantId => $_getSZ(3);
  @$pb.TagNumber(4)
  set tenantId($core.String value) => $_setString(3, value);
  @$pb.TagNumber(4)
  $core.bool hasTenantId() => $_has(3);
  @$pb.TagNumber(4)
  void clearTenantId() => $_clearField(4);

  /// エージェント実行に渡す追加コンテキスト
  @$pb.TagNumber(5)
  $pb.PbMap<$core.String, $core.String> get context => $_getMap(4);
}

/// ストリーミング実行レスポンス: ストリーミング中に送信される個別イベント
class ExecuteStreamResponse extends $pb.GeneratedMessage {
  factory ExecuteStreamResponse({
    $core.String? executionId,
    $core.String? eventType,
    $core.String? data,
    $core.int? stepIndex,
  }) {
    final result = create();
    if (executionId != null) result.executionId = executionId;
    if (eventType != null) result.eventType = eventType;
    if (data != null) result.data = data;
    if (stepIndex != null) result.stepIndex = stepIndex;
    return result;
  }

  ExecuteStreamResponse._();

  factory ExecuteStreamResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory ExecuteStreamResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ExecuteStreamResponse',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.aiagent.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'executionId')
    ..aOS(2, _omitFieldNames ? '' : 'eventType')
    ..aOS(3, _omitFieldNames ? '' : 'data')
    ..aI(4, _omitFieldNames ? '' : 'stepIndex')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ExecuteStreamResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ExecuteStreamResponse copyWith(
          void Function(ExecuteStreamResponse) updates) =>
      super.copyWith((message) => updates(message as ExecuteStreamResponse))
          as ExecuteStreamResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static ExecuteStreamResponse create() => ExecuteStreamResponse._();
  @$core.override
  ExecuteStreamResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static ExecuteStreamResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ExecuteStreamResponse>(create);
  static ExecuteStreamResponse? _defaultInstance;

  /// 実行の一意識別子
  @$pb.TagNumber(1)
  $core.String get executionId => $_getSZ(0);
  @$pb.TagNumber(1)
  set executionId($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasExecutionId() => $_has(0);
  @$pb.TagNumber(1)
  void clearExecutionId() => $_clearField(1);

  /// イベント種別（step_start / step_end / tool_call / output / error）
  @$pb.TagNumber(2)
  $core.String get eventType => $_getSZ(1);
  @$pb.TagNumber(2)
  set eventType($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasEventType() => $_has(1);
  @$pb.TagNumber(2)
  void clearEventType() => $_clearField(2);

  /// イベントデータ（JSON 形式）
  @$pb.TagNumber(3)
  $core.String get data => $_getSZ(2);
  @$pb.TagNumber(3)
  set data($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasData() => $_has(2);
  @$pb.TagNumber(3)
  void clearData() => $_clearField(3);

  /// 関連するステップのインデックス
  @$pb.TagNumber(4)
  $core.int get stepIndex => $_getIZ(3);
  @$pb.TagNumber(4)
  set stepIndex($core.int value) => $_setSignedInt32(3, value);
  @$pb.TagNumber(4)
  $core.bool hasStepIndex() => $_has(3);
  @$pb.TagNumber(4)
  void clearStepIndex() => $_clearField(4);
}

/// 実行ステップ: エージェントが実行した個別ステップの詳細
class ExecutionStep extends $pb.GeneratedMessage {
  factory ExecutionStep({
    $core.int? index,
    $core.String? stepType,
    $core.String? input,
    $core.String? output,
    $core.String? toolName,
    $core.String? status,
  }) {
    final result = create();
    if (index != null) result.index = index;
    if (stepType != null) result.stepType = stepType;
    if (input != null) result.input = input;
    if (output != null) result.output = output;
    if (toolName != null) result.toolName = toolName;
    if (status != null) result.status = status;
    return result;
  }

  ExecutionStep._();

  factory ExecutionStep.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory ExecutionStep.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ExecutionStep',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.aiagent.v1'),
      createEmptyInstance: create)
    ..aI(1, _omitFieldNames ? '' : 'index')
    ..aOS(2, _omitFieldNames ? '' : 'stepType')
    ..aOS(3, _omitFieldNames ? '' : 'input')
    ..aOS(4, _omitFieldNames ? '' : 'output')
    ..aOS(5, _omitFieldNames ? '' : 'toolName')
    ..aOS(6, _omitFieldNames ? '' : 'status')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ExecutionStep clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ExecutionStep copyWith(void Function(ExecutionStep) updates) =>
      super.copyWith((message) => updates(message as ExecutionStep))
          as ExecutionStep;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static ExecutionStep create() => ExecutionStep._();
  @$core.override
  ExecutionStep createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static ExecutionStep getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ExecutionStep>(create);
  static ExecutionStep? _defaultInstance;

  /// ステップのインデックス番号
  @$pb.TagNumber(1)
  $core.int get index => $_getIZ(0);
  @$pb.TagNumber(1)
  set index($core.int value) => $_setSignedInt32(0, value);
  @$pb.TagNumber(1)
  $core.bool hasIndex() => $_has(0);
  @$pb.TagNumber(1)
  void clearIndex() => $_clearField(1);

  /// ステップ種別（thinking / tool_call / output）
  @$pb.TagNumber(2)
  $core.String get stepType => $_getSZ(1);
  @$pb.TagNumber(2)
  set stepType($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasStepType() => $_has(1);
  @$pb.TagNumber(2)
  void clearStepType() => $_clearField(2);

  /// ステップへの入力
  @$pb.TagNumber(3)
  $core.String get input => $_getSZ(2);
  @$pb.TagNumber(3)
  set input($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasInput() => $_has(2);
  @$pb.TagNumber(3)
  void clearInput() => $_clearField(3);

  /// ステップの出力
  @$pb.TagNumber(4)
  $core.String get output => $_getSZ(3);
  @$pb.TagNumber(4)
  set output($core.String value) => $_setString(3, value);
  @$pb.TagNumber(4)
  $core.bool hasOutput() => $_has(3);
  @$pb.TagNumber(4)
  void clearOutput() => $_clearField(4);

  /// 使用されたツール名（tool_call の場合）
  @$pb.TagNumber(5)
  $core.String get toolName => $_getSZ(4);
  @$pb.TagNumber(5)
  set toolName($core.String value) => $_setString(4, value);
  @$pb.TagNumber(5)
  $core.bool hasToolName() => $_has(4);
  @$pb.TagNumber(5)
  void clearToolName() => $_clearField(5);

  /// ステップの状態（pending / running / completed / failed）
  @$pb.TagNumber(6)
  $core.String get status => $_getSZ(5);
  @$pb.TagNumber(6)
  set status($core.String value) => $_setString(5, value);
  @$pb.TagNumber(6)
  $core.bool hasStatus() => $_has(5);
  @$pb.TagNumber(6)
  void clearStatus() => $_clearField(6);
}

/// 実行キャンセルリクエスト: 実行中のエージェントを停止する
class CancelExecutionRequest extends $pb.GeneratedMessage {
  factory CancelExecutionRequest({
    $core.String? executionId,
  }) {
    final result = create();
    if (executionId != null) result.executionId = executionId;
    return result;
  }

  CancelExecutionRequest._();

  factory CancelExecutionRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory CancelExecutionRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'CancelExecutionRequest',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.aiagent.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'executionId')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  CancelExecutionRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  CancelExecutionRequest copyWith(
          void Function(CancelExecutionRequest) updates) =>
      super.copyWith((message) => updates(message as CancelExecutionRequest))
          as CancelExecutionRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static CancelExecutionRequest create() => CancelExecutionRequest._();
  @$core.override
  CancelExecutionRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static CancelExecutionRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<CancelExecutionRequest>(create);
  static CancelExecutionRequest? _defaultInstance;

  /// キャンセルする実行の識別子
  @$pb.TagNumber(1)
  $core.String get executionId => $_getSZ(0);
  @$pb.TagNumber(1)
  set executionId($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasExecutionId() => $_has(0);
  @$pb.TagNumber(1)
  void clearExecutionId() => $_clearField(1);
}

/// 実行キャンセルレスポンス: キャンセル結果
class CancelExecutionResponse extends $pb.GeneratedMessage {
  factory CancelExecutionResponse({
    $core.bool? success,
  }) {
    final result = create();
    if (success != null) result.success = success;
    return result;
  }

  CancelExecutionResponse._();

  factory CancelExecutionResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory CancelExecutionResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'CancelExecutionResponse',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.aiagent.v1'),
      createEmptyInstance: create)
    ..aOB(1, _omitFieldNames ? '' : 'success')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  CancelExecutionResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  CancelExecutionResponse copyWith(
          void Function(CancelExecutionResponse) updates) =>
      super.copyWith((message) => updates(message as CancelExecutionResponse))
          as CancelExecutionResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static CancelExecutionResponse create() => CancelExecutionResponse._();
  @$core.override
  CancelExecutionResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static CancelExecutionResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<CancelExecutionResponse>(create);
  static CancelExecutionResponse? _defaultInstance;

  /// キャンセルが成功したかどうか
  @$pb.TagNumber(1)
  $core.bool get success => $_getBF(0);
  @$pb.TagNumber(1)
  set success($core.bool value) => $_setBool(0, value);
  @$pb.TagNumber(1)
  $core.bool hasSuccess() => $_has(0);
  @$pb.TagNumber(1)
  void clearSuccess() => $_clearField(1);
}

/// ステップレビューリクエスト: 人間によるステップの承認・却下
class ReviewStepRequest extends $pb.GeneratedMessage {
  factory ReviewStepRequest({
    $core.String? executionId,
    $core.int? stepIndex,
    $core.bool? approved,
    $core.String? feedback,
  }) {
    final result = create();
    if (executionId != null) result.executionId = executionId;
    if (stepIndex != null) result.stepIndex = stepIndex;
    if (approved != null) result.approved = approved;
    if (feedback != null) result.feedback = feedback;
    return result;
  }

  ReviewStepRequest._();

  factory ReviewStepRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory ReviewStepRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ReviewStepRequest',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.aiagent.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'executionId')
    ..aI(2, _omitFieldNames ? '' : 'stepIndex')
    ..aOB(3, _omitFieldNames ? '' : 'approved')
    ..aOS(4, _omitFieldNames ? '' : 'feedback')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ReviewStepRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ReviewStepRequest copyWith(void Function(ReviewStepRequest) updates) =>
      super.copyWith((message) => updates(message as ReviewStepRequest))
          as ReviewStepRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static ReviewStepRequest create() => ReviewStepRequest._();
  @$core.override
  ReviewStepRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static ReviewStepRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ReviewStepRequest>(create);
  static ReviewStepRequest? _defaultInstance;

  /// レビュー対象の実行識別子
  @$pb.TagNumber(1)
  $core.String get executionId => $_getSZ(0);
  @$pb.TagNumber(1)
  set executionId($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasExecutionId() => $_has(0);
  @$pb.TagNumber(1)
  void clearExecutionId() => $_clearField(1);

  /// レビュー対象のステップインデックス
  @$pb.TagNumber(2)
  $core.int get stepIndex => $_getIZ(1);
  @$pb.TagNumber(2)
  set stepIndex($core.int value) => $_setSignedInt32(1, value);
  @$pb.TagNumber(2)
  $core.bool hasStepIndex() => $_has(1);
  @$pb.TagNumber(2)
  void clearStepIndex() => $_clearField(2);

  /// ステップを承認するかどうか
  @$pb.TagNumber(3)
  $core.bool get approved => $_getBF(2);
  @$pb.TagNumber(3)
  set approved($core.bool value) => $_setBool(2, value);
  @$pb.TagNumber(3)
  $core.bool hasApproved() => $_has(2);
  @$pb.TagNumber(3)
  void clearApproved() => $_clearField(3);

  /// レビューコメント
  @$pb.TagNumber(4)
  $core.String get feedback => $_getSZ(3);
  @$pb.TagNumber(4)
  set feedback($core.String value) => $_setString(3, value);
  @$pb.TagNumber(4)
  $core.bool hasFeedback() => $_has(3);
  @$pb.TagNumber(4)
  void clearFeedback() => $_clearField(4);
}

/// ステップレビューレスポンス: レビュー後の実行状態
class ReviewStepResponse extends $pb.GeneratedMessage {
  factory ReviewStepResponse({
    $core.String? executionId,
    $core.bool? resumed,
  }) {
    final result = create();
    if (executionId != null) result.executionId = executionId;
    if (resumed != null) result.resumed = resumed;
    return result;
  }

  ReviewStepResponse._();

  factory ReviewStepResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory ReviewStepResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ReviewStepResponse',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.aiagent.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'executionId')
    ..aOB(2, _omitFieldNames ? '' : 'resumed')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ReviewStepResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ReviewStepResponse copyWith(void Function(ReviewStepResponse) updates) =>
      super.copyWith((message) => updates(message as ReviewStepResponse))
          as ReviewStepResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static ReviewStepResponse create() => ReviewStepResponse._();
  @$core.override
  ReviewStepResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static ReviewStepResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ReviewStepResponse>(create);
  static ReviewStepResponse? _defaultInstance;

  /// 実行の識別子
  @$pb.TagNumber(1)
  $core.String get executionId => $_getSZ(0);
  @$pb.TagNumber(1)
  set executionId($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasExecutionId() => $_has(0);
  @$pb.TagNumber(1)
  void clearExecutionId() => $_clearField(1);

  /// 実行が再開されたかどうか
  @$pb.TagNumber(2)
  $core.bool get resumed => $_getBF(1);
  @$pb.TagNumber(2)
  set resumed($core.bool value) => $_setBool(1, value);
  @$pb.TagNumber(2)
  $core.bool hasResumed() => $_has(1);
  @$pb.TagNumber(2)
  void clearResumed() => $_clearField(2);
}

const $core.bool _omitFieldNames =
    $core.bool.fromEnvironment('protobuf.omit_field_names');
const $core.bool _omitMessageNames =
    $core.bool.fromEnvironment('protobuf.omit_message_names');
