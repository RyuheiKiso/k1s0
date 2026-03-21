// This is a generated file - do not edit.
//
// Generated from k1s0/system/ratelimit/v1/ratelimit.proto.

// @dart = 3.3

// ignore_for_file: annotate_overrides, camel_case_types, comment_references
// ignore_for_file: constant_identifier_names
// ignore_for_file: curly_braces_in_flow_control_structures
// ignore_for_file: deprecated_member_use_from_same_package, library_prefixes
// ignore_for_file: non_constant_identifier_names, prefer_relative_imports

import 'dart:core' as $core;

import 'package:fixnum/fixnum.dart' as $fixnum;
import 'package:protobuf/protobuf.dart' as $pb;

import '../../common/v1/types.pb.dart' as $1;
import 'ratelimit.pbenum.dart';

export 'package:protobuf/protobuf.dart' show GeneratedMessageGenericExtensions;

export 'ratelimit.pbenum.dart';

class CheckRateLimitRequest extends $pb.GeneratedMessage {
  factory CheckRateLimitRequest({
    $core.String? scope,
    $core.String? identifier,
    $fixnum.Int64? window,
  }) {
    final result = create();
    if (scope != null) result.scope = scope;
    if (identifier != null) result.identifier = identifier;
    if (window != null) result.window = window;
    return result;
  }

  CheckRateLimitRequest._();

  factory CheckRateLimitRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory CheckRateLimitRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'CheckRateLimitRequest',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.ratelimit.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'scope')
    ..aOS(2, _omitFieldNames ? '' : 'identifier')
    ..aInt64(3, _omitFieldNames ? '' : 'window')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  CheckRateLimitRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  CheckRateLimitRequest copyWith(
          void Function(CheckRateLimitRequest) updates) =>
      super.copyWith((message) => updates(message as CheckRateLimitRequest))
          as CheckRateLimitRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static CheckRateLimitRequest create() => CheckRateLimitRequest._();
  @$core.override
  CheckRateLimitRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static CheckRateLimitRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<CheckRateLimitRequest>(create);
  static CheckRateLimitRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get scope => $_getSZ(0);
  @$pb.TagNumber(1)
  set scope($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasScope() => $_has(0);
  @$pb.TagNumber(1)
  void clearScope() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get identifier => $_getSZ(1);
  @$pb.TagNumber(2)
  set identifier($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasIdentifier() => $_has(1);
  @$pb.TagNumber(2)
  void clearIdentifier() => $_clearField(2);

  @$pb.TagNumber(3)
  $fixnum.Int64 get window => $_getI64(2);
  @$pb.TagNumber(3)
  set window($fixnum.Int64 value) => $_setInt64(2, value);
  @$pb.TagNumber(3)
  $core.bool hasWindow() => $_has(2);
  @$pb.TagNumber(3)
  void clearWindow() => $_clearField(3);
}

class CheckRateLimitResponse extends $pb.GeneratedMessage {
  factory CheckRateLimitResponse({
    $core.bool? allowed,
    $fixnum.Int64? remaining,
    $fixnum.Int64? resetAt,
    $core.String? reason,
    $fixnum.Int64? limit,
    $core.String? scope,
    $core.String? identifier,
    $fixnum.Int64? used,
    $core.String? ruleId,
  }) {
    final result = create();
    if (allowed != null) result.allowed = allowed;
    if (remaining != null) result.remaining = remaining;
    if (resetAt != null) result.resetAt = resetAt;
    if (reason != null) result.reason = reason;
    if (limit != null) result.limit = limit;
    if (scope != null) result.scope = scope;
    if (identifier != null) result.identifier = identifier;
    if (used != null) result.used = used;
    if (ruleId != null) result.ruleId = ruleId;
    return result;
  }

  CheckRateLimitResponse._();

  factory CheckRateLimitResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory CheckRateLimitResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'CheckRateLimitResponse',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.ratelimit.v1'),
      createEmptyInstance: create)
    ..aOB(1, _omitFieldNames ? '' : 'allowed')
    ..aInt64(2, _omitFieldNames ? '' : 'remaining')
    ..aInt64(3, _omitFieldNames ? '' : 'resetAt')
    ..aOS(4, _omitFieldNames ? '' : 'reason')
    ..aInt64(5, _omitFieldNames ? '' : 'limit')
    ..aOS(6, _omitFieldNames ? '' : 'scope')
    ..aOS(7, _omitFieldNames ? '' : 'identifier')
    ..aInt64(8, _omitFieldNames ? '' : 'used')
    ..aOS(9, _omitFieldNames ? '' : 'ruleId')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  CheckRateLimitResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  CheckRateLimitResponse copyWith(
          void Function(CheckRateLimitResponse) updates) =>
      super.copyWith((message) => updates(message as CheckRateLimitResponse))
          as CheckRateLimitResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static CheckRateLimitResponse create() => CheckRateLimitResponse._();
  @$core.override
  CheckRateLimitResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static CheckRateLimitResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<CheckRateLimitResponse>(create);
  static CheckRateLimitResponse? _defaultInstance;

  @$pb.TagNumber(1)
  $core.bool get allowed => $_getBF(0);
  @$pb.TagNumber(1)
  set allowed($core.bool value) => $_setBool(0, value);
  @$pb.TagNumber(1)
  $core.bool hasAllowed() => $_has(0);
  @$pb.TagNumber(1)
  void clearAllowed() => $_clearField(1);

  @$pb.TagNumber(2)
  $fixnum.Int64 get remaining => $_getI64(1);
  @$pb.TagNumber(2)
  set remaining($fixnum.Int64 value) => $_setInt64(1, value);
  @$pb.TagNumber(2)
  $core.bool hasRemaining() => $_has(1);
  @$pb.TagNumber(2)
  void clearRemaining() => $_clearField(2);

  @$pb.TagNumber(3)
  $fixnum.Int64 get resetAt => $_getI64(2);
  @$pb.TagNumber(3)
  set resetAt($fixnum.Int64 value) => $_setInt64(2, value);
  @$pb.TagNumber(3)
  $core.bool hasResetAt() => $_has(2);
  @$pb.TagNumber(3)
  void clearResetAt() => $_clearField(3);

  @$pb.TagNumber(4)
  $core.String get reason => $_getSZ(3);
  @$pb.TagNumber(4)
  set reason($core.String value) => $_setString(3, value);
  @$pb.TagNumber(4)
  $core.bool hasReason() => $_has(3);
  @$pb.TagNumber(4)
  void clearReason() => $_clearField(4);

  @$pb.TagNumber(5)
  $fixnum.Int64 get limit => $_getI64(4);
  @$pb.TagNumber(5)
  set limit($fixnum.Int64 value) => $_setInt64(4, value);
  @$pb.TagNumber(5)
  $core.bool hasLimit() => $_has(4);
  @$pb.TagNumber(5)
  void clearLimit() => $_clearField(5);

  @$pb.TagNumber(6)
  $core.String get scope => $_getSZ(5);
  @$pb.TagNumber(6)
  set scope($core.String value) => $_setString(5, value);
  @$pb.TagNumber(6)
  $core.bool hasScope() => $_has(5);
  @$pb.TagNumber(6)
  void clearScope() => $_clearField(6);

  @$pb.TagNumber(7)
  $core.String get identifier => $_getSZ(6);
  @$pb.TagNumber(7)
  set identifier($core.String value) => $_setString(6, value);
  @$pb.TagNumber(7)
  $core.bool hasIdentifier() => $_has(6);
  @$pb.TagNumber(7)
  void clearIdentifier() => $_clearField(7);

  @$pb.TagNumber(8)
  $fixnum.Int64 get used => $_getI64(7);
  @$pb.TagNumber(8)
  set used($fixnum.Int64 value) => $_setInt64(7, value);
  @$pb.TagNumber(8)
  $core.bool hasUsed() => $_has(7);
  @$pb.TagNumber(8)
  void clearUsed() => $_clearField(8);

  @$pb.TagNumber(9)
  $core.String get ruleId => $_getSZ(8);
  @$pb.TagNumber(9)
  set ruleId($core.String value) => $_setString(8, value);
  @$pb.TagNumber(9)
  $core.bool hasRuleId() => $_has(8);
  @$pb.TagNumber(9)
  void clearRuleId() => $_clearField(9);
}

