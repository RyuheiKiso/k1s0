// This is a generated file - do not edit.
//
// Generated from k1s0/system/quota/v1/quota.proto.

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

import 'quota.pb.dart' as $0;

export 'quota.pb.dart';

/// QuotaService はクォータポリシー管理サービス。
@$pb.GrpcServiceName('k1s0.system.quota.v1.QuotaService')
class QuotaServiceClient extends $grpc.Client {
  /// The hostname for this service.
  static const $core.String defaultHost = '';

  /// OAuth scopes needed for the client.
  static const $core.List<$core.String> oauthScopes = [
    '',
  ];

  QuotaServiceClient(super.channel, {super.options, super.interceptors});

  $grpc.ResponseFuture<$0.CreateQuotaPolicyResponse> createQuotaPolicy(
    $0.CreateQuotaPolicyRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$createQuotaPolicy, request, options: options);
  }

  $grpc.ResponseFuture<$0.GetQuotaPolicyResponse> getQuotaPolicy(
    $0.GetQuotaPolicyRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$getQuotaPolicy, request, options: options);
  }

  $grpc.ResponseFuture<$0.ListQuotaPoliciesResponse> listQuotaPolicies(
    $0.ListQuotaPoliciesRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$listQuotaPolicies, request, options: options);
  }

  $grpc.ResponseFuture<$0.UpdateQuotaPolicyResponse> updateQuotaPolicy(
    $0.UpdateQuotaPolicyRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$updateQuotaPolicy, request, options: options);
  }

  $grpc.ResponseFuture<$0.DeleteQuotaPolicyResponse> deleteQuotaPolicy(
    $0.DeleteQuotaPolicyRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$deleteQuotaPolicy, request, options: options);
  }

  $grpc.ResponseFuture<$0.GetQuotaUsageResponse> getQuotaUsage(
    $0.GetQuotaUsageRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$getQuotaUsage, request, options: options);
  }

  $grpc.ResponseFuture<$0.CheckQuotaResponse> checkQuota(
    $0.CheckQuotaRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$checkQuota, request, options: options);
  }

  $grpc.ResponseFuture<$0.IncrementQuotaUsageResponse> incrementQuotaUsage(
    $0.IncrementQuotaUsageRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$incrementQuotaUsage, request, options: options);
  }

  $grpc.ResponseFuture<$0.ResetQuotaUsageResponse> resetQuotaUsage(
    $0.ResetQuotaUsageRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$resetQuotaUsage, request, options: options);
  }

  // method descriptors

  static final _$createQuotaPolicy = $grpc.ClientMethod<
          $0.CreateQuotaPolicyRequest, $0.CreateQuotaPolicyResponse>(
      '/k1s0.system.quota.v1.QuotaService/CreateQuotaPolicy',
      ($0.CreateQuotaPolicyRequest value) => value.writeToBuffer(),
      $0.CreateQuotaPolicyResponse.fromBuffer);
  static final _$getQuotaPolicy =
      $grpc.ClientMethod<$0.GetQuotaPolicyRequest, $0.GetQuotaPolicyResponse>(
          '/k1s0.system.quota.v1.QuotaService/GetQuotaPolicy',
          ($0.GetQuotaPolicyRequest value) => value.writeToBuffer(),
          $0.GetQuotaPolicyResponse.fromBuffer);
  static final _$listQuotaPolicies = $grpc.ClientMethod<
          $0.ListQuotaPoliciesRequest, $0.ListQuotaPoliciesResponse>(
      '/k1s0.system.quota.v1.QuotaService/ListQuotaPolicies',
      ($0.ListQuotaPoliciesRequest value) => value.writeToBuffer(),
      $0.ListQuotaPoliciesResponse.fromBuffer);
  static final _$updateQuotaPolicy = $grpc.ClientMethod<
          $0.UpdateQuotaPolicyRequest, $0.UpdateQuotaPolicyResponse>(
      '/k1s0.system.quota.v1.QuotaService/UpdateQuotaPolicy',
      ($0.UpdateQuotaPolicyRequest value) => value.writeToBuffer(),
      $0.UpdateQuotaPolicyResponse.fromBuffer);
  static final _$deleteQuotaPolicy = $grpc.ClientMethod<
          $0.DeleteQuotaPolicyRequest, $0.DeleteQuotaPolicyResponse>(
      '/k1s0.system.quota.v1.QuotaService/DeleteQuotaPolicy',
      ($0.DeleteQuotaPolicyRequest value) => value.writeToBuffer(),
      $0.DeleteQuotaPolicyResponse.fromBuffer);
  static final _$getQuotaUsage =
      $grpc.ClientMethod<$0.GetQuotaUsageRequest, $0.GetQuotaUsageResponse>(
          '/k1s0.system.quota.v1.QuotaService/GetQuotaUsage',
          ($0.GetQuotaUsageRequest value) => value.writeToBuffer(),
          $0.GetQuotaUsageResponse.fromBuffer);
  static final _$checkQuota =
      $grpc.ClientMethod<$0.CheckQuotaRequest, $0.CheckQuotaResponse>(
          '/k1s0.system.quota.v1.QuotaService/CheckQuota',
          ($0.CheckQuotaRequest value) => value.writeToBuffer(),
          $0.CheckQuotaResponse.fromBuffer);
  static final _$incrementQuotaUsage = $grpc.ClientMethod<
          $0.IncrementQuotaUsageRequest, $0.IncrementQuotaUsageResponse>(
      '/k1s0.system.quota.v1.QuotaService/IncrementQuotaUsage',
      ($0.IncrementQuotaUsageRequest value) => value.writeToBuffer(),
      $0.IncrementQuotaUsageResponse.fromBuffer);
  static final _$resetQuotaUsage =
      $grpc.ClientMethod<$0.ResetQuotaUsageRequest, $0.ResetQuotaUsageResponse>(
          '/k1s0.system.quota.v1.QuotaService/ResetQuotaUsage',
          ($0.ResetQuotaUsageRequest value) => value.writeToBuffer(),
          $0.ResetQuotaUsageResponse.fromBuffer);
}

