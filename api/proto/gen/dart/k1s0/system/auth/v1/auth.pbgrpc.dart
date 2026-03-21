// This is a generated file - do not edit.
//
// Generated from k1s0/system/auth/v1/auth.proto.

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

import 'auth.pb.dart' as $0;

export 'auth.pb.dart';

/// AuthService は JWT トークン検証・ユーザー情報管理・パーミッション確認を提供する。
@$pb.GrpcServiceName('k1s0.system.auth.v1.AuthService')
class AuthServiceClient extends $grpc.Client {
  /// The hostname for this service.
  static const $core.String defaultHost = '';

  /// OAuth scopes needed for the client.
  static const $core.List<$core.String> oauthScopes = [
    '',
  ];

  AuthServiceClient(super.channel, {super.options, super.interceptors});

  /// JWT トークン検証
  $grpc.ResponseFuture<$0.ValidateTokenResponse> validateToken(
    $0.ValidateTokenRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$validateToken, request, options: options);
  }

  /// ユーザー情報取得
  $grpc.ResponseFuture<$0.GetUserResponse> getUser(
    $0.GetUserRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$getUser, request, options: options);
  }

  /// ユーザー一覧取得
  $grpc.ResponseFuture<$0.ListUsersResponse> listUsers(
    $0.ListUsersRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$listUsers, request, options: options);
  }

  /// ユーザーロール取得
  $grpc.ResponseFuture<$0.GetUserRolesResponse> getUserRoles(
    $0.GetUserRolesRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$getUserRoles, request, options: options);
  }

  /// パーミッション確認
  $grpc.ResponseFuture<$0.CheckPermissionResponse> checkPermission(
    $0.CheckPermissionRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$checkPermission, request, options: options);
  }

  // method descriptors

  static final _$validateToken =
      $grpc.ClientMethod<$0.ValidateTokenRequest, $0.ValidateTokenResponse>(
          '/k1s0.system.auth.v1.AuthService/ValidateToken',
          ($0.ValidateTokenRequest value) => value.writeToBuffer(),
          $0.ValidateTokenResponse.fromBuffer);
  static final _$getUser =
      $grpc.ClientMethod<$0.GetUserRequest, $0.GetUserResponse>(
          '/k1s0.system.auth.v1.AuthService/GetUser',
          ($0.GetUserRequest value) => value.writeToBuffer(),
          $0.GetUserResponse.fromBuffer);
  static final _$listUsers =
      $grpc.ClientMethod<$0.ListUsersRequest, $0.ListUsersResponse>(
          '/k1s0.system.auth.v1.AuthService/ListUsers',
          ($0.ListUsersRequest value) => value.writeToBuffer(),
          $0.ListUsersResponse.fromBuffer);
  static final _$getUserRoles =
      $grpc.ClientMethod<$0.GetUserRolesRequest, $0.GetUserRolesResponse>(
          '/k1s0.system.auth.v1.AuthService/GetUserRoles',
          ($0.GetUserRolesRequest value) => value.writeToBuffer(),
          $0.GetUserRolesResponse.fromBuffer);
  static final _$checkPermission =
      $grpc.ClientMethod<$0.CheckPermissionRequest, $0.CheckPermissionResponse>(
          '/k1s0.system.auth.v1.AuthService/CheckPermission',
          ($0.CheckPermissionRequest value) => value.writeToBuffer(),
          $0.CheckPermissionResponse.fromBuffer);
}

@$pb.GrpcServiceName('k1s0.system.auth.v1.AuthService')
abstract class AuthServiceBase extends $grpc.Service {
  $core.String get $name => 'k1s0.system.auth.v1.AuthService';

