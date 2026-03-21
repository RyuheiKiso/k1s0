// This is a generated file - do not edit.
//
// Generated from k1s0/system/ai_agent/v1/ai_agent.proto.

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

import 'ai_agent.pb.dart' as $0;

export 'ai_agent.pb.dart';

/// AI エージェントサービス: エージェントの実行とステップ管理を行う
@$pb.GrpcServiceName('k1s0.system.aiagent.v1.AiAgentService')
class AiAgentServiceClient extends $grpc.Client {
  /// The hostname for this service.
  static const $core.String defaultHost = '';

  /// OAuth scopes needed for the client.
  static const $core.List<$core.String> oauthScopes = [
    '',
  ];

  AiAgentServiceClient(super.channel, {super.options, super.interceptors});

  /// エージェントを実行し、結果を返す
  $grpc.ResponseFuture<$0.ExecuteResponse> execute(
    $0.ExecuteRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$execute, request, options: options);
  }

  /// エージェントを実行し、ストリーミング形式でイベントを返す
  $grpc.ResponseStream<$0.ExecuteStreamResponse> executeStream(
    $0.ExecuteStreamRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createStreamingCall(
        _$executeStream, $async.Stream.fromIterable([request]),
        options: options);
  }

  /// 実行中のエージェントをキャンセルする
  $grpc.ResponseFuture<$0.CancelExecutionResponse> cancelExecution(
    $0.CancelExecutionRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$cancelExecution, request, options: options);
  }

  /// 実行ステップをレビュー（承認/却下）する
  $grpc.ResponseFuture<$0.ReviewStepResponse> reviewStep(
    $0.ReviewStepRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$reviewStep, request, options: options);
  }

  // method descriptors

  static final _$execute =
      $grpc.ClientMethod<$0.ExecuteRequest, $0.ExecuteResponse>(
          '/k1s0.system.aiagent.v1.AiAgentService/Execute',
          ($0.ExecuteRequest value) => value.writeToBuffer(),
          $0.ExecuteResponse.fromBuffer);
  static final _$executeStream =
      $grpc.ClientMethod<$0.ExecuteStreamRequest, $0.ExecuteStreamResponse>(
          '/k1s0.system.aiagent.v1.AiAgentService/ExecuteStream',
          ($0.ExecuteStreamRequest value) => value.writeToBuffer(),
          $0.ExecuteStreamResponse.fromBuffer);
  static final _$cancelExecution =
      $grpc.ClientMethod<$0.CancelExecutionRequest, $0.CancelExecutionResponse>(
          '/k1s0.system.aiagent.v1.AiAgentService/CancelExecution',
          ($0.CancelExecutionRequest value) => value.writeToBuffer(),
          $0.CancelExecutionResponse.fromBuffer);
  static final _$reviewStep =
      $grpc.ClientMethod<$0.ReviewStepRequest, $0.ReviewStepResponse>(
          '/k1s0.system.aiagent.v1.AiAgentService/ReviewStep',
          ($0.ReviewStepRequest value) => value.writeToBuffer(),
          $0.ReviewStepResponse.fromBuffer);
}

@$pb.GrpcServiceName('k1s0.system.aiagent.v1.AiAgentService')
abstract class AiAgentServiceBase extends $grpc.Service {
  $core.String get $name => 'k1s0.system.aiagent.v1.AiAgentService';

  AiAgentServiceBase() {
    $addMethod($grpc.ServiceMethod<$0.ExecuteRequest, $0.ExecuteResponse>(
        'Execute',
        execute_Pre,
        false,
        false,
        ($core.List<$core.int> value) => $0.ExecuteRequest.fromBuffer(value),
        ($0.ExecuteResponse value) => value.writeToBuffer()));
    $addMethod(
        $grpc.ServiceMethod<$0.ExecuteStreamRequest, $0.ExecuteStreamResponse>(
            'ExecuteStream',
            executeStream_Pre,
            false,
            true,
            ($core.List<$core.int> value) =>
                $0.ExecuteStreamRequest.fromBuffer(value),
            ($0.ExecuteStreamResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.CancelExecutionRequest,
            $0.CancelExecutionResponse>(
        'CancelExecution',
        cancelExecution_Pre,
        false,
        false,
        ($core.List<$core.int> value) =>
            $0.CancelExecutionRequest.fromBuffer(value),
        ($0.CancelExecutionResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.ReviewStepRequest, $0.ReviewStepResponse>(
        'ReviewStep',
        reviewStep_Pre,
        false,
        false,
        ($core.List<$core.int> value) => $0.ReviewStepRequest.fromBuffer(value),
        ($0.ReviewStepResponse value) => value.writeToBuffer()));
  }

  $async.Future<$0.ExecuteResponse> execute_Pre($grpc.ServiceCall $call,
      $async.Future<$0.ExecuteRequest> $request) async {
    return execute($call, await $request);
  }

  $async.Future<$0.ExecuteResponse> execute(
      $grpc.ServiceCall call, $0.ExecuteRequest request);

  $async.Stream<$0.ExecuteStreamResponse> executeStream_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.ExecuteStreamRequest> $request) async* {
    yield* executeStream($call, await $request);
  }

  $async.Stream<$0.ExecuteStreamResponse> executeStream(
      $grpc.ServiceCall call, $0.ExecuteStreamRequest request);

  $async.Future<$0.CancelExecutionResponse> cancelExecution_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.CancelExecutionRequest> $request) async {
    return cancelExecution($call, await $request);
  }

  $async.Future<$0.CancelExecutionResponse> cancelExecution(
      $grpc.ServiceCall call, $0.CancelExecutionRequest request);

  $async.Future<$0.ReviewStepResponse> reviewStep_Pre($grpc.ServiceCall $call,
      $async.Future<$0.ReviewStepRequest> $request) async {
    return reviewStep($call, await $request);
  }

  $async.Future<$0.ReviewStepResponse> reviewStep(
      $grpc.ServiceCall call, $0.ReviewStepRequest request);
}
