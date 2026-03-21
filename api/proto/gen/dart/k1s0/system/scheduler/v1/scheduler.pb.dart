// This is a generated file - do not edit.
//
// Generated from k1s0/system/scheduler/v1/scheduler.proto.

// @dart = 3.3

// ignore_for_file: annotate_overrides, camel_case_types, comment_references
// ignore_for_file: constant_identifier_names
// ignore_for_file: curly_braces_in_flow_control_structures
// ignore_for_file: deprecated_member_use_from_same_package, library_prefixes
// ignore_for_file: non_constant_identifier_names, prefer_relative_imports

import 'dart:core' as $core;

import 'package:fixnum/fixnum.dart' as $fixnum;
import 'package:protobuf/protobuf.dart' as $pb;
import 'package:protobuf/well_known_types/google/protobuf/struct.pb.dart' as $1;

import '../../common/v1/types.pb.dart' as $2;

export 'package:protobuf/protobuf.dart' show GeneratedMessageGenericExtensions;

class Job extends $pb.GeneratedMessage {
  factory Job({
    $core.String? id,
    $core.String? name,
    $core.String? description,
    $core.String? cronExpression,
    $core.String? timezone,
    $core.String? targetType,
    $core.String? target,
    $1.Struct? payload,
    $core.String? status,
    $2.Timestamp? nextRunAt,
    $2.Timestamp? lastRunAt,
    $2.Timestamp? createdAt,
    $2.Timestamp? updatedAt,
  }) {
    final result = create();
    if (id != null) result.id = id;
    if (name != null) result.name = name;
    if (description != null) result.description = description;
    if (cronExpression != null) result.cronExpression = cronExpression;
    if (timezone != null) result.timezone = timezone;
    if (targetType != null) result.targetType = targetType;
    if (target != null) result.target = target;
    if (payload != null) result.payload = payload;
    if (status != null) result.status = status;
    if (nextRunAt != null) result.nextRunAt = nextRunAt;
    if (lastRunAt != null) result.lastRunAt = lastRunAt;
    if (createdAt != null) result.createdAt = createdAt;
    if (updatedAt != null) result.updatedAt = updatedAt;
    return result;
  }

  Job._();

  factory Job.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory Job.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'Job',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.scheduler.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'id')
    ..aOS(2, _omitFieldNames ? '' : 'name')
    ..aOS(3, _omitFieldNames ? '' : 'description')
    ..aOS(4, _omitFieldNames ? '' : 'cronExpression')
    ..aOS(5, _omitFieldNames ? '' : 'timezone')
    ..aOS(6, _omitFieldNames ? '' : 'targetType')
    ..aOS(7, _omitFieldNames ? '' : 'target')
    ..aOM<$1.Struct>(8, _omitFieldNames ? '' : 'payload',
        subBuilder: $1.Struct.create)
    ..aOS(9, _omitFieldNames ? '' : 'status')
    ..aOM<$2.Timestamp>(10, _omitFieldNames ? '' : 'nextRunAt',
        subBuilder: $2.Timestamp.create)
    ..aOM<$2.Timestamp>(11, _omitFieldNames ? '' : 'lastRunAt',
        subBuilder: $2.Timestamp.create)
    ..aOM<$2.Timestamp>(12, _omitFieldNames ? '' : 'createdAt',
        subBuilder: $2.Timestamp.create)
    ..aOM<$2.Timestamp>(13, _omitFieldNames ? '' : 'updatedAt',
        subBuilder: $2.Timestamp.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  Job clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  Job copyWith(void Function(Job) updates) =>
      super.copyWith((message) => updates(message as Job)) as Job;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static Job create() => Job._();
  @$core.override
  Job createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static Job getDefault() =>
      _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<Job>(create);
  static Job? _defaultInstance;

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
  $core.String get cronExpression => $_getSZ(3);
  @$pb.TagNumber(4)
  set cronExpression($core.String value) => $_setString(3, value);
  @$pb.TagNumber(4)
  $core.bool hasCronExpression() => $_has(3);
  @$pb.TagNumber(4)
  void clearCronExpression() => $_clearField(4);

  @$pb.TagNumber(5)
  $core.String get timezone => $_getSZ(4);
  @$pb.TagNumber(5)
  set timezone($core.String value) => $_setString(4, value);
  @$pb.TagNumber(5)
  $core.bool hasTimezone() => $_has(4);
  @$pb.TagNumber(5)
  void clearTimezone() => $_clearField(5);

  @$pb.TagNumber(6)
  $core.String get targetType => $_getSZ(5);
  @$pb.TagNumber(6)
  set targetType($core.String value) => $_setString(5, value);
  @$pb.TagNumber(6)
  $core.bool hasTargetType() => $_has(5);
  @$pb.TagNumber(6)
  void clearTargetType() => $_clearField(6);

  @$pb.TagNumber(7)
  $core.String get target => $_getSZ(6);
  @$pb.TagNumber(7)
  set target($core.String value) => $_setString(6, value);
  @$pb.TagNumber(7)
  $core.bool hasTarget() => $_has(6);
  @$pb.TagNumber(7)
  void clearTarget() => $_clearField(7);

  @$pb.TagNumber(8)
  $1.Struct get payload => $_getN(7);
  @$pb.TagNumber(8)
  set payload($1.Struct value) => $_setField(8, value);
  @$pb.TagNumber(8)
  $core.bool hasPayload() => $_has(7);
  @$pb.TagNumber(8)
  void clearPayload() => $_clearField(8);
  @$pb.TagNumber(8)
  $1.Struct ensurePayload() => $_ensure(7);

  @$pb.TagNumber(9)
  $core.String get status => $_getSZ(8);
  @$pb.TagNumber(9)
  set status($core.String value) => $_setString(8, value);
  @$pb.TagNumber(9)
  $core.bool hasStatus() => $_has(8);
  @$pb.TagNumber(9)
  void clearStatus() => $_clearField(9);

  @$pb.TagNumber(10)
  $2.Timestamp get nextRunAt => $_getN(9);
  @$pb.TagNumber(10)
  set nextRunAt($2.Timestamp value) => $_setField(10, value);
  @$pb.TagNumber(10)
  $core.bool hasNextRunAt() => $_has(9);
  @$pb.TagNumber(10)
  void clearNextRunAt() => $_clearField(10);
  @$pb.TagNumber(10)
  $2.Timestamp ensureNextRunAt() => $_ensure(9);

  @$pb.TagNumber(11)
  $2.Timestamp get lastRunAt => $_getN(10);
  @$pb.TagNumber(11)
  set lastRunAt($2.Timestamp value) => $_setField(11, value);
  @$pb.TagNumber(11)
  $core.bool hasLastRunAt() => $_has(10);
  @$pb.TagNumber(11)
  void clearLastRunAt() => $_clearField(11);
  @$pb.TagNumber(11)
  $2.Timestamp ensureLastRunAt() => $_ensure(10);

  @$pb.TagNumber(12)
  $2.Timestamp get createdAt => $_getN(11);
  @$pb.TagNumber(12)
  set createdAt($2.Timestamp value) => $_setField(12, value);
  @$pb.TagNumber(12)
  $core.bool hasCreatedAt() => $_has(11);
  @$pb.TagNumber(12)
  void clearCreatedAt() => $_clearField(12);
  @$pb.TagNumber(12)
  $2.Timestamp ensureCreatedAt() => $_ensure(11);

  @$pb.TagNumber(13)
  $2.Timestamp get updatedAt => $_getN(12);
  @$pb.TagNumber(13)
  set updatedAt($2.Timestamp value) => $_setField(13, value);
  @$pb.TagNumber(13)
  $core.bool hasUpdatedAt() => $_has(12);
  @$pb.TagNumber(13)
  void clearUpdatedAt() => $_clearField(13);
  @$pb.TagNumber(13)
  $2.Timestamp ensureUpdatedAt() => $_ensure(12);
}

