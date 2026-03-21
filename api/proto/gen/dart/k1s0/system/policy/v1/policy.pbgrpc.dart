// This is a generated file - do not edit.
//
// Generated from k1s0/system/policy/v1/policy.proto.

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

import 'policy.pb.dart' as $0;

export 'policy.pb.dart';

@$pb.GrpcServiceName('k1s0.system.policy.v1.PolicyService')
class PolicyServiceClient extends $grpc.Client {
  /// The hostname for this service.
  static const $core.String defaultHost = '';

  /// OAuth scopes needed for the client.
  static const $core.List<$core.String> oauthScopes = [
    '',
  ];

  PolicyServiceClient(super.channel, {super.options, super.interceptors});

  $grpc.ResponseFuture<$0.EvaluatePolicyResponse> evaluatePolicy(
    $0.EvaluatePolicyRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$evaluatePolicy, request, options: options);
  }

  $grpc.ResponseFuture<$0.GetPolicyResponse> getPolicy(
    $0.GetPolicyRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$getPolicy, request, options: options);
  }

  $grpc.ResponseFuture<$0.ListPoliciesResponse> listPolicies(
    $0.ListPoliciesRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$listPolicies, request, options: options);
  }

  $grpc.ResponseFuture<$0.CreatePolicyResponse> createPolicy(
    $0.CreatePolicyRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$createPolicy, request, options: options);
  }

  $grpc.ResponseFuture<$0.UpdatePolicyResponse> updatePolicy(
    $0.UpdatePolicyRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$updatePolicy, request, options: options);
  }

  $grpc.ResponseFuture<$0.DeletePolicyResponse> deletePolicy(
    $0.DeletePolicyRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$deletePolicy, request, options: options);
  }

  $grpc.ResponseFuture<$0.CreateBundleResponse> createBundle(
    $0.CreateBundleRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$createBundle, request, options: options);
  }

  $grpc.ResponseFuture<$0.ListBundlesResponse> listBundles(
    $0.ListBundlesRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$listBundles, request, options: options);
  }

  $grpc.ResponseFuture<$0.GetBundleResponse> getBundle(
    $0.GetBundleRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$getBundle, request, options: options);
  }

  // method descriptors

  static final _$evaluatePolicy =
      $grpc.ClientMethod<$0.EvaluatePolicyRequest, $0.EvaluatePolicyResponse>(
          '/k1s0.system.policy.v1.PolicyService/EvaluatePolicy',
          ($0.EvaluatePolicyRequest value) => value.writeToBuffer(),
          $0.EvaluatePolicyResponse.fromBuffer);
  static final _$getPolicy =
      $grpc.ClientMethod<$0.GetPolicyRequest, $0.GetPolicyResponse>(
          '/k1s0.system.policy.v1.PolicyService/GetPolicy',
          ($0.GetPolicyRequest value) => value.writeToBuffer(),
          $0.GetPolicyResponse.fromBuffer);
  static final _$listPolicies =
      $grpc.ClientMethod<$0.ListPoliciesRequest, $0.ListPoliciesResponse>(
          '/k1s0.system.policy.v1.PolicyService/ListPolicies',
          ($0.ListPoliciesRequest value) => value.writeToBuffer(),
          $0.ListPoliciesResponse.fromBuffer);
  static final _$createPolicy =
      $grpc.ClientMethod<$0.CreatePolicyRequest, $0.CreatePolicyResponse>(
          '/k1s0.system.policy.v1.PolicyService/CreatePolicy',
          ($0.CreatePolicyRequest value) => value.writeToBuffer(),
          $0.CreatePolicyResponse.fromBuffer);
  static final _$updatePolicy =
      $grpc.ClientMethod<$0.UpdatePolicyRequest, $0.UpdatePolicyResponse>(
          '/k1s0.system.policy.v1.PolicyService/UpdatePolicy',
          ($0.UpdatePolicyRequest value) => value.writeToBuffer(),
          $0.UpdatePolicyResponse.fromBuffer);
  static final _$deletePolicy =
      $grpc.ClientMethod<$0.DeletePolicyRequest, $0.DeletePolicyResponse>(
          '/k1s0.system.policy.v1.PolicyService/DeletePolicy',
          ($0.DeletePolicyRequest value) => value.writeToBuffer(),
          $0.DeletePolicyResponse.fromBuffer);
  static final _$createBundle =
      $grpc.ClientMethod<$0.CreateBundleRequest, $0.CreateBundleResponse>(
          '/k1s0.system.policy.v1.PolicyService/CreateBundle',
          ($0.CreateBundleRequest value) => value.writeToBuffer(),
          $0.CreateBundleResponse.fromBuffer);
  static final _$listBundles =
      $grpc.ClientMethod<$0.ListBundlesRequest, $0.ListBundlesResponse>(
          '/k1s0.system.policy.v1.PolicyService/ListBundles',
          ($0.ListBundlesRequest value) => value.writeToBuffer(),
          $0.ListBundlesResponse.fromBuffer);
  static final _$getBundle =
      $grpc.ClientMethod<$0.GetBundleRequest, $0.GetBundleResponse>(
          '/k1s0.system.policy.v1.PolicyService/GetBundle',
          ($0.GetBundleRequest value) => value.writeToBuffer(),
          $0.GetBundleResponse.fromBuffer);
}

