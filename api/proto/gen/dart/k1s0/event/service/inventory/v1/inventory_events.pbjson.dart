// This is a generated file - do not edit.
//
// Generated from k1s0/event/service/inventory/v1/inventory_events.proto.

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

@$core.Deprecated('Use inventoryReservedEventDescriptor instead')
const InventoryReservedEvent$json = {
  '1': 'InventoryReservedEvent',
  '2': [
    {
      '1': 'metadata',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.common.v1.EventMetadata',
      '10': 'metadata'
    },
    {'1': 'order_id', '3': 2, '4': 1, '5': 9, '10': 'orderId'},
    {'1': 'product_id', '3': 3, '4': 1, '5': 9, '10': 'productId'},
    {'1': 'quantity', '3': 4, '4': 1, '5': 5, '10': 'quantity'},
    {'1': 'warehouse_id', '3': 5, '4': 1, '5': 9, '10': 'warehouseId'},
    {
      '1': 'reserved_at',
      '3': 6,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.common.v1.Timestamp',
      '10': 'reservedAt'
    },
  ],
};

/// Descriptor for `InventoryReservedEvent`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List inventoryReservedEventDescriptor = $convert.base64Decode(
    'ChZJbnZlbnRvcnlSZXNlcnZlZEV2ZW50EkAKCG1ldGFkYXRhGAEgASgLMiQuazFzMC5zeXN0ZW'
    '0uY29tbW9uLnYxLkV2ZW50TWV0YWRhdGFSCG1ldGFkYXRhEhkKCG9yZGVyX2lkGAIgASgJUgdv'
    'cmRlcklkEh0KCnByb2R1Y3RfaWQYAyABKAlSCXByb2R1Y3RJZBIaCghxdWFudGl0eRgEIAEoBV'
    'IIcXVhbnRpdHkSIQoMd2FyZWhvdXNlX2lkGAUgASgJUgt3YXJlaG91c2VJZBJBCgtyZXNlcnZl'
    'ZF9hdBgGIAEoCzIgLmsxczAuc3lzdGVtLmNvbW1vbi52MS5UaW1lc3RhbXBSCnJlc2VydmVkQX'
    'Q=');

@$core.Deprecated('Use inventoryReleasedEventDescriptor instead')
const InventoryReleasedEvent$json = {
  '1': 'InventoryReleasedEvent',
  '2': [
    {
      '1': 'metadata',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.common.v1.EventMetadata',
      '10': 'metadata'
    },
    {'1': 'order_id', '3': 2, '4': 1, '5': 9, '10': 'orderId'},
    {'1': 'product_id', '3': 3, '4': 1, '5': 9, '10': 'productId'},
    {'1': 'quantity', '3': 4, '4': 1, '5': 5, '10': 'quantity'},
    {'1': 'warehouse_id', '3': 5, '4': 1, '5': 9, '10': 'warehouseId'},
    {'1': 'reason', '3': 6, '4': 1, '5': 9, '10': 'reason'},
    {
      '1': 'released_at',
      '3': 7,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.common.v1.Timestamp',
      '10': 'releasedAt'
    },
  ],
};

/// Descriptor for `InventoryReleasedEvent`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List inventoryReleasedEventDescriptor = $convert.base64Decode(
    'ChZJbnZlbnRvcnlSZWxlYXNlZEV2ZW50EkAKCG1ldGFkYXRhGAEgASgLMiQuazFzMC5zeXN0ZW'
    '0uY29tbW9uLnYxLkV2ZW50TWV0YWRhdGFSCG1ldGFkYXRhEhkKCG9yZGVyX2lkGAIgASgJUgdv'
    'cmRlcklkEh0KCnByb2R1Y3RfaWQYAyABKAlSCXByb2R1Y3RJZBIaCghxdWFudGl0eRgEIAEoBV'
    'IIcXVhbnRpdHkSIQoMd2FyZWhvdXNlX2lkGAUgASgJUgt3YXJlaG91c2VJZBIWCgZyZWFzb24Y'
    'BiABKAlSBnJlYXNvbhJBCgtyZWxlYXNlZF9hdBgHIAEoCzIgLmsxczAuc3lzdGVtLmNvbW1vbi'
    '52MS5UaW1lc3RhbXBSCnJlbGVhc2VkQXQ=');
