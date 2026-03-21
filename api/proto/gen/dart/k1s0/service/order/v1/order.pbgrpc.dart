// This is a generated file - do not edit.
//
// Generated from k1s0/service/order/v1/order.proto.

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

import 'order.pb.dart' as $0;

export 'order.pb.dart';

@$pb.GrpcServiceName('k1s0.service.order.v1.OrderService')
class OrderServiceClient extends $grpc.Client {
  /// The hostname for this service.
  static const $core.String defaultHost = '';

  /// OAuth scopes needed for the client.
  static const $core.List<$core.String> oauthScopes = [
    '',
  ];

  OrderServiceClient(super.channel, {super.options, super.interceptors});

  /// 注文作成
  $grpc.ResponseFuture<$0.CreateOrderResponse> createOrder(
    $0.CreateOrderRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$createOrder, request, options: options);
  }

  /// 注文取得
  $grpc.ResponseFuture<$0.GetOrderResponse> getOrder(
    $0.GetOrderRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$getOrder, request, options: options);
  }

  /// 注文一覧
  $grpc.ResponseFuture<$0.ListOrdersResponse> listOrders(
    $0.ListOrdersRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$listOrders, request, options: options);
  }

  /// 注文ステータス更新
  $grpc.ResponseFuture<$0.UpdateOrderStatusResponse> updateOrderStatus(
    $0.UpdateOrderStatusRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$updateOrderStatus, request, options: options);
  }

  // method descriptors

  static final _$createOrder =
      $grpc.ClientMethod<$0.CreateOrderRequest, $0.CreateOrderResponse>(
          '/k1s0.service.order.v1.OrderService/CreateOrder',
          ($0.CreateOrderRequest value) => value.writeToBuffer(),
          $0.CreateOrderResponse.fromBuffer);
  static final _$getOrder =
      $grpc.ClientMethod<$0.GetOrderRequest, $0.GetOrderResponse>(
          '/k1s0.service.order.v1.OrderService/GetOrder',
          ($0.GetOrderRequest value) => value.writeToBuffer(),
          $0.GetOrderResponse.fromBuffer);
  static final _$listOrders =
      $grpc.ClientMethod<$0.ListOrdersRequest, $0.ListOrdersResponse>(
          '/k1s0.service.order.v1.OrderService/ListOrders',
          ($0.ListOrdersRequest value) => value.writeToBuffer(),
          $0.ListOrdersResponse.fromBuffer);
  static final _$updateOrderStatus = $grpc.ClientMethod<
          $0.UpdateOrderStatusRequest, $0.UpdateOrderStatusResponse>(
      '/k1s0.service.order.v1.OrderService/UpdateOrderStatus',
      ($0.UpdateOrderStatusRequest value) => value.writeToBuffer(),
      $0.UpdateOrderStatusResponse.fromBuffer);
}

@$pb.GrpcServiceName('k1s0.service.order.v1.OrderService')
abstract class OrderServiceBase extends $grpc.Service {
  $core.String get $name => 'k1s0.service.order.v1.OrderService';

  OrderServiceBase() {
    $addMethod(
        $grpc.ServiceMethod<$0.CreateOrderRequest, $0.CreateOrderResponse>(
            'CreateOrder',
            createOrder_Pre,
            false,
            false,
            ($core.List<$core.int> value) =>
                $0.CreateOrderRequest.fromBuffer(value),
            ($0.CreateOrderResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.GetOrderRequest, $0.GetOrderResponse>(
        'GetOrder',
        getOrder_Pre,
        false,
        false,
        ($core.List<$core.int> value) => $0.GetOrderRequest.fromBuffer(value),
        ($0.GetOrderResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.ListOrdersRequest, $0.ListOrdersResponse>(
        'ListOrders',
        listOrders_Pre,
        false,
        false,
        ($core.List<$core.int> value) => $0.ListOrdersRequest.fromBuffer(value),
        ($0.ListOrdersResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.UpdateOrderStatusRequest,
            $0.UpdateOrderStatusResponse>(
        'UpdateOrderStatus',
        updateOrderStatus_Pre,
        false,
        false,
        ($core.List<$core.int> value) =>
            $0.UpdateOrderStatusRequest.fromBuffer(value),
        ($0.UpdateOrderStatusResponse value) => value.writeToBuffer()));
  }

  $async.Future<$0.CreateOrderResponse> createOrder_Pre($grpc.ServiceCall $call,
      $async.Future<$0.CreateOrderRequest> $request) async {
    return createOrder($call, await $request);
  }

  $async.Future<$0.CreateOrderResponse> createOrder(
      $grpc.ServiceCall call, $0.CreateOrderRequest request);

  $async.Future<$0.GetOrderResponse> getOrder_Pre($grpc.ServiceCall $call,
      $async.Future<$0.GetOrderRequest> $request) async {
    return getOrder($call, await $request);
  }

  $async.Future<$0.GetOrderResponse> getOrder(
      $grpc.ServiceCall call, $0.GetOrderRequest request);

  $async.Future<$0.ListOrdersResponse> listOrders_Pre($grpc.ServiceCall $call,
      $async.Future<$0.ListOrdersRequest> $request) async {
    return listOrders($call, await $request);
  }

  $async.Future<$0.ListOrdersResponse> listOrders(
      $grpc.ServiceCall call, $0.ListOrdersRequest request);

  $async.Future<$0.UpdateOrderStatusResponse> updateOrderStatus_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.UpdateOrderStatusRequest> $request) async {
    return updateOrderStatus($call, await $request);
  }

  $async.Future<$0.UpdateOrderStatusResponse> updateOrderStatus(
      $grpc.ServiceCall call, $0.UpdateOrderStatusRequest request);
}
