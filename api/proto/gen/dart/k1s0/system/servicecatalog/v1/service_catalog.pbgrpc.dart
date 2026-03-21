// This is a generated file - do not edit.
//
// Generated from k1s0/system/servicecatalog/v1/service_catalog.proto.

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

import 'service_catalog.pb.dart' as $0;

export 'service_catalog.pb.dart';

/// ServiceCatalogService はサービスカタログの CRUD とヘルスチェックを提供する。
@$pb.GrpcServiceName('k1s0.system.servicecatalog.v1.ServiceCatalogService')
class ServiceCatalogServiceClient extends $grpc.Client {
  /// The hostname for this service.
  static const $core.String defaultHost = '';

  /// OAuth scopes needed for the client.
  static const $core.List<$core.String> oauthScopes = [
    '',
  ];

  ServiceCatalogServiceClient(super.channel,
      {super.options, super.interceptors});

  /// サービス登録
  $grpc.ResponseFuture<$0.RegisterServiceResponse> registerService(
    $0.RegisterServiceRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$registerService, request, options: options);
  }

  /// サービス取得
  $grpc.ResponseFuture<$0.GetServiceResponse> getService(
    $0.GetServiceRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$getService, request, options: options);
  }

  /// サービス一覧取得
  $grpc.ResponseFuture<$0.ListServicesResponse> listServices(
    $0.ListServicesRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$listServices, request, options: options);
  }

  /// サービス更新
  $grpc.ResponseFuture<$0.UpdateServiceResponse> updateService(
    $0.UpdateServiceRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$updateService, request, options: options);
  }

  /// サービス削除
  $grpc.ResponseFuture<$0.DeleteServiceResponse> deleteService(
    $0.DeleteServiceRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$deleteService, request, options: options);
  }

  /// ヘルスチェック
  $grpc.ResponseFuture<$0.HealthCheckResponse> healthCheck(
    $0.HealthCheckRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$healthCheck, request, options: options);
  }

  // method descriptors

  static final _$registerService = $grpc.ClientMethod<$0.RegisterServiceRequest,
          $0.RegisterServiceResponse>(
      '/k1s0.system.servicecatalog.v1.ServiceCatalogService/RegisterService',
      ($0.RegisterServiceRequest value) => value.writeToBuffer(),
      $0.RegisterServiceResponse.fromBuffer);
  static final _$getService =
      $grpc.ClientMethod<$0.GetServiceRequest, $0.GetServiceResponse>(
          '/k1s0.system.servicecatalog.v1.ServiceCatalogService/GetService',
          ($0.GetServiceRequest value) => value.writeToBuffer(),
          $0.GetServiceResponse.fromBuffer);
  static final _$listServices =
      $grpc.ClientMethod<$0.ListServicesRequest, $0.ListServicesResponse>(
          '/k1s0.system.servicecatalog.v1.ServiceCatalogService/ListServices',
          ($0.ListServicesRequest value) => value.writeToBuffer(),
          $0.ListServicesResponse.fromBuffer);
  static final _$updateService =
      $grpc.ClientMethod<$0.UpdateServiceRequest, $0.UpdateServiceResponse>(
          '/k1s0.system.servicecatalog.v1.ServiceCatalogService/UpdateService',
          ($0.UpdateServiceRequest value) => value.writeToBuffer(),
          $0.UpdateServiceResponse.fromBuffer);
  static final _$deleteService =
      $grpc.ClientMethod<$0.DeleteServiceRequest, $0.DeleteServiceResponse>(
          '/k1s0.system.servicecatalog.v1.ServiceCatalogService/DeleteService',
          ($0.DeleteServiceRequest value) => value.writeToBuffer(),
          $0.DeleteServiceResponse.fromBuffer);
  static final _$healthCheck =
      $grpc.ClientMethod<$0.HealthCheckRequest, $0.HealthCheckResponse>(
          '/k1s0.system.servicecatalog.v1.ServiceCatalogService/HealthCheck',
          ($0.HealthCheckRequest value) => value.writeToBuffer(),
          $0.HealthCheckResponse.fromBuffer);
}

