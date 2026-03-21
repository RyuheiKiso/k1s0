// This is a generated file - do not edit.
//
// Generated from k1s0/system/dlq/v1/dlq.proto.

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

@$core.Deprecated('Use dlqMessageStatusDescriptor instead')
const DlqMessageStatus$json = {
  '1': 'DlqMessageStatus',
  '2': [
    {'1': 'DLQ_MESSAGE_STATUS_UNSPECIFIED', '2': 0},
    {'1': 'DLQ_MESSAGE_STATUS_PENDING', '2': 1},
    {'1': 'DLQ_MESSAGE_STATUS_RETRYING', '2': 2},
    {'1': 'DLQ_MESSAGE_STATUS_SUCCEEDED', '2': 3},
    {'1': 'DLQ_MESSAGE_STATUS_FAILED', '2': 4},
  ],
};

/// Descriptor for `DlqMessageStatus`. Decode as a `google.protobuf.EnumDescriptorProto`.
final $typed_data.Uint8List dlqMessageStatusDescriptor = $convert.base64Decode(
    'ChBEbHFNZXNzYWdlU3RhdHVzEiIKHkRMUV9NRVNTQUdFX1NUQVRVU19VTlNQRUNJRklFRBAAEh'
    '4KGkRMUV9NRVNTQUdFX1NUQVRVU19QRU5ESU5HEAESHwobRExRX01FU1NBR0VfU1RBVFVTX1JF'
    'VFJZSU5HEAISIAocRExRX01FU1NBR0VfU1RBVFVTX1NVQ0NFRURFRBADEh0KGURMUV9NRVNTQU'
    'dFX1NUQVRVU19GQUlMRUQQBA==');

@$core.Deprecated('Use dlqMessageDescriptor instead')
const DlqMessage$json = {
  '1': 'DlqMessage',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 9, '10': 'id'},
    {'1': 'original_topic', '3': 2, '4': 1, '5': 9, '10': 'originalTopic'},
    {'1': 'error_message', '3': 3, '4': 1, '5': 9, '10': 'errorMessage'},
    {'1': 'retry_count', '3': 4, '4': 1, '5': 5, '10': 'retryCount'},
    {'1': 'max_retries', '3': 5, '4': 1, '5': 5, '10': 'maxRetries'},
    {'1': 'payload', '3': 6, '4': 1, '5': 12, '10': 'payload'},
    {'1': 'status', '3': 7, '4': 1, '5': 9, '10': 'status'},
    {
      '1': 'created_at',
      '3': 8,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.common.v1.Timestamp',
      '10': 'createdAt'
    },
    {
      '1': 'updated_at',
      '3': 9,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.common.v1.Timestamp',
      '10': 'updatedAt'
    },
    {
      '1': 'last_retry_at',
      '3': 10,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.common.v1.Timestamp',
      '9': 0,
      '10': 'lastRetryAt',
      '17': true
    },
    {
      '1': 'status_enum',
      '3': 11,
      '4': 1,
      '5': 14,
      '6': '.k1s0.system.dlq.v1.DlqMessageStatus',
      '10': 'statusEnum'
    },
  ],
  '8': [
    {'1': '_last_retry_at'},
  ],
};

