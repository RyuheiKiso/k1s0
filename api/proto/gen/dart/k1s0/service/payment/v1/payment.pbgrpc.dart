// This is a generated file - do not edit.
//
// Generated from k1s0/service/payment/v1/payment.proto.

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

import 'payment.pb.dart' as $0;

export 'payment.pb.dart';

@$pb.GrpcServiceName('k1s0.service.payment.v1.PaymentService')
class PaymentServiceClient extends $grpc.Client {
  /// The hostname for this service.
  static const $core.String defaultHost = '';

  /// OAuth scopes needed for the client.
  static const $core.List<$core.String> oauthScopes = [
    '',
  ];

  PaymentServiceClient(super.channel, {super.options, super.interceptors});

  /// 決済開始
  $grpc.ResponseFuture<$0.InitiatePaymentResponse> initiatePayment(
    $0.InitiatePaymentRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$initiatePayment, request, options: options);
  }

  /// 決済取得
  $grpc.ResponseFuture<$0.GetPaymentResponse> getPayment(
    $0.GetPaymentRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$getPayment, request, options: options);
  }

  /// 決済一覧
  $grpc.ResponseFuture<$0.ListPaymentsResponse> listPayments(
    $0.ListPaymentsRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$listPayments, request, options: options);
  }

  /// 決済完了
  $grpc.ResponseFuture<$0.CompletePaymentResponse> completePayment(
    $0.CompletePaymentRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$completePayment, request, options: options);
  }

  /// 決済失敗
  $grpc.ResponseFuture<$0.FailPaymentResponse> failPayment(
    $0.FailPaymentRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$failPayment, request, options: options);
  }

  /// 決済返金
  $grpc.ResponseFuture<$0.RefundPaymentResponse> refundPayment(
    $0.RefundPaymentRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$refundPayment, request, options: options);
  }

  // method descriptors

  static final _$initiatePayment =
      $grpc.ClientMethod<$0.InitiatePaymentRequest, $0.InitiatePaymentResponse>(
          '/k1s0.service.payment.v1.PaymentService/InitiatePayment',
          ($0.InitiatePaymentRequest value) => value.writeToBuffer(),
          $0.InitiatePaymentResponse.fromBuffer);
  static final _$getPayment =
      $grpc.ClientMethod<$0.GetPaymentRequest, $0.GetPaymentResponse>(
          '/k1s0.service.payment.v1.PaymentService/GetPayment',
          ($0.GetPaymentRequest value) => value.writeToBuffer(),
          $0.GetPaymentResponse.fromBuffer);
  static final _$listPayments =
      $grpc.ClientMethod<$0.ListPaymentsRequest, $0.ListPaymentsResponse>(
          '/k1s0.service.payment.v1.PaymentService/ListPayments',
          ($0.ListPaymentsRequest value) => value.writeToBuffer(),
          $0.ListPaymentsResponse.fromBuffer);
  static final _$completePayment =
      $grpc.ClientMethod<$0.CompletePaymentRequest, $0.CompletePaymentResponse>(
          '/k1s0.service.payment.v1.PaymentService/CompletePayment',
          ($0.CompletePaymentRequest value) => value.writeToBuffer(),
          $0.CompletePaymentResponse.fromBuffer);
  static final _$failPayment =
      $grpc.ClientMethod<$0.FailPaymentRequest, $0.FailPaymentResponse>(
          '/k1s0.service.payment.v1.PaymentService/FailPayment',
          ($0.FailPaymentRequest value) => value.writeToBuffer(),
          $0.FailPaymentResponse.fromBuffer);
  static final _$refundPayment =
      $grpc.ClientMethod<$0.RefundPaymentRequest, $0.RefundPaymentResponse>(
          '/k1s0.service.payment.v1.PaymentService/RefundPayment',
          ($0.RefundPaymentRequest value) => value.writeToBuffer(),
          $0.RefundPaymentResponse.fromBuffer);
}

@$pb.GrpcServiceName('k1s0.service.payment.v1.PaymentService')
abstract class PaymentServiceBase extends $grpc.Service {
  $core.String get $name => 'k1s0.service.payment.v1.PaymentService';

