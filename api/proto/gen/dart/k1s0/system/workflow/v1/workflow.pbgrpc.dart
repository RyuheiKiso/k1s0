// This is a generated file - do not edit.
//
// Generated from k1s0/system/workflow/v1/workflow.proto.

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

import 'workflow.pb.dart' as $0;

export 'workflow.pb.dart';

@$pb.GrpcServiceName('k1s0.system.workflow.v1.WorkflowService')
class WorkflowServiceClient extends $grpc.Client {
  /// The hostname for this service.
  static const $core.String defaultHost = '';

  /// OAuth scopes needed for the client.
  static const $core.List<$core.String> oauthScopes = [
    '',
  ];

  WorkflowServiceClient(super.channel, {super.options, super.interceptors});

  $grpc.ResponseFuture<$0.ListWorkflowsResponse> listWorkflows(
    $0.ListWorkflowsRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$listWorkflows, request, options: options);
  }

  $grpc.ResponseFuture<$0.CreateWorkflowResponse> createWorkflow(
    $0.CreateWorkflowRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$createWorkflow, request, options: options);
  }

  $grpc.ResponseFuture<$0.GetWorkflowResponse> getWorkflow(
    $0.GetWorkflowRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$getWorkflow, request, options: options);
  }

  $grpc.ResponseFuture<$0.UpdateWorkflowResponse> updateWorkflow(
    $0.UpdateWorkflowRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$updateWorkflow, request, options: options);
  }

  $grpc.ResponseFuture<$0.DeleteWorkflowResponse> deleteWorkflow(
    $0.DeleteWorkflowRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$deleteWorkflow, request, options: options);
  }

  $grpc.ResponseFuture<$0.StartInstanceResponse> startInstance(
    $0.StartInstanceRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$startInstance, request, options: options);
  }

  $grpc.ResponseFuture<$0.GetInstanceResponse> getInstance(
    $0.GetInstanceRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$getInstance, request, options: options);
  }

  $grpc.ResponseFuture<$0.ListInstancesResponse> listInstances(
    $0.ListInstancesRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$listInstances, request, options: options);
  }

  $grpc.ResponseFuture<$0.CancelInstanceResponse> cancelInstance(
    $0.CancelInstanceRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$cancelInstance, request, options: options);
  }

  $grpc.ResponseFuture<$0.ListTasksResponse> listTasks(
    $0.ListTasksRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$listTasks, request, options: options);
  }

  $grpc.ResponseFuture<$0.ReassignTaskResponse> reassignTask(
    $0.ReassignTaskRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$reassignTask, request, options: options);
  }

  $grpc.ResponseFuture<$0.ApproveTaskResponse> approveTask(
    $0.ApproveTaskRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$approveTask, request, options: options);
  }

  $grpc.ResponseFuture<$0.RejectTaskResponse> rejectTask(
    $0.RejectTaskRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$rejectTask, request, options: options);
  }

  // method descriptors

  static final _$listWorkflows =
      $grpc.ClientMethod<$0.ListWorkflowsRequest, $0.ListWorkflowsResponse>(
          '/k1s0.system.workflow.v1.WorkflowService/ListWorkflows',
          ($0.ListWorkflowsRequest value) => value.writeToBuffer(),
          $0.ListWorkflowsResponse.fromBuffer);
  static final _$createWorkflow =
      $grpc.ClientMethod<$0.CreateWorkflowRequest, $0.CreateWorkflowResponse>(
          '/k1s0.system.workflow.v1.WorkflowService/CreateWorkflow',
          ($0.CreateWorkflowRequest value) => value.writeToBuffer(),
          $0.CreateWorkflowResponse.fromBuffer);
  static final _$getWorkflow =
      $grpc.ClientMethod<$0.GetWorkflowRequest, $0.GetWorkflowResponse>(
          '/k1s0.system.workflow.v1.WorkflowService/GetWorkflow',
          ($0.GetWorkflowRequest value) => value.writeToBuffer(),
          $0.GetWorkflowResponse.fromBuffer);
  static final _$updateWorkflow =
      $grpc.ClientMethod<$0.UpdateWorkflowRequest, $0.UpdateWorkflowResponse>(
          '/k1s0.system.workflow.v1.WorkflowService/UpdateWorkflow',
          ($0.UpdateWorkflowRequest value) => value.writeToBuffer(),
          $0.UpdateWorkflowResponse.fromBuffer);
  static final _$deleteWorkflow =
      $grpc.ClientMethod<$0.DeleteWorkflowRequest, $0.DeleteWorkflowResponse>(
          '/k1s0.system.workflow.v1.WorkflowService/DeleteWorkflow',
          ($0.DeleteWorkflowRequest value) => value.writeToBuffer(),
          $0.DeleteWorkflowResponse.fromBuffer);
  static final _$startInstance =
      $grpc.ClientMethod<$0.StartInstanceRequest, $0.StartInstanceResponse>(
          '/k1s0.system.workflow.v1.WorkflowService/StartInstance',
          ($0.StartInstanceRequest value) => value.writeToBuffer(),
          $0.StartInstanceResponse.fromBuffer);
  static final _$getInstance =
      $grpc.ClientMethod<$0.GetInstanceRequest, $0.GetInstanceResponse>(
          '/k1s0.system.workflow.v1.WorkflowService/GetInstance',
          ($0.GetInstanceRequest value) => value.writeToBuffer(),
          $0.GetInstanceResponse.fromBuffer);
  static final _$listInstances =
      $grpc.ClientMethod<$0.ListInstancesRequest, $0.ListInstancesResponse>(
          '/k1s0.system.workflow.v1.WorkflowService/ListInstances',
          ($0.ListInstancesRequest value) => value.writeToBuffer(),
          $0.ListInstancesResponse.fromBuffer);
  static final _$cancelInstance =
      $grpc.ClientMethod<$0.CancelInstanceRequest, $0.CancelInstanceResponse>(
          '/k1s0.system.workflow.v1.WorkflowService/CancelInstance',
          ($0.CancelInstanceRequest value) => value.writeToBuffer(),
          $0.CancelInstanceResponse.fromBuffer);
  static final _$listTasks =
      $grpc.ClientMethod<$0.ListTasksRequest, $0.ListTasksResponse>(
          '/k1s0.system.workflow.v1.WorkflowService/ListTasks',
          ($0.ListTasksRequest value) => value.writeToBuffer(),
          $0.ListTasksResponse.fromBuffer);
  static final _$reassignTask =
      $grpc.ClientMethod<$0.ReassignTaskRequest, $0.ReassignTaskResponse>(
          '/k1s0.system.workflow.v1.WorkflowService/ReassignTask',
          ($0.ReassignTaskRequest value) => value.writeToBuffer(),
          $0.ReassignTaskResponse.fromBuffer);
  static final _$approveTask =
      $grpc.ClientMethod<$0.ApproveTaskRequest, $0.ApproveTaskResponse>(
          '/k1s0.system.workflow.v1.WorkflowService/ApproveTask',
          ($0.ApproveTaskRequest value) => value.writeToBuffer(),
          $0.ApproveTaskResponse.fromBuffer);
  static final _$rejectTask =
      $grpc.ClientMethod<$0.RejectTaskRequest, $0.RejectTaskResponse>(
          '/k1s0.system.workflow.v1.WorkflowService/RejectTask',
          ($0.RejectTaskRequest value) => value.writeToBuffer(),
          $0.RejectTaskResponse.fromBuffer);
}

