// This is a generated file - do not edit.
//
// Generated from k1s0/system/ai_gateway/v1/ai_gateway.proto.

// @dart = 3.3

// ignore_for_file: annotate_overrides, camel_case_types, comment_references
// ignore_for_file: constant_identifier_names
// ignore_for_file: curly_braces_in_flow_control_structures
// ignore_for_file: deprecated_member_use_from_same_package, library_prefixes
// ignore_for_file: non_constant_identifier_names, prefer_relative_imports

import 'dart:core' as $core;

import 'package:fixnum/fixnum.dart' as $fixnum;
import 'package:protobuf/protobuf.dart' as $pb;

export 'package:protobuf/protobuf.dart' show GeneratedMessageGenericExtensions;

/// 補完リクエスト: AI モデルに送信するメッセージと設定を含む
class CompleteRequest extends $pb.GeneratedMessage {
  factory CompleteRequest({
    $core.String? model,
    $core.Iterable<Message>? messages,
    $core.int? maxTokens,
    $core.double? temperature,
    $core.bool? stream,
    $core.String? tenantId,
  }) {
    final result = create();
    if (model != null) result.model = model;
    if (messages != null) result.messages.addAll(messages);
    if (maxTokens != null) result.maxTokens = maxTokens;
    if (temperature != null) result.temperature = temperature;
    if (stream != null) result.stream = stream;
    if (tenantId != null) result.tenantId = tenantId;
    return result;
  }

  CompleteRequest._();

  factory CompleteRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory CompleteRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'CompleteRequest',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.aigateway.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'model')
    ..pPM<Message>(2, _omitFieldNames ? '' : 'messages',
        subBuilder: Message.create)
    ..aI(3, _omitFieldNames ? '' : 'maxTokens')
    ..aD(4, _omitFieldNames ? '' : 'temperature', fieldType: $pb.PbFieldType.OF)
    ..aOB(5, _omitFieldNames ? '' : 'stream')
    ..aOS(6, _omitFieldNames ? '' : 'tenantId')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  CompleteRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  CompleteRequest copyWith(void Function(CompleteRequest) updates) =>
      super.copyWith((message) => updates(message as CompleteRequest))
          as CompleteRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static CompleteRequest create() => CompleteRequest._();
  @$core.override
  CompleteRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static CompleteRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<CompleteRequest>(create);
  static CompleteRequest? _defaultInstance;

  /// 使用する AI モデルの識別子
  @$pb.TagNumber(1)
  $core.String get model => $_getSZ(0);
  @$pb.TagNumber(1)
  set model($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasModel() => $_has(0);
  @$pb.TagNumber(1)
  void clearModel() => $_clearField(1);

  /// 補完に使用するメッセージ履歴
  @$pb.TagNumber(2)
  $pb.PbList<Message> get messages => $_getList(1);

  /// 生成する最大トークン数
  @$pb.TagNumber(3)
  $core.int get maxTokens => $_getIZ(2);
  @$pb.TagNumber(3)
  set maxTokens($core.int value) => $_setSignedInt32(2, value);
  @$pb.TagNumber(3)
  $core.bool hasMaxTokens() => $_has(2);
  @$pb.TagNumber(3)
  void clearMaxTokens() => $_clearField(3);

  /// 生成のランダム性を制御する温度パラメータ（0.0〜2.0）
  @$pb.TagNumber(4)
  $core.double get temperature => $_getN(3);
  @$pb.TagNumber(4)
  set temperature($core.double value) => $_setFloat(3, value);
  @$pb.TagNumber(4)
  $core.bool hasTemperature() => $_has(3);
  @$pb.TagNumber(4)
  void clearTemperature() => $_clearField(4);

  /// ストリーミングモードを有効にするかどうか
  @$pb.TagNumber(5)
  $core.bool get stream => $_getBF(4);
  @$pb.TagNumber(5)
  set stream($core.bool value) => $_setBool(4, value);
  @$pb.TagNumber(5)
  $core.bool hasStream() => $_has(4);
  @$pb.TagNumber(5)
  void clearStream() => $_clearField(5);

  /// リクエスト元のテナント識別子
  @$pb.TagNumber(6)
  $core.String get tenantId => $_getSZ(5);
  @$pb.TagNumber(6)
  set tenantId($core.String value) => $_setString(5, value);
  @$pb.TagNumber(6)
  $core.bool hasTenantId() => $_has(5);
  @$pb.TagNumber(6)
  void clearTenantId() => $_clearField(6);
}

