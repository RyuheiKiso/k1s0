// This is a generated file - do not edit.
//
// Generated from k1s0/service/payment/v1/payment.proto.

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

@$core.Deprecated('Use paymentStatusDescriptor instead')
const PaymentStatus$json = {
  '1': 'PaymentStatus',
  '2': [
    {'1': 'PAYMENT_STATUS_UNSPECIFIED', '2': 0},
    {'1': 'PAYMENT_STATUS_PENDING', '2': 1},
    {'1': 'PAYMENT_STATUS_PROCESSING', '2': 2},
    {'1': 'PAYMENT_STATUS_SUCCEEDED', '2': 3},
    {'1': 'PAYMENT_STATUS_FAILED', '2': 4},
    {'1': 'PAYMENT_STATUS_CANCELLED', '2': 5},
    {'1': 'PAYMENT_STATUS_REFUNDED', '2': 6},
  ],
};

/// Descriptor for `PaymentStatus`. Decode as a `google.protobuf.EnumDescriptorProto`.
final $typed_data.Uint8List paymentStatusDescriptor = $convert.base64Decode(
    'Cg1QYXltZW50U3RhdHVzEh4KGlBBWU1FTlRfU1RBVFVTX1VOU1BFQ0lGSUVEEAASGgoWUEFZTU'
    'VOVF9TVEFUVVNfUEVORElORxABEh0KGVBBWU1FTlRfU1RBVFVTX1BST0NFU1NJTkcQAhIcChhQ'
    'QVlNRU5UX1NUQVRVU19TVUNDRUVERUQQAxIZChVQQVlNRU5UX1NUQVRVU19GQUlMRUQQBBIcCh'
    'hQQVlNRU5UX1NUQVRVU19DQU5DRUxMRUQQBRIbChdQQVlNRU5UX1NUQVRVU19SRUZVTkRFRBAG');

@$core.Deprecated('Use paymentDescriptor instead')
const Payment$json = {
  '1': 'Payment',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 9, '10': 'id'},
    {'1': 'order_id', '3': 2, '4': 1, '5': 9, '10': 'orderId'},
    {'1': 'customer_id', '3': 3, '4': 1, '5': 9, '10': 'customerId'},
    {'1': 'amount', '3': 4, '4': 1, '5': 3, '10': 'amount'},
    {'1': 'currency', '3': 5, '4': 1, '5': 9, '10': 'currency'},
    {'1': 'status', '3': 6, '4': 1, '5': 9, '10': 'status'},
    {
      '1': 'payment_method',
      '3': 7,
      '4': 1,
      '5': 9,
      '9': 0,
      '10': 'paymentMethod',
      '17': true
    },
    {
      '1': 'transaction_id',
      '3': 8,
      '4': 1,
      '5': 9,
      '9': 1,
      '10': 'transactionId',
      '17': true
    },
    {
      '1': 'error_code',
      '3': 9,
      '4': 1,
      '5': 9,
      '9': 2,
      '10': 'errorCode',
      '17': true
    },
    {
      '1': 'error_message',
      '3': 10,
      '4': 1,
      '5': 9,
      '9': 3,
      '10': 'errorMessage',
      '17': true
    },
    {'1': 'version', '3': 11, '4': 1, '5': 5, '10': 'version'},
    {
      '1': 'created_at',
      '3': 12,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.common.v1.Timestamp',
      '10': 'createdAt'
    },
    {
      '1': 'updated_at',
      '3': 13,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.common.v1.Timestamp',
      '10': 'updatedAt'
    },
    {
      '1': 'status_enum',
      '3': 14,
      '4': 1,
      '5': 14,
      '6': '.k1s0.service.payment.v1.PaymentStatus',
      '10': 'statusEnum'
    },
  ],
  '8': [
    {'1': '_payment_method'},
    {'1': '_transaction_id'},
    {'1': '_error_code'},
    {'1': '_error_message'},
  ],
};

