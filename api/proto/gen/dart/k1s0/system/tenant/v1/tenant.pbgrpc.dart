// This is a generated file - do not edit.
//
// Generated from k1s0/system/tenant/v1/tenant.proto.

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

import 'tenant.pb.dart' as $0;

export 'tenant.pb.dart';

/// TenantService はマルチテナント管理サービス。
/// Keycloak realm プロビジョニングおよびメンバー管理を担当する。
@$pb.GrpcServiceName('k1s0.system.tenant.v1.TenantService')
class TenantServiceClient extends $grpc.Client {
  /// The hostname for this service.
  static const $core.String defaultHost = '';

  /// OAuth scopes needed for the client.
  static const $core.List<$core.String> oauthScopes = [
    '',
  ];

  TenantServiceClient(super.channel, {super.options, super.interceptors});

  /// CreateTenant は新しいテナントを作成し、Keycloak realm をプロビジョニングする。
  $grpc.ResponseFuture<$0.CreateTenantResponse> createTenant(
    $0.CreateTenantRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$createTenant, request, options: options);
  }

  /// GetTenant はテナントIDでテナント情報を取得する。
  $grpc.ResponseFuture<$0.GetTenantResponse> getTenant(
    $0.GetTenantRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$getTenant, request, options: options);
  }

  /// ListTenants は全テナントの一覧をページネーション付きで返す。
  $grpc.ResponseFuture<$0.ListTenantsResponse> listTenants(
    $0.ListTenantsRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$listTenants, request, options: options);
  }

  /// UpdateTenant はテナント情報を更新する。
  $grpc.ResponseFuture<$0.UpdateTenantResponse> updateTenant(
    $0.UpdateTenantRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$updateTenant, request, options: options);
  }

  /// SuspendTenant はアクティブなテナントを停止する。
  $grpc.ResponseFuture<$0.SuspendTenantResponse> suspendTenant(
    $0.SuspendTenantRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$suspendTenant, request, options: options);
  }

  /// ActivateTenant は停止中のテナントを再開する。
  $grpc.ResponseFuture<$0.ActivateTenantResponse> activateTenant(
    $0.ActivateTenantRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$activateTenant, request, options: options);
  }

  /// DeleteTenant はテナントを論理削除する。
  $grpc.ResponseFuture<$0.DeleteTenantResponse> deleteTenant(
    $0.DeleteTenantRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$deleteTenant, request, options: options);
  }

  /// AddMember はテナントにメンバーを追加する。
  $grpc.ResponseFuture<$0.AddMemberResponse> addMember(
    $0.AddMemberRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$addMember, request, options: options);
  }

  /// ListMembers はテナントのメンバー一覧を取得する。
  $grpc.ResponseFuture<$0.ListMembersResponse> listMembers(
    $0.ListMembersRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$listMembers, request, options: options);
  }

  /// RemoveMember はテナントからメンバーを削除する。
  $grpc.ResponseFuture<$0.RemoveMemberResponse> removeMember(
    $0.RemoveMemberRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$removeMember, request, options: options);
  }

  /// GetProvisioningStatus はテナントプロビジョニングジョブのステータスを返す。
  $grpc.ResponseFuture<$0.GetProvisioningStatusResponse> getProvisioningStatus(
    $0.GetProvisioningStatusRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$getProvisioningStatus, request, options: options);
  }

  /// WatchTenant はテナント変更の監視（Server-Side Streaming）。
  $grpc.ResponseStream<$0.WatchTenantResponse> watchTenant(
    $0.WatchTenantRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createStreamingCall(
        _$watchTenant, $async.Stream.fromIterable([request]),
        options: options);
  }

  // method descriptors

  static final _$createTenant =
      $grpc.ClientMethod<$0.CreateTenantRequest, $0.CreateTenantResponse>(
          '/k1s0.system.tenant.v1.TenantService/CreateTenant',
          ($0.CreateTenantRequest value) => value.writeToBuffer(),
          $0.CreateTenantResponse.fromBuffer);
  static final _$getTenant =
      $grpc.ClientMethod<$0.GetTenantRequest, $0.GetTenantResponse>(
          '/k1s0.system.tenant.v1.TenantService/GetTenant',
          ($0.GetTenantRequest value) => value.writeToBuffer(),
          $0.GetTenantResponse.fromBuffer);
  static final _$listTenants =
      $grpc.ClientMethod<$0.ListTenantsRequest, $0.ListTenantsResponse>(
          '/k1s0.system.tenant.v1.TenantService/ListTenants',
          ($0.ListTenantsRequest value) => value.writeToBuffer(),
          $0.ListTenantsResponse.fromBuffer);
  static final _$updateTenant =
      $grpc.ClientMethod<$0.UpdateTenantRequest, $0.UpdateTenantResponse>(
          '/k1s0.system.tenant.v1.TenantService/UpdateTenant',
          ($0.UpdateTenantRequest value) => value.writeToBuffer(),
          $0.UpdateTenantResponse.fromBuffer);
  static final _$suspendTenant =
      $grpc.ClientMethod<$0.SuspendTenantRequest, $0.SuspendTenantResponse>(
          '/k1s0.system.tenant.v1.TenantService/SuspendTenant',
          ($0.SuspendTenantRequest value) => value.writeToBuffer(),
          $0.SuspendTenantResponse.fromBuffer);
  static final _$activateTenant =
      $grpc.ClientMethod<$0.ActivateTenantRequest, $0.ActivateTenantResponse>(
          '/k1s0.system.tenant.v1.TenantService/ActivateTenant',
          ($0.ActivateTenantRequest value) => value.writeToBuffer(),
          $0.ActivateTenantResponse.fromBuffer);
  static final _$deleteTenant =
      $grpc.ClientMethod<$0.DeleteTenantRequest, $0.DeleteTenantResponse>(
          '/k1s0.system.tenant.v1.TenantService/DeleteTenant',
          ($0.DeleteTenantRequest value) => value.writeToBuffer(),
          $0.DeleteTenantResponse.fromBuffer);
  static final _$addMember =
      $grpc.ClientMethod<$0.AddMemberRequest, $0.AddMemberResponse>(
          '/k1s0.system.tenant.v1.TenantService/AddMember',
          ($0.AddMemberRequest value) => value.writeToBuffer(),
          $0.AddMemberResponse.fromBuffer);
  static final _$listMembers =
      $grpc.ClientMethod<$0.ListMembersRequest, $0.ListMembersResponse>(
          '/k1s0.system.tenant.v1.TenantService/ListMembers',
          ($0.ListMembersRequest value) => value.writeToBuffer(),
          $0.ListMembersResponse.fromBuffer);
  static final _$removeMember =
      $grpc.ClientMethod<$0.RemoveMemberRequest, $0.RemoveMemberResponse>(
          '/k1s0.system.tenant.v1.TenantService/RemoveMember',
          ($0.RemoveMemberRequest value) => value.writeToBuffer(),
          $0.RemoveMemberResponse.fromBuffer);
  static final _$getProvisioningStatus = $grpc.ClientMethod<
          $0.GetProvisioningStatusRequest, $0.GetProvisioningStatusResponse>(
      '/k1s0.system.tenant.v1.TenantService/GetProvisioningStatus',
      ($0.GetProvisioningStatusRequest value) => value.writeToBuffer(),
      $0.GetProvisioningStatusResponse.fromBuffer);
  static final _$watchTenant =
      $grpc.ClientMethod<$0.WatchTenantRequest, $0.WatchTenantResponse>(
          '/k1s0.system.tenant.v1.TenantService/WatchTenant',
          ($0.WatchTenantRequest value) => value.writeToBuffer(),
          $0.WatchTenantResponse.fromBuffer);
}

