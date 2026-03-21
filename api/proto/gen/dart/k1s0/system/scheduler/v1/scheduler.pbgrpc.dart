// This is a generated file - do not edit.
//
// Generated from k1s0/system/scheduler/v1/scheduler.proto.

// @dart = 3.3

// ignore_for_file: annotate_overrides, camel_case_types, comment_references
// ignore_for_file: constant_identifier_names
// ignore_for_file: curly_braces_in_flow_control_structures
// ignore_for_file: deprecated_member_use_from_same_package, library_prefixes
// ignore_for_file: non_constant_identifier_names, prefer_relative_imports

import 'dart:async' as $async;
import 'dart:core' as $core;

import 'package:grpc/service_api.dart' as $grpc;
import 'package:protobuf/protobuf.dart' as $pb;

import 'scheduler.pb.dart' as $0;

export 'scheduler.pb.dart';

@$pb.GrpcServiceName('k1s0.system.scheduler.v1.SchedulerService')
class SchedulerServiceClient extends $grpc.Client {
  /// The hostname for this service.
  static const $core.String defaultHost = '';

  /// OAuth scopes needed for the client.
  static const $core.List<$core.String> oauthScopes = [
    '',
  ];

  SchedulerServiceClient(super.channel, {super.options, super.interceptors});

  $grpc.ResponseFuture<$0.CreateJobResponse> createJob(
    $0.CreateJobRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$createJob, request, options: options);
  }

  $grpc.ResponseFuture<$0.GetJobResponse> getJob(
    $0.GetJobRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$getJob, request, options: options);
  }

  $grpc.ResponseFuture<$0.ListJobsResponse> listJobs(
    $0.ListJobsRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$listJobs, request, options: options);
  }

  $grpc.ResponseFuture<$0.UpdateJobResponse> updateJob(
    $0.UpdateJobRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$updateJob, request, options: options);
  }

  $grpc.ResponseFuture<$0.DeleteJobResponse> deleteJob(
    $0.DeleteJobRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$deleteJob, request, options: options);
  }

  $grpc.ResponseFuture<$0.PauseJobResponse> pauseJob(
    $0.PauseJobRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$pauseJob, request, options: options);
  }

  $grpc.ResponseFuture<$0.ResumeJobResponse> resumeJob(
    $0.ResumeJobRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$resumeJob, request, options: options);
  }

  $grpc.ResponseFuture<$0.TriggerJobResponse> triggerJob(
    $0.TriggerJobRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$triggerJob, request, options: options);
  }

  $grpc.ResponseFuture<$0.GetJobExecutionResponse> getJobExecution(
    $0.GetJobExecutionRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$getJobExecution, request, options: options);
  }

  $grpc.ResponseFuture<$0.ListExecutionsResponse> listExecutions(
    $0.ListExecutionsRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$listExecutions, request, options: options);
  }

  // method descriptors

  static final _$createJob =
      $grpc.ClientMethod<$0.CreateJobRequest, $0.CreateJobResponse>(
          '/k1s0.system.scheduler.v1.SchedulerService/CreateJob',
          ($0.CreateJobRequest value) => value.writeToBuffer(),
          $0.CreateJobResponse.fromBuffer);
  static final _$getJob =
      $grpc.ClientMethod<$0.GetJobRequest, $0.GetJobResponse>(
          '/k1s0.system.scheduler.v1.SchedulerService/GetJob',
          ($0.GetJobRequest value) => value.writeToBuffer(),
          $0.GetJobResponse.fromBuffer);
  static final _$listJobs =
      $grpc.ClientMethod<$0.ListJobsRequest, $0.ListJobsResponse>(
          '/k1s0.system.scheduler.v1.SchedulerService/ListJobs',
          ($0.ListJobsRequest value) => value.writeToBuffer(),
          $0.ListJobsResponse.fromBuffer);
  static final _$updateJob =
      $grpc.ClientMethod<$0.UpdateJobRequest, $0.UpdateJobResponse>(
          '/k1s0.system.scheduler.v1.SchedulerService/UpdateJob',
          ($0.UpdateJobRequest value) => value.writeToBuffer(),
          $0.UpdateJobResponse.fromBuffer);
  static final _$deleteJob =
      $grpc.ClientMethod<$0.DeleteJobRequest, $0.DeleteJobResponse>(
          '/k1s0.system.scheduler.v1.SchedulerService/DeleteJob',
          ($0.DeleteJobRequest value) => value.writeToBuffer(),
          $0.DeleteJobResponse.fromBuffer);
  static final _$pauseJob =
      $grpc.ClientMethod<$0.PauseJobRequest, $0.PauseJobResponse>(
          '/k1s0.system.scheduler.v1.SchedulerService/PauseJob',
          ($0.PauseJobRequest value) => value.writeToBuffer(),
          $0.PauseJobResponse.fromBuffer);
  static final _$resumeJob =
      $grpc.ClientMethod<$0.ResumeJobRequest, $0.ResumeJobResponse>(
          '/k1s0.system.scheduler.v1.SchedulerService/ResumeJob',
          ($0.ResumeJobRequest value) => value.writeToBuffer(),
          $0.ResumeJobResponse.fromBuffer);
  static final _$triggerJob =
      $grpc.ClientMethod<$0.TriggerJobRequest, $0.TriggerJobResponse>(
          '/k1s0.system.scheduler.v1.SchedulerService/TriggerJob',
          ($0.TriggerJobRequest value) => value.writeToBuffer(),
          $0.TriggerJobResponse.fromBuffer);
  static final _$getJobExecution =
      $grpc.ClientMethod<$0.GetJobExecutionRequest, $0.GetJobExecutionResponse>(
          '/k1s0.system.scheduler.v1.SchedulerService/GetJobExecution',
          ($0.GetJobExecutionRequest value) => value.writeToBuffer(),
          $0.GetJobExecutionResponse.fromBuffer);
  static final _$listExecutions =
      $grpc.ClientMethod<$0.ListExecutionsRequest, $0.ListExecutionsResponse>(
          '/k1s0.system.scheduler.v1.SchedulerService/ListExecutions',
          ($0.ListExecutionsRequest value) => value.writeToBuffer(),
          $0.ListExecutionsResponse.fromBuffer);
}