class CreateRuleRequest extends $pb.GeneratedMessage {
  factory CreateRuleRequest({
    $core.String? scope,
    $core.String? identifierPattern,
    $fixnum.Int64? limit,
    $fixnum.Int64? windowSeconds,
    $core.bool? enabled,
  }) {
    final result = create();
    if (scope != null) result.scope = scope;
    if (identifierPattern != null) result.identifierPattern = identifierPattern;
    if (limit != null) result.limit = limit;
    if (windowSeconds != null) result.windowSeconds = windowSeconds;
    if (enabled != null) result.enabled = enabled;
    return result;
  }

  CreateRuleRequest._();

  factory CreateRuleRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory CreateRuleRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'CreateRuleRequest',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.ratelimit.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'scope')
    ..aOS(2, _omitFieldNames ? '' : 'identifierPattern')
    ..aInt64(3, _omitFieldNames ? '' : 'limit')
    ..aInt64(4, _omitFieldNames ? '' : 'windowSeconds')
    ..aOB(5, _omitFieldNames ? '' : 'enabled')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  CreateRuleRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  CreateRuleRequest copyWith(void Function(CreateRuleRequest) updates) =>
      super.copyWith((message) => updates(message as CreateRuleRequest))
          as CreateRuleRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static CreateRuleRequest create() => CreateRuleRequest._();
  @$core.override
  CreateRuleRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static CreateRuleRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<CreateRuleRequest>(create);
  static CreateRuleRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get scope => $_getSZ(0);
  @$pb.TagNumber(1)
  set scope($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasScope() => $_has(0);
  @$pb.TagNumber(1)
  void clearScope() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get identifierPattern => $_getSZ(1);
  @$pb.TagNumber(2)
  set identifierPattern($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasIdentifierPattern() => $_has(1);
  @$pb.TagNumber(2)
  void clearIdentifierPattern() => $_clearField(2);

  @$pb.TagNumber(3)
  $fixnum.Int64 get limit => $_getI64(2);
  @$pb.TagNumber(3)
  set limit($fixnum.Int64 value) => $_setInt64(2, value);
  @$pb.TagNumber(3)
  $core.bool hasLimit() => $_has(2);
  @$pb.TagNumber(3)
  void clearLimit() => $_clearField(3);

  @$pb.TagNumber(4)
  $fixnum.Int64 get windowSeconds => $_getI64(3);
  @$pb.TagNumber(4)
  set windowSeconds($fixnum.Int64 value) => $_setInt64(3, value);
  @$pb.TagNumber(4)
  $core.bool hasWindowSeconds() => $_has(3);
  @$pb.TagNumber(4)
  void clearWindowSeconds() => $_clearField(4);

  @$pb.TagNumber(5)
  $core.bool get enabled => $_getBF(4);
  @$pb.TagNumber(5)
  set enabled($core.bool value) => $_setBool(4, value);
  @$pb.TagNumber(5)
  $core.bool hasEnabled() => $_has(4);
  @$pb.TagNumber(5)
  void clearEnabled() => $_clearField(5);
}

class CreateRuleResponse extends $pb.GeneratedMessage {
  factory CreateRuleResponse({
    RateLimitRule? rule,
  }) {
    final result = create();
    if (rule != null) result.rule = rule;
    return result;
  }

  CreateRuleResponse._();

  factory CreateRuleResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory CreateRuleResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'CreateRuleResponse',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.ratelimit.v1'),
      createEmptyInstance: create)
    ..aOM<RateLimitRule>(1, _omitFieldNames ? '' : 'rule',
        subBuilder: RateLimitRule.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  CreateRuleResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  CreateRuleResponse copyWith(void Function(CreateRuleResponse) updates) =>
      super.copyWith((message) => updates(message as CreateRuleResponse))
          as CreateRuleResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static CreateRuleResponse create() => CreateRuleResponse._();
  @$core.override
  CreateRuleResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static CreateRuleResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<CreateRuleResponse>(create);
  static CreateRuleResponse? _defaultInstance;

  @$pb.TagNumber(1)
  RateLimitRule get rule => $_getN(0);
  @$pb.TagNumber(1)
  set rule(RateLimitRule value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasRule() => $_has(0);
  @$pb.TagNumber(1)
  void clearRule() => $_clearField(1);
  @$pb.TagNumber(1)
  RateLimitRule ensureRule() => $_ensure(0);
}

class GetRuleRequest extends $pb.GeneratedMessage {
  factory GetRuleRequest({
    $core.String? ruleId,
  }) {
    final result = create();
    if (ruleId != null) result.ruleId = ruleId;
    return result;
  }

  GetRuleRequest._();

  factory GetRuleRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory GetRuleRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'GetRuleRequest',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.ratelimit.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'ruleId')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetRuleRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetRuleRequest copyWith(void Function(GetRuleRequest) updates) =>
      super.copyWith((message) => updates(message as GetRuleRequest))
          as GetRuleRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static GetRuleRequest create() => GetRuleRequest._();
  @$core.override
  GetRuleRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static GetRuleRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<GetRuleRequest>(create);
  static GetRuleRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get ruleId => $_getSZ(0);
  @$pb.TagNumber(1)
  set ruleId($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasRuleId() => $_has(0);
  @$pb.TagNumber(1)
  void clearRuleId() => $_clearField(1);
}

class GetRuleResponse extends $pb.GeneratedMessage {
  factory GetRuleResponse({
    RateLimitRule? rule,
  }) {
    final result = create();
    if (rule != null) result.rule = rule;
    return result;
  }

  GetRuleResponse._();

  factory GetRuleResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory GetRuleResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'GetRuleResponse',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.ratelimit.v1'),
      createEmptyInstance: create)
    ..aOM<RateLimitRule>(1, _omitFieldNames ? '' : 'rule',
        subBuilder: RateLimitRule.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetRuleResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetRuleResponse copyWith(void Function(GetRuleResponse) updates) =>
      super.copyWith((message) => updates(message as GetRuleResponse))
          as GetRuleResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static GetRuleResponse create() => GetRuleResponse._();
  @$core.override
  GetRuleResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static GetRuleResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<GetRuleResponse>(create);
  static GetRuleResponse? _defaultInstance;

  @$pb.TagNumber(1)
  RateLimitRule get rule => $_getN(0);
  @$pb.TagNumber(1)
  set rule(RateLimitRule value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasRule() => $_has(0);
  @$pb.TagNumber(1)
  void clearRule() => $_clearField(1);
  @$pb.TagNumber(1)
  RateLimitRule ensureRule() => $_ensure(0);
}

class UpdateRuleRequest extends $pb.GeneratedMessage {
  factory UpdateRuleRequest({
    $core.String? ruleId,
    $core.String? scope,
    $core.String? identifierPattern,
    $fixnum.Int64? limit,
    $fixnum.Int64? windowSeconds,
    $core.bool? enabled,
  }) {
    final result = create();
    if (ruleId != null) result.ruleId = ruleId;
    if (scope != null) result.scope = scope;
    if (identifierPattern != null) result.identifierPattern = identifierPattern;
    if (limit != null) result.limit = limit;
    if (windowSeconds != null) result.windowSeconds = windowSeconds;
    if (enabled != null) result.enabled = enabled;
    return result;
  }

  UpdateRuleRequest._();

  factory UpdateRuleRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory UpdateRuleRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'UpdateRuleRequest',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.ratelimit.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'ruleId')
    ..aOS(2, _omitFieldNames ? '' : 'scope')
    ..aOS(3, _omitFieldNames ? '' : 'identifierPattern')
    ..aInt64(4, _omitFieldNames ? '' : 'limit')
    ..aInt64(5, _omitFieldNames ? '' : 'windowSeconds')
    ..aOB(6, _omitFieldNames ? '' : 'enabled')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  UpdateRuleRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  UpdateRuleRequest copyWith(void Function(UpdateRuleRequest) updates) =>
      super.copyWith((message) => updates(message as UpdateRuleRequest))
          as UpdateRuleRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static UpdateRuleRequest create() => UpdateRuleRequest._();
  @$core.override
  UpdateRuleRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static UpdateRuleRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<UpdateRuleRequest>(create);
  static UpdateRuleRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get ruleId => $_getSZ(0);
  @$pb.TagNumber(1)
  set ruleId($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasRuleId() => $_has(0);
  @$pb.TagNumber(1)
  void clearRuleId() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get scope => $_getSZ(1);
  @$pb.TagNumber(2)
  set scope($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasScope() => $_has(1);
  @$pb.TagNumber(2)
  void clearScope() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.String get identifierPattern => $_getSZ(2);
  @$pb.TagNumber(3)
  set identifierPattern($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasIdentifierPattern() => $_has(2);
  @$pb.TagNumber(3)
  void clearIdentifierPattern() => $_clearField(3);

  @$pb.TagNumber(4)
  $fixnum.Int64 get limit => $_getI64(3);
  @$pb.TagNumber(4)
  set limit($fixnum.Int64 value) => $_setInt64(3, value);
  @$pb.TagNumber(4)
  $core.bool hasLimit() => $_has(3);
  @$pb.TagNumber(4)
  void clearLimit() => $_clearField(4);

  @$pb.TagNumber(5)
  $fixnum.Int64 get windowSeconds => $_getI64(4);
  @$pb.TagNumber(5)
  set windowSeconds($fixnum.Int64 value) => $_setInt64(4, value);
  @$pb.TagNumber(5)
  $core.bool hasWindowSeconds() => $_has(4);
  @$pb.TagNumber(5)
  void clearWindowSeconds() => $_clearField(5);

  @$pb.TagNumber(6)
  $core.bool get enabled => $_getBF(5);
  @$pb.TagNumber(6)
  set enabled($core.bool value) => $_setBool(5, value);
  @$pb.TagNumber(6)
  $core.bool hasEnabled() => $_has(5);
  @$pb.TagNumber(6)
  void clearEnabled() => $_clearField(6);
}

class UpdateRuleResponse extends $pb.GeneratedMessage {
  factory UpdateRuleResponse({
    RateLimitRule? rule,
  }) {
    final result = create();
    if (rule != null) result.rule = rule;
    return result;
  }

  UpdateRuleResponse._();

  factory UpdateRuleResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory UpdateRuleResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'UpdateRuleResponse',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.ratelimit.v1'),
      createEmptyInstance: create)
    ..aOM<RateLimitRule>(1, _omitFieldNames ? '' : 'rule',
        subBuilder: RateLimitRule.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  UpdateRuleResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  UpdateRuleResponse copyWith(void Function(UpdateRuleResponse) updates) =>
      super.copyWith((message) => updates(message as UpdateRuleResponse))
          as UpdateRuleResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static UpdateRuleResponse create() => UpdateRuleResponse._();
  @$core.override
  UpdateRuleResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static UpdateRuleResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<UpdateRuleResponse>(create);
  static UpdateRuleResponse? _defaultInstance;

  @$pb.TagNumber(1)
  RateLimitRule get rule => $_getN(0);
  @$pb.TagNumber(1)
  set rule(RateLimitRule value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasRule() => $_has(0);
  @$pb.TagNumber(1)
  void clearRule() => $_clearField(1);
  @$pb.TagNumber(1)
  RateLimitRule ensureRule() => $_ensure(0);
}

class DeleteRuleRequest extends $pb.GeneratedMessage {
  factory DeleteRuleRequest({
    $core.String? ruleId,
  }) {
    final result = create();
    if (ruleId != null) result.ruleId = ruleId;
    return result;
  }

  DeleteRuleRequest._();

  factory DeleteRuleRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory DeleteRuleRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'DeleteRuleRequest',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.ratelimit.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'ruleId')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  DeleteRuleRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  DeleteRuleRequest copyWith(void Function(DeleteRuleRequest) updates) =>
      super.copyWith((message) => updates(message as DeleteRuleRequest))
          as DeleteRuleRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static DeleteRuleRequest create() => DeleteRuleRequest._();
  @$core.override
  DeleteRuleRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static DeleteRuleRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<DeleteRuleRequest>(create);
  static DeleteRuleRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get ruleId => $_getSZ(0);
  @$pb.TagNumber(1)
  set ruleId($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasRuleId() => $_has(0);
  @$pb.TagNumber(1)
  void clearRuleId() => $_clearField(1);
}

class DeleteRuleResponse extends $pb.GeneratedMessage {
  factory DeleteRuleResponse({
    $core.bool? success,
  }) {
    final result = create();
    if (success != null) result.success = success;
    return result;
  }

  DeleteRuleResponse._();

  factory DeleteRuleResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory DeleteRuleResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'DeleteRuleResponse',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.ratelimit.v1'),
      createEmptyInstance: create)
    ..aOB(1, _omitFieldNames ? '' : 'success')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  DeleteRuleResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  DeleteRuleResponse copyWith(void Function(DeleteRuleResponse) updates) =>
      super.copyWith((message) => updates(message as DeleteRuleResponse))
          as DeleteRuleResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static DeleteRuleResponse create() => DeleteRuleResponse._();
  @$core.override
  DeleteRuleResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static DeleteRuleResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<DeleteRuleResponse>(create);
  static DeleteRuleResponse? _defaultInstance;

  @$pb.TagNumber(1)
  $core.bool get success => $_getBF(0);
  @$pb.TagNumber(1)
  set success($core.bool value) => $_setBool(0, value);
  @$pb.TagNumber(1)
  $core.bool hasSuccess() => $_has(0);
  @$pb.TagNumber(1)
  void clearSuccess() => $_clearField(1);
}

class ListRulesRequest extends $pb.GeneratedMessage {
  factory ListRulesRequest({
    $core.String? scope,
    $core.bool? enabledOnly,
    $1.Pagination? pagination,
  }) {
    final result = create();
    if (scope != null) result.scope = scope;
    if (enabledOnly != null) result.enabledOnly = enabledOnly;
    if (pagination != null) result.pagination = pagination;
    return result;
  }

  ListRulesRequest._();

  factory ListRulesRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory ListRulesRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ListRulesRequest',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.ratelimit.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'scope')
    ..aOB(2, _omitFieldNames ? '' : 'enabledOnly')
    ..aOM<$1.Pagination>(3, _omitFieldNames ? '' : 'pagination',
        subBuilder: $1.Pagination.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListRulesRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListRulesRequest copyWith(void Function(ListRulesRequest) updates) =>
      super.copyWith((message) => updates(message as ListRulesRequest))
          as ListRulesRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static ListRulesRequest create() => ListRulesRequest._();
  @$core.override
  ListRulesRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static ListRulesRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ListRulesRequest>(create);
  static ListRulesRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get scope => $_getSZ(0);
  @$pb.TagNumber(1)
  set scope($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasScope() => $_has(0);
  @$pb.TagNumber(1)
  void clearScope() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.bool get enabledOnly => $_getBF(1);
  @$pb.TagNumber(2)
  set enabledOnly($core.bool value) => $_setBool(1, value);
  @$pb.TagNumber(2)
  $core.bool hasEnabledOnly() => $_has(1);
  @$pb.TagNumber(2)
  void clearEnabledOnly() => $_clearField(2);

  /// ページネーションパラメータを共通型に統一
  @$pb.TagNumber(3)
  $1.Pagination get pagination => $_getN(2);
  @$pb.TagNumber(3)
  set pagination($1.Pagination value) => $_setField(3, value);
  @$pb.TagNumber(3)
  $core.bool hasPagination() => $_has(2);
  @$pb.TagNumber(3)
  void clearPagination() => $_clearField(3);
  @$pb.TagNumber(3)
  $1.Pagination ensurePagination() => $_ensure(2);
}

class ListRulesResponse extends $pb.GeneratedMessage {
  factory ListRulesResponse({
    $core.Iterable<RateLimitRule>? rules,
    $1.PaginationResult? pagination,
  }) {
    final result = create();
    if (rules != null) result.rules.addAll(rules);
    if (pagination != null) result.pagination = pagination;
    return result;
  }

  ListRulesResponse._();

  factory ListRulesResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory ListRulesResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ListRulesResponse',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.ratelimit.v1'),
      createEmptyInstance: create)
    ..pPM<RateLimitRule>(1, _omitFieldNames ? '' : 'rules',
        subBuilder: RateLimitRule.create)
    ..aOM<$1.PaginationResult>(2, _omitFieldNames ? '' : 'pagination',
        subBuilder: $1.PaginationResult.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListRulesResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListRulesResponse copyWith(void Function(ListRulesResponse) updates) =>
      super.copyWith((message) => updates(message as ListRulesResponse))
          as ListRulesResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static ListRulesResponse create() => ListRulesResponse._();
  @$core.override
  ListRulesResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static ListRulesResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ListRulesResponse>(create);
  static ListRulesResponse? _defaultInstance;

  @$pb.TagNumber(1)
  $pb.PbList<RateLimitRule> get rules => $_getList(0);

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

class RateLimitRule extends $pb.GeneratedMessage {
  factory RateLimitRule({
    $core.String? id,
    $core.String? scope,
    $core.String? identifierPattern,
    $fixnum.Int64? limit,
    $fixnum.Int64? windowSeconds,
    $core.String? algorithm,
    $core.bool? enabled,
    $1.Timestamp? createdAt,
    $1.Timestamp? updatedAt,
    $core.String? name,
    RateLimitAlgorithm? algorithmEnum,
  }) {
    final result = create();
    if (id != null) result.id = id;
    if (scope != null) result.scope = scope;
    if (identifierPattern != null) result.identifierPattern = identifierPattern;
    if (limit != null) result.limit = limit;
    if (windowSeconds != null) result.windowSeconds = windowSeconds;
    if (algorithm != null) result.algorithm = algorithm;
    if (enabled != null) result.enabled = enabled;
    if (createdAt != null) result.createdAt = createdAt;
    if (updatedAt != null) result.updatedAt = updatedAt;
    if (name != null) result.name = name;
    if (algorithmEnum != null) result.algorithmEnum = algorithmEnum;
    return result;
  }

  RateLimitRule._();

  factory RateLimitRule.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory RateLimitRule.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'RateLimitRule',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.ratelimit.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'id')
    ..aOS(2, _omitFieldNames ? '' : 'scope')
    ..aOS(3, _omitFieldNames ? '' : 'identifierPattern')
    ..aInt64(4, _omitFieldNames ? '' : 'limit')
    ..aInt64(5, _omitFieldNames ? '' : 'windowSeconds')
    ..aOS(6, _omitFieldNames ? '' : 'algorithm')
    ..aOB(7, _omitFieldNames ? '' : 'enabled')
    ..aOM<$1.Timestamp>(8, _omitFieldNames ? '' : 'createdAt',
        subBuilder: $1.Timestamp.create)
    ..aOM<$1.Timestamp>(9, _omitFieldNames ? '' : 'updatedAt',
        subBuilder: $1.Timestamp.create)
    ..aOS(10, _omitFieldNames ? '' : 'name')
    ..aE<RateLimitAlgorithm>(11, _omitFieldNames ? '' : 'algorithmEnum',
        enumValues: RateLimitAlgorithm.values)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  RateLimitRule clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  RateLimitRule copyWith(void Function(RateLimitRule) updates) =>
      super.copyWith((message) => updates(message as RateLimitRule))
          as RateLimitRule;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static RateLimitRule create() => RateLimitRule._();
  @$core.override
  RateLimitRule createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static RateLimitRule getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<RateLimitRule>(create);
  static RateLimitRule? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get id => $_getSZ(0);
  @$pb.TagNumber(1)
  set id($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasId() => $_has(0);
  @$pb.TagNumber(1)
  void clearId() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get scope => $_getSZ(1);
  @$pb.TagNumber(2)
  set scope($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasScope() => $_has(1);
  @$pb.TagNumber(2)
  void clearScope() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.String get identifierPattern => $_getSZ(2);
  @$pb.TagNumber(3)
  set identifierPattern($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasIdentifierPattern() => $_has(2);
  @$pb.TagNumber(3)
  void clearIdentifierPattern() => $_clearField(3);

  @$pb.TagNumber(4)
  $fixnum.Int64 get limit => $_getI64(3);
  @$pb.TagNumber(4)
  set limit($fixnum.Int64 value) => $_setInt64(3, value);
  @$pb.TagNumber(4)
  $core.bool hasLimit() => $_has(3);
  @$pb.TagNumber(4)
  void clearLimit() => $_clearField(4);

  @$pb.TagNumber(5)
  $fixnum.Int64 get windowSeconds => $_getI64(4);
  @$pb.TagNumber(5)
  set windowSeconds($fixnum.Int64 value) => $_setInt64(4, value);
  @$pb.TagNumber(5)
  $core.bool hasWindowSeconds() => $_has(4);
  @$pb.TagNumber(5)
  void clearWindowSeconds() => $_clearField(5);

  /// Deprecated: use algorithm_enum instead.
  @$pb.TagNumber(6)
  $core.String get algorithm => $_getSZ(5);
  @$pb.TagNumber(6)
  set algorithm($core.String value) => $_setString(5, value);
  @$pb.TagNumber(6)
  $core.bool hasAlgorithm() => $_has(5);
  @$pb.TagNumber(6)
  void clearAlgorithm() => $_clearField(6);

  @$pb.TagNumber(7)
  $core.bool get enabled => $_getBF(6);
  @$pb.TagNumber(7)
  set enabled($core.bool value) => $_setBool(6, value);
  @$pb.TagNumber(7)
  $core.bool hasEnabled() => $_has(6);
  @$pb.TagNumber(7)
  void clearEnabled() => $_clearField(7);

  @$pb.TagNumber(8)
  $1.Timestamp get createdAt => $_getN(7);
  @$pb.TagNumber(8)
  set createdAt($1.Timestamp value) => $_setField(8, value);
  @$pb.TagNumber(8)
  $core.bool hasCreatedAt() => $_has(7);
  @$pb.TagNumber(8)
  void clearCreatedAt() => $_clearField(8);
  @$pb.TagNumber(8)
  $1.Timestamp ensureCreatedAt() => $_ensure(7);

  @$pb.TagNumber(9)
  $1.Timestamp get updatedAt => $_getN(8);
  @$pb.TagNumber(9)
  set updatedAt($1.Timestamp value) => $_setField(9, value);
  @$pb.TagNumber(9)
  $core.bool hasUpdatedAt() => $_has(8);
  @$pb.TagNumber(9)
  void clearUpdatedAt() => $_clearField(9);
  @$pb.TagNumber(9)
  $1.Timestamp ensureUpdatedAt() => $_ensure(8);

  @$pb.TagNumber(10)
  $core.String get name => $_getSZ(9);
  @$pb.TagNumber(10)
  set name($core.String value) => $_setString(9, value);
  @$pb.TagNumber(10)
  $core.bool hasName() => $_has(9);
  @$pb.TagNumber(10)
  void clearName() => $_clearField(10);

  /// レートリミットアルゴリズムの enum 版（algorithm の型付き版）。
  @$pb.TagNumber(11)
  RateLimitAlgorithm get algorithmEnum => $_getN(10);
  @$pb.TagNumber(11)
  set algorithmEnum(RateLimitAlgorithm value) => $_setField(11, value);
  @$pb.TagNumber(11)
  $core.bool hasAlgorithmEnum() => $_has(10);
  @$pb.TagNumber(11)
  void clearAlgorithmEnum() => $_clearField(11);
}

class GetUsageRequest extends $pb.GeneratedMessage {
  factory GetUsageRequest({
    $core.String? ruleId,
  }) {
    final result = create();
    if (ruleId != null) result.ruleId = ruleId;
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
          _omitMessageNames ? '' : 'k1s0.system.ratelimit.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'ruleId')
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

  @$pb.TagNumber(1)
  $core.String get ruleId => $_getSZ(0);
  @$pb.TagNumber(1)
  set ruleId($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasRuleId() => $_has(0);
  @$pb.TagNumber(1)
  void clearRuleId() => $_clearField(1);
}

class GetUsageResponse extends $pb.GeneratedMessage {
  factory GetUsageResponse({
    $core.String? ruleId,
    $core.String? ruleName,
    $fixnum.Int64? limit,
    $fixnum.Int64? windowSeconds,
    $core.String? algorithm,
    $core.bool? enabled,
    $fixnum.Int64? used,
    $fixnum.Int64? remaining,
    $fixnum.Int64? resetAt,
    RateLimitAlgorithm? algorithmEnum,
  }) {
    final result = create();
    if (ruleId != null) result.ruleId = ruleId;
    if (ruleName != null) result.ruleName = ruleName;
    if (limit != null) result.limit = limit;
    if (windowSeconds != null) result.windowSeconds = windowSeconds;
    if (algorithm != null) result.algorithm = algorithm;
    if (enabled != null) result.enabled = enabled;
    if (used != null) result.used = used;
    if (remaining != null) result.remaining = remaining;
    if (resetAt != null) result.resetAt = resetAt;
    if (algorithmEnum != null) result.algorithmEnum = algorithmEnum;
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
          _omitMessageNames ? '' : 'k1s0.system.ratelimit.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'ruleId')
    ..aOS(2, _omitFieldNames ? '' : 'ruleName')
    ..aInt64(3, _omitFieldNames ? '' : 'limit')
    ..aInt64(4, _omitFieldNames ? '' : 'windowSeconds')
    ..aOS(5, _omitFieldNames ? '' : 'algorithm')
    ..aOB(6, _omitFieldNames ? '' : 'enabled')
    ..aInt64(7, _omitFieldNames ? '' : 'used')
    ..aInt64(8, _omitFieldNames ? '' : 'remaining')
    ..aInt64(9, _omitFieldNames ? '' : 'resetAt')
    ..aE<RateLimitAlgorithm>(10, _omitFieldNames ? '' : 'algorithmEnum',
        enumValues: RateLimitAlgorithm.values)
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

  @$pb.TagNumber(1)
  $core.String get ruleId => $_getSZ(0);
  @$pb.TagNumber(1)
  set ruleId($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasRuleId() => $_has(0);
  @$pb.TagNumber(1)
  void clearRuleId() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get ruleName => $_getSZ(1);
  @$pb.TagNumber(2)
  set ruleName($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasRuleName() => $_has(1);
  @$pb.TagNumber(2)
  void clearRuleName() => $_clearField(2);

  @$pb.TagNumber(3)
  $fixnum.Int64 get limit => $_getI64(2);
  @$pb.TagNumber(3)
  set limit($fixnum.Int64 value) => $_setInt64(2, value);
  @$pb.TagNumber(3)
  $core.bool hasLimit() => $_has(2);
  @$pb.TagNumber(3)
  void clearLimit() => $_clearField(3);

  @$pb.TagNumber(4)
  $fixnum.Int64 get windowSeconds => $_getI64(3);
  @$pb.TagNumber(4)
  set windowSeconds($fixnum.Int64 value) => $_setInt64(3, value);
  @$pb.TagNumber(4)
  $core.bool hasWindowSeconds() => $_has(3);
  @$pb.TagNumber(4)
  void clearWindowSeconds() => $_clearField(4);

  /// Deprecated: use algorithm_enum instead.
  @$pb.TagNumber(5)
  $core.String get algorithm => $_getSZ(4);
  @$pb.TagNumber(5)
  set algorithm($core.String value) => $_setString(4, value);
  @$pb.TagNumber(5)
  $core.bool hasAlgorithm() => $_has(4);
  @$pb.TagNumber(5)
  void clearAlgorithm() => $_clearField(5);

  @$pb.TagNumber(6)
  $core.bool get enabled => $_getBF(5);
  @$pb.TagNumber(6)
  set enabled($core.bool value) => $_setBool(5, value);
  @$pb.TagNumber(6)
  $core.bool hasEnabled() => $_has(5);
  @$pb.TagNumber(6)
  void clearEnabled() => $_clearField(6);

  @$pb.TagNumber(7)
  $fixnum.Int64 get used => $_getI64(6);
  @$pb.TagNumber(7)
  set used($fixnum.Int64 value) => $_setInt64(6, value);
  @$pb.TagNumber(7)
  $core.bool hasUsed() => $_has(6);
  @$pb.TagNumber(7)
  void clearUsed() => $_clearField(7);

  @$pb.TagNumber(8)
  $fixnum.Int64 get remaining => $_getI64(7);
  @$pb.TagNumber(8)
  set remaining($fixnum.Int64 value) => $_setInt64(7, value);
  @$pb.TagNumber(8)
  $core.bool hasRemaining() => $_has(7);
  @$pb.TagNumber(8)
  void clearRemaining() => $_clearField(8);

  @$pb.TagNumber(9)
  $fixnum.Int64 get resetAt => $_getI64(8);
  @$pb.TagNumber(9)
  set resetAt($fixnum.Int64 value) => $_setInt64(8, value);
  @$pb.TagNumber(9)
  $core.bool hasResetAt() => $_has(8);
  @$pb.TagNumber(9)
  void clearResetAt() => $_clearField(9);

  /// レートリミットアルゴリズムの enum 版（algorithm の型付き版）。
  @$pb.TagNumber(10)
  RateLimitAlgorithm get algorithmEnum => $_getN(9);
  @$pb.TagNumber(10)
  set algorithmEnum(RateLimitAlgorithm value) => $_setField(10, value);
  @$pb.TagNumber(10)
  $core.bool hasAlgorithmEnum() => $_has(9);
  @$pb.TagNumber(10)
  void clearAlgorithmEnum() => $_clearField(10);
}

class ResetLimitRequest extends $pb.GeneratedMessage {
  factory ResetLimitRequest({
    $core.String? scope,
    $core.String? identifier,
  }) {
    final result = create();
    if (scope != null) result.scope = scope;
    if (identifier != null) result.identifier = identifier;
    return result;
  }

  ResetLimitRequest._();

  factory ResetLimitRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory ResetLimitRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ResetLimitRequest',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.ratelimit.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'scope')
    ..aOS(2, _omitFieldNames ? '' : 'identifier')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ResetLimitRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ResetLimitRequest copyWith(void Function(ResetLimitRequest) updates) =>
      super.copyWith((message) => updates(message as ResetLimitRequest))
          as ResetLimitRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static ResetLimitRequest create() => ResetLimitRequest._();
  @$core.override
  ResetLimitRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static ResetLimitRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ResetLimitRequest>(create);
  static ResetLimitRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get scope => $_getSZ(0);
  @$pb.TagNumber(1)
  set scope($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasScope() => $_has(0);
  @$pb.TagNumber(1)
  void clearScope() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get identifier => $_getSZ(1);
  @$pb.TagNumber(2)
  set identifier($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasIdentifier() => $_has(1);
  @$pb.TagNumber(2)
  void clearIdentifier() => $_clearField(2);
}

class ResetLimitResponse extends $pb.GeneratedMessage {
  factory ResetLimitResponse({
    $core.bool? success,
  }) {
    final result = create();
    if (success != null) result.success = success;
    return result;
  }

  ResetLimitResponse._();

  factory ResetLimitResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory ResetLimitResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ResetLimitResponse',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.ratelimit.v1'),
      createEmptyInstance: create)
    ..aOB(1, _omitFieldNames ? '' : 'success')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ResetLimitResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ResetLimitResponse copyWith(void Function(ResetLimitResponse) updates) =>
      super.copyWith((message) => updates(message as ResetLimitResponse))
          as ResetLimitResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static ResetLimitResponse create() => ResetLimitResponse._();
  @$core.override
  ResetLimitResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static ResetLimitResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ResetLimitResponse>(create);
  static ResetLimitResponse? _defaultInstance;

  @$pb.TagNumber(1)
  $core.bool get success => $_getBF(0);
  @$pb.TagNumber(1)
  set success($core.bool value) => $_setBool(0, value);
  @$pb.TagNumber(1)
  $core.bool hasSuccess() => $_has(0);
  @$pb.TagNumber(1)
  void clearSuccess() => $_clearField(1);
}

const $core.bool _omitFieldNames =
    $core.bool.fromEnvironment('protobuf.omit_field_names');
const $core.bool _omitMessageNames =
    $core.bool.fromEnvironment('protobuf.omit_message_names');