/// Descriptor for `Payment`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List paymentDescriptor = $convert.base64Decode(
    'CgdQYXltZW50Eg4KAmlkGAEgASgJUgJpZBIZCghvcmRlcl9pZBgCIAEoCVIHb3JkZXJJZBIfCg'
    'tjdXN0b21lcl9pZBgDIAEoCVIKY3VzdG9tZXJJZBIWCgZhbW91bnQYBCABKANSBmFtb3VudBIa'
    'CghjdXJyZW5jeRgFIAEoCVIIY3VycmVuY3kSFgoGc3RhdHVzGAYgASgJUgZzdGF0dXMSKgoOcG'
    'F5bWVudF9tZXRob2QYByABKAlIAFINcGF5bWVudE1ldGhvZIgBARIqCg50cmFuc2FjdGlvbl9p'
    'ZBgIIAEoCUgBUg10cmFuc2FjdGlvbklkiAEBEiIKCmVycm9yX2NvZGUYCSABKAlIAlIJZXJyb3'
    'JDb2RliAEBEigKDWVycm9yX21lc3NhZ2UYCiABKAlIA1IMZXJyb3JNZXNzYWdliAEBEhgKB3Zl'
    'cnNpb24YCyABKAVSB3ZlcnNpb24SPwoKY3JlYXRlZF9hdBgMIAEoCzIgLmsxczAuc3lzdGVtLm'
    'NvbW1vbi52MS5UaW1lc3RhbXBSCWNyZWF0ZWRBdBI/Cgp1cGRhdGVkX2F0GA0gASgLMiAuazFz'
    'MC5zeXN0ZW0uY29tbW9uLnYxLlRpbWVzdGFtcFIJdXBkYXRlZEF0EkcKC3N0YXR1c19lbnVtGA'
    '4gASgOMiYuazFzMC5zZXJ2aWNlLnBheW1lbnQudjEuUGF5bWVudFN0YXR1c1IKc3RhdHVzRW51'
    'bUIRCg9fcGF5bWVudF9tZXRob2RCEQoPX3RyYW5zYWN0aW9uX2lkQg0KC19lcnJvcl9jb2RlQh'
    'AKDl9lcnJvcl9tZXNzYWdl');

@$core.Deprecated('Use initiatePaymentRequestDescriptor instead')
const InitiatePaymentRequest$json = {
  '1': 'InitiatePaymentRequest',
  '2': [
    {'1': 'order_id', '3': 1, '4': 1, '5': 9, '10': 'orderId'},
    {'1': 'customer_id', '3': 2, '4': 1, '5': 9, '10': 'customerId'},
    {'1': 'amount', '3': 3, '4': 1, '5': 3, '10': 'amount'},
    {'1': 'currency', '3': 4, '4': 1, '5': 9, '10': 'currency'},
    {
      '1': 'payment_method',
      '3': 5,
      '4': 1,
      '5': 9,
      '9': 0,
      '10': 'paymentMethod',
      '17': true
    },
  ],
  '8': [
    {'1': '_payment_method'},
  ],
};

/// Descriptor for `InitiatePaymentRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List initiatePaymentRequestDescriptor = $convert.base64Decode(
    'ChZJbml0aWF0ZVBheW1lbnRSZXF1ZXN0EhkKCG9yZGVyX2lkGAEgASgJUgdvcmRlcklkEh8KC2'
    'N1c3RvbWVyX2lkGAIgASgJUgpjdXN0b21lcklkEhYKBmFtb3VudBgDIAEoA1IGYW1vdW50EhoK'
    'CGN1cnJlbmN5GAQgASgJUghjdXJyZW5jeRIqCg5wYXltZW50X21ldGhvZBgFIAEoCUgAUg1wYX'
    'ltZW50TWV0aG9kiAEBQhEKD19wYXltZW50X21ldGhvZA==');

@$core.Deprecated('Use initiatePaymentResponseDescriptor instead')
const InitiatePaymentResponse$json = {
  '1': 'InitiatePaymentResponse',
  '2': [
    {
      '1': 'payment',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.k1s0.service.payment.v1.Payment',
      '10': 'payment'
    },
  ],
};

/// Descriptor for `InitiatePaymentResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List initiatePaymentResponseDescriptor =
    $convert.base64Decode(
        'ChdJbml0aWF0ZVBheW1lbnRSZXNwb25zZRI6CgdwYXltZW50GAEgASgLMiAuazFzMC5zZXJ2aW'
        'NlLnBheW1lbnQudjEuUGF5bWVudFIHcGF5bWVudA==');

@$core.Deprecated('Use getPaymentRequestDescriptor instead')
const GetPaymentRequest$json = {
  '1': 'GetPaymentRequest',
  '2': [
    {'1': 'payment_id', '3': 1, '4': 1, '5': 9, '10': 'paymentId'},
  ],
};

/// Descriptor for `GetPaymentRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List getPaymentRequestDescriptor = $convert.base64Decode(
    'ChFHZXRQYXltZW50UmVxdWVzdBIdCgpwYXltZW50X2lkGAEgASgJUglwYXltZW50SWQ=');

