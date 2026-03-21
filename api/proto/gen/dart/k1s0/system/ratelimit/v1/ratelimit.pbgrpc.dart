// This is a generated file - do not edit.
//
// Generated from k1s0/system/ratelimit/v1/ratelimit.proto.

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

import 'ratelimit.pb.dart' as $0;

export 'ratelimit.pb.dart';

/// RateLimitService は API レート制限サービス。
/// スライディングウィンドウ・トークンバケット等のアルゴリズムをサポートする。
@$pb.GrpcServiceName('k1s0.system.ratelimit.v1.RateLimitService')
class RateLimitServiceClient extends $grpc.Client {
  /// The hostname for this service.
  static const $core.String defaultHost = '';

  /// OAuth scopes needed for the client.
  static const $core.List<$core.String> oauthScopes = [
    '',
  ];

  RateLimitServiceClient(super.channel, {super.options, super.interceptors});

  /// CheckRateLimit はリクエストがレートリミットに引っかかるか確認する。
  $grpc.ResponseFuture<$0.CheckRateLimitResponse> checkRateLimit(
    $0.CheckRateLimitRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$checkRateLimit, request, options: options);
  }

  /// CreateRule は新しいレートリミットルールを作成する。
  $grpc.ResponseFuture<$0.CreateRuleResponse> createRule(
    $0.CreateRuleRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$createRule, request, options: options);
  }

  /// GetRule はルールIDでルール情報を取得する。
  $grpc.ResponseFuture<$0.GetRuleResponse> getRule(
    $0.GetRuleRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$getRule, request, options: options);
  }

  /// UpdateRule はルールを更新する。
  $grpc.ResponseFuture<$0.UpdateRuleResponse> updateRule(
    $0.UpdateRuleRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$updateRule, request, options: options);
  }

  /// DeleteRule はルールを削除する。
  $grpc.ResponseFuture<$0.DeleteRuleResponse> deleteRule(
    $0.DeleteRuleRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$deleteRule, request, options: options);
  }

  /// ListRules はルール一覧を取得する。
  $grpc.ResponseFuture<$0.ListRulesResponse> listRules(
    $0.ListRulesRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$listRules, request, options: options);
  }

  /// GetUsage はレートリミットの使用状況を取得する。
  $grpc.ResponseFuture<$0.GetUsageResponse> getUsage(
    $0.GetUsageRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$getUsage, request, options: options);
  }

  /// ResetLimit はレートリミットの状態をリセットする。
  $grpc.ResponseFuture<$0.ResetLimitResponse> resetLimit(
    $0.ResetLimitRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$resetLimit, request, options: options);
  }

  // method descriptors

  static final _$checkRateLimit =
      $grpc.ClientMethod<$0.CheckRateLimitRequest, $0.CheckRateLimitResponse>(
          '/k1s0.system.ratelimit.v1.RateLimitService/CheckRateLimit',
          ($0.CheckRateLimitRequest value) => value.writeToBuffer(),
          $0.CheckRateLimitResponse.fromBuffer);
  static final _$createRule =
      $grpc.ClientMethod<$0.CreateRuleRequest, $0.CreateRuleResponse>(
          '/k1s0.system.ratelimit.v1.RateLimitService/CreateRule',
          ($0.CreateRuleRequest value) => value.writeToBuffer(),
          $0.CreateRuleResponse.fromBuffer);
  static final _$getRule =
      $grpc.ClientMethod<$0.GetRuleRequest, $0.GetRuleResponse>(
          '/k1s0.system.ratelimit.v1.RateLimitService/GetRule',
          ($0.GetRuleRequest value) => value.writeToBuffer(),
          $0.GetRuleResponse.fromBuffer);
  static final _$updateRule =
      $grpc.ClientMethod<$0.UpdateRuleRequest, $0.UpdateRuleResponse>(
          '/k1s0.system.ratelimit.v1.RateLimitService/UpdateRule',
          ($0.UpdateRuleRequest value) => value.writeToBuffer(),
          $0.UpdateRuleResponse.fromBuffer);
  static final _$deleteRule =
      $grpc.ClientMethod<$0.DeleteRuleRequest, $0.DeleteRuleResponse>(
          '/k1s0.system.ratelimit.v1.RateLimitService/DeleteRule',
          ($0.DeleteRuleRequest value) => value.writeToBuffer(),
          $0.DeleteRuleResponse.fromBuffer);
  static final _$listRules =
      $grpc.ClientMethod<$0.ListRulesRequest, $0.ListRulesResponse>(
          '/k1s0.system.ratelimit.v1.RateLimitService/ListRules',
          ($0.ListRulesRequest value) => value.writeToBuffer(),
          $0.ListRulesResponse.fromBuffer);
  static final _$getUsage =
      $grpc.ClientMethod<$0.GetUsageRequest, $0.GetUsageResponse>(
          '/k1s0.system.ratelimit.v1.RateLimitService/GetUsage',
          ($0.GetUsageRequest value) => value.writeToBuffer(),
          $0.GetUsageResponse.fromBuffer);
  static final _$resetLimit =
      $grpc.ClientMethod<$0.ResetLimitRequest, $0.ResetLimitResponse>(
          '/k1s0.system.ratelimit.v1.RateLimitService/ResetLimit',
          ($0.ResetLimitRequest value) => value.writeToBuffer(),
          $0.ResetLimitResponse.fromBuffer);
}