@$pb.GrpcServiceName('k1s0.system.quota.v1.QuotaService')
abstract class QuotaServiceBase extends $grpc.Service {
  $core.String get $name => 'k1s0.system.quota.v1.QuotaService';

  QuotaServiceBase() {
    $addMethod($grpc.ServiceMethod<$0.CreateQuotaPolicyRequest,
            $0.CreateQuotaPolicyResponse>(
        'CreateQuotaPolicy',
        createQuotaPolicy_Pre,
        false,
        false,
        ($core.List<$core.int> value) =>
            $0.CreateQuotaPolicyRequest.fromBuffer(value),
        ($0.CreateQuotaPolicyResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.GetQuotaPolicyRequest,
            $0.GetQuotaPolicyResponse>(
        'GetQuotaPolicy',
        getQuotaPolicy_Pre,
        false,
        false,
        ($core.List<$core.int> value) =>
            $0.GetQuotaPolicyRequest.fromBuffer(value),
        ($0.GetQuotaPolicyResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.ListQuotaPoliciesRequest,
            $0.ListQuotaPoliciesResponse>(
        'ListQuotaPolicies',
        listQuotaPolicies_Pre,
        false,
        false,
        ($core.List<$core.int> value) =>
            $0.ListQuotaPoliciesRequest.fromBuffer(value),
        ($0.ListQuotaPoliciesResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.UpdateQuotaPolicyRequest,
            $0.UpdateQuotaPolicyResponse>(
        'UpdateQuotaPolicy',
        updateQuotaPolicy_Pre,
        false,
        false,
        ($core.List<$core.int> value) =>
            $0.UpdateQuotaPolicyRequest.fromBuffer(value),
        ($0.UpdateQuotaPolicyResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.DeleteQuotaPolicyRequest,
            $0.DeleteQuotaPolicyResponse>(
        'DeleteQuotaPolicy',
        deleteQuotaPolicy_Pre,
        false,
        false,
        ($core.List<$core.int> value) =>
            $0.DeleteQuotaPolicyRequest.fromBuffer(value),
        ($0.DeleteQuotaPolicyResponse value) => value.writeToBuffer()));
    $addMethod(
        $grpc.ServiceMethod<$0.GetQuotaUsageRequest, $0.GetQuotaUsageResponse>(
            'GetQuotaUsage',
            getQuotaUsage_Pre,
            false,
            false,
            ($core.List<$core.int> value) =>
                $0.GetQuotaUsageRequest.fromBuffer(value),
            ($0.GetQuotaUsageResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.CheckQuotaRequest, $0.CheckQuotaResponse>(
        'CheckQuota',
        checkQuota_Pre,
        false,
        false,
        ($core.List<$core.int> value) => $0.CheckQuotaRequest.fromBuffer(value),
        ($0.CheckQuotaResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.IncrementQuotaUsageRequest,
            $0.IncrementQuotaUsageResponse>(
        'IncrementQuotaUsage',
        incrementQuotaUsage_Pre,
        false,
        false,
        ($core.List<$core.int> value) =>
            $0.IncrementQuotaUsageRequest.fromBuffer(value),
        ($0.IncrementQuotaUsageResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.ResetQuotaUsageRequest,
            $0.ResetQuotaUsageResponse>(
        'ResetQuotaUsage',
        resetQuotaUsage_Pre,
        false,
        false,
        ($core.List<$core.int> value) =>
            $0.ResetQuotaUsageRequest.fromBuffer(value),
        ($0.ResetQuotaUsageResponse value) => value.writeToBuffer()));
  }

  $async.Future<$0.CreateQuotaPolicyResponse> createQuotaPolicy_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.CreateQuotaPolicyRequest> $request) async {
    return createQuotaPolicy($call, await $request);
  }

  $async.Future<$0.CreateQuotaPolicyResponse> createQuotaPolicy(
      $grpc.ServiceCall call, $0.CreateQuotaPolicyRequest request);

  $async.Future<$0.GetQuotaPolicyResponse> getQuotaPolicy_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.GetQuotaPolicyRequest> $request) async {
    return getQuotaPolicy($call, await $request);
  }

  $async.Future<$0.GetQuotaPolicyResponse> getQuotaPolicy(
      $grpc.ServiceCall call, $0.GetQuotaPolicyRequest request);

  $async.Future<$0.ListQuotaPoliciesResponse> listQuotaPolicies_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.ListQuotaPoliciesRequest> $request) async {
    return listQuotaPolicies($call, await $request);
  }

  $async.Future<$0.ListQuotaPoliciesResponse> listQuotaPolicies(
      $grpc.ServiceCall call, $0.ListQuotaPoliciesRequest request);

  $async.Future<$0.UpdateQuotaPolicyResponse> updateQuotaPolicy_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.UpdateQuotaPolicyRequest> $request) async {
    return updateQuotaPolicy($call, await $request);
  }

  $async.Future<$0.UpdateQuotaPolicyResponse> updateQuotaPolicy(
      $grpc.ServiceCall call, $0.UpdateQuotaPolicyRequest request);

  $async.Future<$0.DeleteQuotaPolicyResponse> deleteQuotaPolicy_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.DeleteQuotaPolicyRequest> $request) async {
    return deleteQuotaPolicy($call, await $request);
  }

  $async.Future<$0.DeleteQuotaPolicyResponse> deleteQuotaPolicy(
      $grpc.ServiceCall call, $0.DeleteQuotaPolicyRequest request);

  $async.Future<$0.GetQuotaUsageResponse> getQuotaUsage_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.GetQuotaUsageRequest> $request) async {
    return getQuotaUsage($call, await $request);
  }

  $async.Future<$0.GetQuotaUsageResponse> getQuotaUsage(
      $grpc.ServiceCall call, $0.GetQuotaUsageRequest request);

  $async.Future<$0.CheckQuotaResponse> checkQuota_Pre($grpc.ServiceCall $call,
      $async.Future<$0.CheckQuotaRequest> $request) async {
    return checkQuota($call, await $request);
  }

  $async.Future<$0.CheckQuotaResponse> checkQuota(
      $grpc.ServiceCall call, $0.CheckQuotaRequest request);

  $async.Future<$0.IncrementQuotaUsageResponse> incrementQuotaUsage_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.IncrementQuotaUsageRequest> $request) async {
    return incrementQuotaUsage($call, await $request);
  }

  $async.Future<$0.IncrementQuotaUsageResponse> incrementQuotaUsage(
      $grpc.ServiceCall call, $0.IncrementQuotaUsageRequest request);

  $async.Future<$0.ResetQuotaUsageResponse> resetQuotaUsage_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.ResetQuotaUsageRequest> $request) async {
    return resetQuotaUsage($call, await $request);
  }

  $async.Future<$0.ResetQuotaUsageResponse> resetQuotaUsage(
      $grpc.ServiceCall call, $0.ResetQuotaUsageRequest request);
}
