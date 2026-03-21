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

import 'navigation.pbenum.dart';

export 'package:protobuf/protobuf.dart' show GeneratedMessageGenericExtensions;

export 'navigation.pbenum.dart';

/// GetNavigationRequest はナビゲーション設定取得リクエスト。
class GetNavigationRequest extends $pb.GeneratedMessage {
  factory GetNavigationRequest({
    $core.String? bearerToken,
  }) {
    final result = create();
    if (bearerToken != null) result.bearerToken = bearerToken;
    return result;
  }

  GetNavigationRequest._();

  factory GetNavigationRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory GetNavigationRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'GetNavigationRequest',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.navigation.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'bearerToken')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetNavigationRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetNavigationRequest copyWith(void Function(GetNavigationRequest) updates) =>
      super.copyWith((message) => updates(message as GetNavigationRequest))
          as GetNavigationRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static GetNavigationRequest create() => GetNavigationRequest._();
  @$core.override
  GetNavigationRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static GetNavigationRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<GetNavigationRequest>(create);
  static GetNavigationRequest? _defaultInstance;

  /// 省略時は公開ルートのみ返す
  @$pb.TagNumber(1)
  $core.String get bearerToken => $_getSZ(0);
  @$pb.TagNumber(1)
  set bearerToken($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasBearerToken() => $_has(0);
  @$pb.TagNumber(1)
  void clearBearerToken() => $_clearField(1);
}

/// GetNavigationResponse はナビゲーション設定取得レスポンス。
class GetNavigationResponse extends $pb.GeneratedMessage {
  factory GetNavigationResponse({
    $core.Iterable<Route>? routes,
    $core.Iterable<Guard>? guards,
  }) {
    final result = create();
    if (routes != null) result.routes.addAll(routes);
    if (guards != null) result.guards.addAll(guards);
    return result;
  }

  GetNavigationResponse._();

  factory GetNavigationResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory GetNavigationResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'GetNavigationResponse',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.navigation.v1'),
      createEmptyInstance: create)
    ..pPM<Route>(1, _omitFieldNames ? '' : 'routes', subBuilder: Route.create)
    ..pPM<Guard>(2, _omitFieldNames ? '' : 'guards', subBuilder: Guard.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetNavigationResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetNavigationResponse copyWith(
          void Function(GetNavigationResponse) updates) =>
      super.copyWith((message) => updates(message as GetNavigationResponse))
          as GetNavigationResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static GetNavigationResponse create() => GetNavigationResponse._();
  @$core.override
  GetNavigationResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static GetNavigationResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<GetNavigationResponse>(create);
  static GetNavigationResponse? _defaultInstance;

  @$pb.TagNumber(1)
  $pb.PbList<Route> get routes => $_getList(0);

  @$pb.TagNumber(2)
  $pb.PbList<Guard> get guards => $_getList(1);
}

/// Route はルーティング定義。
class Route extends $pb.GeneratedMessage {
  factory Route({
    $core.String? id,
    $core.String? path,
    $core.String? componentId,
    $core.Iterable<$core.String>? guardIds,
    $core.Iterable<Route>? children,
    TransitionConfig? transition,
    $core.Iterable<Param>? params,
    $core.String? redirectTo,
  }) {
    final result = create();
    if (id != null) result.id = id;
    if (path != null) result.path = path;
    if (componentId != null) result.componentId = componentId;
    if (guardIds != null) result.guardIds.addAll(guardIds);
    if (children != null) result.children.addAll(children);
    if (transition != null) result.transition = transition;
    if (params != null) result.params.addAll(params);
    if (redirectTo != null) result.redirectTo = redirectTo;
    return result;
  }

  Route._();