@$core.Deprecated('Use getPaymentResponseDescriptor instead')
const GetPaymentResponse$json = {
  '1': 'GetPaymentResponse',
  '2': [
    {
      '1': 'payment',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.k1s0.service.payment.v1.Payment',
      '10': 'payment'
    },
  ],
};

/// Descriptor for `GetPaymentResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List getPaymentResponseDescriptor = $convert.base64Decode(
    'ChJHZXRQYXltZW50UmVzcG9uc2USOgoHcGF5bWVudBgBIAEoCzIgLmsxczAuc2VydmljZS5wYX'
    'ltZW50LnYxLlBheW1lbnRSB3BheW1lbnQ=');

@$core.Deprecated('Use listPaymentsRequestDescriptor instead')
const ListPaymentsRequest$json = {
  '1': 'ListPaymentsRequest',
  '2': [
    {
      '1': 'order_id',
      '3': 1,
      '4': 1,
      '5': 9,
      '9': 0,
      '10': 'orderId',
      '17': true
    },
    {
      '1': 'customer_id',
      '3': 2,
      '4': 1,
      '5': 9,
      '9': 1,
      '10': 'customerId',
      '17': true
    },
    {'1': 'status', '3': 3, '4': 1, '5': 9, '9': 2, '10': 'status', '17': true},
    {
      '1': 'pagination',
      '3': 4,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.common.v1.Pagination',
      '10': 'pagination'
    },
    {
      '1': 'status_enum',
      '3': 5,
      '4': 1,
      '5': 14,
      '6': '.k1s0.service.payment.v1.PaymentStatus',
      '9': 3,
      '10': 'statusEnum',
      '17': true
    },
  ],
  '8': [
    {'1': '_order_id'},
    {'1': '_customer_id'},
    {'1': '_status'},
    {'1': '_status_enum'},
  ],
};

/// Descriptor for `ListPaymentsRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List listPaymentsRequestDescriptor = $convert.base64Decode(
    'ChNMaXN0UGF5bWVudHNSZXF1ZXN0Eh4KCG9yZGVyX2lkGAEgASgJSABSB29yZGVySWSIAQESJA'
    'oLY3VzdG9tZXJfaWQYAiABKAlIAVIKY3VzdG9tZXJJZIgBARIbCgZzdGF0dXMYAyABKAlIAlIG'
    'c3RhdHVziAEBEkEKCnBhZ2luYXRpb24YBCABKAsyIS5rMXMwLnN5c3RlbS5jb21tb24udjEuUG'
    'FnaW5hdGlvblIKcGFnaW5hdGlvbhJMCgtzdGF0dXNfZW51bRgFIAEoDjImLmsxczAuc2Vydmlj'
    'ZS5wYXltZW50LnYxLlBheW1lbnRTdGF0dXNIA1IKc3RhdHVzRW51bYgBAUILCglfb3JkZXJfaW'
    'RCDgoMX2N1c3RvbWVyX2lkQgkKB19zdGF0dXNCDgoMX3N0YXR1c19lbnVt');

@$core.Deprecated('Use listPaymentsResponseDescriptor instead')
const ListPaymentsResponse$json = {
  '1': 'ListPaymentsResponse',
  '2': [
    {
      '1': 'payments',
      '3': 1,
      '4': 3,
      '5': 11,
      '6': '.k1s0.service.payment.v1.Payment',
      '10': 'payments'
    },
    {
      '1': 'pagination',
      '3': 3,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.common.v1.PaginationResult',
      '10': 'pagination'
    },
  ],
  '9': [
    {'1': 2, '2': 3},
  ],
  '10': ['total_count'],
};

/// Descriptor for `ListPaymentsResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List listPaymentsResponseDescriptor = $convert.base64Decode(
    'ChRMaXN0UGF5bWVudHNSZXNwb25zZRI8CghwYXltZW50cxgBIAMoCzIgLmsxczAuc2VydmljZS'
    '5wYXltZW50LnYxLlBheW1lbnRSCHBheW1lbnRzEkcKCnBhZ2luYXRpb24YAyABKAsyJy5rMXMw'
    'LnN5c3RlbS5jb21tb24udjEuUGFnaW5hdGlvblJlc3VsdFIKcGFnaW5hdGlvbkoECAIQA1ILdG'
    '90YWxfY291bnQ=');

@$core.Deprecated('Use completePaymentRequestDescriptor instead')
const CompletePaymentRequest$json = {
  '1': 'CompletePaymentRequest',
  '2': [
    {'1': 'payment_id', '3': 1, '4': 1, '5': 9, '10': 'paymentId'},
    {'1': 'transaction_id', '3': 2, '4': 1, '5': 9, '10': 'transactionId'},
  ],
};