/// Descriptor for `DlqMessage`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List dlqMessageDescriptor = $convert.base64Decode(
    'CgpEbHFNZXNzYWdlEg4KAmlkGAEgASgJUgJpZBIlCg5vcmlnaW5hbF90b3BpYxgCIAEoCVINb3'
    'JpZ2luYWxUb3BpYxIjCg1lcnJvcl9tZXNzYWdlGAMgASgJUgxlcnJvck1lc3NhZ2USHwoLcmV0'
    'cnlfY291bnQYBCABKAVSCnJldHJ5Q291bnQSHwoLbWF4X3JldHJpZXMYBSABKAVSCm1heFJldH'
    'JpZXMSGAoHcGF5bG9hZBgGIAEoDFIHcGF5bG9hZBIWCgZzdGF0dXMYByABKAlSBnN0YXR1cxI/'
    'CgpjcmVhdGVkX2F0GAggASgLMiAuazFzMC5zeXN0ZW0uY29tbW9uLnYxLlRpbWVzdGFtcFIJY3'
    'JlYXRlZEF0Ej8KCnVwZGF0ZWRfYXQYCSABKAsyIC5rMXMwLnN5c3RlbS5jb21tb24udjEuVGlt'
    'ZXN0YW1wUgl1cGRhdGVkQXQSSQoNbGFzdF9yZXRyeV9hdBgKIAEoCzIgLmsxczAuc3lzdGVtLm'
    'NvbW1vbi52MS5UaW1lc3RhbXBIAFILbGFzdFJldHJ5QXSIAQESRQoLc3RhdHVzX2VudW0YCyAB'
    'KA4yJC5rMXMwLnN5c3RlbS5kbHEudjEuRGxxTWVzc2FnZVN0YXR1c1IKc3RhdHVzRW51bUIQCg'
    '5fbGFzdF9yZXRyeV9hdA==');

@$core.Deprecated('Use listMessagesRequestDescriptor instead')
const ListMessagesRequest$json = {
  '1': 'ListMessagesRequest',
  '2': [
    {'1': 'topic', '3': 1, '4': 1, '5': 9, '10': 'topic'},
    {
      '1': 'pagination',
      '3': 2,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.common.v1.Pagination',
      '10': 'pagination'
    },
  ],
  '9': [
    {'1': 3, '2': 4},
  ],
  '10': ['page_size'],
};

/// Descriptor for `ListMessagesRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List listMessagesRequestDescriptor = $convert.base64Decode(
    'ChNMaXN0TWVzc2FnZXNSZXF1ZXN0EhQKBXRvcGljGAEgASgJUgV0b3BpYxJBCgpwYWdpbmF0aW'
    '9uGAIgASgLMiEuazFzMC5zeXN0ZW0uY29tbW9uLnYxLlBhZ2luYXRpb25SCnBhZ2luYXRpb25K'
    'BAgDEARSCXBhZ2Vfc2l6ZQ==');

@$core.Deprecated('Use listMessagesResponseDescriptor instead')
const ListMessagesResponse$json = {
  '1': 'ListMessagesResponse',
  '2': [
    {
      '1': 'messages',
      '3': 1,
      '4': 3,
      '5': 11,
      '6': '.k1s0.system.dlq.v1.DlqMessage',
      '10': 'messages'
    },
    {
      '1': 'pagination',
      '3': 2,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.common.v1.PaginationResult',
      '10': 'pagination'
    },
  ],
};

/// Descriptor for `ListMessagesResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List listMessagesResponseDescriptor = $convert.base64Decode(
    'ChRMaXN0TWVzc2FnZXNSZXNwb25zZRI6CghtZXNzYWdlcxgBIAMoCzIeLmsxczAuc3lzdGVtLm'
    'RscS52MS5EbHFNZXNzYWdlUghtZXNzYWdlcxJHCgpwYWdpbmF0aW9uGAIgASgLMicuazFzMC5z'
    'eXN0ZW0uY29tbW9uLnYxLlBhZ2luYXRpb25SZXN1bHRSCnBhZ2luYXRpb24=');

@$core.Deprecated('Use getMessageRequestDescriptor instead')
const GetMessageRequest$json = {
  '1': 'GetMessageRequest',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 9, '10': 'id'},
  ],
};

/// Descriptor for `GetMessageRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List getMessageRequestDescriptor =
    $convert.base64Decode('ChFHZXRNZXNzYWdlUmVxdWVzdBIOCgJpZBgBIAEoCVICaWQ=');