  factory Route.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory Route.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'Route',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.navigation.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'id')
    ..aOS(2, _omitFieldNames ? '' : 'path')
    ..aOS(3, _omitFieldNames ? '' : 'componentId')
    ..pPS(4, _omitFieldNames ? '' : 'guardIds')
    ..pPM<Route>(5, _omitFieldNames ? '' : 'children', subBuilder: Route.create)
    ..aOM<TransitionConfig>(6, _omitFieldNames ? '' : 'transition',
        subBuilder: TransitionConfig.create)
    ..pPM<Param>(7, _omitFieldNames ? '' : 'params', subBuilder: Param.create)
    ..aOS(8, _omitFieldNames ? '' : 'redirectTo')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  Route clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  Route copyWith(void Function(Route) updates) =>
      super.copyWith((message) => updates(message as Route)) as Route;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static Route create() => Route._();
  @$core.override
  Route createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static Route getDefault() =>
      _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<Route>(create);
  static Route? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get id => $_getSZ(0);
  @$pb.TagNumber(1)
  set id($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasId() => $_has(0);
  @$pb.TagNumber(1)
  void clearId() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get path => $_getSZ(1);
  @$pb.TagNumber(2)
  set path($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasPath() => $_has(1);
  @$pb.TagNumber(2)
  void clearPath() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.String get componentId => $_getSZ(2);
  @$pb.TagNumber(3)
  set componentId($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasComponentId() => $_has(2);
  @$pb.TagNumber(3)
  void clearComponentId() => $_clearField(3);

  @$pb.TagNumber(4)
  $pb.PbList<$core.String> get guardIds => $_getList(3);

  @$pb.TagNumber(5)
  $pb.PbList<Route> get children => $_getList(4);

  @$pb.TagNumber(6)
  TransitionConfig get transition => $_getN(5);
  @$pb.TagNumber(6)
  set transition(TransitionConfig value) => $_setField(6, value);
  @$pb.TagNumber(6)
  $core.bool hasTransition() => $_has(5);
  @$pb.TagNumber(6)
  void clearTransition() => $_clearField(6);
  @$pb.TagNumber(6)
  TransitionConfig ensureTransition() => $_ensure(5);

  @$pb.TagNumber(7)
  $pb.PbList<Param> get params => $_getList(6);

  @$pb.TagNumber(8)
  $core.String get redirectTo => $_getSZ(7);
  @$pb.TagNumber(8)
  set redirectTo($core.String value) => $_setString(7, value);
  @$pb.TagNumber(8)
  $core.bool hasRedirectTo() => $_has(7);
  @$pb.TagNumber(8)
  void clearRedirectTo() => $_clearField(8);
}

/// Guard はルートガード定義。
class Guard extends $pb.GeneratedMessage {
  factory Guard({
    $core.String? id,
    GuardType? type,
    $core.String? redirectTo,
    $core.Iterable<$core.String>? roles,
  }) {
    final result = create();
    if (id != null) result.id = id;
    if (type != null) result.type = type;
    if (redirectTo != null) result.redirectTo = redirectTo;
    if (roles != null) result.roles.addAll(roles);
    return result;
  }

  Guard._();

  factory Guard.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory Guard.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'Guard',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.navigation.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'id')
    ..aE<GuardType>(2, _omitFieldNames ? '' : 'type',
        enumValues: GuardType.values)
    ..aOS(3, _omitFieldNames ? '' : 'redirectTo')
    ..pPS(4, _omitFieldNames ? '' : 'roles')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  Guard clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  Guard copyWith(void Function(Guard) updates) =>
      super.copyWith((message) => updates(message as Guard)) as Guard;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static Guard create() => Guard._();
  @$core.override
  Guard createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static Guard getDefault() =>
      _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<Guard>(create);
  static Guard? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get id => $_getSZ(0);
  @$pb.TagNumber(1)
  set id($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasId() => $_has(0);
  @$pb.TagNumber(1)
  void clearId() => $_clearField(1);

  @$pb.TagNumber(2)
  GuardType get type => $_getN(1);
  @$pb.TagNumber(2)
  set type(GuardType value) => $_setField(2, value);
  @$pb.TagNumber(2)
  $core.bool hasType() => $_has(1);
  @$pb.TagNumber(2)
  void clearType() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.String get redirectTo => $_getSZ(2);
  @$pb.TagNumber(3)
  set redirectTo($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasRedirectTo() => $_has(2);
  @$pb.TagNumber(3)
  void clearRedirectTo() => $_clearField(3);

  @$pb.TagNumber(4)
  $pb.PbList<$core.String> get roles => $_getList(3);
}

/// Param はルートパラメータ定義。
class Param extends $pb.GeneratedMessage {
  factory Param({
    $core.String? name,
    ParamType? type,
  }) {
    final result = create();
    if (name != null) result.name = name;
    if (type != null) result.type = type;
    return result;
  }

  Param._();

  factory Param.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory Param.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'Param',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.navigation.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'name')
    ..aE<ParamType>(2, _omitFieldNames ? '' : 'type',
        enumValues: ParamType.values)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  Param clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  Param copyWith(void Function(Param) updates) =>
      super.copyWith((message) => updates(message as Param)) as Param;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static Param create() => Param._();
  @$core.override
  Param createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static Param getDefault() =>
      _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<Param>(create);
  static Param? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get name => $_getSZ(0);
  @$pb.TagNumber(1)
  set name($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasName() => $_has(0);
  @$pb.TagNumber(1)
  void clearName() => $_clearField(1);

  @$pb.TagNumber(2)
  ParamType get type => $_getN(1);
  @$pb.TagNumber(2)
  set type(ParamType value) => $_setField(2, value);
  @$pb.TagNumber(2)
  $core.bool hasType() => $_has(1);
  @$pb.TagNumber(2)
  void clearType() => $_clearField(2);
}

/// TransitionConfig はページ遷移アニメーション設定。
class TransitionConfig extends $pb.GeneratedMessage {
  factory TransitionConfig({
    TransitionType? type,
    $core.int? durationMs,
  }) {
    final result = create();
    if (type != null) result.type = type;
    if (durationMs != null) result.durationMs = durationMs;
    return result;
  }

  TransitionConfig._();

  factory TransitionConfig.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory TransitionConfig.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'TransitionConfig',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.navigation.v1'),
      createEmptyInstance: create)
    ..aE<TransitionType>(1, _omitFieldNames ? '' : 'type',
        enumValues: TransitionType.values)
    ..aI(2, _omitFieldNames ? '' : 'durationMs', fieldType: $pb.PbFieldType.OU3)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  TransitionConfig clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  TransitionConfig copyWith(void Function(TransitionConfig) updates) =>
      super.copyWith((message) => updates(message as TransitionConfig))
          as TransitionConfig;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static TransitionConfig create() => TransitionConfig._();
  @$core.override
  TransitionConfig createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static TransitionConfig getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<TransitionConfig>(create);
  static TransitionConfig? _defaultInstance;

  @$pb.TagNumber(1)
  TransitionType get type => $_getN(0);
  @$pb.TagNumber(1)
  set type(TransitionType value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasType() => $_has(0);
  @$pb.TagNumber(1)
  void clearType() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.int get durationMs => $_getIZ(1);
  @$pb.TagNumber(2)
  set durationMs($core.int value) => $_setUnsignedInt32(1, value);
  @$pb.TagNumber(2)
  $core.bool hasDurationMs() => $_has(1);
  @$pb.TagNumber(2)
  void clearDurationMs() => $_clearField(2);
}

const $core.bool _omitFieldNames =
    $core.bool.fromEnvironment('protobuf.omit_field_names');
const $core.bool _omitMessageNames =
    $core.bool.fromEnvironment('protobuf.omit_message_names');
