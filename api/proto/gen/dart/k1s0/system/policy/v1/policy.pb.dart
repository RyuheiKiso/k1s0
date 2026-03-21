// This is a generated file - do not edit.
//
// Generated from k1s0/system/policy/v1/policy.proto.

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

class EvaluatePolicyRequest extends $pb.GeneratedMessage {
  factory EvaluatePolicyRequest({
    $core.String? policyId,
    $core.List<$core.int>? inputJson,
  }) {
    final result = create();
    if (policyId != null) result.policyId = policyId;
    if (inputJson != null) result.inputJson = inputJson;
    return result;
  }

  EvaluatePolicyRequest._();

  factory EvaluatePolicyRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory EvaluatePolicyRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'EvaluatePolicyRequest',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.policy.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'policyId')
    ..a<$core.List<$core.int>>(
        2, _omitFieldNames ? '' : 'inputJson', $pb.PbFieldType.OY)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  EvaluatePolicyRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  EvaluatePolicyRequest copyWith(
          void Function(EvaluatePolicyRequest) updates) =>
      super.copyWith((message) => updates(message as EvaluatePolicyRequest))
          as EvaluatePolicyRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static EvaluatePolicyRequest create() => EvaluatePolicyRequest._();
  @$core.override
  EvaluatePolicyRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static EvaluatePolicyRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<EvaluatePolicyRequest>(create);
  static EvaluatePolicyRequest? _defaultInstance;

  /// 評価対象 Policy ID
  @$pb.TagNumber(1)
  $core.String get policyId => $_getSZ(0);
  @$pb.TagNumber(1)
  set policyId($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasPolicyId() => $_has(0);
  @$pb.TagNumber(1)
  void clearPolicyId() => $_clearField(1);

  /// 評価入力データ（JSON バイト列）
  @$pb.TagNumber(2)
  $core.List<$core.int> get inputJson => $_getN(1);
  @$pb.TagNumber(2)
  set inputJson($core.List<$core.int> value) => $_setBytes(1, value);
  @$pb.TagNumber(2)
  $core.bool hasInputJson() => $_has(1);
  @$pb.TagNumber(2)
  void clearInputJson() => $_clearField(2);
}

class EvaluatePolicyResponse extends $pb.GeneratedMessage {
  factory EvaluatePolicyResponse({
    $core.bool? allowed,
    $core.String? packagePath,
    $core.String? decisionId,
    $core.bool? cached,
  }) {
    final result = create();
    if (allowed != null) result.allowed = allowed;
    if (packagePath != null) result.packagePath = packagePath;
    if (decisionId != null) result.decisionId = decisionId;
    if (cached != null) result.cached = cached;
    return result;
  }

  EvaluatePolicyResponse._();

