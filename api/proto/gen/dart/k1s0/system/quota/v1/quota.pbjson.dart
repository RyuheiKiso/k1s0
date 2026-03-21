// This is a generated file - do not edit.
//
// Generated from k1s0/system/quota/v1/quota.proto.

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

@$core.Deprecated('Use quotaPolicyDescriptor instead')
const QuotaPolicy$json = {
  '1': 'QuotaPolicy',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 9, '10': 'id'},
    {'1': 'name', '3': 2, '4': 1, '5': 9, '10': 'name'},
    {'1': 'subject_type', '3': 3, '4': 1, '5': 9, '10': 'subjectType'},
    {'1': 'subject_id', '3': 4, '4': 1, '5': 9, '10': 'subjectId'},
    {'1': 'limit', '3': 5, '4': 1, '5': 4, '10': 'limit'},
    {'1': 'period', '3': 6, '4': 1, '5': 9, '10': 'period'},
    {'1': 'enabled', '3': 7, '4': 1, '5': 8, '10': 'enabled'},
    {
      '1': 'alert_threshold_percent',
      '3': 8,
      '4': 1,
      '5': 13,
      '9': 0,
      '10': 'alertThresholdPercent',
      '17': true
    },
    {
      '1': 'created_at',
      '3': 9,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.common.v1.Timestamp',
      '10': 'createdAt'
    },
    {
      '1': 'updated_at',
      '3': 10,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.common.v1.Timestamp',
      '10': 'updatedAt'
    },
  ],
  '8': [
    {'1': '_alert_threshold_percent'},
  ],
};

/// Descriptor for `QuotaPolicy`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List quotaPolicyDescriptor = $convert.base64Decode(
    'CgtRdW90YVBvbGljeRIOCgJpZBgBIAEoCVICaWQSEgoEbmFtZRgCIAEoCVIEbmFtZRIhCgxzdW'
    'JqZWN0X3R5cGUYAyABKAlSC3N1YmplY3RUeXBlEh0KCnN1YmplY3RfaWQYBCABKAlSCXN1Ympl'
    'Y3RJZBIUCgVsaW1pdBgFIAEoBFIFbGltaXQSFgoGcGVyaW9kGAYgASgJUgZwZXJpb2QSGAoHZW'
    '5hYmxlZBgHIAEoCFIHZW5hYmxlZBI7ChdhbGVydF90aHJlc2hvbGRfcGVyY2VudBgIIAEoDUgA'
    'UhVhbGVydFRocmVzaG9sZFBlcmNlbnSIAQESPwoKY3JlYXRlZF9hdBgJIAEoCzIgLmsxczAuc3'
    'lzdGVtLmNvbW1vbi52MS5UaW1lc3RhbXBSCWNyZWF0ZWRBdBI/Cgp1cGRhdGVkX2F0GAogASgL'
    'MiAuazFzMC5zeXN0ZW0uY29tbW9uLnYxLlRpbWVzdGFtcFIJdXBkYXRlZEF0QhoKGF9hbGVydF'
    '90aHJlc2hvbGRfcGVyY2VudA==');

@$core.Deprecated('Use quotaUsageDescriptor instead')
const QuotaUsage$json = {
  '1': 'QuotaUsage',
  '2': [
    {'1': 'quota_id', '3': 1, '4': 1, '5': 9, '10': 'quotaId'},
    {'1': 'subject_type', '3': 2, '4': 1, '5': 9, '10': 'subjectType'},
    {'1': 'subject_id', '3': 3, '4': 1, '5': 9, '10': 'subjectId'},
    {'1': 'period', '3': 4, '4': 1, '5': 9, '10': 'period'},
    {'1': 'limit', '3': 5, '4': 1, '5': 4, '10': 'limit'},
    {'1': 'used', '3': 6, '4': 1, '5': 4, '10': 'used'},
    {'1': 'remaining', '3': 7, '4': 1, '5': 4, '10': 'remaining'},
    {'1': 'usage_percent', '3': 8, '4': 1, '5': 1, '10': 'usagePercent'},
    {'1': 'exceeded', '3': 9, '4': 1, '5': 8, '10': 'exceeded'},
    {
      '1': 'period_start',
      '3': 10,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.common.v1.Timestamp',
      '10': 'periodStart'
    },
    {
      '1': 'period_end',
      '3': 11,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.common.v1.Timestamp',
      '10': 'periodEnd'
    },
    {
      '1': 'reset_at',
      '3': 12,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.common.v1.Timestamp',
      '10': 'resetAt'
    },
  ],
};

