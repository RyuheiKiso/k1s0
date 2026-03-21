// This is a generated file - do not edit.
//
// Generated from k1s0/system/saga/v1/saga.proto.

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

import 'saga.pb.dart' as $0;

export 'saga.pb.dart';

/// SagaService は Saga オーケストレーション機能を提供する。
@$pb.GrpcServiceName('k1s0.system.saga.v1.SagaService')
class SagaServiceClient extends $grpc.Client {
  /// The hostname for this service.
  static const $core.String defaultHost = '';

  /// OAuth scopes needed for the client.
  static const $core.List<$core.String> oauthScopes = [
    '',
  ];

  SagaServiceClient(super.channel, {super.options, super.interceptors});

  /// Saga 開始（非同期実行）
  $grpc.ResponseFuture<$0.StartSagaResponse> startSaga(
    $0.StartSagaRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$startSaga, request, options: options);
  }

  /// Saga 詳細取得（ステップログ含む）
  $grpc.ResponseFuture<$0.GetSagaResponse> getSaga(
    $0.GetSagaRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$getSaga, request, options: options);
  }

  /// Saga 一覧取得
  $grpc.ResponseFuture<$0.ListSagasResponse> listSagas(
    $0.ListSagasRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$listSagas, request, options: options);
  }

  /// Saga キャンセル
  $grpc.ResponseFuture<$0.CancelSagaResponse> cancelSaga(
    $0.CancelSagaRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$cancelSaga, request, options: options);
  }

  /// Saga 補償実行
  $grpc.ResponseFuture<$0.CompensateSagaResponse> compensateSaga(
    $0.CompensateSagaRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$compensateSaga, request, options: options);
  }

  /// ワークフロー登録（YAML 文字列）
  $grpc.ResponseFuture<$0.RegisterWorkflowResponse> registerWorkflow(
    $0.RegisterWorkflowRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$registerWorkflow, request, options: options);
  }

  /// ワークフロー一覧取得
  $grpc.ResponseFuture<$0.ListWorkflowsResponse> listWorkflows(
    $0.ListWorkflowsRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$listWorkflows, request, options: options);
  }

  // method descriptors

  static final _$startSaga =
      $grpc.ClientMethod<$0.StartSagaRequest, $0.StartSagaResponse>(
          '/k1s0.system.saga.v1.SagaService/StartSaga',
          ($0.StartSagaRequest value) => value.writeToBuffer(),
          $0.StartSagaResponse.fromBuffer);
  static final _$getSaga =
      $grpc.ClientMethod<$0.GetSagaRequest, $0.GetSagaResponse>(
          '/k1s0.system.saga.v1.SagaService/GetSaga',
          ($0.GetSagaRequest value) => value.writeToBuffer(),
          $0.GetSagaResponse.fromBuffer);
  static final _$listSagas =
      $grpc.ClientMethod<$0.ListSagasRequest, $0.ListSagasResponse>(
          '/k1s0.system.saga.v1.SagaService/ListSagas',
          ($0.ListSagasRequest value) => value.writeToBuffer(),
          $0.ListSagasResponse.fromBuffer);
  static final _$cancelSaga =
      $grpc.ClientMethod<$0.CancelSagaRequest, $0.CancelSagaResponse>(
          '/k1s0.system.saga.v1.SagaService/CancelSaga',
          ($0.CancelSagaRequest value) => value.writeToBuffer(),
          $0.CancelSagaResponse.fromBuffer);
  static final _$compensateSaga =
      $grpc.ClientMethod<$0.CompensateSagaRequest, $0.CompensateSagaResponse>(
          '/k1s0.system.saga.v1.SagaService/CompensateSaga',
          ($0.CompensateSagaRequest value) => value.writeToBuffer(),
          $0.CompensateSagaResponse.fromBuffer);
  static final _$registerWorkflow = $grpc.ClientMethod<
          $0.RegisterWorkflowRequest, $0.RegisterWorkflowResponse>(
      '/k1s0.system.saga.v1.SagaService/RegisterWorkflow',
      ($0.RegisterWorkflowRequest value) => value.writeToBuffer(),
      $0.RegisterWorkflowResponse.fromBuffer);
  static final _$listWorkflows =
      $grpc.ClientMethod<$0.ListWorkflowsRequest, $0.ListWorkflowsResponse>(
          '/k1s0.system.saga.v1.SagaService/ListWorkflows',
          ($0.ListWorkflowsRequest value) => value.writeToBuffer(),
          $0.ListWorkflowsResponse.fromBuffer);
}

@$pb.GrpcServiceName('k1s0.system.saga.v1.SagaService')
abstract class SagaServiceBase extends $grpc.Service {
  $core.String get $name => 'k1s0.system.saga.v1.SagaService';

