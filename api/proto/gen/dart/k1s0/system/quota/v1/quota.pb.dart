// This is a generated file - do not edit.
//
// Generated from k1s0/system/quota/v1/quota.proto.

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

export 'package:protobuf/protobuf.dart' show GeneratedMessageGenericExtensions;

class QuotaPolicy extends $pb.GeneratedMessage {
  factory QuotaPolicy({
    $core.String? id,
    $core.String? name,
    $core.String? subjectType,
    $core.String? subjectId,
    $fixnum.Int64? limit,
    $core.String? period,
    $core.bool? enabled,
    $core.int? alertThresholdPercent,
    $1.Timestamp? createdAt,
    $1.Timestamp? updatedAt,
  }) {
    final result = create();
    if (id != null) result.id = id;
    if (name != null) result.name = name;
    if (subjectType != null) result.subjectType = subjectType;
    if (subjectId != null) result.subjectId = subjectId;
    if (limit != null) result.limit = limit;
    if (period != null) result.period = period;
    if (enabled != null) result.enabled = enabled;
    if (alertThresholdPercent != null)
      result.alertThresholdPercent = alertThresholdPercent;
    if (createdAt != null) result.createdAt = createdAt;
    if (updatedAt != null) result.updatedAt = updatedAt;
    return result;
  }

  QuotaPolicy._();

