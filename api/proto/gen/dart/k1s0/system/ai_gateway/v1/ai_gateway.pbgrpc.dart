// This is a generated file - do not edit.
//
// Generated from k1s0/system/ai_gateway/v1/ai_gateway.proto.

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

import 'ai_gateway.pb.dart' as $0;

export 'ai_gateway.pb.dart';

/// AI ゲートウェイサービス: AI モデルへの補完・埋め込みリクエストを仲介する
@$pb.GrpcServiceName('k1s0.system.aigateway.v1.AiGatewayService')
class AiGatewayServiceClient extends $grpc.Client {
  /// The hostname for this service.
  static const $core.String defaultHost = '';

  /// OAuth scopes needed for the client.
  static const $core.List<$core.String> oauthScopes = [
    '',
  ];

  AiGatewayServiceClient(super.channel, {super.options, super.interceptors});

  /// テキスト補完リクエストを実行する
  $grpc.ResponseFuture<$0.CompleteResponse> complete(
    $0.CompleteRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$complete, request, options: options);
  }

  /// ストリーミング形式でテキスト補完を実行する
  $grpc.ResponseStream<$0.CompleteStreamResponse> completeStream(
    $0.CompleteStreamRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createStreamingCall(
        _$completeStream, $async.Stream.fromIterable([request]),
        options: options);
  }

  /// テキスト埋め込みベクトルを生成する
  $grpc.ResponseFuture<$0.EmbedResponse> embed(
    $0.EmbedRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$embed, request, options: options);
  }

  /// 利用可能な AI モデル一覧を取得する
  $grpc.ResponseFuture<$0.ListModelsResponse> listModels(
    $0.ListModelsRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$listModels, request, options: options);
  }

  /// テナントごとの使用量を取得する
  $grpc.ResponseFuture<$0.GetUsageResponse> getUsage(
    $0.GetUsageRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$getUsage, request, options: options);
  }

  // method descriptors

  static final _$complete =
      $grpc.ClientMethod<$0.CompleteRequest, $0.CompleteResponse>(
          '/k1s0.system.aigateway.v1.AiGatewayService/Complete',
          ($0.CompleteRequest value) => value.writeToBuffer(),
          $0.CompleteResponse.fromBuffer);
  static final _$completeStream =
      $grpc.ClientMethod<$0.CompleteStreamRequest, $0.CompleteStreamResponse>(
          '/k1s0.system.aigateway.v1.AiGatewayService/CompleteStream',
          ($0.CompleteStreamRequest value) => value.writeToBuffer(),
          $0.CompleteStreamResponse.fromBuffer);
  static final _$embed = $grpc.ClientMethod<$0.EmbedRequest, $0.EmbedResponse>(
      '/k1s0.system.aigateway.v1.AiGatewayService/Embed',
      ($0.EmbedRequest value) => value.writeToBuffer(),
      $0.EmbedResponse.fromBuffer);
  static final _$listModels =
      $grpc.ClientMethod<$0.ListModelsRequest, $0.ListModelsResponse>(
          '/k1s0.system.aigateway.v1.AiGatewayService/ListModels',
          ($0.ListModelsRequest value) => value.writeToBuffer(),
          $0.ListModelsResponse.fromBuffer);
  static final _$getUsage =
      $grpc.ClientMethod<$0.GetUsageRequest, $0.GetUsageResponse>(
          '/k1s0.system.aigateway.v1.AiGatewayService/GetUsage',
          ($0.GetUsageRequest value) => value.writeToBuffer(),
          $0.GetUsageResponse.fromBuffer);
}

@$pb.GrpcServiceName('k1s0.system.aigateway.v1.AiGatewayService')
abstract class AiGatewayServiceBase extends $grpc.Service {
  $core.String get $name => 'k1s0.system.aigateway.v1.AiGatewayService';

  AiGatewayServiceBase() {
    $addMethod($grpc.ServiceMethod<$0.CompleteRequest, $0.CompleteResponse>(
        'Complete',
        complete_Pre,
        false,
        false,
        ($core.List<$core.int> value) => $0.CompleteRequest.fromBuffer(value),
        ($0.CompleteResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.CompleteStreamRequest,
            $0.CompleteStreamResponse>(
        'CompleteStream',
        completeStream_Pre,
        false,
        true,
        ($core.List<$core.int> value) =>
            $0.CompleteStreamRequest.fromBuffer(value),
        ($0.CompleteStreamResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.EmbedRequest, $0.EmbedResponse>(
        'Embed',
        embed_Pre,
        false,
        false,
        ($core.List<$core.int> value) => $0.EmbedRequest.fromBuffer(value),
        ($0.EmbedResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.ListModelsRequest, $0.ListModelsResponse>(
        'ListModels',
        listModels_Pre,
        false,
        false,
        ($core.List<$core.int> value) => $0.ListModelsRequest.fromBuffer(value),
        ($0.ListModelsResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.GetUsageRequest, $0.GetUsageResponse>(
        'GetUsage',
        getUsage_Pre,
        false,
        false,
        ($core.List<$core.int> value) => $0.GetUsageRequest.fromBuffer(value),
        ($0.GetUsageResponse value) => value.writeToBuffer()));
  }

  $async.Future<$0.CompleteResponse> complete_Pre($grpc.ServiceCall $call,
      $async.Future<$0.CompleteRequest> $request) async {
    return complete($call, await $request);
  }

  $async.Future<$0.CompleteResponse> complete(
      $grpc.ServiceCall call, $0.CompleteRequest request);

  $async.Stream<$0.CompleteStreamResponse> completeStream_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.CompleteStreamRequest> $request) async* {
    yield* completeStream($call, await $request);
  }

  $async.Stream<$0.CompleteStreamResponse> completeStream(
      $grpc.ServiceCall call, $0.CompleteStreamRequest request);

  $async.Future<$0.EmbedResponse> embed_Pre(
      $grpc.ServiceCall $call, $async.Future<$0.EmbedRequest> $request) async {
    return embed($call, await $request);
  }

  $async.Future<$0.EmbedResponse> embed(
      $grpc.ServiceCall call, $0.EmbedRequest request);

  $async.Future<$0.ListModelsResponse> listModels_Pre($grpc.ServiceCall $call,
      $async.Future<$0.ListModelsRequest> $request) async {
    return listModels($call, await $request);
  }

  $async.Future<$0.ListModelsResponse> listModels(
      $grpc.ServiceCall call, $0.ListModelsRequest request);

  $async.Future<$0.GetUsageResponse> getUsage_Pre($grpc.ServiceCall $call,
      $async.Future<$0.GetUsageRequest> $request) async {
    return getUsage($call, await $request);
  }

  $async.Future<$0.GetUsageResponse> getUsage(
      $grpc.ServiceCall call, $0.GetUsageRequest request);
}
