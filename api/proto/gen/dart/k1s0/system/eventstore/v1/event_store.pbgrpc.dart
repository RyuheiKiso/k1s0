// This is a generated file - do not edit.
//
// Generated from k1s0/system/eventstore/v1/event_store.proto.

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

import 'event_store.pb.dart' as $0;

export 'event_store.pb.dart';

@$pb.GrpcServiceName('k1s0.system.eventstore.v1.EventStoreService')
class EventStoreServiceClient extends $grpc.Client {
  /// The hostname for this service.
  static const $core.String defaultHost = '';

  /// OAuth scopes needed for the client.
  static const $core.List<$core.String> oauthScopes = [
    '',
  ];

  EventStoreServiceClient(super.channel, {super.options, super.interceptors});

  $grpc.ResponseFuture<$0.ListStreamsResponse> listStreams(
    $0.ListStreamsRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$listStreams, request, options: options);
  }

  $grpc.ResponseFuture<$0.AppendEventsResponse> appendEvents(
    $0.AppendEventsRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$appendEvents, request, options: options);
  }

  $grpc.ResponseFuture<$0.ReadEventsResponse> readEvents(
    $0.ReadEventsRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$readEvents, request, options: options);
  }

  $grpc.ResponseFuture<$0.ReadEventBySequenceResponse> readEventBySequence(
    $0.ReadEventBySequenceRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$readEventBySequence, request, options: options);
  }

  $grpc.ResponseFuture<$0.CreateSnapshotResponse> createSnapshot(
    $0.CreateSnapshotRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$createSnapshot, request, options: options);
  }

  $grpc.ResponseFuture<$0.GetLatestSnapshotResponse> getLatestSnapshot(
    $0.GetLatestSnapshotRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$getLatestSnapshot, request, options: options);
  }

  $grpc.ResponseFuture<$0.DeleteStreamResponse> deleteStream(
    $0.DeleteStreamRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$deleteStream, request, options: options);
  }

  // method descriptors

  static final _$listStreams =
      $grpc.ClientMethod<$0.ListStreamsRequest, $0.ListStreamsResponse>(
          '/k1s0.system.eventstore.v1.EventStoreService/ListStreams',
          ($0.ListStreamsRequest value) => value.writeToBuffer(),
          $0.ListStreamsResponse.fromBuffer);
  static final _$appendEvents =
      $grpc.ClientMethod<$0.AppendEventsRequest, $0.AppendEventsResponse>(
          '/k1s0.system.eventstore.v1.EventStoreService/AppendEvents',
          ($0.AppendEventsRequest value) => value.writeToBuffer(),
          $0.AppendEventsResponse.fromBuffer);
  static final _$readEvents =
      $grpc.ClientMethod<$0.ReadEventsRequest, $0.ReadEventsResponse>(
          '/k1s0.system.eventstore.v1.EventStoreService/ReadEvents',
          ($0.ReadEventsRequest value) => value.writeToBuffer(),
          $0.ReadEventsResponse.fromBuffer);
  static final _$readEventBySequence = $grpc.ClientMethod<
          $0.ReadEventBySequenceRequest, $0.ReadEventBySequenceResponse>(
      '/k1s0.system.eventstore.v1.EventStoreService/ReadEventBySequence',
      ($0.ReadEventBySequenceRequest value) => value.writeToBuffer(),
      $0.ReadEventBySequenceResponse.fromBuffer);
  static final _$createSnapshot =
      $grpc.ClientMethod<$0.CreateSnapshotRequest, $0.CreateSnapshotResponse>(
          '/k1s0.system.eventstore.v1.EventStoreService/CreateSnapshot',
          ($0.CreateSnapshotRequest value) => value.writeToBuffer(),
          $0.CreateSnapshotResponse.fromBuffer);
  static final _$getLatestSnapshot = $grpc.ClientMethod<
          $0.GetLatestSnapshotRequest, $0.GetLatestSnapshotResponse>(
      '/k1s0.system.eventstore.v1.EventStoreService/GetLatestSnapshot',
      ($0.GetLatestSnapshotRequest value) => value.writeToBuffer(),
      $0.GetLatestSnapshotResponse.fromBuffer);
  static final _$deleteStream =
      $grpc.ClientMethod<$0.DeleteStreamRequest, $0.DeleteStreamResponse>(
          '/k1s0.system.eventstore.v1.EventStoreService/DeleteStream',
          ($0.DeleteStreamRequest value) => value.writeToBuffer(),
          $0.DeleteStreamResponse.fromBuffer);
}

@$pb.GrpcServiceName('k1s0.system.eventstore.v1.EventStoreService')
abstract class EventStoreServiceBase extends $grpc.Service {
  $core.String get $name => 'k1s0.system.eventstore.v1.EventStoreService';