  AuthServiceBase() {
    $addMethod(
        $grpc.ServiceMethod<$0.ValidateTokenRequest, $0.ValidateTokenResponse>(
            'ValidateToken',
            validateToken_Pre,
            false,
            false,
            ($core.List<$core.int> value) =>
                $0.ValidateTokenRequest.fromBuffer(value),
            ($0.ValidateTokenResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.GetUserRequest, $0.GetUserResponse>(
        'GetUser',
        getUser_Pre,
        false,
        false,
        ($core.List<$core.int> value) => $0.GetUserRequest.fromBuffer(value),
        ($0.GetUserResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.ListUsersRequest, $0.ListUsersResponse>(
        'ListUsers',
        listUsers_Pre,
        false,
        false,
        ($core.List<$core.int> value) => $0.ListUsersRequest.fromBuffer(value),
        ($0.ListUsersResponse value) => value.writeToBuffer()));
    $addMethod(
        $grpc.ServiceMethod<$0.GetUserRolesRequest, $0.GetUserRolesResponse>(
            'GetUserRoles',
            getUserRoles_Pre,
            false,
            false,
            ($core.List<$core.int> value) =>
                $0.GetUserRolesRequest.fromBuffer(value),
            ($0.GetUserRolesResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.CheckPermissionRequest,
            $0.CheckPermissionResponse>(
        'CheckPermission',
        checkPermission_Pre,
        false,
        false,
        ($core.List<$core.int> value) =>
            $0.CheckPermissionRequest.fromBuffer(value),
        ($0.CheckPermissionResponse value) => value.writeToBuffer()));
  }

  $async.Future<$0.ValidateTokenResponse> validateToken_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.ValidateTokenRequest> $request) async {
    return validateToken($call, await $request);
  }

  $async.Future<$0.ValidateTokenResponse> validateToken(
      $grpc.ServiceCall call, $0.ValidateTokenRequest request);

  $async.Future<$0.GetUserResponse> getUser_Pre($grpc.ServiceCall $call,
      $async.Future<$0.GetUserRequest> $request) async {
    return getUser($call, await $request);
  }

  $async.Future<$0.GetUserResponse> getUser(
      $grpc.ServiceCall call, $0.GetUserRequest request);

  $async.Future<$0.ListUsersResponse> listUsers_Pre($grpc.ServiceCall $call,
      $async.Future<$0.ListUsersRequest> $request) async {
    return listUsers($call, await $request);
  }

  $async.Future<$0.ListUsersResponse> listUsers(
      $grpc.ServiceCall call, $0.ListUsersRequest request);

  $async.Future<$0.GetUserRolesResponse> getUserRoles_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.GetUserRolesRequest> $request) async {
    return getUserRoles($call, await $request);
  }

  $async.Future<$0.GetUserRolesResponse> getUserRoles(
      $grpc.ServiceCall call, $0.GetUserRolesRequest request);

  $async.Future<$0.CheckPermissionResponse> checkPermission_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.CheckPermissionRequest> $request) async {
    return checkPermission($call, await $request);
  }

  $async.Future<$0.CheckPermissionResponse> checkPermission(
      $grpc.ServiceCall call, $0.CheckPermissionRequest request);
}

/// AuditService は監査ログの記録・検索を提供する。
@$pb.GrpcServiceName('k1s0.system.auth.v1.AuditService')
class AuditServiceClient extends $grpc.Client {
  /// The hostname for this service.
  static const $core.String defaultHost = '';

  /// OAuth scopes needed for the client.
  static const $core.List<$core.String> oauthScopes = [
    '',
  ];

  AuditServiceClient(super.channel, {super.options, super.interceptors});

  /// 監査ログ記録
  $grpc.ResponseFuture<$0.RecordAuditLogResponse> recordAuditLog(
    $0.RecordAuditLogRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$recordAuditLog, request, options: options);
  }

  /// 監査ログ検索
  $grpc.ResponseFuture<$0.SearchAuditLogsResponse> searchAuditLogs(
    $0.SearchAuditLogsRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$searchAuditLogs, request, options: options);
  }

  // method descriptors

  static final _$recordAuditLog =
      $grpc.ClientMethod<$0.RecordAuditLogRequest, $0.RecordAuditLogResponse>(
          '/k1s0.system.auth.v1.AuditService/RecordAuditLog',
          ($0.RecordAuditLogRequest value) => value.writeToBuffer(),
          $0.RecordAuditLogResponse.fromBuffer);
  static final _$searchAuditLogs =
      $grpc.ClientMethod<$0.SearchAuditLogsRequest, $0.SearchAuditLogsResponse>(
          '/k1s0.system.auth.v1.AuditService/SearchAuditLogs',
          ($0.SearchAuditLogsRequest value) => value.writeToBuffer(),
          $0.SearchAuditLogsResponse.fromBuffer);
}

@$pb.GrpcServiceName('k1s0.system.auth.v1.AuditService')
abstract class AuditServiceBase extends $grpc.Service {
  $core.String get $name => 'k1s0.system.auth.v1.AuditService';

  AuditServiceBase() {
    $addMethod($grpc.ServiceMethod<$0.RecordAuditLogRequest,
            $0.RecordAuditLogResponse>(
        'RecordAuditLog',
        recordAuditLog_Pre,
        false,
        false,
        ($core.List<$core.int> value) =>
            $0.RecordAuditLogRequest.fromBuffer(value),
        ($0.RecordAuditLogResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.SearchAuditLogsRequest,
            $0.SearchAuditLogsResponse>(
        'SearchAuditLogs',
        searchAuditLogs_Pre,
        false,
        false,
        ($core.List<$core.int> value) =>
            $0.SearchAuditLogsRequest.fromBuffer(value),
        ($0.SearchAuditLogsResponse value) => value.writeToBuffer()));
  }

  $async.Future<$0.RecordAuditLogResponse> recordAuditLog_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.RecordAuditLogRequest> $request) async {
    return recordAuditLog($call, await $request);
  }

  $async.Future<$0.RecordAuditLogResponse> recordAuditLog(
      $grpc.ServiceCall call, $0.RecordAuditLogRequest request);

  $async.Future<$0.SearchAuditLogsResponse> searchAuditLogs_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.SearchAuditLogsRequest> $request) async {
    return searchAuditLogs($call, await $request);
  }

  $async.Future<$0.SearchAuditLogsResponse> searchAuditLogs(
      $grpc.ServiceCall call, $0.SearchAuditLogsRequest request);
}
