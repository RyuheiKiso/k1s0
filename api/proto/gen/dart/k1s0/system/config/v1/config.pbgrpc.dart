// This is a generated file - do not edit.
//
// Generated from k1s0/system/config/v1/config.proto.

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

import 'config.pb.dart' as $0;

export 'config.pb.dart';

/// ConfigService は設定値の取得・更新・削除・監視を提供する。
@$pb.GrpcServiceName('k1s0.system.config.v1.ConfigService')
class ConfigServiceClient extends $grpc.Client {
  /// The hostname for this service.
  static const $core.String defaultHost = '';

  /// OAuth scopes needed for the client.
  static const $core.List<$core.String> oauthScopes = [
    '',
  ];

  ConfigServiceClient(super.channel, {super.options, super.interceptors});

  /// 設定値取得
  $grpc.ResponseFuture<$0.GetConfigResponse> getConfig(
    $0.GetConfigRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$getConfig, request, options: options);
  }

  /// namespace 内の設定値一覧取得
  $grpc.ResponseFuture<$0.ListConfigsResponse> listConfigs(
    $0.ListConfigsRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$listConfigs, request, options: options);
  }

  /// 設定値更新
  $grpc.ResponseFuture<$0.UpdateConfigResponse> updateConfig(
    $0.UpdateConfigRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$updateConfig, request, options: options);
  }

  /// 設定値削除
  $grpc.ResponseFuture<$0.DeleteConfigResponse> deleteConfig(
    $0.DeleteConfigRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$deleteConfig, request, options: options);
  }

  /// サービス向け設定一括取得
  $grpc.ResponseFuture<$0.GetServiceConfigResponse> getServiceConfig(
    $0.GetServiceConfigRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$getServiceConfig, request, options: options);
  }

  /// 設定変更の監視（Server-Side Streaming）
  $grpc.ResponseStream<$0.WatchConfigResponse> watchConfig(
    $0.WatchConfigRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createStreamingCall(
        _$watchConfig, $async.Stream.fromIterable([request]),
        options: options);
  }

  /// 設定スキーマ取得
  $grpc.ResponseFuture<$0.GetConfigSchemaResponse> getConfigSchema(
    $0.GetConfigSchemaRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$getConfigSchema, request, options: options);
  }

  /// 設定スキーマ作成・更新
  $grpc.ResponseFuture<$0.UpsertConfigSchemaResponse> upsertConfigSchema(
    $0.UpsertConfigSchemaRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$upsertConfigSchema, request, options: options);
  }

  /// 設定スキーマ一覧取得
  $grpc.ResponseFuture<$0.ListConfigSchemasResponse> listConfigSchemas(
    $0.ListConfigSchemasRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$listConfigSchemas, request, options: options);
  }

  // method descriptors

  static final _$getConfig =
      $grpc.ClientMethod<$0.GetConfigRequest, $0.GetConfigResponse>(
          '/k1s0.system.config.v1.ConfigService/GetConfig',
          ($0.GetConfigRequest value) => value.writeToBuffer(),
          $0.GetConfigResponse.fromBuffer);
  static final _$listConfigs =
      $grpc.ClientMethod<$0.ListConfigsRequest, $0.ListConfigsResponse>(
          '/k1s0.system.config.v1.ConfigService/ListConfigs',
          ($0.ListConfigsRequest value) => value.writeToBuffer(),
          $0.ListConfigsResponse.fromBuffer);
  static final _$updateConfig =
      $grpc.ClientMethod<$0.UpdateConfigRequest, $0.UpdateConfigResponse>(
          '/k1s0.system.config.v1.ConfigService/UpdateConfig',
          ($0.UpdateConfigRequest value) => value.writeToBuffer(),
          $0.UpdateConfigResponse.fromBuffer);
  static final _$deleteConfig =
      $grpc.ClientMethod<$0.DeleteConfigRequest, $0.DeleteConfigResponse>(
          '/k1s0.system.config.v1.ConfigService/DeleteConfig',
          ($0.DeleteConfigRequest value) => value.writeToBuffer(),
          $0.DeleteConfigResponse.fromBuffer);
  static final _$getServiceConfig = $grpc.ClientMethod<
          $0.GetServiceConfigRequest, $0.GetServiceConfigResponse>(
      '/k1s0.system.config.v1.ConfigService/GetServiceConfig',
      ($0.GetServiceConfigRequest value) => value.writeToBuffer(),
      $0.GetServiceConfigResponse.fromBuffer);
  static final _$watchConfig =
      $grpc.ClientMethod<$0.WatchConfigRequest, $0.WatchConfigResponse>(
          '/k1s0.system.config.v1.ConfigService/WatchConfig',
          ($0.WatchConfigRequest value) => value.writeToBuffer(),
          $0.WatchConfigResponse.fromBuffer);
  static final _$getConfigSchema =
      $grpc.ClientMethod<$0.GetConfigSchemaRequest, $0.GetConfigSchemaResponse>(
          '/k1s0.system.config.v1.ConfigService/GetConfigSchema',
          ($0.GetConfigSchemaRequest value) => value.writeToBuffer(),
          $0.GetConfigSchemaResponse.fromBuffer);
  static final _$upsertConfigSchema = $grpc.ClientMethod<
          $0.UpsertConfigSchemaRequest, $0.UpsertConfigSchemaResponse>(
      '/k1s0.system.config.v1.ConfigService/UpsertConfigSchema',
      ($0.UpsertConfigSchemaRequest value) => value.writeToBuffer(),
      $0.UpsertConfigSchemaResponse.fromBuffer);
  static final _$listConfigSchemas = $grpc.ClientMethod<
          $0.ListConfigSchemasRequest, $0.ListConfigSchemasResponse>(
      '/k1s0.system.config.v1.ConfigService/ListConfigSchemas',
      ($0.ListConfigSchemasRequest value) => value.writeToBuffer(),
      $0.ListConfigSchemasResponse.fromBuffer);
}