@$pb.GrpcServiceName('k1s0.system.ratelimit.v1.RateLimitService')
abstract class RateLimitServiceBase extends $grpc.Service {
  $core.String get $name => 'k1s0.system.ratelimit.v1.RateLimitService';

  RateLimitServiceBase() {
    $addMethod($grpc.ServiceMethod<$0.CheckRateLimitRequest,
            $0.CheckRateLimitResponse>(
        'CheckRateLimit',
        checkRateLimit_Pre,
        false,
        false,
        ($core.List<$core.int> value) =>
            $0.CheckRateLimitRequest.fromBuffer(value),
        ($0.CheckRateLimitResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.CreateRuleRequest, $0.CreateRuleResponse>(
        'CreateRule',
        createRule_Pre,
        false,
        false,
        ($core.List<$core.int> value) => $0.CreateRuleRequest.fromBuffer(value),
        ($0.CreateRuleResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.GetRuleRequest, $0.GetRuleResponse>(
        'GetRule',
        getRule_Pre,
        false,
        false,
        ($core.List<$core.int> value) => $0.GetRuleRequest.fromBuffer(value),
        ($0.GetRuleResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.UpdateRuleRequest, $0.UpdateRuleResponse>(
        'UpdateRule',
        updateRule_Pre,
        false,
        false,
        ($core.List<$core.int> value) => $0.UpdateRuleRequest.fromBuffer(value),
        ($0.UpdateRuleResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.DeleteRuleRequest, $0.DeleteRuleResponse>(
        'DeleteRule',
        deleteRule_Pre,
        false,
        false,
        ($core.List<$core.int> value) => $0.DeleteRuleRequest.fromBuffer(value),
        ($0.DeleteRuleResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.ListRulesRequest, $0.ListRulesResponse>(
        'ListRules',
        listRules_Pre,
        false,
        false,
        ($core.List<$core.int> value) => $0.ListRulesRequest.fromBuffer(value),
        ($0.ListRulesResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.GetUsageRequest, $0.GetUsageResponse>(
        'GetUsage',
        getUsage_Pre,
        false,
        false,
        ($core.List<$core.int> value) => $0.GetUsageRequest.fromBuffer(value),
        ($0.GetUsageResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.ResetLimitRequest, $0.ResetLimitResponse>(
        'ResetLimit',
        resetLimit_Pre,
        false,
        false,
        ($core.List<$core.int> value) => $0.ResetLimitRequest.fromBuffer(value),
        ($0.ResetLimitResponse value) => value.writeToBuffer()));
  }

  $async.Future<$0.CheckRateLimitResponse> checkRateLimit_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.CheckRateLimitRequest> $request) async {
    return checkRateLimit($call, await $request);
  }

  $async.Future<$0.CheckRateLimitResponse> checkRateLimit(
      $grpc.ServiceCall call, $0.CheckRateLimitRequest request);

  $async.Future<$0.CreateRuleResponse> createRule_Pre($grpc.ServiceCall $call,
      $async.Future<$0.CreateRuleRequest> $request) async {
    return createRule($call, await $request);
  }

  $async.Future<$0.CreateRuleResponse> createRule(
      $grpc.ServiceCall call, $0.CreateRuleRequest request);

  $async.Future<$0.GetRuleResponse> getRule_Pre($grpc.ServiceCall $call,
      $async.Future<$0.GetRuleRequest> $request) async {
    return getRule($call, await $request);
  }

  $async.Future<$0.GetRuleResponse> getRule(
      $grpc.ServiceCall call, $0.GetRuleRequest request);

  $async.Future<$0.UpdateRuleResponse> updateRule_Pre($grpc.ServiceCall $call,
      $async.Future<$0.UpdateRuleRequest> $request) async {
    return updateRule($call, await $request);
  }

  $async.Future<$0.UpdateRuleResponse> updateRule(
      $grpc.ServiceCall call, $0.UpdateRuleRequest request);

  $async.Future<$0.DeleteRuleResponse> deleteRule_Pre($grpc.ServiceCall $call,
      $async.Future<$0.DeleteRuleRequest> $request) async {
    return deleteRule($call, await $request);
  }

  $async.Future<$0.DeleteRuleResponse> deleteRule(
      $grpc.ServiceCall call, $0.DeleteRuleRequest request);

  $async.Future<$0.ListRulesResponse> listRules_Pre($grpc.ServiceCall $call,
      $async.Future<$0.ListRulesRequest> $request) async {
    return listRules($call, await $request);
  }

  $async.Future<$0.ListRulesResponse> listRules(
      $grpc.ServiceCall call, $0.ListRulesRequest request);

  $async.Future<$0.GetUsageResponse> getUsage_Pre($grpc.ServiceCall $call,
      $async.Future<$0.GetUsageRequest> $request) async {
    return getUsage($call, await $request);
  }

  $async.Future<$0.GetUsageResponse> getUsage(
      $grpc.ServiceCall call, $0.GetUsageRequest request);

  $async.Future<$0.ResetLimitResponse> resetLimit_Pre($grpc.ServiceCall $call,
      $async.Future<$0.ResetLimitRequest> $request) async {
    return resetLimit($call, await $request);
  }

  $async.Future<$0.ResetLimitResponse> resetLimit(
      $grpc.ServiceCall call, $0.ResetLimitRequest request);
}