@$pb.GrpcServiceName('k1s0.system.scheduler.v1.SchedulerService')
abstract class SchedulerServiceBase extends $grpc.Service {
  $core.String get $name => 'k1s0.system.scheduler.v1.SchedulerService';

  SchedulerServiceBase() {
    $addMethod($grpc.ServiceMethod<$0.CreateJobRequest, $0.CreateJobResponse>(
        'CreateJob',
        createJob_Pre,
        false,
        false,
        ($core.List<$core.int> value) => $0.CreateJobRequest.fromBuffer(value),
        ($0.CreateJobResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.GetJobRequest, $0.GetJobResponse>(
        'GetJob',
        getJob_Pre,
        false,
        false,
        ($core.List<$core.int> value) => $0.GetJobRequest.fromBuffer(value),
        ($0.GetJobResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.ListJobsRequest, $0.ListJobsResponse>(
        'ListJobs',
        listJobs_Pre,
        false,
        false,
        ($core.List<$core.int> value) => $0.ListJobsRequest.fromBuffer(value),
        ($0.ListJobsResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.UpdateJobRequest, $0.UpdateJobResponse>(
        'UpdateJob',
        updateJob_Pre,
        false,
        false,
        ($core.List<$core.int> value) => $0.UpdateJobRequest.fromBuffer(value),
        ($0.UpdateJobResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.DeleteJobRequest, $0.DeleteJobResponse>(
        'DeleteJob',
        deleteJob_Pre,
        false,
        false,
        ($core.List<$core.int> value) => $0.DeleteJobRequest.fromBuffer(value),
        ($0.DeleteJobResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.PauseJobRequest, $0.PauseJobResponse>(
        'PauseJob',
        pauseJob_Pre,
        false,
        false,
        ($core.List<$core.int> value) => $0.PauseJobRequest.fromBuffer(value),
        ($0.PauseJobResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.ResumeJobRequest, $0.ResumeJobResponse>(
        'ResumeJob',
        resumeJob_Pre,
        false,
        false,
        ($core.List<$core.int> value) => $0.ResumeJobRequest.fromBuffer(value),
        ($0.ResumeJobResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.TriggerJobRequest, $0.TriggerJobResponse>(
        'TriggerJob',
        triggerJob_Pre,
        false,
        false,
        ($core.List<$core.int> value) => $0.TriggerJobRequest.fromBuffer(value),
        ($0.TriggerJobResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.GetJobExecutionRequest,
            $0.GetJobExecutionResponse>(
        'GetJobExecution',
        getJobExecution_Pre,
        false,
        false,
        ($core.List<$core.int> value) =>
            $0.GetJobExecutionRequest.fromBuffer(value),
        ($0.GetJobExecutionResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.ListExecutionsRequest,
            $0.ListExecutionsResponse>(
        'ListExecutions',
        listExecutions_Pre,
        false,
        false,
        ($core.List<$core.int> value) =>
            $0.ListExecutionsRequest.fromBuffer(value),
        ($0.ListExecutionsResponse value) => value.writeToBuffer()));
  }

  $async.Future<$0.CreateJobResponse> createJob_Pre($grpc.ServiceCall $call,
      $async.Future<$0.CreateJobRequest> $request) async {
    return createJob($call, await $request);
  }

  $async.Future<$0.CreateJobResponse> createJob(
      $grpc.ServiceCall call, $0.CreateJobRequest request);

  $async.Future<$0.GetJobResponse> getJob_Pre(
      $grpc.ServiceCall $call, $async.Future<$0.GetJobRequest> $request) async {
    return getJob($call, await $request);
  }

  $async.Future<$0.GetJobResponse> getJob(
      $grpc.ServiceCall call, $0.GetJobRequest request);

  $async.Future<$0.ListJobsResponse> listJobs_Pre($grpc.ServiceCall $call,
      $async.Future<$0.ListJobsRequest> $request) async {
    return listJobs($call, await $request);
  }

  $async.Future<$0.ListJobsResponse> listJobs(
      $grpc.ServiceCall call, $0.ListJobsRequest request);

  $async.Future<$0.UpdateJobResponse> updateJob_Pre($grpc.ServiceCall $call,
      $async.Future<$0.UpdateJobRequest> $request) async {
    return updateJob($call, await $request);
  }

  $async.Future<$0.UpdateJobResponse> updateJob(
      $grpc.ServiceCall call, $0.UpdateJobRequest request);

  $async.Future<$0.DeleteJobResponse> deleteJob_Pre($grpc.ServiceCall $call,
      $async.Future<$0.DeleteJobRequest> $request) async {
    return deleteJob($call, await $request);
  }

  $async.Future<$0.DeleteJobResponse> deleteJob(
      $grpc.ServiceCall call, $0.DeleteJobRequest request);

  $async.Future<$0.PauseJobResponse> pauseJob_Pre($grpc.ServiceCall $call,
      $async.Future<$0.PauseJobRequest> $request) async {
    return pauseJob($call, await $request);
  }

  $async.Future<$0.PauseJobResponse> pauseJob(
      $grpc.ServiceCall call, $0.PauseJobRequest request);

  $async.Future<$0.ResumeJobResponse> resumeJob_Pre($grpc.ServiceCall $call,
      $async.Future<$0.ResumeJobRequest> $request) async {
    return resumeJob($call, await $request);
  }

  $async.Future<$0.ResumeJobResponse> resumeJob(
      $grpc.ServiceCall call, $0.ResumeJobRequest request);

  $async.Future<$0.TriggerJobResponse> triggerJob_Pre($grpc.ServiceCall $call,
      $async.Future<$0.TriggerJobRequest> $request) async {
    return triggerJob($call, await $request);
  }

  $async.Future<$0.TriggerJobResponse> triggerJob(
      $grpc.ServiceCall call, $0.TriggerJobRequest request);

  $async.Future<$0.GetJobExecutionResponse> getJobExecution_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.GetJobExecutionRequest> $request) async {
    return getJobExecution($call, await $request);
  }

  $async.Future<$0.GetJobExecutionResponse> getJobExecution(
      $grpc.ServiceCall call, $0.GetJobExecutionRequest request);

  $async.Future<$0.ListExecutionsResponse> listExecutions_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.ListExecutionsRequest> $request) async {
    return listExecutions($call, await $request);
  }

  $async.Future<$0.ListExecutionsResponse> listExecutions(
      $grpc.ServiceCall call, $0.ListExecutionsRequest request);
}
