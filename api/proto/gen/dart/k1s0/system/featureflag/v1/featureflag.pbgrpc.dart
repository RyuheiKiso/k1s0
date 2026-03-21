// This is a generated file - do not edit.
//
// Generated from k1s0/system/featureflag/v1/featureflag.proto.

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

import 'featureflag.pb.dart' as $0;

export 'featureflag.pb.dart';

@$pb.GrpcServiceName('k1s0.system.featureflag.v1.FeatureFlagService')
class FeatureFlagServiceClient extends $grpc.Client {
  /// The hostname for this service.
  static const $core.String defaultHost = '';

  /// OAuth scopes needed for the client.
  static const $core.List<$core.String> oauthScopes = [
    '',
  ];

  FeatureFlagServiceClient(super.channel, {super.options, super.interceptors});

  $grpc.ResponseFuture<$0.EvaluateFlagResponse> evaluateFlag(
    $0.EvaluateFlagRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$evaluateFlag, request, options: options);
  }

  $grpc.ResponseFuture<$0.GetFlagResponse> getFlag(
    $0.GetFlagRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$getFlag, request, options: options);
  }

  $grpc.ResponseFuture<$0.ListFlagsResponse> listFlags(
    $0.ListFlagsRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$listFlags, request, options: options);
  }

  $grpc.ResponseFuture<$0.CreateFlagResponse> createFlag(
    $0.CreateFlagRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$createFlag, request, options: options);
  }

  $grpc.ResponseFuture<$0.UpdateFlagResponse> updateFlag(
    $0.UpdateFlagRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$updateFlag, request, options: options);
  }

  $grpc.ResponseFuture<$0.DeleteFlagResponse> deleteFlag(
    $0.DeleteFlagRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$deleteFlag, request, options: options);
  }

  /// WatchFeatureFlag はフラグ変更の監視（Server-Side Streaming）。
  $grpc.ResponseStream<$0.WatchFeatureFlagResponse> watchFeatureFlag(
    $0.WatchFeatureFlagRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createStreamingCall(
        _$watchFeatureFlag, $async.Stream.fromIterable([request]),
        options: options);
  }

  // method descriptors

  static final _$evaluateFlag =
      $grpc.ClientMethod<$0.EvaluateFlagRequest, $0.EvaluateFlagResponse>(
          '/k1s0.system.featureflag.v1.FeatureFlagService/EvaluateFlag',
          ($0.EvaluateFlagRequest value) => value.writeToBuffer(),
          $0.EvaluateFlagResponse.fromBuffer);
  static final _$getFlag =
      $grpc.ClientMethod<$0.GetFlagRequest, $0.GetFlagResponse>(
          '/k1s0.system.featureflag.v1.FeatureFlagService/GetFlag',
          ($0.GetFlagRequest value) => value.writeToBuffer(),
          $0.GetFlagResponse.fromBuffer);
  static final _$listFlags =
      $grpc.ClientMethod<$0.ListFlagsRequest, $0.ListFlagsResponse>(
          '/k1s0.system.featureflag.v1.FeatureFlagService/ListFlags',
          ($0.ListFlagsRequest value) => value.writeToBuffer(),
          $0.ListFlagsResponse.fromBuffer);
  static final _$createFlag =
      $grpc.ClientMethod<$0.CreateFlagRequest, $0.CreateFlagResponse>(
          '/k1s0.system.featureflag.v1.FeatureFlagService/CreateFlag',
          ($0.CreateFlagRequest value) => value.writeToBuffer(),
          $0.CreateFlagResponse.fromBuffer);
  static final _$updateFlag =
      $grpc.ClientMethod<$0.UpdateFlagRequest, $0.UpdateFlagResponse>(
          '/k1s0.system.featureflag.v1.FeatureFlagService/UpdateFlag',
          ($0.UpdateFlagRequest value) => value.writeToBuffer(),
          $0.UpdateFlagResponse.fromBuffer);
  static final _$deleteFlag =
      $grpc.ClientMethod<$0.DeleteFlagRequest, $0.DeleteFlagResponse>(
          '/k1s0.system.featureflag.v1.FeatureFlagService/DeleteFlag',
          ($0.DeleteFlagRequest value) => value.writeToBuffer(),
          $0.DeleteFlagResponse.fromBuffer);
  static final _$watchFeatureFlag = $grpc.ClientMethod<
          $0.WatchFeatureFlagRequest, $0.WatchFeatureFlagResponse>(
      '/k1s0.system.featureflag.v1.FeatureFlagService/WatchFeatureFlag',
      ($0.WatchFeatureFlagRequest value) => value.writeToBuffer(),
      $0.WatchFeatureFlagResponse.fromBuffer);
}

