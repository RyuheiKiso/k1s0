// This is a generated file - do not edit.
//
// Generated from k1s0/system/notification/v1/notification.proto.

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

import 'notification.pb.dart' as $0;

export 'notification.pb.dart';

@$pb.GrpcServiceName('k1s0.system.notification.v1.NotificationService')
class NotificationServiceClient extends $grpc.Client {
  /// The hostname for this service.
  static const $core.String defaultHost = '';

  /// OAuth scopes needed for the client.
  static const $core.List<$core.String> oauthScopes = [
    '',
  ];

  NotificationServiceClient(super.channel, {super.options, super.interceptors});

  $grpc.ResponseFuture<$0.SendNotificationResponse> sendNotification(
    $0.SendNotificationRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$sendNotification, request, options: options);
  }

  $grpc.ResponseFuture<$0.GetNotificationResponse> getNotification(
    $0.GetNotificationRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$getNotification, request, options: options);
  }

  $grpc.ResponseFuture<$0.RetryNotificationResponse> retryNotification(
    $0.RetryNotificationRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$retryNotification, request, options: options);
  }

  $grpc.ResponseFuture<$0.ListNotificationsResponse> listNotifications(
    $0.ListNotificationsRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$listNotifications, request, options: options);
  }

  $grpc.ResponseFuture<$0.ListChannelsResponse> listChannels(
    $0.ListChannelsRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$listChannels, request, options: options);
  }

  $grpc.ResponseFuture<$0.CreateChannelResponse> createChannel(
    $0.CreateChannelRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$createChannel, request, options: options);
  }

  $grpc.ResponseFuture<$0.GetChannelResponse> getChannel(
    $0.GetChannelRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$getChannel, request, options: options);
  }

  $grpc.ResponseFuture<$0.UpdateChannelResponse> updateChannel(
    $0.UpdateChannelRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$updateChannel, request, options: options);
  }

  $grpc.ResponseFuture<$0.DeleteChannelResponse> deleteChannel(
    $0.DeleteChannelRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$deleteChannel, request, options: options);
  }

  $grpc.ResponseFuture<$0.ListTemplatesResponse> listTemplates(
    $0.ListTemplatesRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$listTemplates, request, options: options);
  }

  $grpc.ResponseFuture<$0.CreateTemplateResponse> createTemplate(
    $0.CreateTemplateRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$createTemplate, request, options: options);
  }

  $grpc.ResponseFuture<$0.GetTemplateResponse> getTemplate(
    $0.GetTemplateRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$getTemplate, request, options: options);
  }

  $grpc.ResponseFuture<$0.UpdateTemplateResponse> updateTemplate(
    $0.UpdateTemplateRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$updateTemplate, request, options: options);
  }

  $grpc.ResponseFuture<$0.DeleteTemplateResponse> deleteTemplate(
    $0.DeleteTemplateRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$deleteTemplate, request, options: options);
  }

  // method descriptors

  static final _$sendNotification = $grpc.ClientMethod<
          $0.SendNotificationRequest, $0.SendNotificationResponse>(
      '/k1s0.system.notification.v1.NotificationService/SendNotification',
      ($0.SendNotificationRequest value) => value.writeToBuffer(),
      $0.SendNotificationResponse.fromBuffer);
  static final _$getNotification =
      $grpc.ClientMethod<$0.GetNotificationRequest, $0.GetNotificationResponse>(
          '/k1s0.system.notification.v1.NotificationService/GetNotification',
          ($0.GetNotificationRequest value) => value.writeToBuffer(),
          $0.GetNotificationResponse.fromBuffer);
  static final _$retryNotification = $grpc.ClientMethod<
          $0.RetryNotificationRequest, $0.RetryNotificationResponse>(
      '/k1s0.system.notification.v1.NotificationService/RetryNotification',
      ($0.RetryNotificationRequest value) => value.writeToBuffer(),
      $0.RetryNotificationResponse.fromBuffer);
  static final _$listNotifications = $grpc.ClientMethod<
          $0.ListNotificationsRequest, $0.ListNotificationsResponse>(
      '/k1s0.system.notification.v1.NotificationService/ListNotifications',
      ($0.ListNotificationsRequest value) => value.writeToBuffer(),
      $0.ListNotificationsResponse.fromBuffer);
  static final _$listChannels =
      $grpc.ClientMethod<$0.ListChannelsRequest, $0.ListChannelsResponse>(
          '/k1s0.system.notification.v1.NotificationService/ListChannels',
          ($0.ListChannelsRequest value) => value.writeToBuffer(),
          $0.ListChannelsResponse.fromBuffer);
  static final _$createChannel =
      $grpc.ClientMethod<$0.CreateChannelRequest, $0.CreateChannelResponse>(
          '/k1s0.system.notification.v1.NotificationService/CreateChannel',
          ($0.CreateChannelRequest value) => value.writeToBuffer(),
          $0.CreateChannelResponse.fromBuffer);
  static final _$getChannel =
      $grpc.ClientMethod<$0.GetChannelRequest, $0.GetChannelResponse>(
          '/k1s0.system.notification.v1.NotificationService/GetChannel',
          ($0.GetChannelRequest value) => value.writeToBuffer(),
          $0.GetChannelResponse.fromBuffer);
  static final _$updateChannel =
      $grpc.ClientMethod<$0.UpdateChannelRequest, $0.UpdateChannelResponse>(
          '/k1s0.system.notification.v1.NotificationService/UpdateChannel',
          ($0.UpdateChannelRequest value) => value.writeToBuffer(),
          $0.UpdateChannelResponse.fromBuffer);
  static final _$deleteChannel =
      $grpc.ClientMethod<$0.DeleteChannelRequest, $0.DeleteChannelResponse>(
          '/k1s0.system.notification.v1.NotificationService/DeleteChannel',
          ($0.DeleteChannelRequest value) => value.writeToBuffer(),
          $0.DeleteChannelResponse.fromBuffer);
  static final _$listTemplates =
      $grpc.ClientMethod<$0.ListTemplatesRequest, $0.ListTemplatesResponse>(
          '/k1s0.system.notification.v1.NotificationService/ListTemplates',
          ($0.ListTemplatesRequest value) => value.writeToBuffer(),
          $0.ListTemplatesResponse.fromBuffer);
  static final _$createTemplate =
      $grpc.ClientMethod<$0.CreateTemplateRequest, $0.CreateTemplateResponse>(
          '/k1s0.system.notification.v1.NotificationService/CreateTemplate',
          ($0.CreateTemplateRequest value) => value.writeToBuffer(),
          $0.CreateTemplateResponse.fromBuffer);
  static final _$getTemplate =
      $grpc.ClientMethod<$0.GetTemplateRequest, $0.GetTemplateResponse>(
          '/k1s0.system.notification.v1.NotificationService/GetTemplate',
          ($0.GetTemplateRequest value) => value.writeToBuffer(),
          $0.GetTemplateResponse.fromBuffer);
  static final _$updateTemplate =
      $grpc.ClientMethod<$0.UpdateTemplateRequest, $0.UpdateTemplateResponse>(
          '/k1s0.system.notification.v1.NotificationService/UpdateTemplate',
          ($0.UpdateTemplateRequest value) => value.writeToBuffer(),
          $0.UpdateTemplateResponse.fromBuffer);
  static final _$deleteTemplate =
      $grpc.ClientMethod<$0.DeleteTemplateRequest, $0.DeleteTemplateResponse>(
          '/k1s0.system.notification.v1.NotificationService/DeleteTemplate',
          ($0.DeleteTemplateRequest value) => value.writeToBuffer(),
          $0.DeleteTemplateResponse.fromBuffer);
}