  factory QuotaPolicy.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory QuotaPolicy.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'QuotaPolicy',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.quota.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'id')
    ..aOS(2, _omitFieldNames ? '' : 'name')
    ..aOS(3, _omitFieldNames ? '' : 'subjectType')
    ..aOS(4, _omitFieldNames ? '' : 'subjectId')
    ..a<$fixnum.Int64>(5, _omitFieldNames ? '' : 'limit', $pb.PbFieldType.OU6,
        defaultOrMaker: $fixnum.Int64.ZERO)
    ..aOS(6, _omitFieldNames ? '' : 'period')
    ..aOB(7, _omitFieldNames ? '' : 'enabled')
    ..aI(8, _omitFieldNames ? '' : 'alertThresholdPercent',
        fieldType: $pb.PbFieldType.OU3)
    ..aOM<$1.Timestamp>(9, _omitFieldNames ? '' : 'createdAt',
        subBuilder: $1.Timestamp.create)
    ..aOM<$1.Timestamp>(10, _omitFieldNames ? '' : 'updatedAt',
        subBuilder: $1.Timestamp.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  QuotaPolicy clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  QuotaPolicy copyWith(void Function(QuotaPolicy) updates) =>
      super.copyWith((message) => updates(message as QuotaPolicy))
          as QuotaPolicy;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static QuotaPolicy create() => QuotaPolicy._();
  @$core.override
  QuotaPolicy createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static QuotaPolicy getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<QuotaPolicy>(create);
  static QuotaPolicy? _defaultInstance;

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
  $core.String get subjectType => $_getSZ(2);
  @$pb.TagNumber(3)
  set subjectType($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasSubjectType() => $_has(2);
  @$pb.TagNumber(3)
  void clearSubjectType() => $_clearField(3);

  @$pb.TagNumber(4)
  $core.String get subjectId => $_getSZ(3);
  @$pb.TagNumber(4)
  set subjectId($core.String value) => $_setString(3, value);
  @$pb.TagNumber(4)
  $core.bool hasSubjectId() => $_has(3);
  @$pb.TagNumber(4)
  void clearSubjectId() => $_clearField(4);

  @$pb.TagNumber(5)
  $fixnum.Int64 get limit => $_getI64(4);
  @$pb.TagNumber(5)
  set limit($fixnum.Int64 value) => $_setInt64(4, value);
  @$pb.TagNumber(5)
  $core.bool hasLimit() => $_has(4);
  @$pb.TagNumber(5)
  void clearLimit() => $_clearField(5);

  @$pb.TagNumber(6)
  $core.String get period => $_getSZ(5);
  @$pb.TagNumber(6)
  set period($core.String value) => $_setString(5, value);
  @$pb.TagNumber(6)
  $core.bool hasPeriod() => $_has(5);
  @$pb.TagNumber(6)
  void clearPeriod() => $_clearField(6);

  @$pb.TagNumber(7)
  $core.bool get enabled => $_getBF(6);
  @$pb.TagNumber(7)
  set enabled($core.bool value) => $_setBool(6, value);
  @$pb.TagNumber(7)
  $core.bool hasEnabled() => $_has(6);
  @$pb.TagNumber(7)
  void clearEnabled() => $_clearField(7);

  @$pb.TagNumber(8)
  $core.int get alertThresholdPercent => $_getIZ(7);
  @$pb.TagNumber(8)
  set alertThresholdPercent($core.int value) => $_setUnsignedInt32(7, value);
  @$pb.TagNumber(8)
  $core.bool hasAlertThresholdPercent() => $_has(7);
  @$pb.TagNumber(8)
  void clearAlertThresholdPercent() => $_clearField(8);

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

class QuotaUsage extends $pb.GeneratedMessage {
  factory QuotaUsage({
    $core.String? quotaId,
    $core.String? subjectType,
    $core.String? subjectId,
    $core.String? period,
    $fixnum.Int64? limit,
    $fixnum.Int64? used,
    $fixnum.Int64? remaining,
    $core.double? usagePercent,
    $core.bool? exceeded,
    $1.Timestamp? periodStart,
    $1.Timestamp? periodEnd,
    $1.Timestamp? resetAt,
  }) {
    final result = create();
    if (quotaId != null) result.quotaId = quotaId;
    if (subjectType != null) result.subjectType = subjectType;
    if (subjectId != null) result.subjectId = subjectId;
    if (period != null) result.period = period;
    if (limit != null) result.limit = limit;
    if (used != null) result.used = used;
    if (remaining != null) result.remaining = remaining;
    if (usagePercent != null) result.usagePercent = usagePercent;
    if (exceeded != null) result.exceeded = exceeded;
    if (periodStart != null) result.periodStart = periodStart;
    if (periodEnd != null) result.periodEnd = periodEnd;
    if (resetAt != null) result.resetAt = resetAt;
    return result;
  }

  QuotaUsage._();

  factory QuotaUsage.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory QuotaUsage.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'QuotaUsage',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.quota.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'quotaId')
    ..aOS(2, _omitFieldNames ? '' : 'subjectType')
    ..aOS(3, _omitFieldNames ? '' : 'subjectId')
    ..aOS(4, _omitFieldNames ? '' : 'period')
    ..a<$fixnum.Int64>(5, _omitFieldNames ? '' : 'limit', $pb.PbFieldType.OU6,
        defaultOrMaker: $fixnum.Int64.ZERO)
    ..a<$fixnum.Int64>(6, _omitFieldNames ? '' : 'used', $pb.PbFieldType.OU6,
        defaultOrMaker: $fixnum.Int64.ZERO)
    ..a<$fixnum.Int64>(
        7, _omitFieldNames ? '' : 'remaining', $pb.PbFieldType.OU6,
        defaultOrMaker: $fixnum.Int64.ZERO)
    ..aD(8, _omitFieldNames ? '' : 'usagePercent')
    ..aOB(9, _omitFieldNames ? '' : 'exceeded')
    ..aOM<$1.Timestamp>(10, _omitFieldNames ? '' : 'periodStart',
        subBuilder: $1.Timestamp.create)
    ..aOM<$1.Timestamp>(11, _omitFieldNames ? '' : 'periodEnd',
        subBuilder: $1.Timestamp.create)
    ..aOM<$1.Timestamp>(12, _omitFieldNames ? '' : 'resetAt',
        subBuilder: $1.Timestamp.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  QuotaUsage clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  QuotaUsage copyWith(void Function(QuotaUsage) updates) =>
      super.copyWith((message) => updates(message as QuotaUsage)) as QuotaUsage;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static QuotaUsage create() => QuotaUsage._();
  @$core.override
  QuotaUsage createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static QuotaUsage getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<QuotaUsage>(create);
  static QuotaUsage? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get quotaId => $_getSZ(0);
  @$pb.TagNumber(1)
  set quotaId($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasQuotaId() => $_has(0);
  @$pb.TagNumber(1)
  void clearQuotaId() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get subjectType => $_getSZ(1);
  @$pb.TagNumber(2)
  set subjectType($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasSubjectType() => $_has(1);
  @$pb.TagNumber(2)
  void clearSubjectType() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.String get subjectId => $_getSZ(2);
  @$pb.TagNumber(3)
  set subjectId($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasSubjectId() => $_has(2);
  @$pb.TagNumber(3)
  void clearSubjectId() => $_clearField(3);

  @$pb.TagNumber(4)
  $core.String get period => $_getSZ(3);
  @$pb.TagNumber(4)
  set period($core.String value) => $_setString(3, value);
  @$pb.TagNumber(4)
  $core.bool hasPeriod() => $_has(3);
  @$pb.TagNumber(4)
  void clearPeriod() => $_clearField(4);

  @$pb.TagNumber(5)
  $fixnum.Int64 get limit => $_getI64(4);
  @$pb.TagNumber(5)
  set limit($fixnum.Int64 value) => $_setInt64(4, value);
  @$pb.TagNumber(5)
  $core.bool hasLimit() => $_has(4);
  @$pb.TagNumber(5)
  void clearLimit() => $_clearField(5);

  @$pb.TagNumber(6)
  $fixnum.Int64 get used => $_getI64(5);
  @$pb.TagNumber(6)
  set used($fixnum.Int64 value) => $_setInt64(5, value);
  @$pb.TagNumber(6)
  $core.bool hasUsed() => $_has(5);
  @$pb.TagNumber(6)
  void clearUsed() => $_clearField(6);

  @$pb.TagNumber(7)
  $fixnum.Int64 get remaining => $_getI64(6);
  @$pb.TagNumber(7)
  set remaining($fixnum.Int64 value) => $_setInt64(6, value);
  @$pb.TagNumber(7)
  $core.bool hasRemaining() => $_has(6);
  @$pb.TagNumber(7)
  void clearRemaining() => $_clearField(7);

  @$pb.TagNumber(8)
  $core.double get usagePercent => $_getN(7);
  @$pb.TagNumber(8)
  set usagePercent($core.double value) => $_setDouble(7, value);
  @$pb.TagNumber(8)
  $core.bool hasUsagePercent() => $_has(7);
  @$pb.TagNumber(8)
  void clearUsagePercent() => $_clearField(8);

  @$pb.TagNumber(9)
  $core.bool get exceeded => $_getBF(8);
  @$pb.TagNumber(9)
  set exceeded($core.bool value) => $_setBool(8, value);
  @$pb.TagNumber(9)
  $core.bool hasExceeded() => $_has(8);
  @$pb.TagNumber(9)
  void clearExceeded() => $_clearField(9);

  @$pb.TagNumber(10)
  $1.Timestamp get periodStart => $_getN(9);
  @$pb.TagNumber(10)
  set periodStart($1.Timestamp value) => $_setField(10, value);
  @$pb.TagNumber(10)
  $core.bool hasPeriodStart() => $_has(9);
  @$pb.TagNumber(10)
  void clearPeriodStart() => $_clearField(10);
  @$pb.TagNumber(10)
  $1.Timestamp ensurePeriodStart() => $_ensure(9);

  @$pb.TagNumber(11)
  $1.Timestamp get periodEnd => $_getN(10);
  @$pb.TagNumber(11)
  set periodEnd($1.Timestamp value) => $_setField(11, value);
  @$pb.TagNumber(11)
  $core.bool hasPeriodEnd() => $_has(10);
  @$pb.TagNumber(11)
  void clearPeriodEnd() => $_clearField(11);
  @$pb.TagNumber(11)
  $1.Timestamp ensurePeriodEnd() => $_ensure(10);

  @$pb.TagNumber(12)
  $1.Timestamp get resetAt => $_getN(11);
  @$pb.TagNumber(12)
  set resetAt($1.Timestamp value) => $_setField(12, value);
  @$pb.TagNumber(12)
  $core.bool hasResetAt() => $_has(11);
  @$pb.TagNumber(12)
  void clearResetAt() => $_clearField(12);
  @$pb.TagNumber(12)
  $1.Timestamp ensureResetAt() => $_ensure(11);
}

class CreateQuotaPolicyRequest extends $pb.GeneratedMessage {
  factory CreateQuotaPolicyRequest({
    $core.String? name,
    $core.String? subjectType,
    $core.String? subjectId,
    $fixnum.Int64? limit,
    $core.String? period,
    $core.bool? enabled,
    $core.int? alertThresholdPercent,
  }) {
    final result = create();
    if (name != null) result.name = name;
    if (subjectType != null) result.subjectType = subjectType;
    if (subjectId != null) result.subjectId = subjectId;
    if (limit != null) result.limit = limit;
    if (period != null) result.period = period;
    if (enabled != null) result.enabled = enabled;
    if (alertThresholdPercent != null)
      result.alertThresholdPercent = alertThresholdPercent;
    return result;
  }

  CreateQuotaPolicyRequest._();

  factory CreateQuotaPolicyRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory CreateQuotaPolicyRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'CreateQuotaPolicyRequest',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.quota.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'name')
    ..aOS(2, _omitFieldNames ? '' : 'subjectType')
    ..aOS(3, _omitFieldNames ? '' : 'subjectId')
    ..a<$fixnum.Int64>(4, _omitFieldNames ? '' : 'limit', $pb.PbFieldType.OU6,
        defaultOrMaker: $fixnum.Int64.ZERO)
    ..aOS(5, _omitFieldNames ? '' : 'period')
    ..aOB(6, _omitFieldNames ? '' : 'enabled')
    ..aI(7, _omitFieldNames ? '' : 'alertThresholdPercent',
        fieldType: $pb.PbFieldType.OU3)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  CreateQuotaPolicyRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  CreateQuotaPolicyRequest copyWith(
          void Function(CreateQuotaPolicyRequest) updates) =>
      super.copyWith((message) => updates(message as CreateQuotaPolicyRequest))
          as CreateQuotaPolicyRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static CreateQuotaPolicyRequest create() => CreateQuotaPolicyRequest._();
  @$core.override
  CreateQuotaPolicyRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static CreateQuotaPolicyRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<CreateQuotaPolicyRequest>(create);
  static CreateQuotaPolicyRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get name => $_getSZ(0);
  @$pb.TagNumber(1)
  set name($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasName() => $_has(0);
  @$pb.TagNumber(1)
  void clearName() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get subjectType => $_getSZ(1);
  @$pb.TagNumber(2)
  set subjectType($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasSubjectType() => $_has(1);
  @$pb.TagNumber(2)
  void clearSubjectType() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.String get subjectId => $_getSZ(2);
  @$pb.TagNumber(3)
  set subjectId($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasSubjectId() => $_has(2);
  @$pb.TagNumber(3)
  void clearSubjectId() => $_clearField(3);

  @$pb.TagNumber(4)
  $fixnum.Int64 get limit => $_getI64(3);
  @$pb.TagNumber(4)
  set limit($fixnum.Int64 value) => $_setInt64(3, value);
  @$pb.TagNumber(4)
  $core.bool hasLimit() => $_has(3);
  @$pb.TagNumber(4)
  void clearLimit() => $_clearField(4);

  @$pb.TagNumber(5)
  $core.String get period => $_getSZ(4);
  @$pb.TagNumber(5)
  set period($core.String value) => $_setString(4, value);
  @$pb.TagNumber(5)
  $core.bool hasPeriod() => $_has(4);
  @$pb.TagNumber(5)
  void clearPeriod() => $_clearField(5);

  @$pb.TagNumber(6)
  $core.bool get enabled => $_getBF(5);
  @$pb.TagNumber(6)
  set enabled($core.bool value) => $_setBool(5, value);
  @$pb.TagNumber(6)
  $core.bool hasEnabled() => $_has(5);
  @$pb.TagNumber(6)
  void clearEnabled() => $_clearField(6);

  @$pb.TagNumber(7)
  $core.int get alertThresholdPercent => $_getIZ(6);
  @$pb.TagNumber(7)
  set alertThresholdPercent($core.int value) => $_setUnsignedInt32(6, value);
  @$pb.TagNumber(7)
  $core.bool hasAlertThresholdPercent() => $_has(6);
  @$pb.TagNumber(7)
  void clearAlertThresholdPercent() => $_clearField(7);
}

class CreateQuotaPolicyResponse extends $pb.GeneratedMessage {
  factory CreateQuotaPolicyResponse({
    QuotaPolicy? policy,
  }) {
    final result = create();
    if (policy != null) result.policy = policy;
    return result;
  }

  CreateQuotaPolicyResponse._();

  factory CreateQuotaPolicyResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory CreateQuotaPolicyResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'CreateQuotaPolicyResponse',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.quota.v1'),
      createEmptyInstance: create)
    ..aOM<QuotaPolicy>(1, _omitFieldNames ? '' : 'policy',
        subBuilder: QuotaPolicy.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  CreateQuotaPolicyResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  CreateQuotaPolicyResponse copyWith(
          void Function(CreateQuotaPolicyResponse) updates) =>
      super.copyWith((message) => updates(message as CreateQuotaPolicyResponse))
          as CreateQuotaPolicyResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static CreateQuotaPolicyResponse create() => CreateQuotaPolicyResponse._();
  @$core.override
  CreateQuotaPolicyResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static CreateQuotaPolicyResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<CreateQuotaPolicyResponse>(create);
  static CreateQuotaPolicyResponse? _defaultInstance;

  @$pb.TagNumber(1)
  QuotaPolicy get policy => $_getN(0);
  @$pb.TagNumber(1)
  set policy(QuotaPolicy value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasPolicy() => $_has(0);
  @$pb.TagNumber(1)
  void clearPolicy() => $_clearField(1);
  @$pb.TagNumber(1)
  QuotaPolicy ensurePolicy() => $_ensure(0);
}

class GetQuotaPolicyRequest extends $pb.GeneratedMessage {
  factory GetQuotaPolicyRequest({
    $core.String? id,
  }) {
    final result = create();
    if (id != null) result.id = id;
    return result;
  }

  GetQuotaPolicyRequest._();

  factory GetQuotaPolicyRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory GetQuotaPolicyRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'GetQuotaPolicyRequest',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.quota.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'id')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetQuotaPolicyRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetQuotaPolicyRequest copyWith(
          void Function(GetQuotaPolicyRequest) updates) =>
      super.copyWith((message) => updates(message as GetQuotaPolicyRequest))
          as GetQuotaPolicyRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static GetQuotaPolicyRequest create() => GetQuotaPolicyRequest._();
  @$core.override
  GetQuotaPolicyRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static GetQuotaPolicyRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<GetQuotaPolicyRequest>(create);
  static GetQuotaPolicyRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get id => $_getSZ(0);
  @$pb.TagNumber(1)
  set id($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasId() => $_has(0);
  @$pb.TagNumber(1)
  void clearId() => $_clearField(1);
}

class GetQuotaPolicyResponse extends $pb.GeneratedMessage {
  factory GetQuotaPolicyResponse({
    QuotaPolicy? policy,
  }) {
    final result = create();
    if (policy != null) result.policy = policy;
    return result;
  }

  GetQuotaPolicyResponse._();

  factory GetQuotaPolicyResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory GetQuotaPolicyResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'GetQuotaPolicyResponse',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.quota.v1'),
      createEmptyInstance: create)
    ..aOM<QuotaPolicy>(1, _omitFieldNames ? '' : 'policy',
        subBuilder: QuotaPolicy.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetQuotaPolicyResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetQuotaPolicyResponse copyWith(
          void Function(GetQuotaPolicyResponse) updates) =>
      super.copyWith((message) => updates(message as GetQuotaPolicyResponse))
          as GetQuotaPolicyResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static GetQuotaPolicyResponse create() => GetQuotaPolicyResponse._();
  @$core.override
  GetQuotaPolicyResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static GetQuotaPolicyResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<GetQuotaPolicyResponse>(create);
  static GetQuotaPolicyResponse? _defaultInstance;

  @$pb.TagNumber(1)
  QuotaPolicy get policy => $_getN(0);
  @$pb.TagNumber(1)
  set policy(QuotaPolicy value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasPolicy() => $_has(0);
  @$pb.TagNumber(1)
  void clearPolicy() => $_clearField(1);
  @$pb.TagNumber(1)
  QuotaPolicy ensurePolicy() => $_ensure(0);
}

class ListQuotaPoliciesRequest extends $pb.GeneratedMessage {
  factory ListQuotaPoliciesRequest({
    $1.Pagination? pagination,
    $core.String? subjectType,
    $core.String? subjectId,
    $core.bool? enabledOnly,
  }) {
    final result = create();
    if (pagination != null) result.pagination = pagination;
    if (subjectType != null) result.subjectType = subjectType;
    if (subjectId != null) result.subjectId = subjectId;
    if (enabledOnly != null) result.enabledOnly = enabledOnly;
    return result;
  }

  ListQuotaPoliciesRequest._();

  factory ListQuotaPoliciesRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory ListQuotaPoliciesRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ListQuotaPoliciesRequest',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.quota.v1'),
      createEmptyInstance: create)
    ..aOM<$1.Pagination>(1, _omitFieldNames ? '' : 'pagination',
        subBuilder: $1.Pagination.create)
    ..aOS(3, _omitFieldNames ? '' : 'subjectType')
    ..aOS(4, _omitFieldNames ? '' : 'subjectId')
    ..aOB(5, _omitFieldNames ? '' : 'enabledOnly')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListQuotaPoliciesRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListQuotaPoliciesRequest copyWith(
          void Function(ListQuotaPoliciesRequest) updates) =>
      super.copyWith((message) => updates(message as ListQuotaPoliciesRequest))
          as ListQuotaPoliciesRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static ListQuotaPoliciesRequest create() => ListQuotaPoliciesRequest._();
  @$core.override
  ListQuotaPoliciesRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static ListQuotaPoliciesRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ListQuotaPoliciesRequest>(create);
  static ListQuotaPoliciesRequest? _defaultInstance;

  /// ページネーションパラメータを共通型に統一
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

  @$pb.TagNumber(3)
  $core.String get subjectType => $_getSZ(1);
  @$pb.TagNumber(3)
  set subjectType($core.String value) => $_setString(1, value);
  @$pb.TagNumber(3)
  $core.bool hasSubjectType() => $_has(1);
  @$pb.TagNumber(3)
  void clearSubjectType() => $_clearField(3);

  @$pb.TagNumber(4)
  $core.String get subjectId => $_getSZ(2);
  @$pb.TagNumber(4)
  set subjectId($core.String value) => $_setString(2, value);
  @$pb.TagNumber(4)
  $core.bool hasSubjectId() => $_has(2);
  @$pb.TagNumber(4)
  void clearSubjectId() => $_clearField(4);

  @$pb.TagNumber(5)
  $core.bool get enabledOnly => $_getBF(3);
  @$pb.TagNumber(5)
  set enabledOnly($core.bool value) => $_setBool(3, value);
  @$pb.TagNumber(5)
  $core.bool hasEnabledOnly() => $_has(3);
  @$pb.TagNumber(5)
  void clearEnabledOnly() => $_clearField(5);
}

class ListQuotaPoliciesResponse extends $pb.GeneratedMessage {
  factory ListQuotaPoliciesResponse({
    $core.Iterable<QuotaPolicy>? policies,
    $1.PaginationResult? pagination,
  }) {
    final result = create();
    if (policies != null) result.policies.addAll(policies);
    if (pagination != null) result.pagination = pagination;
    return result;
  }

  ListQuotaPoliciesResponse._();

  factory ListQuotaPoliciesResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory ListQuotaPoliciesResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ListQuotaPoliciesResponse',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.quota.v1'),
      createEmptyInstance: create)
    ..pPM<QuotaPolicy>(1, _omitFieldNames ? '' : 'policies',
        subBuilder: QuotaPolicy.create)
    ..aOM<$1.PaginationResult>(2, _omitFieldNames ? '' : 'pagination',
        subBuilder: $1.PaginationResult.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListQuotaPoliciesResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListQuotaPoliciesResponse copyWith(
          void Function(ListQuotaPoliciesResponse) updates) =>
      super.copyWith((message) => updates(message as ListQuotaPoliciesResponse))
          as ListQuotaPoliciesResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static ListQuotaPoliciesResponse create() => ListQuotaPoliciesResponse._();
  @$core.override
  ListQuotaPoliciesResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static ListQuotaPoliciesResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ListQuotaPoliciesResponse>(create);
  static ListQuotaPoliciesResponse? _defaultInstance;

  @$pb.TagNumber(1)
  $pb.PbList<QuotaPolicy> get policies => $_getList(0);

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

class UpdateQuotaPolicyRequest extends $pb.GeneratedMessage {
  factory UpdateQuotaPolicyRequest({
    $core.String? id,
    $core.bool? enabled,
    $fixnum.Int64? limit,
    $core.String? name,
    $core.String? subjectType,
    $core.String? subjectId,
    $core.String? period,
    $core.int? alertThresholdPercent,
  }) {
    final result = create();
    if (id != null) result.id = id;
    if (enabled != null) result.enabled = enabled;
    if (limit != null) result.limit = limit;
    if (name != null) result.name = name;
    if (subjectType != null) result.subjectType = subjectType;
    if (subjectId != null) result.subjectId = subjectId;
    if (period != null) result.period = period;
    if (alertThresholdPercent != null)
      result.alertThresholdPercent = alertThresholdPercent;
    return result;
  }

  UpdateQuotaPolicyRequest._();

  factory UpdateQuotaPolicyRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory UpdateQuotaPolicyRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'UpdateQuotaPolicyRequest',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.quota.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'id')
    ..aOB(2, _omitFieldNames ? '' : 'enabled')
    ..a<$fixnum.Int64>(3, _omitFieldNames ? '' : 'limit', $pb.PbFieldType.OU6,
        defaultOrMaker: $fixnum.Int64.ZERO)
    ..aOS(4, _omitFieldNames ? '' : 'name')
    ..aOS(5, _omitFieldNames ? '' : 'subjectType')
    ..aOS(6, _omitFieldNames ? '' : 'subjectId')
    ..aOS(7, _omitFieldNames ? '' : 'period')
    ..aI(8, _omitFieldNames ? '' : 'alertThresholdPercent',
        fieldType: $pb.PbFieldType.OU3)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  UpdateQuotaPolicyRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  UpdateQuotaPolicyRequest copyWith(
          void Function(UpdateQuotaPolicyRequest) updates) =>
      super.copyWith((message) => updates(message as UpdateQuotaPolicyRequest))
          as UpdateQuotaPolicyRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static UpdateQuotaPolicyRequest create() => UpdateQuotaPolicyRequest._();
  @$core.override
  UpdateQuotaPolicyRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static UpdateQuotaPolicyRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<UpdateQuotaPolicyRequest>(create);
  static UpdateQuotaPolicyRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get id => $_getSZ(0);
  @$pb.TagNumber(1)
  set id($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasId() => $_has(0);
  @$pb.TagNumber(1)
  void clearId() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.bool get enabled => $_getBF(1);
  @$pb.TagNumber(2)
  set enabled($core.bool value) => $_setBool(1, value);
  @$pb.TagNumber(2)
  $core.bool hasEnabled() => $_has(1);
  @$pb.TagNumber(2)
  void clearEnabled() => $_clearField(2);

  @$pb.TagNumber(3)
  $fixnum.Int64 get limit => $_getI64(2);
  @$pb.TagNumber(3)
  set limit($fixnum.Int64 value) => $_setInt64(2, value);
  @$pb.TagNumber(3)
  $core.bool hasLimit() => $_has(2);
  @$pb.TagNumber(3)
  void clearLimit() => $_clearField(3);

  @$pb.TagNumber(4)
  $core.String get name => $_getSZ(3);
  @$pb.TagNumber(4)
  set name($core.String value) => $_setString(3, value);
  @$pb.TagNumber(4)
  $core.bool hasName() => $_has(3);
  @$pb.TagNumber(4)
  void clearName() => $_clearField(4);

  @$pb.TagNumber(5)
  $core.String get subjectType => $_getSZ(4);
  @$pb.TagNumber(5)
  set subjectType($core.String value) => $_setString(4, value);
  @$pb.TagNumber(5)
  $core.bool hasSubjectType() => $_has(4);
  @$pb.TagNumber(5)
  void clearSubjectType() => $_clearField(5);

  @$pb.TagNumber(6)
  $core.String get subjectId => $_getSZ(5);
  @$pb.TagNumber(6)
  set subjectId($core.String value) => $_setString(5, value);
  @$pb.TagNumber(6)
  $core.bool hasSubjectId() => $_has(5);
  @$pb.TagNumber(6)
  void clearSubjectId() => $_clearField(6);

  @$pb.TagNumber(7)
  $core.String get period => $_getSZ(6);
  @$pb.TagNumber(7)
  set period($core.String value) => $_setString(6, value);
  @$pb.TagNumber(7)
  $core.bool hasPeriod() => $_has(6);
  @$pb.TagNumber(7)
  void clearPeriod() => $_clearField(7);

  @$pb.TagNumber(8)
  $core.int get alertThresholdPercent => $_getIZ(7);
  @$pb.TagNumber(8)
  set alertThresholdPercent($core.int value) => $_setUnsignedInt32(7, value);
  @$pb.TagNumber(8)
  $core.bool hasAlertThresholdPercent() => $_has(7);
  @$pb.TagNumber(8)
  void clearAlertThresholdPercent() => $_clearField(8);
}

class UpdateQuotaPolicyResponse extends $pb.GeneratedMessage {
  factory UpdateQuotaPolicyResponse({
    QuotaPolicy? policy,
  }) {
    final result = create();
    if (policy != null) result.policy = policy;
    return result;
  }

  UpdateQuotaPolicyResponse._();

  factory UpdateQuotaPolicyResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory UpdateQuotaPolicyResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'UpdateQuotaPolicyResponse',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.quota.v1'),
      createEmptyInstance: create)
    ..aOM<QuotaPolicy>(1, _omitFieldNames ? '' : 'policy',
        subBuilder: QuotaPolicy.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  UpdateQuotaPolicyResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  UpdateQuotaPolicyResponse copyWith(
          void Function(UpdateQuotaPolicyResponse) updates) =>
      super.copyWith((message) => updates(message as UpdateQuotaPolicyResponse))
          as UpdateQuotaPolicyResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static UpdateQuotaPolicyResponse create() => UpdateQuotaPolicyResponse._();
  @$core.override
  UpdateQuotaPolicyResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static UpdateQuotaPolicyResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<UpdateQuotaPolicyResponse>(create);
  static UpdateQuotaPolicyResponse? _defaultInstance;

  @$pb.TagNumber(1)
  QuotaPolicy get policy => $_getN(0);
  @$pb.TagNumber(1)
  set policy(QuotaPolicy value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasPolicy() => $_has(0);
  @$pb.TagNumber(1)
  void clearPolicy() => $_clearField(1);
  @$pb.TagNumber(1)
  QuotaPolicy ensurePolicy() => $_ensure(0);
}

class DeleteQuotaPolicyRequest extends $pb.GeneratedMessage {
  factory DeleteQuotaPolicyRequest({
    $core.String? id,
  }) {
    final result = create();
    if (id != null) result.id = id;
    return result;
  }

  DeleteQuotaPolicyRequest._();

  factory DeleteQuotaPolicyRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory DeleteQuotaPolicyRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'DeleteQuotaPolicyRequest',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.quota.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'id')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  DeleteQuotaPolicyRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  DeleteQuotaPolicyRequest copyWith(
          void Function(DeleteQuotaPolicyRequest) updates) =>
      super.copyWith((message) => updates(message as DeleteQuotaPolicyRequest))
          as DeleteQuotaPolicyRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static DeleteQuotaPolicyRequest create() => DeleteQuotaPolicyRequest._();
  @$core.override
  DeleteQuotaPolicyRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static DeleteQuotaPolicyRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<DeleteQuotaPolicyRequest>(create);
  static DeleteQuotaPolicyRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get id => $_getSZ(0);
  @$pb.TagNumber(1)
  set id($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasId() => $_has(0);
  @$pb.TagNumber(1)
  void clearId() => $_clearField(1);
}

class DeleteQuotaPolicyResponse extends $pb.GeneratedMessage {
  factory DeleteQuotaPolicyResponse({
    $core.String? id,
    $core.bool? deleted,
  }) {
    final result = create();
    if (id != null) result.id = id;
    if (deleted != null) result.deleted = deleted;
    return result;
  }

  DeleteQuotaPolicyResponse._();

  factory DeleteQuotaPolicyResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory DeleteQuotaPolicyResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'DeleteQuotaPolicyResponse',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.quota.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'id')
    ..aOB(2, _omitFieldNames ? '' : 'deleted')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  DeleteQuotaPolicyResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  DeleteQuotaPolicyResponse copyWith(
          void Function(DeleteQuotaPolicyResponse) updates) =>
      super.copyWith((message) => updates(message as DeleteQuotaPolicyResponse))
          as DeleteQuotaPolicyResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static DeleteQuotaPolicyResponse create() => DeleteQuotaPolicyResponse._();
  @$core.override
  DeleteQuotaPolicyResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static DeleteQuotaPolicyResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<DeleteQuotaPolicyResponse>(create);
  static DeleteQuotaPolicyResponse? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get id => $_getSZ(0);
  @$pb.TagNumber(1)
  set id($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasId() => $_has(0);
  @$pb.TagNumber(1)
  void clearId() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.bool get deleted => $_getBF(1);
  @$pb.TagNumber(2)
  set deleted($core.bool value) => $_setBool(1, value);
  @$pb.TagNumber(2)
  $core.bool hasDeleted() => $_has(1);
  @$pb.TagNumber(2)
  void clearDeleted() => $_clearField(2);
}

class GetQuotaUsageRequest extends $pb.GeneratedMessage {
  factory GetQuotaUsageRequest({
    $core.String? quotaId,
  }) {
    final result = create();
    if (quotaId != null) result.quotaId = quotaId;
    return result;
  }

  GetQuotaUsageRequest._();

  factory GetQuotaUsageRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory GetQuotaUsageRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'GetQuotaUsageRequest',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.quota.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'quotaId')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetQuotaUsageRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetQuotaUsageRequest copyWith(void Function(GetQuotaUsageRequest) updates) =>
      super.copyWith((message) => updates(message as GetQuotaUsageRequest))
          as GetQuotaUsageRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static GetQuotaUsageRequest create() => GetQuotaUsageRequest._();
  @$core.override
  GetQuotaUsageRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static GetQuotaUsageRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<GetQuotaUsageRequest>(create);
  static GetQuotaUsageRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get quotaId => $_getSZ(0);
  @$pb.TagNumber(1)
  set quotaId($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasQuotaId() => $_has(0);
  @$pb.TagNumber(1)
  void clearQuotaId() => $_clearField(1);
}

class GetQuotaUsageResponse extends $pb.GeneratedMessage {
  factory GetQuotaUsageResponse({
    QuotaUsage? usage,
  }) {
    final result = create();
    if (usage != null) result.usage = usage;
    return result;
  }

  GetQuotaUsageResponse._();

  factory GetQuotaUsageResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory GetQuotaUsageResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'GetQuotaUsageResponse',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.quota.v1'),
      createEmptyInstance: create)
    ..aOM<QuotaUsage>(1, _omitFieldNames ? '' : 'usage',
        subBuilder: QuotaUsage.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetQuotaUsageResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetQuotaUsageResponse copyWith(
          void Function(GetQuotaUsageResponse) updates) =>
      super.copyWith((message) => updates(message as GetQuotaUsageResponse))
          as GetQuotaUsageResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static GetQuotaUsageResponse create() => GetQuotaUsageResponse._();
  @$core.override
  GetQuotaUsageResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static GetQuotaUsageResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<GetQuotaUsageResponse>(create);
  static GetQuotaUsageResponse? _defaultInstance;

  @$pb.TagNumber(1)
  QuotaUsage get usage => $_getN(0);
  @$pb.TagNumber(1)
  set usage(QuotaUsage value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasUsage() => $_has(0);
  @$pb.TagNumber(1)
  void clearUsage() => $_clearField(1);
  @$pb.TagNumber(1)
  QuotaUsage ensureUsage() => $_ensure(0);
}

class CheckQuotaRequest extends $pb.GeneratedMessage {
  factory CheckQuotaRequest({
    $core.String? quotaId,
  }) {
    final result = create();
    if (quotaId != null) result.quotaId = quotaId;
    return result;
  }

  CheckQuotaRequest._();

  factory CheckQuotaRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory CheckQuotaRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'CheckQuotaRequest',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.quota.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'quotaId')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  CheckQuotaRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  CheckQuotaRequest copyWith(void Function(CheckQuotaRequest) updates) =>
      super.copyWith((message) => updates(message as CheckQuotaRequest))
          as CheckQuotaRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static CheckQuotaRequest create() => CheckQuotaRequest._();
  @$core.override
  CheckQuotaRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static CheckQuotaRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<CheckQuotaRequest>(create);
  static CheckQuotaRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get quotaId => $_getSZ(0);
  @$pb.TagNumber(1)
  set quotaId($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasQuotaId() => $_has(0);
  @$pb.TagNumber(1)
  void clearQuotaId() => $_clearField(1);
}

class CheckQuotaResponse extends $pb.GeneratedMessage {
  factory CheckQuotaResponse({
    QuotaUsage? usage,
  }) {
    final result = create();
    if (usage != null) result.usage = usage;
    return result;
  }

  CheckQuotaResponse._();

  factory CheckQuotaResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory CheckQuotaResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'CheckQuotaResponse',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.quota.v1'),
      createEmptyInstance: create)
    ..aOM<QuotaUsage>(1, _omitFieldNames ? '' : 'usage',
        subBuilder: QuotaUsage.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  CheckQuotaResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  CheckQuotaResponse copyWith(void Function(CheckQuotaResponse) updates) =>
      super.copyWith((message) => updates(message as CheckQuotaResponse))
          as CheckQuotaResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static CheckQuotaResponse create() => CheckQuotaResponse._();
  @$core.override
  CheckQuotaResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static CheckQuotaResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<CheckQuotaResponse>(create);
  static CheckQuotaResponse? _defaultInstance;

  @$pb.TagNumber(1)
  QuotaUsage get usage => $_getN(0);
  @$pb.TagNumber(1)
  set usage(QuotaUsage value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasUsage() => $_has(0);
  @$pb.TagNumber(1)
  void clearUsage() => $_clearField(1);
  @$pb.TagNumber(1)
  QuotaUsage ensureUsage() => $_ensure(0);
}

class IncrementQuotaUsageRequest extends $pb.GeneratedMessage {
  factory IncrementQuotaUsageRequest({
    $core.String? quotaId,
    $fixnum.Int64? amount,
    $core.String? requestId,
  }) {
    final result = create();
    if (quotaId != null) result.quotaId = quotaId;
    if (amount != null) result.amount = amount;
    if (requestId != null) result.requestId = requestId;
    return result;
  }

  IncrementQuotaUsageRequest._();

  factory IncrementQuotaUsageRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory IncrementQuotaUsageRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'IncrementQuotaUsageRequest',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.quota.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'quotaId')
    ..a<$fixnum.Int64>(2, _omitFieldNames ? '' : 'amount', $pb.PbFieldType.OU6,
        defaultOrMaker: $fixnum.Int64.ZERO)
    ..aOS(3, _omitFieldNames ? '' : 'requestId')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  IncrementQuotaUsageRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  IncrementQuotaUsageRequest copyWith(
          void Function(IncrementQuotaUsageRequest) updates) =>
      super.copyWith(
              (message) => updates(message as IncrementQuotaUsageRequest))
          as IncrementQuotaUsageRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static IncrementQuotaUsageRequest create() => IncrementQuotaUsageRequest._();
  @$core.override
  IncrementQuotaUsageRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static IncrementQuotaUsageRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<IncrementQuotaUsageRequest>(create);
  static IncrementQuotaUsageRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get quotaId => $_getSZ(0);
  @$pb.TagNumber(1)
  set quotaId($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasQuotaId() => $_has(0);
  @$pb.TagNumber(1)
  void clearQuotaId() => $_clearField(1);

  @$pb.TagNumber(2)
  $fixnum.Int64 get amount => $_getI64(1);
  @$pb.TagNumber(2)
  set amount($fixnum.Int64 value) => $_setInt64(1, value);
  @$pb.TagNumber(2)
  $core.bool hasAmount() => $_has(1);
  @$pb.TagNumber(2)
  void clearAmount() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.String get requestId => $_getSZ(2);
  @$pb.TagNumber(3)
  set requestId($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasRequestId() => $_has(2);
  @$pb.TagNumber(3)
  void clearRequestId() => $_clearField(3);
}

class IncrementQuotaUsageResponse extends $pb.GeneratedMessage {
  factory IncrementQuotaUsageResponse({
    $core.String? quotaId,
    $fixnum.Int64? used,
    $fixnum.Int64? remaining,
    $core.double? usagePercent,
    $core.bool? exceeded,
    $core.bool? allowed,
  }) {
    final result = create();
    if (quotaId != null) result.quotaId = quotaId;
    if (used != null) result.used = used;
    if (remaining != null) result.remaining = remaining;
    if (usagePercent != null) result.usagePercent = usagePercent;
    if (exceeded != null) result.exceeded = exceeded;
    if (allowed != null) result.allowed = allowed;
    return result;
  }

  IncrementQuotaUsageResponse._();

  factory IncrementQuotaUsageResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory IncrementQuotaUsageResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'IncrementQuotaUsageResponse',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.quota.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'quotaId')
    ..a<$fixnum.Int64>(2, _omitFieldNames ? '' : 'used', $pb.PbFieldType.OU6,
        defaultOrMaker: $fixnum.Int64.ZERO)
    ..a<$fixnum.Int64>(
        3, _omitFieldNames ? '' : 'remaining', $pb.PbFieldType.OU6,
        defaultOrMaker: $fixnum.Int64.ZERO)
    ..aD(4, _omitFieldNames ? '' : 'usagePercent')
    ..aOB(5, _omitFieldNames ? '' : 'exceeded')
    ..aOB(6, _omitFieldNames ? '' : 'allowed')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  IncrementQuotaUsageResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  IncrementQuotaUsageResponse copyWith(
          void Function(IncrementQuotaUsageResponse) updates) =>
      super.copyWith(
              (message) => updates(message as IncrementQuotaUsageResponse))
          as IncrementQuotaUsageResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static IncrementQuotaUsageResponse create() =>
      IncrementQuotaUsageResponse._();
  @$core.override
  IncrementQuotaUsageResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static IncrementQuotaUsageResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<IncrementQuotaUsageResponse>(create);
  static IncrementQuotaUsageResponse? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get quotaId => $_getSZ(0);
  @$pb.TagNumber(1)
  set quotaId($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasQuotaId() => $_has(0);
  @$pb.TagNumber(1)
  void clearQuotaId() => $_clearField(1);

  @$pb.TagNumber(2)
  $fixnum.Int64 get used => $_getI64(1);
  @$pb.TagNumber(2)
  set used($fixnum.Int64 value) => $_setInt64(1, value);
  @$pb.TagNumber(2)
  $core.bool hasUsed() => $_has(1);
  @$pb.TagNumber(2)
  void clearUsed() => $_clearField(2);

  @$pb.TagNumber(3)
  $fixnum.Int64 get remaining => $_getI64(2);
  @$pb.TagNumber(3)
  set remaining($fixnum.Int64 value) => $_setInt64(2, value);
  @$pb.TagNumber(3)
  $core.bool hasRemaining() => $_has(2);
  @$pb.TagNumber(3)
  void clearRemaining() => $_clearField(3);

  @$pb.TagNumber(4)
  $core.double get usagePercent => $_getN(3);
  @$pb.TagNumber(4)
  set usagePercent($core.double value) => $_setDouble(3, value);
  @$pb.TagNumber(4)
  $core.bool hasUsagePercent() => $_has(3);
  @$pb.TagNumber(4)
  void clearUsagePercent() => $_clearField(4);

  @$pb.TagNumber(5)
  $core.bool get exceeded => $_getBF(4);
  @$pb.TagNumber(5)
  set exceeded($core.bool value) => $_setBool(4, value);
  @$pb.TagNumber(5)
  $core.bool hasExceeded() => $_has(4);
  @$pb.TagNumber(5)
  void clearExceeded() => $_clearField(5);

  @$pb.TagNumber(6)
  $core.bool get allowed => $_getBF(5);
  @$pb.TagNumber(6)
  set allowed($core.bool value) => $_setBool(5, value);
  @$pb.TagNumber(6)
  $core.bool hasAllowed() => $_has(5);
  @$pb.TagNumber(6)
  void clearAllowed() => $_clearField(6);
}

class ResetQuotaUsageRequest extends $pb.GeneratedMessage {
  factory ResetQuotaUsageRequest({
    $core.String? quotaId,
    $core.String? reason,
    $core.String? resetBy,
  }) {
    final result = create();
    if (quotaId != null) result.quotaId = quotaId;
    if (reason != null) result.reason = reason;
    if (resetBy != null) result.resetBy = resetBy;
    return result;
  }

  ResetQuotaUsageRequest._();

  factory ResetQuotaUsageRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory ResetQuotaUsageRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ResetQuotaUsageRequest',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.quota.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'quotaId')
    ..aOS(2, _omitFieldNames ? '' : 'reason')
    ..aOS(3, _omitFieldNames ? '' : 'resetBy')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ResetQuotaUsageRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ResetQuotaUsageRequest copyWith(
          void Function(ResetQuotaUsageRequest) updates) =>
      super.copyWith((message) => updates(message as ResetQuotaUsageRequest))
          as ResetQuotaUsageRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static ResetQuotaUsageRequest create() => ResetQuotaUsageRequest._();
  @$core.override
  ResetQuotaUsageRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static ResetQuotaUsageRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ResetQuotaUsageRequest>(create);
  static ResetQuotaUsageRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get quotaId => $_getSZ(0);
  @$pb.TagNumber(1)
  set quotaId($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasQuotaId() => $_has(0);
  @$pb.TagNumber(1)
  void clearQuotaId() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get reason => $_getSZ(1);
  @$pb.TagNumber(2)
  set reason($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasReason() => $_has(1);
  @$pb.TagNumber(2)
  void clearReason() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.String get resetBy => $_getSZ(2);
  @$pb.TagNumber(3)
  set resetBy($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasResetBy() => $_has(2);
  @$pb.TagNumber(3)
  void clearResetBy() => $_clearField(3);
}

class ResetQuotaUsageResponse extends $pb.GeneratedMessage {
  factory ResetQuotaUsageResponse({
    QuotaUsage? usage,
  }) {
    final result = create();
    if (usage != null) result.usage = usage;
    return result;
  }

  ResetQuotaUsageResponse._();

  factory ResetQuotaUsageResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory ResetQuotaUsageResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ResetQuotaUsageResponse',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.quota.v1'),
      createEmptyInstance: create)
    ..aOM<QuotaUsage>(1, _omitFieldNames ? '' : 'usage',
        subBuilder: QuotaUsage.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ResetQuotaUsageResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ResetQuotaUsageResponse copyWith(
          void Function(ResetQuotaUsageResponse) updates) =>
      super.copyWith((message) => updates(message as ResetQuotaUsageResponse))
          as ResetQuotaUsageResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static ResetQuotaUsageResponse create() => ResetQuotaUsageResponse._();
  @$core.override
  ResetQuotaUsageResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static ResetQuotaUsageResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ResetQuotaUsageResponse>(create);
  static ResetQuotaUsageResponse? _defaultInstance;

  @$pb.TagNumber(1)
  QuotaUsage get usage => $_getN(0);
  @$pb.TagNumber(1)
  set usage(QuotaUsage value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasUsage() => $_has(0);
  @$pb.TagNumber(1)
  void clearUsage() => $_clearField(1);
  @$pb.TagNumber(1)
  QuotaUsage ensureUsage() => $_ensure(0);
}

const $core.bool _omitFieldNames =
    $core.bool.fromEnvironment('protobuf.omit_field_names');
const $core.bool _omitMessageNames =
    $core.bool.fromEnvironment('protobuf.omit_message_names');
