// This is a generated file - do not edit.
//
// Generated from k1s0/system/file/v1/file.proto.

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

import 'file.pb.dart' as $0;

export 'file.pb.dart';

@$pb.GrpcServiceName('k1s0.system.file.v1.FileService')
class FileServiceClient extends $grpc.Client {
  /// The hostname for this service.
  static const $core.String defaultHost = '';

  /// OAuth scopes needed for the client.
  static const $core.List<$core.String> oauthScopes = [
    '',
  ];

  FileServiceClient(super.channel, {super.options, super.interceptors});

  $grpc.ResponseFuture<$0.GetFileMetadataResponse> getFileMetadata(
    $0.GetFileMetadataRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$getFileMetadata, request, options: options);
  }

  $grpc.ResponseFuture<$0.ListFilesResponse> listFiles(
    $0.ListFilesRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$listFiles, request, options: options);
  }

  $grpc.ResponseFuture<$0.GenerateUploadUrlResponse> generateUploadUrl(
    $0.GenerateUploadUrlRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$generateUploadUrl, request, options: options);
  }

  $grpc.ResponseFuture<$0.CompleteUploadResponse> completeUpload(
    $0.CompleteUploadRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$completeUpload, request, options: options);
  }

  $grpc.ResponseFuture<$0.GenerateDownloadUrlResponse> generateDownloadUrl(
    $0.GenerateDownloadUrlRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$generateDownloadUrl, request, options: options);
  }

  $grpc.ResponseFuture<$0.UpdateFileTagsResponse> updateFileTags(
    $0.UpdateFileTagsRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$updateFileTags, request, options: options);
  }

  $grpc.ResponseFuture<$0.DeleteFileResponse> deleteFile(
    $0.DeleteFileRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$deleteFile, request, options: options);
  }

  // method descriptors

  static final _$getFileMetadata =
      $grpc.ClientMethod<$0.GetFileMetadataRequest, $0.GetFileMetadataResponse>(
          '/k1s0.system.file.v1.FileService/GetFileMetadata',
          ($0.GetFileMetadataRequest value) => value.writeToBuffer(),
          $0.GetFileMetadataResponse.fromBuffer);
  static final _$listFiles =
      $grpc.ClientMethod<$0.ListFilesRequest, $0.ListFilesResponse>(
          '/k1s0.system.file.v1.FileService/ListFiles',
          ($0.ListFilesRequest value) => value.writeToBuffer(),
          $0.ListFilesResponse.fromBuffer);
  static final _$generateUploadUrl = $grpc.ClientMethod<
          $0.GenerateUploadUrlRequest, $0.GenerateUploadUrlResponse>(
      '/k1s0.system.file.v1.FileService/GenerateUploadUrl',
      ($0.GenerateUploadUrlRequest value) => value.writeToBuffer(),
      $0.GenerateUploadUrlResponse.fromBuffer);
  static final _$completeUpload =
      $grpc.ClientMethod<$0.CompleteUploadRequest, $0.CompleteUploadResponse>(
          '/k1s0.system.file.v1.FileService/CompleteUpload',
          ($0.CompleteUploadRequest value) => value.writeToBuffer(),
          $0.CompleteUploadResponse.fromBuffer);
  static final _$generateDownloadUrl = $grpc.ClientMethod<
          $0.GenerateDownloadUrlRequest, $0.GenerateDownloadUrlResponse>(
      '/k1s0.system.file.v1.FileService/GenerateDownloadUrl',
      ($0.GenerateDownloadUrlRequest value) => value.writeToBuffer(),
      $0.GenerateDownloadUrlResponse.fromBuffer);
  static final _$updateFileTags =
      $grpc.ClientMethod<$0.UpdateFileTagsRequest, $0.UpdateFileTagsResponse>(
          '/k1s0.system.file.v1.FileService/UpdateFileTags',
          ($0.UpdateFileTagsRequest value) => value.writeToBuffer(),
          $0.UpdateFileTagsResponse.fromBuffer);
  static final _$deleteFile =
      $grpc.ClientMethod<$0.DeleteFileRequest, $0.DeleteFileResponse>(
          '/k1s0.system.file.v1.FileService/DeleteFile',
          ($0.DeleteFileRequest value) => value.writeToBuffer(),
          $0.DeleteFileResponse.fromBuffer);
}

@$pb.GrpcServiceName('k1s0.system.file.v1.FileService')
abstract class FileServiceBase extends $grpc.Service {
  $core.String get $name => 'k1s0.system.file.v1.FileService';

