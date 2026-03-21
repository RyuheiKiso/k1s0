// This is a generated file - do not edit.
//
// Generated from k1s0/event/service/inventory/v1/inventory_events.proto.

// @dart = 3.3

// ignore_for_file: annotate_overrides, camel_case_types, comment_references
// ignore_for_file: constant_identifier_names
// ignore_for_file: curly_braces_in_flow_control_structures
// ignore_for_file: deprecated_member_use_from_same_package, library_prefixes
// ignore_for_file: non_constant_identifier_names, prefer_relative_imports

import 'dart:core' as $core;

import 'package:protobuf/protobuf.dart' as $pb;

import '../../../../system/common/v1/event_metadata.pb.dart' as $0;
import '../../../../system/common/v1/types.pb.dart' as $1;

export 'package:protobuf/protobuf.dart' show GeneratedMessageGenericExtensions;

class InventoryReservedEvent extends $pb.GeneratedMessage {
  factory InventoryReservedEvent({
    $0.EventMetadata? metadata,
    $core.String? orderId,
    $core.String? productId,
    $core.int? quantity,
    $core.String? warehouseId,
    $1.Timestamp? reservedAt,
  }) {
    final result = create();
    if (metadata != null) result.metadata = metadata;
    if (orderId != null) result.orderId = orderId;
    if (productId != null) result.productId = productId;
    if (quantity != null) result.quantity = quantity;
    if (warehouseId != null) result.warehouseId = warehouseId;
    if (reservedAt != null) result.reservedAt = reservedAt;
    return result;
  }

  InventoryReservedEvent._();