@$pb.GrpcServiceName('k1s0.system.tenant.v1.TenantService')
abstract class TenantServiceBase extends $grpc.Service {
  $core.String get $name => 'k1s0.system.tenant.v1.TenantService';

  TenantServiceBase() {
    $addMethod(
        $grpc.ServiceMethod<$0.CreateTenantRequest, $0.CreateTenantResponse>(
            'CreateTenant',
            createTenant_Pre,
            false,
            false,
            ($core.List<$core.int> value) =>
                $0.CreateTenantRequest.fromBuffer(value),
            ($0.CreateTenantResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.GetTenantRequest, $0.GetTenantResponse>(
        'GetTenant',
        getTenant_Pre,
        false,
        false,
        ($core.List<$core.int> value) => $0.GetTenantRequest.fromBuffer(value),
        ($0.GetTenantResponse value) => value.writeToBuffer()));
    $addMethod(
        $grpc.ServiceMethod<$0.ListTenantsRequest, $0.ListTenantsResponse>(
            'ListTenants',
            listTenants_Pre,
            false,
            false,
            ($core.List<$core.int> value) =>
                $0.ListTenantsRequest.fromBuffer(value),
            ($0.ListTenantsResponse value) => value.writeToBuffer()));
    $addMethod(
        $grpc.ServiceMethod<$0.UpdateTenantRequest, $0.UpdateTenantResponse>(
            'UpdateTenant',
            updateTenant_Pre,
            false,
            false,
            ($core.List<$core.int> value) =>
                $0.UpdateTenantRequest.fromBuffer(value),
            ($0.UpdateTenantResponse value) => value.writeToBuffer()));
    $addMethod(
        $grpc.ServiceMethod<$0.SuspendTenantRequest, $0.SuspendTenantResponse>(
            'SuspendTenant',
            suspendTenant_Pre,
            false,
            false,
            ($core.List<$core.int> value) =>
                $0.SuspendTenantRequest.fromBuffer(value),
            ($0.SuspendTenantResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.ActivateTenantRequest,
            $0.ActivateTenantResponse>(
        'ActivateTenant',
        activateTenant_Pre,
        false,
        false,
        ($core.List<$core.int> value) =>
            $0.ActivateTenantRequest.fromBuffer(value),
        ($0.ActivateTenantResponse value) => value.writeToBuffer()));
    $addMethod(
        $grpc.ServiceMethod<$0.DeleteTenantRequest, $0.DeleteTenantResponse>(
            'DeleteTenant',
            deleteTenant_Pre,
            false,
            false,
            ($core.List<$core.int> value) =>
                $0.DeleteTenantRequest.fromBuffer(value),
            ($0.DeleteTenantResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.AddMemberRequest, $0.AddMemberResponse>(
        'AddMember',
        addMember_Pre,
        false,
        false,
        ($core.List<$core.int> value) => $0.AddMemberRequest.fromBuffer(value),
        ($0.AddMemberResponse value) => value.writeToBuffer()));
    $addMethod(
        $grpc.ServiceMethod<$0.ListMembersRequest, $0.ListMembersResponse>(
            'ListMembers',
            listMembers_Pre,
            false,
            false,
            ($core.List<$core.int> value) =>
                $0.ListMembersRequest.fromBuffer(value),
            ($0.ListMembersResponse value) => value.writeToBuffer()));
    $addMethod(
        $grpc.ServiceMethod<$0.RemoveMemberRequest, $0.RemoveMemberResponse>(
            'RemoveMember',
            removeMember_Pre,
            false,
            false,
            ($core.List<$core.int> value) =>
                $0.RemoveMemberRequest.fromBuffer(value),
            ($0.RemoveMemberResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.GetProvisioningStatusRequest,
            $0.GetProvisioningStatusResponse>(
        'GetProvisioningStatus',
        getProvisioningStatus_Pre,
        false,
        false,
        ($core.List<$core.int> value) =>
            $0.GetProvisioningStatusRequest.fromBuffer(value),
        ($0.GetProvisioningStatusResponse value) => value.writeToBuffer()));
    $addMethod(
        $grpc.ServiceMethod<$0.WatchTenantRequest, $0.WatchTenantResponse>(
            'WatchTenant',
            watchTenant_Pre,
            false,
            true,
            ($core.List<$core.int> value) =>
                $0.WatchTenantRequest.fromBuffer(value),
            ($0.WatchTenantResponse value) => value.writeToBuffer()));
  }

  $async.Future<$0.CreateTenantResponse> createTenant_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.CreateTenantRequest> $request) async {
    return createTenant($call, await $request);
  }

  $async.Future<$0.CreateTenantResponse> createTenant(
      $grpc.ServiceCall call, $0.CreateTenantRequest request);

  $async.Future<$0.GetTenantResponse> getTenant_Pre($grpc.ServiceCall $call,
      $async.Future<$0.GetTenantRequest> $request) async {
    return getTenant($call, await $request);
  }

  $async.Future<$0.GetTenantResponse> getTenant(
      $grpc.ServiceCall call, $0.GetTenantRequest request);

  $async.Future<$0.ListTenantsResponse> listTenants_Pre($grpc.ServiceCall $call,
      $async.Future<$0.ListTenantsRequest> $request) async {
    return listTenants($call, await $request);
  }

  $async.Future<$0.ListTenantsResponse> listTenants(
      $grpc.ServiceCall call, $0.ListTenantsRequest request);

  $async.Future<$0.UpdateTenantResponse> updateTenant_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.UpdateTenantRequest> $request) async {
    return updateTenant($call, await $request);
  }

  $async.Future<$0.UpdateTenantResponse> updateTenant(
      $grpc.ServiceCall call, $0.UpdateTenantRequest request);

  $async.Future<$0.SuspendTenantResponse> suspendTenant_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.SuspendTenantRequest> $request) async {
    return suspendTenant($call, await $request);
  }

  $async.Future<$0.SuspendTenantResponse> suspendTenant(
      $grpc.ServiceCall call, $0.SuspendTenantRequest request);

  $async.Future<$0.ActivateTenantResponse> activateTenant_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.ActivateTenantRequest> $request) async {
    return activateTenant($call, await $request);
  }

  $async.Future<$0.ActivateTenantResponse> activateTenant(
      $grpc.ServiceCall call, $0.ActivateTenantRequest request);

  $async.Future<$0.DeleteTenantResponse> deleteTenant_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.DeleteTenantRequest> $request) async {
    return deleteTenant($call, await $request);
  }

  $async.Future<$0.DeleteTenantResponse> deleteTenant(
      $grpc.ServiceCall call, $0.DeleteTenantRequest request);

  $async.Future<$0.AddMemberResponse> addMember_Pre($grpc.ServiceCall $call,
      $async.Future<$0.AddMemberRequest> $request) async {
    return addMember($call, await $request);
  }

  $async.Future<$0.AddMemberResponse> addMember(
      $grpc.ServiceCall call, $0.AddMemberRequest request);

  $async.Future<$0.ListMembersResponse> listMembers_Pre($grpc.ServiceCall $call,
      $async.Future<$0.ListMembersRequest> $request) async {
    return listMembers($call, await $request);
  }

  $async.Future<$0.ListMembersResponse> listMembers(
      $grpc.ServiceCall call, $0.ListMembersRequest request);

  $async.Future<$0.RemoveMemberResponse> removeMember_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.RemoveMemberRequest> $request) async {
    return removeMember($call, await $request);
  }

  $async.Future<$0.RemoveMemberResponse> removeMember(
      $grpc.ServiceCall call, $0.RemoveMemberRequest request);

  $async.Future<$0.GetProvisioningStatusResponse> getProvisioningStatus_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.GetProvisioningStatusRequest> $request) async {
    return getProvisioningStatus($call, await $request);
  }

  $async.Future<$0.GetProvisioningStatusResponse> getProvisioningStatus(
      $grpc.ServiceCall call, $0.GetProvisioningStatusRequest request);

  $async.Stream<$0.WatchTenantResponse> watchTenant_Pre($grpc.ServiceCall $call,
      $async.Future<$0.WatchTenantRequest> $request) async* {
    yield* watchTenant($call, await $request);
  }

  $async.Stream<$0.WatchTenantResponse> watchTenant(
      $grpc.ServiceCall call, $0.WatchTenantRequest request);
}
