// This is a generated file - do not edit.
//
// Generated from k1s0/event/service/payment/v1/payment_events.proto.

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
import '../../../../system/common/v1/types.pb.dart' as $1;

export 'package:protobuf/protobuf.dart' show GeneratedMessageGenericExtensions;

class PaymentInitiatedEvent extends $pb.GeneratedMessage {
  factory PaymentInitiatedEvent({
    $0.EventMetadata? metadata,
    $core.String? paymentId,
    $core.String? orderId,
    $core.String? customerId,
    $fixnum.Int64? amount,
    $core.String? currency,
    $core.String? paymentMethod,
    $1.Timestamp? initiatedAt,
  }) {
    final result = create();
    if (metadata != null) result.metadata = metadata;
    if (paymentId != null) result.paymentId = paymentId;
    if (orderId != null) result.orderId = orderId;
    if (customerId != null) result.customerId = customerId;
    if (amount != null) result.amount = amount;
    if (currency != null) result.currency = currency;
    if (paymentMethod != null) result.paymentMethod = paymentMethod;
    if (initiatedAt != null) result.initiatedAt = initiatedAt;
    return result;
  }

  PaymentInitiatedEvent._();

  factory PaymentInitiatedEvent.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory PaymentInitiatedEvent.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'PaymentInitiatedEvent',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.event.service.payment.v1'),
      createEmptyInstance: create)
    ..aOM<$0.EventMetadata>(1, _omitFieldNames ? '' : 'metadata',
        subBuilder: $0.EventMetadata.create)
    ..aOS(2, _omitFieldNames ? '' : 'paymentId')
    ..aOS(3, _omitFieldNames ? '' : 'orderId')
    ..aOS(4, _omitFieldNames ? '' : 'customerId')
    ..aInt64(5, _omitFieldNames ? '' : 'amount')
    ..aOS(6, _omitFieldNames ? '' : 'currency')
    ..aOS(7, _omitFieldNames ? '' : 'paymentMethod')
    ..aOM<$1.Timestamp>(8, _omitFieldNames ? '' : 'initiatedAt',
        subBuilder: $1.Timestamp.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  PaymentInitiatedEvent clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  PaymentInitiatedEvent copyWith(
          void Function(PaymentInitiatedEvent) updates) =>
      super.copyWith((message) => updates(message as PaymentInitiatedEvent))
          as PaymentInitiatedEvent;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static PaymentInitiatedEvent create() => PaymentInitiatedEvent._();
  @$core.override
  PaymentInitiatedEvent createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static PaymentInitiatedEvent getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<PaymentInitiatedEvent>(create);
  static PaymentInitiatedEvent? _defaultInstance;

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
  $core.String get paymentId => $_getSZ(1);
  @$pb.TagNumber(2)
  set paymentId($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasPaymentId() => $_has(1);
  @$pb.TagNumber(2)
  void clearPaymentId() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.String get orderId => $_getSZ(2);
  @$pb.TagNumber(3)
  set orderId($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasOrderId() => $_has(2);
  @$pb.TagNumber(3)
  void clearOrderId() => $_clearField(3);

  @$pb.TagNumber(4)
  $core.String get customerId => $_getSZ(3);
  @$pb.TagNumber(4)
  set customerId($core.String value) => $_setString(3, value);
  @$pb.TagNumber(4)
  $core.bool hasCustomerId() => $_has(3);
  @$pb.TagNumber(4)
  void clearCustomerId() => $_clearField(4);

  @$pb.TagNumber(5)
  $fixnum.Int64 get amount => $_getI64(4);
  @$pb.TagNumber(5)
  set amount($fixnum.Int64 value) => $_setInt64(4, value);
  @$pb.TagNumber(5)
  $core.bool hasAmount() => $_has(4);
  @$pb.TagNumber(5)
  void clearAmount() => $_clearField(5);

  @$pb.TagNumber(6)
  $core.String get currency => $_getSZ(5);
  @$pb.TagNumber(6)
  set currency($core.String value) => $_setString(5, value);
  @$pb.TagNumber(6)
  $core.bool hasCurrency() => $_has(5);
  @$pb.TagNumber(6)
  void clearCurrency() => $_clearField(6);

  @$pb.TagNumber(7)
  $core.String get paymentMethod => $_getSZ(6);
  @$pb.TagNumber(7)
  set paymentMethod($core.String value) => $_setString(6, value);
  @$pb.TagNumber(7)
  $core.bool hasPaymentMethod() => $_has(6);
  @$pb.TagNumber(7)
  void clearPaymentMethod() => $_clearField(7);

  @$pb.TagNumber(8)
  $1.Timestamp get initiatedAt => $_getN(7);
  @$pb.TagNumber(8)
  set initiatedAt($1.Timestamp value) => $_setField(8, value);
  @$pb.TagNumber(8)
  $core.bool hasInitiatedAt() => $_has(7);
  @$pb.TagNumber(8)
  void clearInitiatedAt() => $_clearField(8);
  @$pb.TagNumber(8)
  $1.Timestamp ensureInitiatedAt() => $_ensure(7);
}

class PaymentCompletedEvent extends $pb.GeneratedMessage {
  factory PaymentCompletedEvent({
    $0.EventMetadata? metadata,
    $core.String? paymentId,
    $core.String? orderId,
    $fixnum.Int64? amount,
    $core.String? currency,
    $core.String? transactionId,
    $1.Timestamp? completedAt,
  }) {
    final result = create();
    if (metadata != null) result.metadata = metadata;
    if (paymentId != null) result.paymentId = paymentId;
    if (orderId != null) result.orderId = orderId;
    if (amount != null) result.amount = amount;
    if (currency != null) result.currency = currency;
    if (transactionId != null) result.transactionId = transactionId;
    if (completedAt != null) result.completedAt = completedAt;
    return result;
  }

  PaymentCompletedEvent._();

  factory PaymentCompletedEvent.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory PaymentCompletedEvent.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'PaymentCompletedEvent',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.event.service.payment.v1'),
      createEmptyInstance: create)
    ..aOM<$0.EventMetadata>(1, _omitFieldNames ? '' : 'metadata',
        subBuilder: $0.EventMetadata.create)
    ..aOS(2, _omitFieldNames ? '' : 'paymentId')
    ..aOS(3, _omitFieldNames ? '' : 'orderId')
    ..aInt64(4, _omitFieldNames ? '' : 'amount')
    ..aOS(5, _omitFieldNames ? '' : 'currency')
    ..aOS(6, _omitFieldNames ? '' : 'transactionId')
    ..aOM<$1.Timestamp>(7, _omitFieldNames ? '' : 'completedAt',
        subBuilder: $1.Timestamp.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  PaymentCompletedEvent clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  PaymentCompletedEvent copyWith(
          void Function(PaymentCompletedEvent) updates) =>
      super.copyWith((message) => updates(message as PaymentCompletedEvent))
          as PaymentCompletedEvent;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static PaymentCompletedEvent create() => PaymentCompletedEvent._();
  @$core.override
  PaymentCompletedEvent createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static PaymentCompletedEvent getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<PaymentCompletedEvent>(create);
  static PaymentCompletedEvent? _defaultInstance;

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
  $core.String get paymentId => $_getSZ(1);
  @$pb.TagNumber(2)
  set paymentId($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasPaymentId() => $_has(1);
  @$pb.TagNumber(2)
  void clearPaymentId() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.String get orderId => $_getSZ(2);
  @$pb.TagNumber(3)
  set orderId($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasOrderId() => $_has(2);
  @$pb.TagNumber(3)
  void clearOrderId() => $_clearField(3);

  @$pb.TagNumber(4)
  $fixnum.Int64 get amount => $_getI64(3);
  @$pb.TagNumber(4)
  set amount($fixnum.Int64 value) => $_setInt64(3, value);
  @$pb.TagNumber(4)
  $core.bool hasAmount() => $_has(3);
  @$pb.TagNumber(4)
  void clearAmount() => $_clearField(4);

  @$pb.TagNumber(5)
  $core.String get currency => $_getSZ(4);
  @$pb.TagNumber(5)
  set currency($core.String value) => $_setString(4, value);
  @$pb.TagNumber(5)
  $core.bool hasCurrency() => $_has(4);
  @$pb.TagNumber(5)
  void clearCurrency() => $_clearField(5);

  @$pb.TagNumber(6)
  $core.String get transactionId => $_getSZ(5);
  @$pb.TagNumber(6)
  set transactionId($core.String value) => $_setString(5, value);
  @$pb.TagNumber(6)
  $core.bool hasTransactionId() => $_has(5);
  @$pb.TagNumber(6)
  void clearTransactionId() => $_clearField(6);

  @$pb.TagNumber(7)
  $1.Timestamp get completedAt => $_getN(6);
  @$pb.TagNumber(7)
  set completedAt($1.Timestamp value) => $_setField(7, value);
  @$pb.TagNumber(7)
  $core.bool hasCompletedAt() => $_has(6);
  @$pb.TagNumber(7)
  void clearCompletedAt() => $_clearField(7);
  @$pb.TagNumber(7)
  $1.Timestamp ensureCompletedAt() => $_ensure(6);
}

class PaymentFailedEvent extends $pb.GeneratedMessage {
  factory PaymentFailedEvent({
    $0.EventMetadata? metadata,
    $core.String? paymentId,
    $core.String? orderId,
    $core.String? reason,
    $core.String? errorCode,
    $1.Timestamp? failedAt,
  }) {
    final result = create();
    if (metadata != null) result.metadata = metadata;
    if (paymentId != null) result.paymentId = paymentId;
    if (orderId != null) result.orderId = orderId;
    if (reason != null) result.reason = reason;
    if (errorCode != null) result.errorCode = errorCode;
    if (failedAt != null) result.failedAt = failedAt;
    return result;
  }

  PaymentFailedEvent._();

  factory PaymentFailedEvent.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory PaymentFailedEvent.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'PaymentFailedEvent',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.event.service.payment.v1'),
      createEmptyInstance: create)
    ..aOM<$0.EventMetadata>(1, _omitFieldNames ? '' : 'metadata',
        subBuilder: $0.EventMetadata.create)
    ..aOS(2, _omitFieldNames ? '' : 'paymentId')
    ..aOS(3, _omitFieldNames ? '' : 'orderId')
    ..aOS(4, _omitFieldNames ? '' : 'reason')
    ..aOS(5, _omitFieldNames ? '' : 'errorCode')
    ..aOM<$1.Timestamp>(6, _omitFieldNames ? '' : 'failedAt',
        subBuilder: $1.Timestamp.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  PaymentFailedEvent clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  PaymentFailedEvent copyWith(void Function(PaymentFailedEvent) updates) =>
      super.copyWith((message) => updates(message as PaymentFailedEvent))
          as PaymentFailedEvent;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static PaymentFailedEvent create() => PaymentFailedEvent._();
  @$core.override
  PaymentFailedEvent createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static PaymentFailedEvent getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<PaymentFailedEvent>(create);
  static PaymentFailedEvent? _defaultInstance;

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
  $core.String get paymentId => $_getSZ(1);
  @$pb.TagNumber(2)
  set paymentId($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasPaymentId() => $_has(1);
  @$pb.TagNumber(2)
  void clearPaymentId() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.String get orderId => $_getSZ(2);
  @$pb.TagNumber(3)
  set orderId($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasOrderId() => $_has(2);
  @$pb.TagNumber(3)
  void clearOrderId() => $_clearField(3);

  @$pb.TagNumber(4)
  $core.String get reason => $_getSZ(3);
  @$pb.TagNumber(4)
  set reason($core.String value) => $_setString(3, value);
  @$pb.TagNumber(4)
  $core.bool hasReason() => $_has(3);
  @$pb.TagNumber(4)
  void clearReason() => $_clearField(4);

  @$pb.TagNumber(5)
  $core.String get errorCode => $_getSZ(4);
  @$pb.TagNumber(5)
  set errorCode($core.String value) => $_setString(4, value);
  @$pb.TagNumber(5)
  $core.bool hasErrorCode() => $_has(4);
  @$pb.TagNumber(5)
  void clearErrorCode() => $_clearField(5);

  @$pb.TagNumber(6)
  $1.Timestamp get failedAt => $_getN(5);
  @$pb.TagNumber(6)
  set failedAt($1.Timestamp value) => $_setField(6, value);
  @$pb.TagNumber(6)
  $core.bool hasFailedAt() => $_has(5);
  @$pb.TagNumber(6)
  void clearFailedAt() => $_clearField(6);
  @$pb.TagNumber(6)
  $1.Timestamp ensureFailedAt() => $_ensure(5);
}

class PaymentRefundedEvent extends $pb.GeneratedMessage {
  factory PaymentRefundedEvent({
    $0.EventMetadata? metadata,
    $core.String? paymentId,
    $core.String? orderId,
    $fixnum.Int64? refundAmount,
    $core.String? currency,
    $core.String? reason,
    $1.Timestamp? refundedAt,
  }) {
    final result = create();
    if (metadata != null) result.metadata = metadata;
    if (paymentId != null) result.paymentId = paymentId;
    if (orderId != null) result.orderId = orderId;
    if (refundAmount != null) result.refundAmount = refundAmount;
    if (currency != null) result.currency = currency;
    if (reason != null) result.reason = reason;
    if (refundedAt != null) result.refundedAt = refundedAt;
    return result;
  }

  PaymentRefundedEvent._();

  factory PaymentRefundedEvent.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory PaymentRefundedEvent.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'PaymentRefundedEvent',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.event.service.payment.v1'),
      createEmptyInstance: create)
    ..aOM<$0.EventMetadata>(1, _omitFieldNames ? '' : 'metadata',
        subBuilder: $0.EventMetadata.create)
    ..aOS(2, _omitFieldNames ? '' : 'paymentId')
    ..aOS(3, _omitFieldNames ? '' : 'orderId')
    ..aInt64(4, _omitFieldNames ? '' : 'refundAmount')
    ..aOS(5, _omitFieldNames ? '' : 'currency')
    ..aOS(6, _omitFieldNames ? '' : 'reason')
    ..aOM<$1.Timestamp>(7, _omitFieldNames ? '' : 'refundedAt',
        subBuilder: $1.Timestamp.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  PaymentRefundedEvent clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  PaymentRefundedEvent copyWith(void Function(PaymentRefundedEvent) updates) =>
      super.copyWith((message) => updates(message as PaymentRefundedEvent))
          as PaymentRefundedEvent;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static PaymentRefundedEvent create() => PaymentRefundedEvent._();
  @$core.override
  PaymentRefundedEvent createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static PaymentRefundedEvent getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<PaymentRefundedEvent>(create);
  static PaymentRefundedEvent? _defaultInstance;

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
  $core.String get paymentId => $_getSZ(1);
  @$pb.TagNumber(2)
  set paymentId($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasPaymentId() => $_has(1);
  @$pb.TagNumber(2)
  void clearPaymentId() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.String get orderId => $_getSZ(2);
  @$pb.TagNumber(3)
  set orderId($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasOrderId() => $_has(2);
  @$pb.TagNumber(3)
  void clearOrderId() => $_clearField(3);

  @$pb.TagNumber(4)
  $fixnum.Int64 get refundAmount => $_getI64(3);
  @$pb.TagNumber(4)
  set refundAmount($fixnum.Int64 value) => $_setInt64(3, value);
  @$pb.TagNumber(4)
  $core.bool hasRefundAmount() => $_has(3);
  @$pb.TagNumber(4)
  void clearRefundAmount() => $_clearField(4);

  @$pb.TagNumber(5)
  $core.String get currency => $_getSZ(4);
  @$pb.TagNumber(5)
  set currency($core.String value) => $_setString(4, value);
  @$pb.TagNumber(5)
  $core.bool hasCurrency() => $_has(4);
  @$pb.TagNumber(5)
  void clearCurrency() => $_clearField(5);

  @$pb.TagNumber(6)
  $core.String get reason => $_getSZ(5);
  @$pb.TagNumber(6)
  set reason($core.String value) => $_setString(5, value);
  @$pb.TagNumber(6)
  $core.bool hasReason() => $_has(5);
  @$pb.TagNumber(6)
  void clearReason() => $_clearField(6);

  @$pb.TagNumber(7)
  $1.Timestamp get refundedAt => $_getN(6);
  @$pb.TagNumber(7)
  set refundedAt($1.Timestamp value) => $_setField(7, value);
  @$pb.TagNumber(7)
  $core.bool hasRefundedAt() => $_has(6);
  @$pb.TagNumber(7)
  void clearRefundedAt() => $_clearField(7);
  @$pb.TagNumber(7)
  $1.Timestamp ensureRefundedAt() => $_ensure(6);
}

const $core.bool _omitFieldNames =
    $core.bool.fromEnvironment('protobuf.omit_field_names');
const $core.bool _omitMessageNames =
    $core.bool.fromEnvironment('protobuf.omit_message_names');
