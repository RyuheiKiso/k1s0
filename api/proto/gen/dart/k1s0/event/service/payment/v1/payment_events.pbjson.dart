// This is a generated file - do not edit.
//
// Generated from k1s0/event/service/payment/v1/payment_events.proto.

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

@$core.Deprecated('Use paymentInitiatedEventDescriptor instead')
const PaymentInitiatedEvent$json = {
  '1': 'PaymentInitiatedEvent',
  '2': [
    {
      '1': 'metadata',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.common.v1.EventMetadata',
      '10': 'metadata'
    },
    {'1': 'payment_id', '3': 2, '4': 1, '5': 9, '10': 'paymentId'},
    {'1': 'order_id', '3': 3, '4': 1, '5': 9, '10': 'orderId'},
    {'1': 'customer_id', '3': 4, '4': 1, '5': 9, '10': 'customerId'},
    {'1': 'amount', '3': 5, '4': 1, '5': 3, '10': 'amount'},
    {'1': 'currency', '3': 6, '4': 1, '5': 9, '10': 'currency'},
    {'1': 'payment_method', '3': 7, '4': 1, '5': 9, '10': 'paymentMethod'},
    {
      '1': 'initiated_at',
      '3': 8,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.common.v1.Timestamp',
      '10': 'initiatedAt'
    },
  ],
};

/// Descriptor for `PaymentInitiatedEvent`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List paymentInitiatedEventDescriptor = $convert.base64Decode(
    'ChVQYXltZW50SW5pdGlhdGVkRXZlbnQSQAoIbWV0YWRhdGEYASABKAsyJC5rMXMwLnN5c3RlbS'
    '5jb21tb24udjEuRXZlbnRNZXRhZGF0YVIIbWV0YWRhdGESHQoKcGF5bWVudF9pZBgCIAEoCVIJ'
    'cGF5bWVudElkEhkKCG9yZGVyX2lkGAMgASgJUgdvcmRlcklkEh8KC2N1c3RvbWVyX2lkGAQgAS'
    'gJUgpjdXN0b21lcklkEhYKBmFtb3VudBgFIAEoA1IGYW1vdW50EhoKCGN1cnJlbmN5GAYgASgJ'
    'UghjdXJyZW5jeRIlCg5wYXltZW50X21ldGhvZBgHIAEoCVINcGF5bWVudE1ldGhvZBJDCgxpbm'
    'l0aWF0ZWRfYXQYCCABKAsyIC5rMXMwLnN5c3RlbS5jb21tb24udjEuVGltZXN0YW1wUgtpbml0'
    'aWF0ZWRBdA==');

@$core.Deprecated('Use paymentCompletedEventDescriptor instead')
const PaymentCompletedEvent$json = {
  '1': 'PaymentCompletedEvent',
  '2': [
    {
      '1': 'metadata',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.common.v1.EventMetadata',
      '10': 'metadata'
    },
    {'1': 'payment_id', '3': 2, '4': 1, '5': 9, '10': 'paymentId'},
    {'1': 'order_id', '3': 3, '4': 1, '5': 9, '10': 'orderId'},
    {'1': 'amount', '3': 4, '4': 1, '5': 3, '10': 'amount'},
    {'1': 'currency', '3': 5, '4': 1, '5': 9, '10': 'currency'},
    {'1': 'transaction_id', '3': 6, '4': 1, '5': 9, '10': 'transactionId'},
    {
      '1': 'completed_at',
      '3': 7,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.common.v1.Timestamp',
      '10': 'completedAt'
    },
  ],
};

/// Descriptor for `PaymentCompletedEvent`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List paymentCompletedEventDescriptor = $convert.base64Decode(
    'ChVQYXltZW50Q29tcGxldGVkRXZlbnQSQAoIbWV0YWRhdGEYASABKAsyJC5rMXMwLnN5c3RlbS'
    '5jb21tb24udjEuRXZlbnRNZXRhZGF0YVIIbWV0YWRhdGESHQoKcGF5bWVudF9pZBgCIAEoCVIJ'
    'cGF5bWVudElkEhkKCG9yZGVyX2lkGAMgASgJUgdvcmRlcklkEhYKBmFtb3VudBgEIAEoA1IGYW'
    '1vdW50EhoKCGN1cnJlbmN5GAUgASgJUghjdXJyZW5jeRIlCg50cmFuc2FjdGlvbl9pZBgGIAEo'
    'CVINdHJhbnNhY3Rpb25JZBJDCgxjb21wbGV0ZWRfYXQYByABKAsyIC5rMXMwLnN5c3RlbS5jb2'
    '1tb24udjEuVGltZXN0YW1wUgtjb21wbGV0ZWRBdA==');

