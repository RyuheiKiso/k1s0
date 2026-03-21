// This is a generated file - do not edit.
//
// Generated from k1s0/system/search/v1/search.proto.

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

import 'search.pb.dart' as $0;

export 'search.pb.dart';

@$pb.GrpcServiceName('k1s0.system.search.v1.SearchService')
class SearchServiceClient extends $grpc.Client {
  /// The hostname for this service.
  static const $core.String defaultHost = '';

  /// OAuth scopes needed for the client.
  static const $core.List<$core.String> oauthScopes = [
    '',
  ];

  SearchServiceClient(super.channel, {super.options, super.interceptors});

  $grpc.ResponseFuture<$0.CreateIndexResponse> createIndex(
    $0.CreateIndexRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$createIndex, request, options: options);
  }

  $grpc.ResponseFuture<$0.ListIndicesResponse> listIndices(
    $0.ListIndicesRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$listIndices, request, options: options);
  }

  $grpc.ResponseFuture<$0.IndexDocumentResponse> indexDocument(
    $0.IndexDocumentRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$indexDocument, request, options: options);
  }

  $grpc.ResponseFuture<$0.SearchResponse> search(
    $0.SearchRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$search, request, options: options);
  }

  $grpc.ResponseFuture<$0.DeleteDocumentResponse> deleteDocument(
    $0.DeleteDocumentRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$deleteDocument, request, options: options);
  }

  // method descriptors

  static final _$createIndex =
      $grpc.ClientMethod<$0.CreateIndexRequest, $0.CreateIndexResponse>(
          '/k1s0.system.search.v1.SearchService/CreateIndex',
          ($0.CreateIndexRequest value) => value.writeToBuffer(),
          $0.CreateIndexResponse.fromBuffer);
  static final _$listIndices =
      $grpc.ClientMethod<$0.ListIndicesRequest, $0.ListIndicesResponse>(
          '/k1s0.system.search.v1.SearchService/ListIndices',
          ($0.ListIndicesRequest value) => value.writeToBuffer(),
          $0.ListIndicesResponse.fromBuffer);
  static final _$indexDocument =
      $grpc.ClientMethod<$0.IndexDocumentRequest, $0.IndexDocumentResponse>(
          '/k1s0.system.search.v1.SearchService/IndexDocument',
          ($0.IndexDocumentRequest value) => value.writeToBuffer(),
          $0.IndexDocumentResponse.fromBuffer);
  static final _$search =
      $grpc.ClientMethod<$0.SearchRequest, $0.SearchResponse>(
          '/k1s0.system.search.v1.SearchService/Search',
          ($0.SearchRequest value) => value.writeToBuffer(),
          $0.SearchResponse.fromBuffer);
  static final _$deleteDocument =
      $grpc.ClientMethod<$0.DeleteDocumentRequest, $0.DeleteDocumentResponse>(
          '/k1s0.system.search.v1.SearchService/DeleteDocument',
          ($0.DeleteDocumentRequest value) => value.writeToBuffer(),
          $0.DeleteDocumentResponse.fromBuffer);
}

@$pb.GrpcServiceName('k1s0.system.search.v1.SearchService')
abstract class SearchServiceBase extends $grpc.Service {
  $core.String get $name => 'k1s0.system.search.v1.SearchService';

  SearchServiceBase() {
    $addMethod(
        $grpc.ServiceMethod<$0.CreateIndexRequest, $0.CreateIndexResponse>(
            'CreateIndex',
            createIndex_Pre,
            false,
            false,
            ($core.List<$core.int> value) =>
                $0.CreateIndexRequest.fromBuffer(value),
            ($0.CreateIndexResponse value) => value.writeToBuffer()));
    $addMethod(
        $grpc.ServiceMethod<$0.ListIndicesRequest, $0.ListIndicesResponse>(
            'ListIndices',
            listIndices_Pre,
            false,
            false,
            ($core.List<$core.int> value) =>
                $0.ListIndicesRequest.fromBuffer(value),
            ($0.ListIndicesResponse value) => value.writeToBuffer()));
    $addMethod(
        $grpc.ServiceMethod<$0.IndexDocumentRequest, $0.IndexDocumentResponse>(
            'IndexDocument',
            indexDocument_Pre,
            false,
            false,
            ($core.List<$core.int> value) =>
                $0.IndexDocumentRequest.fromBuffer(value),
            ($0.IndexDocumentResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.SearchRequest, $0.SearchResponse>(
        'Search',
        search_Pre,
        false,
        false,
        ($core.List<$core.int> value) => $0.SearchRequest.fromBuffer(value),
        ($0.SearchResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.DeleteDocumentRequest,
            $0.DeleteDocumentResponse>(
        'DeleteDocument',
        deleteDocument_Pre,
        false,
        false,
        ($core.List<$core.int> value) =>
            $0.DeleteDocumentRequest.fromBuffer(value),
        ($0.DeleteDocumentResponse value) => value.writeToBuffer()));
  }

  $async.Future<$0.CreateIndexResponse> createIndex_Pre($grpc.ServiceCall $call,
      $async.Future<$0.CreateIndexRequest> $request) async {
    return createIndex($call, await $request);
  }

  $async.Future<$0.CreateIndexResponse> createIndex(
      $grpc.ServiceCall call, $0.CreateIndexRequest request);

  $async.Future<$0.ListIndicesResponse> listIndices_Pre($grpc.ServiceCall $call,
      $async.Future<$0.ListIndicesRequest> $request) async {
    return listIndices($call, await $request);
  }

  $async.Future<$0.ListIndicesResponse> listIndices(
      $grpc.ServiceCall call, $0.ListIndicesRequest request);

  $async.Future<$0.IndexDocumentResponse> indexDocument_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.IndexDocumentRequest> $request) async {
    return indexDocument($call, await $request);
  }

  $async.Future<$0.IndexDocumentResponse> indexDocument(
      $grpc.ServiceCall call, $0.IndexDocumentRequest request);

  $async.Future<$0.SearchResponse> search_Pre(
      $grpc.ServiceCall $call, $async.Future<$0.SearchRequest> $request) async {
    return search($call, await $request);
  }

  $async.Future<$0.SearchResponse> search(
      $grpc.ServiceCall call, $0.SearchRequest request);

  $async.Future<$0.DeleteDocumentResponse> deleteDocument_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.DeleteDocumentRequest> $request) async {
    return deleteDocument($call, await $request);
  }

  $async.Future<$0.DeleteDocumentResponse> deleteDocument(
      $grpc.ServiceCall call, $0.DeleteDocumentRequest request);
}
