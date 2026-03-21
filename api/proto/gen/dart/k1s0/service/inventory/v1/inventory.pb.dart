// This is a generated file - do not edit.
//
// Generated from k1s0/service/inventory/v1/inventory.proto.

// @dart = 3.3

// ignore_for_file: annotate_overrides, camel_case_types, comment_references
// ignore_for_file: constant_identifier_names
// ignore_for_file: curly_braces_in_flow_control_structures
// ignore_for_file: deprecated_member_use_from_same_package, library_prefixes
// ignore_for_file: non_constant_identifier_names, prefer_relative_imports

import 'dart:core' as $core;

import 'package:protobuf/protobuf.dart' as $pb;

import '../../../system/common/v1/types.pb.dart' as $1;

export 'package:protobuf/protobuf.dart' show GeneratedMessageGenericExtensions;

/// 在庫アイテム
class InventoryItem extends $pb.GeneratedMessage {
  factory InventoryItem({
    $core.String? id,
    $core.String? productId,
    $core.String? warehouseId,
    $core.int? qtyAvailable,
    $core.int? qtyReserved,
    $core.int? version,
    $1.Timestamp? createdAt,
    $1.Timestamp? updatedAt,
  }) {
    final result = create();
    if (id != null) result.id = id;
    if (productId != null) result.productId = productId;
    if (warehouseId != null) result.warehouseId = warehouseId;
    if (qtyAvailable != null) result.qtyAvailable = qtyAvailable;
    if (qtyReserved != null) result.qtyReserved = qtyReserved;
    if (version != null) result.version = version;
    if (createdAt != null) result.createdAt = createdAt;
    if (updatedAt != null) result.updatedAt = updatedAt;
    return result;
  }

  InventoryItem._();