  factory EvaluatePolicyResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory EvaluatePolicyResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'EvaluatePolicyResponse',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.policy.v1'),
      createEmptyInstance: create)
    ..aOB(1, _omitFieldNames ? '' : 'allowed')
    ..aOS(2, _omitFieldNames ? '' : 'packagePath')
    ..aOS(3, _omitFieldNames ? '' : 'decisionId')
    ..aOB(4, _omitFieldNames ? '' : 'cached')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  EvaluatePolicyResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  EvaluatePolicyResponse copyWith(
          void Function(EvaluatePolicyResponse) updates) =>
      super.copyWith((message) => updates(message as EvaluatePolicyResponse))
          as EvaluatePolicyResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static EvaluatePolicyResponse create() => EvaluatePolicyResponse._();
  @$core.override
  EvaluatePolicyResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static EvaluatePolicyResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<EvaluatePolicyResponse>(create);
  static EvaluatePolicyResponse? _defaultInstance;

  /// 評価結果（true: 許可 / false: 拒否）
  @$pb.TagNumber(1)
  $core.bool get allowed => $_getBF(0);
  @$pb.TagNumber(1)
  set allowed($core.bool value) => $_setBool(0, value);
  @$pb.TagNumber(1)
  $core.bool hasAllowed() => $_has(0);
  @$pb.TagNumber(1)
  void clearAllowed() => $_clearField(1);

  /// 評価対象 Rego パッケージパス
  @$pb.TagNumber(2)
  $core.String get packagePath => $_getSZ(1);
  @$pb.TagNumber(2)
  set packagePath($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasPackagePath() => $_has(1);
  @$pb.TagNumber(2)
  void clearPackagePath() => $_clearField(2);

  /// OPA 評価 ID
  @$pb.TagNumber(3)
  $core.String get decisionId => $_getSZ(2);
  @$pb.TagNumber(3)
  set decisionId($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasDecisionId() => $_has(2);
  @$pb.TagNumber(3)
  void clearDecisionId() => $_clearField(3);

  /// キャッシュヒットフラグ
  @$pb.TagNumber(4)
  $core.bool get cached => $_getBF(3);
  @$pb.TagNumber(4)
  set cached($core.bool value) => $_setBool(3, value);
  @$pb.TagNumber(4)
  $core.bool hasCached() => $_has(3);
  @$pb.TagNumber(4)
  void clearCached() => $_clearField(4);
}

class GetPolicyRequest extends $pb.GeneratedMessage {
  factory GetPolicyRequest({
    $core.String? id,
  }) {
    final result = create();
    if (id != null) result.id = id;
    return result;
  }

  GetPolicyRequest._();

  factory GetPolicyRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory GetPolicyRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'GetPolicyRequest',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.policy.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'id')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetPolicyRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetPolicyRequest copyWith(void Function(GetPolicyRequest) updates) =>
      super.copyWith((message) => updates(message as GetPolicyRequest))
          as GetPolicyRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static GetPolicyRequest create() => GetPolicyRequest._();
  @$core.override
  GetPolicyRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static GetPolicyRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<GetPolicyRequest>(create);
  static GetPolicyRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get id => $_getSZ(0);
  @$pb.TagNumber(1)
  set id($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasId() => $_has(0);
  @$pb.TagNumber(1)
  void clearId() => $_clearField(1);
}

class GetPolicyResponse extends $pb.GeneratedMessage {
  factory GetPolicyResponse({
    Policy? policy,
  }) {
    final result = create();
    if (policy != null) result.policy = policy;
    return result;
  }

  GetPolicyResponse._();

  factory GetPolicyResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory GetPolicyResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'GetPolicyResponse',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.policy.v1'),
      createEmptyInstance: create)
    ..aOM<Policy>(1, _omitFieldNames ? '' : 'policy', subBuilder: Policy.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetPolicyResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetPolicyResponse copyWith(void Function(GetPolicyResponse) updates) =>
      super.copyWith((message) => updates(message as GetPolicyResponse))
          as GetPolicyResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static GetPolicyResponse create() => GetPolicyResponse._();
  @$core.override
  GetPolicyResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static GetPolicyResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<GetPolicyResponse>(create);
  static GetPolicyResponse? _defaultInstance;

  @$pb.TagNumber(1)
  Policy get policy => $_getN(0);
  @$pb.TagNumber(1)
  set policy(Policy value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasPolicy() => $_has(0);
  @$pb.TagNumber(1)
  void clearPolicy() => $_clearField(1);
  @$pb.TagNumber(1)
  Policy ensurePolicy() => $_ensure(0);
}

class ListPoliciesRequest extends $pb.GeneratedMessage {
  factory ListPoliciesRequest({
    $1.Pagination? pagination,
    $core.String? bundleId,
    $core.bool? enabledOnly,
  }) {
    final result = create();
    if (pagination != null) result.pagination = pagination;
    if (bundleId != null) result.bundleId = bundleId;
    if (enabledOnly != null) result.enabledOnly = enabledOnly;
    return result;
  }

  ListPoliciesRequest._();

  factory ListPoliciesRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory ListPoliciesRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ListPoliciesRequest',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.policy.v1'),
      createEmptyInstance: create)
    ..aOM<$1.Pagination>(1, _omitFieldNames ? '' : 'pagination',
        subBuilder: $1.Pagination.create)
    ..aOS(2, _omitFieldNames ? '' : 'bundleId')
    ..aOB(3, _omitFieldNames ? '' : 'enabledOnly')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListPoliciesRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListPoliciesRequest copyWith(void Function(ListPoliciesRequest) updates) =>
      super.copyWith((message) => updates(message as ListPoliciesRequest))
          as ListPoliciesRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static ListPoliciesRequest create() => ListPoliciesRequest._();
  @$core.override
  ListPoliciesRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static ListPoliciesRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ListPoliciesRequest>(create);
  static ListPoliciesRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $1.Pagination get pagination => $_getN(0);
  @$pb.TagNumber(1)
  set pagination($1.Pagination value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasPagination() => $_has(0);
  @$pb.TagNumber(1)
  void clearPagination() => $_clearField(1);
  @$pb.TagNumber(1)
  $1.Pagination ensurePagination() => $_ensure(0);

  @$pb.TagNumber(2)
  $core.String get bundleId => $_getSZ(1);
  @$pb.TagNumber(2)
  set bundleId($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasBundleId() => $_has(1);
  @$pb.TagNumber(2)
  void clearBundleId() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.bool get enabledOnly => $_getBF(2);
  @$pb.TagNumber(3)
  set enabledOnly($core.bool value) => $_setBool(2, value);
  @$pb.TagNumber(3)
  $core.bool hasEnabledOnly() => $_has(2);
  @$pb.TagNumber(3)
  void clearEnabledOnly() => $_clearField(3);
}

class ListPoliciesResponse extends $pb.GeneratedMessage {
  factory ListPoliciesResponse({
    $core.Iterable<Policy>? policies,
    $1.PaginationResult? pagination,
  }) {
    final result = create();
    if (policies != null) result.policies.addAll(policies);
    if (pagination != null) result.pagination = pagination;
    return result;
  }

  ListPoliciesResponse._();

  factory ListPoliciesResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory ListPoliciesResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ListPoliciesResponse',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.policy.v1'),
      createEmptyInstance: create)
    ..pPM<Policy>(1, _omitFieldNames ? '' : 'policies',
        subBuilder: Policy.create)
    ..aOM<$1.PaginationResult>(2, _omitFieldNames ? '' : 'pagination',
        subBuilder: $1.PaginationResult.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListPoliciesResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListPoliciesResponse copyWith(void Function(ListPoliciesResponse) updates) =>
      super.copyWith((message) => updates(message as ListPoliciesResponse))
          as ListPoliciesResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static ListPoliciesResponse create() => ListPoliciesResponse._();
  @$core.override
  ListPoliciesResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static ListPoliciesResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ListPoliciesResponse>(create);
  static ListPoliciesResponse? _defaultInstance;

  @$pb.TagNumber(1)
  $pb.PbList<Policy> get policies => $_getList(0);

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

class CreatePolicyRequest extends $pb.GeneratedMessage {
  factory CreatePolicyRequest({
    $core.String? name,
    $core.String? description,
    $core.String? regoContent,
    $core.String? packagePath,
    $core.String? bundleId,
  }) {
    final result = create();
    if (name != null) result.name = name;
    if (description != null) result.description = description;
    if (regoContent != null) result.regoContent = regoContent;
    if (packagePath != null) result.packagePath = packagePath;
    if (bundleId != null) result.bundleId = bundleId;
    return result;
  }

  CreatePolicyRequest._();

  factory CreatePolicyRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory CreatePolicyRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'CreatePolicyRequest',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.policy.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'name')
    ..aOS(2, _omitFieldNames ? '' : 'description')
    ..aOS(3, _omitFieldNames ? '' : 'regoContent')
    ..aOS(4, _omitFieldNames ? '' : 'packagePath')
    ..aOS(5, _omitFieldNames ? '' : 'bundleId')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  CreatePolicyRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  CreatePolicyRequest copyWith(void Function(CreatePolicyRequest) updates) =>
      super.copyWith((message) => updates(message as CreatePolicyRequest))
          as CreatePolicyRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static CreatePolicyRequest create() => CreatePolicyRequest._();
  @$core.override
  CreatePolicyRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static CreatePolicyRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<CreatePolicyRequest>(create);
  static CreatePolicyRequest? _defaultInstance;

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
  $core.String get regoContent => $_getSZ(2);
  @$pb.TagNumber(3)
  set regoContent($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasRegoContent() => $_has(2);
  @$pb.TagNumber(3)
  void clearRegoContent() => $_clearField(3);

  @$pb.TagNumber(4)
  $core.String get packagePath => $_getSZ(3);
  @$pb.TagNumber(4)
  set packagePath($core.String value) => $_setString(3, value);
  @$pb.TagNumber(4)
  $core.bool hasPackagePath() => $_has(3);
  @$pb.TagNumber(4)
  void clearPackagePath() => $_clearField(4);

  @$pb.TagNumber(5)
  $core.String get bundleId => $_getSZ(4);
  @$pb.TagNumber(5)
  set bundleId($core.String value) => $_setString(4, value);
  @$pb.TagNumber(5)
  $core.bool hasBundleId() => $_has(4);
  @$pb.TagNumber(5)
  void clearBundleId() => $_clearField(5);
}

class CreatePolicyResponse extends $pb.GeneratedMessage {
  factory CreatePolicyResponse({
    Policy? policy,
  }) {
    final result = create();
    if (policy != null) result.policy = policy;
    return result;
  }

  CreatePolicyResponse._();

  factory CreatePolicyResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory CreatePolicyResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'CreatePolicyResponse',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.policy.v1'),
      createEmptyInstance: create)
    ..aOM<Policy>(1, _omitFieldNames ? '' : 'policy', subBuilder: Policy.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  CreatePolicyResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  CreatePolicyResponse copyWith(void Function(CreatePolicyResponse) updates) =>
      super.copyWith((message) => updates(message as CreatePolicyResponse))
          as CreatePolicyResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static CreatePolicyResponse create() => CreatePolicyResponse._();
  @$core.override
  CreatePolicyResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static CreatePolicyResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<CreatePolicyResponse>(create);
  static CreatePolicyResponse? _defaultInstance;

  @$pb.TagNumber(1)
  Policy get policy => $_getN(0);
  @$pb.TagNumber(1)
  set policy(Policy value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasPolicy() => $_has(0);
  @$pb.TagNumber(1)
  void clearPolicy() => $_clearField(1);
  @$pb.TagNumber(1)
  Policy ensurePolicy() => $_ensure(0);
}

class UpdatePolicyRequest extends $pb.GeneratedMessage {
  factory UpdatePolicyRequest({
    $core.String? id,
    $core.String? description,
    $core.String? regoContent,
    $core.bool? enabled,
  }) {
    final result = create();
    if (id != null) result.id = id;
    if (description != null) result.description = description;
    if (regoContent != null) result.regoContent = regoContent;
    if (enabled != null) result.enabled = enabled;
    return result;
  }

  UpdatePolicyRequest._();

  factory UpdatePolicyRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory UpdatePolicyRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'UpdatePolicyRequest',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.policy.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'id')
    ..aOS(2, _omitFieldNames ? '' : 'description')
    ..aOS(3, _omitFieldNames ? '' : 'regoContent')
    ..aOB(4, _omitFieldNames ? '' : 'enabled')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  UpdatePolicyRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  UpdatePolicyRequest copyWith(void Function(UpdatePolicyRequest) updates) =>
      super.copyWith((message) => updates(message as UpdatePolicyRequest))
          as UpdatePolicyRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static UpdatePolicyRequest create() => UpdatePolicyRequest._();
  @$core.override
  UpdatePolicyRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static UpdatePolicyRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<UpdatePolicyRequest>(create);
  static UpdatePolicyRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get id => $_getSZ(0);
  @$pb.TagNumber(1)
  set id($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasId() => $_has(0);
  @$pb.TagNumber(1)
  void clearId() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get description => $_getSZ(1);
  @$pb.TagNumber(2)
  set description($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasDescription() => $_has(1);
  @$pb.TagNumber(2)
  void clearDescription() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.String get regoContent => $_getSZ(2);
  @$pb.TagNumber(3)
  set regoContent($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasRegoContent() => $_has(2);
  @$pb.TagNumber(3)
  void clearRegoContent() => $_clearField(3);

  @$pb.TagNumber(4)
  $core.bool get enabled => $_getBF(3);
  @$pb.TagNumber(4)
  set enabled($core.bool value) => $_setBool(3, value);
  @$pb.TagNumber(4)
  $core.bool hasEnabled() => $_has(3);
  @$pb.TagNumber(4)
  void clearEnabled() => $_clearField(4);
}

class UpdatePolicyResponse extends $pb.GeneratedMessage {
  factory UpdatePolicyResponse({
    Policy? policy,
  }) {
    final result = create();
    if (policy != null) result.policy = policy;
    return result;
  }

  UpdatePolicyResponse._();

  factory UpdatePolicyResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory UpdatePolicyResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'UpdatePolicyResponse',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.policy.v1'),
      createEmptyInstance: create)
    ..aOM<Policy>(1, _omitFieldNames ? '' : 'policy', subBuilder: Policy.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  UpdatePolicyResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  UpdatePolicyResponse copyWith(void Function(UpdatePolicyResponse) updates) =>
      super.copyWith((message) => updates(message as UpdatePolicyResponse))
          as UpdatePolicyResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static UpdatePolicyResponse create() => UpdatePolicyResponse._();
  @$core.override
  UpdatePolicyResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static UpdatePolicyResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<UpdatePolicyResponse>(create);
  static UpdatePolicyResponse? _defaultInstance;

  @$pb.TagNumber(1)
  Policy get policy => $_getN(0);
  @$pb.TagNumber(1)
  set policy(Policy value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasPolicy() => $_has(0);
  @$pb.TagNumber(1)
  void clearPolicy() => $_clearField(1);
  @$pb.TagNumber(1)
  Policy ensurePolicy() => $_ensure(0);
}

class DeletePolicyRequest extends $pb.GeneratedMessage {
  factory DeletePolicyRequest({
    $core.String? id,
  }) {
    final result = create();
    if (id != null) result.id = id;
    return result;
  }

  DeletePolicyRequest._();

  factory DeletePolicyRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory DeletePolicyRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'DeletePolicyRequest',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.policy.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'id')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  DeletePolicyRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  DeletePolicyRequest copyWith(void Function(DeletePolicyRequest) updates) =>
      super.copyWith((message) => updates(message as DeletePolicyRequest))
          as DeletePolicyRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static DeletePolicyRequest create() => DeletePolicyRequest._();
  @$core.override
  DeletePolicyRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static DeletePolicyRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<DeletePolicyRequest>(create);
  static DeletePolicyRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get id => $_getSZ(0);
  @$pb.TagNumber(1)
  set id($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasId() => $_has(0);
  @$pb.TagNumber(1)
  void clearId() => $_clearField(1);
}

class DeletePolicyResponse extends $pb.GeneratedMessage {
  factory DeletePolicyResponse({
    $core.bool? success,
    $core.String? message,
  }) {
    final result = create();
    if (success != null) result.success = success;
    if (message != null) result.message = message;
    return result;
  }

  DeletePolicyResponse._();

  factory DeletePolicyResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory DeletePolicyResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'DeletePolicyResponse',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.policy.v1'),
      createEmptyInstance: create)
    ..aOB(1, _omitFieldNames ? '' : 'success')
    ..aOS(2, _omitFieldNames ? '' : 'message')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  DeletePolicyResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  DeletePolicyResponse copyWith(void Function(DeletePolicyResponse) updates) =>
      super.copyWith((message) => updates(message as DeletePolicyResponse))
          as DeletePolicyResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static DeletePolicyResponse create() => DeletePolicyResponse._();
  @$core.override
  DeletePolicyResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static DeletePolicyResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<DeletePolicyResponse>(create);
  static DeletePolicyResponse? _defaultInstance;

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

class CreateBundleRequest extends $pb.GeneratedMessage {
  factory CreateBundleRequest({
    $core.String? name,
    $core.Iterable<$core.String>? policyIds,
    $core.String? description,
    $core.bool? enabled,
  }) {
    final result = create();
    if (name != null) result.name = name;
    if (policyIds != null) result.policyIds.addAll(policyIds);
    if (description != null) result.description = description;
    if (enabled != null) result.enabled = enabled;
    return result;
  }

  CreateBundleRequest._();

  factory CreateBundleRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory CreateBundleRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'CreateBundleRequest',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.policy.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'name')
    ..pPS(2, _omitFieldNames ? '' : 'policyIds')
    ..aOS(3, _omitFieldNames ? '' : 'description')
    ..aOB(4, _omitFieldNames ? '' : 'enabled')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  CreateBundleRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  CreateBundleRequest copyWith(void Function(CreateBundleRequest) updates) =>
      super.copyWith((message) => updates(message as CreateBundleRequest))
          as CreateBundleRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static CreateBundleRequest create() => CreateBundleRequest._();
  @$core.override
  CreateBundleRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static CreateBundleRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<CreateBundleRequest>(create);
  static CreateBundleRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get name => $_getSZ(0);
  @$pb.TagNumber(1)
  set name($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasName() => $_has(0);
  @$pb.TagNumber(1)
  void clearName() => $_clearField(1);

  @$pb.TagNumber(2)
  $pb.PbList<$core.String> get policyIds => $_getList(1);

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
}

class CreateBundleResponse extends $pb.GeneratedMessage {
  factory CreateBundleResponse({
    PolicyBundle? bundle,
  }) {
    final result = create();
    if (bundle != null) result.bundle = bundle;
    return result;
  }

  CreateBundleResponse._();

  factory CreateBundleResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory CreateBundleResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'CreateBundleResponse',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.policy.v1'),
      createEmptyInstance: create)
    ..aOM<PolicyBundle>(1, _omitFieldNames ? '' : 'bundle',
        subBuilder: PolicyBundle.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  CreateBundleResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  CreateBundleResponse copyWith(void Function(CreateBundleResponse) updates) =>
      super.copyWith((message) => updates(message as CreateBundleResponse))
          as CreateBundleResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static CreateBundleResponse create() => CreateBundleResponse._();
  @$core.override
  CreateBundleResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static CreateBundleResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<CreateBundleResponse>(create);
  static CreateBundleResponse? _defaultInstance;

  @$pb.TagNumber(1)
  PolicyBundle get bundle => $_getN(0);
  @$pb.TagNumber(1)
  set bundle(PolicyBundle value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasBundle() => $_has(0);
  @$pb.TagNumber(1)
  void clearBundle() => $_clearField(1);
  @$pb.TagNumber(1)
  PolicyBundle ensureBundle() => $_ensure(0);
}

class ListBundlesRequest extends $pb.GeneratedMessage {
  factory ListBundlesRequest() => create();

  ListBundlesRequest._();

  factory ListBundlesRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory ListBundlesRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ListBundlesRequest',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.policy.v1'),
      createEmptyInstance: create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListBundlesRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListBundlesRequest copyWith(void Function(ListBundlesRequest) updates) =>
      super.copyWith((message) => updates(message as ListBundlesRequest))
          as ListBundlesRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static ListBundlesRequest create() => ListBundlesRequest._();
  @$core.override
  ListBundlesRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static ListBundlesRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ListBundlesRequest>(create);
  static ListBundlesRequest? _defaultInstance;
}

class ListBundlesResponse extends $pb.GeneratedMessage {
  factory ListBundlesResponse({
    $core.Iterable<PolicyBundle>? bundles,
  }) {
    final result = create();
    if (bundles != null) result.bundles.addAll(bundles);
    return result;
  }

  ListBundlesResponse._();

  factory ListBundlesResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory ListBundlesResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ListBundlesResponse',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.policy.v1'),
      createEmptyInstance: create)
    ..pPM<PolicyBundle>(1, _omitFieldNames ? '' : 'bundles',
        subBuilder: PolicyBundle.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListBundlesResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListBundlesResponse copyWith(void Function(ListBundlesResponse) updates) =>
      super.copyWith((message) => updates(message as ListBundlesResponse))
          as ListBundlesResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static ListBundlesResponse create() => ListBundlesResponse._();
  @$core.override
  ListBundlesResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static ListBundlesResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ListBundlesResponse>(create);
  static ListBundlesResponse? _defaultInstance;

  @$pb.TagNumber(1)
  $pb.PbList<PolicyBundle> get bundles => $_getList(0);
}

class GetBundleRequest extends $pb.GeneratedMessage {
  factory GetBundleRequest({
    $core.String? id,
  }) {
    final result = create();
    if (id != null) result.id = id;
    return result;
  }

  GetBundleRequest._();

  factory GetBundleRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory GetBundleRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'GetBundleRequest',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.policy.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'id')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetBundleRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetBundleRequest copyWith(void Function(GetBundleRequest) updates) =>
      super.copyWith((message) => updates(message as GetBundleRequest))
          as GetBundleRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static GetBundleRequest create() => GetBundleRequest._();
  @$core.override
  GetBundleRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static GetBundleRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<GetBundleRequest>(create);
  static GetBundleRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get id => $_getSZ(0);
  @$pb.TagNumber(1)
  set id($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasId() => $_has(0);
  @$pb.TagNumber(1)
  void clearId() => $_clearField(1);
}

class GetBundleResponse extends $pb.GeneratedMessage {
  factory GetBundleResponse({
    PolicyBundle? bundle,
  }) {
    final result = create();
    if (bundle != null) result.bundle = bundle;
    return result;
  }

  GetBundleResponse._();

  factory GetBundleResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory GetBundleResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'GetBundleResponse',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.policy.v1'),
      createEmptyInstance: create)
    ..aOM<PolicyBundle>(1, _omitFieldNames ? '' : 'bundle',
        subBuilder: PolicyBundle.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetBundleResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetBundleResponse copyWith(void Function(GetBundleResponse) updates) =>
      super.copyWith((message) => updates(message as GetBundleResponse))
          as GetBundleResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static GetBundleResponse create() => GetBundleResponse._();
  @$core.override
  GetBundleResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static GetBundleResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<GetBundleResponse>(create);
  static GetBundleResponse? _defaultInstance;

  @$pb.TagNumber(1)
  PolicyBundle get bundle => $_getN(0);
  @$pb.TagNumber(1)
  set bundle(PolicyBundle value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasBundle() => $_has(0);
  @$pb.TagNumber(1)
  void clearBundle() => $_clearField(1);
  @$pb.TagNumber(1)
  PolicyBundle ensureBundle() => $_ensure(0);
}

class Policy extends $pb.GeneratedMessage {
  factory Policy({
    $core.String? id,
    $core.String? name,
    $core.String? description,
    $core.String? packagePath,
    $core.String? regoContent,
    $core.String? bundleId,
    $core.bool? enabled,
    $core.int? version,
    $1.Timestamp? createdAt,
    $1.Timestamp? updatedAt,
  }) {
    final result = create();
    if (id != null) result.id = id;
    if (name != null) result.name = name;
    if (description != null) result.description = description;
    if (packagePath != null) result.packagePath = packagePath;
    if (regoContent != null) result.regoContent = regoContent;
    if (bundleId != null) result.bundleId = bundleId;
    if (enabled != null) result.enabled = enabled;
    if (version != null) result.version = version;
    if (createdAt != null) result.createdAt = createdAt;
    if (updatedAt != null) result.updatedAt = updatedAt;
    return result;
  }

  Policy._();

  factory Policy.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory Policy.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'Policy',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.policy.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'id')
    ..aOS(2, _omitFieldNames ? '' : 'name')
    ..aOS(3, _omitFieldNames ? '' : 'description')
    ..aOS(4, _omitFieldNames ? '' : 'packagePath')
    ..aOS(5, _omitFieldNames ? '' : 'regoContent')
    ..aOS(6, _omitFieldNames ? '' : 'bundleId')
    ..aOB(7, _omitFieldNames ? '' : 'enabled')
    ..aI(8, _omitFieldNames ? '' : 'version', fieldType: $pb.PbFieldType.OU3)
    ..aOM<$1.Timestamp>(9, _omitFieldNames ? '' : 'createdAt',
        subBuilder: $1.Timestamp.create)
    ..aOM<$1.Timestamp>(10, _omitFieldNames ? '' : 'updatedAt',
        subBuilder: $1.Timestamp.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  Policy clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  Policy copyWith(void Function(Policy) updates) =>
      super.copyWith((message) => updates(message as Policy)) as Policy;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static Policy create() => Policy._();
  @$core.override
  Policy createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static Policy getDefault() =>
      _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<Policy>(create);
  static Policy? _defaultInstance;

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
  $core.String get packagePath => $_getSZ(3);
  @$pb.TagNumber(4)
  set packagePath($core.String value) => $_setString(3, value);
  @$pb.TagNumber(4)
  $core.bool hasPackagePath() => $_has(3);
  @$pb.TagNumber(4)
  void clearPackagePath() => $_clearField(4);

  @$pb.TagNumber(5)
  $core.String get regoContent => $_getSZ(4);
  @$pb.TagNumber(5)
  set regoContent($core.String value) => $_setString(4, value);
  @$pb.TagNumber(5)
  $core.bool hasRegoContent() => $_has(4);
  @$pb.TagNumber(5)
  void clearRegoContent() => $_clearField(5);

  @$pb.TagNumber(6)
  $core.String get bundleId => $_getSZ(5);
  @$pb.TagNumber(6)
  set bundleId($core.String value) => $_setString(5, value);
  @$pb.TagNumber(6)
  $core.bool hasBundleId() => $_has(5);
  @$pb.TagNumber(6)
  void clearBundleId() => $_clearField(6);

  @$pb.TagNumber(7)
  $core.bool get enabled => $_getBF(6);
  @$pb.TagNumber(7)
  set enabled($core.bool value) => $_setBool(6, value);
  @$pb.TagNumber(7)
  $core.bool hasEnabled() => $_has(6);
  @$pb.TagNumber(7)
  void clearEnabled() => $_clearField(7);

  @$pb.TagNumber(8)
  $core.int get version => $_getIZ(7);
  @$pb.TagNumber(8)
  set version($core.int value) => $_setUnsignedInt32(7, value);
  @$pb.TagNumber(8)
  $core.bool hasVersion() => $_has(7);
  @$pb.TagNumber(8)
  void clearVersion() => $_clearField(8);

  @$pb.TagNumber(9)
  $1.Timestamp get createdAt => $_getN(8);
  @$pb.TagNumber(9)
  set createdAt($1.Timestamp value) => $_setField(9, value);
  @$pb.TagNumber(9)
  $core.bool hasCreatedAt() => $_has(8);
  @$pb.TagNumber(9)
  void clearCreatedAt() => $_clearField(9);
  @$pb.TagNumber(9)
  $1.Timestamp ensureCreatedAt() => $_ensure(8);

  @$pb.TagNumber(10)
  $1.Timestamp get updatedAt => $_getN(9);
  @$pb.TagNumber(10)
  set updatedAt($1.Timestamp value) => $_setField(10, value);
  @$pb.TagNumber(10)
  $core.bool hasUpdatedAt() => $_has(9);
  @$pb.TagNumber(10)
  void clearUpdatedAt() => $_clearField(10);
  @$pb.TagNumber(10)
  $1.Timestamp ensureUpdatedAt() => $_ensure(9);
}

class PolicyBundle extends $pb.GeneratedMessage {
  factory PolicyBundle({
    $core.String? id,
    $core.String? name,
    $core.Iterable<$core.String>? policyIds,
    $1.Timestamp? createdAt,
    $1.Timestamp? updatedAt,
    $core.String? description,
    $core.bool? enabled,
  }) {
    final result = create();
    if (id != null) result.id = id;
    if (name != null) result.name = name;
    if (policyIds != null) result.policyIds.addAll(policyIds);
    if (createdAt != null) result.createdAt = createdAt;
    if (updatedAt != null) result.updatedAt = updatedAt;
    if (description != null) result.description = description;
    if (enabled != null) result.enabled = enabled;
    return result;
  }

  PolicyBundle._();

  factory PolicyBundle.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory PolicyBundle.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'PolicyBundle',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.policy.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'id')
    ..aOS(2, _omitFieldNames ? '' : 'name')
    ..pPS(3, _omitFieldNames ? '' : 'policyIds')
    ..aOM<$1.Timestamp>(4, _omitFieldNames ? '' : 'createdAt',
        subBuilder: $1.Timestamp.create)
    ..aOM<$1.Timestamp>(5, _omitFieldNames ? '' : 'updatedAt',
        subBuilder: $1.Timestamp.create)
    ..aOS(6, _omitFieldNames ? '' : 'description')
    ..aOB(7, _omitFieldNames ? '' : 'enabled')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  PolicyBundle clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  PolicyBundle copyWith(void Function(PolicyBundle) updates) =>
      super.copyWith((message) => updates(message as PolicyBundle))
          as PolicyBundle;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static PolicyBundle create() => PolicyBundle._();
  @$core.override
  PolicyBundle createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static PolicyBundle getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<PolicyBundle>(create);
  static PolicyBundle? _defaultInstance;

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
  $pb.PbList<$core.String> get policyIds => $_getList(2);

  @$pb.TagNumber(4)
  $1.Timestamp get createdAt => $_getN(3);
  @$pb.TagNumber(4)
  set createdAt($1.Timestamp value) => $_setField(4, value);
  @$pb.TagNumber(4)
  $core.bool hasCreatedAt() => $_has(3);
  @$pb.TagNumber(4)
  void clearCreatedAt() => $_clearField(4);
  @$pb.TagNumber(4)
  $1.Timestamp ensureCreatedAt() => $_ensure(3);

  @$pb.TagNumber(5)
  $1.Timestamp get updatedAt => $_getN(4);
  @$pb.TagNumber(5)
  set updatedAt($1.Timestamp value) => $_setField(5, value);
  @$pb.TagNumber(5)
  $core.bool hasUpdatedAt() => $_has(4);
  @$pb.TagNumber(5)
  void clearUpdatedAt() => $_clearField(5);
  @$pb.TagNumber(5)
  $1.Timestamp ensureUpdatedAt() => $_ensure(4);

  @$pb.TagNumber(6)
  $core.String get description => $_getSZ(5);
  @$pb.TagNumber(6)
  set description($core.String value) => $_setString(5, value);
  @$pb.TagNumber(6)
  $core.bool hasDescription() => $_has(5);
  @$pb.TagNumber(6)
  void clearDescription() => $_clearField(6);

  @$pb.TagNumber(7)
  $core.bool get enabled => $_getBF(6);
  @$pb.TagNumber(7)
  set enabled($core.bool value) => $_setBool(6, value);
  @$pb.TagNumber(7)
  $core.bool hasEnabled() => $_has(6);
  @$pb.TagNumber(7)
  void clearEnabled() => $_clearField(7);
}

const $core.bool _omitFieldNames =
    $core.bool.fromEnvironment('protobuf.omit_field_names');
const $core.bool _omitMessageNames =
    $core.bool.fromEnvironment('protobuf.omit_message_names');
