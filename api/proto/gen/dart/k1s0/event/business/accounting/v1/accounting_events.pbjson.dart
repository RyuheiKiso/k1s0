// This is a generated file - do not edit.
//
// Generated from k1s0/event/business/accounting/v1/accounting_events.proto.

// @dart = 3.3

// ignore_for_file: annotate_overrides, camel_case_types, comment_references
// ignore_for_file: constant_identifier_names
// ignore_for_file: curly_braces_in_flow_control_structures
// ignore_for_file: deprecated_member_use_from_same_package, library_prefixes
// ignore_for_file: non_constant_identifier_names, prefer_relative_imports
// ignore_for_file: unused_import

import 'dart:convert' as $convert;
import 'dart:core' as $core;
import 'dart:typed_data' as $typed_data;

@$core.Deprecated('Use entryTypeDescriptor instead')
const EntryType$json = {
  '1': 'EntryType',
  '2': [
    {'1': 'ENTRY_TYPE_UNSPECIFIED', '2': 0},
    {'1': 'ENTRY_TYPE_DEBIT', '2': 1},
    {'1': 'ENTRY_TYPE_CREDIT', '2': 2},
  ],
};

/// Descriptor for `EntryType`. Decode as a `google.protobuf.EnumDescriptorProto`.
final $typed_data.Uint8List entryTypeDescriptor = $convert.base64Decode(
    'CglFbnRyeVR5cGUSGgoWRU5UUllfVFlQRV9VTlNQRUNJRklFRBAAEhQKEEVOVFJZX1RZUEVfRE'
    'VCSVQQARIVChFFTlRSWV9UWVBFX0NSRURJVBAC');

@$core.Deprecated('Use entryCreatedEventDescriptor instead')
const EntryCreatedEvent$json = {
  '1': 'EntryCreatedEvent',
  '2': [
    {
      '1': 'metadata',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.common.v1.EventMetadata',
      '10': 'metadata'
    },
    {'1': 'entry_id', '3': 2, '4': 1, '5': 9, '10': 'entryId'},
    {'1': 'account_id', '3': 3, '4': 1, '5': 9, '10': 'accountId'},
    {'1': 'amount', '3': 4, '4': 1, '5': 3, '10': 'amount'},
    {'1': 'currency', '3': 5, '4': 1, '5': 9, '10': 'currency'},
    {
      '1': 'entry_type',
      '3': 6,
      '4': 1,
      '5': 14,
      '6': '.k1s0.event.business.accounting.v1.EntryType',
      '10': 'entryType'
    },
    {
      '1': 'created_at',
      '3': 7,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.common.v1.Timestamp',
      '10': 'createdAt'
    },
  ],
};

/// Descriptor for `EntryCreatedEvent`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List entryCreatedEventDescriptor = $convert.base64Decode(
    'ChFFbnRyeUNyZWF0ZWRFdmVudBJACghtZXRhZGF0YRgBIAEoCzIkLmsxczAuc3lzdGVtLmNvbW'
    '1vbi52MS5FdmVudE1ldGFkYXRhUghtZXRhZGF0YRIZCghlbnRyeV9pZBgCIAEoCVIHZW50cnlJ'
    'ZBIdCgphY2NvdW50X2lkGAMgASgJUglhY2NvdW50SWQSFgoGYW1vdW50GAQgASgDUgZhbW91bn'
    'QSGgoIY3VycmVuY3kYBSABKAlSCGN1cnJlbmN5EksKCmVudHJ5X3R5cGUYBiABKA4yLC5rMXMw'
    'LmV2ZW50LmJ1c2luZXNzLmFjY291bnRpbmcudjEuRW50cnlUeXBlUgllbnRyeVR5cGUSPwoKY3'
    'JlYXRlZF9hdBgHIAEoCzIgLmsxczAuc3lzdGVtLmNvbW1vbi52MS5UaW1lc3RhbXBSCWNyZWF0'
    'ZWRBdA==');

@$core.Deprecated('Use entryApprovedEventDescriptor instead')
const EntryApprovedEvent$json = {
  '1': 'EntryApprovedEvent',
  '2': [
    {
      '1': 'metadata',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.common.v1.EventMetadata',
      '10': 'metadata'
    },
    {'1': 'entry_id', '3': 2, '4': 1, '5': 9, '10': 'entryId'},
    {'1': 'approved_by', '3': 3, '4': 1, '5': 9, '10': 'approvedBy'},
    {
      '1': 'approved_at',
      '3': 4,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.common.v1.Timestamp',
      '10': 'approvedAt'
    },
  ],
};

/// Descriptor for `EntryApprovedEvent`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List entryApprovedEventDescriptor = $convert.base64Decode(
    'ChJFbnRyeUFwcHJvdmVkRXZlbnQSQAoIbWV0YWRhdGEYASABKAsyJC5rMXMwLnN5c3RlbS5jb2'
    '1tb24udjEuRXZlbnRNZXRhZGF0YVIIbWV0YWRhdGESGQoIZW50cnlfaWQYAiABKAlSB2VudHJ5'
    'SWQSHwoLYXBwcm92ZWRfYnkYAyABKAlSCmFwcHJvdmVkQnkSQQoLYXBwcm92ZWRfYXQYBCABKA'
    'syIC5rMXMwLnN5c3RlbS5jb21tb24udjEuVGltZXN0YW1wUgphcHByb3ZlZEF0');