@$pb.GrpcServiceName('k1s0.system.policy.v1.PolicyService')
abstract class PolicyServiceBase extends $grpc.Service {
  $core.String get $name => 'k1s0.system.policy.v1.PolicyService';

  PolicyServiceBase() {
    $addMethod($grpc.ServiceMethod<$0.EvaluatePolicyRequest,
            $0.EvaluatePolicyResponse>(
        'EvaluatePolicy',
        evaluatePolicy_Pre,
        false,
        false,
        ($core.List<$core.int> value) =>
            $0.EvaluatePolicyRequest.fromBuffer(value),
        ($0.EvaluatePolicyResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.GetPolicyRequest, $0.GetPolicyResponse>(
        'GetPolicy',
        getPolicy_Pre,
        false,
        false,
        ($core.List<$core.int> value) => $0.GetPolicyRequest.fromBuffer(value),
        ($0.GetPolicyResponse value) => value.writeToBuffer()));
    $addMethod(
        $grpc.ServiceMethod<$0.ListPoliciesRequest, $0.ListPoliciesResponse>(
            'ListPolicies',
            listPolicies_Pre,
            false,
            false,
            ($core.List<$core.int> value) =>
                $0.ListPoliciesRequest.fromBuffer(value),
            ($0.ListPoliciesResponse value) => value.writeToBuffer()));
    $addMethod(
        $grpc.ServiceMethod<$0.CreatePolicyRequest, $0.CreatePolicyResponse>(
            'CreatePolicy',
            createPolicy_Pre,
            false,
            false,
            ($core.List<$core.int> value) =>
                $0.CreatePolicyRequest.fromBuffer(value),
            ($0.CreatePolicyResponse value) => value.writeToBuffer()));
    $addMethod(
        $grpc.ServiceMethod<$0.UpdatePolicyRequest, $0.UpdatePolicyResponse>(
            'UpdatePolicy',
            updatePolicy_Pre,
            false,
            false,
            ($core.List<$core.int> value) =>
                $0.UpdatePolicyRequest.fromBuffer(value),
            ($0.UpdatePolicyResponse value) => value.writeToBuffer()));
    $addMethod(
        $grpc.ServiceMethod<$0.DeletePolicyRequest, $0.DeletePolicyResponse>(
            'DeletePolicy',
            deletePolicy_Pre,
            false,
            false,
            ($core.List<$core.int> value) =>
                $0.DeletePolicyRequest.fromBuffer(value),
            ($0.DeletePolicyResponse value) => value.writeToBuffer()));
    $addMethod(
        $grpc.ServiceMethod<$0.CreateBundleRequest, $0.CreateBundleResponse>(
            'CreateBundle',
            createBundle_Pre,
            false,
            false,
            ($core.List<$core.int> value) =>
                $0.CreateBundleRequest.fromBuffer(value),
            ($0.CreateBundleResponse value) => value.writeToBuffer()));
    $addMethod(
        $grpc.ServiceMethod<$0.ListBundlesRequest, $0.ListBundlesResponse>(
            'ListBundles',
            listBundles_Pre,
            false,
            false,
            ($core.List<$core.int> value) =>
                $0.ListBundlesRequest.fromBuffer(value),
            ($0.ListBundlesResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.GetBundleRequest, $0.GetBundleResponse>(
        'GetBundle',
        getBundle_Pre,
        false,
        false,
        ($core.List<$core.int> value) => $0.GetBundleRequest.fromBuffer(value),
        ($0.GetBundleResponse value) => value.writeToBuffer()));
  }

  $async.Future<$0.EvaluatePolicyResponse> evaluatePolicy_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.EvaluatePolicyRequest> $request) async {
    return evaluatePolicy($call, await $request);
  }

  $async.Future<$0.EvaluatePolicyResponse> evaluatePolicy(
      $grpc.ServiceCall call, $0.EvaluatePolicyRequest request);

  $async.Future<$0.GetPolicyResponse> getPolicy_Pre($grpc.ServiceCall $call,
      $async.Future<$0.GetPolicyRequest> $request) async {
    return getPolicy($call, await $request);
  }

  $async.Future<$0.GetPolicyResponse> getPolicy(
      $grpc.ServiceCall call, $0.GetPolicyRequest request);

  $async.Future<$0.ListPoliciesResponse> listPolicies_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.ListPoliciesRequest> $request) async {
    return listPolicies($call, await $request);
  }

  $async.Future<$0.ListPoliciesResponse> listPolicies(
      $grpc.ServiceCall call, $0.ListPoliciesRequest request);

  $async.Future<$0.CreatePolicyResponse> createPolicy_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.CreatePolicyRequest> $request) async {
    return createPolicy($call, await $request);
  }

  $async.Future<$0.CreatePolicyResponse> createPolicy(
      $grpc.ServiceCall call, $0.CreatePolicyRequest request);

  $async.Future<$0.UpdatePolicyResponse> updatePolicy_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.UpdatePolicyRequest> $request) async {
    return updatePolicy($call, await $request);
  }

