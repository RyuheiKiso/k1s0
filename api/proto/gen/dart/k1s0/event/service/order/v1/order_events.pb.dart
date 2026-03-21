// This is a generated file - do not edit.
//
// Generated from k1s0/event/service/order/v1/order_events.proto.

// @dart = 3.3

// ignore_for_file: annotate_overrides, camel_case_types, comment_references
// ignore_for_file: constant_identifier_names
// ignore_for_file: curly_braces_in_flow_control_structures
// ignore_for_file: deprecated_member_use_from_same_package, library_prefixes
// ignore_for_file: non_constant_identifier_names, prefer_relative_imports

import 'dart:core' as $core;

import 'package:fixnum/fixnum.dart' as $fixnum;
import 'package:protobuf/protobuf.dart' as $pb;

import '../../../../system/common/v1/event_metadata.pb.dart' as $0;

export 'package:protobuf/protobuf.dart' show GeneratedMessageGenericExtensions;

class OrderCreatedEvent extends $pb.GeneratedMessage {
  factory OrderCreatedEvent({
    $0.EventMetadata? metadata,
    $core.String? orderId,
    $core.String? customerId,
    $core.Iterable<OrderItem>? items,
    $fixnum.Int64? totalAmount,
    $core.String? currency,
  }) {
    final result = create();
    if (metadata != null) result.metadata = metadata;
    if (orderId != null) result.orderId = orderId;
    if (customerId != null) result.customerId = customerId;
    if (items != null) result.items.addAll(items);
    if (totalAmount != null) result.totalAmount = totalAmount;
    if (currency != null) result.currency = currency;
    return result;
  }

  OrderCreatedEvent._();

  factory OrderCreatedEvent.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory OrderCreatedEvent.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'OrderCreatedEvent',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.event.service.order.v1'),
      createEmptyInstance: create)
    ..aOM<$0.EventMetadata>(1, _omitFieldNames ? '' : 'metadata',
        subBuilder: $0.EventMetadata.create)
    ..aOS(2, _omitFieldNames ? '' : 'orderId')
    ..aOS(3, _omitFieldNames ? '' : 'customerId')
    ..pPM<OrderItem>(4, _omitFieldNames ? '' : 'items',
        subBuilder: OrderItem.create)
    ..aInt64(5, _omitFieldNames ? '' : 'totalAmount')
    ..aOS(6, _omitFieldNames ? '' : 'currency')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  OrderCreatedEvent clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  OrderCreatedEvent copyWith(void Function(OrderCreatedEvent) updates) =>
      super.copyWith((message) => updates(message as OrderCreatedEvent))
          as OrderCreatedEvent;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static OrderCreatedEvent create() => OrderCreatedEvent._();
  @$core.override
  OrderCreatedEvent createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static OrderCreatedEvent getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<OrderCreatedEvent>(create);
  static OrderCreatedEvent? _defaultInstance;

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
  $core.String get customerId => $_getSZ(2);
  @$pb.TagNumber(3)
  set customerId($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasCustomerId() => $_has(2);
  @$pb.TagNumber(3)
  void clearCustomerId() => $_clearField(3);

  @$pb.TagNumber(4)
  $pb.PbList<OrderItem> get items => $_getList(3);

  @$pb.TagNumber(5)
  $fixnum.Int64 get totalAmount => $_getI64(4);
  @$pb.TagNumber(5)
  set totalAmount($fixnum.Int64 value) => $_setInt64(4, value);
  @$pb.TagNumber(5)
  $core.bool hasTotalAmount() => $_has(4);
  @$pb.TagNumber(5)
  void clearTotalAmount() => $_clearField(5);

  @$pb.TagNumber(6)
  $core.String get currency => $_getSZ(5);
  @$pb.TagNumber(6)
  set currency($core.String value) => $_setString(5, value);
  @$pb.TagNumber(6)
  $core.bool hasCurrency() => $_has(5);
  @$pb.TagNumber(6)
  void clearCurrency() => $_clearField(6);
}

class OrderItem extends $pb.GeneratedMessage {
  factory OrderItem({
    $core.String? productId,
    $core.int? quantity,
    $fixnum.Int64? unitPrice,
  }) {
    final result = create();
    if (productId != null) result.productId = productId;
    if (quantity != null) result.quantity = quantity;
    if (unitPrice != null) result.unitPrice = unitPrice;
    return result;
  }

  OrderItem._();

  factory OrderItem.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory OrderItem.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'OrderItem',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.event.service.order.v1'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'productId')
    ..aI(2, _omitFieldNames ? '' : 'quantity')
    ..aInt64(3, _omitFieldNames ? '' : 'unitPrice')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  OrderItem clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  OrderItem copyWith(void Function(OrderItem) updates) =>
      super.copyWith((message) => updates(message as OrderItem)) as OrderItem;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static OrderItem create() => OrderItem._();
  @$core.override
  OrderItem createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static OrderItem getDefault() =>
      _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<OrderItem>(create);
  static OrderItem? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get productId => $_getSZ(0);
  @$pb.TagNumber(1)
  set productId($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasProductId() => $_has(0);
  @$pb.TagNumber(1)
  void clearProductId() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.int get quantity => $_getIZ(1);
  @$pb.TagNumber(2)
  set quantity($core.int value) => $_setSignedInt32(1, value);
  @$pb.TagNumber(2)
  $core.bool hasQuantity() => $_has(1);
  @$pb.TagNumber(2)
  void clearQuantity() => $_clearField(2);

  @$pb.TagNumber(3)
  $fixnum.Int64 get unitPrice => $_getI64(2);
  @$pb.TagNumber(3)
  set unitPrice($fixnum.Int64 value) => $_setInt64(2, value);
  @$pb.TagNumber(3)
  $core.bool hasUnitPrice() => $_has(2);
  @$pb.TagNumber(3)
  void clearUnitPrice() => $_clearField(3);
}

class OrderUpdatedEvent extends $pb.GeneratedMessage {
  factory OrderUpdatedEvent({
    $0.EventMetadata? metadata,
    $core.String? orderId,
    $core.String? userId,
    $core.Iterable<OrderItem>? items,
    $fixnum.Int64? totalAmount,
    $core.String? status,
  }) {
    final result = create();
    if (metadata != null) result.metadata = metadata;
    if (orderId != null) result.orderId = orderId;
    if (userId != null) result.userId = userId;
    if (items != null) result.items.addAll(items);
    if (totalAmount != null) result.totalAmount = totalAmount;
    if (status != null) result.status = status;
    return result;
  }