@$pb.GrpcServiceName('k1s0.system.servicecatalog.v1.ServiceCatalogService')
abstract class ServiceCatalogServiceBase extends $grpc.Service {
  $core.String get $name =>
      'k1s0.system.servicecatalog.v1.ServiceCatalogService';

  ServiceCatalogServiceBase() {
    $addMethod($grpc.ServiceMethod<$0.RegisterServiceRequest,
            $0.RegisterServiceResponse>(
        'RegisterService',
        registerService_Pre,
        false,
        false,
        ($core.List<$core.int> value) =>
            $0.RegisterServiceRequest.fromBuffer(value),
        ($0.RegisterServiceResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.GetServiceRequest, $0.GetServiceResponse>(
        'GetService',
        getService_Pre,
        false,
        false,
        ($core.List<$core.int> value) => $0.GetServiceRequest.fromBuffer(value),
        ($0.GetServiceResponse value) => value.writeToBuffer()));
    $addMethod(
        $grpc.ServiceMethod<$0.ListServicesRequest, $0.ListServicesResponse>(
            'ListServices',
            listServices_Pre,
            false,
            false,
            ($core.List<$core.int> value) =>
                $0.ListServicesRequest.fromBuffer(value),
            ($0.ListServicesResponse value) => value.writeToBuffer()));
    $addMethod(
        $grpc.ServiceMethod<$0.UpdateServiceRequest, $0.UpdateServiceResponse>(
            'UpdateService',
            updateService_Pre,
            false,
            false,
            ($core.List<$core.int> value) =>
                $0.UpdateServiceRequest.fromBuffer(value),
            ($0.UpdateServiceResponse value) => value.writeToBuffer()));
    $addMethod(
        $grpc.ServiceMethod<$0.DeleteServiceRequest, $0.DeleteServiceResponse>(
            'DeleteService',
            deleteService_Pre,
            false,
            false,
            ($core.List<$core.int> value) =>
                $0.DeleteServiceRequest.fromBuffer(value),
            ($0.DeleteServiceResponse value) => value.writeToBuffer()));
    $addMethod(
        $grpc.ServiceMethod<$0.HealthCheckRequest, $0.HealthCheckResponse>(
            'HealthCheck',
            healthCheck_Pre,
            false,
            false,
            ($core.List<$core.int> value) =>
                $0.HealthCheckRequest.fromBuffer(value),
            ($0.HealthCheckResponse value) => value.writeToBuffer()));
  }

  $async.Future<$0.RegisterServiceResponse> registerService_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.RegisterServiceRequest> $request) async {
    return registerService($call, await $request);
  }

  $async.Future<$0.RegisterServiceResponse> registerService(
      $grpc.ServiceCall call, $0.RegisterServiceRequest request);

  $async.Future<$0.GetServiceResponse> getService_Pre($grpc.ServiceCall $call,
      $async.Future<$0.GetServiceRequest> $request) async {
    return getService($call, await $request);
  }

  $async.Future<$0.GetServiceResponse> getService(
      $grpc.ServiceCall call, $0.GetServiceRequest request);

  $async.Future<$0.ListServicesResponse> listServices_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.ListServicesRequest> $request) async {
    return listServices($call, await $request);
  }

  $async.Future<$0.ListServicesResponse> listServices(
      $grpc.ServiceCall call, $0.ListServicesRequest request);

  $async.Future<$0.UpdateServiceResponse> updateService_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.UpdateServiceRequest> $request) async {
    return updateService($call, await $request);
  }

  $async.Future<$0.UpdateServiceResponse> updateService(
      $grpc.ServiceCall call, $0.UpdateServiceRequest request);

  $async.Future<$0.DeleteServiceResponse> deleteService_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.DeleteServiceRequest> $request) async {
    return deleteService($call, await $request);
  }

  $async.Future<$0.DeleteServiceResponse> deleteService(
      $grpc.ServiceCall call, $0.DeleteServiceRequest request);

  $async.Future<$0.HealthCheckResponse> healthCheck_Pre($grpc.ServiceCall $call,
      $async.Future<$0.HealthCheckRequest> $request) async {
    return healthCheck($call, await $request);
  }

  $async.Future<$0.HealthCheckResponse> healthCheck(
      $grpc.ServiceCall call, $0.HealthCheckRequest request);
}