  $async.Future<$0.UpdatePolicyResponse> updatePolicy(
      $grpc.ServiceCall call, $0.UpdatePolicyRequest request);

  $async.Future<$0.DeletePolicyResponse> deletePolicy_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.DeletePolicyRequest> $request) async {
    return deletePolicy($call, await $request);
  }

  $async.Future<$0.DeletePolicyResponse> deletePolicy(
      $grpc.ServiceCall call, $0.DeletePolicyRequest request);

  $async.Future<$0.CreateBundleResponse> createBundle_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.CreateBundleRequest> $request) async {
    return createBundle($call, await $request);
  }

  $async.Future<$0.CreateBundleResponse> createBundle(
      $grpc.ServiceCall call, $0.CreateBundleRequest request);

  $async.Future<$0.ListBundlesResponse> listBundles_Pre($grpc.ServiceCall $call,
      $async.Future<$0.ListBundlesRequest> $request) async {
    return listBundles($call, await $request);
  }

  $async.Future<$0.ListBundlesResponse> listBundles(
      $grpc.ServiceCall call, $0.ListBundlesRequest request);

  $async.Future<$0.GetBundleResponse> getBundle_Pre($grpc.ServiceCall $call,
      $async.Future<$0.GetBundleRequest> $request) async {
    return getBundle($call, await $request);
  }

  $async.Future<$0.GetBundleResponse> getBundle(
      $grpc.ServiceCall call, $0.GetBundleRequest request);
}