@$pb.GrpcServiceName('k1s0.system.config.v1.ConfigService')
abstract class ConfigServiceBase extends $grpc.Service {
  $core.String get $name => 'k1s0.system.config.v1.ConfigService';

  ConfigServiceBase() {
    $addMethod($grpc.ServiceMethod<$0.GetConfigRequest, $0.GetConfigResponse>(
        'GetConfig',
        getConfig_Pre,
        false,
        false,
        ($core.List<$core.int> value) => $0.GetConfigRequest.fromBuffer(value),
        ($0.GetConfigResponse value) => value.writeToBuffer()));
    $addMethod(
        $grpc.ServiceMethod<$0.ListConfigsRequest, $0.ListConfigsResponse>(
            'ListConfigs',
            listConfigs_Pre,
            false,
            false,
            ($core.List<$core.int> value) =>
                $0.ListConfigsRequest.fromBuffer(value),
            ($0.ListConfigsResponse value) => value.writeToBuffer()));
    $addMethod(
        $grpc.ServiceMethod<$0.UpdateConfigRequest, $0.UpdateConfigResponse>(
            'UpdateConfig',
            updateConfig_Pre,
            false,
            false,
            ($core.List<$core.int> value) =>
                $0.UpdateConfigRequest.fromBuffer(value),
            ($0.UpdateConfigResponse value) => value.writeToBuffer()));
    $addMethod(
        $grpc.ServiceMethod<$0.DeleteConfigRequest, $0.DeleteConfigResponse>(
            'DeleteConfig',
            deleteConfig_Pre,
            false,
            false,
            ($core.List<$core.int> value) =>
                $0.DeleteConfigRequest.fromBuffer(value),
            ($0.DeleteConfigResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.GetServiceConfigRequest,
            $0.GetServiceConfigResponse>(
        'GetServiceConfig',
        getServiceConfig_Pre,
        false,
        false,
        ($core.List<$core.int> value) =>
            $0.GetServiceConfigRequest.fromBuffer(value),
        ($0.GetServiceConfigResponse value) => value.writeToBuffer()));
    $addMethod(
        $grpc.ServiceMethod<$0.WatchConfigRequest, $0.WatchConfigResponse>(
            'WatchConfig',
            watchConfig_Pre,
            false,
            true,
            ($core.List<$core.int> value) =>
                $0.WatchConfigRequest.fromBuffer(value),
            ($0.WatchConfigResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.GetConfigSchemaRequest,
            $0.GetConfigSchemaResponse>(
        'GetConfigSchema',
        getConfigSchema_Pre,
        false,
        false,
        ($core.List<$core.int> value) =>
            $0.GetConfigSchemaRequest.fromBuffer(value),
        ($0.GetConfigSchemaResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.UpsertConfigSchemaRequest,
            $0.UpsertConfigSchemaResponse>(
        'UpsertConfigSchema',
        upsertConfigSchema_Pre,
        false,
        false,
        ($core.List<$core.int> value) =>
            $0.UpsertConfigSchemaRequest.fromBuffer(value),
        ($0.UpsertConfigSchemaResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.ListConfigSchemasRequest,
            $0.ListConfigSchemasResponse>(
        'ListConfigSchemas',
        listConfigSchemas_Pre,
        false,
        false,
        ($core.List<$core.int> value) =>
            $0.ListConfigSchemasRequest.fromBuffer(value),
        ($0.ListConfigSchemasResponse value) => value.writeToBuffer()));
  }

  $async.Future<$0.GetConfigResponse> getConfig_Pre($grpc.ServiceCall $call,
      $async.Future<$0.GetConfigRequest> $request) async {
    return getConfig($call, await $request);
  }

  $async.Future<$0.GetConfigResponse> getConfig(
      $grpc.ServiceCall call, $0.GetConfigRequest request);

  $async.Future<$0.ListConfigsResponse> listConfigs_Pre($grpc.ServiceCall $call,
      $async.Future<$0.ListConfigsRequest> $request) async {
    return listConfigs($call, await $request);
  }

  $async.Future<$0.ListConfigsResponse> listConfigs(
      $grpc.ServiceCall call, $0.ListConfigsRequest request);

  $async.Future<$0.UpdateConfigResponse> updateConfig_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.UpdateConfigRequest> $request) async {
    return updateConfig($call, await $request);
  }

  $async.Future<$0.UpdateConfigResponse> updateConfig(
      $grpc.ServiceCall call, $0.UpdateConfigRequest request);

  $async.Future<$0.DeleteConfigResponse> deleteConfig_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.DeleteConfigRequest> $request) async {
    return deleteConfig($call, await $request);
  }

  $async.Future<$0.DeleteConfigResponse> deleteConfig(
      $grpc.ServiceCall call, $0.DeleteConfigRequest request);

  $async.Future<$0.GetServiceConfigResponse> getServiceConfig_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.GetServiceConfigRequest> $request) async {
    return getServiceConfig($call, await $request);
  }

  $async.Future<$0.GetServiceConfigResponse> getServiceConfig(
      $grpc.ServiceCall call, $0.GetServiceConfigRequest request);

  $async.Stream<$0.WatchConfigResponse> watchConfig_Pre($grpc.ServiceCall $call,
      $async.Future<$0.WatchConfigRequest> $request) async* {
    yield* watchConfig($call, await $request);
  }

  $async.Stream<$0.WatchConfigResponse> watchConfig(
      $grpc.ServiceCall call, $0.WatchConfigRequest request);

  $async.Future<$0.GetConfigSchemaResponse> getConfigSchema_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.GetConfigSchemaRequest> $request) async {
    return getConfigSchema($call, await $request);
  }

  $async.Future<$0.GetConfigSchemaResponse> getConfigSchema(
      $grpc.ServiceCall call, $0.GetConfigSchemaRequest request);

  $async.Future<$0.UpsertConfigSchemaResponse> upsertConfigSchema_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.UpsertConfigSchemaRequest> $request) async {
    return upsertConfigSchema($call, await $request);
  }

  $async.Future<$0.UpsertConfigSchemaResponse> upsertConfigSchema(
      $grpc.ServiceCall call, $0.UpsertConfigSchemaRequest request);

  $async.Future<$0.ListConfigSchemasResponse> listConfigSchemas_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.ListConfigSchemasRequest> $request) async {
    return listConfigSchemas($call, await $request);
  }

  $async.Future<$0.ListConfigSchemasResponse> listConfigSchemas(
      $grpc.ServiceCall call, $0.ListConfigSchemasRequest request);
}