  FileServiceBase() {
    $addMethod($grpc.ServiceMethod<$0.GetFileMetadataRequest,
            $0.GetFileMetadataResponse>(
        'GetFileMetadata',
        getFileMetadata_Pre,
        false,
        false,
        ($core.List<$core.int> value) =>
            $0.GetFileMetadataRequest.fromBuffer(value),
        ($0.GetFileMetadataResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.ListFilesRequest, $0.ListFilesResponse>(
        'ListFiles',
        listFiles_Pre,
        false,
        false,
        ($core.List<$core.int> value) => $0.ListFilesRequest.fromBuffer(value),
        ($0.ListFilesResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.GenerateUploadUrlRequest,
            $0.GenerateUploadUrlResponse>(
        'GenerateUploadUrl',
        generateUploadUrl_Pre,
        false,
        false,
        ($core.List<$core.int> value) =>
            $0.GenerateUploadUrlRequest.fromBuffer(value),
        ($0.GenerateUploadUrlResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.CompleteUploadRequest,
            $0.CompleteUploadResponse>(
        'CompleteUpload',
        completeUpload_Pre,
        false,
        false,
        ($core.List<$core.int> value) =>
            $0.CompleteUploadRequest.fromBuffer(value),
        ($0.CompleteUploadResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.GenerateDownloadUrlRequest,
            $0.GenerateDownloadUrlResponse>(
        'GenerateDownloadUrl',
        generateDownloadUrl_Pre,
        false,
        false,
        ($core.List<$core.int> value) =>
            $0.GenerateDownloadUrlRequest.fromBuffer(value),
        ($0.GenerateDownloadUrlResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.UpdateFileTagsRequest,
            $0.UpdateFileTagsResponse>(
        'UpdateFileTags',
        updateFileTags_Pre,
        false,
        false,
        ($core.List<$core.int> value) =>
            $0.UpdateFileTagsRequest.fromBuffer(value),
        ($0.UpdateFileTagsResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.DeleteFileRequest, $0.DeleteFileResponse>(
        'DeleteFile',
        deleteFile_Pre,
        false,
        false,
        ($core.List<$core.int> value) => $0.DeleteFileRequest.fromBuffer(value),
        ($0.DeleteFileResponse value) => value.writeToBuffer()));
  }

  $async.Future<$0.GetFileMetadataResponse> getFileMetadata_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.GetFileMetadataRequest> $request) async {
    return getFileMetadata($call, await $request);
  }

  $async.Future<$0.GetFileMetadataResponse> getFileMetadata(
      $grpc.ServiceCall call, $0.GetFileMetadataRequest request);

  $async.Future<$0.ListFilesResponse> listFiles_Pre($grpc.ServiceCall $call,
      $async.Future<$0.ListFilesRequest> $request) async {
    return listFiles($call, await $request);
  }

  $async.Future<$0.ListFilesResponse> listFiles(
      $grpc.ServiceCall call, $0.ListFilesRequest request);

  $async.Future<$0.GenerateUploadUrlResponse> generateUploadUrl_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.GenerateUploadUrlRequest> $request) async {
    return generateUploadUrl($call, await $request);
  }

  $async.Future<$0.GenerateUploadUrlResponse> generateUploadUrl(
      $grpc.ServiceCall call, $0.GenerateUploadUrlRequest request);

  $async.Future<$0.CompleteUploadResponse> completeUpload_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.CompleteUploadRequest> $request) async {
    return completeUpload($call, await $request);
  }

  $async.Future<$0.CompleteUploadResponse> completeUpload(
      $grpc.ServiceCall call, $0.CompleteUploadRequest request);

  $async.Future<$0.GenerateDownloadUrlResponse> generateDownloadUrl_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.GenerateDownloadUrlRequest> $request) async {
    return generateDownloadUrl($call, await $request);
  }

  $async.Future<$0.GenerateDownloadUrlResponse> generateDownloadUrl(
      $grpc.ServiceCall call, $0.GenerateDownloadUrlRequest request);

  $async.Future<$0.UpdateFileTagsResponse> updateFileTags_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.UpdateFileTagsRequest> $request) async {
    return updateFileTags($call, await $request);
  }

  $async.Future<$0.UpdateFileTagsResponse> updateFileTags(
      $grpc.ServiceCall call, $0.UpdateFileTagsRequest request);

  $async.Future<$0.DeleteFileResponse> deleteFile_Pre($grpc.ServiceCall $call,
      $async.Future<$0.DeleteFileRequest> $request) async {
    return deleteFile($call, await $request);
  }

  $async.Future<$0.DeleteFileResponse> deleteFile(
      $grpc.ServiceCall call, $0.DeleteFileRequest request);
}
