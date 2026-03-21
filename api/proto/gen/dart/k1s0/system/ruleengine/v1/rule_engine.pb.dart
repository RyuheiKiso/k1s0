// This is a generated file - do not edit.
//
// Generated from k1s0/system/ruleengine/v1/rule_engine.proto.

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

class Rule extends $pb.GeneratedMessage {
  factory Rule({
    $core.String? id,
    $core.String? name,
    $core.String? description,
    $core.int? priority,
    $core.List<$core.int>? whenJson,
    $core.List<$core.int>? thenJson,
    $core.bool? enabled,
    $core.int? version,
    $1.Timestamp? createdAt,
    $1.Timestamp? updatedAt,
  }) {
    final result = create();
    if (id != null) result.id = id;
    if (name != null) result.name = name;
    if (description != null) result.description = description;
    if (priority != null) result.priority = priority;
    if (whenJson != null) result.whenJson = whenJson;
    if (thenJson != null) result.thenJson = thenJson;
    if (enabled != null) result.enabled = enabled;
    if (version != null) result.version = version;
    if (createdAt != null) result.createdAt = createdAt;
    if (updatedAt != null) result.updatedAt = updatedAt;
    return result;
  }

  Rule._();

  factory Rule.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory Rule.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'Rule',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.ruleengine.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'id')
    ..aOS(2, _omitFieldNames ? '' : 'name')
    ..aOS(3, _omitFieldNames ? '' : 'description')
    ..aI(4, _omitFieldNames ? '' : 'priority')
    ..a<$core.List<$core.int>>(
        5, _omitFieldNames ? '' : 'whenJson', $pb.PbFieldType.OY)
    ..a<$core.List<$core.int>>(
        6, _omitFieldNames ? '' : 'thenJson', $pb.PbFieldType.OY)
    ..aOB(7, _omitFieldNames ? '' : 'enabled')
    ..aI(8, _omitFieldNames ? '' : 'version', fieldType: $pb.PbFieldType.OU3)
    ..aOM<$1.Timestamp>(9, _omitFieldNames ? '' : 'createdAt',
        subBuilder: $1.Timestamp.create)
    ..aOM<$1.Timestamp>(10, _omitFieldNames ? '' : 'updatedAt',
        subBuilder: $1.Timestamp.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  Rule clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  Rule copyWith(void Function(Rule) updates) =>
      super.copyWith((message) => updates(message as Rule)) as Rule;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static Rule create() => Rule._();
  @$core.override
  Rule createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static Rule getDefault() =>
      _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<Rule>(create);
  static Rule? _defaultInstance;

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
  $core.int get priority => $_getIZ(3);
  @$pb.TagNumber(4)
  set priority($core.int value) => $_setSignedInt32(3, value);
  @$pb.TagNumber(4)
  $core.bool hasPriority() => $_has(3);
  @$pb.TagNumber(4)
  void clearPriority() => $_clearField(4);

  @$pb.TagNumber(5)
  $core.List<$core.int> get whenJson => $_getN(4);
  @$pb.TagNumber(5)
  set whenJson($core.List<$core.int> value) => $_setBytes(4, value);
  @$pb.TagNumber(5)
  $core.bool hasWhenJson() => $_has(4);
  @$pb.TagNumber(5)
  void clearWhenJson() => $_clearField(5);

  @$pb.TagNumber(6)
  $core.List<$core.int> get thenJson => $_getN(5);
  @$pb.TagNumber(6)
  set thenJson($core.List<$core.int> value) => $_setBytes(5, value);
  @$pb.TagNumber(6)
  $core.bool hasThenJson() => $_has(5);
  @$pb.TagNumber(6)
  void clearThenJson() => $_clearField(6);

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

class ListRulesRequest extends $pb.GeneratedMessage {
  factory ListRulesRequest({
    $1.Pagination? pagination,
    $core.String? ruleSetId,
    $core.String? domain,
  }) {
    final result = create();
    if (pagination != null) result.pagination = pagination;
    if (ruleSetId != null) result.ruleSetId = ruleSetId;
    if (domain != null) result.domain = domain;
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
          _omitMessageNames ? '' : 'k1s0.system.ruleengine.v1'),
      createEmptyInstance: create)
    ..aOM<$1.Pagination>(1, _omitFieldNames ? '' : 'pagination',
        subBuilder: $1.Pagination.create)
    ..aOS(2, _omitFieldNames ? '' : 'ruleSetId')
    ..aOS(3, _omitFieldNames ? '' : 'domain')
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
  $core.String get ruleSetId => $_getSZ(1);
  @$pb.TagNumber(2)
  set ruleSetId($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasRuleSetId() => $_has(1);
  @$pb.TagNumber(2)
  void clearRuleSetId() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.String get domain => $_getSZ(2);
  @$pb.TagNumber(3)
  set domain($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasDomain() => $_has(2);
  @$pb.TagNumber(3)
  void clearDomain() => $_clearField(3);
}

class ListRulesResponse extends $pb.GeneratedMessage {
  factory ListRulesResponse({
    $core.Iterable<Rule>? rules,
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
          _omitMessageNames ? '' : 'k1s0.system.ruleengine.v1'),
      createEmptyInstance: create)
    ..pPM<Rule>(1, _omitFieldNames ? '' : 'rules', subBuilder: Rule.create)
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
  $pb.PbList<Rule> get rules => $_getList(0);

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

class GetRuleRequest extends $pb.GeneratedMessage {
  factory GetRuleRequest({
    $core.String? id,
  }) {
    final result = create();
    if (id != null) result.id = id;
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
          _omitMessageNames ? '' : 'k1s0.system.ruleengine.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'id')
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
  $core.String get id => $_getSZ(0);
  @$pb.TagNumber(1)
  set id($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasId() => $_has(0);
  @$pb.TagNumber(1)
  void clearId() => $_clearField(1);
}

class GetRuleResponse extends $pb.GeneratedMessage {
  factory GetRuleResponse({
    Rule? rule,
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
          _omitMessageNames ? '' : 'k1s0.system.ruleengine.v1'),
      createEmptyInstance: create)
    ..aOM<Rule>(1, _omitFieldNames ? '' : 'rule', subBuilder: Rule.create)
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
  Rule get rule => $_getN(0);
  @$pb.TagNumber(1)
  set rule(Rule value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasRule() => $_has(0);
  @$pb.TagNumber(1)
  void clearRule() => $_clearField(1);
  @$pb.TagNumber(1)
  Rule ensureRule() => $_ensure(0);
}

class CreateRuleRequest extends $pb.GeneratedMessage {
  factory CreateRuleRequest({
    $core.String? name,
    $core.String? description,
    $core.int? priority,
    $core.List<$core.int>? whenJson,
    $core.List<$core.int>? thenJson,
  }) {
    final result = create();
    if (name != null) result.name = name;
    if (description != null) result.description = description;
    if (priority != null) result.priority = priority;
    if (whenJson != null) result.whenJson = whenJson;
    if (thenJson != null) result.thenJson = thenJson;
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
          _omitMessageNames ? '' : 'k1s0.system.ruleengine.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'name')
    ..aOS(2, _omitFieldNames ? '' : 'description')
    ..aI(3, _omitFieldNames ? '' : 'priority')
    ..a<$core.List<$core.int>>(
        4, _omitFieldNames ? '' : 'whenJson', $pb.PbFieldType.OY)
    ..a<$core.List<$core.int>>(
        5, _omitFieldNames ? '' : 'thenJson', $pb.PbFieldType.OY)
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
  $core.int get priority => $_getIZ(2);
  @$pb.TagNumber(3)
  set priority($core.int value) => $_setSignedInt32(2, value);
  @$pb.TagNumber(3)
  $core.bool hasPriority() => $_has(2);
  @$pb.TagNumber(3)
  void clearPriority() => $_clearField(3);

  @$pb.TagNumber(4)
  $core.List<$core.int> get whenJson => $_getN(3);
  @$pb.TagNumber(4)
  set whenJson($core.List<$core.int> value) => $_setBytes(3, value);
  @$pb.TagNumber(4)
  $core.bool hasWhenJson() => $_has(3);
  @$pb.TagNumber(4)
  void clearWhenJson() => $_clearField(4);

  @$pb.TagNumber(5)
  $core.List<$core.int> get thenJson => $_getN(4);
  @$pb.TagNumber(5)
  set thenJson($core.List<$core.int> value) => $_setBytes(4, value);
  @$pb.TagNumber(5)
  $core.bool hasThenJson() => $_has(4);
  @$pb.TagNumber(5)
  void clearThenJson() => $_clearField(5);
}

class CreateRuleResponse extends $pb.GeneratedMessage {
  factory CreateRuleResponse({
    Rule? rule,
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
          _omitMessageNames ? '' : 'k1s0.system.ruleengine.v1'),
      createEmptyInstance: create)
    ..aOM<Rule>(1, _omitFieldNames ? '' : 'rule', subBuilder: Rule.create)
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
  Rule get rule => $_getN(0);
  @$pb.TagNumber(1)
  set rule(Rule value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasRule() => $_has(0);
  @$pb.TagNumber(1)
  void clearRule() => $_clearField(1);
  @$pb.TagNumber(1)
  Rule ensureRule() => $_ensure(0);
}

class UpdateRuleRequest extends $pb.GeneratedMessage {
  factory UpdateRuleRequest({
    $core.String? id,
    $core.String? description,
    $core.int? priority,
    $core.List<$core.int>? whenJson,
    $core.List<$core.int>? thenJson,
    $core.bool? enabled,
  }) {
    final result = create();
    if (id != null) result.id = id;
    if (description != null) result.description = description;
    if (priority != null) result.priority = priority;
    if (whenJson != null) result.whenJson = whenJson;
    if (thenJson != null) result.thenJson = thenJson;
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
          _omitMessageNames ? '' : 'k1s0.system.ruleengine.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'id')
    ..aOS(2, _omitFieldNames ? '' : 'description')
    ..aI(3, _omitFieldNames ? '' : 'priority')
    ..a<$core.List<$core.int>>(
        4, _omitFieldNames ? '' : 'whenJson', $pb.PbFieldType.OY)
    ..a<$core.List<$core.int>>(
        5, _omitFieldNames ? '' : 'thenJson', $pb.PbFieldType.OY)
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
  $core.int get priority => $_getIZ(2);
  @$pb.TagNumber(3)
  set priority($core.int value) => $_setSignedInt32(2, value);
  @$pb.TagNumber(3)
  $core.bool hasPriority() => $_has(2);
  @$pb.TagNumber(3)
  void clearPriority() => $_clearField(3);

  @$pb.TagNumber(4)
  $core.List<$core.int> get whenJson => $_getN(3);
  @$pb.TagNumber(4)
  set whenJson($core.List<$core.int> value) => $_setBytes(3, value);
  @$pb.TagNumber(4)
  $core.bool hasWhenJson() => $_has(3);
  @$pb.TagNumber(4)
  void clearWhenJson() => $_clearField(4);

  @$pb.TagNumber(5)
  $core.List<$core.int> get thenJson => $_getN(4);
  @$pb.TagNumber(5)
  set thenJson($core.List<$core.int> value) => $_setBytes(4, value);
  @$pb.TagNumber(5)
  $core.bool hasThenJson() => $_has(4);
  @$pb.TagNumber(5)
  void clearThenJson() => $_clearField(5);

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
    Rule? rule,
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
          _omitMessageNames ? '' : 'k1s0.system.ruleengine.v1'),
      createEmptyInstance: create)
    ..aOM<Rule>(1, _omitFieldNames ? '' : 'rule', subBuilder: Rule.create)
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
  Rule get rule => $_getN(0);
  @$pb.TagNumber(1)
  set rule(Rule value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasRule() => $_has(0);
  @$pb.TagNumber(1)
  void clearRule() => $_clearField(1);
  @$pb.TagNumber(1)
  Rule ensureRule() => $_ensure(0);
}

class DeleteRuleRequest extends $pb.GeneratedMessage {
  factory DeleteRuleRequest({
    $core.String? id,
  }) {
    final result = create();
    if (id != null) result.id = id;
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
          _omitMessageNames ? '' : 'k1s0.system.ruleengine.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'id')
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
  $core.String get id => $_getSZ(0);
  @$pb.TagNumber(1)
  set id($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasId() => $_has(0);
  @$pb.TagNumber(1)
  void clearId() => $_clearField(1);
}

class DeleteRuleResponse extends $pb.GeneratedMessage {
  factory DeleteRuleResponse({
    $core.bool? success,
    $core.String? message,
  }) {
    final result = create();
    if (success != null) result.success = success;
    if (message != null) result.message = message;
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
          _omitMessageNames ? '' : 'k1s0.system.ruleengine.v1'),
      createEmptyInstance: create)
    ..aOB(1, _omitFieldNames ? '' : 'success')
    ..aOS(2, _omitFieldNames ? '' : 'message')
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

  @$pb.TagNumber(2)
  $core.String get message => $_getSZ(1);
  @$pb.TagNumber(2)
  set message($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasMessage() => $_has(1);
  @$pb.TagNumber(2)
  void clearMessage() => $_clearField(2);
}

class RuleSet extends $pb.GeneratedMessage {
  factory RuleSet({
    $core.String? id,
    $core.String? name,
    $core.String? description,
    $core.String? domain,
    $core.String? evaluationMode,
    $core.List<$core.int>? defaultResultJson,
    $core.Iterable<$core.String>? ruleIds,
    $core.int? currentVersion,
    $core.bool? enabled,
    $1.Timestamp? createdAt,
    $1.Timestamp? updatedAt,
  }) {
    final result = create();
    if (id != null) result.id = id;
    if (name != null) result.name = name;
    if (description != null) result.description = description;
    if (domain != null) result.domain = domain;
    if (evaluationMode != null) result.evaluationMode = evaluationMode;
    if (defaultResultJson != null) result.defaultResultJson = defaultResultJson;
    if (ruleIds != null) result.ruleIds.addAll(ruleIds);
    if (currentVersion != null) result.currentVersion = currentVersion;
    if (enabled != null) result.enabled = enabled;
    if (createdAt != null) result.createdAt = createdAt;
    if (updatedAt != null) result.updatedAt = updatedAt;
    return result;
  }

  RuleSet._();

  factory RuleSet.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory RuleSet.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'RuleSet',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.ruleengine.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'id')
    ..aOS(2, _omitFieldNames ? '' : 'name')
    ..aOS(3, _omitFieldNames ? '' : 'description')
    ..aOS(4, _omitFieldNames ? '' : 'domain')
    ..aOS(5, _omitFieldNames ? '' : 'evaluationMode')
    ..a<$core.List<$core.int>>(
        6, _omitFieldNames ? '' : 'defaultResultJson', $pb.PbFieldType.OY)
    ..pPS(7, _omitFieldNames ? '' : 'ruleIds')
    ..aI(8, _omitFieldNames ? '' : 'currentVersion',
        fieldType: $pb.PbFieldType.OU3)
    ..aOB(9, _omitFieldNames ? '' : 'enabled')
    ..aOM<$1.Timestamp>(10, _omitFieldNames ? '' : 'createdAt',
        subBuilder: $1.Timestamp.create)
    ..aOM<$1.Timestamp>(11, _omitFieldNames ? '' : 'updatedAt',
        subBuilder: $1.Timestamp.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  RuleSet clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  RuleSet copyWith(void Function(RuleSet) updates) =>
      super.copyWith((message) => updates(message as RuleSet)) as RuleSet;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static RuleSet create() => RuleSet._();
  @$core.override
  RuleSet createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static RuleSet getDefault() =>
      _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<RuleSet>(create);
  static RuleSet? _defaultInstance;

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
  $core.String get domain => $_getSZ(3);
  @$pb.TagNumber(4)
  set domain($core.String value) => $_setString(3, value);
  @$pb.TagNumber(4)
  $core.bool hasDomain() => $_has(3);
  @$pb.TagNumber(4)
  void clearDomain() => $_clearField(4);

  @$pb.TagNumber(5)
  $core.String get evaluationMode => $_getSZ(4);
  @$pb.TagNumber(5)
  set evaluationMode($core.String value) => $_setString(4, value);
  @$pb.TagNumber(5)
  $core.bool hasEvaluationMode() => $_has(4);
  @$pb.TagNumber(5)
  void clearEvaluationMode() => $_clearField(5);

  @$pb.TagNumber(6)
  $core.List<$core.int> get defaultResultJson => $_getN(5);
  @$pb.TagNumber(6)
  set defaultResultJson($core.List<$core.int> value) => $_setBytes(5, value);
  @$pb.TagNumber(6)
  $core.bool hasDefaultResultJson() => $_has(5);
  @$pb.TagNumber(6)
  void clearDefaultResultJson() => $_clearField(6);

  @$pb.TagNumber(7)
  $pb.PbList<$core.String> get ruleIds => $_getList(6);

  @$pb.TagNumber(8)
  $core.int get currentVersion => $_getIZ(7);
  @$pb.TagNumber(8)
  set currentVersion($core.int value) => $_setUnsignedInt32(7, value);
  @$pb.TagNumber(8)
  $core.bool hasCurrentVersion() => $_has(7);
  @$pb.TagNumber(8)
  void clearCurrentVersion() => $_clearField(8);

  @$pb.TagNumber(9)
  $core.bool get enabled => $_getBF(8);
  @$pb.TagNumber(9)
  set enabled($core.bool value) => $_setBool(8, value);
  @$pb.TagNumber(9)
  $core.bool hasEnabled() => $_has(8);
  @$pb.TagNumber(9)
  void clearEnabled() => $_clearField(9);

  @$pb.TagNumber(10)
  $1.Timestamp get createdAt => $_getN(9);
  @$pb.TagNumber(10)
  set createdAt($1.Timestamp value) => $_setField(10, value);
  @$pb.TagNumber(10)
  $core.bool hasCreatedAt() => $_has(9);
  @$pb.TagNumber(10)
  void clearCreatedAt() => $_clearField(10);
  @$pb.TagNumber(10)
  $1.Timestamp ensureCreatedAt() => $_ensure(9);

  @$pb.TagNumber(11)
  $1.Timestamp get updatedAt => $_getN(10);
  @$pb.TagNumber(11)
  set updatedAt($1.Timestamp value) => $_setField(11, value);
  @$pb.TagNumber(11)
  $core.bool hasUpdatedAt() => $_has(10);
  @$pb.TagNumber(11)
  void clearUpdatedAt() => $_clearField(11);
  @$pb.TagNumber(11)
  $1.Timestamp ensureUpdatedAt() => $_ensure(10);
}

class ListRuleSetsRequest extends $pb.GeneratedMessage {
  factory ListRuleSetsRequest({
    $1.Pagination? pagination,
    $core.String? domain,
  }) {
    final result = create();
    if (pagination != null) result.pagination = pagination;
    if (domain != null) result.domain = domain;
    return result;
  }

  ListRuleSetsRequest._();

  factory ListRuleSetsRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory ListRuleSetsRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ListRuleSetsRequest',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.ruleengine.v1'),
      createEmptyInstance: create)
    ..aOM<$1.Pagination>(1, _omitFieldNames ? '' : 'pagination',
        subBuilder: $1.Pagination.create)
    ..aOS(2, _omitFieldNames ? '' : 'domain')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListRuleSetsRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListRuleSetsRequest copyWith(void Function(ListRuleSetsRequest) updates) =>
      super.copyWith((message) => updates(message as ListRuleSetsRequest))
          as ListRuleSetsRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static ListRuleSetsRequest create() => ListRuleSetsRequest._();
  @$core.override
  ListRuleSetsRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static ListRuleSetsRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ListRuleSetsRequest>(create);
  static ListRuleSetsRequest? _defaultInstance;

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
  $core.String get domain => $_getSZ(1);
  @$pb.TagNumber(2)
  set domain($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasDomain() => $_has(1);
  @$pb.TagNumber(2)
  void clearDomain() => $_clearField(2);
}

class ListRuleSetsResponse extends $pb.GeneratedMessage {
  factory ListRuleSetsResponse({
    $core.Iterable<RuleSet>? ruleSets,
    $1.PaginationResult? pagination,
  }) {
    final result = create();
    if (ruleSets != null) result.ruleSets.addAll(ruleSets);
    if (pagination != null) result.pagination = pagination;
    return result;
  }

  ListRuleSetsResponse._();

  factory ListRuleSetsResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory ListRuleSetsResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ListRuleSetsResponse',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.ruleengine.v1'),
      createEmptyInstance: create)
    ..pPM<RuleSet>(1, _omitFieldNames ? '' : 'ruleSets',
        subBuilder: RuleSet.create)
    ..aOM<$1.PaginationResult>(2, _omitFieldNames ? '' : 'pagination',
        subBuilder: $1.PaginationResult.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListRuleSetsResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListRuleSetsResponse copyWith(void Function(ListRuleSetsResponse) updates) =>
      super.copyWith((message) => updates(message as ListRuleSetsResponse))
          as ListRuleSetsResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static ListRuleSetsResponse create() => ListRuleSetsResponse._();
  @$core.override
  ListRuleSetsResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static ListRuleSetsResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ListRuleSetsResponse>(create);
  static ListRuleSetsResponse? _defaultInstance;

  @$pb.TagNumber(1)
  $pb.PbList<RuleSet> get ruleSets => $_getList(0);

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

class GetRuleSetRequest extends $pb.GeneratedMessage {
  factory GetRuleSetRequest({
    $core.String? id,
  }) {
    final result = create();
    if (id != null) result.id = id;
    return result;
  }

  GetRuleSetRequest._();

  factory GetRuleSetRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory GetRuleSetRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'GetRuleSetRequest',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.ruleengine.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'id')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetRuleSetRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetRuleSetRequest copyWith(void Function(GetRuleSetRequest) updates) =>
      super.copyWith((message) => updates(message as GetRuleSetRequest))
          as GetRuleSetRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static GetRuleSetRequest create() => GetRuleSetRequest._();
  @$core.override
  GetRuleSetRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static GetRuleSetRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<GetRuleSetRequest>(create);
  static GetRuleSetRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get id => $_getSZ(0);
  @$pb.TagNumber(1)
  set id($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasId() => $_has(0);
  @$pb.TagNumber(1)
  void clearId() => $_clearField(1);
}

class GetRuleSetResponse extends $pb.GeneratedMessage {
  factory GetRuleSetResponse({
    RuleSet? ruleSet,
  }) {
    final result = create();
    if (ruleSet != null) result.ruleSet = ruleSet;
    return result;
  }

  GetRuleSetResponse._();

  factory GetRuleSetResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory GetRuleSetResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'GetRuleSetResponse',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.ruleengine.v1'),
      createEmptyInstance: create)
    ..aOM<RuleSet>(1, _omitFieldNames ? '' : 'ruleSet',
        subBuilder: RuleSet.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetRuleSetResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetRuleSetResponse copyWith(void Function(GetRuleSetResponse) updates) =>
      super.copyWith((message) => updates(message as GetRuleSetResponse))
          as GetRuleSetResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static GetRuleSetResponse create() => GetRuleSetResponse._();
  @$core.override
  GetRuleSetResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static GetRuleSetResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<GetRuleSetResponse>(create);
  static GetRuleSetResponse? _defaultInstance;

  @$pb.TagNumber(1)
  RuleSet get ruleSet => $_getN(0);
  @$pb.TagNumber(1)
  set ruleSet(RuleSet value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasRuleSet() => $_has(0);
  @$pb.TagNumber(1)
  void clearRuleSet() => $_clearField(1);
  @$pb.TagNumber(1)
  RuleSet ensureRuleSet() => $_ensure(0);
}

class CreateRuleSetRequest extends $pb.GeneratedMessage {
  factory CreateRuleSetRequest({
    $core.String? name,
    $core.String? description,
    $core.String? domain,
    $core.String? evaluationMode,
    $core.List<$core.int>? defaultResultJson,
    $core.Iterable<$core.String>? ruleIds,
  }) {
    final result = create();
    if (name != null) result.name = name;
    if (description != null) result.description = description;
    if (domain != null) result.domain = domain;
    if (evaluationMode != null) result.evaluationMode = evaluationMode;
    if (defaultResultJson != null) result.defaultResultJson = defaultResultJson;
    if (ruleIds != null) result.ruleIds.addAll(ruleIds);
    return result;
  }

  CreateRuleSetRequest._();

  factory CreateRuleSetRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory CreateRuleSetRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'CreateRuleSetRequest',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.ruleengine.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'name')
    ..aOS(2, _omitFieldNames ? '' : 'description')
    ..aOS(3, _omitFieldNames ? '' : 'domain')
    ..aOS(4, _omitFieldNames ? '' : 'evaluationMode')
    ..a<$core.List<$core.int>>(
        5, _omitFieldNames ? '' : 'defaultResultJson', $pb.PbFieldType.OY)
    ..pPS(6, _omitFieldNames ? '' : 'ruleIds')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  CreateRuleSetRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  CreateRuleSetRequest copyWith(void Function(CreateRuleSetRequest) updates) =>
      super.copyWith((message) => updates(message as CreateRuleSetRequest))
          as CreateRuleSetRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static CreateRuleSetRequest create() => CreateRuleSetRequest._();
  @$core.override
  CreateRuleSetRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static CreateRuleSetRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<CreateRuleSetRequest>(create);
  static CreateRuleSetRequest? _defaultInstance;

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
  $core.String get domain => $_getSZ(2);
  @$pb.TagNumber(3)
  set domain($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasDomain() => $_has(2);
  @$pb.TagNumber(3)
  void clearDomain() => $_clearField(3);

  @$pb.TagNumber(4)
  $core.String get evaluationMode => $_getSZ(3);
  @$pb.TagNumber(4)
  set evaluationMode($core.String value) => $_setString(3, value);
  @$pb.TagNumber(4)
  $core.bool hasEvaluationMode() => $_has(3);
  @$pb.TagNumber(4)
  void clearEvaluationMode() => $_clearField(4);

  @$pb.TagNumber(5)
  $core.List<$core.int> get defaultResultJson => $_getN(4);
  @$pb.TagNumber(5)
  set defaultResultJson($core.List<$core.int> value) => $_setBytes(4, value);
  @$pb.TagNumber(5)
  $core.bool hasDefaultResultJson() => $_has(4);
  @$pb.TagNumber(5)
  void clearDefaultResultJson() => $_clearField(5);

  @$pb.TagNumber(6)
  $pb.PbList<$core.String> get ruleIds => $_getList(5);
}

class CreateRuleSetResponse extends $pb.GeneratedMessage {
  factory CreateRuleSetResponse({
    RuleSet? ruleSet,
  }) {
    final result = create();
    if (ruleSet != null) result.ruleSet = ruleSet;
    return result;
  }

  CreateRuleSetResponse._();

  factory CreateRuleSetResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory CreateRuleSetResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'CreateRuleSetResponse',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.ruleengine.v1'),
      createEmptyInstance: create)
    ..aOM<RuleSet>(1, _omitFieldNames ? '' : 'ruleSet',
        subBuilder: RuleSet.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  CreateRuleSetResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  CreateRuleSetResponse copyWith(
          void Function(CreateRuleSetResponse) updates) =>
      super.copyWith((message) => updates(message as CreateRuleSetResponse))
          as CreateRuleSetResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static CreateRuleSetResponse create() => CreateRuleSetResponse._();
  @$core.override
  CreateRuleSetResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static CreateRuleSetResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<CreateRuleSetResponse>(create);
  static CreateRuleSetResponse? _defaultInstance;

  @$pb.TagNumber(1)
  RuleSet get ruleSet => $_getN(0);
  @$pb.TagNumber(1)
  set ruleSet(RuleSet value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasRuleSet() => $_has(0);
  @$pb.TagNumber(1)
  void clearRuleSet() => $_clearField(1);
  @$pb.TagNumber(1)
  RuleSet ensureRuleSet() => $_ensure(0);
}

class UpdateRuleSetRequest extends $pb.GeneratedMessage {
  factory UpdateRuleSetRequest({
    $core.String? id,
    $core.String? description,
    $core.String? evaluationMode,
    $core.List<$core.int>? defaultResultJson,
    $core.Iterable<$core.String>? ruleIds,
    $core.bool? enabled,
  }) {
    final result = create();
    if (id != null) result.id = id;
    if (description != null) result.description = description;
    if (evaluationMode != null) result.evaluationMode = evaluationMode;
    if (defaultResultJson != null) result.defaultResultJson = defaultResultJson;
    if (ruleIds != null) result.ruleIds.addAll(ruleIds);
    if (enabled != null) result.enabled = enabled;
    return result;
  }

  UpdateRuleSetRequest._();

  factory UpdateRuleSetRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory UpdateRuleSetRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'UpdateRuleSetRequest',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.ruleengine.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'id')
    ..aOS(2, _omitFieldNames ? '' : 'description')
    ..aOS(3, _omitFieldNames ? '' : 'evaluationMode')
    ..a<$core.List<$core.int>>(
        4, _omitFieldNames ? '' : 'defaultResultJson', $pb.PbFieldType.OY)
    ..pPS(5, _omitFieldNames ? '' : 'ruleIds')
    ..aOB(6, _omitFieldNames ? '' : 'enabled')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  UpdateRuleSetRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  UpdateRuleSetRequest copyWith(void Function(UpdateRuleSetRequest) updates) =>
      super.copyWith((message) => updates(message as UpdateRuleSetRequest))
          as UpdateRuleSetRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static UpdateRuleSetRequest create() => UpdateRuleSetRequest._();
  @$core.override
  UpdateRuleSetRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static UpdateRuleSetRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<UpdateRuleSetRequest>(create);
  static UpdateRuleSetRequest? _defaultInstance;

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
  $core.String get evaluationMode => $_getSZ(2);
  @$pb.TagNumber(3)
  set evaluationMode($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasEvaluationMode() => $_has(2);
  @$pb.TagNumber(3)
  void clearEvaluationMode() => $_clearField(3);

  @$pb.TagNumber(4)
  $core.List<$core.int> get defaultResultJson => $_getN(3);
  @$pb.TagNumber(4)
  set defaultResultJson($core.List<$core.int> value) => $_setBytes(3, value);
  @$pb.TagNumber(4)
  $core.bool hasDefaultResultJson() => $_has(3);
  @$pb.TagNumber(4)
  void clearDefaultResultJson() => $_clearField(4);

  @$pb.TagNumber(5)
  $pb.PbList<$core.String> get ruleIds => $_getList(4);

  @$pb.TagNumber(6)
  $core.bool get enabled => $_getBF(5);
  @$pb.TagNumber(6)
  set enabled($core.bool value) => $_setBool(5, value);
  @$pb.TagNumber(6)
  $core.bool hasEnabled() => $_has(5);
  @$pb.TagNumber(6)
  void clearEnabled() => $_clearField(6);
}

class UpdateRuleSetResponse extends $pb.GeneratedMessage {
  factory UpdateRuleSetResponse({
    RuleSet? ruleSet,
  }) {
    final result = create();
    if (ruleSet != null) result.ruleSet = ruleSet;
    return result;
  }

  UpdateRuleSetResponse._();

  factory UpdateRuleSetResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory UpdateRuleSetResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'UpdateRuleSetResponse',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.ruleengine.v1'),
      createEmptyInstance: create)
    ..aOM<RuleSet>(1, _omitFieldNames ? '' : 'ruleSet',
        subBuilder: RuleSet.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  UpdateRuleSetResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  UpdateRuleSetResponse copyWith(
          void Function(UpdateRuleSetResponse) updates) =>
      super.copyWith((message) => updates(message as UpdateRuleSetResponse))
          as UpdateRuleSetResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static UpdateRuleSetResponse create() => UpdateRuleSetResponse._();
  @$core.override
  UpdateRuleSetResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static UpdateRuleSetResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<UpdateRuleSetResponse>(create);
  static UpdateRuleSetResponse? _defaultInstance;

  @$pb.TagNumber(1)
  RuleSet get ruleSet => $_getN(0);
  @$pb.TagNumber(1)
  set ruleSet(RuleSet value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasRuleSet() => $_has(0);
  @$pb.TagNumber(1)
  void clearRuleSet() => $_clearField(1);
  @$pb.TagNumber(1)
  RuleSet ensureRuleSet() => $_ensure(0);
}

class DeleteRuleSetRequest extends $pb.GeneratedMessage {
  factory DeleteRuleSetRequest({
    $core.String? id,
  }) {
    final result = create();
    if (id != null) result.id = id;
    return result;
  }

  DeleteRuleSetRequest._();

  factory DeleteRuleSetRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory DeleteRuleSetRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'DeleteRuleSetRequest',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.ruleengine.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'id')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  DeleteRuleSetRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  DeleteRuleSetRequest copyWith(void Function(DeleteRuleSetRequest) updates) =>
      super.copyWith((message) => updates(message as DeleteRuleSetRequest))
          as DeleteRuleSetRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static DeleteRuleSetRequest create() => DeleteRuleSetRequest._();
  @$core.override
  DeleteRuleSetRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static DeleteRuleSetRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<DeleteRuleSetRequest>(create);
  static DeleteRuleSetRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get id => $_getSZ(0);
  @$pb.TagNumber(1)
  set id($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasId() => $_has(0);
  @$pb.TagNumber(1)
  void clearId() => $_clearField(1);
}

class DeleteRuleSetResponse extends $pb.GeneratedMessage {
  factory DeleteRuleSetResponse({
    $core.bool? success,
    $core.String? message,
  }) {
    final result = create();
    if (success != null) result.success = success;
    if (message != null) result.message = message;
    return result;
  }

  DeleteRuleSetResponse._();

  factory DeleteRuleSetResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory DeleteRuleSetResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'DeleteRuleSetResponse',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.ruleengine.v1'),
      createEmptyInstance: create)
    ..aOB(1, _omitFieldNames ? '' : 'success')
    ..aOS(2, _omitFieldNames ? '' : 'message')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  DeleteRuleSetResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  DeleteRuleSetResponse copyWith(
          void Function(DeleteRuleSetResponse) updates) =>
      super.copyWith((message) => updates(message as DeleteRuleSetResponse))
          as DeleteRuleSetResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static DeleteRuleSetResponse create() => DeleteRuleSetResponse._();
  @$core.override
  DeleteRuleSetResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static DeleteRuleSetResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<DeleteRuleSetResponse>(create);
  static DeleteRuleSetResponse? _defaultInstance;

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

class PublishRuleSetRequest extends $pb.GeneratedMessage {
  factory PublishRuleSetRequest({
    $core.String? id,
  }) {
    final result = create();
    if (id != null) result.id = id;
    return result;
  }

  PublishRuleSetRequest._();

  factory PublishRuleSetRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory PublishRuleSetRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'PublishRuleSetRequest',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.ruleengine.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'id')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  PublishRuleSetRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  PublishRuleSetRequest copyWith(
          void Function(PublishRuleSetRequest) updates) =>
      super.copyWith((message) => updates(message as PublishRuleSetRequest))
          as PublishRuleSetRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static PublishRuleSetRequest create() => PublishRuleSetRequest._();
  @$core.override
  PublishRuleSetRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static PublishRuleSetRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<PublishRuleSetRequest>(create);
  static PublishRuleSetRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get id => $_getSZ(0);
  @$pb.TagNumber(1)
  set id($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasId() => $_has(0);
  @$pb.TagNumber(1)
  void clearId() => $_clearField(1);
}

class PublishRuleSetResponse extends $pb.GeneratedMessage {
  factory PublishRuleSetResponse({
    $core.String? id,
    $core.int? publishedVersion,
    $core.int? previousVersion,
    $1.Timestamp? publishedAt,
  }) {
    final result = create();
    if (id != null) result.id = id;
    if (publishedVersion != null) result.publishedVersion = publishedVersion;
    if (previousVersion != null) result.previousVersion = previousVersion;
    if (publishedAt != null) result.publishedAt = publishedAt;
    return result;
  }

  PublishRuleSetResponse._();

  factory PublishRuleSetResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory PublishRuleSetResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'PublishRuleSetResponse',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.ruleengine.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'id')
    ..aI(2, _omitFieldNames ? '' : 'publishedVersion',
        fieldType: $pb.PbFieldType.OU3)
    ..aI(3, _omitFieldNames ? '' : 'previousVersion',
        fieldType: $pb.PbFieldType.OU3)
    ..aOM<$1.Timestamp>(4, _omitFieldNames ? '' : 'publishedAt',
        subBuilder: $1.Timestamp.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  PublishRuleSetResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  PublishRuleSetResponse copyWith(
          void Function(PublishRuleSetResponse) updates) =>
      super.copyWith((message) => updates(message as PublishRuleSetResponse))
          as PublishRuleSetResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static PublishRuleSetResponse create() => PublishRuleSetResponse._();
  @$core.override
  PublishRuleSetResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static PublishRuleSetResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<PublishRuleSetResponse>(create);
  static PublishRuleSetResponse? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get id => $_getSZ(0);
  @$pb.TagNumber(1)
  set id($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasId() => $_has(0);
  @$pb.TagNumber(1)
  void clearId() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.int get publishedVersion => $_getIZ(1);
  @$pb.TagNumber(2)
  set publishedVersion($core.int value) => $_setUnsignedInt32(1, value);
  @$pb.TagNumber(2)
  $core.bool hasPublishedVersion() => $_has(1);
  @$pb.TagNumber(2)
  void clearPublishedVersion() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.int get previousVersion => $_getIZ(2);
  @$pb.TagNumber(3)
  set previousVersion($core.int value) => $_setUnsignedInt32(2, value);
  @$pb.TagNumber(3)
  $core.bool hasPreviousVersion() => $_has(2);
  @$pb.TagNumber(3)
  void clearPreviousVersion() => $_clearField(3);

  @$pb.TagNumber(4)
  $1.Timestamp get publishedAt => $_getN(3);
  @$pb.TagNumber(4)
  set publishedAt($1.Timestamp value) => $_setField(4, value);
  @$pb.TagNumber(4)
  $core.bool hasPublishedAt() => $_has(3);
  @$pb.TagNumber(4)
  void clearPublishedAt() => $_clearField(4);
  @$pb.TagNumber(4)
  $1.Timestamp ensurePublishedAt() => $_ensure(3);
}

class RollbackRuleSetRequest extends $pb.GeneratedMessage {
  factory RollbackRuleSetRequest({
    $core.String? id,
  }) {
    final result = create();
    if (id != null) result.id = id;
    return result;
  }

  RollbackRuleSetRequest._();

  factory RollbackRuleSetRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory RollbackRuleSetRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'RollbackRuleSetRequest',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.ruleengine.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'id')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  RollbackRuleSetRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  RollbackRuleSetRequest copyWith(
          void Function(RollbackRuleSetRequest) updates) =>
      super.copyWith((message) => updates(message as RollbackRuleSetRequest))
          as RollbackRuleSetRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static RollbackRuleSetRequest create() => RollbackRuleSetRequest._();
  @$core.override
  RollbackRuleSetRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static RollbackRuleSetRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<RollbackRuleSetRequest>(create);
  static RollbackRuleSetRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get id => $_getSZ(0);
  @$pb.TagNumber(1)
  set id($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasId() => $_has(0);
  @$pb.TagNumber(1)
  void clearId() => $_clearField(1);
}

class RollbackRuleSetResponse extends $pb.GeneratedMessage {
  factory RollbackRuleSetResponse({
    $core.String? id,
    $core.int? rolledBackToVersion,
    $core.int? previousVersion,
    $1.Timestamp? rolledBackAt,
  }) {
    final result = create();
    if (id != null) result.id = id;
    if (rolledBackToVersion != null)
      result.rolledBackToVersion = rolledBackToVersion;
    if (previousVersion != null) result.previousVersion = previousVersion;
    if (rolledBackAt != null) result.rolledBackAt = rolledBackAt;
    return result;
  }

  RollbackRuleSetResponse._();

  factory RollbackRuleSetResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory RollbackRuleSetResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'RollbackRuleSetResponse',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.ruleengine.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'id')
    ..aI(2, _omitFieldNames ? '' : 'rolledBackToVersion',
        fieldType: $pb.PbFieldType.OU3)
    ..aI(3, _omitFieldNames ? '' : 'previousVersion',
        fieldType: $pb.PbFieldType.OU3)
    ..aOM<$1.Timestamp>(4, _omitFieldNames ? '' : 'rolledBackAt',
        subBuilder: $1.Timestamp.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  RollbackRuleSetResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  RollbackRuleSetResponse copyWith(
          void Function(RollbackRuleSetResponse) updates) =>
      super.copyWith((message) => updates(message as RollbackRuleSetResponse))
          as RollbackRuleSetResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static RollbackRuleSetResponse create() => RollbackRuleSetResponse._();
  @$core.override
  RollbackRuleSetResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static RollbackRuleSetResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<RollbackRuleSetResponse>(create);
  static RollbackRuleSetResponse? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get id => $_getSZ(0);
  @$pb.TagNumber(1)
  set id($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasId() => $_has(0);
  @$pb.TagNumber(1)
  void clearId() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.int get rolledBackToVersion => $_getIZ(1);
  @$pb.TagNumber(2)
  set rolledBackToVersion($core.int value) => $_setUnsignedInt32(1, value);
  @$pb.TagNumber(2)
  $core.bool hasRolledBackToVersion() => $_has(1);
  @$pb.TagNumber(2)
  void clearRolledBackToVersion() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.int get previousVersion => $_getIZ(2);
  @$pb.TagNumber(3)
  set previousVersion($core.int value) => $_setUnsignedInt32(2, value);
  @$pb.TagNumber(3)
  $core.bool hasPreviousVersion() => $_has(2);
  @$pb.TagNumber(3)
  void clearPreviousVersion() => $_clearField(3);

  @$pb.TagNumber(4)
  $1.Timestamp get rolledBackAt => $_getN(3);
  @$pb.TagNumber(4)
  set rolledBackAt($1.Timestamp value) => $_setField(4, value);
  @$pb.TagNumber(4)
  $core.bool hasRolledBackAt() => $_has(3);
  @$pb.TagNumber(4)
  void clearRolledBackAt() => $_clearField(4);
  @$pb.TagNumber(4)
  $1.Timestamp ensureRolledBackAt() => $_ensure(3);
}

class EvaluateRequest extends $pb.GeneratedMessage {
  factory EvaluateRequest({
    $core.String? ruleSet,
    $core.List<$core.int>? inputJson,
    $core.List<$core.int>? contextJson,
  }) {
    final result = create();
    if (ruleSet != null) result.ruleSet = ruleSet;
    if (inputJson != null) result.inputJson = inputJson;
    if (contextJson != null) result.contextJson = contextJson;
    return result;
  }

  EvaluateRequest._();

  factory EvaluateRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory EvaluateRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'EvaluateRequest',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.ruleengine.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'ruleSet')
    ..a<$core.List<$core.int>>(
        2, _omitFieldNames ? '' : 'inputJson', $pb.PbFieldType.OY)
    ..a<$core.List<$core.int>>(
        3, _omitFieldNames ? '' : 'contextJson', $pb.PbFieldType.OY)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  EvaluateRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  EvaluateRequest copyWith(void Function(EvaluateRequest) updates) =>
      super.copyWith((message) => updates(message as EvaluateRequest))
          as EvaluateRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static EvaluateRequest create() => EvaluateRequest._();
  @$core.override
  EvaluateRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static EvaluateRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<EvaluateRequest>(create);
  static EvaluateRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get ruleSet => $_getSZ(0);
  @$pb.TagNumber(1)
  set ruleSet($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasRuleSet() => $_has(0);
  @$pb.TagNumber(1)
  void clearRuleSet() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.List<$core.int> get inputJson => $_getN(1);
  @$pb.TagNumber(2)
  set inputJson($core.List<$core.int> value) => $_setBytes(1, value);
  @$pb.TagNumber(2)
  $core.bool hasInputJson() => $_has(1);
  @$pb.TagNumber(2)
  void clearInputJson() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.List<$core.int> get contextJson => $_getN(2);
  @$pb.TagNumber(3)
  set contextJson($core.List<$core.int> value) => $_setBytes(2, value);
  @$pb.TagNumber(3)
  $core.bool hasContextJson() => $_has(2);
  @$pb.TagNumber(3)
  void clearContextJson() => $_clearField(3);
}

class MatchedRule extends $pb.GeneratedMessage {
  factory MatchedRule({
    $core.String? id,
    $core.String? name,
    $core.int? priority,
    $core.List<$core.int>? resultJson,
  }) {
    final result = create();
    if (id != null) result.id = id;
    if (name != null) result.name = name;
    if (priority != null) result.priority = priority;
    if (resultJson != null) result.resultJson = resultJson;
    return result;
  }

  MatchedRule._();

  factory MatchedRule.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory MatchedRule.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'MatchedRule',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.ruleengine.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'id')
    ..aOS(2, _omitFieldNames ? '' : 'name')
    ..aI(3, _omitFieldNames ? '' : 'priority')
    ..a<$core.List<$core.int>>(
        4, _omitFieldNames ? '' : 'resultJson', $pb.PbFieldType.OY)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  MatchedRule clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  MatchedRule copyWith(void Function(MatchedRule) updates) =>
      super.copyWith((message) => updates(message as MatchedRule))
          as MatchedRule;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static MatchedRule create() => MatchedRule._();
  @$core.override
  MatchedRule createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static MatchedRule getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<MatchedRule>(create);
  static MatchedRule? _defaultInstance;

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
  $core.int get priority => $_getIZ(2);
  @$pb.TagNumber(3)
  set priority($core.int value) => $_setSignedInt32(2, value);
  @$pb.TagNumber(3)
  $core.bool hasPriority() => $_has(2);
  @$pb.TagNumber(3)
  void clearPriority() => $_clearField(3);

  @$pb.TagNumber(4)
  $core.List<$core.int> get resultJson => $_getN(3);
  @$pb.TagNumber(4)
  set resultJson($core.List<$core.int> value) => $_setBytes(3, value);
  @$pb.TagNumber(4)
  $core.bool hasResultJson() => $_has(3);
  @$pb.TagNumber(4)
  void clearResultJson() => $_clearField(4);
}

class EvaluateResponse extends $pb.GeneratedMessage {
  factory EvaluateResponse({
    $core.String? evaluationId,
    $core.String? ruleSet,
    $core.int? ruleSetVersion,
    $core.Iterable<MatchedRule>? matchedRules,
    $core.List<$core.int>? resultJson,
    $core.bool? defaultApplied,
    $core.bool? cached,
    $1.Timestamp? evaluatedAt,
  }) {
    final result = create();
    if (evaluationId != null) result.evaluationId = evaluationId;
    if (ruleSet != null) result.ruleSet = ruleSet;
    if (ruleSetVersion != null) result.ruleSetVersion = ruleSetVersion;
    if (matchedRules != null) result.matchedRules.addAll(matchedRules);
    if (resultJson != null) result.resultJson = resultJson;
    if (defaultApplied != null) result.defaultApplied = defaultApplied;
    if (cached != null) result.cached = cached;
    if (evaluatedAt != null) result.evaluatedAt = evaluatedAt;
    return result;
  }

  EvaluateResponse._();

  factory EvaluateResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory EvaluateResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'EvaluateResponse',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.ruleengine.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'evaluationId')
    ..aOS(2, _omitFieldNames ? '' : 'ruleSet')
    ..aI(3, _omitFieldNames ? '' : 'ruleSetVersion',
        fieldType: $pb.PbFieldType.OU3)
    ..pPM<MatchedRule>(4, _omitFieldNames ? '' : 'matchedRules',
        subBuilder: MatchedRule.create)
    ..a<$core.List<$core.int>>(
        5, _omitFieldNames ? '' : 'resultJson', $pb.PbFieldType.OY)
    ..aOB(6, _omitFieldNames ? '' : 'defaultApplied')
    ..aOB(7, _omitFieldNames ? '' : 'cached')
    ..aOM<$1.Timestamp>(8, _omitFieldNames ? '' : 'evaluatedAt',
        subBuilder: $1.Timestamp.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  EvaluateResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  EvaluateResponse copyWith(void Function(EvaluateResponse) updates) =>
      super.copyWith((message) => updates(message as EvaluateResponse))
          as EvaluateResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static EvaluateResponse create() => EvaluateResponse._();
  @$core.override
  EvaluateResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static EvaluateResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<EvaluateResponse>(create);
  static EvaluateResponse? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get evaluationId => $_getSZ(0);
  @$pb.TagNumber(1)
  set evaluationId($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasEvaluationId() => $_has(0);
  @$pb.TagNumber(1)
  void clearEvaluationId() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get ruleSet => $_getSZ(1);
  @$pb.TagNumber(2)
  set ruleSet($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasRuleSet() => $_has(1);
  @$pb.TagNumber(2)
  void clearRuleSet() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.int get ruleSetVersion => $_getIZ(2);
  @$pb.TagNumber(3)
  set ruleSetVersion($core.int value) => $_setUnsignedInt32(2, value);
  @$pb.TagNumber(3)
  $core.bool hasRuleSetVersion() => $_has(2);
  @$pb.TagNumber(3)
  void clearRuleSetVersion() => $_clearField(3);

  @$pb.TagNumber(4)
  $pb.PbList<MatchedRule> get matchedRules => $_getList(3);

  @$pb.TagNumber(5)
  $core.List<$core.int> get resultJson => $_getN(4);
  @$pb.TagNumber(5)
  set resultJson($core.List<$core.int> value) => $_setBytes(4, value);
  @$pb.TagNumber(5)
  $core.bool hasResultJson() => $_has(4);
  @$pb.TagNumber(5)
  void clearResultJson() => $_clearField(5);

  @$pb.TagNumber(6)
  $core.bool get defaultApplied => $_getBF(5);
  @$pb.TagNumber(6)
  set defaultApplied($core.bool value) => $_setBool(5, value);
  @$pb.TagNumber(6)
  $core.bool hasDefaultApplied() => $_has(5);
  @$pb.TagNumber(6)
  void clearDefaultApplied() => $_clearField(6);

  @$pb.TagNumber(7)
  $core.bool get cached => $_getBF(6);
  @$pb.TagNumber(7)
  set cached($core.bool value) => $_setBool(6, value);
  @$pb.TagNumber(7)
  $core.bool hasCached() => $_has(6);
  @$pb.TagNumber(7)
  void clearCached() => $_clearField(7);

  @$pb.TagNumber(8)
  $1.Timestamp get evaluatedAt => $_getN(7);
  @$pb.TagNumber(8)
  set evaluatedAt($1.Timestamp value) => $_setField(8, value);
  @$pb.TagNumber(8)
  $core.bool hasEvaluatedAt() => $_has(7);
  @$pb.TagNumber(8)
  void clearEvaluatedAt() => $_clearField(8);
  @$pb.TagNumber(8)
  $1.Timestamp ensureEvaluatedAt() => $_ensure(7);
}

/// ドライラン評価リクエスト: EvaluateDryRun RPC 専用のリクエストメッセージ
class EvaluateDryRunRequest extends $pb.GeneratedMessage {
  factory EvaluateDryRunRequest({
    $core.String? ruleSet,
    $core.List<$core.int>? inputJson,
    $core.List<$core.int>? contextJson,
  }) {
    final result = create();
    if (ruleSet != null) result.ruleSet = ruleSet;
    if (inputJson != null) result.inputJson = inputJson;
    if (contextJson != null) result.contextJson = contextJson;
    return result;
  }

  EvaluateDryRunRequest._();

  factory EvaluateDryRunRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory EvaluateDryRunRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'EvaluateDryRunRequest',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.ruleengine.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'ruleSet')
    ..a<$core.List<$core.int>>(
        2, _omitFieldNames ? '' : 'inputJson', $pb.PbFieldType.OY)
    ..a<$core.List<$core.int>>(
        3, _omitFieldNames ? '' : 'contextJson', $pb.PbFieldType.OY)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  EvaluateDryRunRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  EvaluateDryRunRequest copyWith(
          void Function(EvaluateDryRunRequest) updates) =>
      super.copyWith((message) => updates(message as EvaluateDryRunRequest))
          as EvaluateDryRunRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static EvaluateDryRunRequest create() => EvaluateDryRunRequest._();
  @$core.override
  EvaluateDryRunRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static EvaluateDryRunRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<EvaluateDryRunRequest>(create);
  static EvaluateDryRunRequest? _defaultInstance;

  /// 評価対象のルールセット識別子
  @$pb.TagNumber(1)
  $core.String get ruleSet => $_getSZ(0);
  @$pb.TagNumber(1)
  set ruleSet($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasRuleSet() => $_has(0);
  @$pb.TagNumber(1)
  void clearRuleSet() => $_clearField(1);

  /// 評価入力データ（JSON 形式）
  @$pb.TagNumber(2)
  $core.List<$core.int> get inputJson => $_getN(1);
  @$pb.TagNumber(2)
  set inputJson($core.List<$core.int> value) => $_setBytes(1, value);
  @$pb.TagNumber(2)
  $core.bool hasInputJson() => $_has(1);
  @$pb.TagNumber(2)
  void clearInputJson() => $_clearField(2);

  /// 評価コンテキスト（JSON 形式）
  @$pb.TagNumber(3)
  $core.List<$core.int> get contextJson => $_getN(2);
  @$pb.TagNumber(3)
  set contextJson($core.List<$core.int> value) => $_setBytes(2, value);
  @$pb.TagNumber(3)
  $core.bool hasContextJson() => $_has(2);
  @$pb.TagNumber(3)
  void clearContextJson() => $_clearField(3);
}

/// ドライラン評価レスポンス: EvaluateDryRun RPC 専用のレスポンスメッセージ
class EvaluateDryRunResponse extends $pb.GeneratedMessage {
  factory EvaluateDryRunResponse({
    $core.String? evaluationId,
    $core.String? ruleSet,
    $core.int? ruleSetVersion,
    $core.Iterable<MatchedRule>? matchedRules,
    $core.List<$core.int>? resultJson,
    $core.bool? defaultApplied,
    $core.bool? cached,
    $1.Timestamp? evaluatedAt,
  }) {
    final result = create();
    if (evaluationId != null) result.evaluationId = evaluationId;
    if (ruleSet != null) result.ruleSet = ruleSet;
    if (ruleSetVersion != null) result.ruleSetVersion = ruleSetVersion;
    if (matchedRules != null) result.matchedRules.addAll(matchedRules);
    if (resultJson != null) result.resultJson = resultJson;
    if (defaultApplied != null) result.defaultApplied = defaultApplied;
    if (cached != null) result.cached = cached;
    if (evaluatedAt != null) result.evaluatedAt = evaluatedAt;
    return result;
  }

  EvaluateDryRunResponse._();

  factory EvaluateDryRunResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory EvaluateDryRunResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'EvaluateDryRunResponse',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.ruleengine.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'evaluationId')
    ..aOS(2, _omitFieldNames ? '' : 'ruleSet')
    ..aI(3, _omitFieldNames ? '' : 'ruleSetVersion',
        fieldType: $pb.PbFieldType.OU3)
    ..pPM<MatchedRule>(4, _omitFieldNames ? '' : 'matchedRules',
        subBuilder: MatchedRule.create)
    ..a<$core.List<$core.int>>(
        5, _omitFieldNames ? '' : 'resultJson', $pb.PbFieldType.OY)
    ..aOB(6, _omitFieldNames ? '' : 'defaultApplied')
    ..aOB(7, _omitFieldNames ? '' : 'cached')
    ..aOM<$1.Timestamp>(8, _omitFieldNames ? '' : 'evaluatedAt',
        subBuilder: $1.Timestamp.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  EvaluateDryRunResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  EvaluateDryRunResponse copyWith(
          void Function(EvaluateDryRunResponse) updates) =>
      super.copyWith((message) => updates(message as EvaluateDryRunResponse))
          as EvaluateDryRunResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static EvaluateDryRunResponse create() => EvaluateDryRunResponse._();
  @$core.override
  EvaluateDryRunResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static EvaluateDryRunResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<EvaluateDryRunResponse>(create);
  static EvaluateDryRunResponse? _defaultInstance;

  /// 評価の一意識別子
  @$pb.TagNumber(1)
  $core.String get evaluationId => $_getSZ(0);
  @$pb.TagNumber(1)
  set evaluationId($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasEvaluationId() => $_has(0);
  @$pb.TagNumber(1)
  void clearEvaluationId() => $_clearField(1);

  /// 評価対象のルールセット識別子
  @$pb.TagNumber(2)
  $core.String get ruleSet => $_getSZ(1);
  @$pb.TagNumber(2)
  set ruleSet($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasRuleSet() => $_has(1);
  @$pb.TagNumber(2)
  void clearRuleSet() => $_clearField(2);

  /// 使用されたルールセットのバージョン
  @$pb.TagNumber(3)
  $core.int get ruleSetVersion => $_getIZ(2);
  @$pb.TagNumber(3)
  set ruleSetVersion($core.int value) => $_setUnsignedInt32(2, value);
  @$pb.TagNumber(3)
  $core.bool hasRuleSetVersion() => $_has(2);
  @$pb.TagNumber(3)
  void clearRuleSetVersion() => $_clearField(3);

  /// マッチしたルールのリスト
  @$pb.TagNumber(4)
  $pb.PbList<MatchedRule> get matchedRules => $_getList(3);

  /// 最終評価結果（JSON 形式）
  @$pb.TagNumber(5)
  $core.List<$core.int> get resultJson => $_getN(4);
  @$pb.TagNumber(5)
  set resultJson($core.List<$core.int> value) => $_setBytes(4, value);
  @$pb.TagNumber(5)
  $core.bool hasResultJson() => $_has(4);
  @$pb.TagNumber(5)
  void clearResultJson() => $_clearField(5);

  /// デフォルト値が適用されたかどうか
  @$pb.TagNumber(6)
  $core.bool get defaultApplied => $_getBF(5);
  @$pb.TagNumber(6)
  set defaultApplied($core.bool value) => $_setBool(5, value);
  @$pb.TagNumber(6)
  $core.bool hasDefaultApplied() => $_has(5);
  @$pb.TagNumber(6)
  void clearDefaultApplied() => $_clearField(6);

  /// キャッシュから返されたかどうか
  @$pb.TagNumber(7)
  $core.bool get cached => $_getBF(6);
  @$pb.TagNumber(7)
  set cached($core.bool value) => $_setBool(6, value);
  @$pb.TagNumber(7)
  $core.bool hasCached() => $_has(6);
  @$pb.TagNumber(7)
  void clearCached() => $_clearField(7);

  /// 評価日時
  @$pb.TagNumber(8)
  $1.Timestamp get evaluatedAt => $_getN(7);
  @$pb.TagNumber(8)
  set evaluatedAt($1.Timestamp value) => $_setField(8, value);
  @$pb.TagNumber(8)
  $core.bool hasEvaluatedAt() => $_has(7);
  @$pb.TagNumber(8)
  void clearEvaluatedAt() => $_clearField(8);
  @$pb.TagNumber(8)
  $1.Timestamp ensureEvaluatedAt() => $_ensure(7);
}

const $core.bool _omitFieldNames =
    $core.bool.fromEnvironment('protobuf.omit_field_names');
const $core.bool _omitMessageNames =
    $core.bool.fromEnvironment('protobuf.omit_message_names');