/// メッセージ: 会話内の単一メッセージ（ロールと内容）
class Message extends $pb.GeneratedMessage {
  factory Message({
    $core.String? role,
    $core.String? content,
  }) {
    final result = create();
    if (role != null) result.role = role;
    if (content != null) result.content = content;
    return result;
  }

  Message._();

  factory Message.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory Message.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'Message',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.aigateway.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'role')
    ..aOS(2, _omitFieldNames ? '' : 'content')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  Message clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  Message copyWith(void Function(Message) updates) =>
      super.copyWith((message) => updates(message as Message)) as Message;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static Message create() => Message._();
  @$core.override
  Message createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static Message getDefault() =>
      _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<Message>(create);
  static Message? _defaultInstance;

  /// メッセージの役割（system / user / assistant）
  @$pb.TagNumber(1)
  $core.String get role => $_getSZ(0);
  @$pb.TagNumber(1)
  set role($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasRole() => $_has(0);
  @$pb.TagNumber(1)
  void clearRole() => $_clearField(1);

  /// メッセージの本文
  @$pb.TagNumber(2)
  $core.String get content => $_getSZ(1);
  @$pb.TagNumber(2)
  set content($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasContent() => $_has(1);
  @$pb.TagNumber(2)
  void clearContent() => $_clearField(2);
}

/// 補完レスポンス: AI モデルからの生成結果とトークン使用量
class CompleteResponse extends $pb.GeneratedMessage {
  factory CompleteResponse({
    $core.String? id,
    $core.String? model,
    $core.String? content,
    $core.int? promptTokens,
    $core.int? completionTokens,
  }) {
    final result = create();
    if (id != null) result.id = id;
    if (model != null) result.model = model;
    if (content != null) result.content = content;
    if (promptTokens != null) result.promptTokens = promptTokens;
    if (completionTokens != null) result.completionTokens = completionTokens;
    return result;
  }

  CompleteResponse._();

  factory CompleteResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory CompleteResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'CompleteResponse',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.aigateway.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'id')
    ..aOS(2, _omitFieldNames ? '' : 'model')
    ..aOS(3, _omitFieldNames ? '' : 'content')
    ..aI(4, _omitFieldNames ? '' : 'promptTokens')
    ..aI(5, _omitFieldNames ? '' : 'completionTokens')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  CompleteResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  CompleteResponse copyWith(void Function(CompleteResponse) updates) =>
      super.copyWith((message) => updates(message as CompleteResponse))
          as CompleteResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static CompleteResponse create() => CompleteResponse._();
  @$core.override
  CompleteResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static CompleteResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<CompleteResponse>(create);
  static CompleteResponse? _defaultInstance;

  /// レスポンスの一意識別子
  @$pb.TagNumber(1)
  $core.String get id => $_getSZ(0);
  @$pb.TagNumber(1)
  set id($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasId() => $_has(0);
  @$pb.TagNumber(1)
  void clearId() => $_clearField(1);

  /// 使用されたモデル名
  @$pb.TagNumber(2)
  $core.String get model => $_getSZ(1);
  @$pb.TagNumber(2)
  set model($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasModel() => $_has(1);
  @$pb.TagNumber(2)
  void clearModel() => $_clearField(2);

  /// 生成されたテキスト内容
  @$pb.TagNumber(3)
  $core.String get content => $_getSZ(2);
  @$pb.TagNumber(3)
  set content($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasContent() => $_has(2);
  @$pb.TagNumber(3)
  void clearContent() => $_clearField(3);

  /// 入力に使用されたトークン数
  @$pb.TagNumber(4)
  $core.int get promptTokens => $_getIZ(3);
  @$pb.TagNumber(4)
  set promptTokens($core.int value) => $_setSignedInt32(3, value);
  @$pb.TagNumber(4)
  $core.bool hasPromptTokens() => $_has(3);
  @$pb.TagNumber(4)
  void clearPromptTokens() => $_clearField(4);

  /// 出力に使用されたトークン数
  @$pb.TagNumber(5)
  $core.int get completionTokens => $_getIZ(4);
  @$pb.TagNumber(5)
  set completionTokens($core.int value) => $_setSignedInt32(4, value);
  @$pb.TagNumber(5)
  $core.bool hasCompletionTokens() => $_has(4);
  @$pb.TagNumber(5)
  void clearCompletionTokens() => $_clearField(5);
}

/// ストリーミング補完リクエスト: CompleteStream RPC 専用のリクエストメッセージ
class CompleteStreamRequest extends $pb.GeneratedMessage {
  factory CompleteStreamRequest({
    $core.String? model,
    $core.Iterable<Message>? messages,
    $core.int? maxTokens,
    $core.double? temperature,
    $core.String? tenantId,
  }) {
    final result = create();
    if (model != null) result.model = model;
    if (messages != null) result.messages.addAll(messages);
    if (maxTokens != null) result.maxTokens = maxTokens;
    if (temperature != null) result.temperature = temperature;
    if (tenantId != null) result.tenantId = tenantId;
    return result;
  }

  CompleteStreamRequest._();

  factory CompleteStreamRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory CompleteStreamRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'CompleteStreamRequest',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.aigateway.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'model')
    ..pPM<Message>(2, _omitFieldNames ? '' : 'messages',
        subBuilder: Message.create)
    ..aI(3, _omitFieldNames ? '' : 'maxTokens')
    ..aD(4, _omitFieldNames ? '' : 'temperature', fieldType: $pb.PbFieldType.OF)
    ..aOS(5, _omitFieldNames ? '' : 'tenantId')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  CompleteStreamRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  CompleteStreamRequest copyWith(
          void Function(CompleteStreamRequest) updates) =>
      super.copyWith((message) => updates(message as CompleteStreamRequest))
          as CompleteStreamRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static CompleteStreamRequest create() => CompleteStreamRequest._();
  @$core.override
  CompleteStreamRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static CompleteStreamRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<CompleteStreamRequest>(create);
  static CompleteStreamRequest? _defaultInstance;

  /// 使用する AI モデルの識別子
  @$pb.TagNumber(1)
  $core.String get model => $_getSZ(0);
  @$pb.TagNumber(1)
  set model($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasModel() => $_has(0);
  @$pb.TagNumber(1)
  void clearModel() => $_clearField(1);

  /// 補完に使用するメッセージ履歴
  @$pb.TagNumber(2)
  $pb.PbList<Message> get messages => $_getList(1);

  /// 生成する最大トークン数
  @$pb.TagNumber(3)
  $core.int get maxTokens => $_getIZ(2);
  @$pb.TagNumber(3)
  set maxTokens($core.int value) => $_setSignedInt32(2, value);
  @$pb.TagNumber(3)
  $core.bool hasMaxTokens() => $_has(2);
  @$pb.TagNumber(3)
  void clearMaxTokens() => $_clearField(3);

  /// 生成のランダム性を制御する温度パラメータ（0.0〜2.0）
  @$pb.TagNumber(4)
  $core.double get temperature => $_getN(3);
  @$pb.TagNumber(4)
  set temperature($core.double value) => $_setFloat(3, value);
  @$pb.TagNumber(4)
  $core.bool hasTemperature() => $_has(3);
  @$pb.TagNumber(4)
  void clearTemperature() => $_clearField(4);

  /// リクエスト元のテナント識別子
  @$pb.TagNumber(5)
  $core.String get tenantId => $_getSZ(4);
  @$pb.TagNumber(5)
  set tenantId($core.String value) => $_setString(4, value);
  @$pb.TagNumber(5)
  $core.bool hasTenantId() => $_has(4);
  @$pb.TagNumber(5)
  void clearTenantId() => $_clearField(5);
}

/// ストリーミング補完レスポンス: ストリーミング補完の部分的なレスポンス
class CompleteStreamResponse extends $pb.GeneratedMessage {
  factory CompleteStreamResponse({
    $core.String? id,
    $core.String? delta,
    $core.bool? finished,
  }) {
    final result = create();
    if (id != null) result.id = id;
    if (delta != null) result.delta = delta;
    if (finished != null) result.finished = finished;
    return result;
  }

  CompleteStreamResponse._();

  factory CompleteStreamResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory CompleteStreamResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'CompleteStreamResponse',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.aigateway.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'id')
    ..aOS(2, _omitFieldNames ? '' : 'delta')
    ..aOB(3, _omitFieldNames ? '' : 'finished')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  CompleteStreamResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  CompleteStreamResponse copyWith(
          void Function(CompleteStreamResponse) updates) =>
      super.copyWith((message) => updates(message as CompleteStreamResponse))
          as CompleteStreamResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static CompleteStreamResponse create() => CompleteStreamResponse._();
  @$core.override
  CompleteStreamResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static CompleteStreamResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<CompleteStreamResponse>(create);
  static CompleteStreamResponse? _defaultInstance;

  /// ストリームの一意識別子
  @$pb.TagNumber(1)
  $core.String get id => $_getSZ(0);
  @$pb.TagNumber(1)
  set id($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasId() => $_has(0);
  @$pb.TagNumber(1)
  void clearId() => $_clearField(1);

  /// 差分テキスト
  @$pb.TagNumber(2)
  $core.String get delta => $_getSZ(1);
  @$pb.TagNumber(2)
  set delta($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasDelta() => $_has(1);
  @$pb.TagNumber(2)
  void clearDelta() => $_clearField(2);

  /// ストリームが完了したかどうか
  @$pb.TagNumber(3)
  $core.bool get finished => $_getBF(2);
  @$pb.TagNumber(3)
  set finished($core.bool value) => $_setBool(2, value);
  @$pb.TagNumber(3)
  $core.bool hasFinished() => $_has(2);
  @$pb.TagNumber(3)
  void clearFinished() => $_clearField(3);
}

/// 埋め込みリクエスト: テキストのベクトル化を要求する
class EmbedRequest extends $pb.GeneratedMessage {
  factory EmbedRequest({
    $core.String? model,
    $core.Iterable<$core.String>? inputs,
    $core.String? tenantId,
  }) {
    final result = create();
    if (model != null) result.model = model;
    if (inputs != null) result.inputs.addAll(inputs);
    if (tenantId != null) result.tenantId = tenantId;
    return result;
  }

  EmbedRequest._();

  factory EmbedRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory EmbedRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'EmbedRequest',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.aigateway.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'model')
    ..pPS(2, _omitFieldNames ? '' : 'inputs')
    ..aOS(3, _omitFieldNames ? '' : 'tenantId')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  EmbedRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  EmbedRequest copyWith(void Function(EmbedRequest) updates) =>
      super.copyWith((message) => updates(message as EmbedRequest))
          as EmbedRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static EmbedRequest create() => EmbedRequest._();
  @$core.override
  EmbedRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static EmbedRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<EmbedRequest>(create);
  static EmbedRequest? _defaultInstance;

  /// 埋め込みに使用するモデル名
  @$pb.TagNumber(1)
  $core.String get model => $_getSZ(0);
  @$pb.TagNumber(1)
  set model($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasModel() => $_has(0);
  @$pb.TagNumber(1)
  void clearModel() => $_clearField(1);

  /// ベクトル化する入力テキストのリスト
  @$pb.TagNumber(2)
  $pb.PbList<$core.String> get inputs => $_getList(1);

  /// リクエスト元のテナント識別子
  @$pb.TagNumber(3)
  $core.String get tenantId => $_getSZ(2);
  @$pb.TagNumber(3)
  set tenantId($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasTenantId() => $_has(2);
  @$pb.TagNumber(3)
  void clearTenantId() => $_clearField(3);
}

/// 埋め込みレスポンス: 生成されたベクトルを含む
class EmbedResponse extends $pb.GeneratedMessage {
  factory EmbedResponse({
    $core.String? model,
    $core.Iterable<Embedding>? embeddings,
  }) {
    final result = create();
    if (model != null) result.model = model;
    if (embeddings != null) result.embeddings.addAll(embeddings);
    return result;
  }

  EmbedResponse._();

  factory EmbedResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory EmbedResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'EmbedResponse',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.aigateway.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'model')
    ..pPM<Embedding>(2, _omitFieldNames ? '' : 'embeddings',
        subBuilder: Embedding.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  EmbedResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  EmbedResponse copyWith(void Function(EmbedResponse) updates) =>
      super.copyWith((message) => updates(message as EmbedResponse))
          as EmbedResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static EmbedResponse create() => EmbedResponse._();
  @$core.override
  EmbedResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static EmbedResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<EmbedResponse>(create);
  static EmbedResponse? _defaultInstance;

  /// 使用されたモデル名
  @$pb.TagNumber(1)
  $core.String get model => $_getSZ(0);
  @$pb.TagNumber(1)
  set model($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasModel() => $_has(0);
  @$pb.TagNumber(1)
  void clearModel() => $_clearField(1);

  /// 生成された埋め込みベクトルのリスト
  @$pb.TagNumber(2)
  $pb.PbList<Embedding> get embeddings => $_getList(1);
}

/// 埋め込み: 単一入力に対するベクトル表現
class Embedding extends $pb.GeneratedMessage {
  factory Embedding({
    $core.int? index,
    $core.Iterable<$core.double>? values,
  }) {
    final result = create();
    if (index != null) result.index = index;
    if (values != null) result.values.addAll(values);
    return result;
  }

  Embedding._();

  factory Embedding.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory Embedding.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'Embedding',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.aigateway.v1'),
      createEmptyInstance: create)
    ..aI(1, _omitFieldNames ? '' : 'index')
    ..p<$core.double>(2, _omitFieldNames ? '' : 'values', $pb.PbFieldType.KF)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  Embedding clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  Embedding copyWith(void Function(Embedding) updates) =>
      super.copyWith((message) => updates(message as Embedding)) as Embedding;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static Embedding create() => Embedding._();
  @$core.override
  Embedding createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static Embedding getDefault() =>
      _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<Embedding>(create);
  static Embedding? _defaultInstance;

  /// 入力テキストのインデックス
  @$pb.TagNumber(1)
  $core.int get index => $_getIZ(0);
  @$pb.TagNumber(1)
  set index($core.int value) => $_setSignedInt32(0, value);
  @$pb.TagNumber(1)
  $core.bool hasIndex() => $_has(0);
  @$pb.TagNumber(1)
  void clearIndex() => $_clearField(1);

  /// 埋め込みベクトルの値
  @$pb.TagNumber(2)
  $pb.PbList<$core.double> get values => $_getList(1);
}

/// AI モデル情報: 利用可能なモデルの詳細
class AiModel extends $pb.GeneratedMessage {
  factory AiModel({
    $core.String? id,
    $core.String? name,
    $core.String? provider,
    $core.int? contextWindow,
    $core.bool? enabled,
  }) {
    final result = create();
    if (id != null) result.id = id;
    if (name != null) result.name = name;
    if (provider != null) result.provider = provider;
    if (contextWindow != null) result.contextWindow = contextWindow;
    if (enabled != null) result.enabled = enabled;
    return result;
  }

  AiModel._();

  factory AiModel.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory AiModel.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'AiModel',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.aigateway.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'id')
    ..aOS(2, _omitFieldNames ? '' : 'name')
    ..aOS(3, _omitFieldNames ? '' : 'provider')
    ..aI(4, _omitFieldNames ? '' : 'contextWindow')
    ..aOB(5, _omitFieldNames ? '' : 'enabled')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  AiModel clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  AiModel copyWith(void Function(AiModel) updates) =>
      super.copyWith((message) => updates(message as AiModel)) as AiModel;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static AiModel create() => AiModel._();
  @$core.override
  AiModel createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static AiModel getDefault() =>
      _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<AiModel>(create);
  static AiModel? _defaultInstance;

  /// モデルの一意識別子
  @$pb.TagNumber(1)
  $core.String get id => $_getSZ(0);
  @$pb.TagNumber(1)
  set id($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasId() => $_has(0);
  @$pb.TagNumber(1)
  void clearId() => $_clearField(1);

  /// モデルの表示名
  @$pb.TagNumber(2)
  $core.String get name => $_getSZ(1);
  @$pb.TagNumber(2)
  set name($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasName() => $_has(1);
  @$pb.TagNumber(2)
  void clearName() => $_clearField(2);

  /// プロバイダー名（openai / anthropic / google 等）
  @$pb.TagNumber(3)
  $core.String get provider => $_getSZ(2);
  @$pb.TagNumber(3)
  set provider($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasProvider() => $_has(2);
  @$pb.TagNumber(3)
  void clearProvider() => $_clearField(3);

  /// コンテキストウィンドウのトークン数
  @$pb.TagNumber(4)
  $core.int get contextWindow => $_getIZ(3);
  @$pb.TagNumber(4)
  set contextWindow($core.int value) => $_setSignedInt32(3, value);
  @$pb.TagNumber(4)
  $core.bool hasContextWindow() => $_has(3);
  @$pb.TagNumber(4)
  void clearContextWindow() => $_clearField(4);

  /// モデルが有効かどうか
  @$pb.TagNumber(5)
  $core.bool get enabled => $_getBF(4);
  @$pb.TagNumber(5)
  set enabled($core.bool value) => $_setBool(4, value);
  @$pb.TagNumber(5)
  $core.bool hasEnabled() => $_has(4);
  @$pb.TagNumber(5)
  void clearEnabled() => $_clearField(5);
}

/// モデル一覧リクエスト: プロバイダーでフィルタリング可能
class ListModelsRequest extends $pb.GeneratedMessage {
  factory ListModelsRequest({
    $core.String? providerFilter,
  }) {
    final result = create();
    if (providerFilter != null) result.providerFilter = providerFilter;
    return result;
  }

  ListModelsRequest._();

  factory ListModelsRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory ListModelsRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ListModelsRequest',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.aigateway.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'providerFilter')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListModelsRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListModelsRequest copyWith(void Function(ListModelsRequest) updates) =>
      super.copyWith((message) => updates(message as ListModelsRequest))
          as ListModelsRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static ListModelsRequest create() => ListModelsRequest._();
  @$core.override
  ListModelsRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static ListModelsRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ListModelsRequest>(create);
  static ListModelsRequest? _defaultInstance;

  /// プロバイダーでフィルタリング（空の場合は全件取得）
  @$pb.TagNumber(1)
  $core.String get providerFilter => $_getSZ(0);
  @$pb.TagNumber(1)
  set providerFilter($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasProviderFilter() => $_has(0);
  @$pb.TagNumber(1)
  void clearProviderFilter() => $_clearField(1);
}

/// モデル一覧レスポンス: 利用可能なモデルのリスト
class ListModelsResponse extends $pb.GeneratedMessage {
  factory ListModelsResponse({
    $core.Iterable<AiModel>? models,
  }) {
    final result = create();
    if (models != null) result.models.addAll(models);
    return result;
  }

  ListModelsResponse._();

  factory ListModelsResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory ListModelsResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ListModelsResponse',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.aigateway.v1'),
      createEmptyInstance: create)
    ..pPM<AiModel>(1, _omitFieldNames ? '' : 'models',
        subBuilder: AiModel.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListModelsResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListModelsResponse copyWith(void Function(ListModelsResponse) updates) =>
      super.copyWith((message) => updates(message as ListModelsResponse))
          as ListModelsResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static ListModelsResponse create() => ListModelsResponse._();
  @$core.override
  ListModelsResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static ListModelsResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ListModelsResponse>(create);
  static ListModelsResponse? _defaultInstance;

  /// AI モデルのリスト
  @$pb.TagNumber(1)
  $pb.PbList<AiModel> get models => $_getList(0);
}

/// 使用量取得リクエスト: テナントの期間指定使用量を取得する
class GetUsageRequest extends $pb.GeneratedMessage {
  factory GetUsageRequest({
    $core.String? tenantId,
    $core.String? startDate,
    $core.String? endDate,
  }) {
    final result = create();
    if (tenantId != null) result.tenantId = tenantId;
    if (startDate != null) result.startDate = startDate;
    if (endDate != null) result.endDate = endDate;
    return result;
  }

  GetUsageRequest._();

  factory GetUsageRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory GetUsageRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'GetUsageRequest',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.aigateway.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'tenantId')
    ..aOS(2, _omitFieldNames ? '' : 'startDate')
    ..aOS(3, _omitFieldNames ? '' : 'endDate')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetUsageRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetUsageRequest copyWith(void Function(GetUsageRequest) updates) =>
      super.copyWith((message) => updates(message as GetUsageRequest))
          as GetUsageRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static GetUsageRequest create() => GetUsageRequest._();
  @$core.override
  GetUsageRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static GetUsageRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<GetUsageRequest>(create);
  static GetUsageRequest? _defaultInstance;

  /// 使用量を取得するテナント識別子
  @$pb.TagNumber(1)
  $core.String get tenantId => $_getSZ(0);
  @$pb.TagNumber(1)
  set tenantId($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasTenantId() => $_has(0);
  @$pb.TagNumber(1)
  void clearTenantId() => $_clearField(1);

  /// 集計開始日（YYYY-MM-DD 形式）
  @$pb.TagNumber(2)
  $core.String get startDate => $_getSZ(1);
  @$pb.TagNumber(2)
  set startDate($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasStartDate() => $_has(1);
  @$pb.TagNumber(2)
  void clearStartDate() => $_clearField(2);

  /// 集計終了日（YYYY-MM-DD 形式）
  @$pb.TagNumber(3)
  $core.String get endDate => $_getSZ(2);
  @$pb.TagNumber(3)
  set endDate($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasEndDate() => $_has(2);
  @$pb.TagNumber(3)
  void clearEndDate() => $_clearField(3);
}

/// 使用量レスポンス: テナントのトークン使用量とコスト
class GetUsageResponse extends $pb.GeneratedMessage {
  factory GetUsageResponse({
    $core.String? tenantId,
    $fixnum.Int64? totalPromptTokens,
    $fixnum.Int64? totalCompletionTokens,
    $core.double? totalCostUsd,
  }) {
    final result = create();
    if (tenantId != null) result.tenantId = tenantId;
    if (totalPromptTokens != null) result.totalPromptTokens = totalPromptTokens;
    if (totalCompletionTokens != null)
      result.totalCompletionTokens = totalCompletionTokens;
    if (totalCostUsd != null) result.totalCostUsd = totalCostUsd;
    return result;
  }

  GetUsageResponse._();

  factory GetUsageResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory GetUsageResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'GetUsageResponse',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.aigateway.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'tenantId')
    ..aInt64(2, _omitFieldNames ? '' : 'totalPromptTokens')
    ..aInt64(3, _omitFieldNames ? '' : 'totalCompletionTokens')
    ..aD(4, _omitFieldNames ? '' : 'totalCostUsd')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetUsageResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetUsageResponse copyWith(void Function(GetUsageResponse) updates) =>
      super.copyWith((message) => updates(message as GetUsageResponse))
          as GetUsageResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static GetUsageResponse create() => GetUsageResponse._();
  @$core.override
  GetUsageResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static GetUsageResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<GetUsageResponse>(create);
  static GetUsageResponse? _defaultInstance;

  /// テナント識別子
  @$pb.TagNumber(1)
  $core.String get tenantId => $_getSZ(0);
  @$pb.TagNumber(1)
  set tenantId($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasTenantId() => $_has(0);
  @$pb.TagNumber(1)
  void clearTenantId() => $_clearField(1);

  /// 入力トークンの合計使用量
  @$pb.TagNumber(2)
  $fixnum.Int64 get totalPromptTokens => $_getI64(1);
  @$pb.TagNumber(2)
  set totalPromptTokens($fixnum.Int64 value) => $_setInt64(1, value);
  @$pb.TagNumber(2)
  $core.bool hasTotalPromptTokens() => $_has(1);
  @$pb.TagNumber(2)
  void clearTotalPromptTokens() => $_clearField(2);

  /// 出力トークンの合計使用量
  @$pb.TagNumber(3)
  $fixnum.Int64 get totalCompletionTokens => $_getI64(2);
  @$pb.TagNumber(3)
  set totalCompletionTokens($fixnum.Int64 value) => $_setInt64(2, value);
  @$pb.TagNumber(3)
  $core.bool hasTotalCompletionTokens() => $_has(2);
  @$pb.TagNumber(3)
  void clearTotalCompletionTokens() => $_clearField(3);

  /// 合計コスト（USD）
  @$pb.TagNumber(4)
  $core.double get totalCostUsd => $_getN(3);
  @$pb.TagNumber(4)
  set totalCostUsd($core.double value) => $_setDouble(3, value);
  @$pb.TagNumber(4)
  $core.bool hasTotalCostUsd() => $_has(3);
  @$pb.TagNumber(4)
  void clearTotalCostUsd() => $_clearField(4);
}

const $core.bool _omitFieldNames =
    $core.bool.fromEnvironment('protobuf.omit_field_names');
const $core.bool _omitMessageNames =
    $core.bool.fromEnvironment('protobuf.omit_message_names');
