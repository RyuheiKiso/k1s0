// This is a generated file - do not edit.
//
// Generated from k1s0/event/service/order/v1/order_events.proto.

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

@$core.Deprecated('Use orderCreatedEventDescriptor instead')
const OrderCreatedEvent$json = {
  '1': 'OrderCreatedEvent',
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
    {'1': 'customer_id', '3': 3, '4': 1, '5': 9, '10': 'customerId'},
    {
      '1': 'items',
      '3': 4,
      '4': 3,
      '5': 11,
      '6': '.k1s0.event.service.order.v1.OrderItem',
      '10': 'items'
    },
    {'1': 'total_amount', '3': 5, '4': 1, '5': 3, '10': 'totalAmount'},
    {'1': 'currency', '3': 6, '4': 1, '5': 9, '10': 'currency'},
  ],
};

/// Descriptor for `OrderCreatedEvent`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List orderCreatedEventDescriptor = $convert.base64Decode(
    'ChFPcmRlckNyZWF0ZWRFdmVudBJACghtZXRhZGF0YRgBIAEoCzIkLmsxczAuc3lzdGVtLmNvbW'
    '1vbi52MS5FdmVudE1ldGFkYXRhUghtZXRhZGF0YRIZCghvcmRlcl9pZBgCIAEoCVIHb3JkZXJJ'
    'ZBIfCgtjdXN0b21lcl9pZBgDIAEoCVIKY3VzdG9tZXJJZBI8CgVpdGVtcxgEIAMoCzImLmsxcz'
    'AuZXZlbnQuc2VydmljZS5vcmRlci52MS5PcmRlckl0ZW1SBWl0ZW1zEiEKDHRvdGFsX2Ftb3Vu'
    'dBgFIAEoA1ILdG90YWxBbW91bnQSGgoIY3VycmVuY3kYBiABKAlSCGN1cnJlbmN5');

@$core.Deprecated('Use orderItemDescriptor instead')
const OrderItem$json = {
  '1': 'OrderItem',
  '2': [
    {'1': 'product_id', '3': 1, '4': 1, '5': 9, '10': 'productId'},
    {'1': 'quantity', '3': 2, '4': 1, '5': 5, '10': 'quantity'},
    {'1': 'unit_price', '3': 3, '4': 1, '5': 3, '10': 'unitPrice'},
  ],
};

/// Descriptor for `OrderItem`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List orderItemDescriptor = $convert.base64Decode(
    'CglPcmRlckl0ZW0SHQoKcHJvZHVjdF9pZBgBIAEoCVIJcHJvZHVjdElkEhoKCHF1YW50aXR5GA'
    'IgASgFUghxdWFudGl0eRIdCgp1bml0X3ByaWNlGAMgASgDUgl1bml0UHJpY2U=');

@$core.Deprecated('Use orderUpdatedEventDescriptor instead')
const OrderUpdatedEvent$json = {
  '1': 'OrderUpdatedEvent',
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
    {'1': 'user_id', '3': 3, '4': 1, '5': 9, '10': 'userId'},
    {
      '1': 'items',
      '3': 4,
      '4': 3,
      '5': 11,
      '6': '.k1s0.event.service.order.v1.OrderItem',
      '10': 'items'
    },
    {'1': 'total_amount', '3': 5, '4': 1, '5': 3, '10': 'totalAmount'},
    {'1': 'status', '3': 6, '4': 1, '5': 9, '10': 'status'},
  ],
};

/// Descriptor for `OrderUpdatedEvent`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List orderUpdatedEventDescriptor = $convert.base64Decode(
    'ChFPcmRlclVwZGF0ZWRFdmVudBJACghtZXRhZGF0YRgBIAEoCzIkLmsxczAuc3lzdGVtLmNvbW'
    '1vbi52MS5FdmVudE1ldGFkYXRhUghtZXRhZGF0YRIZCghvcmRlcl9pZBgCIAEoCVIHb3JkZXJJ'
    'ZBIXCgd1c2VyX2lkGAMgASgJUgZ1c2VySWQSPAoFaXRlbXMYBCADKAsyJi5rMXMwLmV2ZW50Ln'
    'NlcnZpY2Uub3JkZXIudjEuT3JkZXJJdGVtUgVpdGVtcxIhCgx0b3RhbF9hbW91bnQYBSABKANS'
    'C3RvdGFsQW1vdW50EhYKBnN0YXR1cxgGIAEoCVIGc3RhdHVz');

@$core.Deprecated('Use orderCancelledEventDescriptor instead')
const OrderCancelledEvent$json = {
  '1': 'OrderCancelledEvent',
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
    {'1': 'user_id', '3': 3, '4': 1, '5': 9, '10': 'userId'},
    {'1': 'reason', '3': 4, '4': 1, '5': 9, '10': 'reason'},
  ],
};

/// Descriptor for `OrderCancelledEvent`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List orderCancelledEventDescriptor = $convert.base64Decode(
    'ChNPcmRlckNhbmNlbGxlZEV2ZW50EkAKCG1ldGFkYXRhGAEgASgLMiQuazFzMC5zeXN0ZW0uY2'
    '9tbW9uLnYxLkV2ZW50TWV0YWRhdGFSCG1ldGFkYXRhEhkKCG9yZGVyX2lkGAIgASgJUgdvcmRl'
    'cklkEhcKB3VzZXJfaWQYAyABKAlSBnVzZXJJZBIWCgZyZWFzb24YBCABKAlSBnJlYXNvbg==');
