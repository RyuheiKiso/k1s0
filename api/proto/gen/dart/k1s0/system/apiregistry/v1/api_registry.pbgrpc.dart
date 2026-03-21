// This is a generated file - do not edit.
//
// Generated from k1s0/system/apiregistry/v1/api_registry.proto.

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

import 'api_registry.pb.dart' as $0;

export 'api_registry.pb.dart';

@$pb.GrpcServiceName('k1s0.system.apiregistry.v1.ApiRegistryService')
class ApiRegistryServiceClient extends $grpc.Client {
  /// The hostname for this service.
  static const $core.String defaultHost = '';

  /// OAuth scopes needed for the client.
  static const $core.List<$core.String> oauthScopes = [
    '',
  ];

  ApiRegistryServiceClient(super.channel, {super.options, super.interceptors});

  $grpc.ResponseFuture<$0.ListSchemasResponse> listSchemas(
    $0.ListSchemasRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$listSchemas, request, options: options);
  }

  $grpc.ResponseFuture<$0.RegisterSchemaResponse> registerSchema(
    $0.RegisterSchemaRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$registerSchema, request, options: options);
  }

  $grpc.ResponseFuture<$0.GetSchemaResponse> getSchema(
    $0.GetSchemaRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$getSchema, request, options: options);
  }

  $grpc.ResponseFuture<$0.ListVersionsResponse> listVersions(
    $0.ListVersionsRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$listVersions, request, options: options);
  }

  $grpc.ResponseFuture<$0.RegisterVersionResponse> registerVersion(
    $0.RegisterVersionRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$registerVersion, request, options: options);
  }

  $grpc.ResponseFuture<$0.GetSchemaVersionResponse> getSchemaVersion(
    $0.GetSchemaVersionRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$getSchemaVersion, request, options: options);
  }

  $grpc.ResponseFuture<$0.DeleteVersionResponse> deleteVersion(
    $0.DeleteVersionRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$deleteVersion, request, options: options);
  }

  $grpc.ResponseFuture<$0.CheckCompatibilityResponse> checkCompatibility(
    $0.CheckCompatibilityRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$checkCompatibility, request, options: options);
  }

  $grpc.ResponseFuture<$0.GetDiffResponse> getDiff(
    $0.GetDiffRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$getDiff, request, options: options);
  }

  // method descriptors

  static final _$listSchemas =
      $grpc.ClientMethod<$0.ListSchemasRequest, $0.ListSchemasResponse>(
          '/k1s0.system.apiregistry.v1.ApiRegistryService/ListSchemas',
          ($0.ListSchemasRequest value) => value.writeToBuffer(),
          $0.ListSchemasResponse.fromBuffer);
  static final _$registerSchema =
      $grpc.ClientMethod<$0.RegisterSchemaRequest, $0.RegisterSchemaResponse>(
          '/k1s0.system.apiregistry.v1.ApiRegistryService/RegisterSchema',
          ($0.RegisterSchemaRequest value) => value.writeToBuffer(),
          $0.RegisterSchemaResponse.fromBuffer);
  static final _$getSchema =
      $grpc.ClientMethod<$0.GetSchemaRequest, $0.GetSchemaResponse>(
          '/k1s0.system.apiregistry.v1.ApiRegistryService/GetSchema',
          ($0.GetSchemaRequest value) => value.writeToBuffer(),
          $0.GetSchemaResponse.fromBuffer);
  static final _$listVersions =
      $grpc.ClientMethod<$0.ListVersionsRequest, $0.ListVersionsResponse>(
          '/k1s0.system.apiregistry.v1.ApiRegistryService/ListVersions',
          ($0.ListVersionsRequest value) => value.writeToBuffer(),
          $0.ListVersionsResponse.fromBuffer);
  static final _$registerVersion =
      $grpc.ClientMethod<$0.RegisterVersionRequest, $0.RegisterVersionResponse>(
          '/k1s0.system.apiregistry.v1.ApiRegistryService/RegisterVersion',
          ($0.RegisterVersionRequest value) => value.writeToBuffer(),
          $0.RegisterVersionResponse.fromBuffer);
  static final _$getSchemaVersion = $grpc.ClientMethod<
          $0.GetSchemaVersionRequest, $0.GetSchemaVersionResponse>(
      '/k1s0.system.apiregistry.v1.ApiRegistryService/GetSchemaVersion',
      ($0.GetSchemaVersionRequest value) => value.writeToBuffer(),
      $0.GetSchemaVersionResponse.fromBuffer);
  static final _$deleteVersion =
      $grpc.ClientMethod<$0.DeleteVersionRequest, $0.DeleteVersionResponse>(
          '/k1s0.system.apiregistry.v1.ApiRegistryService/DeleteVersion',
          ($0.DeleteVersionRequest value) => value.writeToBuffer(),
          $0.DeleteVersionResponse.fromBuffer);
  static final _$checkCompatibility = $grpc.ClientMethod<
          $0.CheckCompatibilityRequest, $0.CheckCompatibilityResponse>(
      '/k1s0.system.apiregistry.v1.ApiRegistryService/CheckCompatibility',
      ($0.CheckCompatibilityRequest value) => value.writeToBuffer(),
      $0.CheckCompatibilityResponse.fromBuffer);
  static final _$getDiff =
      $grpc.ClientMethod<$0.GetDiffRequest, $0.GetDiffResponse>(
          '/k1s0.system.apiregistry.v1.ApiRegistryService/GetDiff',
          ($0.GetDiffRequest value) => value.writeToBuffer(),
          $0.GetDiffResponse.fromBuffer);
}

