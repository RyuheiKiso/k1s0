// This is a generated file - do not edit.
//
// Generated from k1s0/business/accounting/domainmaster/v1/domain_master.proto.

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

import 'domain_master.pb.dart' as $0;

export 'domain_master.pb.dart';

@$pb.GrpcServiceName(
    'k1s0.business.accounting.domainmaster.v1.DomainMasterService')
class DomainMasterServiceClient extends $grpc.Client {
  /// The hostname for this service.
  static const $core.String defaultHost = '';

  /// OAuth scopes needed for the client.
  static const $core.List<$core.String> oauthScopes = [
    '',
  ];

  DomainMasterServiceClient(super.channel, {super.options, super.interceptors});

  /// カテゴリ操作
  $grpc.ResponseFuture<$0.ListCategoriesResponse> listCategories(
    $0.ListCategoriesRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$listCategories, request, options: options);
  }

  $grpc.ResponseFuture<$0.GetCategoryResponse> getCategory(
    $0.GetCategoryRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$getCategory, request, options: options);
  }

  $grpc.ResponseFuture<$0.CreateCategoryResponse> createCategory(
    $0.CreateCategoryRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$createCategory, request, options: options);
  }

  $grpc.ResponseFuture<$0.UpdateCategoryResponse> updateCategory(
    $0.UpdateCategoryRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$updateCategory, request, options: options);
  }

  $grpc.ResponseFuture<$0.DeleteCategoryResponse> deleteCategory(
    $0.DeleteCategoryRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$deleteCategory, request, options: options);
  }

  /// 項目操作
  $grpc.ResponseFuture<$0.ListItemsResponse> listItems(
    $0.ListItemsRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$listItems, request, options: options);
  }

  $grpc.ResponseFuture<$0.GetItemResponse> getItem(
    $0.GetItemRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$getItem, request, options: options);
  }

  $grpc.ResponseFuture<$0.CreateItemResponse> createItem(
    $0.CreateItemRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$createItem, request, options: options);
  }

  $grpc.ResponseFuture<$0.UpdateItemResponse> updateItem(
    $0.UpdateItemRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$updateItem, request, options: options);
  }

  $grpc.ResponseFuture<$0.DeleteItemResponse> deleteItem(
    $0.DeleteItemRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$deleteItem, request, options: options);
  }

  /// バージョン操作
  $grpc.ResponseFuture<$0.ListItemVersionsResponse> listItemVersions(
    $0.ListItemVersionsRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$listItemVersions, request, options: options);
  }

  /// テナント拡張操作
  $grpc.ResponseFuture<$0.GetTenantExtensionResponse> getTenantExtension(
    $0.GetTenantExtensionRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$getTenantExtension, request, options: options);
  }

  $grpc.ResponseFuture<$0.UpsertTenantExtensionResponse> upsertTenantExtension(
    $0.UpsertTenantExtensionRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$upsertTenantExtension, request, options: options);
  }

  $grpc.ResponseFuture<$0.DeleteTenantExtensionResponse> deleteTenantExtension(
    $0.DeleteTenantExtensionRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$deleteTenantExtension, request, options: options);
  }

  $grpc.ResponseFuture<$0.ListTenantItemsResponse> listTenantItems(
    $0.ListTenantItemsRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$listTenantItems, request, options: options);
  }

  // method descriptors

  static final _$listCategories = $grpc.ClientMethod<$0.ListCategoriesRequest,
          $0.ListCategoriesResponse>(
      '/k1s0.business.accounting.domainmaster.v1.DomainMasterService/ListCategories',
      ($0.ListCategoriesRequest value) => value.writeToBuffer(),
      $0.ListCategoriesResponse.fromBuffer);
  static final _$getCategory = $grpc.ClientMethod<$0.GetCategoryRequest,
          $0.GetCategoryResponse>(
      '/k1s0.business.accounting.domainmaster.v1.DomainMasterService/GetCategory',
      ($0.GetCategoryRequest value) => value.writeToBuffer(),
      $0.GetCategoryResponse.fromBuffer);
  static final _$createCategory = $grpc.ClientMethod<$0.CreateCategoryRequest,
          $0.CreateCategoryResponse>(
      '/k1s0.business.accounting.domainmaster.v1.DomainMasterService/CreateCategory',
      ($0.CreateCategoryRequest value) => value.writeToBuffer(),
      $0.CreateCategoryResponse.fromBuffer);
  static final _$updateCategory = $grpc.ClientMethod<$0.UpdateCategoryRequest,
          $0.UpdateCategoryResponse>(
      '/k1s0.business.accounting.domainmaster.v1.DomainMasterService/UpdateCategory',
      ($0.UpdateCategoryRequest value) => value.writeToBuffer(),
      $0.UpdateCategoryResponse.fromBuffer);
  static final _$deleteCategory = $grpc.ClientMethod<$0.DeleteCategoryRequest,
          $0.DeleteCategoryResponse>(
      '/k1s0.business.accounting.domainmaster.v1.DomainMasterService/DeleteCategory',
      ($0.DeleteCategoryRequest value) => value.writeToBuffer(),
      $0.DeleteCategoryResponse.fromBuffer);
  static final _$listItems = $grpc.ClientMethod<$0.ListItemsRequest,
          $0.ListItemsResponse>(
      '/k1s0.business.accounting.domainmaster.v1.DomainMasterService/ListItems',
      ($0.ListItemsRequest value) => value.writeToBuffer(),
      $0.ListItemsResponse.fromBuffer);
  static final _$getItem = $grpc.ClientMethod<$0.GetItemRequest,
          $0.GetItemResponse>(
      '/k1s0.business.accounting.domainmaster.v1.DomainMasterService/GetItem',
      ($0.GetItemRequest value) => value.writeToBuffer(),
      $0.GetItemResponse.fromBuffer);
  static final _$createItem = $grpc.ClientMethod<$0.CreateItemRequest,
          $0.CreateItemResponse>(
      '/k1s0.business.accounting.domainmaster.v1.DomainMasterService/CreateItem',
      ($0.CreateItemRequest value) => value.writeToBuffer(),
      $0.CreateItemResponse.fromBuffer);
  static final _$updateItem = $grpc.ClientMethod<$0.UpdateItemRequest,
          $0.UpdateItemResponse>(
      '/k1s0.business.accounting.domainmaster.v1.DomainMasterService/UpdateItem',
      ($0.UpdateItemRequest value) => value.writeToBuffer(),
      $0.UpdateItemResponse.fromBuffer);
  static final _$deleteItem = $grpc.ClientMethod<$0.DeleteItemRequest,
          $0.DeleteItemResponse>(
      '/k1s0.business.accounting.domainmaster.v1.DomainMasterService/DeleteItem',
      ($0.DeleteItemRequest value) => value.writeToBuffer(),
      $0.DeleteItemResponse.fromBuffer);
  static final _$listItemVersions = $grpc.ClientMethod<
          $0.ListItemVersionsRequest, $0.ListItemVersionsResponse>(
      '/k1s0.business.accounting.domainmaster.v1.DomainMasterService/ListItemVersions',
      ($0.ListItemVersionsRequest value) => value.writeToBuffer(),
      $0.ListItemVersionsResponse.fromBuffer);
  static final _$getTenantExtension = $grpc.ClientMethod<
          $0.GetTenantExtensionRequest, $0.GetTenantExtensionResponse>(
      '/k1s0.business.accounting.domainmaster.v1.DomainMasterService/GetTenantExtension',
      ($0.GetTenantExtensionRequest value) => value.writeToBuffer(),
      $0.GetTenantExtensionResponse.fromBuffer);
  static final _$upsertTenantExtension = $grpc.ClientMethod<
          $0.UpsertTenantExtensionRequest, $0.UpsertTenantExtensionResponse>(
      '/k1s0.business.accounting.domainmaster.v1.DomainMasterService/UpsertTenantExtension',
      ($0.UpsertTenantExtensionRequest value) => value.writeToBuffer(),
      $0.UpsertTenantExtensionResponse.fromBuffer);
  static final _$deleteTenantExtension = $grpc.ClientMethod<
          $0.DeleteTenantExtensionRequest, $0.DeleteTenantExtensionResponse>(
      '/k1s0.business.accounting.domainmaster.v1.DomainMasterService/DeleteTenantExtension',
      ($0.DeleteTenantExtensionRequest value) => value.writeToBuffer(),
      $0.DeleteTenantExtensionResponse.fromBuffer);
  static final _$listTenantItems = $grpc.ClientMethod<$0.ListTenantItemsRequest,
          $0.ListTenantItemsResponse>(
      '/k1s0.business.accounting.domainmaster.v1.DomainMasterService/ListTenantItems',
      ($0.ListTenantItemsRequest value) => value.writeToBuffer(),
      $0.ListTenantItemsResponse.fromBuffer);
}

