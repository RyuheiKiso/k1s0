// This is a generated file - do not edit.
//
// Generated from k1s0/system/session/v1/session.proto.

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

import 'session.pb.dart' as $0;

export 'session.pb.dart';

@$pb.GrpcServiceName('k1s0.system.session.v1.SessionService')
class SessionServiceClient extends $grpc.Client {
  /// The hostname for this service.
  static const $core.String defaultHost = '';

  /// OAuth scopes needed for the client.
  static const $core.List<$core.String> oauthScopes = [
    '',
  ];

  SessionServiceClient(super.channel, {super.options, super.interceptors});

  $grpc.ResponseFuture<$0.CreateSessionResponse> createSession(
    $0.CreateSessionRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$createSession, request, options: options);
  }

  $grpc.ResponseFuture<$0.GetSessionResponse> getSession(
    $0.GetSessionRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$getSession, request, options: options);
  }

  $grpc.ResponseFuture<$0.RefreshSessionResponse> refreshSession(
    $0.RefreshSessionRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$refreshSession, request, options: options);
  }

  $grpc.ResponseFuture<$0.RevokeSessionResponse> revokeSession(
    $0.RevokeSessionRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$revokeSession, request, options: options);
  }

  $grpc.ResponseFuture<$0.RevokeAllSessionsResponse> revokeAllSessions(
    $0.RevokeAllSessionsRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$revokeAllSessions, request, options: options);
  }

  $grpc.ResponseFuture<$0.ListUserSessionsResponse> listUserSessions(
    $0.ListUserSessionsRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$listUserSessions, request, options: options);
  }

  // method descriptors

  static final _$createSession =
      $grpc.ClientMethod<$0.CreateSessionRequest, $0.CreateSessionResponse>(
          '/k1s0.system.session.v1.SessionService/CreateSession',
          ($0.CreateSessionRequest value) => value.writeToBuffer(),
          $0.CreateSessionResponse.fromBuffer);
  static final _$getSession =
      $grpc.ClientMethod<$0.GetSessionRequest, $0.GetSessionResponse>(
          '/k1s0.system.session.v1.SessionService/GetSession',
          ($0.GetSessionRequest value) => value.writeToBuffer(),
          $0.GetSessionResponse.fromBuffer);
  static final _$refreshSession =
      $grpc.ClientMethod<$0.RefreshSessionRequest, $0.RefreshSessionResponse>(
          '/k1s0.system.session.v1.SessionService/RefreshSession',
          ($0.RefreshSessionRequest value) => value.writeToBuffer(),
          $0.RefreshSessionResponse.fromBuffer);
  static final _$revokeSession =
      $grpc.ClientMethod<$0.RevokeSessionRequest, $0.RevokeSessionResponse>(
          '/k1s0.system.session.v1.SessionService/RevokeSession',
          ($0.RevokeSessionRequest value) => value.writeToBuffer(),
          $0.RevokeSessionResponse.fromBuffer);
  static final _$revokeAllSessions = $grpc.ClientMethod<
          $0.RevokeAllSessionsRequest, $0.RevokeAllSessionsResponse>(
      '/k1s0.system.session.v1.SessionService/RevokeAllSessions',
      ($0.RevokeAllSessionsRequest value) => value.writeToBuffer(),
      $0.RevokeAllSessionsResponse.fromBuffer);
  static final _$listUserSessions = $grpc.ClientMethod<
          $0.ListUserSessionsRequest, $0.ListUserSessionsResponse>(
      '/k1s0.system.session.v1.SessionService/ListUserSessions',
      ($0.ListUserSessionsRequest value) => value.writeToBuffer(),
      $0.ListUserSessionsResponse.fromBuffer);
}

@$pb.GrpcServiceName('k1s0.system.session.v1.SessionService')
abstract class SessionServiceBase extends $grpc.Service {
  $core.String get $name => 'k1s0.system.session.v1.SessionService';