class CreateJobRequest extends $pb.GeneratedMessage {
  factory CreateJobRequest({
    $core.String? name,
    $core.String? description,
    $core.String? cronExpression,
    $core.String? timezone,
    $core.String? targetType,
    $core.String? target,
    $1.Struct? payload,
  }) {
    final result = create();
    if (name != null) result.name = name;
    if (description != null) result.description = description;
    if (cronExpression != null) result.cronExpression = cronExpression;
    if (timezone != null) result.timezone = timezone;
    if (targetType != null) result.targetType = targetType;
    if (target != null) result.target = target;
    if (payload != null) result.payload = payload;
    return result;
  }

  CreateJobRequest._();

  factory CreateJobRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory CreateJobRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'CreateJobRequest',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.scheduler.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'name')
    ..aOS(2, _omitFieldNames ? '' : 'description')
    ..aOS(3, _omitFieldNames ? '' : 'cronExpression')
    ..aOS(4, _omitFieldNames ? '' : 'timezone')
    ..aOS(5, _omitFieldNames ? '' : 'targetType')
    ..aOS(6, _omitFieldNames ? '' : 'target')
    ..aOM<$1.Struct>(7, _omitFieldNames ? '' : 'payload',
        subBuilder: $1.Struct.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  CreateJobRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  CreateJobRequest copyWith(void Function(CreateJobRequest) updates) =>
      super.copyWith((message) => updates(message as CreateJobRequest))
          as CreateJobRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static CreateJobRequest create() => CreateJobRequest._();
  @$core.override
  CreateJobRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static CreateJobRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<CreateJobRequest>(create);
  static CreateJobRequest? _defaultInstance;

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
  $core.String get cronExpression => $_getSZ(2);
  @$pb.TagNumber(3)
  set cronExpression($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasCronExpression() => $_has(2);
  @$pb.TagNumber(3)
  void clearCronExpression() => $_clearField(3);

  @$pb.TagNumber(4)
  $core.String get timezone => $_getSZ(3);
  @$pb.TagNumber(4)
  set timezone($core.String value) => $_setString(3, value);
  @$pb.TagNumber(4)
  $core.bool hasTimezone() => $_has(3);
  @$pb.TagNumber(4)
  void clearTimezone() => $_clearField(4);

  @$pb.TagNumber(5)
  $core.String get targetType => $_getSZ(4);
  @$pb.TagNumber(5)
  set targetType($core.String value) => $_setString(4, value);
  @$pb.TagNumber(5)
  $core.bool hasTargetType() => $_has(4);
  @$pb.TagNumber(5)
  void clearTargetType() => $_clearField(5);

  @$pb.TagNumber(6)
  $core.String get target => $_getSZ(5);
  @$pb.TagNumber(6)
  set target($core.String value) => $_setString(5, value);
  @$pb.TagNumber(6)
  $core.bool hasTarget() => $_has(5);
  @$pb.TagNumber(6)
  void clearTarget() => $_clearField(6);

  @$pb.TagNumber(7)
  $1.Struct get payload => $_getN(6);
  @$pb.TagNumber(7)
  set payload($1.Struct value) => $_setField(7, value);
  @$pb.TagNumber(7)
  $core.bool hasPayload() => $_has(6);
  @$pb.TagNumber(7)
  void clearPayload() => $_clearField(7);
  @$pb.TagNumber(7)
  $1.Struct ensurePayload() => $_ensure(6);
}

class CreateJobResponse extends $pb.GeneratedMessage {
  factory CreateJobResponse({
    Job? job,
  }) {
    final result = create();
    if (job != null) result.job = job;
    return result;
  }

  CreateJobResponse._();

  factory CreateJobResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory CreateJobResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'CreateJobResponse',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.scheduler.v1'),
      createEmptyInstance: create)
    ..aOM<Job>(1, _omitFieldNames ? '' : 'job', subBuilder: Job.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  CreateJobResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  CreateJobResponse copyWith(void Function(CreateJobResponse) updates) =>
      super.copyWith((message) => updates(message as CreateJobResponse))
          as CreateJobResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static CreateJobResponse create() => CreateJobResponse._();
  @$core.override
  CreateJobResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static CreateJobResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<CreateJobResponse>(create);
  static CreateJobResponse? _defaultInstance;

  @$pb.TagNumber(1)
  Job get job => $_getN(0);
  @$pb.TagNumber(1)
  set job(Job value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasJob() => $_has(0);
  @$pb.TagNumber(1)
  void clearJob() => $_clearField(1);
  @$pb.TagNumber(1)
  Job ensureJob() => $_ensure(0);
}

class GetJobRequest extends $pb.GeneratedMessage {
  factory GetJobRequest({
    $core.String? jobId,
  }) {
    final result = create();
    if (jobId != null) result.jobId = jobId;
    return result;
  }

  GetJobRequest._();

  factory GetJobRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory GetJobRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'GetJobRequest',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.scheduler.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'jobId')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetJobRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetJobRequest copyWith(void Function(GetJobRequest) updates) =>
      super.copyWith((message) => updates(message as GetJobRequest))
          as GetJobRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static GetJobRequest create() => GetJobRequest._();
  @$core.override
  GetJobRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static GetJobRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<GetJobRequest>(create);
  static GetJobRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get jobId => $_getSZ(0);
  @$pb.TagNumber(1)
  set jobId($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasJobId() => $_has(0);
  @$pb.TagNumber(1)
  void clearJobId() => $_clearField(1);
}

class GetJobResponse extends $pb.GeneratedMessage {
  factory GetJobResponse({
    Job? job,
  }) {
    final result = create();
    if (job != null) result.job = job;
    return result;
  }

  GetJobResponse._();

  factory GetJobResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory GetJobResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'GetJobResponse',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.scheduler.v1'),
      createEmptyInstance: create)
    ..aOM<Job>(1, _omitFieldNames ? '' : 'job', subBuilder: Job.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetJobResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetJobResponse copyWith(void Function(GetJobResponse) updates) =>
      super.copyWith((message) => updates(message as GetJobResponse))
          as GetJobResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static GetJobResponse create() => GetJobResponse._();
  @$core.override
  GetJobResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static GetJobResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<GetJobResponse>(create);
  static GetJobResponse? _defaultInstance;

  @$pb.TagNumber(1)
  Job get job => $_getN(0);
  @$pb.TagNumber(1)
  set job(Job value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasJob() => $_has(0);
  @$pb.TagNumber(1)
  void clearJob() => $_clearField(1);
  @$pb.TagNumber(1)
  Job ensureJob() => $_ensure(0);
}

class ListJobsRequest extends $pb.GeneratedMessage {
  factory ListJobsRequest({
    $core.String? status,
    $2.Pagination? pagination,
  }) {
    final result = create();
    if (status != null) result.status = status;
    if (pagination != null) result.pagination = pagination;
    return result;
  }

  ListJobsRequest._();

  factory ListJobsRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory ListJobsRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ListJobsRequest',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.scheduler.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'status')
    ..aOM<$2.Pagination>(2, _omitFieldNames ? '' : 'pagination',
        subBuilder: $2.Pagination.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListJobsRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListJobsRequest copyWith(void Function(ListJobsRequest) updates) =>
      super.copyWith((message) => updates(message as ListJobsRequest))
          as ListJobsRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static ListJobsRequest create() => ListJobsRequest._();
  @$core.override
  ListJobsRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static ListJobsRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ListJobsRequest>(create);
  static ListJobsRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get status => $_getSZ(0);
  @$pb.TagNumber(1)
  set status($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasStatus() => $_has(0);
  @$pb.TagNumber(1)
  void clearStatus() => $_clearField(1);

  @$pb.TagNumber(2)
  $2.Pagination get pagination => $_getN(1);
  @$pb.TagNumber(2)
  set pagination($2.Pagination value) => $_setField(2, value);
  @$pb.TagNumber(2)
  $core.bool hasPagination() => $_has(1);
  @$pb.TagNumber(2)
  void clearPagination() => $_clearField(2);
  @$pb.TagNumber(2)
  $2.Pagination ensurePagination() => $_ensure(1);
}

class ListJobsResponse extends $pb.GeneratedMessage {
  factory ListJobsResponse({
    $core.Iterable<Job>? jobs,
    $2.PaginationResult? pagination,
  }) {
    final result = create();
    if (jobs != null) result.jobs.addAll(jobs);
    if (pagination != null) result.pagination = pagination;
    return result;
  }

  ListJobsResponse._();

  factory ListJobsResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory ListJobsResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ListJobsResponse',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.scheduler.v1'),
      createEmptyInstance: create)
    ..pPM<Job>(1, _omitFieldNames ? '' : 'jobs', subBuilder: Job.create)
    ..aOM<$2.PaginationResult>(2, _omitFieldNames ? '' : 'pagination',
        subBuilder: $2.PaginationResult.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListJobsResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListJobsResponse copyWith(void Function(ListJobsResponse) updates) =>
      super.copyWith((message) => updates(message as ListJobsResponse))
          as ListJobsResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static ListJobsResponse create() => ListJobsResponse._();
  @$core.override
  ListJobsResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static ListJobsResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ListJobsResponse>(create);
  static ListJobsResponse? _defaultInstance;

  @$pb.TagNumber(1)
  $pb.PbList<Job> get jobs => $_getList(0);

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

class UpdateJobRequest extends $pb.GeneratedMessage {
  factory UpdateJobRequest({
    $core.String? jobId,
    $core.String? name,
    $core.String? description,
    $core.String? cronExpression,
    $core.String? timezone,
    $core.String? targetType,
    $core.String? target,
    $1.Struct? payload,
  }) {
    final result = create();
    if (jobId != null) result.jobId = jobId;
    if (name != null) result.name = name;
    if (description != null) result.description = description;
    if (cronExpression != null) result.cronExpression = cronExpression;
    if (timezone != null) result.timezone = timezone;
    if (targetType != null) result.targetType = targetType;
    if (target != null) result.target = target;
    if (payload != null) result.payload = payload;
    return result;
  }

  UpdateJobRequest._();

  factory UpdateJobRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory UpdateJobRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'UpdateJobRequest',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.scheduler.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'jobId')
    ..aOS(2, _omitFieldNames ? '' : 'name')
    ..aOS(3, _omitFieldNames ? '' : 'description')
    ..aOS(4, _omitFieldNames ? '' : 'cronExpression')
    ..aOS(5, _omitFieldNames ? '' : 'timezone')
    ..aOS(6, _omitFieldNames ? '' : 'targetType')
    ..aOS(7, _omitFieldNames ? '' : 'target')
    ..aOM<$1.Struct>(8, _omitFieldNames ? '' : 'payload',
        subBuilder: $1.Struct.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  UpdateJobRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  UpdateJobRequest copyWith(void Function(UpdateJobRequest) updates) =>
      super.copyWith((message) => updates(message as UpdateJobRequest))
          as UpdateJobRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static UpdateJobRequest create() => UpdateJobRequest._();
  @$core.override
  UpdateJobRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static UpdateJobRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<UpdateJobRequest>(create);
  static UpdateJobRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get jobId => $_getSZ(0);
  @$pb.TagNumber(1)
  set jobId($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasJobId() => $_has(0);
  @$pb.TagNumber(1)
  void clearJobId() => $_clearField(1);

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
  $core.String get cronExpression => $_getSZ(3);
  @$pb.TagNumber(4)
  set cronExpression($core.String value) => $_setString(3, value);
  @$pb.TagNumber(4)
  $core.bool hasCronExpression() => $_has(3);
  @$pb.TagNumber(4)
  void clearCronExpression() => $_clearField(4);

  @$pb.TagNumber(5)
  $core.String get timezone => $_getSZ(4);
  @$pb.TagNumber(5)
  set timezone($core.String value) => $_setString(4, value);
  @$pb.TagNumber(5)
  $core.bool hasTimezone() => $_has(4);
  @$pb.TagNumber(5)
  void clearTimezone() => $_clearField(5);

  @$pb.TagNumber(6)
  $core.String get targetType => $_getSZ(5);
  @$pb.TagNumber(6)
  set targetType($core.String value) => $_setString(5, value);
  @$pb.TagNumber(6)
  $core.bool hasTargetType() => $_has(5);
  @$pb.TagNumber(6)
  void clearTargetType() => $_clearField(6);

  @$pb.TagNumber(7)
  $core.String get target => $_getSZ(6);
  @$pb.TagNumber(7)
  set target($core.String value) => $_setString(6, value);
  @$pb.TagNumber(7)
  $core.bool hasTarget() => $_has(6);
  @$pb.TagNumber(7)
  void clearTarget() => $_clearField(7);

  @$pb.TagNumber(8)
  $1.Struct get payload => $_getN(7);
  @$pb.TagNumber(8)
  set payload($1.Struct value) => $_setField(8, value);
  @$pb.TagNumber(8)
  $core.bool hasPayload() => $_has(7);
  @$pb.TagNumber(8)
  void clearPayload() => $_clearField(8);
  @$pb.TagNumber(8)
  $1.Struct ensurePayload() => $_ensure(7);
}

class UpdateJobResponse extends $pb.GeneratedMessage {
  factory UpdateJobResponse({
    Job? job,
  }) {
    final result = create();
    if (job != null) result.job = job;
    return result;
  }

  UpdateJobResponse._();

  factory UpdateJobResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory UpdateJobResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'UpdateJobResponse',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.scheduler.v1'),
      createEmptyInstance: create)
    ..aOM<Job>(1, _omitFieldNames ? '' : 'job', subBuilder: Job.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  UpdateJobResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  UpdateJobResponse copyWith(void Function(UpdateJobResponse) updates) =>
      super.copyWith((message) => updates(message as UpdateJobResponse))
          as UpdateJobResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static UpdateJobResponse create() => UpdateJobResponse._();
  @$core.override
  UpdateJobResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static UpdateJobResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<UpdateJobResponse>(create);
  static UpdateJobResponse? _defaultInstance;

  @$pb.TagNumber(1)
  Job get job => $_getN(0);
  @$pb.TagNumber(1)
  set job(Job value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasJob() => $_has(0);
  @$pb.TagNumber(1)
  void clearJob() => $_clearField(1);
  @$pb.TagNumber(1)
  Job ensureJob() => $_ensure(0);
}

class DeleteJobRequest extends $pb.GeneratedMessage {
  factory DeleteJobRequest({
    $core.String? jobId,
  }) {
    final result = create();
    if (jobId != null) result.jobId = jobId;
    return result;
  }

  DeleteJobRequest._();

  factory DeleteJobRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory DeleteJobRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'DeleteJobRequest',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.scheduler.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'jobId')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  DeleteJobRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  DeleteJobRequest copyWith(void Function(DeleteJobRequest) updates) =>
      super.copyWith((message) => updates(message as DeleteJobRequest))
          as DeleteJobRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static DeleteJobRequest create() => DeleteJobRequest._();
  @$core.override
  DeleteJobRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static DeleteJobRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<DeleteJobRequest>(create);
  static DeleteJobRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get jobId => $_getSZ(0);
  @$pb.TagNumber(1)
  set jobId($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasJobId() => $_has(0);
  @$pb.TagNumber(1)
  void clearJobId() => $_clearField(1);
}

class DeleteJobResponse extends $pb.GeneratedMessage {
  factory DeleteJobResponse({
    $core.bool? success,
    $core.String? message,
  }) {
    final result = create();
    if (success != null) result.success = success;
    if (message != null) result.message = message;
    return result;
  }

  DeleteJobResponse._();

  factory DeleteJobResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory DeleteJobResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'DeleteJobResponse',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.scheduler.v1'),
      createEmptyInstance: create)
    ..aOB(1, _omitFieldNames ? '' : 'success')
    ..aOS(2, _omitFieldNames ? '' : 'message')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  DeleteJobResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  DeleteJobResponse copyWith(void Function(DeleteJobResponse) updates) =>
      super.copyWith((message) => updates(message as DeleteJobResponse))
          as DeleteJobResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static DeleteJobResponse create() => DeleteJobResponse._();
  @$core.override
  DeleteJobResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static DeleteJobResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<DeleteJobResponse>(create);
  static DeleteJobResponse? _defaultInstance;

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

class PauseJobRequest extends $pb.GeneratedMessage {
  factory PauseJobRequest({
    $core.String? jobId,
  }) {
    final result = create();
    if (jobId != null) result.jobId = jobId;
    return result;
  }

  PauseJobRequest._();

  factory PauseJobRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory PauseJobRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'PauseJobRequest',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.scheduler.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'jobId')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  PauseJobRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  PauseJobRequest copyWith(void Function(PauseJobRequest) updates) =>
      super.copyWith((message) => updates(message as PauseJobRequest))
          as PauseJobRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static PauseJobRequest create() => PauseJobRequest._();
  @$core.override
  PauseJobRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static PauseJobRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<PauseJobRequest>(create);
  static PauseJobRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get jobId => $_getSZ(0);
  @$pb.TagNumber(1)
  set jobId($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasJobId() => $_has(0);
  @$pb.TagNumber(1)
  void clearJobId() => $_clearField(1);
}

class PauseJobResponse extends $pb.GeneratedMessage {
  factory PauseJobResponse({
    Job? job,
  }) {
    final result = create();
    if (job != null) result.job = job;
    return result;
  }

  PauseJobResponse._();

  factory PauseJobResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory PauseJobResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'PauseJobResponse',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.scheduler.v1'),
      createEmptyInstance: create)
    ..aOM<Job>(1, _omitFieldNames ? '' : 'job', subBuilder: Job.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  PauseJobResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  PauseJobResponse copyWith(void Function(PauseJobResponse) updates) =>
      super.copyWith((message) => updates(message as PauseJobResponse))
          as PauseJobResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static PauseJobResponse create() => PauseJobResponse._();
  @$core.override
  PauseJobResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static PauseJobResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<PauseJobResponse>(create);
  static PauseJobResponse? _defaultInstance;

  @$pb.TagNumber(1)
  Job get job => $_getN(0);
  @$pb.TagNumber(1)
  set job(Job value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasJob() => $_has(0);
  @$pb.TagNumber(1)
  void clearJob() => $_clearField(1);
  @$pb.TagNumber(1)
  Job ensureJob() => $_ensure(0);
}

class ResumeJobRequest extends $pb.GeneratedMessage {
  factory ResumeJobRequest({
    $core.String? jobId,
  }) {
    final result = create();
    if (jobId != null) result.jobId = jobId;
    return result;
  }

  ResumeJobRequest._();

  factory ResumeJobRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory ResumeJobRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ResumeJobRequest',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.scheduler.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'jobId')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ResumeJobRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ResumeJobRequest copyWith(void Function(ResumeJobRequest) updates) =>
      super.copyWith((message) => updates(message as ResumeJobRequest))
          as ResumeJobRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static ResumeJobRequest create() => ResumeJobRequest._();
  @$core.override
  ResumeJobRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static ResumeJobRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ResumeJobRequest>(create);
  static ResumeJobRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get jobId => $_getSZ(0);
  @$pb.TagNumber(1)
  set jobId($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasJobId() => $_has(0);
  @$pb.TagNumber(1)
  void clearJobId() => $_clearField(1);
}

class ResumeJobResponse extends $pb.GeneratedMessage {
  factory ResumeJobResponse({
    Job? job,
  }) {
    final result = create();
    if (job != null) result.job = job;
    return result;
  }

  ResumeJobResponse._();

  factory ResumeJobResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory ResumeJobResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ResumeJobResponse',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.scheduler.v1'),
      createEmptyInstance: create)
    ..aOM<Job>(1, _omitFieldNames ? '' : 'job', subBuilder: Job.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ResumeJobResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ResumeJobResponse copyWith(void Function(ResumeJobResponse) updates) =>
      super.copyWith((message) => updates(message as ResumeJobResponse))
          as ResumeJobResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static ResumeJobResponse create() => ResumeJobResponse._();
  @$core.override
  ResumeJobResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static ResumeJobResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ResumeJobResponse>(create);
  static ResumeJobResponse? _defaultInstance;

  @$pb.TagNumber(1)
  Job get job => $_getN(0);
  @$pb.TagNumber(1)
  set job(Job value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasJob() => $_has(0);
  @$pb.TagNumber(1)
  void clearJob() => $_clearField(1);
  @$pb.TagNumber(1)
  Job ensureJob() => $_ensure(0);
}

class TriggerJobRequest extends $pb.GeneratedMessage {
  factory TriggerJobRequest({
    $core.String? jobId,
  }) {
    final result = create();
    if (jobId != null) result.jobId = jobId;
    return result;
  }

  TriggerJobRequest._();

  factory TriggerJobRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory TriggerJobRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'TriggerJobRequest',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.scheduler.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'jobId')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  TriggerJobRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  TriggerJobRequest copyWith(void Function(TriggerJobRequest) updates) =>
      super.copyWith((message) => updates(message as TriggerJobRequest))
          as TriggerJobRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static TriggerJobRequest create() => TriggerJobRequest._();
  @$core.override
  TriggerJobRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static TriggerJobRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<TriggerJobRequest>(create);
  static TriggerJobRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get jobId => $_getSZ(0);
  @$pb.TagNumber(1)
  set jobId($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasJobId() => $_has(0);
  @$pb.TagNumber(1)
  void clearJobId() => $_clearField(1);
}

class TriggerJobResponse extends $pb.GeneratedMessage {
  factory TriggerJobResponse({
    $core.String? executionId,
    $core.String? jobId,
    $core.String? status,
    $2.Timestamp? triggeredAt,
  }) {
    final result = create();
    if (executionId != null) result.executionId = executionId;
    if (jobId != null) result.jobId = jobId;
    if (status != null) result.status = status;
    if (triggeredAt != null) result.triggeredAt = triggeredAt;
    return result;
  }

  TriggerJobResponse._();

  factory TriggerJobResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory TriggerJobResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'TriggerJobResponse',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.scheduler.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'executionId')
    ..aOS(2, _omitFieldNames ? '' : 'jobId')
    ..aOS(3, _omitFieldNames ? '' : 'status')
    ..aOM<$2.Timestamp>(4, _omitFieldNames ? '' : 'triggeredAt',
        subBuilder: $2.Timestamp.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  TriggerJobResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  TriggerJobResponse copyWith(void Function(TriggerJobResponse) updates) =>
      super.copyWith((message) => updates(message as TriggerJobResponse))
          as TriggerJobResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static TriggerJobResponse create() => TriggerJobResponse._();
  @$core.override
  TriggerJobResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static TriggerJobResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<TriggerJobResponse>(create);
  static TriggerJobResponse? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get executionId => $_getSZ(0);
  @$pb.TagNumber(1)
  set executionId($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasExecutionId() => $_has(0);
  @$pb.TagNumber(1)
  void clearExecutionId() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get jobId => $_getSZ(1);
  @$pb.TagNumber(2)
  set jobId($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasJobId() => $_has(1);
  @$pb.TagNumber(2)
  void clearJobId() => $_clearField(2);

  /// 実行状態（running / succeeded / failed）
  @$pb.TagNumber(3)
  $core.String get status => $_getSZ(2);
  @$pb.TagNumber(3)
  set status($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasStatus() => $_has(2);
  @$pb.TagNumber(3)
  void clearStatus() => $_clearField(3);

  @$pb.TagNumber(4)
  $2.Timestamp get triggeredAt => $_getN(3);
  @$pb.TagNumber(4)
  set triggeredAt($2.Timestamp value) => $_setField(4, value);
  @$pb.TagNumber(4)
  $core.bool hasTriggeredAt() => $_has(3);
  @$pb.TagNumber(4)
  void clearTriggeredAt() => $_clearField(4);
  @$pb.TagNumber(4)
  $2.Timestamp ensureTriggeredAt() => $_ensure(3);
}

class GetJobExecutionRequest extends $pb.GeneratedMessage {
  factory GetJobExecutionRequest({
    $core.String? executionId,
  }) {
    final result = create();
    if (executionId != null) result.executionId = executionId;
    return result;
  }

  GetJobExecutionRequest._();

  factory GetJobExecutionRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory GetJobExecutionRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'GetJobExecutionRequest',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.scheduler.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'executionId')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetJobExecutionRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetJobExecutionRequest copyWith(
          void Function(GetJobExecutionRequest) updates) =>
      super.copyWith((message) => updates(message as GetJobExecutionRequest))
          as GetJobExecutionRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static GetJobExecutionRequest create() => GetJobExecutionRequest._();
  @$core.override
  GetJobExecutionRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static GetJobExecutionRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<GetJobExecutionRequest>(create);
  static GetJobExecutionRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get executionId => $_getSZ(0);
  @$pb.TagNumber(1)
  set executionId($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasExecutionId() => $_has(0);
  @$pb.TagNumber(1)
  void clearExecutionId() => $_clearField(1);
}

class GetJobExecutionResponse extends $pb.GeneratedMessage {
  factory GetJobExecutionResponse({
    JobExecution? execution,
  }) {
    final result = create();
    if (execution != null) result.execution = execution;
    return result;
  }

  GetJobExecutionResponse._();

  factory GetJobExecutionResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory GetJobExecutionResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'GetJobExecutionResponse',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.scheduler.v1'),
      createEmptyInstance: create)
    ..aOM<JobExecution>(1, _omitFieldNames ? '' : 'execution',
        subBuilder: JobExecution.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetJobExecutionResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetJobExecutionResponse copyWith(
          void Function(GetJobExecutionResponse) updates) =>
      super.copyWith((message) => updates(message as GetJobExecutionResponse))
          as GetJobExecutionResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static GetJobExecutionResponse create() => GetJobExecutionResponse._();
  @$core.override
  GetJobExecutionResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static GetJobExecutionResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<GetJobExecutionResponse>(create);
  static GetJobExecutionResponse? _defaultInstance;

  @$pb.TagNumber(1)
  JobExecution get execution => $_getN(0);
  @$pb.TagNumber(1)
  set execution(JobExecution value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasExecution() => $_has(0);
  @$pb.TagNumber(1)
  void clearExecution() => $_clearField(1);
  @$pb.TagNumber(1)
  JobExecution ensureExecution() => $_ensure(0);
}

class ListExecutionsRequest extends $pb.GeneratedMessage {
  factory ListExecutionsRequest({
    $core.String? jobId,
    $2.Pagination? pagination,
    $core.String? status,
    $2.Timestamp? from,
    $2.Timestamp? to,
  }) {
    final result = create();
    if (jobId != null) result.jobId = jobId;
    if (pagination != null) result.pagination = pagination;
    if (status != null) result.status = status;
    if (from != null) result.from = from;
    if (to != null) result.to = to;
    return result;
  }

  ListExecutionsRequest._();

  factory ListExecutionsRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory ListExecutionsRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ListExecutionsRequest',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.scheduler.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'jobId')
    ..aOM<$2.Pagination>(2, _omitFieldNames ? '' : 'pagination',
        subBuilder: $2.Pagination.create)
    ..aOS(3, _omitFieldNames ? '' : 'status')
    ..aOM<$2.Timestamp>(4, _omitFieldNames ? '' : 'from',
        subBuilder: $2.Timestamp.create)
    ..aOM<$2.Timestamp>(5, _omitFieldNames ? '' : 'to',
        subBuilder: $2.Timestamp.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListExecutionsRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListExecutionsRequest copyWith(
          void Function(ListExecutionsRequest) updates) =>
      super.copyWith((message) => updates(message as ListExecutionsRequest))
          as ListExecutionsRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static ListExecutionsRequest create() => ListExecutionsRequest._();
  @$core.override
  ListExecutionsRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static ListExecutionsRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ListExecutionsRequest>(create);
  static ListExecutionsRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get jobId => $_getSZ(0);
  @$pb.TagNumber(1)
  set jobId($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasJobId() => $_has(0);
  @$pb.TagNumber(1)
  void clearJobId() => $_clearField(1);

  @$pb.TagNumber(2)
  $2.Pagination get pagination => $_getN(1);
  @$pb.TagNumber(2)
  set pagination($2.Pagination value) => $_setField(2, value);
  @$pb.TagNumber(2)
  $core.bool hasPagination() => $_has(1);
  @$pb.TagNumber(2)
  void clearPagination() => $_clearField(2);
  @$pb.TagNumber(2)
  $2.Pagination ensurePagination() => $_ensure(1);

  @$pb.TagNumber(3)
  $core.String get status => $_getSZ(2);
  @$pb.TagNumber(3)
  set status($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasStatus() => $_has(2);
  @$pb.TagNumber(3)
  void clearStatus() => $_clearField(3);

  @$pb.TagNumber(4)
  $2.Timestamp get from => $_getN(3);
  @$pb.TagNumber(4)
  set from($2.Timestamp value) => $_setField(4, value);
  @$pb.TagNumber(4)
  $core.bool hasFrom() => $_has(3);
  @$pb.TagNumber(4)
  void clearFrom() => $_clearField(4);
  @$pb.TagNumber(4)
  $2.Timestamp ensureFrom() => $_ensure(3);

  @$pb.TagNumber(5)
  $2.Timestamp get to => $_getN(4);
  @$pb.TagNumber(5)
  set to($2.Timestamp value) => $_setField(5, value);
  @$pb.TagNumber(5)
  $core.bool hasTo() => $_has(4);
  @$pb.TagNumber(5)
  void clearTo() => $_clearField(5);
  @$pb.TagNumber(5)
  $2.Timestamp ensureTo() => $_ensure(4);
}

class ListExecutionsResponse extends $pb.GeneratedMessage {
  factory ListExecutionsResponse({
    $core.Iterable<JobExecution>? executions,
    $2.PaginationResult? pagination,
  }) {
    final result = create();
    if (executions != null) result.executions.addAll(executions);
    if (pagination != null) result.pagination = pagination;
    return result;
  }

  ListExecutionsResponse._();

  factory ListExecutionsResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory ListExecutionsResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ListExecutionsResponse',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.scheduler.v1'),
      createEmptyInstance: create)
    ..pPM<JobExecution>(1, _omitFieldNames ? '' : 'executions',
        subBuilder: JobExecution.create)
    ..aOM<$2.PaginationResult>(2, _omitFieldNames ? '' : 'pagination',
        subBuilder: $2.PaginationResult.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListExecutionsResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListExecutionsResponse copyWith(
          void Function(ListExecutionsResponse) updates) =>
      super.copyWith((message) => updates(message as ListExecutionsResponse))
          as ListExecutionsResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static ListExecutionsResponse create() => ListExecutionsResponse._();
  @$core.override
  ListExecutionsResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static ListExecutionsResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ListExecutionsResponse>(create);
  static ListExecutionsResponse? _defaultInstance;

  @$pb.TagNumber(1)
  $pb.PbList<JobExecution> get executions => $_getList(0);

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

class JobExecution extends $pb.GeneratedMessage {
  factory JobExecution({
    $core.String? id,
    $core.String? jobId,
    $core.String? status,
    $core.String? triggeredBy,
    $2.Timestamp? startedAt,
    $2.Timestamp? finishedAt,
    $fixnum.Int64? durationMs,
    $core.String? errorMessage,
  }) {
    final result = create();
    if (id != null) result.id = id;
    if (jobId != null) result.jobId = jobId;
    if (status != null) result.status = status;
    if (triggeredBy != null) result.triggeredBy = triggeredBy;
    if (startedAt != null) result.startedAt = startedAt;
    if (finishedAt != null) result.finishedAt = finishedAt;
    if (durationMs != null) result.durationMs = durationMs;
    if (errorMessage != null) result.errorMessage = errorMessage;
    return result;
  }

  JobExecution._();

  factory JobExecution.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory JobExecution.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'JobExecution',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.system.scheduler.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'id')
    ..aOS(2, _omitFieldNames ? '' : 'jobId')
    ..aOS(3, _omitFieldNames ? '' : 'status')
    ..aOS(4, _omitFieldNames ? '' : 'triggeredBy')
    ..aOM<$2.Timestamp>(5, _omitFieldNames ? '' : 'startedAt',
        subBuilder: $2.Timestamp.create)
    ..aOM<$2.Timestamp>(6, _omitFieldNames ? '' : 'finishedAt',
        subBuilder: $2.Timestamp.create)
    ..a<$fixnum.Int64>(
        7, _omitFieldNames ? '' : 'durationMs', $pb.PbFieldType.OU6,
        defaultOrMaker: $fixnum.Int64.ZERO)
    ..aOS(8, _omitFieldNames ? '' : 'errorMessage')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  JobExecution clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  JobExecution copyWith(void Function(JobExecution) updates) =>
      super.copyWith((message) => updates(message as JobExecution))
          as JobExecution;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static JobExecution create() => JobExecution._();
  @$core.override
  JobExecution createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static JobExecution getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<JobExecution>(create);
  static JobExecution? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get id => $_getSZ(0);
  @$pb.TagNumber(1)
  set id($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasId() => $_has(0);
  @$pb.TagNumber(1)
  void clearId() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get jobId => $_getSZ(1);
  @$pb.TagNumber(2)
  set jobId($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasJobId() => $_has(1);
  @$pb.TagNumber(2)
  void clearJobId() => $_clearField(2);

  /// 実行状態（running / succeeded / failed）
  @$pb.TagNumber(3)
  $core.String get status => $_getSZ(2);
  @$pb.TagNumber(3)
  set status($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasStatus() => $_has(2);
  @$pb.TagNumber(3)
  void clearStatus() => $_clearField(3);

  /// 実行トリガー（scheduler / manual）
  @$pb.TagNumber(4)
  $core.String get triggeredBy => $_getSZ(3);
  @$pb.TagNumber(4)
  set triggeredBy($core.String value) => $_setString(3, value);
  @$pb.TagNumber(4)
  $core.bool hasTriggeredBy() => $_has(3);
  @$pb.TagNumber(4)
  void clearTriggeredBy() => $_clearField(4);

  @$pb.TagNumber(5)
  $2.Timestamp get startedAt => $_getN(4);
  @$pb.TagNumber(5)
  set startedAt($2.Timestamp value) => $_setField(5, value);
  @$pb.TagNumber(5)
  $core.bool hasStartedAt() => $_has(4);
  @$pb.TagNumber(5)
  void clearStartedAt() => $_clearField(5);
  @$pb.TagNumber(5)
  $2.Timestamp ensureStartedAt() => $_ensure(4);

  @$pb.TagNumber(6)
  $2.Timestamp get finishedAt => $_getN(5);
  @$pb.TagNumber(6)
  set finishedAt($2.Timestamp value) => $_setField(6, value);
  @$pb.TagNumber(6)
  $core.bool hasFinishedAt() => $_has(5);
  @$pb.TagNumber(6)
  void clearFinishedAt() => $_clearField(6);
  @$pb.TagNumber(6)
  $2.Timestamp ensureFinishedAt() => $_ensure(5);

  /// 実行時間（ミリ秒）
  @$pb.TagNumber(7)
  $fixnum.Int64 get durationMs => $_getI64(6);
  @$pb.TagNumber(7)
  set durationMs($fixnum.Int64 value) => $_setInt64(6, value);
  @$pb.TagNumber(7)
  $core.bool hasDurationMs() => $_has(6);
  @$pb.TagNumber(7)
  void clearDurationMs() => $_clearField(7);

  @$pb.TagNumber(8)
  $core.String get errorMessage => $_getSZ(7);
  @$pb.TagNumber(8)
  set errorMessage($core.String value) => $_setString(7, value);
  @$pb.TagNumber(8)
  $core.bool hasErrorMessage() => $_has(7);
  @$pb.TagNumber(8)
  void clearErrorMessage() => $_clearField(8);
}

const $core.bool _omitFieldNames =
    $core.bool.fromEnvironment('protobuf.omit_field_names');
const $core.bool _omitMessageNames =
    $core.bool.fromEnvironment('protobuf.omit_message_names');
