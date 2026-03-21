// This is a generated file - do not edit.
//
// Generated from k1s0/system/eventmonitor/v1/event_monitor.proto.

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

import 'event_monitor.pb.dart' as $0;

export 'event_monitor.pb.dart';

@$pb.GrpcServiceName('k1s0.system.eventmonitor.v1.EventMonitorService')
class EventMonitorServiceClient extends $grpc.Client {
  /// The hostname for this service.
  static const $core.String defaultHost = '';

  /// OAuth scopes needed for the client.
  static const $core.List<$core.String> oauthScopes = [
    '',
  ];

  EventMonitorServiceClient(super.channel, {super.options, super.interceptors});

  $grpc.ResponseFuture<$0.ListEventsResponse> listEvents(
    $0.ListEventsRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$listEvents, request, options: options);
  }

  $grpc.ResponseFuture<$0.TraceByCorrelationResponse> traceByCorrelation(
    $0.TraceByCorrelationRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$traceByCorrelation, request, options: options);
  }

  $grpc.ResponseFuture<$0.ListFlowsResponse> listFlows(
    $0.ListFlowsRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$listFlows, request, options: options);
  }

  $grpc.ResponseFuture<$0.GetFlowResponse> getFlow(
    $0.GetFlowRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$getFlow, request, options: options);
  }

  $grpc.ResponseFuture<$0.CreateFlowResponse> createFlow(
    $0.CreateFlowRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$createFlow, request, options: options);
  }

  $grpc.ResponseFuture<$0.UpdateFlowResponse> updateFlow(
    $0.UpdateFlowRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$updateFlow, request, options: options);
  }

  $grpc.ResponseFuture<$0.DeleteFlowResponse> deleteFlow(
    $0.DeleteFlowRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$deleteFlow, request, options: options);
  }

  $grpc.ResponseFuture<$0.GetFlowKpiResponse> getFlowKpi(
    $0.GetFlowKpiRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$getFlowKpi, request, options: options);
  }

  $grpc.ResponseFuture<$0.GetKpiSummaryResponse> getKpiSummary(
    $0.GetKpiSummaryRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$getKpiSummary, request, options: options);
  }

  $grpc.ResponseFuture<$0.GetSloStatusResponse> getSloStatus(
    $0.GetSloStatusRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$getSloStatus, request, options: options);
  }

  $grpc.ResponseFuture<$0.GetSloBurnRateResponse> getSloBurnRate(
    $0.GetSloBurnRateRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$getSloBurnRate, request, options: options);
  }

  $grpc.ResponseFuture<$0.PreviewReplayResponse> previewReplay(
    $0.PreviewReplayRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$previewReplay, request, options: options);
  }

  $grpc.ResponseFuture<$0.ExecuteReplayResponse> executeReplay(
    $0.ExecuteReplayRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$executeReplay, request, options: options);
  }

  // method descriptors

  static final _$listEvents =
      $grpc.ClientMethod<$0.ListEventsRequest, $0.ListEventsResponse>(
          '/k1s0.system.eventmonitor.v1.EventMonitorService/ListEvents',
          ($0.ListEventsRequest value) => value.writeToBuffer(),
          $0.ListEventsResponse.fromBuffer);
  static final _$traceByCorrelation = $grpc.ClientMethod<
          $0.TraceByCorrelationRequest, $0.TraceByCorrelationResponse>(
      '/k1s0.system.eventmonitor.v1.EventMonitorService/TraceByCorrelation',
      ($0.TraceByCorrelationRequest value) => value.writeToBuffer(),
      $0.TraceByCorrelationResponse.fromBuffer);
  static final _$listFlows =
      $grpc.ClientMethod<$0.ListFlowsRequest, $0.ListFlowsResponse>(
          '/k1s0.system.eventmonitor.v1.EventMonitorService/ListFlows',
          ($0.ListFlowsRequest value) => value.writeToBuffer(),
          $0.ListFlowsResponse.fromBuffer);
  static final _$getFlow =
      $grpc.ClientMethod<$0.GetFlowRequest, $0.GetFlowResponse>(
          '/k1s0.system.eventmonitor.v1.EventMonitorService/GetFlow',
          ($0.GetFlowRequest value) => value.writeToBuffer(),
          $0.GetFlowResponse.fromBuffer);
  static final _$createFlow =
      $grpc.ClientMethod<$0.CreateFlowRequest, $0.CreateFlowResponse>(
          '/k1s0.system.eventmonitor.v1.EventMonitorService/CreateFlow',
          ($0.CreateFlowRequest value) => value.writeToBuffer(),
          $0.CreateFlowResponse.fromBuffer);
  static final _$updateFlow =
      $grpc.ClientMethod<$0.UpdateFlowRequest, $0.UpdateFlowResponse>(
          '/k1s0.system.eventmonitor.v1.EventMonitorService/UpdateFlow',
          ($0.UpdateFlowRequest value) => value.writeToBuffer(),
          $0.UpdateFlowResponse.fromBuffer);
  static final _$deleteFlow =
      $grpc.ClientMethod<$0.DeleteFlowRequest, $0.DeleteFlowResponse>(
          '/k1s0.system.eventmonitor.v1.EventMonitorService/DeleteFlow',
          ($0.DeleteFlowRequest value) => value.writeToBuffer(),
          $0.DeleteFlowResponse.fromBuffer);
  static final _$getFlowKpi =
      $grpc.ClientMethod<$0.GetFlowKpiRequest, $0.GetFlowKpiResponse>(
          '/k1s0.system.eventmonitor.v1.EventMonitorService/GetFlowKpi',
          ($0.GetFlowKpiRequest value) => value.writeToBuffer(),
          $0.GetFlowKpiResponse.fromBuffer);
  static final _$getKpiSummary =
      $grpc.ClientMethod<$0.GetKpiSummaryRequest, $0.GetKpiSummaryResponse>(
          '/k1s0.system.eventmonitor.v1.EventMonitorService/GetKpiSummary',
          ($0.GetKpiSummaryRequest value) => value.writeToBuffer(),
          $0.GetKpiSummaryResponse.fromBuffer);
  static final _$getSloStatus =
      $grpc.ClientMethod<$0.GetSloStatusRequest, $0.GetSloStatusResponse>(
          '/k1s0.system.eventmonitor.v1.EventMonitorService/GetSloStatus',
          ($0.GetSloStatusRequest value) => value.writeToBuffer(),
          $0.GetSloStatusResponse.fromBuffer);
  static final _$getSloBurnRate =
      $grpc.ClientMethod<$0.GetSloBurnRateRequest, $0.GetSloBurnRateResponse>(
          '/k1s0.system.eventmonitor.v1.EventMonitorService/GetSloBurnRate',
          ($0.GetSloBurnRateRequest value) => value.writeToBuffer(),
          $0.GetSloBurnRateResponse.fromBuffer);
  static final _$previewReplay =
      $grpc.ClientMethod<$0.PreviewReplayRequest, $0.PreviewReplayResponse>(
          '/k1s0.system.eventmonitor.v1.EventMonitorService/PreviewReplay',
          ($0.PreviewReplayRequest value) => value.writeToBuffer(),
          $0.PreviewReplayResponse.fromBuffer);
  static final _$executeReplay =
      $grpc.ClientMethod<$0.ExecuteReplayRequest, $0.ExecuteReplayResponse>(
          '/k1s0.system.eventmonitor.v1.EventMonitorService/ExecuteReplay',
          ($0.ExecuteReplayRequest value) => value.writeToBuffer(),
          $0.ExecuteReplayResponse.fromBuffer);
}