/// Descriptor for `QuotaUsage`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List quotaUsageDescriptor = $convert.base64Decode(
    'CgpRdW90YVVzYWdlEhkKCHF1b3RhX2lkGAEgASgJUgdxdW90YUlkEiEKDHN1YmplY3RfdHlwZR'
    'gCIAEoCVILc3ViamVjdFR5cGUSHQoKc3ViamVjdF9pZBgDIAEoCVIJc3ViamVjdElkEhYKBnBl'
    'cmlvZBgEIAEoCVIGcGVyaW9kEhQKBWxpbWl0GAUgASgEUgVsaW1pdBISCgR1c2VkGAYgASgEUg'
    'R1c2VkEhwKCXJlbWFpbmluZxgHIAEoBFIJcmVtYWluaW5nEiMKDXVzYWdlX3BlcmNlbnQYCCAB'
    'KAFSDHVzYWdlUGVyY2VudBIaCghleGNlZWRlZBgJIAEoCFIIZXhjZWVkZWQSQwoMcGVyaW9kX3'
    'N0YXJ0GAogASgLMiAuazFzMC5zeXN0ZW0uY29tbW9uLnYxLlRpbWVzdGFtcFILcGVyaW9kU3Rh'
    'cnQSPwoKcGVyaW9kX2VuZBgLIAEoCzIgLmsxczAuc3lzdGVtLmNvbW1vbi52MS5UaW1lc3RhbX'
    'BSCXBlcmlvZEVuZBI7CghyZXNldF9hdBgMIAEoCzIgLmsxczAuc3lzdGVtLmNvbW1vbi52MS5U'
    'aW1lc3RhbXBSB3Jlc2V0QXQ=');

@$core.Deprecated('Use createQuotaPolicyRequestDescriptor instead')
const CreateQuotaPolicyRequest$json = {
  '1': 'CreateQuotaPolicyRequest',
  '2': [
    {'1': 'name', '3': 1, '4': 1, '5': 9, '10': 'name'},
    {'1': 'subject_type', '3': 2, '4': 1, '5': 9, '10': 'subjectType'},
    {'1': 'subject_id', '3': 3, '4': 1, '5': 9, '10': 'subjectId'},
    {'1': 'limit', '3': 4, '4': 1, '5': 4, '10': 'limit'},
    {'1': 'period', '3': 5, '4': 1, '5': 9, '10': 'period'},
    {'1': 'enabled', '3': 6, '4': 1, '5': 8, '10': 'enabled'},
    {
      '1': 'alert_threshold_percent',
      '3': 7,
      '4': 1,
      '5': 13,
      '9': 0,
      '10': 'alertThresholdPercent',
      '17': true
    },
  ],
  '8': [
    {'1': '_alert_threshold_percent'},
  ],
};

/// Descriptor for `CreateQuotaPolicyRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List createQuotaPolicyRequestDescriptor = $convert.base64Decode(
    'ChhDcmVhdGVRdW90YVBvbGljeVJlcXVlc3QSEgoEbmFtZRgBIAEoCVIEbmFtZRIhCgxzdWJqZW'
    'N0X3R5cGUYAiABKAlSC3N1YmplY3RUeXBlEh0KCnN1YmplY3RfaWQYAyABKAlSCXN1YmplY3RJ'
    'ZBIUCgVsaW1pdBgEIAEoBFIFbGltaXQSFgoGcGVyaW9kGAUgASgJUgZwZXJpb2QSGAoHZW5hYm'
    'xlZBgGIAEoCFIHZW5hYmxlZBI7ChdhbGVydF90aHJlc2hvbGRfcGVyY2VudBgHIAEoDUgAUhVh'
    'bGVydFRocmVzaG9sZFBlcmNlbnSIAQFCGgoYX2FsZXJ0X3RocmVzaG9sZF9wZXJjZW50');

@$core.Deprecated('Use createQuotaPolicyResponseDescriptor instead')
const CreateQuotaPolicyResponse$json = {
  '1': 'CreateQuotaPolicyResponse',
  '2': [
    {
      '1': 'policy',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.quota.v1.QuotaPolicy',
      '10': 'policy'
    },
  ],
};