  factory InventoryItem.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory InventoryItem.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'InventoryItem',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.service.inventory.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'id')
    ..aOS(2, _omitFieldNames ? '' : 'productId')
    ..aOS(3, _omitFieldNames ? '' : 'warehouseId')
    ..aI(4, _omitFieldNames ? '' : 'qtyAvailable')
    ..aI(5, _omitFieldNames ? '' : 'qtyReserved')
    ..aI(6, _omitFieldNames ? '' : 'version')
    ..aOM<$1.Timestamp>(7, _omitFieldNames ? '' : 'createdAt',
        subBuilder: $1.Timestamp.create)
    ..aOM<$1.Timestamp>(8, _omitFieldNames ? '' : 'updatedAt',
        subBuilder: $1.Timestamp.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  InventoryItem clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  InventoryItem copyWith(void Function(InventoryItem) updates) =>
      super.copyWith((message) => updates(message as InventoryItem))
          as InventoryItem;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static InventoryItem create() => InventoryItem._();
  @$core.override
  InventoryItem createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static InventoryItem getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<InventoryItem>(create);
  static InventoryItem? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get id => $_getSZ(0);
  @$pb.TagNumber(1)
  set id($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasId() => $_has(0);
  @$pb.TagNumber(1)
  void clearId() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get productId => $_getSZ(1);
  @$pb.TagNumber(2)
  set productId($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasProductId() => $_has(1);
  @$pb.TagNumber(2)
  void clearProductId() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.String get warehouseId => $_getSZ(2);
  @$pb.TagNumber(3)
  set warehouseId($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasWarehouseId() => $_has(2);
  @$pb.TagNumber(3)
  void clearWarehouseId() => $_clearField(3);

  @$pb.TagNumber(4)
  $core.int get qtyAvailable => $_getIZ(3);
  @$pb.TagNumber(4)
  set qtyAvailable($core.int value) => $_setSignedInt32(3, value);
  @$pb.TagNumber(4)
  $core.bool hasQtyAvailable() => $_has(3);
  @$pb.TagNumber(4)
  void clearQtyAvailable() => $_clearField(4);

  @$pb.TagNumber(5)
  $core.int get qtyReserved => $_getIZ(4);
  @$pb.TagNumber(5)
  set qtyReserved($core.int value) => $_setSignedInt32(4, value);
  @$pb.TagNumber(5)
  $core.bool hasQtyReserved() => $_has(4);
  @$pb.TagNumber(5)
  void clearQtyReserved() => $_clearField(5);

  @$pb.TagNumber(6)
  $core.int get version => $_getIZ(5);
  @$pb.TagNumber(6)
  set version($core.int value) => $_setSignedInt32(5, value);
  @$pb.TagNumber(6)
  $core.bool hasVersion() => $_has(5);
  @$pb.TagNumber(6)
  void clearVersion() => $_clearField(6);

  @$pb.TagNumber(7)
  $1.Timestamp get createdAt => $_getN(6);
  @$pb.TagNumber(7)
  set createdAt($1.Timestamp value) => $_setField(7, value);
  @$pb.TagNumber(7)
  $core.bool hasCreatedAt() => $_has(6);
  @$pb.TagNumber(7)
  void clearCreatedAt() => $_clearField(7);
  @$pb.TagNumber(7)
  $1.Timestamp ensureCreatedAt() => $_ensure(6);

  @$pb.TagNumber(8)
  $1.Timestamp get updatedAt => $_getN(7);
  @$pb.TagNumber(8)
  set updatedAt($1.Timestamp value) => $_setField(8, value);
  @$pb.TagNumber(8)
  $core.bool hasUpdatedAt() => $_has(7);
  @$pb.TagNumber(8)
  void clearUpdatedAt() => $_clearField(8);
  @$pb.TagNumber(8)
  $1.Timestamp ensureUpdatedAt() => $_ensure(7);
}

class ReserveStockRequest extends $pb.GeneratedMessage {
  factory ReserveStockRequest({
    $core.String? orderId,
    $core.String? productId,
    $core.String? warehouseId,
    $core.int? quantity,
  }) {
    final result = create();
    if (orderId != null) result.orderId = orderId;
    if (productId != null) result.productId = productId;
    if (warehouseId != null) result.warehouseId = warehouseId;
    if (quantity != null) result.quantity = quantity;
    return result;
  }

  ReserveStockRequest._();

  factory ReserveStockRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory ReserveStockRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ReserveStockRequest',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.service.inventory.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'orderId')
    ..aOS(2, _omitFieldNames ? '' : 'productId')
    ..aOS(3, _omitFieldNames ? '' : 'warehouseId')
    ..aI(4, _omitFieldNames ? '' : 'quantity')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ReserveStockRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ReserveStockRequest copyWith(void Function(ReserveStockRequest) updates) =>
      super.copyWith((message) => updates(message as ReserveStockRequest))
          as ReserveStockRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static ReserveStockRequest create() => ReserveStockRequest._();
  @$core.override
  ReserveStockRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static ReserveStockRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ReserveStockRequest>(create);
  static ReserveStockRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get orderId => $_getSZ(0);
  @$pb.TagNumber(1)
  set orderId($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasOrderId() => $_has(0);
  @$pb.TagNumber(1)
  void clearOrderId() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get productId => $_getSZ(1);
  @$pb.TagNumber(2)
  set productId($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasProductId() => $_has(1);
  @$pb.TagNumber(2)
  void clearProductId() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.String get warehouseId => $_getSZ(2);
  @$pb.TagNumber(3)
  set warehouseId($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasWarehouseId() => $_has(2);
  @$pb.TagNumber(3)
  void clearWarehouseId() => $_clearField(3);

  @$pb.TagNumber(4)
  $core.int get quantity => $_getIZ(3);
  @$pb.TagNumber(4)
  set quantity($core.int value) => $_setSignedInt32(3, value);
  @$pb.TagNumber(4)
  $core.bool hasQuantity() => $_has(3);
  @$pb.TagNumber(4)
  void clearQuantity() => $_clearField(4);
}

class ReserveStockResponse extends $pb.GeneratedMessage {
  factory ReserveStockResponse({
    InventoryItem? item,
  }) {
    final result = create();
    if (item != null) result.item = item;
    return result;
  }

  ReserveStockResponse._();

  factory ReserveStockResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory ReserveStockResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ReserveStockResponse',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.service.inventory.v1'),
      createEmptyInstance: create)
    ..aOM<InventoryItem>(1, _omitFieldNames ? '' : 'item',
        subBuilder: InventoryItem.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ReserveStockResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ReserveStockResponse copyWith(void Function(ReserveStockResponse) updates) =>
      super.copyWith((message) => updates(message as ReserveStockResponse))
          as ReserveStockResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static ReserveStockResponse create() => ReserveStockResponse._();
  @$core.override
  ReserveStockResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static ReserveStockResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ReserveStockResponse>(create);
  static ReserveStockResponse? _defaultInstance;

  @$pb.TagNumber(1)
  InventoryItem get item => $_getN(0);
  @$pb.TagNumber(1)
  set item(InventoryItem value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasItem() => $_has(0);
  @$pb.TagNumber(1)
  void clearItem() => $_clearField(1);
  @$pb.TagNumber(1)
  InventoryItem ensureItem() => $_ensure(0);
}

class ReleaseStockRequest extends $pb.GeneratedMessage {
  factory ReleaseStockRequest({
    $core.String? orderId,
    $core.String? productId,
    $core.String? warehouseId,
    $core.int? quantity,
    $core.String? reason,
  }) {
    final result = create();
    if (orderId != null) result.orderId = orderId;
    if (productId != null) result.productId = productId;
    if (warehouseId != null) result.warehouseId = warehouseId;
    if (quantity != null) result.quantity = quantity;
    if (reason != null) result.reason = reason;
    return result;
  }

  ReleaseStockRequest._();

  factory ReleaseStockRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory ReleaseStockRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ReleaseStockRequest',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.service.inventory.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'orderId')
    ..aOS(2, _omitFieldNames ? '' : 'productId')
    ..aOS(3, _omitFieldNames ? '' : 'warehouseId')
    ..aI(4, _omitFieldNames ? '' : 'quantity')
    ..aOS(5, _omitFieldNames ? '' : 'reason')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ReleaseStockRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ReleaseStockRequest copyWith(void Function(ReleaseStockRequest) updates) =>
      super.copyWith((message) => updates(message as ReleaseStockRequest))
          as ReleaseStockRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static ReleaseStockRequest create() => ReleaseStockRequest._();
  @$core.override
  ReleaseStockRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static ReleaseStockRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ReleaseStockRequest>(create);
  static ReleaseStockRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get orderId => $_getSZ(0);
  @$pb.TagNumber(1)
  set orderId($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasOrderId() => $_has(0);
  @$pb.TagNumber(1)
  void clearOrderId() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get productId => $_getSZ(1);
  @$pb.TagNumber(2)
  set productId($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasProductId() => $_has(1);
  @$pb.TagNumber(2)
  void clearProductId() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.String get warehouseId => $_getSZ(2);
  @$pb.TagNumber(3)
  set warehouseId($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasWarehouseId() => $_has(2);
  @$pb.TagNumber(3)
  void clearWarehouseId() => $_clearField(3);

  @$pb.TagNumber(4)
  $core.int get quantity => $_getIZ(3);
  @$pb.TagNumber(4)
  set quantity($core.int value) => $_setSignedInt32(3, value);
  @$pb.TagNumber(4)
  $core.bool hasQuantity() => $_has(3);
  @$pb.TagNumber(4)
  void clearQuantity() => $_clearField(4);

  @$pb.TagNumber(5)
  $core.String get reason => $_getSZ(4);
  @$pb.TagNumber(5)
  set reason($core.String value) => $_setString(4, value);
  @$pb.TagNumber(5)
  $core.bool hasReason() => $_has(4);
  @$pb.TagNumber(5)
  void clearReason() => $_clearField(5);
}

class ReleaseStockResponse extends $pb.GeneratedMessage {
  factory ReleaseStockResponse({
    InventoryItem? item,
  }) {
    final result = create();
    if (item != null) result.item = item;
    return result;
  }

  ReleaseStockResponse._();

  factory ReleaseStockResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory ReleaseStockResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ReleaseStockResponse',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.service.inventory.v1'),
      createEmptyInstance: create)
    ..aOM<InventoryItem>(1, _omitFieldNames ? '' : 'item',
        subBuilder: InventoryItem.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ReleaseStockResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ReleaseStockResponse copyWith(void Function(ReleaseStockResponse) updates) =>
      super.copyWith((message) => updates(message as ReleaseStockResponse))
          as ReleaseStockResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static ReleaseStockResponse create() => ReleaseStockResponse._();
  @$core.override
  ReleaseStockResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static ReleaseStockResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ReleaseStockResponse>(create);
  static ReleaseStockResponse? _defaultInstance;

  @$pb.TagNumber(1)
  InventoryItem get item => $_getN(0);
  @$pb.TagNumber(1)
  set item(InventoryItem value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasItem() => $_has(0);
  @$pb.TagNumber(1)
  void clearItem() => $_clearField(1);
  @$pb.TagNumber(1)
  InventoryItem ensureItem() => $_ensure(0);
}

class GetInventoryRequest extends $pb.GeneratedMessage {
  factory GetInventoryRequest({
    $core.String? inventoryId,
  }) {
    final result = create();
    if (inventoryId != null) result.inventoryId = inventoryId;
    return result;
  }

  GetInventoryRequest._();

  factory GetInventoryRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory GetInventoryRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'GetInventoryRequest',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.service.inventory.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'inventoryId')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetInventoryRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetInventoryRequest copyWith(void Function(GetInventoryRequest) updates) =>
      super.copyWith((message) => updates(message as GetInventoryRequest))
          as GetInventoryRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static GetInventoryRequest create() => GetInventoryRequest._();
  @$core.override
  GetInventoryRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static GetInventoryRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<GetInventoryRequest>(create);
  static GetInventoryRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get inventoryId => $_getSZ(0);
  @$pb.TagNumber(1)
  set inventoryId($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasInventoryId() => $_has(0);
  @$pb.TagNumber(1)
  void clearInventoryId() => $_clearField(1);
}

class GetInventoryResponse extends $pb.GeneratedMessage {
  factory GetInventoryResponse({
    InventoryItem? item,
  }) {
    final result = create();
    if (item != null) result.item = item;
    return result;
  }

  GetInventoryResponse._();

  factory GetInventoryResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory GetInventoryResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'GetInventoryResponse',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.service.inventory.v1'),
      createEmptyInstance: create)
    ..aOM<InventoryItem>(1, _omitFieldNames ? '' : 'item',
        subBuilder: InventoryItem.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetInventoryResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetInventoryResponse copyWith(void Function(GetInventoryResponse) updates) =>
      super.copyWith((message) => updates(message as GetInventoryResponse))
          as GetInventoryResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static GetInventoryResponse create() => GetInventoryResponse._();
  @$core.override
  GetInventoryResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static GetInventoryResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<GetInventoryResponse>(create);
  static GetInventoryResponse? _defaultInstance;

  @$pb.TagNumber(1)
  InventoryItem get item => $_getN(0);
  @$pb.TagNumber(1)
  set item(InventoryItem value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasItem() => $_has(0);
  @$pb.TagNumber(1)
  void clearItem() => $_clearField(1);
  @$pb.TagNumber(1)
  InventoryItem ensureItem() => $_ensure(0);
}

class ListInventoryRequest extends $pb.GeneratedMessage {
  factory ListInventoryRequest({
    $core.String? productId,
    $core.String? warehouseId,
    $1.Pagination? pagination,
  }) {
    final result = create();
    if (productId != null) result.productId = productId;
    if (warehouseId != null) result.warehouseId = warehouseId;
    if (pagination != null) result.pagination = pagination;
    return result;
  }

  ListInventoryRequest._();

  factory ListInventoryRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory ListInventoryRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ListInventoryRequest',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.service.inventory.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'productId')
    ..aOS(2, _omitFieldNames ? '' : 'warehouseId')
    ..aOM<$1.Pagination>(3, _omitFieldNames ? '' : 'pagination',
        subBuilder: $1.Pagination.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListInventoryRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListInventoryRequest copyWith(void Function(ListInventoryRequest) updates) =>
      super.copyWith((message) => updates(message as ListInventoryRequest))
          as ListInventoryRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static ListInventoryRequest create() => ListInventoryRequest._();
  @$core.override
  ListInventoryRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static ListInventoryRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ListInventoryRequest>(create);
  static ListInventoryRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get productId => $_getSZ(0);
  @$pb.TagNumber(1)
  set productId($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasProductId() => $_has(0);
  @$pb.TagNumber(1)
  void clearProductId() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get warehouseId => $_getSZ(1);
  @$pb.TagNumber(2)
  set warehouseId($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasWarehouseId() => $_has(1);
  @$pb.TagNumber(2)
  void clearWarehouseId() => $_clearField(2);

  @$pb.TagNumber(3)
  $1.Pagination get pagination => $_getN(2);
  @$pb.TagNumber(3)
  set pagination($1.Pagination value) => $_setField(3, value);
  @$pb.TagNumber(3)
  $core.bool hasPagination() => $_has(2);
  @$pb.TagNumber(3)
  void clearPagination() => $_clearField(3);
  @$pb.TagNumber(3)
  $1.Pagination ensurePagination() => $_ensure(2);
}

class ListInventoryResponse extends $pb.GeneratedMessage {
  factory ListInventoryResponse({
    $core.Iterable<InventoryItem>? items,
    $1.PaginationResult? pagination,
  }) {
    final result = create();
    if (items != null) result.items.addAll(items);
    if (pagination != null) result.pagination = pagination;
    return result;
  }

  ListInventoryResponse._();

  factory ListInventoryResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory ListInventoryResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ListInventoryResponse',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.service.inventory.v1'),
      createEmptyInstance: create)
    ..pPM<InventoryItem>(1, _omitFieldNames ? '' : 'items',
        subBuilder: InventoryItem.create)
    ..aOM<$1.PaginationResult>(3, _omitFieldNames ? '' : 'pagination',
        subBuilder: $1.PaginationResult.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListInventoryResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListInventoryResponse copyWith(
          void Function(ListInventoryResponse) updates) =>
      super.copyWith((message) => updates(message as ListInventoryResponse))
          as ListInventoryResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static ListInventoryResponse create() => ListInventoryResponse._();
  @$core.override
  ListInventoryResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static ListInventoryResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ListInventoryResponse>(create);
  static ListInventoryResponse? _defaultInstance;

  @$pb.TagNumber(1)
  $pb.PbList<InventoryItem> get items => $_getList(0);

  @$pb.TagNumber(3)
  $1.PaginationResult get pagination => $_getN(1);
  @$pb.TagNumber(3)
  set pagination($1.PaginationResult value) => $_setField(3, value);
  @$pb.TagNumber(3)
  $core.bool hasPagination() => $_has(1);
  @$pb.TagNumber(3)
  void clearPagination() => $_clearField(3);
  @$pb.TagNumber(3)
  $1.PaginationResult ensurePagination() => $_ensure(1);
}

class UpdateStockRequest extends $pb.GeneratedMessage {
  factory UpdateStockRequest({
    $core.String? inventoryId,
    $core.int? qtyAvailable,
    $core.int? expectedVersion,
  }) {
    final result = create();
    if (inventoryId != null) result.inventoryId = inventoryId;
    if (qtyAvailable != null) result.qtyAvailable = qtyAvailable;
    if (expectedVersion != null) result.expectedVersion = expectedVersion;
    return result;
  }

  UpdateStockRequest._();

  factory UpdateStockRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory UpdateStockRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'UpdateStockRequest',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.service.inventory.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'inventoryId')
    ..aI(2, _omitFieldNames ? '' : 'qtyAvailable')
    ..aI(3, _omitFieldNames ? '' : 'expectedVersion')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  UpdateStockRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  UpdateStockRequest copyWith(void Function(UpdateStockRequest) updates) =>
      super.copyWith((message) => updates(message as UpdateStockRequest))
          as UpdateStockRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static UpdateStockRequest create() => UpdateStockRequest._();
  @$core.override
  UpdateStockRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static UpdateStockRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<UpdateStockRequest>(create);
  static UpdateStockRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get inventoryId => $_getSZ(0);
  @$pb.TagNumber(1)
  set inventoryId($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasInventoryId() => $_has(0);
  @$pb.TagNumber(1)
  void clearInventoryId() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.int get qtyAvailable => $_getIZ(1);
  @$pb.TagNumber(2)
  set qtyAvailable($core.int value) => $_setSignedInt32(1, value);
  @$pb.TagNumber(2)
  $core.bool hasQtyAvailable() => $_has(1);
  @$pb.TagNumber(2)
  void clearQtyAvailable() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.int get expectedVersion => $_getIZ(2);
  @$pb.TagNumber(3)
  set expectedVersion($core.int value) => $_setSignedInt32(2, value);
  @$pb.TagNumber(3)
  $core.bool hasExpectedVersion() => $_has(2);
  @$pb.TagNumber(3)
  void clearExpectedVersion() => $_clearField(3);
}

class UpdateStockResponse extends $pb.GeneratedMessage {
  factory UpdateStockResponse({
    InventoryItem? item,
  }) {
    final result = create();
    if (item != null) result.item = item;
    return result;
  }

  UpdateStockResponse._();

  factory UpdateStockResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory UpdateStockResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'UpdateStockResponse',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.service.inventory.v1'),
      createEmptyInstance: create)
    ..aOM<InventoryItem>(1, _omitFieldNames ? '' : 'item',
        subBuilder: InventoryItem.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  UpdateStockResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  UpdateStockResponse copyWith(void Function(UpdateStockResponse) updates) =>
      super.copyWith((message) => updates(message as UpdateStockResponse))
          as UpdateStockResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static UpdateStockResponse create() => UpdateStockResponse._();
  @$core.override
  UpdateStockResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static UpdateStockResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<UpdateStockResponse>(create);
  static UpdateStockResponse? _defaultInstance;

  @$pb.TagNumber(1)
  InventoryItem get item => $_getN(0);
  @$pb.TagNumber(1)
  set item(InventoryItem value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasItem() => $_has(0);
  @$pb.TagNumber(1)
  void clearItem() => $_clearField(1);
  @$pb.TagNumber(1)
  InventoryItem ensureItem() => $_ensure(0);
}

const $core.bool _omitFieldNames =
    $core.bool.fromEnvironment('protobuf.omit_field_names');
const $core.bool _omitMessageNames =
    $core.bool.fromEnvironment('protobuf.omit_message_names');
