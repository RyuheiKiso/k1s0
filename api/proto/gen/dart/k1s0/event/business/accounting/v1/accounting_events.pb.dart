// This is a generated file - do not edit.
//
// Generated from k1s0/event/business/accounting/v1/accounting_events.proto.

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
import 'accounting_events.pbenum.dart';

export 'package:protobuf/protobuf.dart' show GeneratedMessageGenericExtensions;

export 'accounting_events.pbenum.dart';

class EntryCreatedEvent extends $pb.GeneratedMessage {
  factory EntryCreatedEvent({
    $0.EventMetadata? metadata,
    $core.String? entryId,
    $core.String? accountId,
    $fixnum.Int64? amount,
    $core.String? currency,
    EntryType? entryType,
    $1.Timestamp? createdAt,
  }) {
    final result = create();
    if (metadata != null) result.metadata = metadata;
    if (entryId != null) result.entryId = entryId;
    if (accountId != null) result.accountId = accountId;
    if (amount != null) result.amount = amount;
    if (currency != null) result.currency = currency;
    if (entryType != null) result.entryType = entryType;
    if (createdAt != null) result.createdAt = createdAt;
    return result;
  }

  EntryCreatedEvent._();

  factory EntryCreatedEvent.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory EntryCreatedEvent.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'EntryCreatedEvent',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.event.business.accounting.v1'),
      createEmptyInstance: create)
    ..aOM<$0.EventMetadata>(1, _omitFieldNames ? '' : 'metadata',
        subBuilder: $0.EventMetadata.create)
    ..aOS(2, _omitFieldNames ? '' : 'entryId')
    ..aOS(3, _omitFieldNames ? '' : 'accountId')
    ..aInt64(4, _omitFieldNames ? '' : 'amount')
    ..aOS(5, _omitFieldNames ? '' : 'currency')
    ..aE<EntryType>(6, _omitFieldNames ? '' : 'entryType',
        enumValues: EntryType.values)
    ..aOM<$1.Timestamp>(7, _omitFieldNames ? '' : 'createdAt',
        subBuilder: $1.Timestamp.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  EntryCreatedEvent clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  EntryCreatedEvent copyWith(void Function(EntryCreatedEvent) updates) =>
      super.copyWith((message) => updates(message as EntryCreatedEvent))
          as EntryCreatedEvent;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static EntryCreatedEvent create() => EntryCreatedEvent._();
  @$core.override
  EntryCreatedEvent createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static EntryCreatedEvent getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<EntryCreatedEvent>(create);
  static EntryCreatedEvent? _defaultInstance;

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
  $core.String get entryId => $_getSZ(1);
  @$pb.TagNumber(2)
  set entryId($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasEntryId() => $_has(1);
  @$pb.TagNumber(2)
  void clearEntryId() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.String get accountId => $_getSZ(2);
  @$pb.TagNumber(3)
  set accountId($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasAccountId() => $_has(2);
  @$pb.TagNumber(3)
  void clearAccountId() => $_clearField(3);

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
  EntryType get entryType => $_getN(5);
  @$pb.TagNumber(6)
  set entryType(EntryType value) => $_setField(6, value);
  @$pb.TagNumber(6)
  $core.bool hasEntryType() => $_has(5);
  @$pb.TagNumber(6)
  void clearEntryType() => $_clearField(6);

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
}

class EntryApprovedEvent extends $pb.GeneratedMessage {
  factory EntryApprovedEvent({
    $0.EventMetadata? metadata,
    $core.String? entryId,
    $core.String? approvedBy,
    $1.Timestamp? approvedAt,
  }) {
    final result = create();
    if (metadata != null) result.metadata = metadata;
    if (entryId != null) result.entryId = entryId;
    if (approvedBy != null) result.approvedBy = approvedBy;
    if (approvedAt != null) result.approvedAt = approvedAt;
    return result;
  }

  EntryApprovedEvent._();

  factory EntryApprovedEvent.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory EntryApprovedEvent.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'EntryApprovedEvent',
      package: const $pb.PackageName(
          _omitMessageNames ? '' : 'k1s0.event.business.accounting.v1'),
      createEmptyInstance: create)
    ..aOM<$0.EventMetadata>(1, _omitFieldNames ? '' : 'metadata',
        subBuilder: $0.EventMetadata.create)
    ..aOS(2, _omitFieldNames ? '' : 'entryId')
    ..aOS(3, _omitFieldNames ? '' : 'approvedBy')
    ..aOM<$1.Timestamp>(4, _omitFieldNames ? '' : 'approvedAt',
        subBuilder: $1.Timestamp.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  EntryApprovedEvent clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  EntryApprovedEvent copyWith(void Function(EntryApprovedEvent) updates) =>
      super.copyWith((message) => updates(message as EntryApprovedEvent))
          as EntryApprovedEvent;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static EntryApprovedEvent create() => EntryApprovedEvent._();
  @$core.override
  EntryApprovedEvent createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static EntryApprovedEvent getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<EntryApprovedEvent>(create);
  static EntryApprovedEvent? _defaultInstance;

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
  $core.String get entryId => $_getSZ(1);
  @$pb.TagNumber(2)
  set entryId($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasEntryId() => $_has(1);
  @$pb.TagNumber(2)
  void clearEntryId() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.String get approvedBy => $_getSZ(2);
  @$pb.TagNumber(3)
  set approvedBy($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasApprovedBy() => $_has(2);
  @$pb.TagNumber(3)
  void clearApprovedBy() => $_clearField(3);

  @$pb.TagNumber(4)
  $1.Timestamp get approvedAt => $_getN(3);
  @$pb.TagNumber(4)
  set approvedAt($1.Timestamp value) => $_setField(4, value);
  @$pb.TagNumber(4)
  $core.bool hasApprovedAt() => $_has(3);
  @$pb.TagNumber(4)
  void clearApprovedAt() => $_clearField(4);
  @$pb.TagNumber(4)
  $1.Timestamp ensureApprovedAt() => $_ensure(3);
}

const $core.bool _omitFieldNames =
    $core.bool.fromEnvironment('protobuf.omit_field_names');
const $core.bool _omitMessageNames =
    $core.bool.fromEnvironment('protobuf.omit_message_names');