  EventStoreServiceBase() {
    $addMethod(
        $grpc.ServiceMethod<$0.ListStreamsRequest, $0.ListStreamsResponse>(
            'ListStreams',
            listStreams_Pre,
            false,
            false,
            ($core.List<$core.int> value) =>
                $0.ListStreamsRequest.fromBuffer(value),
            ($0.ListStreamsResponse value) => value.writeToBuffer()));
    $addMethod(
        $grpc.ServiceMethod<$0.AppendEventsRequest, $0.AppendEventsResponse>(
            'AppendEvents',
            appendEvents_Pre,
            false,
            false,
            ($core.List<$core.int> value) =>
                $0.AppendEventsRequest.fromBuffer(value),
            ($0.AppendEventsResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.ReadEventsRequest, $0.ReadEventsResponse>(
        'ReadEvents',
        readEvents_Pre,
        false,
        false,
        ($core.List<$core.int> value) => $0.ReadEventsRequest.fromBuffer(value),
        ($0.ReadEventsResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.ReadEventBySequenceRequest,
            $0.ReadEventBySequenceResponse>(
        'ReadEventBySequence',
        readEventBySequence_Pre,
        false,
        false,
        ($core.List<$core.int> value) =>
            $0.ReadEventBySequenceRequest.fromBuffer(value),
        ($0.ReadEventBySequenceResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.CreateSnapshotRequest,
            $0.CreateSnapshotResponse>(
        'CreateSnapshot',
        createSnapshot_Pre,
        false,
        false,
        ($core.List<$core.int> value) =>
            $0.CreateSnapshotRequest.fromBuffer(value),
        ($0.CreateSnapshotResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.GetLatestSnapshotRequest,
            $0.GetLatestSnapshotResponse>(
        'GetLatestSnapshot',
        getLatestSnapshot_Pre,
        false,
        false,
        ($core.List<$core.int> value) =>
            $0.GetLatestSnapshotRequest.fromBuffer(value),
        ($0.GetLatestSnapshotResponse value) => value.writeToBuffer()));
    $addMethod(
        $grpc.ServiceMethod<$0.DeleteStreamRequest, $0.DeleteStreamResponse>(
            'DeleteStream',
            deleteStream_Pre,
            false,
            false,
            ($core.List<$core.int> value) =>
                $0.DeleteStreamRequest.fromBuffer(value),
            ($0.DeleteStreamResponse value) => value.writeToBuffer()));
  }

  $async.Future<$0.ListStreamsResponse> listStreams_Pre($grpc.ServiceCall $call,
      $async.Future<$0.ListStreamsRequest> $request) async {
    return listStreams($call, await $request);
  }

  $async.Future<$0.ListStreamsResponse> listStreams(
      $grpc.ServiceCall call, $0.ListStreamsRequest request);

  $async.Future<$0.AppendEventsResponse> appendEvents_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.AppendEventsRequest> $request) async {
    return appendEvents($call, await $request);
  }

  $async.Future<$0.AppendEventsResponse> appendEvents(
      $grpc.ServiceCall call, $0.AppendEventsRequest request);

  $async.Future<$0.ReadEventsResponse> readEvents_Pre($grpc.ServiceCall $call,
      $async.Future<$0.ReadEventsRequest> $request) async {
    return readEvents($call, await $request);
  }

  $async.Future<$0.ReadEventsResponse> readEvents(
      $grpc.ServiceCall call, $0.ReadEventsRequest request);

  $async.Future<$0.ReadEventBySequenceResponse> readEventBySequence_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.ReadEventBySequenceRequest> $request) async {
    return readEventBySequence($call, await $request);
  }

  $async.Future<$0.ReadEventBySequenceResponse> readEventBySequence(
      $grpc.ServiceCall call, $0.ReadEventBySequenceRequest request);

  $async.Future<$0.CreateSnapshotResponse> createSnapshot_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.CreateSnapshotRequest> $request) async {
    return createSnapshot($call, await $request);
  }

  $async.Future<$0.CreateSnapshotResponse> createSnapshot(
      $grpc.ServiceCall call, $0.CreateSnapshotRequest request);

  $async.Future<$0.GetLatestSnapshotResponse> getLatestSnapshot_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.GetLatestSnapshotRequest> $request) async {
    return getLatestSnapshot($call, await $request);
  }

  $async.Future<$0.GetLatestSnapshotResponse> getLatestSnapshot(
      $grpc.ServiceCall call, $0.GetLatestSnapshotRequest request);

  $async.Future<$0.DeleteStreamResponse> deleteStream_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.DeleteStreamRequest> $request) async {
    return deleteStream($call, await $request);
  }

  $async.Future<$0.DeleteStreamResponse> deleteStream(
      $grpc.ServiceCall call, $0.DeleteStreamRequest request);
}
