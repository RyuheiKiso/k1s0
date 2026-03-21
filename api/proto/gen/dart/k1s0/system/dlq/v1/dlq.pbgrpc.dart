// This is a generated file - do not edit.
//
// Generated from k1s0/system/dlq/v1/dlq.proto.

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

import 'dlq.pb.dart' as $0;

export 'dlq.pb.dart';

/// DlqService は DLQ メッセージ管理サービス。
@$pb.GrpcServiceName('k1s0.system.dlq.v1.DlqService')
class DlqServiceClient extends $grpc.Client {
  /// The hostname for this service.
  static const $core.String defaultHost = '';

  /// OAuth scopes needed for the client.
  static const $core.List<$core.String> oauthScopes = [
    '',
  ];

  DlqServiceClient(super.channel, {super.options, super.interceptors});

  /// DLQ メッセージ一覧取得
  $grpc.ResponseFuture<$0.ListMessagesResponse> listMessages(
    $0.ListMessagesRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$listMessages, request, options: options);
  }

  /// DLQ メッセージ取得
  $grpc.ResponseFuture<$0.GetMessageResponse> getMessage(
    $0.GetMessageRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$getMessage, request, options: options);
  }

  /// DLQ メッセージのリトライ
  $grpc.ResponseFuture<$0.RetryMessageResponse> retryMessage(
    $0.RetryMessageRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$retryMessage, request, options: options);
  }

  /// DLQ メッセージ削除
  $grpc.ResponseFuture<$0.DeleteMessageResponse> deleteMessage(
    $0.DeleteMessageRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$deleteMessage, request, options: options);
  }

  /// DLQ メッセージ一括リトライ
  $grpc.ResponseFuture<$0.RetryAllResponse> retryAll(
    $0.RetryAllRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$retryAll, request, options: options);
  }

  // method descriptors

  static final _$listMessages =
      $grpc.ClientMethod<$0.ListMessagesRequest, $0.ListMessagesResponse>(
          '/k1s0.system.dlq.v1.DlqService/ListMessages',
          ($0.ListMessagesRequest value) => value.writeToBuffer(),
          $0.ListMessagesResponse.fromBuffer);
  static final _$getMessage =
      $grpc.ClientMethod<$0.GetMessageRequest, $0.GetMessageResponse>(
          '/k1s0.system.dlq.v1.DlqService/GetMessage',
          ($0.GetMessageRequest value) => value.writeToBuffer(),
          $0.GetMessageResponse.fromBuffer);
  static final _$retryMessage =
      $grpc.ClientMethod<$0.RetryMessageRequest, $0.RetryMessageResponse>(
          '/k1s0.system.dlq.v1.DlqService/RetryMessage',
          ($0.RetryMessageRequest value) => value.writeToBuffer(),
          $0.RetryMessageResponse.fromBuffer);
  static final _$deleteMessage =
      $grpc.ClientMethod<$0.DeleteMessageRequest, $0.DeleteMessageResponse>(
          '/k1s0.system.dlq.v1.DlqService/DeleteMessage',
          ($0.DeleteMessageRequest value) => value.writeToBuffer(),
          $0.DeleteMessageResponse.fromBuffer);
  static final _$retryAll =
      $grpc.ClientMethod<$0.RetryAllRequest, $0.RetryAllResponse>(
          '/k1s0.system.dlq.v1.DlqService/RetryAll',
          ($0.RetryAllRequest value) => value.writeToBuffer(),
          $0.RetryAllResponse.fromBuffer);
}

@$pb.GrpcServiceName('k1s0.system.dlq.v1.DlqService')
abstract class DlqServiceBase extends $grpc.Service {
  $core.String get $name => 'k1s0.system.dlq.v1.DlqService';

  DlqServiceBase() {
    $addMethod(
        $grpc.ServiceMethod<$0.ListMessagesRequest, $0.ListMessagesResponse>(
            'ListMessages',
            listMessages_Pre,
            false,
            false,
            ($core.List<$core.int> value) =>
                $0.ListMessagesRequest.fromBuffer(value),
            ($0.ListMessagesResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.GetMessageRequest, $0.GetMessageResponse>(
        'GetMessage',
        getMessage_Pre,
        false,
        false,
        ($core.List<$core.int> value) => $0.GetMessageRequest.fromBuffer(value),
        ($0.GetMessageResponse value) => value.writeToBuffer()));
    $addMethod(
        $grpc.ServiceMethod<$0.RetryMessageRequest, $0.RetryMessageResponse>(
            'RetryMessage',
            retryMessage_Pre,
            false,
            false,
            ($core.List<$core.int> value) =>
                $0.RetryMessageRequest.fromBuffer(value),
            ($0.RetryMessageResponse value) => value.writeToBuffer()));
    $addMethod(
        $grpc.ServiceMethod<$0.DeleteMessageRequest, $0.DeleteMessageResponse>(
            'DeleteMessage',
            deleteMessage_Pre,
            false,
            false,
            ($core.List<$core.int> value) =>
                $0.DeleteMessageRequest.fromBuffer(value),
            ($0.DeleteMessageResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.RetryAllRequest, $0.RetryAllResponse>(
        'RetryAll',
        retryAll_Pre,
        false,
        false,
        ($core.List<$core.int> value) => $0.RetryAllRequest.fromBuffer(value),
        ($0.RetryAllResponse value) => value.writeToBuffer()));
  }

  $async.Future<$0.ListMessagesResponse> listMessages_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.ListMessagesRequest> $request) async {
    return listMessages($call, await $request);
  }

  $async.Future<$0.ListMessagesResponse> listMessages(
      $grpc.ServiceCall call, $0.ListMessagesRequest request);

  $async.Future<$0.GetMessageResponse> getMessage_Pre($grpc.ServiceCall $call,
      $async.Future<$0.GetMessageRequest> $request) async {
    return getMessage($call, await $request);
  }

  $async.Future<$0.GetMessageResponse> getMessage(
      $grpc.ServiceCall call, $0.GetMessageRequest request);

  $async.Future<$0.RetryMessageResponse> retryMessage_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.RetryMessageRequest> $request) async {
    return retryMessage($call, await $request);
  }

  $async.Future<$0.RetryMessageResponse> retryMessage(
      $grpc.ServiceCall call, $0.RetryMessageRequest request);

  $async.Future<$0.DeleteMessageResponse> deleteMessage_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.DeleteMessageRequest> $request) async {
    return deleteMessage($call, await $request);
  }

  $async.Future<$0.DeleteMessageResponse> deleteMessage(
      $grpc.ServiceCall call, $0.DeleteMessageRequest request);

  $async.Future<$0.RetryAllResponse> retryAll_Pre($grpc.ServiceCall $call,
      $async.Future<$0.RetryAllRequest> $request) async {
    return retryAll($call, await $request);
  }

  $async.Future<$0.RetryAllResponse> retryAll(
      $grpc.ServiceCall call, $0.RetryAllRequest request);
}