  factory InventoryReservedEvent.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory InventoryReservedEvent.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'InventoryReservedEvent',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.event.service.inventory.v1'),
      createEmptyInstance: create)
    ..aOM<$0.EventMetadata>(1, _omitFieldNames ? '' : 'metadata',
        subBuilder: $0.EventMetadata.create)
    ..aOS(2, _omitFieldNames ? '' : 'orderId')
    ..aOS(3, _omitFieldNames ? '' : 'productId')
    ..aI(4, _omitFieldNames ? '' : 'quantity')
    ..aOS(5, _omitFieldNames ? '' : 'warehouseId')
    ..aOM<$1.Timestamp>(6, _omitFieldNames ? '' : 'reservedAt',
        subBuilder: $1.Timestamp.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  InventoryReservedEvent clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  InventoryReservedEvent copyWith(
          void Function(InventoryReservedEvent) updates) =>
      super.copyWith((message) => updates(message as InventoryReservedEvent))
          as InventoryReservedEvent;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static InventoryReservedEvent create() => InventoryReservedEvent._();
  @$core.override
  InventoryReservedEvent createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static InventoryReservedEvent getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<InventoryReservedEvent>(create);
  static InventoryReservedEvent? _defaultInstance;

  @$pb.TagNumber(1)
  $0.EventMetadata get metadata => $_getN(0);
  @$pb.TagNumber(1)
  set metadata($0.EventMetadata value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasMetadata() => $_has(0);
  @$pb.TagNumber(1)
  void clearMetadata() => $_clearField(1);
  @$pb.TagNumber(1)
  $0.EventMetadata ensureMetadata() => $_ensure(0);

  @$pb.TagNumber(2)
  $core.String get orderId => $_getSZ(1);
  @$pb.TagNumber(2)
  set orderId($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasOrderId() => $_has(1);
  @$pb.TagNumber(2)
  void clearOrderId() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.String get productId => $_getSZ(2);
  @$pb.TagNumber(3)
  set productId($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasProductId() => $_has(2);
  @$pb.TagNumber(3)
  void clearProductId() => $_clearField(3);

  @$pb.TagNumber(4)
  $core.int get quantity => $_getIZ(3);
  @$pb.TagNumber(4)
  set quantity($core.int value) => $_setSignedInt32(3, value);
  @$pb.TagNumber(4)
  $core.bool hasQuantity() => $_has(3);
  @$pb.TagNumber(4)
  void clearQuantity() => $_clearField(4);

  @$pb.TagNumber(5)
  $core.String get warehouseId => $_getSZ(4);
  @$pb.TagNumber(5)
  set warehouseId($core.String value) => $_setString(4, value);
  @$pb.TagNumber(5)
  $core.bool hasWarehouseId() => $_has(4);
  @$pb.TagNumber(5)
  void clearWarehouseId() => $_clearField(5);

  @$pb.TagNumber(6)
  $1.Timestamp get reservedAt => $_getN(5);
  @$pb.TagNumber(6)
  set reservedAt($1.Timestamp value) => $_setField(6, value);
  @$pb.TagNumber(6)
  $core.bool hasReservedAt() => $_has(5);
  @$pb.TagNumber(6)
  void clearReservedAt() => $_clearField(6);
  @$pb.TagNumber(6)
  $1.Timestamp ensureReservedAt() => $_ensure(5);
}

class InventoryReleasedEvent extends $pb.GeneratedMessage {
  factory InventoryReleasedEvent({
    $0.EventMetadata? metadata,
    $core.String? orderId,
    $core.String? productId,
    $core.int? quantity,
    $core.String? warehouseId,
    $core.String? reason,
    $1.Timestamp? releasedAt,
  }) {
    final result = create();
    if (metadata != null) result.metadata = metadata;
    if (orderId != null) result.orderId = orderId;
    if (productId != null) result.productId = productId;
    if (quantity != null) result.quantity = quantity;
    if (warehouseId != null) result.warehouseId = warehouseId;
    if (reason != null) result.reason = reason;
    if (releasedAt != null) result.releasedAt = releasedAt;
    return result;
  }

  InventoryReleasedEvent._();

  factory InventoryReleasedEvent.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory InventoryReleasedEvent.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'InventoryReleasedEvent',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.event.service.inventory.v1'),
      createEmptyInstance: create)
    ..aOM<$0.EventMetadata>(1, _omitFieldNames ? '' : 'metadata',
        subBuilder: $0.EventMetadata.create)
    ..aOS(2, _omitFieldNames ? '' : 'orderId')
    ..aOS(3, _omitFieldNames ? '' : 'productId')
    ..aI(4, _omitFieldNames ? '' : 'quantity')
    ..aOS(5, _omitFieldNames ? '' : 'warehouseId')
    ..aOS(6, _omitFieldNames ? '' : 'reason')
    ..aOM<$1.Timestamp>(7, _omitFieldNames ? '' : 'releasedAt',
        subBuilder: $1.Timestamp.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  InventoryReleasedEvent clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  InventoryReleasedEvent copyWith(
          void Function(InventoryReleasedEvent) updates) =>
      super.copyWith((message) => updates(message as InventoryReleasedEvent))
          as InventoryReleasedEvent;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static InventoryReleasedEvent create() => InventoryReleasedEvent._();
  @$core.override
  InventoryReleasedEvent createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static InventoryReleasedEvent getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<InventoryReleasedEvent>(create);
  static InventoryReleasedEvent? _defaultInstance;

  @$pb.TagNumber(1)
  $0.EventMetadata get metadata => $_getN(0);
  @$pb.TagNumber(1)
  set metadata($0.EventMetadata value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasMetadata() => $_has(0);
  @$pb.TagNumber(1)
  void clearMetadata() => $_clearField(1);
  @$pb.TagNumber(1)
  $0.EventMetadata ensureMetadata() => $_ensure(0);

  @$pb.TagNumber(2)
  $core.String get orderId => $_getSZ(1);
  @$pb.TagNumber(2)
  set orderId($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasOrderId() => $_has(1);
  @$pb.TagNumber(2)
  void clearOrderId() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.String get productId => $_getSZ(2);
  @$pb.TagNumber(3)
  set productId($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasProductId() => $_has(2);
  @$pb.TagNumber(3)
  void clearProductId() => $_clearField(3);

  @$pb.TagNumber(4)
  $core.int get quantity => $_getIZ(3);
  @$pb.TagNumber(4)
  set quantity($core.int value) => $_setSignedInt32(3, value);
  @$pb.TagNumber(4)
  $core.bool hasQuantity() => $_has(3);
  @$pb.TagNumber(4)
  void clearQuantity() => $_clearField(4);

  @$pb.TagNumber(5)
  $core.String get warehouseId => $_getSZ(4);
  @$pb.TagNumber(5)
  set warehouseId($core.String value) => $_setString(4, value);
  @$pb.TagNumber(5)
  $core.bool hasWarehouseId() => $_has(4);
  @$pb.TagNumber(5)
  void clearWarehouseId() => $_clearField(5);

  @$pb.TagNumber(6)
  $core.String get reason => $_getSZ(5);
  @$pb.TagNumber(6)
  set reason($core.String value) => $_setString(5, value);
  @$pb.TagNumber(6)
  $core.bool hasReason() => $_has(5);
  @$pb.TagNumber(6)
  void clearReason() => $_clearField(6);

  @$pb.TagNumber(7)
  $1.Timestamp get releasedAt => $_getN(6);
  @$pb.TagNumber(7)
  set releasedAt($1.Timestamp value) => $_setField(7, value);
  @$pb.TagNumber(7)
  $core.bool hasReleasedAt() => $_has(6);
  @$pb.TagNumber(7)
  void clearReleasedAt() => $_clearField(7);
  @$pb.TagNumber(7)
  $1.Timestamp ensureReleasedAt() => $_ensure(6);
}

const $core.bool _omitFieldNames =
    $core.bool.fromEnvironment('protobuf.omit_field_names');
const $core.bool _omitMessageNames =
    $core.bool.fromEnvironment('protobuf.omit_message_names');
