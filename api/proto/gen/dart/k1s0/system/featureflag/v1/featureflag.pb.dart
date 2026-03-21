// This is a generated file - do not edit.
//
// Generated from k1s0/system/featureflag/v1/featureflag.proto.

// @dart = 3.3

// ignore_for_file: annotate_overrides, camel_case_types, comment_references
// ignore_for_file: constant_identifier_names
// ignore_for_file: curly_braces_in_flow_control_structures
// ignore_for_file: deprecated_member_use_from_same_package, library_prefixes
// ignore_for_file: non_constant_identifier_names, prefer_relative_imports

import 'dart:core' as $core;

import 'package:protobuf/protobuf.dart' as $pb;

import '../../common/v1/types.pb.dart' as $1;

export 'package:protobuf/protobuf.dart' show GeneratedMessageGenericExtensions;

class EvaluateFlagRequest extends $pb.GeneratedMessage {
  factory EvaluateFlagRequest({
    $core.String? flagKey,
    EvaluationContext? context,
  }) {
    final result = create();
    if (flagKey != null) result.flagKey = flagKey;
    if (context != null) result.context = context;
    return result;
  }

  EvaluateFlagRequest._();

  factory EvaluateFlagRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory EvaluateFlagRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'EvaluateFlagRequest',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.featureflag.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'flagKey')
    ..aOM<EvaluationContext>(2, _omitFieldNames ? '' : 'context',
        subBuilder: EvaluationContext.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  EvaluateFlagRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  EvaluateFlagRequest copyWith(void Function(EvaluateFlagRequest) updates) =>
      super.copyWith((message) => updates(message as EvaluateFlagRequest))
          as EvaluateFlagRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static EvaluateFlagRequest create() => EvaluateFlagRequest._();
  @$core.override
  EvaluateFlagRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static EvaluateFlagRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<EvaluateFlagRequest>(create);
  static EvaluateFlagRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get flagKey => $_getSZ(0);
  @$pb.TagNumber(1)
  set flagKey($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasFlagKey() => $_has(0);
  @$pb.TagNumber(1)
  void clearFlagKey() => $_clearField(1);

  @$pb.TagNumber(2)
  EvaluationContext get context => $_getN(1);
  @$pb.TagNumber(2)
  set context(EvaluationContext value) => $_setField(2, value);
  @$pb.TagNumber(2)
  $core.bool hasContext() => $_has(1);
  @$pb.TagNumber(2)
  void clearContext() => $_clearField(2);
  @$pb.TagNumber(2)
  EvaluationContext ensureContext() => $_ensure(1);
}

class EvaluateFlagResponse extends $pb.GeneratedMessage {
  factory EvaluateFlagResponse({
    $core.String? flagKey,
    $core.bool? enabled,
    $core.String? variant,
    $core.String? reason,
  }) {
    final result = create();
    if (flagKey != null) result.flagKey = flagKey;
    if (enabled != null) result.enabled = enabled;
    if (variant != null) result.variant = variant;
    if (reason != null) result.reason = reason;
    return result;
  }

  EvaluateFlagResponse._();

  factory EvaluateFlagResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory EvaluateFlagResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'EvaluateFlagResponse',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.featureflag.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'flagKey')
    ..aOB(2, _omitFieldNames ? '' : 'enabled')
    ..aOS(3, _omitFieldNames ? '' : 'variant')
    ..aOS(4, _omitFieldNames ? '' : 'reason')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  EvaluateFlagResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  EvaluateFlagResponse copyWith(void Function(EvaluateFlagResponse) updates) =>
      super.copyWith((message) => updates(message as EvaluateFlagResponse))
          as EvaluateFlagResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static EvaluateFlagResponse create() => EvaluateFlagResponse._();
  @$core.override
  EvaluateFlagResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static EvaluateFlagResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<EvaluateFlagResponse>(create);
  static EvaluateFlagResponse? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get flagKey => $_getSZ(0);
  @$pb.TagNumber(1)
  set flagKey($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasFlagKey() => $_has(0);
  @$pb.TagNumber(1)
  void clearFlagKey() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.bool get enabled => $_getBF(1);
  @$pb.TagNumber(2)
  set enabled($core.bool value) => $_setBool(1, value);
  @$pb.TagNumber(2)
  $core.bool hasEnabled() => $_has(1);
  @$pb.TagNumber(2)
  void clearEnabled() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.String get variant => $_getSZ(2);
  @$pb.TagNumber(3)
  set variant($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasVariant() => $_has(2);
  @$pb.TagNumber(3)
  void clearVariant() => $_clearField(3);

  @$pb.TagNumber(4)
  $core.String get reason => $_getSZ(3);
  @$pb.TagNumber(4)
  set reason($core.String value) => $_setString(3, value);
  @$pb.TagNumber(4)
  $core.bool hasReason() => $_has(3);
  @$pb.TagNumber(4)
  void clearReason() => $_clearField(4);
}

class EvaluationContext extends $pb.GeneratedMessage {
  factory EvaluationContext({
    $core.String? userId,
    $core.String? tenantId,
    $core.Iterable<$core.MapEntry<$core.String, $core.String>>? attributes,
  }) {
    final result = create();
    if (userId != null) result.userId = userId;
    if (tenantId != null) result.tenantId = tenantId;
    if (attributes != null) result.attributes.addEntries(attributes);
    return result;
  }

  EvaluationContext._();

  factory EvaluationContext.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory EvaluationContext.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'EvaluationContext',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.featureflag.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'userId')
    ..aOS(2, _omitFieldNames ? '' : 'tenantId')
    ..m<$core.String, $core.String>(3, _omitFieldNames ? '' : 'attributes',
        entryClassName: 'EvaluationContext.AttributesEntry',
        keyFieldType: $pb.PbFieldType.OS,
        valueFieldType: $pb.PbFieldType.OS,
        packageName: const $pb.PackageName('k1s0.system.featureflag.v1'))
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  EvaluationContext clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  EvaluationContext copyWith(void Function(EvaluationContext) updates) =>
      super.copyWith((message) => updates(message as EvaluationContext))
          as EvaluationContext;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static EvaluationContext create() => EvaluationContext._();
  @$core.override
  EvaluationContext createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static EvaluationContext getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<EvaluationContext>(create);
  static EvaluationContext? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get userId => $_getSZ(0);
  @$pb.TagNumber(1)
  set userId($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasUserId() => $_has(0);
  @$pb.TagNumber(1)
  void clearUserId() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get tenantId => $_getSZ(1);
  @$pb.TagNumber(2)
  set tenantId($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasTenantId() => $_has(1);
  @$pb.TagNumber(2)
  void clearTenantId() => $_clearField(2);

  @$pb.TagNumber(3)
  $pb.PbMap<$core.String, $core.String> get attributes => $_getMap(2);
}

class GetFlagRequest extends $pb.GeneratedMessage {
  factory GetFlagRequest({
    $core.String? flagKey,
  }) {
    final result = create();
    if (flagKey != null) result.flagKey = flagKey;
    return result;
  }

  GetFlagRequest._();

  factory GetFlagRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory GetFlagRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'GetFlagRequest',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.featureflag.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'flagKey')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetFlagRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetFlagRequest copyWith(void Function(GetFlagRequest) updates) =>
      super.copyWith((message) => updates(message as GetFlagRequest))
          as GetFlagRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static GetFlagRequest create() => GetFlagRequest._();
  @$core.override
  GetFlagRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static GetFlagRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<GetFlagRequest>(create);
  static GetFlagRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get flagKey => $_getSZ(0);
  @$pb.TagNumber(1)
  set flagKey($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasFlagKey() => $_has(0);
  @$pb.TagNumber(1)
  void clearFlagKey() => $_clearField(1);
}

class GetFlagResponse extends $pb.GeneratedMessage {
  factory GetFlagResponse({
    FeatureFlag? flag,
  }) {
    final result = create();
    if (flag != null) result.flag = flag;
    return result;
  }

  GetFlagResponse._();

  factory GetFlagResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory GetFlagResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'GetFlagResponse',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.featureflag.v1'),
      createEmptyInstance: create)
    ..aOM<FeatureFlag>(1, _omitFieldNames ? '' : 'flag',
        subBuilder: FeatureFlag.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetFlagResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetFlagResponse copyWith(void Function(GetFlagResponse) updates) =>
      super.copyWith((message) => updates(message as GetFlagResponse))
          as GetFlagResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static GetFlagResponse create() => GetFlagResponse._();
  @$core.override
  GetFlagResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static GetFlagResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<GetFlagResponse>(create);
  static GetFlagResponse? _defaultInstance;

  @$pb.TagNumber(1)
  FeatureFlag get flag => $_getN(0);
  @$pb.TagNumber(1)
  set flag(FeatureFlag value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasFlag() => $_has(0);
  @$pb.TagNumber(1)
  void clearFlag() => $_clearField(1);
  @$pb.TagNumber(1)
  FeatureFlag ensureFlag() => $_ensure(0);
}

class ListFlagsRequest extends $pb.GeneratedMessage {
  factory ListFlagsRequest() => create();

  ListFlagsRequest._();

  factory ListFlagsRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory ListFlagsRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ListFlagsRequest',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.featureflag.v1'),
      createEmptyInstance: create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListFlagsRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListFlagsRequest copyWith(void Function(ListFlagsRequest) updates) =>
      super.copyWith((message) => updates(message as ListFlagsRequest))
          as ListFlagsRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static ListFlagsRequest create() => ListFlagsRequest._();
  @$core.override
  ListFlagsRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static ListFlagsRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ListFlagsRequest>(create);
  static ListFlagsRequest? _defaultInstance;
}

class ListFlagsResponse extends $pb.GeneratedMessage {
  factory ListFlagsResponse({
    $core.Iterable<FeatureFlag>? flags,
  }) {
    final result = create();
    if (flags != null) result.flags.addAll(flags);
    return result;
  }

  ListFlagsResponse._();

  factory ListFlagsResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory ListFlagsResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ListFlagsResponse',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.featureflag.v1'),
      createEmptyInstance: create)
    ..pPM<FeatureFlag>(1, _omitFieldNames ? '' : 'flags',
        subBuilder: FeatureFlag.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListFlagsResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListFlagsResponse copyWith(void Function(ListFlagsResponse) updates) =>
      super.copyWith((message) => updates(message as ListFlagsResponse))
          as ListFlagsResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static ListFlagsResponse create() => ListFlagsResponse._();
  @$core.override
  ListFlagsResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static ListFlagsResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ListFlagsResponse>(create);
  static ListFlagsResponse? _defaultInstance;

  @$pb.TagNumber(1)
  $pb.PbList<FeatureFlag> get flags => $_getList(0);
}

class CreateFlagRequest extends $pb.GeneratedMessage {
  factory CreateFlagRequest({
    $core.String? flagKey,
    $core.String? description,
    $core.bool? enabled,
    $core.Iterable<FlagVariant>? variants,
  }) {
    final result = create();
    if (flagKey != null) result.flagKey = flagKey;
    if (description != null) result.description = description;
    if (enabled != null) result.enabled = enabled;
    if (variants != null) result.variants.addAll(variants);
    return result;
  }

  CreateFlagRequest._();

  factory CreateFlagRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory CreateFlagRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'CreateFlagRequest',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.featureflag.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'flagKey')
    ..aOS(2, _omitFieldNames ? '' : 'description')
    ..aOB(3, _omitFieldNames ? '' : 'enabled')
    ..pPM<FlagVariant>(4, _omitFieldNames ? '' : 'variants',
        subBuilder: FlagVariant.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  CreateFlagRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  CreateFlagRequest copyWith(void Function(CreateFlagRequest) updates) =>
      super.copyWith((message) => updates(message as CreateFlagRequest))
          as CreateFlagRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static CreateFlagRequest create() => CreateFlagRequest._();
  @$core.override
  CreateFlagRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static CreateFlagRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<CreateFlagRequest>(create);
  static CreateFlagRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get flagKey => $_getSZ(0);
  @$pb.TagNumber(1)
  set flagKey($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasFlagKey() => $_has(0);
  @$pb.TagNumber(1)
  void clearFlagKey() => $_clearField(1);

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
  $pb.PbList<FlagVariant> get variants => $_getList(3);
}

class CreateFlagResponse extends $pb.GeneratedMessage {
  factory CreateFlagResponse({
    FeatureFlag? flag,
  }) {
    final result = create();
    if (flag != null) result.flag = flag;
    return result;
  }

  CreateFlagResponse._();

  factory CreateFlagResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory CreateFlagResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'CreateFlagResponse',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.featureflag.v1'),
      createEmptyInstance: create)
    ..aOM<FeatureFlag>(1, _omitFieldNames ? '' : 'flag',
        subBuilder: FeatureFlag.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  CreateFlagResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  CreateFlagResponse copyWith(void Function(CreateFlagResponse) updates) =>
      super.copyWith((message) => updates(message as CreateFlagResponse))
          as CreateFlagResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static CreateFlagResponse create() => CreateFlagResponse._();
  @$core.override
  CreateFlagResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static CreateFlagResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<CreateFlagResponse>(create);
  static CreateFlagResponse? _defaultInstance;

  @$pb.TagNumber(1)
  FeatureFlag get flag => $_getN(0);
  @$pb.TagNumber(1)
  set flag(FeatureFlag value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasFlag() => $_has(0);
  @$pb.TagNumber(1)
  void clearFlag() => $_clearField(1);
  @$pb.TagNumber(1)
  FeatureFlag ensureFlag() => $_ensure(0);
}

class UpdateFlagRequest extends $pb.GeneratedMessage {
  factory UpdateFlagRequest({
    $core.String? flagKey,
    $core.bool? enabled,
    $core.String? description,
    $core.Iterable<FlagVariant>? variants,
    $core.Iterable<FlagRule>? rules,
  }) {
    final result = create();
    if (flagKey != null) result.flagKey = flagKey;
    if (enabled != null) result.enabled = enabled;
    if (description != null) result.description = description;
    if (variants != null) result.variants.addAll(variants);
    if (rules != null) result.rules.addAll(rules);
    return result;
  }

  UpdateFlagRequest._();

  factory UpdateFlagRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory UpdateFlagRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'UpdateFlagRequest',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.featureflag.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'flagKey')
    ..aOB(2, _omitFieldNames ? '' : 'enabled')
    ..aOS(3, _omitFieldNames ? '' : 'description')
    ..pPM<FlagVariant>(4, _omitFieldNames ? '' : 'variants',
        subBuilder: FlagVariant.create)
    ..pPM<FlagRule>(5, _omitFieldNames ? '' : 'rules',
        subBuilder: FlagRule.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  UpdateFlagRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  UpdateFlagRequest copyWith(void Function(UpdateFlagRequest) updates) =>
      super.copyWith((message) => updates(message as UpdateFlagRequest))
          as UpdateFlagRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static UpdateFlagRequest create() => UpdateFlagRequest._();
  @$core.override
  UpdateFlagRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static UpdateFlagRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<UpdateFlagRequest>(create);
  static UpdateFlagRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get flagKey => $_getSZ(0);
  @$pb.TagNumber(1)
  set flagKey($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasFlagKey() => $_has(0);
  @$pb.TagNumber(1)
  void clearFlagKey() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.bool get enabled => $_getBF(1);
  @$pb.TagNumber(2)
  set enabled($core.bool value) => $_setBool(1, value);
  @$pb.TagNumber(2)
  $core.bool hasEnabled() => $_has(1);
  @$pb.TagNumber(2)
  void clearEnabled() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.String get description => $_getSZ(2);
  @$pb.TagNumber(3)
  set description($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasDescription() => $_has(2);
  @$pb.TagNumber(3)
  void clearDescription() => $_clearField(3);

  @$pb.TagNumber(4)
  $pb.PbList<FlagVariant> get variants => $_getList(3);

  @$pb.TagNumber(5)
  $pb.PbList<FlagRule> get rules => $_getList(4);
}

class UpdateFlagResponse extends $pb.GeneratedMessage {
  factory UpdateFlagResponse({
    FeatureFlag? flag,
  }) {
    final result = create();
    if (flag != null) result.flag = flag;
    return result;
  }

  UpdateFlagResponse._();

  factory UpdateFlagResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory UpdateFlagResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'UpdateFlagResponse',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.featureflag.v1'),
      createEmptyInstance: create)
    ..aOM<FeatureFlag>(1, _omitFieldNames ? '' : 'flag',
        subBuilder: FeatureFlag.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  UpdateFlagResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  UpdateFlagResponse copyWith(void Function(UpdateFlagResponse) updates) =>
      super.copyWith((message) => updates(message as UpdateFlagResponse))
          as UpdateFlagResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static UpdateFlagResponse create() => UpdateFlagResponse._();
  @$core.override
  UpdateFlagResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static UpdateFlagResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<UpdateFlagResponse>(create);
  static UpdateFlagResponse? _defaultInstance;

  @$pb.TagNumber(1)
  FeatureFlag get flag => $_getN(0);
  @$pb.TagNumber(1)
  set flag(FeatureFlag value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasFlag() => $_has(0);
  @$pb.TagNumber(1)
  void clearFlag() => $_clearField(1);
  @$pb.TagNumber(1)
  FeatureFlag ensureFlag() => $_ensure(0);
}

class DeleteFlagRequest extends $pb.GeneratedMessage {
  factory DeleteFlagRequest({
    $core.String? flagKey,
  }) {
    final result = create();
    if (flagKey != null) result.flagKey = flagKey;
    return result;
  }

  DeleteFlagRequest._();

  factory DeleteFlagRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory DeleteFlagRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'DeleteFlagRequest',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.featureflag.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'flagKey')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  DeleteFlagRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  DeleteFlagRequest copyWith(void Function(DeleteFlagRequest) updates) =>
      super.copyWith((message) => updates(message as DeleteFlagRequest))
          as DeleteFlagRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static DeleteFlagRequest create() => DeleteFlagRequest._();
  @$core.override
  DeleteFlagRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static DeleteFlagRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<DeleteFlagRequest>(create);
  static DeleteFlagRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get flagKey => $_getSZ(0);
  @$pb.TagNumber(1)
  set flagKey($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasFlagKey() => $_has(0);
  @$pb.TagNumber(1)
  void clearFlagKey() => $_clearField(1);
}

class DeleteFlagResponse extends $pb.GeneratedMessage {
  factory DeleteFlagResponse({
    $core.bool? success,
    $core.String? message,
  }) {
    final result = create();
    if (success != null) result.success = success;
    if (message != null) result.message = message;
    return result;
  }

  DeleteFlagResponse._();

  factory DeleteFlagResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory DeleteFlagResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'DeleteFlagResponse',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.featureflag.v1'),
      createEmptyInstance: create)
    ..aOB(1, _omitFieldNames ? '' : 'success')
    ..aOS(2, _omitFieldNames ? '' : 'message')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  DeleteFlagResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  DeleteFlagResponse copyWith(void Function(DeleteFlagResponse) updates) =>
      super.copyWith((message) => updates(message as DeleteFlagResponse))
          as DeleteFlagResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static DeleteFlagResponse create() => DeleteFlagResponse._();
  @$core.override
  DeleteFlagResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static DeleteFlagResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<DeleteFlagResponse>(create);
  static DeleteFlagResponse? _defaultInstance;

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

class FeatureFlag extends $pb.GeneratedMessage {
  factory FeatureFlag({
    $core.String? id,
    $core.String? flagKey,
    $core.String? description,
    $core.bool? enabled,
    $core.Iterable<FlagVariant>? variants,
    $1.Timestamp? createdAt,
    $1.Timestamp? updatedAt,
    $core.Iterable<FlagRule>? rules,
  }) {
    final result = create();
    if (id != null) result.id = id;
    if (flagKey != null) result.flagKey = flagKey;
    if (description != null) result.description = description;
    if (enabled != null) result.enabled = enabled;
    if (variants != null) result.variants.addAll(variants);
    if (createdAt != null) result.createdAt = createdAt;
    if (updatedAt != null) result.updatedAt = updatedAt;
    if (rules != null) result.rules.addAll(rules);
    return result;
  }

  FeatureFlag._();

  factory FeatureFlag.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory FeatureFlag.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'FeatureFlag',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.featureflag.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'id')
    ..aOS(2, _omitFieldNames ? '' : 'flagKey')
    ..aOS(3, _omitFieldNames ? '' : 'description')
    ..aOB(4, _omitFieldNames ? '' : 'enabled')
    ..pPM<FlagVariant>(5, _omitFieldNames ? '' : 'variants',
        subBuilder: FlagVariant.create)
    ..aOM<$1.Timestamp>(6, _omitFieldNames ? '' : 'createdAt',
        subBuilder: $1.Timestamp.create)
    ..aOM<$1.Timestamp>(7, _omitFieldNames ? '' : 'updatedAt',
        subBuilder: $1.Timestamp.create)
    ..pPM<FlagRule>(8, _omitFieldNames ? '' : 'rules',
        subBuilder: FlagRule.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  FeatureFlag clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  FeatureFlag copyWith(void Function(FeatureFlag) updates) =>
      super.copyWith((message) => updates(message as FeatureFlag))
          as FeatureFlag;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static FeatureFlag create() => FeatureFlag._();
  @$core.override
  FeatureFlag createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static FeatureFlag getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<FeatureFlag>(create);
  static FeatureFlag? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get id => $_getSZ(0);
  @$pb.TagNumber(1)
  set id($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasId() => $_has(0);
  @$pb.TagNumber(1)
  void clearId() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get flagKey => $_getSZ(1);
  @$pb.TagNumber(2)
  set flagKey($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasFlagKey() => $_has(1);
  @$pb.TagNumber(2)
  void clearFlagKey() => $_clearField(2);

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
  $pb.PbList<FlagVariant> get variants => $_getList(4);

  @$pb.TagNumber(6)
  $1.Timestamp get createdAt => $_getN(5);
  @$pb.TagNumber(6)
  set createdAt($1.Timestamp value) => $_setField(6, value);
  @$pb.TagNumber(6)
  $core.bool hasCreatedAt() => $_has(5);
  @$pb.TagNumber(6)
  void clearCreatedAt() => $_clearField(6);
  @$pb.TagNumber(6)
  $1.Timestamp ensureCreatedAt() => $_ensure(5);

  @$pb.TagNumber(7)
  $1.Timestamp get updatedAt => $_getN(6);
  @$pb.TagNumber(7)
  set updatedAt($1.Timestamp value) => $_setField(7, value);
  @$pb.TagNumber(7)
  $core.bool hasUpdatedAt() => $_has(6);
  @$pb.TagNumber(7)
  void clearUpdatedAt() => $_clearField(7);
  @$pb.TagNumber(7)
  $1.Timestamp ensureUpdatedAt() => $_ensure(6);

  @$pb.TagNumber(8)
  $pb.PbList<FlagRule> get rules => $_getList(7);
}

class FlagVariant extends $pb.GeneratedMessage {
  factory FlagVariant({
    $core.String? name,
    $core.String? value,
    $core.int? weight,
  }) {
    final result = create();
    if (name != null) result.name = name;
    if (value != null) result.value = value;
    if (weight != null) result.weight = weight;
    return result;
  }

  FlagVariant._();

  factory FlagVariant.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory FlagVariant.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'FlagVariant',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.featureflag.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'name')
    ..aOS(2, _omitFieldNames ? '' : 'value')
    ..aI(3, _omitFieldNames ? '' : 'weight')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  FlagVariant clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  FlagVariant copyWith(void Function(FlagVariant) updates) =>
      super.copyWith((message) => updates(message as FlagVariant))
          as FlagVariant;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static FlagVariant create() => FlagVariant._();
  @$core.override
  FlagVariant createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static FlagVariant getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<FlagVariant>(create);
  static FlagVariant? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get name => $_getSZ(0);
  @$pb.TagNumber(1)
  set name($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasName() => $_has(0);
  @$pb.TagNumber(1)
  void clearName() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get value => $_getSZ(1);
  @$pb.TagNumber(2)
  set value($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasValue() => $_has(1);
  @$pb.TagNumber(2)
  void clearValue() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.int get weight => $_getIZ(2);
  @$pb.TagNumber(3)
  set weight($core.int value) => $_setSignedInt32(2, value);
  @$pb.TagNumber(3)
  $core.bool hasWeight() => $_has(2);
  @$pb.TagNumber(3)
  void clearWeight() => $_clearField(3);
}

class FlagRule extends $pb.GeneratedMessage {
  factory FlagRule({
    $core.String? attribute,
    $core.String? operator,
    $core.String? value,
    $core.String? variant,
  }) {
    final result = create();
    if (attribute != null) result.attribute = attribute;
    if (operator != null) result.operator = operator;
    if (value != null) result.value = value;
    if (variant != null) result.variant = variant;
    return result;
  }

  FlagRule._();

  factory FlagRule.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory FlagRule.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'FlagRule',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.featureflag.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'attribute')
    ..aOS(2, _omitFieldNames ? '' : 'operator')
    ..aOS(3, _omitFieldNames ? '' : 'value')
    ..aOS(4, _omitFieldNames ? '' : 'variant')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  FlagRule clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  FlagRule copyWith(void Function(FlagRule) updates) =>
      super.copyWith((message) => updates(message as FlagRule)) as FlagRule;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static FlagRule create() => FlagRule._();
  @$core.override
  FlagRule createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static FlagRule getDefault() =>
      _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<FlagRule>(create);
  static FlagRule? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get attribute => $_getSZ(0);
  @$pb.TagNumber(1)
  set attribute($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasAttribute() => $_has(0);
  @$pb.TagNumber(1)
  void clearAttribute() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get operator => $_getSZ(1);
  @$pb.TagNumber(2)
  set operator($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasOperator() => $_has(1);
  @$pb.TagNumber(2)
  void clearOperator() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.String get value => $_getSZ(2);
  @$pb.TagNumber(3)
  set value($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasValue() => $_has(2);
  @$pb.TagNumber(3)
  void clearValue() => $_clearField(3);

  @$pb.TagNumber(4)
  $core.String get variant => $_getSZ(3);
  @$pb.TagNumber(4)
  set variant($core.String value) => $_setString(3, value);
  @$pb.TagNumber(4)
  $core.bool hasVariant() => $_has(3);
  @$pb.TagNumber(4)
  void clearVariant() => $_clearField(4);
}

/// WatchFeatureFlagRequest はフラグ変更監視リクエスト。
class WatchFeatureFlagRequest extends $pb.GeneratedMessage {
  factory WatchFeatureFlagRequest({
    $core.String? flagKey,
  }) {
    final result = create();
    if (flagKey != null) result.flagKey = flagKey;
    return result;
  }

  WatchFeatureFlagRequest._();

  factory WatchFeatureFlagRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory WatchFeatureFlagRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'WatchFeatureFlagRequest',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.featureflag.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'flagKey')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  WatchFeatureFlagRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  WatchFeatureFlagRequest copyWith(
          void Function(WatchFeatureFlagRequest) updates) =>
      super.copyWith((message) => updates(message as WatchFeatureFlagRequest))
          as WatchFeatureFlagRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static WatchFeatureFlagRequest create() => WatchFeatureFlagRequest._();
  @$core.override
  WatchFeatureFlagRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static WatchFeatureFlagRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<WatchFeatureFlagRequest>(create);
  static WatchFeatureFlagRequest? _defaultInstance;

  /// 監視対象のフラグキー（空の場合は全フラグの変更を受け取る）
  @$pb.TagNumber(1)
  $core.String get flagKey => $_getSZ(0);
  @$pb.TagNumber(1)
  set flagKey($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasFlagKey() => $_has(0);
  @$pb.TagNumber(1)
  void clearFlagKey() => $_clearField(1);
}

/// WatchFeatureFlagResponse はフラグ変更の監視レスポンス（ストリーミング）。
class WatchFeatureFlagResponse extends $pb.GeneratedMessage {
  factory WatchFeatureFlagResponse({
    $core.String? flagKey,
    $core.String? changeType,
    FeatureFlag? flag,
    $1.Timestamp? changedAt,
    $1.ChangeType? changeTypeEnum,
  }) {
    final result = create();
    if (flagKey != null) result.flagKey = flagKey;
    if (changeType != null) result.changeType = changeType;
    if (flag != null) result.flag = flag;
    if (changedAt != null) result.changedAt = changedAt;
    if (changeTypeEnum != null) result.changeTypeEnum = changeTypeEnum;
    return result;
  }

  WatchFeatureFlagResponse._();

  factory WatchFeatureFlagResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory WatchFeatureFlagResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'WatchFeatureFlagResponse',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.featureflag.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'flagKey')
    ..aOS(2, _omitFieldNames ? '' : 'changeType')
    ..aOM<FeatureFlag>(3, _omitFieldNames ? '' : 'flag',
        subBuilder: FeatureFlag.create)
    ..aOM<$1.Timestamp>(4, _omitFieldNames ? '' : 'changedAt',
        subBuilder: $1.Timestamp.create)
    ..aE<$1.ChangeType>(5, _omitFieldNames ? '' : 'changeTypeEnum',
        enumValues: $1.ChangeType.values)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  WatchFeatureFlagResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  WatchFeatureFlagResponse copyWith(
          void Function(WatchFeatureFlagResponse) updates) =>
      super.copyWith((message) => updates(message as WatchFeatureFlagResponse))
          as WatchFeatureFlagResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static WatchFeatureFlagResponse create() => WatchFeatureFlagResponse._();
  @$core.override
  WatchFeatureFlagResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static WatchFeatureFlagResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<WatchFeatureFlagResponse>(create);
  static WatchFeatureFlagResponse? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get flagKey => $_getSZ(0);
  @$pb.TagNumber(1)
  set flagKey($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasFlagKey() => $_has(0);
  @$pb.TagNumber(1)
  void clearFlagKey() => $_clearField(1);

  /// Deprecated: use change_type_enum instead.
  /// CREATED, UPDATED, DELETED
  @$pb.TagNumber(2)
  $core.String get changeType => $_getSZ(1);
  @$pb.TagNumber(2)
  set changeType($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasChangeType() => $_has(1);
  @$pb.TagNumber(2)
  void clearChangeType() => $_clearField(2);

  @$pb.TagNumber(3)
  FeatureFlag get flag => $_getN(2);
  @$pb.TagNumber(3)
  set flag(FeatureFlag value) => $_setField(3, value);
  @$pb.TagNumber(3)
  $core.bool hasFlag() => $_has(2);
  @$pb.TagNumber(3)
  void clearFlag() => $_clearField(3);
  @$pb.TagNumber(3)
  FeatureFlag ensureFlag() => $_ensure(2);

  @$pb.TagNumber(4)
  $1.Timestamp get changedAt => $_getN(3);
  @$pb.TagNumber(4)
  set changedAt($1.Timestamp value) => $_setField(4, value);
  @$pb.TagNumber(4)
  $core.bool hasChangedAt() => $_has(3);
  @$pb.TagNumber(4)
  void clearChangedAt() => $_clearField(4);
  @$pb.TagNumber(4)
  $1.Timestamp ensureChangedAt() => $_ensure(3);

  /// 変更操作の種別（change_type の enum 版）。
  @$pb.TagNumber(5)
  $1.ChangeType get changeTypeEnum => $_getN(4);
  @$pb.TagNumber(5)
  set changeTypeEnum($1.ChangeType value) => $_setField(5, value);
  @$pb.TagNumber(5)
  $core.bool hasChangeTypeEnum() => $_has(4);
  @$pb.TagNumber(5)
  void clearChangeTypeEnum() => $_clearField(5);
}

const $core.bool _omitFieldNames =
    $core.bool.fromEnvironment('protobuf.omit_field_names');
const $core.bool _omitMessageNames =
    $core.bool.fromEnvironment('protobuf.omit_message_names');