/// Descriptor for `CompletePaymentRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List completePaymentRequestDescriptor =
    $convert.base64Decode(
        'ChZDb21wbGV0ZVBheW1lbnRSZXF1ZXN0Eh0KCnBheW1lbnRfaWQYASABKAlSCXBheW1lbnRJZB'
        'IlCg50cmFuc2FjdGlvbl9pZBgCIAEoCVINdHJhbnNhY3Rpb25JZA==');

@$core.Deprecated('Use completePaymentResponseDescriptor instead')
const CompletePaymentResponse$json = {
  '1': 'CompletePaymentResponse',
  '2': [
    {
      '1': 'payment',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.k1s0.service.payment.v1.Payment',
      '10': 'payment'
    },
  ],
};

/// Descriptor for `CompletePaymentResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List completePaymentResponseDescriptor =
    $convert.base64Decode(
        'ChdDb21wbGV0ZVBheW1lbnRSZXNwb25zZRI6CgdwYXltZW50GAEgASgLMiAuazFzMC5zZXJ2aW'
        'NlLnBheW1lbnQudjEuUGF5bWVudFIHcGF5bWVudA==');

@$core.Deprecated('Use failPaymentRequestDescriptor instead')
const FailPaymentRequest$json = {
  '1': 'FailPaymentRequest',
  '2': [
    {'1': 'payment_id', '3': 1, '4': 1, '5': 9, '10': 'paymentId'},
    {'1': 'error_code', '3': 2, '4': 1, '5': 9, '10': 'errorCode'},
    {'1': 'error_message', '3': 3, '4': 1, '5': 9, '10': 'errorMessage'},
  ],
};

/// Descriptor for `FailPaymentRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List failPaymentRequestDescriptor = $convert.base64Decode(
    'ChJGYWlsUGF5bWVudFJlcXVlc3QSHQoKcGF5bWVudF9pZBgBIAEoCVIJcGF5bWVudElkEh0KCm'
    'Vycm9yX2NvZGUYAiABKAlSCWVycm9yQ29kZRIjCg1lcnJvcl9tZXNzYWdlGAMgASgJUgxlcnJv'
    'ck1lc3NhZ2U=');

@$core.Deprecated('Use failPaymentResponseDescriptor instead')
const FailPaymentResponse$json = {
  '1': 'FailPaymentResponse',
  '2': [
    {
      '1': 'payment',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.k1s0.service.payment.v1.Payment',
      '10': 'payment'
    },
  ],
};

/// Descriptor for `FailPaymentResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List failPaymentResponseDescriptor = $convert.base64Decode(
    'ChNGYWlsUGF5bWVudFJlc3BvbnNlEjoKB3BheW1lbnQYASABKAsyIC5rMXMwLnNlcnZpY2UucG'
    'F5bWVudC52MS5QYXltZW50UgdwYXltZW50');

@$core.Deprecated('Use refundPaymentRequestDescriptor instead')
const RefundPaymentRequest$json = {
  '1': 'RefundPaymentRequest',
  '2': [
    {'1': 'payment_id', '3': 1, '4': 1, '5': 9, '10': 'paymentId'},
    {'1': 'reason', '3': 2, '4': 1, '5': 9, '9': 0, '10': 'reason', '17': true},
  ],
  '8': [
    {'1': '_reason'},
  ],
};

/// Descriptor for `RefundPaymentRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List refundPaymentRequestDescriptor = $convert.base64Decode(
    'ChRSZWZ1bmRQYXltZW50UmVxdWVzdBIdCgpwYXltZW50X2lkGAEgASgJUglwYXltZW50SWQSGw'
    'oGcmVhc29uGAIgASgJSABSBnJlYXNvbogBAUIJCgdfcmVhc29u');

@$core.Deprecated('Use refundPaymentResponseDescriptor instead')
const RefundPaymentResponse$json = {
  '1': 'RefundPaymentResponse',
  '2': [
    {
      '1': 'payment',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.k1s0.service.payment.v1.Payment',
      '10': 'payment'
    },
  ],
};

/// Descriptor for `RefundPaymentResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List refundPaymentResponseDescriptor = $convert.base64Decode(
    'ChVSZWZ1bmRQYXltZW50UmVzcG9uc2USOgoHcGF5bWVudBgBIAEoCzIgLmsxczAuc2VydmljZS'
    '5wYXltZW50LnYxLlBheW1lbnRSB3BheW1lbnQ=');