/// Descriptor for `CreateQuotaPolicyResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List createQuotaPolicyResponseDescriptor =
    $convert.base64Decode(
        'ChlDcmVhdGVRdW90YVBvbGljeVJlc3BvbnNlEjkKBnBvbGljeRgBIAEoCzIhLmsxczAuc3lzdG'
        'VtLnF1b3RhLnYxLlF1b3RhUG9saWN5UgZwb2xpY3k=');

@$core.Deprecated('Use getQuotaPolicyRequestDescriptor instead')
const GetQuotaPolicyRequest$json = {
  '1': 'GetQuotaPolicyRequest',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 9, '10': 'id'},
  ],
};

/// Descriptor for `GetQuotaPolicyRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List getQuotaPolicyRequestDescriptor = $convert
    .base64Decode('ChVHZXRRdW90YVBvbGljeVJlcXVlc3QSDgoCaWQYASABKAlSAmlk');

@$core.Deprecated('Use getQuotaPolicyResponseDescriptor instead')
const GetQuotaPolicyResponse$json = {
  '1': 'GetQuotaPolicyResponse',
  '2': [
    {
      '1': 'policy',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.quota.v1.QuotaPolicy',
      '10': 'policy'
    },
  ],
};

/// Descriptor for `GetQuotaPolicyResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List getQuotaPolicyResponseDescriptor =
    $convert.base64Decode(
        'ChZHZXRRdW90YVBvbGljeVJlc3BvbnNlEjkKBnBvbGljeRgBIAEoCzIhLmsxczAuc3lzdGVtLn'
        'F1b3RhLnYxLlF1b3RhUG9saWN5UgZwb2xpY3k=');

@$core.Deprecated('Use listQuotaPoliciesRequestDescriptor instead')
const ListQuotaPoliciesRequest$json = {
  '1': 'ListQuotaPoliciesRequest',
  '2': [
    {
      '1': 'pagination',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.common.v1.Pagination',
      '10': 'pagination'
    },
    {
      '1': 'subject_type',
      '3': 3,
      '4': 1,
      '5': 9,
      '9': 0,
      '10': 'subjectType',
      '17': true
    },
    {
      '1': 'subject_id',
      '3': 4,
      '4': 1,
      '5': 9,
      '9': 1,
      '10': 'subjectId',
      '17': true
    },
    {
      '1': 'enabled_only',
      '3': 5,
      '4': 1,
      '5': 8,
      '9': 2,
      '10': 'enabledOnly',
      '17': true
    },
  ],
  '8': [
    {'1': '_subject_type'},
    {'1': '_subject_id'},
    {'1': '_enabled_only'},
  ],
  '9': [
    {'1': 2, '2': 3},
  ],
  '10': ['page_size'],
};

/// Descriptor for `ListQuotaPoliciesRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List listQuotaPoliciesRequestDescriptor = $convert.base64Decode(
    'ChhMaXN0UXVvdGFQb2xpY2llc1JlcXVlc3QSQQoKcGFnaW5hdGlvbhgBIAEoCzIhLmsxczAuc3'
    'lzdGVtLmNvbW1vbi52MS5QYWdpbmF0aW9uUgpwYWdpbmF0aW9uEiYKDHN1YmplY3RfdHlwZRgD'
    'IAEoCUgAUgtzdWJqZWN0VHlwZYgBARIiCgpzdWJqZWN0X2lkGAQgASgJSAFSCXN1YmplY3RJZI'
    'gBARImCgxlbmFibGVkX29ubHkYBSABKAhIAlILZW5hYmxlZE9ubHmIAQFCDwoNX3N1YmplY3Rf'
    'dHlwZUINCgtfc3ViamVjdF9pZEIPCg1fZW5hYmxlZF9vbmx5SgQIAhADUglwYWdlX3NpemU=');