  SagaServiceBase() {
    $addMethod($grpc.ServiceMethod<$0.StartSagaRequest, $0.StartSagaResponse>(
        'StartSaga',
        startSaga_Pre,
        false,
        false,
        ($core.List<$core.int> value) => $0.StartSagaRequest.fromBuffer(value),
        ($0.StartSagaResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.GetSagaRequest, $0.GetSagaResponse>(
        'GetSaga',
        getSaga_Pre,
        false,
        false,
        ($core.List<$core.int> value) => $0.GetSagaRequest.fromBuffer(value),
        ($0.GetSagaResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.ListSagasRequest, $0.ListSagasResponse>(
        'ListSagas',
        listSagas_Pre,
        false,
        false,
        ($core.List<$core.int> value) => $0.ListSagasRequest.fromBuffer(value),
        ($0.ListSagasResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.CancelSagaRequest, $0.CancelSagaResponse>(
        'CancelSaga',
        cancelSaga_Pre,
        false,
        false,
        ($core.List<$core.int> value) => $0.CancelSagaRequest.fromBuffer(value),
        ($0.CancelSagaResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.CompensateSagaRequest,
            $0.CompensateSagaResponse>(
        'CompensateSaga',
        compensateSaga_Pre,
        false,
        false,
        ($core.List<$core.int> value) =>
            $0.CompensateSagaRequest.fromBuffer(value),
        ($0.CompensateSagaResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.RegisterWorkflowRequest,
            $0.RegisterWorkflowResponse>(
        'RegisterWorkflow',
        registerWorkflow_Pre,
        false,
        false,
        ($core.List<$core.int> value) =>
            $0.RegisterWorkflowRequest.fromBuffer(value),
        ($0.RegisterWorkflowResponse value) => value.writeToBuffer()));
    $addMethod(
        $grpc.ServiceMethod<$0.ListWorkflowsRequest, $0.ListWorkflowsResponse>(
            'ListWorkflows',
            listWorkflows_Pre,
            false,
            false,
            ($core.List<$core.int> value) =>
                $0.ListWorkflowsRequest.fromBuffer(value),
            ($0.ListWorkflowsResponse value) => value.writeToBuffer()));
  }

  $async.Future<$0.StartSagaResponse> startSaga_Pre($grpc.ServiceCall $call,
      $async.Future<$0.StartSagaRequest> $request) async {
    return startSaga($call, await $request);
  }

  $async.Future<$0.StartSagaResponse> startSaga(
      $grpc.ServiceCall call, $0.StartSagaRequest request);

  $async.Future<$0.GetSagaResponse> getSaga_Pre($grpc.ServiceCall $call,
      $async.Future<$0.GetSagaRequest> $request) async {
    return getSaga($call, await $request);
  }

  $async.Future<$0.GetSagaResponse> getSaga(
      $grpc.ServiceCall call, $0.GetSagaRequest request);

  $async.Future<$0.ListSagasResponse> listSagas_Pre($grpc.ServiceCall $call,
      $async.Future<$0.ListSagasRequest> $request) async {
    return listSagas($call, await $request);
  }

  $async.Future<$0.ListSagasResponse> listSagas(
      $grpc.ServiceCall call, $0.ListSagasRequest request);

  $async.Future<$0.CancelSagaResponse> cancelSaga_Pre($grpc.ServiceCall $call,
      $async.Future<$0.CancelSagaRequest> $request) async {
    return cancelSaga($call, await $request);
  }

  $async.Future<$0.CancelSagaResponse> cancelSaga(
      $grpc.ServiceCall call, $0.CancelSagaRequest request);

  $async.Future<$0.CompensateSagaResponse> compensateSaga_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.CompensateSagaRequest> $request) async {
    return compensateSaga($call, await $request);
  }

  $async.Future<$0.CompensateSagaResponse> compensateSaga(
      $grpc.ServiceCall call, $0.CompensateSagaRequest request);

  $async.Future<$0.RegisterWorkflowResponse> registerWorkflow_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.RegisterWorkflowRequest> $request) async {
    return registerWorkflow($call, await $request);
  }

  $async.Future<$0.RegisterWorkflowResponse> registerWorkflow(
      $grpc.ServiceCall call, $0.RegisterWorkflowRequest request);

  $async.Future<$0.ListWorkflowsResponse> listWorkflows_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.ListWorkflowsRequest> $request) async {
    return listWorkflows($call, await $request);
  }

  $async.Future<$0.ListWorkflowsResponse> listWorkflows(
      $grpc.ServiceCall call, $0.ListWorkflowsRequest request);
}
