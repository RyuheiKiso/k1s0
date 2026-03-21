// This is a generated file - do not edit.
//
// Generated from k1s0/system/vault/v1/vault.proto.

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

import 'vault.pb.dart' as $0;

export 'vault.pb.dart';

@$pb.GrpcServiceName('k1s0.system.vault.v1.VaultService')
class VaultServiceClient extends $grpc.Client {
  /// The hostname for this service.
  static const $core.String defaultHost = '';

  /// OAuth scopes needed for the client.
  static const $core.List<$core.String> oauthScopes = [
    '',
  ];

  VaultServiceClient(super.channel, {super.options, super.interceptors});

  $grpc.ResponseFuture<$0.GetSecretResponse> getSecret(
    $0.GetSecretRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$getSecret, request, options: options);
  }

  $grpc.ResponseFuture<$0.SetSecretResponse> setSecret(
    $0.SetSecretRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$setSecret, request, options: options);
  }

  $grpc.ResponseFuture<$0.RotateSecretResponse> rotateSecret(
    $0.RotateSecretRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$rotateSecret, request, options: options);
  }

  $grpc.ResponseFuture<$0.DeleteSecretResponse> deleteSecret(
    $0.DeleteSecretRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$deleteSecret, request, options: options);
  }

  $grpc.ResponseFuture<$0.GetSecretMetadataResponse> getSecretMetadata(
    $0.GetSecretMetadataRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$getSecretMetadata, request, options: options);
  }

  $grpc.ResponseFuture<$0.ListSecretsResponse> listSecrets(
    $0.ListSecretsRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$listSecrets, request, options: options);
  }

  $grpc.ResponseFuture<$0.ListAuditLogsResponse> listAuditLogs(
    $0.ListAuditLogsRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$listAuditLogs, request, options: options);
  }

  // method descriptors

  static final _$getSecret =
      $grpc.ClientMethod<$0.GetSecretRequest, $0.GetSecretResponse>(
          '/k1s0.system.vault.v1.VaultService/GetSecret',
          ($0.GetSecretRequest value) => value.writeToBuffer(),
          $0.GetSecretResponse.fromBuffer);
  static final _$setSecret =
      $grpc.ClientMethod<$0.SetSecretRequest, $0.SetSecretResponse>(
          '/k1s0.system.vault.v1.VaultService/SetSecret',
          ($0.SetSecretRequest value) => value.writeToBuffer(),
          $0.SetSecretResponse.fromBuffer);
  static final _$rotateSecret =
      $grpc.ClientMethod<$0.RotateSecretRequest, $0.RotateSecretResponse>(
          '/k1s0.system.vault.v1.VaultService/RotateSecret',
          ($0.RotateSecretRequest value) => value.writeToBuffer(),
          $0.RotateSecretResponse.fromBuffer);
  static final _$deleteSecret =
      $grpc.ClientMethod<$0.DeleteSecretRequest, $0.DeleteSecretResponse>(
          '/k1s0.system.vault.v1.VaultService/DeleteSecret',
          ($0.DeleteSecretRequest value) => value.writeToBuffer(),
          $0.DeleteSecretResponse.fromBuffer);
  static final _$getSecretMetadata = $grpc.ClientMethod<
          $0.GetSecretMetadataRequest, $0.GetSecretMetadataResponse>(
      '/k1s0.system.vault.v1.VaultService/GetSecretMetadata',
      ($0.GetSecretMetadataRequest value) => value.writeToBuffer(),
      $0.GetSecretMetadataResponse.fromBuffer);
  static final _$listSecrets =
      $grpc.ClientMethod<$0.ListSecretsRequest, $0.ListSecretsResponse>(
          '/k1s0.system.vault.v1.VaultService/ListSecrets',
          ($0.ListSecretsRequest value) => value.writeToBuffer(),
          $0.ListSecretsResponse.fromBuffer);
  static final _$listAuditLogs =
      $grpc.ClientMethod<$0.ListAuditLogsRequest, $0.ListAuditLogsResponse>(
          '/k1s0.system.vault.v1.VaultService/ListAuditLogs',
          ($0.ListAuditLogsRequest value) => value.writeToBuffer(),
          $0.ListAuditLogsResponse.fromBuffer);
}

@$pb.GrpcServiceName('k1s0.system.vault.v1.VaultService')
abstract class VaultServiceBase extends $grpc.Service {
  $core.String get $name => 'k1s0.system.vault.v1.VaultService';