@$core.Deprecated('Use listQuotaPoliciesResponseDescriptor instead')
const ListQuotaPoliciesResponse$json = {
  '1': 'ListQuotaPoliciesResponse',
  '2': [
    {
      '1': 'policies',
      '3': 1,
      '4': 3,
      '5': 11,
      '6': '.k1s0.system.quota.v1.QuotaPolicy',
      '10': 'policies'
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

/// Descriptor for `ListQuotaPoliciesResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List listQuotaPoliciesResponseDescriptor = $convert.base64Decode(
    'ChlMaXN0UXVvdGFQb2xpY2llc1Jlc3BvbnNlEj0KCHBvbGljaWVzGAEgAygLMiEuazFzMC5zeX'
    'N0ZW0ucXVvdGEudjEuUXVvdGFQb2xpY3lSCHBvbGljaWVzEkcKCnBhZ2luYXRpb24YAiABKAsy'
    'Jy5rMXMwLnN5c3RlbS5jb21tb24udjEuUGFnaW5hdGlvblJlc3VsdFIKcGFnaW5hdGlvbg==');

@$core.Deprecated('Use updateQuotaPolicyRequestDescriptor instead')
const UpdateQuotaPolicyRequest$json = {
  '1': 'UpdateQuotaPolicyRequest',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 9, '10': 'id'},
    {
      '1': 'enabled',
      '3': 2,
      '4': 1,
      '5': 8,
      '9': 0,
      '10': 'enabled',
      '17': true
    },
    {'1': 'limit', '3': 3, '4': 1, '5': 4, '9': 1, '10': 'limit', '17': true},
    {'1': 'name', '3': 4, '4': 1, '5': 9, '9': 2, '10': 'name', '17': true},
    {
      '1': 'subject_type',
      '3': 5,
      '4': 1,
      '5': 9,
      '9': 3,
      '10': 'subjectType',
      '17': true
    },
    {
      '1': 'subject_id',
      '3': 6,
      '4': 1,
      '5': 9,
      '9': 4,
      '10': 'subjectId',
      '17': true
    },
    {'1': 'period', '3': 7, '4': 1, '5': 9, '9': 5, '10': 'period', '17': true},
    {
      '1': 'alert_threshold_percent',
      '3': 8,
      '4': 1,
      '5': 13,
      '9': 6,
      '10': 'alertThresholdPercent',
      '17': true
    },
  ],
  '8': [
    {'1': '_enabled'},
    {'1': '_limit'},
    {'1': '_name'},
    {'1': '_subject_type'},
    {'1': '_subject_id'},
    {'1': '_period'},
    {'1': '_alert_threshold_percent'},
  ],
};

/// Descriptor for `UpdateQuotaPolicyRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List updateQuotaPolicyRequestDescriptor = $convert.base64Decode(
    'ChhVcGRhdGVRdW90YVBvbGljeVJlcXVlc3QSDgoCaWQYASABKAlSAmlkEh0KB2VuYWJsZWQYAi'
    'ABKAhIAFIHZW5hYmxlZIgBARIZCgVsaW1pdBgDIAEoBEgBUgVsaW1pdIgBARIXCgRuYW1lGAQg'
    'ASgJSAJSBG5hbWWIAQESJgoMc3ViamVjdF90eXBlGAUgASgJSANSC3N1YmplY3RUeXBliAEBEi'
    'IKCnN1YmplY3RfaWQYBiABKAlIBFIJc3ViamVjdElkiAEBEhsKBnBlcmlvZBgHIAEoCUgFUgZw'
    'ZXJpb2SIAQESOwoXYWxlcnRfdGhyZXNob2xkX3BlcmNlbnQYCCABKA1IBlIVYWxlcnRUaHJlc2'
    'hvbGRQZXJjZW50iAEBQgoKCF9lbmFibGVkQggKBl9saW1pdEIHCgVfbmFtZUIPCg1fc3ViamVj'
    'dF90eXBlQg0KC19zdWJqZWN0X2lkQgkKB19wZXJpb2RCGgoYX2FsZXJ0X3RocmVzaG9sZF9wZX'
    'JjZW50');

@$core.Deprecated('Use updateQuotaPolicyResponseDescriptor instead')
const UpdateQuotaPolicyResponse$json = {
  '1': 'UpdateQuotaPolicyResponse',
  '2': [
    {
      '1': 'policy',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.quota.v1.QuotaPolicy',
      '10': 'policy'
    },
  ],
};

/// Descriptor for `UpdateQuotaPolicyResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List updateQuotaPolicyResponseDescriptor =
    $convert.base64Decode(
        'ChlVcGRhdGVRdW90YVBvbGljeVJlc3BvbnNlEjkKBnBvbGljeRgBIAEoCzIhLmsxczAuc3lzdG'
        'VtLnF1b3RhLnYxLlF1b3RhUG9saWN5UgZwb2xpY3k=');