@$pb.GrpcServiceName('k1s0.system.eventmonitor.v1.EventMonitorService')
abstract class EventMonitorServiceBase extends $grpc.Service {
  $core.String get $name => 'k1s0.system.eventmonitor.v1.EventMonitorService';

  EventMonitorServiceBase() {
    $addMethod($grpc.ServiceMethod<$0.ListEventsRequest, $0.ListEventsResponse>(
        'ListEvents',
        listEvents_Pre,
        false,
        false,
        ($core.List<$core.int> value) => $0.ListEventsRequest.fromBuffer(value),
        ($0.ListEventsResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.TraceByCorrelationRequest,
            $0.TraceByCorrelationResponse>(
        'TraceByCorrelation',
        traceByCorrelation_Pre,
        false,
        false,
        ($core.List<$core.int> value) =>
            $0.TraceByCorrelationRequest.fromBuffer(value),
        ($0.TraceByCorrelationResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.ListFlowsRequest, $0.ListFlowsResponse>(
        'ListFlows',
        listFlows_Pre,
        false,
        false,
        ($core.List<$core.int> value) => $0.ListFlowsRequest.fromBuffer(value),
        ($0.ListFlowsResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.GetFlowRequest, $0.GetFlowResponse>(
        'GetFlow',
        getFlow_Pre,
        false,
        false,
        ($core.List<$core.int> value) => $0.GetFlowRequest.fromBuffer(value),
        ($0.GetFlowResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.CreateFlowRequest, $0.CreateFlowResponse>(
        'CreateFlow',
        createFlow_Pre,
        false,
        false,
        ($core.List<$core.int> value) => $0.CreateFlowRequest.fromBuffer(value),
        ($0.CreateFlowResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.UpdateFlowRequest, $0.UpdateFlowResponse>(
        'UpdateFlow',
        updateFlow_Pre,
        false,
        false,
        ($core.List<$core.int> value) => $0.UpdateFlowRequest.fromBuffer(value),
        ($0.UpdateFlowResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.DeleteFlowRequest, $0.DeleteFlowResponse>(
        'DeleteFlow',
        deleteFlow_Pre,
        false,
        false,
        ($core.List<$core.int> value) => $0.DeleteFlowRequest.fromBuffer(value),
        ($0.DeleteFlowResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.GetFlowKpiRequest, $0.GetFlowKpiResponse>(
        'GetFlowKpi',
        getFlowKpi_Pre,
        false,
        false,
        ($core.List<$core.int> value) => $0.GetFlowKpiRequest.fromBuffer(value),
        ($0.GetFlowKpiResponse value) => value.writeToBuffer()));
    $addMethod(
        $grpc.ServiceMethod<$0.GetKpiSummaryRequest, $0.GetKpiSummaryResponse>(
            'GetKpiSummary',
            getKpiSummary_Pre,
            false,
            false,
            ($core.List<$core.int> value) =>
                $0.GetKpiSummaryRequest.fromBuffer(value),
            ($0.GetKpiSummaryResponse value) => value.writeToBuffer()));
    $addMethod(
        $grpc.ServiceMethod<$0.GetSloStatusRequest, $0.GetSloStatusResponse>(
            'GetSloStatus',
            getSloStatus_Pre,
            false,
            false,
            ($core.List<$core.int> value) =>
                $0.GetSloStatusRequest.fromBuffer(value),
            ($0.GetSloStatusResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.GetSloBurnRateRequest,
            $0.GetSloBurnRateResponse>(
        'GetSloBurnRate',
        getSloBurnRate_Pre,
        false,
        false,
        ($core.List<$core.int> value) =>
            $0.GetSloBurnRateRequest.fromBuffer(value),
        ($0.GetSloBurnRateResponse value) => value.writeToBuffer()));
    $addMethod(
        $grpc.ServiceMethod<$0.PreviewReplayRequest, $0.PreviewReplayResponse>(
            'PreviewReplay',
            previewReplay_Pre,
            false,
            false,
            ($core.List<$core.int> value) =>
                $0.PreviewReplayRequest.fromBuffer(value),
            ($0.PreviewReplayResponse value) => value.writeToBuffer()));
    $addMethod(
        $grpc.ServiceMethod<$0.ExecuteReplayRequest, $0.ExecuteReplayResponse>(
            'ExecuteReplay',
            executeReplay_Pre,
            false,
            false,
            ($core.List<$core.int> value) =>
                $0.ExecuteReplayRequest.fromBuffer(value),
            ($0.ExecuteReplayResponse value) => value.writeToBuffer()));
  }

  $async.Future<$0.ListEventsResponse> listEvents_Pre($grpc.ServiceCall $call,
      $async.Future<$0.ListEventsRequest> $request) async {
    return listEvents($call, await $request);
  }

  $async.Future<$0.ListEventsResponse> listEvents(
      $grpc.ServiceCall call, $0.ListEventsRequest request);

  $async.Future<$0.TraceByCorrelationResponse> traceByCorrelation_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.TraceByCorrelationRequest> $request) async {
    return traceByCorrelation($call, await $request);
  }

  $async.Future<$0.TraceByCorrelationResponse> traceByCorrelation(
      $grpc.ServiceCall call, $0.TraceByCorrelationRequest request);

  $async.Future<$0.ListFlowsResponse> listFlows_Pre($grpc.ServiceCall $call,
      $async.Future<$0.ListFlowsRequest> $request) async {
    return listFlows($call, await $request);
  }

  $async.Future<$0.ListFlowsResponse> listFlows(
      $grpc.ServiceCall call, $0.ListFlowsRequest request);

  $async.Future<$0.GetFlowResponse> getFlow_Pre($grpc.ServiceCall $call,
      $async.Future<$0.GetFlowRequest> $request) async {
    return getFlow($call, await $request);
  }

  $async.Future<$0.GetFlowResponse> getFlow(
      $grpc.ServiceCall call, $0.GetFlowRequest request);

  $async.Future<$0.CreateFlowResponse> createFlow_Pre($grpc.ServiceCall $call,
      $async.Future<$0.CreateFlowRequest> $request) async {
    return createFlow($call, await $request);
  }

  $async.Future<$0.CreateFlowResponse> createFlow(
      $grpc.ServiceCall call, $0.CreateFlowRequest request);

  $async.Future<$0.UpdateFlowResponse> updateFlow_Pre($grpc.ServiceCall $call,
      $async.Future<$0.UpdateFlowRequest> $request) async {
    return updateFlow($call, await $request);
  }

  $async.Future<$0.UpdateFlowResponse> updateFlow(
      $grpc.ServiceCall call, $0.UpdateFlowRequest request);

  $async.Future<$0.DeleteFlowResponse> deleteFlow_Pre($grpc.ServiceCall $call,
      $async.Future<$0.DeleteFlowRequest> $request) async {
    return deleteFlow($call, await $request);
  }

  $async.Future<$0.DeleteFlowResponse> deleteFlow(
      $grpc.ServiceCall call, $0.DeleteFlowRequest request);

  $async.Future<$0.GetFlowKpiResponse> getFlowKpi_Pre($grpc.ServiceCall $call,
      $async.Future<$0.GetFlowKpiRequest> $request) async {
    return getFlowKpi($call, await $request);
  }

  $async.Future<$0.GetFlowKpiResponse> getFlowKpi(
      $grpc.ServiceCall call, $0.GetFlowKpiRequest request);

  $async.Future<$0.GetKpiSummaryResponse> getKpiSummary_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.GetKpiSummaryRequest> $request) async {
    return getKpiSummary($call, await $request);
  }

  $async.Future<$0.GetKpiSummaryResponse> getKpiSummary(
      $grpc.ServiceCall call, $0.GetKpiSummaryRequest request);

  $async.Future<$0.GetSloStatusResponse> getSloStatus_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.GetSloStatusRequest> $request) async {
    return getSloStatus($call, await $request);
  }

  $async.Future<$0.GetSloStatusResponse> getSloStatus(
      $grpc.ServiceCall call, $0.GetSloStatusRequest request);

  $async.Future<$0.GetSloBurnRateResponse> getSloBurnRate_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.GetSloBurnRateRequest> $request) async {
    return getSloBurnRate($call, await $request);
  }

  $async.Future<$0.GetSloBurnRateResponse> getSloBurnRate(
      $grpc.ServiceCall call, $0.GetSloBurnRateRequest request);

  $async.Future<$0.PreviewReplayResponse> previewReplay_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.PreviewReplayRequest> $request) async {
    return previewReplay($call, await $request);
  }

  $async.Future<$0.PreviewReplayResponse> previewReplay(
      $grpc.ServiceCall call, $0.PreviewReplayRequest request);

  $async.Future<$0.ExecuteReplayResponse> executeReplay_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.ExecuteReplayRequest> $request) async {
    return executeReplay($call, await $request);
  }

  $async.Future<$0.ExecuteReplayResponse> executeReplay(
      $grpc.ServiceCall call, $0.ExecuteReplayRequest request);
}