  OrderUpdatedEvent._();

  factory OrderUpdatedEvent.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory OrderUpdatedEvent.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'OrderUpdatedEvent',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.event.service.order.v1'),
      createEmptyInstance: create)
    ..aOM<$0.EventMetadata>(1, _omitFieldNames ? '' : 'metadata',
        subBuilder: $0.EventMetadata.create)
    ..aOS(2, _omitFieldNames ? '' : 'orderId')
    ..aOS(3, _omitFieldNames ? '' : 'userId')
    ..pPM<OrderItem>(4, _omitFieldNames ? '' : 'items',
        subBuilder: OrderItem.create)
    ..aInt64(5, _omitFieldNames ? '' : 'totalAmount')
    ..aOS(6, _omitFieldNames ? '' : 'status')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  OrderUpdatedEvent clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  OrderUpdatedEvent copyWith(void Function(OrderUpdatedEvent) updates) =>
      super.copyWith((message) => updates(message as OrderUpdatedEvent))
          as OrderUpdatedEvent;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static OrderUpdatedEvent create() => OrderUpdatedEvent._();
  @$core.override
  OrderUpdatedEvent createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static OrderUpdatedEvent getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<OrderUpdatedEvent>(create);
  static OrderUpdatedEvent? _defaultInstance;

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
  $core.String get userId => $_getSZ(2);
  @$pb.TagNumber(3)
  set userId($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasUserId() => $_has(2);
  @$pb.TagNumber(3)
  void clearUserId() => $_clearField(3);

  @$pb.TagNumber(4)
  $pb.PbList<OrderItem> get items => $_getList(3);

  @$pb.TagNumber(5)
  $fixnum.Int64 get totalAmount => $_getI64(4);
  @$pb.TagNumber(5)
  set totalAmount($fixnum.Int64 value) => $_setInt64(4, value);
  @$pb.TagNumber(5)
  $core.bool hasTotalAmount() => $_has(4);
  @$pb.TagNumber(5)
  void clearTotalAmount() => $_clearField(5);

  @$pb.TagNumber(6)
  $core.String get status => $_getSZ(5);
  @$pb.TagNumber(6)
  set status($core.String value) => $_setString(5, value);
  @$pb.TagNumber(6)
  $core.bool hasStatus() => $_has(5);
  @$pb.TagNumber(6)
  void clearStatus() => $_clearField(6);
}

class OrderCancelledEvent extends $pb.GeneratedMessage {
  factory OrderCancelledEvent({
    $0.EventMetadata? metadata,
    $core.String? orderId,
    $core.String? userId,
    $core.String? reason,
  }) {
    final result = create();
    if (metadata != null) result.metadata = metadata;
    if (orderId != null) result.orderId = orderId;
    if (userId != null) result.userId = userId;
    if (reason != null) result.reason = reason;
    return result;
  }

  OrderCancelledEvent._();

  factory OrderCancelledEvent.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory OrderCancelledEvent.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'OrderCancelledEvent',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.event.service.order.v1'),
      createEmptyInstance: create)
    ..aOM<$0.EventMetadata>(1, _omitFieldNames ? '' : 'metadata',
        subBuilder: $0.EventMetadata.create)
    ..aOS(2, _omitFieldNames ? '' : 'orderId')
    ..aOS(3, _omitFieldNames ? '' : 'userId')
    ..aOS(4, _omitFieldNames ? '' : 'reason')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  OrderCancelledEvent clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  OrderCancelledEvent copyWith(void Function(OrderCancelledEvent) updates) =>
      super.copyWith((message) => updates(message as OrderCancelledEvent))
          as OrderCancelledEvent;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static OrderCancelledEvent create() => OrderCancelledEvent._();
  @$core.override
  OrderCancelledEvent createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static OrderCancelledEvent getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<OrderCancelledEvent>(create);
  static OrderCancelledEvent? _defaultInstance;

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
  $core.String get userId => $_getSZ(2);
  @$pb.TagNumber(3)
  set userId($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasUserId() => $_has(2);
  @$pb.TagNumber(3)
  void clearUserId() => $_clearField(3);

  @$pb.TagNumber(4)
  $core.String get reason => $_getSZ(3);
  @$pb.TagNumber(4)
  set reason($core.String value) => $_setString(3, value);
  @$pb.TagNumber(4)
  $core.bool hasReason() => $_has(3);
  @$pb.TagNumber(4)
  void clearReason() => $_clearField(4);
}

const $core.bool _omitFieldNames =
    $core.bool.fromEnvironment('protobuf.omit_field_names');
const $core.bool _omitMessageNames =
    $core.bool.fromEnvironment('protobuf.omit_message_names');