@$core.Deprecated('Use deleteQuotaPolicyRequestDescriptor instead')
const DeleteQuotaPolicyRequest$json = {
  '1': 'DeleteQuotaPolicyRequest',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 9, '10': 'id'},
  ],
};

/// Descriptor for `DeleteQuotaPolicyRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List deleteQuotaPolicyRequestDescriptor = $convert
    .base64Decode('ChhEZWxldGVRdW90YVBvbGljeVJlcXVlc3QSDgoCaWQYASABKAlSAmlk');

@$core.Deprecated('Use deleteQuotaPolicyResponseDescriptor instead')
const DeleteQuotaPolicyResponse$json = {
  '1': 'DeleteQuotaPolicyResponse',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 9, '10': 'id'},
    {'1': 'deleted', '3': 2, '4': 1, '5': 8, '10': 'deleted'},
  ],
};

/// Descriptor for `DeleteQuotaPolicyResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List deleteQuotaPolicyResponseDescriptor =
    $convert.base64Decode(
        'ChlEZWxldGVRdW90YVBvbGljeVJlc3BvbnNlEg4KAmlkGAEgASgJUgJpZBIYCgdkZWxldGVkGA'
        'IgASgIUgdkZWxldGVk');

@$core.Deprecated('Use getQuotaUsageRequestDescriptor instead')
const GetQuotaUsageRequest$json = {
  '1': 'GetQuotaUsageRequest',
  '2': [
    {'1': 'quota_id', '3': 1, '4': 1, '5': 9, '10': 'quotaId'},
  ],
};

/// Descriptor for `GetQuotaUsageRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List getQuotaUsageRequestDescriptor =
    $convert.base64Decode(
        'ChRHZXRRdW90YVVzYWdlUmVxdWVzdBIZCghxdW90YV9pZBgBIAEoCVIHcXVvdGFJZA==');

@$core.Deprecated('Use getQuotaUsageResponseDescriptor instead')
const GetQuotaUsageResponse$json = {
  '1': 'GetQuotaUsageResponse',
  '2': [
    {
      '1': 'usage',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.quota.v1.QuotaUsage',
      '10': 'usage'
    },
  ],
};

/// Descriptor for `GetQuotaUsageResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List getQuotaUsageResponseDescriptor = $convert.base64Decode(
    'ChVHZXRRdW90YVVzYWdlUmVzcG9uc2USNgoFdXNhZ2UYASABKAsyIC5rMXMwLnN5c3RlbS5xdW'
    '90YS52MS5RdW90YVVzYWdlUgV1c2FnZQ==');

@$core.Deprecated('Use checkQuotaRequestDescriptor instead')
const CheckQuotaRequest$json = {
  '1': 'CheckQuotaRequest',
  '2': [
    {'1': 'quota_id', '3': 1, '4': 1, '5': 9, '10': 'quotaId'},
  ],
};

/// Descriptor for `CheckQuotaRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List checkQuotaRequestDescriptor = $convert.base64Decode(
    'ChFDaGVja1F1b3RhUmVxdWVzdBIZCghxdW90YV9pZBgBIAEoCVIHcXVvdGFJZA==');

@$core.Deprecated('Use checkQuotaResponseDescriptor instead')
const CheckQuotaResponse$json = {
  '1': 'CheckQuotaResponse',
  '2': [
    {
      '1': 'usage',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.quota.v1.QuotaUsage',
      '10': 'usage'
    },
  ],
};

/// Descriptor for `CheckQuotaResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List checkQuotaResponseDescriptor = $convert.base64Decode(
    'ChJDaGVja1F1b3RhUmVzcG9uc2USNgoFdXNhZ2UYASABKAsyIC5rMXMwLnN5c3RlbS5xdW90YS'
    '52MS5RdW90YVVzYWdlUgV1c2FnZQ==');

@$core.Deprecated('Use incrementQuotaUsageRequestDescriptor instead')
const IncrementQuotaUsageRequest$json = {
  '1': 'IncrementQuotaUsageRequest',
  '2': [
    {'1': 'quota_id', '3': 1, '4': 1, '5': 9, '10': 'quotaId'},
    {'1': 'amount', '3': 2, '4': 1, '5': 4, '10': 'amount'},
    {
      '1': 'request_id',
      '3': 3,
      '4': 1,
      '5': 9,
      '9': 0,
      '10': 'requestId',
      '17': true
    },
  ],
  '8': [
    {'1': '_request_id'},
  ],
};