@$pb.GrpcServiceName('k1s0.system.notification.v1.NotificationService')
abstract class NotificationServiceBase extends $grpc.Service {
  $core.String get $name => 'k1s0.system.notification.v1.NotificationService';

  NotificationServiceBase() {
    $addMethod($grpc.ServiceMethod<$0.SendNotificationRequest,
            $0.SendNotificationResponse>(
        'SendNotification',
        sendNotification_Pre,
        false,
        false,
        ($core.List<$core.int> value) =>
            $0.SendNotificationRequest.fromBuffer(value),
        ($0.SendNotificationResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.GetNotificationRequest,
            $0.GetNotificationResponse>(
        'GetNotification',
        getNotification_Pre,
        false,
        false,
        ($core.List<$core.int> value) =>
            $0.GetNotificationRequest.fromBuffer(value),
        ($0.GetNotificationResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.RetryNotificationRequest,
            $0.RetryNotificationResponse>(
        'RetryNotification',
        retryNotification_Pre,
        false,
        false,
        ($core.List<$core.int> value) =>
            $0.RetryNotificationRequest.fromBuffer(value),
        ($0.RetryNotificationResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.ListNotificationsRequest,
            $0.ListNotificationsResponse>(
        'ListNotifications',
        listNotifications_Pre,
        false,
        false,
        ($core.List<$core.int> value) =>
            $0.ListNotificationsRequest.fromBuffer(value),
        ($0.ListNotificationsResponse value) => value.writeToBuffer()));
    $addMethod(
        $grpc.ServiceMethod<$0.ListChannelsRequest, $0.ListChannelsResponse>(
            'ListChannels',
            listChannels_Pre,
            false,
            false,
            ($core.List<$core.int> value) =>
                $0.ListChannelsRequest.fromBuffer(value),
            ($0.ListChannelsResponse value) => value.writeToBuffer()));
    $addMethod(
        $grpc.ServiceMethod<$0.CreateChannelRequest, $0.CreateChannelResponse>(
            'CreateChannel',
            createChannel_Pre,
            false,
            false,
            ($core.List<$core.int> value) =>
                $0.CreateChannelRequest.fromBuffer(value),
            ($0.CreateChannelResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.GetChannelRequest, $0.GetChannelResponse>(
        'GetChannel',
        getChannel_Pre,
        false,
        false,
        ($core.List<$core.int> value) => $0.GetChannelRequest.fromBuffer(value),
        ($0.GetChannelResponse value) => value.writeToBuffer()));
    $addMethod(
        $grpc.ServiceMethod<$0.UpdateChannelRequest, $0.UpdateChannelResponse>(
            'UpdateChannel',
            updateChannel_Pre,
            false,
            false,
            ($core.List<$core.int> value) =>
                $0.UpdateChannelRequest.fromBuffer(value),
            ($0.UpdateChannelResponse value) => value.writeToBuffer()));
    $addMethod(
        $grpc.ServiceMethod<$0.DeleteChannelRequest, $0.DeleteChannelResponse>(
            'DeleteChannel',
            deleteChannel_Pre,
            false,
            false,
            ($core.List<$core.int> value) =>
                $0.DeleteChannelRequest.fromBuffer(value),
            ($0.DeleteChannelResponse value) => value.writeToBuffer()));
    $addMethod(
        $grpc.ServiceMethod<$0.ListTemplatesRequest, $0.ListTemplatesResponse>(
            'ListTemplates',
            listTemplates_Pre,
            false,
            false,
            ($core.List<$core.int> value) =>
                $0.ListTemplatesRequest.fromBuffer(value),
            ($0.ListTemplatesResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.CreateTemplateRequest,
            $0.CreateTemplateResponse>(
        'CreateTemplate',
        createTemplate_Pre,
        false,
        false,
        ($core.List<$core.int> value) =>
            $0.CreateTemplateRequest.fromBuffer(value),
        ($0.CreateTemplateResponse value) => value.writeToBuffer()));
    $addMethod(
        $grpc.ServiceMethod<$0.GetTemplateRequest, $0.GetTemplateResponse>(
            'GetTemplate',
            getTemplate_Pre,
            false,
            false,
            ($core.List<$core.int> value) =>
                $0.GetTemplateRequest.fromBuffer(value),
            ($0.GetTemplateResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.UpdateTemplateRequest,
            $0.UpdateTemplateResponse>(
        'UpdateTemplate',
        updateTemplate_Pre,
        false,
        false,
        ($core.List<$core.int> value) =>
            $0.UpdateTemplateRequest.fromBuffer(value),
        ($0.UpdateTemplateResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.DeleteTemplateRequest,
            $0.DeleteTemplateResponse>(
        'DeleteTemplate',
        deleteTemplate_Pre,
        false,
        false,
        ($core.List<$core.int> value) =>
            $0.DeleteTemplateRequest.fromBuffer(value),
        ($0.DeleteTemplateResponse value) => value.writeToBuffer()));
  }

  $async.Future<$0.SendNotificationResponse> sendNotification_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.SendNotificationRequest> $request) async {
    return sendNotification($call, await $request);
  }

  $async.Future<$0.SendNotificationResponse> sendNotification(
      $grpc.ServiceCall call, $0.SendNotificationRequest request);

  $async.Future<$0.GetNotificationResponse> getNotification_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.GetNotificationRequest> $request) async {
    return getNotification($call, await $request);
  }

  $async.Future<$0.GetNotificationResponse> getNotification(
      $grpc.ServiceCall call, $0.GetNotificationRequest request);

  $async.Future<$0.RetryNotificationResponse> retryNotification_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.RetryNotificationRequest> $request) async {
    return retryNotification($call, await $request);
  }

  $async.Future<$0.RetryNotificationResponse> retryNotification(
      $grpc.ServiceCall call, $0.RetryNotificationRequest request);

  $async.Future<$0.ListNotificationsResponse> listNotifications_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.ListNotificationsRequest> $request) async {
    return listNotifications($call, await $request);
  }

  $async.Future<$0.ListNotificationsResponse> listNotifications(
      $grpc.ServiceCall call, $0.ListNotificationsRequest request);

  $async.Future<$0.ListChannelsResponse> listChannels_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.ListChannelsRequest> $request) async {
    return listChannels($call, await $request);
  }

  $async.Future<$0.ListChannelsResponse> listChannels(
      $grpc.ServiceCall call, $0.ListChannelsRequest request);

  $async.Future<$0.CreateChannelResponse> createChannel_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.CreateChannelRequest> $request) async {
    return createChannel($call, await $request);
  }

  $async.Future<$0.CreateChannelResponse> createChannel(
      $grpc.ServiceCall call, $0.CreateChannelRequest request);

  $async.Future<$0.GetChannelResponse> getChannel_Pre($grpc.ServiceCall $call,
      $async.Future<$0.GetChannelRequest> $request) async {
    return getChannel($call, await $request);
  }

  $async.Future<$0.GetChannelResponse> getChannel(
      $grpc.ServiceCall call, $0.GetChannelRequest request);

  $async.Future<$0.UpdateChannelResponse> updateChannel_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.UpdateChannelRequest> $request) async {
    return updateChannel($call, await $request);
  }

  $async.Future<$0.UpdateChannelResponse> updateChannel(
      $grpc.ServiceCall call, $0.UpdateChannelRequest request);

  $async.Future<$0.DeleteChannelResponse> deleteChannel_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.DeleteChannelRequest> $request) async {
    return deleteChannel($call, await $request);
  }

  $async.Future<$0.DeleteChannelResponse> deleteChannel(
      $grpc.ServiceCall call, $0.DeleteChannelRequest request);

  $async.Future<$0.ListTemplatesResponse> listTemplates_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.ListTemplatesRequest> $request) async {
    return listTemplates($call, await $request);
  }

  $async.Future<$0.ListTemplatesResponse> listTemplates(
      $grpc.ServiceCall call, $0.ListTemplatesRequest request);

  $async.Future<$0.CreateTemplateResponse> createTemplate_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.CreateTemplateRequest> $request) async {
    return createTemplate($call, await $request);
  }

  $async.Future<$0.CreateTemplateResponse> createTemplate(
      $grpc.ServiceCall call, $0.CreateTemplateRequest request);

  $async.Future<$0.GetTemplateResponse> getTemplate_Pre($grpc.ServiceCall $call,
      $async.Future<$0.GetTemplateRequest> $request) async {
    return getTemplate($call, await $request);
  }

  $async.Future<$0.GetTemplateResponse> getTemplate(
      $grpc.ServiceCall call, $0.GetTemplateRequest request);

  $async.Future<$0.UpdateTemplateResponse> updateTemplate_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.UpdateTemplateRequest> $request) async {
    return updateTemplate($call, await $request);
  }

  $async.Future<$0.UpdateTemplateResponse> updateTemplate(
      $grpc.ServiceCall call, $0.UpdateTemplateRequest request);

  $async.Future<$0.DeleteTemplateResponse> deleteTemplate_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.DeleteTemplateRequest> $request) async {
    return deleteTemplate($call, await $request);
  }

  $async.Future<$0.DeleteTemplateResponse> deleteTemplate(
      $grpc.ServiceCall call, $0.DeleteTemplateRequest request);
}