@$pb.GrpcServiceName('k1s0.system.apiregistry.v1.ApiRegistryService')
abstract class ApiRegistryServiceBase extends $grpc.Service {
  $core.String get $name => 'k1s0.system.apiregistry.v1.ApiRegistryService';

  ApiRegistryServiceBase() {
    $addMethod(
        $grpc.ServiceMethod<$0.ListSchemasRequest, $0.ListSchemasResponse>(
            'ListSchemas',
            listSchemas_Pre,
            false,
            false,
            ($core.List<$core.int> value) =>
                $0.ListSchemasRequest.fromBuffer(value),
            ($0.ListSchemasResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.RegisterSchemaRequest,
            $0.RegisterSchemaResponse>(
        'RegisterSchema',
        registerSchema_Pre,
        false,
        false,
        ($core.List<$core.int> value) =>
            $0.RegisterSchemaRequest.fromBuffer(value),
        ($0.RegisterSchemaResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.GetSchemaRequest, $0.GetSchemaResponse>(
        'GetSchema',
        getSchema_Pre,
        false,
        false,
        ($core.List<$core.int> value) => $0.GetSchemaRequest.fromBuffer(value),
        ($0.GetSchemaResponse value) => value.writeToBuffer()));
    $addMethod(
        $grpc.ServiceMethod<$0.ListVersionsRequest, $0.ListVersionsResponse>(
            'ListVersions',
            listVersions_Pre,
            false,
            false,
            ($core.List<$core.int> value) =>
                $0.ListVersionsRequest.fromBuffer(value),
            ($0.ListVersionsResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.RegisterVersionRequest,
            $0.RegisterVersionResponse>(
        'RegisterVersion',
        registerVersion_Pre,
        false,
        false,
        ($core.List<$core.int> value) =>
            $0.RegisterVersionRequest.fromBuffer(value),
        ($0.RegisterVersionResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.GetSchemaVersionRequest,
            $0.GetSchemaVersionResponse>(
        'GetSchemaVersion',
        getSchemaVersion_Pre,
        false,
        false,
        ($core.List<$core.int> value) =>
            $0.GetSchemaVersionRequest.fromBuffer(value),
        ($0.GetSchemaVersionResponse value) => value.writeToBuffer()));
    $addMethod(
        $grpc.ServiceMethod<$0.DeleteVersionRequest, $0.DeleteVersionResponse>(
            'DeleteVersion',
            deleteVersion_Pre,
            false,
            false,
            ($core.List<$core.int> value) =>
                $0.DeleteVersionRequest.fromBuffer(value),
            ($0.DeleteVersionResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.CheckCompatibilityRequest,
            $0.CheckCompatibilityResponse>(
        'CheckCompatibility',
        checkCompatibility_Pre,
        false,
        false,
        ($core.List<$core.int> value) =>
            $0.CheckCompatibilityRequest.fromBuffer(value),
        ($0.CheckCompatibilityResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.GetDiffRequest, $0.GetDiffResponse>(
        'GetDiff',
        getDiff_Pre,
        false,
        false,
        ($core.List<$core.int> value) => $0.GetDiffRequest.fromBuffer(value),
        ($0.GetDiffResponse value) => value.writeToBuffer()));
  }

  $async.Future<$0.ListSchemasResponse> listSchemas_Pre($grpc.ServiceCall $call,
      $async.Future<$0.ListSchemasRequest> $request) async {
    return listSchemas($call, await $request);
  }

  $async.Future<$0.ListSchemasResponse> listSchemas(
      $grpc.ServiceCall call, $0.ListSchemasRequest request);

  $async.Future<$0.RegisterSchemaResponse> registerSchema_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.RegisterSchemaRequest> $request) async {
    return registerSchema($call, await $request);
  }

  $async.Future<$0.RegisterSchemaResponse> registerSchema(
      $grpc.ServiceCall call, $0.RegisterSchemaRequest request);

  $async.Future<$0.GetSchemaResponse> getSchema_Pre($grpc.ServiceCall $call,
      $async.Future<$0.GetSchemaRequest> $request) async {
    return getSchema($call, await $request);
  }

  $async.Future<$0.GetSchemaResponse> getSchema(
      $grpc.ServiceCall call, $0.GetSchemaRequest request);

  $async.Future<$0.ListVersionsResponse> listVersions_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.ListVersionsRequest> $request) async {
    return listVersions($call, await $request);
  }

  $async.Future<$0.ListVersionsResponse> listVersions(
      $grpc.ServiceCall call, $0.ListVersionsRequest request);

  $async.Future<$0.RegisterVersionResponse> registerVersion_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.RegisterVersionRequest> $request) async {
    return registerVersion($call, await $request);
  }

  $async.Future<$0.RegisterVersionResponse> registerVersion(
      $grpc.ServiceCall call, $0.RegisterVersionRequest request);

  $async.Future<$0.GetSchemaVersionResponse> getSchemaVersion_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.GetSchemaVersionRequest> $request) async {
    return getSchemaVersion($call, await $request);
  }

  $async.Future<$0.GetSchemaVersionResponse> getSchemaVersion(
      $grpc.ServiceCall call, $0.GetSchemaVersionRequest request);

  $async.Future<$0.DeleteVersionResponse> deleteVersion_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.DeleteVersionRequest> $request) async {
    return deleteVersion($call, await $request);
  }

  $async.Future<$0.DeleteVersionResponse> deleteVersion(
      $grpc.ServiceCall call, $0.DeleteVersionRequest request);

  $async.Future<$0.CheckCompatibilityResponse> checkCompatibility_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.CheckCompatibilityRequest> $request) async {
    return checkCompatibility($call, await $request);
  }

  $async.Future<$0.CheckCompatibilityResponse> checkCompatibility(
      $grpc.ServiceCall call, $0.CheckCompatibilityRequest request);

  $async.Future<$0.GetDiffResponse> getDiff_Pre($grpc.ServiceCall $call,
      $async.Future<$0.GetDiffRequest> $request) async {
    return getDiff($call, await $request);
  }

  $async.Future<$0.GetDiffResponse> getDiff(
      $grpc.ServiceCall call, $0.GetDiffRequest request);
}