/// Descriptor for `IncrementQuotaUsageRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List incrementQuotaUsageRequestDescriptor =
    $convert.base64Decode(
        'ChpJbmNyZW1lbnRRdW90YVVzYWdlUmVxdWVzdBIZCghxdW90YV9pZBgBIAEoCVIHcXVvdGFJZB'
        'IWCgZhbW91bnQYAiABKARSBmFtb3VudBIiCgpyZXF1ZXN0X2lkGAMgASgJSABSCXJlcXVlc3RJ'
        'ZIgBAUINCgtfcmVxdWVzdF9pZA==');

@$core.Deprecated('Use incrementQuotaUsageResponseDescriptor instead')
const IncrementQuotaUsageResponse$json = {
  '1': 'IncrementQuotaUsageResponse',
  '2': [
    {'1': 'quota_id', '3': 1, '4': 1, '5': 9, '10': 'quotaId'},
    {'1': 'used', '3': 2, '4': 1, '5': 4, '10': 'used'},
    {'1': 'remaining', '3': 3, '4': 1, '5': 4, '10': 'remaining'},
    {'1': 'usage_percent', '3': 4, '4': 1, '5': 1, '10': 'usagePercent'},
    {'1': 'exceeded', '3': 5, '4': 1, '5': 8, '10': 'exceeded'},
    {'1': 'allowed', '3': 6, '4': 1, '5': 8, '10': 'allowed'},
  ],
};

/// Descriptor for `IncrementQuotaUsageResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List incrementQuotaUsageResponseDescriptor = $convert.base64Decode(
    'ChtJbmNyZW1lbnRRdW90YVVzYWdlUmVzcG9uc2USGQoIcXVvdGFfaWQYASABKAlSB3F1b3RhSW'
    'QSEgoEdXNlZBgCIAEoBFIEdXNlZBIcCglyZW1haW5pbmcYAyABKARSCXJlbWFpbmluZxIjCg11'
    'c2FnZV9wZXJjZW50GAQgASgBUgx1c2FnZVBlcmNlbnQSGgoIZXhjZWVkZWQYBSABKAhSCGV4Y2'
    'VlZGVkEhgKB2FsbG93ZWQYBiABKAhSB2FsbG93ZWQ=');

@$core.Deprecated('Use resetQuotaUsageRequestDescriptor instead')
const ResetQuotaUsageRequest$json = {
  '1': 'ResetQuotaUsageRequest',
  '2': [
    {'1': 'quota_id', '3': 1, '4': 1, '5': 9, '10': 'quotaId'},
    {'1': 'reason', '3': 2, '4': 1, '5': 9, '10': 'reason'},
    {'1': 'reset_by', '3': 3, '4': 1, '5': 9, '10': 'resetBy'},
  ],
};

/// Descriptor for `ResetQuotaUsageRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List resetQuotaUsageRequestDescriptor =
    $convert.base64Decode(
        'ChZSZXNldFF1b3RhVXNhZ2VSZXF1ZXN0EhkKCHF1b3RhX2lkGAEgASgJUgdxdW90YUlkEhYKBn'
        'JlYXNvbhgCIAEoCVIGcmVhc29uEhkKCHJlc2V0X2J5GAMgASgJUgdyZXNldEJ5');

@$core.Deprecated('Use resetQuotaUsageResponseDescriptor instead')
const ResetQuotaUsageResponse$json = {
  '1': 'ResetQuotaUsageResponse',
  '2': [
    {
      '1': 'usage',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.quota.v1.QuotaUsage',
      '10': 'usage'
    },
  ],
};

/// Descriptor for `ResetQuotaUsageResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List resetQuotaUsageResponseDescriptor =
    $convert.base64Decode(
        'ChdSZXNldFF1b3RhVXNhZ2VSZXNwb25zZRI2CgV1c2FnZRgBIAEoCzIgLmsxczAuc3lzdGVtLn'
        'F1b3RhLnYxLlF1b3RhVXNhZ2VSBXVzYWdl');