@$core.Deprecated('Use getMessageResponseDescriptor instead')
const GetMessageResponse$json = {
  '1': 'GetMessageResponse',
  '2': [
    {
      '1': 'message',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.dlq.v1.DlqMessage',
      '10': 'message'
    },
  ],
};

/// Descriptor for `GetMessageResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List getMessageResponseDescriptor = $convert.base64Decode(
    'ChJHZXRNZXNzYWdlUmVzcG9uc2USOAoHbWVzc2FnZRgBIAEoCzIeLmsxczAuc3lzdGVtLmRscS'
    '52MS5EbHFNZXNzYWdlUgdtZXNzYWdl');

@$core.Deprecated('Use retryMessageRequestDescriptor instead')
const RetryMessageRequest$json = {
  '1': 'RetryMessageRequest',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 9, '10': 'id'},
  ],
};

/// Descriptor for `RetryMessageRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List retryMessageRequestDescriptor = $convert
    .base64Decode('ChNSZXRyeU1lc3NhZ2VSZXF1ZXN0Eg4KAmlkGAEgASgJUgJpZA==');

@$core.Deprecated('Use retryMessageResponseDescriptor instead')
const RetryMessageResponse$json = {
  '1': 'RetryMessageResponse',
  '2': [
    {
      '1': 'message',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.dlq.v1.DlqMessage',
      '10': 'message'
    },
  ],
};

/// Descriptor for `RetryMessageResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List retryMessageResponseDescriptor = $convert.base64Decode(
    'ChRSZXRyeU1lc3NhZ2VSZXNwb25zZRI4CgdtZXNzYWdlGAEgASgLMh4uazFzMC5zeXN0ZW0uZG'
    'xxLnYxLkRscU1lc3NhZ2VSB21lc3NhZ2U=');

@$core.Deprecated('Use deleteMessageRequestDescriptor instead')
const DeleteMessageRequest$json = {
  '1': 'DeleteMessageRequest',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 9, '10': 'id'},
  ],
};

/// Descriptor for `DeleteMessageRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List deleteMessageRequestDescriptor = $convert
    .base64Decode('ChREZWxldGVNZXNzYWdlUmVxdWVzdBIOCgJpZBgBIAEoCVICaWQ=');

@$core.Deprecated('Use deleteMessageResponseDescriptor instead')
const DeleteMessageResponse$json = {
  '1': 'DeleteMessageResponse',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 9, '10': 'id'},
  ],
};

/// Descriptor for `DeleteMessageResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List deleteMessageResponseDescriptor = $convert
    .base64Decode('ChVEZWxldGVNZXNzYWdlUmVzcG9uc2USDgoCaWQYASABKAlSAmlk');

@$core.Deprecated('Use retryAllRequestDescriptor instead')
const RetryAllRequest$json = {
  '1': 'RetryAllRequest',
  '2': [
    {'1': 'topic', '3': 1, '4': 1, '5': 9, '10': 'topic'},
  ],
};

/// Descriptor for `RetryAllRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List retryAllRequestDescriptor = $convert
    .base64Decode('Cg9SZXRyeUFsbFJlcXVlc3QSFAoFdG9waWMYASABKAlSBXRvcGlj');

@$core.Deprecated('Use retryAllResponseDescriptor instead')
const RetryAllResponse$json = {
  '1': 'RetryAllResponse',
  '2': [
    {'1': 'retried_count', '3': 1, '4': 1, '5': 5, '10': 'retriedCount'},
    {'1': 'message', '3': 2, '4': 1, '5': 9, '10': 'message'},
  ],
};

/// Descriptor for `RetryAllResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List retryAllResponseDescriptor = $convert.base64Decode(
    'ChBSZXRyeUFsbFJlc3BvbnNlEiMKDXJldHJpZWRfY291bnQYASABKAVSDHJldHJpZWRDb3VudB'
    'IYCgdtZXNzYWdlGAIgASgJUgdtZXNzYWdl');