@$pb.GrpcServiceName('k1s0.system.featureflag.v1.FeatureFlagService')
abstract class FeatureFlagServiceBase extends $grpc.Service {
  $core.String get $name => 'k1s0.system.featureflag.v1.FeatureFlagService';

  FeatureFlagServiceBase() {
    $addMethod(
        $grpc.ServiceMethod<$0.EvaluateFlagRequest, $0.EvaluateFlagResponse>(
            'EvaluateFlag',
            evaluateFlag_Pre,
            false,
            false,
            ($core.List<$core.int> value) =>
                $0.EvaluateFlagRequest.fromBuffer(value),
            ($0.EvaluateFlagResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.GetFlagRequest, $0.GetFlagResponse>(
        'GetFlag',
        getFlag_Pre,
        false,
        false,
        ($core.List<$core.int> value) => $0.GetFlagRequest.fromBuffer(value),
        ($0.GetFlagResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.ListFlagsRequest, $0.ListFlagsResponse>(
        'ListFlags',
        listFlags_Pre,
        false,
        false,
        ($core.List<$core.int> value) => $0.ListFlagsRequest.fromBuffer(value),
        ($0.ListFlagsResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.CreateFlagRequest, $0.CreateFlagResponse>(
        'CreateFlag',
        createFlag_Pre,
        false,
        false,
        ($core.List<$core.int> value) => $0.CreateFlagRequest.fromBuffer(value),
        ($0.CreateFlagResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.UpdateFlagRequest, $0.UpdateFlagResponse>(
        'UpdateFlag',
        updateFlag_Pre,
        false,
        false,
        ($core.List<$core.int> value) => $0.UpdateFlagRequest.fromBuffer(value),
        ($0.UpdateFlagResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.DeleteFlagRequest, $0.DeleteFlagResponse>(
        'DeleteFlag',
        deleteFlag_Pre,
        false,
        false,
        ($core.List<$core.int> value) => $0.DeleteFlagRequest.fromBuffer(value),
        ($0.DeleteFlagResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.WatchFeatureFlagRequest,
            $0.WatchFeatureFlagResponse>(
        'WatchFeatureFlag',
        watchFeatureFlag_Pre,
        false,
        true,
        ($core.List<$core.int> value) =>
            $0.WatchFeatureFlagRequest.fromBuffer(value),
        ($0.WatchFeatureFlagResponse value) => value.writeToBuffer()));
  }

  $async.Future<$0.EvaluateFlagResponse> evaluateFlag_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.EvaluateFlagRequest> $request) async {
    return evaluateFlag($call, await $request);
  }

  $async.Future<$0.EvaluateFlagResponse> evaluateFlag(
      $grpc.ServiceCall call, $0.EvaluateFlagRequest request);

  $async.Future<$0.GetFlagResponse> getFlag_Pre($grpc.ServiceCall $call,
      $async.Future<$0.GetFlagRequest> $request) async {
    return getFlag($call, await $request);
  }

  $async.Future<$0.GetFlagResponse> getFlag(
      $grpc.ServiceCall call, $0.GetFlagRequest request);

  $async.Future<$0.ListFlagsResponse> listFlags_Pre($grpc.ServiceCall $call,
      $async.Future<$0.ListFlagsRequest> $request) async {
    return listFlags($call, await $request);
  }

  $async.Future<$0.ListFlagsResponse> listFlags(
      $grpc.ServiceCall call, $0.ListFlagsRequest request);

  $async.Future<$0.CreateFlagResponse> createFlag_Pre($grpc.ServiceCall $call,
      $async.Future<$0.CreateFlagRequest> $request) async {
    return createFlag($call, await $request);
  }

  $async.Future<$0.CreateFlagResponse> createFlag(
      $grpc.ServiceCall call, $0.CreateFlagRequest request);

  $async.Future<$0.UpdateFlagResponse> updateFlag_Pre($grpc.ServiceCall $call,
      $async.Future<$0.UpdateFlagRequest> $request) async {
    return updateFlag($call, await $request);
  }

  $async.Future<$0.UpdateFlagResponse> updateFlag(
      $grpc.ServiceCall call, $0.UpdateFlagRequest request);

  $async.Future<$0.DeleteFlagResponse> deleteFlag_Pre($grpc.ServiceCall $call,
      $async.Future<$0.DeleteFlagRequest> $request) async {
    return deleteFlag($call, await $request);
  }

  $async.Future<$0.DeleteFlagResponse> deleteFlag(
      $grpc.ServiceCall call, $0.DeleteFlagRequest request);

  $async.Stream<$0.WatchFeatureFlagResponse> watchFeatureFlag_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.WatchFeatureFlagRequest> $request) async* {
    yield* watchFeatureFlag($call, await $request);
  }

  $async.Stream<$0.WatchFeatureFlagResponse> watchFeatureFlag(
      $grpc.ServiceCall call, $0.WatchFeatureFlagRequest request);
}
