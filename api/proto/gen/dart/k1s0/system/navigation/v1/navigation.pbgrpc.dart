// This is a generated file - do not edit.
//
// Generated from k1s0/system/navigation/v1/navigation.proto.

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

import 'navigation.pb.dart' as $0;

export 'navigation.pb.dart';

/// NavigationService はクライアントアプリのルーティング設定を提供する。
@$pb.GrpcServiceName('k1s0.system.navigation.v1.NavigationService')
class NavigationServiceClient extends $grpc.Client {
  /// The hostname for this service.
  static const $core.String defaultHost = '';

  /// OAuth scopes needed for the client.
  static const $core.List<$core.String> oauthScopes = [
    '',
  ];

  NavigationServiceClient(super.channel, {super.options, super.interceptors});

  /// ナビゲーション設定取得
  $grpc.ResponseFuture<$0.GetNavigationResponse> getNavigation(
    $0.GetNavigationRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$getNavigation, request, options: options);
  }

  // method descriptors

  static final _$getNavigation =
      $grpc.ClientMethod<$0.GetNavigationRequest, $0.GetNavigationResponse>(
          '/k1s0.system.navigation.v1.NavigationService/GetNavigation',
          ($0.GetNavigationRequest value) => value.writeToBuffer(),
          $0.GetNavigationResponse.fromBuffer);
}

@$pb.GrpcServiceName('k1s0.system.navigation.v1.NavigationService')
abstract class NavigationServiceBase extends $grpc.Service {
  $core.String get $name => 'k1s0.system.navigation.v1.NavigationService';

  NavigationServiceBase() {
    $addMethod(
        $grpc.ServiceMethod<$0.GetNavigationRequest, $0.GetNavigationResponse>(
            'GetNavigation',
            getNavigation_Pre,
            false,
            false,
            ($core.List<$core.int> value) =>
                $0.GetNavigationRequest.fromBuffer(value),
            ($0.GetNavigationResponse value) => value.writeToBuffer()));
  }

  $async.Future<$0.GetNavigationResponse> getNavigation_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.GetNavigationRequest> $request) async {
    return getNavigation($call, await $request);
  }

  $async.Future<$0.GetNavigationResponse> getNavigation(
      $grpc.ServiceCall call, $0.GetNavigationRequest request);
}