@$core.Deprecated('Use paymentFailedEventDescriptor instead')
const PaymentFailedEvent$json = {
  '1': 'PaymentFailedEvent',
  '2': [
    {
      '1': 'metadata',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.common.v1.EventMetadata',
      '10': 'metadata'
    },
    {'1': 'payment_id', '3': 2, '4': 1, '5': 9, '10': 'paymentId'},
    {'1': 'order_id', '3': 3, '4': 1, '5': 9, '10': 'orderId'},
    {'1': 'reason', '3': 4, '4': 1, '5': 9, '10': 'reason'},
    {'1': 'error_code', '3': 5, '4': 1, '5': 9, '10': 'errorCode'},
    {
      '1': 'failed_at',
      '3': 6,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.common.v1.Timestamp',
      '10': 'failedAt'
    },
  ],
};

/// Descriptor for `PaymentFailedEvent`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List paymentFailedEventDescriptor = $convert.base64Decode(
    'ChJQYXltZW50RmFpbGVkRXZlbnQSQAoIbWV0YWRhdGEYASABKAsyJC5rMXMwLnN5c3RlbS5jb2'
    '1tb24udjEuRXZlbnRNZXRhZGF0YVIIbWV0YWRhdGESHQoKcGF5bWVudF9pZBgCIAEoCVIJcGF5'
    'bWVudElkEhkKCG9yZGVyX2lkGAMgASgJUgdvcmRlcklkEhYKBnJlYXNvbhgEIAEoCVIGcmVhc2'
    '9uEh0KCmVycm9yX2NvZGUYBSABKAlSCWVycm9yQ29kZRI9CglmYWlsZWRfYXQYBiABKAsyIC5r'
    'MXMwLnN5c3RlbS5jb21tb24udjEuVGltZXN0YW1wUghmYWlsZWRBdA==');

@$core.Deprecated('Use paymentRefundedEventDescriptor instead')
const PaymentRefundedEvent$json = {
  '1': 'PaymentRefundedEvent',
  '2': [
    {
      '1': 'metadata',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.common.v1.EventMetadata',
      '10': 'metadata'
    },
    {'1': 'payment_id', '3': 2, '4': 1, '5': 9, '10': 'paymentId'},
    {'1': 'order_id', '3': 3, '4': 1, '5': 9, '10': 'orderId'},
    {'1': 'refund_amount', '3': 4, '4': 1, '5': 3, '10': 'refundAmount'},
    {'1': 'currency', '3': 5, '4': 1, '5': 9, '10': 'currency'},
    {'1': 'reason', '3': 6, '4': 1, '5': 9, '10': 'reason'},
    {
      '1': 'refunded_at',
      '3': 7,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.common.v1.Timestamp',
      '10': 'refundedAt'
    },
  ],
};

/// Descriptor for `PaymentRefundedEvent`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List paymentRefundedEventDescriptor = $convert.base64Decode(
    'ChRQYXltZW50UmVmdW5kZWRFdmVudBJACghtZXRhZGF0YRgBIAEoCzIkLmsxczAuc3lzdGVtLm'
    'NvbW1vbi52MS5FdmVudE1ldGFkYXRhUghtZXRhZGF0YRIdCgpwYXltZW50X2lkGAIgASgJUglw'
    'YXltZW50SWQSGQoIb3JkZXJfaWQYAyABKAlSB29yZGVySWQSIwoNcmVmdW5kX2Ftb3VudBgEIA'
    'EoA1IMcmVmdW5kQW1vdW50EhoKCGN1cnJlbmN5GAUgASgJUghjdXJyZW5jeRIWCgZyZWFzb24Y'
    'BiABKAlSBnJlYXNvbhJBCgtyZWZ1bmRlZF9hdBgHIAEoCzIgLmsxczAuc3lzdGVtLmNvbW1vbi'
    '52MS5UaW1lc3RhbXBSCnJlZnVuZGVkQXQ=');