  SessionServiceBase() {
    $addMethod(
        $grpc.ServiceMethod<$0.CreateSessionRequest, $0.CreateSessionResponse>(
            'CreateSession',
            createSession_Pre,
            false,
            false,
            ($core.List<$core.int> value) =>
                $0.CreateSessionRequest.fromBuffer(value),
            ($0.CreateSessionResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.GetSessionRequest, $0.GetSessionResponse>(
        'GetSession',
        getSession_Pre,
        false,
        false,
        ($core.List<$core.int> value) => $0.GetSessionRequest.fromBuffer(value),
        ($0.GetSessionResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.RefreshSessionRequest,
            $0.RefreshSessionResponse>(
        'RefreshSession',
        refreshSession_Pre,
        false,
        false,
        ($core.List<$core.int> value) =>
            $0.RefreshSessionRequest.fromBuffer(value),
        ($0.RefreshSessionResponse value) => value.writeToBuffer()));
    $addMethod(
        $grpc.ServiceMethod<$0.RevokeSessionRequest, $0.RevokeSessionResponse>(
            'RevokeSession',
            revokeSession_Pre,
            false,
            false,
            ($core.List<$core.int> value) =>
                $0.RevokeSessionRequest.fromBuffer(value),
            ($0.RevokeSessionResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.RevokeAllSessionsRequest,
            $0.RevokeAllSessionsResponse>(
        'RevokeAllSessions',
        revokeAllSessions_Pre,
        false,
        false,
        ($core.List<$core.int> value) =>
            $0.RevokeAllSessionsRequest.fromBuffer(value),
        ($0.RevokeAllSessionsResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.ListUserSessionsRequest,
            $0.ListUserSessionsResponse>(
        'ListUserSessions',
        listUserSessions_Pre,
        false,
        false,
        ($core.List<$core.int> value) =>
            $0.ListUserSessionsRequest.fromBuffer(value),
        ($0.ListUserSessionsResponse value) => value.writeToBuffer()));
  }

  $async.Future<$0.CreateSessionResponse> createSession_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.CreateSessionRequest> $request) async {
    return createSession($call, await $request);
  }

  $async.Future<$0.CreateSessionResponse> createSession(
      $grpc.ServiceCall call, $0.CreateSessionRequest request);

  $async.Future<$0.GetSessionResponse> getSession_Pre($grpc.ServiceCall $call,
      $async.Future<$0.GetSessionRequest> $request) async {
    return getSession($call, await $request);
  }

  $async.Future<$0.GetSessionResponse> getSession(
      $grpc.ServiceCall call, $0.GetSessionRequest request);

  $async.Future<$0.RefreshSessionResponse> refreshSession_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.RefreshSessionRequest> $request) async {
    return refreshSession($call, await $request);
  }

  $async.Future<$0.RefreshSessionResponse> refreshSession(
      $grpc.ServiceCall call, $0.RefreshSessionRequest request);

  $async.Future<$0.RevokeSessionResponse> revokeSession_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.RevokeSessionRequest> $request) async {
    return revokeSession($call, await $request);
  }

  $async.Future<$0.RevokeSessionResponse> revokeSession(
      $grpc.ServiceCall call, $0.RevokeSessionRequest request);

  $async.Future<$0.RevokeAllSessionsResponse> revokeAllSessions_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.RevokeAllSessionsRequest> $request) async {
    return revokeAllSessions($call, await $request);
  }

  $async.Future<$0.RevokeAllSessionsResponse> revokeAllSessions(
      $grpc.ServiceCall call, $0.RevokeAllSessionsRequest request);

  $async.Future<$0.ListUserSessionsResponse> listUserSessions_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.ListUserSessionsRequest> $request) async {
    return listUserSessions($call, await $request);
  }

  $async.Future<$0.ListUserSessionsResponse> listUserSessions(
      $grpc.ServiceCall call, $0.ListUserSessionsRequest request);
}