  PaymentServiceBase() {
    $addMethod($grpc.ServiceMethod<$0.InitiatePaymentRequest,
            $0.InitiatePaymentResponse>(
        'InitiatePayment',
        initiatePayment_Pre,
        false,
        false,
        ($core.List<$core.int> value) =>
            $0.InitiatePaymentRequest.fromBuffer(value),
        ($0.InitiatePaymentResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.GetPaymentRequest, $0.GetPaymentResponse>(
        'GetPayment',
        getPayment_Pre,
        false,
        false,
        ($core.List<$core.int> value) => $0.GetPaymentRequest.fromBuffer(value),
        ($0.GetPaymentResponse value) => value.writeToBuffer()));
    $addMethod(
        $grpc.ServiceMethod<$0.ListPaymentsRequest, $0.ListPaymentsResponse>(
            'ListPayments',
            listPayments_Pre,
            false,
            false,
            ($core.List<$core.int> value) =>
                $0.ListPaymentsRequest.fromBuffer(value),
            ($0.ListPaymentsResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.CompletePaymentRequest,
            $0.CompletePaymentResponse>(
        'CompletePayment',
        completePayment_Pre,
        false,
        false,
        ($core.List<$core.int> value) =>
            $0.CompletePaymentRequest.fromBuffer(value),
        ($0.CompletePaymentResponse value) => value.writeToBuffer()));
    $addMethod(
        $grpc.ServiceMethod<$0.FailPaymentRequest, $0.FailPaymentResponse>(
            'FailPayment',
            failPayment_Pre,
            false,
            false,
            ($core.List<$core.int> value) =>
                $0.FailPaymentRequest.fromBuffer(value),
            ($0.FailPaymentResponse value) => value.writeToBuffer()));
    $addMethod(
        $grpc.ServiceMethod<$0.RefundPaymentRequest, $0.RefundPaymentResponse>(
            'RefundPayment',
            refundPayment_Pre,
            false,
            false,
            ($core.List<$core.int> value) =>
                $0.RefundPaymentRequest.fromBuffer(value),
            ($0.RefundPaymentResponse value) => value.writeToBuffer()));
  }

  $async.Future<$0.InitiatePaymentResponse> initiatePayment_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.InitiatePaymentRequest> $request) async {
    return initiatePayment($call, await $request);
  }

  $async.Future<$0.InitiatePaymentResponse> initiatePayment(
      $grpc.ServiceCall call, $0.InitiatePaymentRequest request);

  $async.Future<$0.GetPaymentResponse> getPayment_Pre($grpc.ServiceCall $call,
      $async.Future<$0.GetPaymentRequest> $request) async {
    return getPayment($call, await $request);
  }

  $async.Future<$0.GetPaymentResponse> getPayment(
      $grpc.ServiceCall call, $0.GetPaymentRequest request);

  $async.Future<$0.ListPaymentsResponse> listPayments_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.ListPaymentsRequest> $request) async {
    return listPayments($call, await $request);
  }

  $async.Future<$0.ListPaymentsResponse> listPayments(
      $grpc.ServiceCall call, $0.ListPaymentsRequest request);

  $async.Future<$0.CompletePaymentResponse> completePayment_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.CompletePaymentRequest> $request) async {
    return completePayment($call, await $request);
  }

  $async.Future<$0.CompletePaymentResponse> completePayment(
      $grpc.ServiceCall call, $0.CompletePaymentRequest request);

  $async.Future<$0.FailPaymentResponse> failPayment_Pre($grpc.ServiceCall $call,
      $async.Future<$0.FailPaymentRequest> $request) async {
    return failPayment($call, await $request);
  }

  $async.Future<$0.FailPaymentResponse> failPayment(
      $grpc.ServiceCall call, $0.FailPaymentRequest request);

  $async.Future<$0.RefundPaymentResponse> refundPayment_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.RefundPaymentRequest> $request) async {
    return refundPayment($call, await $request);
  }

  $async.Future<$0.RefundPaymentResponse> refundPayment(
      $grpc.ServiceCall call, $0.RefundPaymentRequest request);
}