  VaultServiceBase() {
    $addMethod($grpc.ServiceMethod<$0.GetSecretRequest, $0.GetSecretResponse>(
        'GetSecret',
        getSecret_Pre,
        false,
        false,
        ($core.List<$core.int> value) => $0.GetSecretRequest.fromBuffer(value),
        ($0.GetSecretResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.SetSecretRequest, $0.SetSecretResponse>(
        'SetSecret',
        setSecret_Pre,
        false,
        false,
        ($core.List<$core.int> value) => $0.SetSecretRequest.fromBuffer(value),
        ($0.SetSecretResponse value) => value.writeToBuffer()));
    $addMethod(
        $grpc.ServiceMethod<$0.RotateSecretRequest, $0.RotateSecretResponse>(
            'RotateSecret',
            rotateSecret_Pre,
            false,
            false,
            ($core.List<$core.int> value) =>
                $0.RotateSecretRequest.fromBuffer(value),
            ($0.RotateSecretResponse value) => value.writeToBuffer()));
    $addMethod(
        $grpc.ServiceMethod<$0.DeleteSecretRequest, $0.DeleteSecretResponse>(
            'DeleteSecret',
            deleteSecret_Pre,
            false,
            false,
            ($core.List<$core.int> value) =>
                $0.DeleteSecretRequest.fromBuffer(value),
            ($0.DeleteSecretResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.GetSecretMetadataRequest,
            $0.GetSecretMetadataResponse>(
        'GetSecretMetadata',
        getSecretMetadata_Pre,
        false,
        false,
        ($core.List<$core.int> value) =>
            $0.GetSecretMetadataRequest.fromBuffer(value),
        ($0.GetSecretMetadataResponse value) => value.writeToBuffer()));
    $addMethod(
        $grpc.ServiceMethod<$0.ListSecretsRequest, $0.ListSecretsResponse>(
            'ListSecrets',
            listSecrets_Pre,
            false,
            false,
            ($core.List<$core.int> value) =>
                $0.ListSecretsRequest.fromBuffer(value),
            ($0.ListSecretsResponse value) => value.writeToBuffer()));
    $addMethod(
        $grpc.ServiceMethod<$0.ListAuditLogsRequest, $0.ListAuditLogsResponse>(
            'ListAuditLogs',
            listAuditLogs_Pre,
            false,
            false,
            ($core.List<$core.int> value) =>
                $0.ListAuditLogsRequest.fromBuffer(value),
            ($0.ListAuditLogsResponse value) => value.writeToBuffer()));
  }

  $async.Future<$0.GetSecretResponse> getSecret_Pre($grpc.ServiceCall $call,
      $async.Future<$0.GetSecretRequest> $request) async {
    return getSecret($call, await $request);
  }

  $async.Future<$0.GetSecretResponse> getSecret(
      $grpc.ServiceCall call, $0.GetSecretRequest request);

  $async.Future<$0.SetSecretResponse> setSecret_Pre($grpc.ServiceCall $call,
      $async.Future<$0.SetSecretRequest> $request) async {
    return setSecret($call, await $request);
  }

  $async.Future<$0.SetSecretResponse> setSecret(
      $grpc.ServiceCall call, $0.SetSecretRequest request);

  $async.Future<$0.RotateSecretResponse> rotateSecret_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.RotateSecretRequest> $request) async {
    return rotateSecret($call, await $request);
  }

  $async.Future<$0.RotateSecretResponse> rotateSecret(
      $grpc.ServiceCall call, $0.RotateSecretRequest request);

  $async.Future<$0.DeleteSecretResponse> deleteSecret_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.DeleteSecretRequest> $request) async {
    return deleteSecret($call, await $request);
  }

  $async.Future<$0.DeleteSecretResponse> deleteSecret(
      $grpc.ServiceCall call, $0.DeleteSecretRequest request);

  $async.Future<$0.GetSecretMetadataResponse> getSecretMetadata_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.GetSecretMetadataRequest> $request) async {
    return getSecretMetadata($call, await $request);
  }

  $async.Future<$0.GetSecretMetadataResponse> getSecretMetadata(
      $grpc.ServiceCall call, $0.GetSecretMetadataRequest request);

  $async.Future<$0.ListSecretsResponse> listSecrets_Pre($grpc.ServiceCall $call,
      $async.Future<$0.ListSecretsRequest> $request) async {
    return listSecrets($call, await $request);
  }

  $async.Future<$0.ListSecretsResponse> listSecrets(
      $grpc.ServiceCall call, $0.ListSecretsRequest request);

  $async.Future<$0.ListAuditLogsResponse> listAuditLogs_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.ListAuditLogsRequest> $request) async {
    return listAuditLogs($call, await $request);
  }

  $async.Future<$0.ListAuditLogsResponse> listAuditLogs(
      $grpc.ServiceCall call, $0.ListAuditLogsRequest request);
}
