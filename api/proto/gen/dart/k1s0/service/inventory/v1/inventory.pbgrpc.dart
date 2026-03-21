// This is a generated file - do not edit.
//
// Generated from k1s0/service/inventory/v1/inventory.proto.

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

import 'inventory.pb.dart' as $0;

export 'inventory.pb.dart';

@$pb.GrpcServiceName('k1s0.service.inventory.v1.InventoryService')
class InventoryServiceClient extends $grpc.Client {
  /// The hostname for this service.
  static const $core.String defaultHost = '';

  /// OAuth scopes needed for the client.
  static const $core.List<$core.String> oauthScopes = [
    '',
  ];

  InventoryServiceClient(super.channel, {super.options, super.interceptors});

  /// 在庫予約
  $grpc.ResponseFuture<$0.ReserveStockResponse> reserveStock(
    $0.ReserveStockRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$reserveStock, request, options: options);
  }

  /// 在庫解放
  $grpc.ResponseFuture<$0.ReleaseStockResponse> releaseStock(
    $0.ReleaseStockRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$releaseStock, request, options: options);
  }

  /// 在庫取得
  $grpc.ResponseFuture<$0.GetInventoryResponse> getInventory(
    $0.GetInventoryRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$getInventory, request, options: options);
  }

  /// 在庫一覧
  $grpc.ResponseFuture<$0.ListInventoryResponse> listInventory(
    $0.ListInventoryRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$listInventory, request, options: options);
  }

  /// 在庫更新
  $grpc.ResponseFuture<$0.UpdateStockResponse> updateStock(
    $0.UpdateStockRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$updateStock, request, options: options);
  }

  // method descriptors

  static final _$reserveStock =
      $grpc.ClientMethod<$0.ReserveStockRequest, $0.ReserveStockResponse>(
          '/k1s0.service.inventory.v1.InventoryService/ReserveStock',
          ($0.ReserveStockRequest value) => value.writeToBuffer(),
          $0.ReserveStockResponse.fromBuffer);
  static final _$releaseStock =
      $grpc.ClientMethod<$0.ReleaseStockRequest, $0.ReleaseStockResponse>(
          '/k1s0.service.inventory.v1.InventoryService/ReleaseStock',
          ($0.ReleaseStockRequest value) => value.writeToBuffer(),
          $0.ReleaseStockResponse.fromBuffer);
  static final _$getInventory =
      $grpc.ClientMethod<$0.GetInventoryRequest, $0.GetInventoryResponse>(
          '/k1s0.service.inventory.v1.InventoryService/GetInventory',
          ($0.GetInventoryRequest value) => value.writeToBuffer(),
          $0.GetInventoryResponse.fromBuffer);
  static final _$listInventory =
      $grpc.ClientMethod<$0.ListInventoryRequest, $0.ListInventoryResponse>(
          '/k1s0.service.inventory.v1.InventoryService/ListInventory',
          ($0.ListInventoryRequest value) => value.writeToBuffer(),
          $0.ListInventoryResponse.fromBuffer);
  static final _$updateStock =
      $grpc.ClientMethod<$0.UpdateStockRequest, $0.UpdateStockResponse>(
          '/k1s0.service.inventory.v1.InventoryService/UpdateStock',
          ($0.UpdateStockRequest value) => value.writeToBuffer(),
          $0.UpdateStockResponse.fromBuffer);
}

@$pb.GrpcServiceName('k1s0.service.inventory.v1.InventoryService')
abstract class InventoryServiceBase extends $grpc.Service {
  $core.String get $name => 'k1s0.service.inventory.v1.InventoryService';

  InventoryServiceBase() {
    $addMethod(
        $grpc.ServiceMethod<$0.ReserveStockRequest, $0.ReserveStockResponse>(
            'ReserveStock',
            reserveStock_Pre,
            false,
            false,
            ($core.List<$core.int> value) =>
                $0.ReserveStockRequest.fromBuffer(value),
            ($0.ReserveStockResponse value) => value.writeToBuffer()));
    $addMethod(
        $grpc.ServiceMethod<$0.ReleaseStockRequest, $0.ReleaseStockResponse>(
            'ReleaseStock',
            releaseStock_Pre,
            false,
            false,
            ($core.List<$core.int> value) =>
                $0.ReleaseStockRequest.fromBuffer(value),
            ($0.ReleaseStockResponse value) => value.writeToBuffer()));
    $addMethod(
        $grpc.ServiceMethod<$0.GetInventoryRequest, $0.GetInventoryResponse>(
            'GetInventory',
            getInventory_Pre,
            false,
            false,
            ($core.List<$core.int> value) =>
                $0.GetInventoryRequest.fromBuffer(value),
            ($0.GetInventoryResponse value) => value.writeToBuffer()));
    $addMethod(
        $grpc.ServiceMethod<$0.ListInventoryRequest, $0.ListInventoryResponse>(
            'ListInventory',
            listInventory_Pre,
            false,
            false,
            ($core.List<$core.int> value) =>
                $0.ListInventoryRequest.fromBuffer(value),
            ($0.ListInventoryResponse value) => value.writeToBuffer()));
    $addMethod(
        $grpc.ServiceMethod<$0.UpdateStockRequest, $0.UpdateStockResponse>(
            'UpdateStock',
            updateStock_Pre,
            false,
            false,
            ($core.List<$core.int> value) =>
                $0.UpdateStockRequest.fromBuffer(value),
            ($0.UpdateStockResponse value) => value.writeToBuffer()));
  }

  $async.Future<$0.ReserveStockResponse> reserveStock_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.ReserveStockRequest> $request) async {
    return reserveStock($call, await $request);
  }

  $async.Future<$0.ReserveStockResponse> reserveStock(
      $grpc.ServiceCall call, $0.ReserveStockRequest request);

  $async.Future<$0.ReleaseStockResponse> releaseStock_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.ReleaseStockRequest> $request) async {
    return releaseStock($call, await $request);
  }

  $async.Future<$0.ReleaseStockResponse> releaseStock(
      $grpc.ServiceCall call, $0.ReleaseStockRequest request);

  $async.Future<$0.GetInventoryResponse> getInventory_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.GetInventoryRequest> $request) async {
    return getInventory($call, await $request);
  }

  $async.Future<$0.GetInventoryResponse> getInventory(
      $grpc.ServiceCall call, $0.GetInventoryRequest request);

  $async.Future<$0.ListInventoryResponse> listInventory_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.ListInventoryRequest> $request) async {
    return listInventory($call, await $request);
  }

  $async.Future<$0.ListInventoryResponse> listInventory(
      $grpc.ServiceCall call, $0.ListInventoryRequest request);

  $async.Future<$0.UpdateStockResponse> updateStock_Pre($grpc.ServiceCall $call,
      $async.Future<$0.UpdateStockRequest> $request) async {
    return updateStock($call, await $request);
  }

  $async.Future<$0.UpdateStockResponse> updateStock(
      $grpc.ServiceCall call, $0.UpdateStockRequest request);
}