@$pb.GrpcServiceName(
    'k1s0.business.accounting.domainmaster.v1.DomainMasterService')
abstract class DomainMasterServiceBase extends $grpc.Service {
  $core.String get $name =>
      'k1s0.business.accounting.domainmaster.v1.DomainMasterService';

  DomainMasterServiceBase() {
    $addMethod($grpc.ServiceMethod<$0.ListCategoriesRequest,
            $0.ListCategoriesResponse>(
        'ListCategories',
        listCategories_Pre,
        false,
        false,
        ($core.List<$core.int> value) =>
            $0.ListCategoriesRequest.fromBuffer(value),
        ($0.ListCategoriesResponse value) => value.writeToBuffer()));
    $addMethod(
        $grpc.ServiceMethod<$0.GetCategoryRequest, $0.GetCategoryResponse>(
            'GetCategory',
            getCategory_Pre,
            false,
            false,
            ($core.List<$core.int> value) =>
                $0.GetCategoryRequest.fromBuffer(value),
            ($0.GetCategoryResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.CreateCategoryRequest,
            $0.CreateCategoryResponse>(
        'CreateCategory',
        createCategory_Pre,
        false,
        false,
        ($core.List<$core.int> value) =>
            $0.CreateCategoryRequest.fromBuffer(value),
        ($0.CreateCategoryResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.UpdateCategoryRequest,
            $0.UpdateCategoryResponse>(
        'UpdateCategory',
        updateCategory_Pre,
        false,
        false,
        ($core.List<$core.int> value) =>
            $0.UpdateCategoryRequest.fromBuffer(value),
        ($0.UpdateCategoryResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.DeleteCategoryRequest,
            $0.DeleteCategoryResponse>(
        'DeleteCategory',
        deleteCategory_Pre,
        false,
        false,
        ($core.List<$core.int> value) =>
            $0.DeleteCategoryRequest.fromBuffer(value),
        ($0.DeleteCategoryResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.ListItemsRequest, $0.ListItemsResponse>(
        'ListItems',
        listItems_Pre,
        false,
        false,
        ($core.List<$core.int> value) => $0.ListItemsRequest.fromBuffer(value),
        ($0.ListItemsResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.GetItemRequest, $0.GetItemResponse>(
        'GetItem',
        getItem_Pre,
        false,
        false,
        ($core.List<$core.int> value) => $0.GetItemRequest.fromBuffer(value),
        ($0.GetItemResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.CreateItemRequest, $0.CreateItemResponse>(
        'CreateItem',
        createItem_Pre,
        false,
        false,
        ($core.List<$core.int> value) => $0.CreateItemRequest.fromBuffer(value),
        ($0.CreateItemResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.UpdateItemRequest, $0.UpdateItemResponse>(
        'UpdateItem',
        updateItem_Pre,
        false,
        false,
        ($core.List<$core.int> value) => $0.UpdateItemRequest.fromBuffer(value),
        ($0.UpdateItemResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.DeleteItemRequest, $0.DeleteItemResponse>(
        'DeleteItem',
        deleteItem_Pre,
        false,
        false,
        ($core.List<$core.int> value) => $0.DeleteItemRequest.fromBuffer(value),
        ($0.DeleteItemResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.ListItemVersionsRequest,
            $0.ListItemVersionsResponse>(
        'ListItemVersions',
        listItemVersions_Pre,
        false,
        false,
        ($core.List<$core.int> value) =>
            $0.ListItemVersionsRequest.fromBuffer(value),
        ($0.ListItemVersionsResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.GetTenantExtensionRequest,
            $0.GetTenantExtensionResponse>(
        'GetTenantExtension',
        getTenantExtension_Pre,
        false,
        false,
        ($core.List<$core.int> value) =>
            $0.GetTenantExtensionRequest.fromBuffer(value),
        ($0.GetTenantExtensionResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.UpsertTenantExtensionRequest,
            $0.UpsertTenantExtensionResponse>(
        'UpsertTenantExtension',
        upsertTenantExtension_Pre,
        false,
        false,
        ($core.List<$core.int> value) =>
            $0.UpsertTenantExtensionRequest.fromBuffer(value),
        ($0.UpsertTenantExtensionResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.DeleteTenantExtensionRequest,
            $0.DeleteTenantExtensionResponse>(
        'DeleteTenantExtension',
        deleteTenantExtension_Pre,
        false,
        false,
        ($core.List<$core.int> value) =>
            $0.DeleteTenantExtensionRequest.fromBuffer(value),
        ($0.DeleteTenantExtensionResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.ListTenantItemsRequest,
            $0.ListTenantItemsResponse>(
        'ListTenantItems',
        listTenantItems_Pre,
        false,
        false,
        ($core.List<$core.int> value) =>
            $0.ListTenantItemsRequest.fromBuffer(value),
        ($0.ListTenantItemsResponse value) => value.writeToBuffer()));
  }

  $async.Future<$0.ListCategoriesResponse> listCategories_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.ListCategoriesRequest> $request) async {
    return listCategories($call, await $request);
  }

  $async.Future<$0.ListCategoriesResponse> listCategories(
      $grpc.ServiceCall call, $0.ListCategoriesRequest request);

  $async.Future<$0.GetCategoryResponse> getCategory_Pre($grpc.ServiceCall $call,
      $async.Future<$0.GetCategoryRequest> $request) async {
    return getCategory($call, await $request);
  }

  $async.Future<$0.GetCategoryResponse> getCategory(
      $grpc.ServiceCall call, $0.GetCategoryRequest request);

  $async.Future<$0.CreateCategoryResponse> createCategory_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.CreateCategoryRequest> $request) async {
    return createCategory($call, await $request);
  }

  $async.Future<$0.CreateCategoryResponse> createCategory(
      $grpc.ServiceCall call, $0.CreateCategoryRequest request);

  $async.Future<$0.UpdateCategoryResponse> updateCategory_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.UpdateCategoryRequest> $request) async {
    return updateCategory($call, await $request);
  }

  $async.Future<$0.UpdateCategoryResponse> updateCategory(
      $grpc.ServiceCall call, $0.UpdateCategoryRequest request);

  $async.Future<$0.DeleteCategoryResponse> deleteCategory_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.DeleteCategoryRequest> $request) async {
    return deleteCategory($call, await $request);
  }

  $async.Future<$0.DeleteCategoryResponse> deleteCategory(
      $grpc.ServiceCall call, $0.DeleteCategoryRequest request);

  $async.Future<$0.ListItemsResponse> listItems_Pre($grpc.ServiceCall $call,
      $async.Future<$0.ListItemsRequest> $request) async {
    return listItems($call, await $request);
  }

  $async.Future<$0.ListItemsResponse> listItems(
      $grpc.ServiceCall call, $0.ListItemsRequest request);

  $async.Future<$0.GetItemResponse> getItem_Pre($grpc.ServiceCall $call,
      $async.Future<$0.GetItemRequest> $request) async {
    return getItem($call, await $request);
  }

  $async.Future<$0.GetItemResponse> getItem(
      $grpc.ServiceCall call, $0.GetItemRequest request);

  $async.Future<$0.CreateItemResponse> createItem_Pre($grpc.ServiceCall $call,
      $async.Future<$0.CreateItemRequest> $request) async {
    return createItem($call, await $request);
  }

  $async.Future<$0.CreateItemResponse> createItem(
      $grpc.ServiceCall call, $0.CreateItemRequest request);

  $async.Future<$0.UpdateItemResponse> updateItem_Pre($grpc.ServiceCall $call,
      $async.Future<$0.UpdateItemRequest> $request) async {
    return updateItem($call, await $request);
  }

  $async.Future<$0.UpdateItemResponse> updateItem(
      $grpc.ServiceCall call, $0.UpdateItemRequest request);

  $async.Future<$0.DeleteItemResponse> deleteItem_Pre($grpc.ServiceCall $call,
      $async.Future<$0.DeleteItemRequest> $request) async {
    return deleteItem($call, await $request);
  }

  $async.Future<$0.DeleteItemResponse> deleteItem(
      $grpc.ServiceCall call, $0.DeleteItemRequest request);

  $async.Future<$0.ListItemVersionsResponse> listItemVersions_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.ListItemVersionsRequest> $request) async {
    return listItemVersions($call, await $request);
  }

  $async.Future<$0.ListItemVersionsResponse> listItemVersions(
      $grpc.ServiceCall call, $0.ListItemVersionsRequest request);

  $async.Future<$0.GetTenantExtensionResponse> getTenantExtension_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.GetTenantExtensionRequest> $request) async {
    return getTenantExtension($call, await $request);
  }

  $async.Future<$0.GetTenantExtensionResponse> getTenantExtension(
      $grpc.ServiceCall call, $0.GetTenantExtensionRequest request);

  $async.Future<$0.UpsertTenantExtensionResponse> upsertTenantExtension_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.UpsertTenantExtensionRequest> $request) async {
    return upsertTenantExtension($call, await $request);
  }

  $async.Future<$0.UpsertTenantExtensionResponse> upsertTenantExtension(
      $grpc.ServiceCall call, $0.UpsertTenantExtensionRequest request);

  $async.Future<$0.DeleteTenantExtensionResponse> deleteTenantExtension_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.DeleteTenantExtensionRequest> $request) async {
    return deleteTenantExtension($call, await $request);
  }

  $async.Future<$0.DeleteTenantExtensionResponse> deleteTenantExtension(
      $grpc.ServiceCall call, $0.DeleteTenantExtensionRequest request);

  $async.Future<$0.ListTenantItemsResponse> listTenantItems_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.ListTenantItemsRequest> $request) async {
    return listTenantItems($call, await $request);
  }

  $async.Future<$0.ListTenantItemsResponse> listTenantItems(
      $grpc.ServiceCall call, $0.ListTenantItemsRequest request);
}