@$pb.GrpcServiceName('k1s0.system.workflow.v1.WorkflowService')
abstract class WorkflowServiceBase extends $grpc.Service {
  $core.String get $name => 'k1s0.system.workflow.v1.WorkflowService';

  WorkflowServiceBase() {
    $addMethod(
        $grpc.ServiceMethod<$0.ListWorkflowsRequest, $0.ListWorkflowsResponse>(
            'ListWorkflows',
            listWorkflows_Pre,
            false,
            false,
            ($core.List<$core.int> value) =>
                $0.ListWorkflowsRequest.fromBuffer(value),
            ($0.ListWorkflowsResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.CreateWorkflowRequest,
            $0.CreateWorkflowResponse>(
        'CreateWorkflow',
        createWorkflow_Pre,
        false,
        false,
        ($core.List<$core.int> value) =>
            $0.CreateWorkflowRequest.fromBuffer(value),
        ($0.CreateWorkflowResponse value) => value.writeToBuffer()));
    $addMethod(
        $grpc.ServiceMethod<$0.GetWorkflowRequest, $0.GetWorkflowResponse>(
            'GetWorkflow',
            getWorkflow_Pre,
            false,
            false,
            ($core.List<$core.int> value) =>
                $0.GetWorkflowRequest.fromBuffer(value),
            ($0.GetWorkflowResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.UpdateWorkflowRequest,
            $0.UpdateWorkflowResponse>(
        'UpdateWorkflow',
        updateWorkflow_Pre,
        false,
        false,
        ($core.List<$core.int> value) =>
            $0.UpdateWorkflowRequest.fromBuffer(value),
        ($0.UpdateWorkflowResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.DeleteWorkflowRequest,
            $0.DeleteWorkflowResponse>(
        'DeleteWorkflow',
        deleteWorkflow_Pre,
        false,
        false,
        ($core.List<$core.int> value) =>
            $0.DeleteWorkflowRequest.fromBuffer(value),
        ($0.DeleteWorkflowResponse value) => value.writeToBuffer()));
    $addMethod(
        $grpc.ServiceMethod<$0.StartInstanceRequest, $0.StartInstanceResponse>(
            'StartInstance',
            startInstance_Pre,
            false,
            false,
            ($core.List<$core.int> value) =>
                $0.StartInstanceRequest.fromBuffer(value),
            ($0.StartInstanceResponse value) => value.writeToBuffer()));
    $addMethod(
        $grpc.ServiceMethod<$0.GetInstanceRequest, $0.GetInstanceResponse>(
            'GetInstance',
            getInstance_Pre,
            false,
            false,
            ($core.List<$core.int> value) =>
                $0.GetInstanceRequest.fromBuffer(value),
            ($0.GetInstanceResponse value) => value.writeToBuffer()));
    $addMethod(
        $grpc.ServiceMethod<$0.ListInstancesRequest, $0.ListInstancesResponse>(
            'ListInstances',
            listInstances_Pre,
            false,
            false,
            ($core.List<$core.int> value) =>
                $0.ListInstancesRequest.fromBuffer(value),
            ($0.ListInstancesResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.CancelInstanceRequest,
            $0.CancelInstanceResponse>(
        'CancelInstance',
        cancelInstance_Pre,
        false,
        false,
        ($core.List<$core.int> value) =>
            $0.CancelInstanceRequest.fromBuffer(value),
        ($0.CancelInstanceResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.ListTasksRequest, $0.ListTasksResponse>(
        'ListTasks',
        listTasks_Pre,
        false,
        false,
        ($core.List<$core.int> value) => $0.ListTasksRequest.fromBuffer(value),
        ($0.ListTasksResponse value) => value.writeToBuffer()));
    $addMethod(
        $grpc.ServiceMethod<$0.ReassignTaskRequest, $0.ReassignTaskResponse>(
            'ReassignTask',
            reassignTask_Pre,
            false,
            false,
            ($core.List<$core.int> value) =>
                $0.ReassignTaskRequest.fromBuffer(value),
            ($0.ReassignTaskResponse value) => value.writeToBuffer()));
    $addMethod(
        $grpc.ServiceMethod<$0.ApproveTaskRequest, $0.ApproveTaskResponse>(
            'ApproveTask',
            approveTask_Pre,
            false,
            false,
            ($core.List<$core.int> value) =>
                $0.ApproveTaskRequest.fromBuffer(value),
            ($0.ApproveTaskResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.RejectTaskRequest, $0.RejectTaskResponse>(
        'RejectTask',
        rejectTask_Pre,
        false,
        false,
        ($core.List<$core.int> value) => $0.RejectTaskRequest.fromBuffer(value),
        ($0.RejectTaskResponse value) => value.writeToBuffer()));
  }

  $async.Future<$0.ListWorkflowsResponse> listWorkflows_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.ListWorkflowsRequest> $request) async {
    return listWorkflows($call, await $request);
  }

  $async.Future<$0.ListWorkflowsResponse> listWorkflows(
      $grpc.ServiceCall call, $0.ListWorkflowsRequest request);

  $async.Future<$0.CreateWorkflowResponse> createWorkflow_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.CreateWorkflowRequest> $request) async {
    return createWorkflow($call, await $request);
  }

  $async.Future<$0.CreateWorkflowResponse> createWorkflow(
      $grpc.ServiceCall call, $0.CreateWorkflowRequest request);

  $async.Future<$0.GetWorkflowResponse> getWorkflow_Pre($grpc.ServiceCall $call,
      $async.Future<$0.GetWorkflowRequest> $request) async {
    return getWorkflow($call, await $request);
  }

  $async.Future<$0.GetWorkflowResponse> getWorkflow(
      $grpc.ServiceCall call, $0.GetWorkflowRequest request);

  $async.Future<$0.UpdateWorkflowResponse> updateWorkflow_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.UpdateWorkflowRequest> $request) async {
    return updateWorkflow($call, await $request);
  }

  $async.Future<$0.UpdateWorkflowResponse> updateWorkflow(
      $grpc.ServiceCall call, $0.UpdateWorkflowRequest request);

  $async.Future<$0.DeleteWorkflowResponse> deleteWorkflow_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.DeleteWorkflowRequest> $request) async {
    return deleteWorkflow($call, await $request);
  }

  $async.Future<$0.DeleteWorkflowResponse> deleteWorkflow(
      $grpc.ServiceCall call, $0.DeleteWorkflowRequest request);

  $async.Future<$0.StartInstanceResponse> startInstance_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.StartInstanceRequest> $request) async {
    return startInstance($call, await $request);
  }

  $async.Future<$0.StartInstanceResponse> startInstance(
      $grpc.ServiceCall call, $0.StartInstanceRequest request);

  $async.Future<$0.GetInstanceResponse> getInstance_Pre($grpc.ServiceCall $call,
      $async.Future<$0.GetInstanceRequest> $request) async {
    return getInstance($call, await $request);
  }

  $async.Future<$0.GetInstanceResponse> getInstance(
      $grpc.ServiceCall call, $0.GetInstanceRequest request);

  $async.Future<$0.ListInstancesResponse> listInstances_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.ListInstancesRequest> $request) async {
    return listInstances($call, await $request);
  }

  $async.Future<$0.ListInstancesResponse> listInstances(
      $grpc.ServiceCall call, $0.ListInstancesRequest request);

  $async.Future<$0.CancelInstanceResponse> cancelInstance_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.CancelInstanceRequest> $request) async {
    return cancelInstance($call, await $request);
  }

  $async.Future<$0.CancelInstanceResponse> cancelInstance(
      $grpc.ServiceCall call, $0.CancelInstanceRequest request);

  $async.Future<$0.ListTasksResponse> listTasks_Pre($grpc.ServiceCall $call,
      $async.Future<$0.ListTasksRequest> $request) async {
    return listTasks($call, await $request);
  }

  $async.Future<$0.ListTasksResponse> listTasks(
      $grpc.ServiceCall call, $0.ListTasksRequest request);

  $async.Future<$0.ReassignTaskResponse> reassignTask_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.ReassignTaskRequest> $request) async {
    return reassignTask($call, await $request);
  }

  $async.Future<$0.ReassignTaskResponse> reassignTask(
      $grpc.ServiceCall call, $0.ReassignTaskRequest request);

  $async.Future<$0.ApproveTaskResponse> approveTask_Pre($grpc.ServiceCall $call,
      $async.Future<$0.ApproveTaskRequest> $request) async {
    return approveTask($call, await $request);
  }

  $async.Future<$0.ApproveTaskResponse> approveTask(
      $grpc.ServiceCall call, $0.ApproveTaskRequest request);

  $async.Future<$0.RejectTaskResponse> rejectTask_Pre($grpc.ServiceCall $call,
      $async.Future<$0.RejectTaskRequest> $request) async {
    return rejectTask($call, await $request);
  }

  $async.Future<$0.RejectTaskResponse> rejectTask(
      $grpc.ServiceCall call, $0.RejectTaskRequest request);
}
