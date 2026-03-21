// This is a generated file - do not edit.
//
// Generated from k1s0/system/ratelimit/v1/ratelimit.proto.

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

@$core.Deprecated('Use rateLimitAlgorithmDescriptor instead')
const RateLimitAlgorithm$json = {
  '1': 'RateLimitAlgorithm',
  '2': [
    {'1': 'RATE_LIMIT_ALGORITHM_UNSPECIFIED', '2': 0},
    {'1': 'RATE_LIMIT_ALGORITHM_SLIDING_WINDOW', '2': 1},
    {'1': 'RATE_LIMIT_ALGORITHM_TOKEN_BUCKET', '2': 2},
    {'1': 'RATE_LIMIT_ALGORITHM_FIXED_WINDOW', '2': 3},
    {'1': 'RATE_LIMIT_ALGORITHM_LEAKY_BUCKET', '2': 4},
  ],
};

/// Descriptor for `RateLimitAlgorithm`. Decode as a `google.protobuf.EnumDescriptorProto`.
final $typed_data.Uint8List rateLimitAlgorithmDescriptor = $convert.base64Decode(
    'ChJSYXRlTGltaXRBbGdvcml0aG0SJAogUkFURV9MSU1JVF9BTEdPUklUSE1fVU5TUEVDSUZJRU'
    'QQABInCiNSQVRFX0xJTUlUX0FMR09SSVRITV9TTElESU5HX1dJTkRPVxABEiUKIVJBVEVfTElN'
    'SVRfQUxHT1JJVEhNX1RPS0VOX0JVQ0tFVBACEiUKIVJBVEVfTElNSVRfQUxHT1JJVEhNX0ZJWE'
    'VEX1dJTkRPVxADEiUKIVJBVEVfTElNSVRfQUxHT1JJVEhNX0xFQUtZX0JVQ0tFVBAE');

@$core.Deprecated('Use checkRateLimitRequestDescriptor instead')
const CheckRateLimitRequest$json = {
  '1': 'CheckRateLimitRequest',
  '2': [
    {'1': 'scope', '3': 1, '4': 1, '5': 9, '10': 'scope'},
    {'1': 'identifier', '3': 2, '4': 1, '5': 9, '10': 'identifier'},
    {'1': 'window', '3': 3, '4': 1, '5': 3, '10': 'window'},
  ],
};

/// Descriptor for `CheckRateLimitRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List checkRateLimitRequestDescriptor = $convert.base64Decode(
    'ChVDaGVja1JhdGVMaW1pdFJlcXVlc3QSFAoFc2NvcGUYASABKAlSBXNjb3BlEh4KCmlkZW50aW'
    'ZpZXIYAiABKAlSCmlkZW50aWZpZXISFgoGd2luZG93GAMgASgDUgZ3aW5kb3c=');

@$core.Deprecated('Use checkRateLimitResponseDescriptor instead')
const CheckRateLimitResponse$json = {
  '1': 'CheckRateLimitResponse',
  '2': [
    {'1': 'allowed', '3': 1, '4': 1, '5': 8, '10': 'allowed'},
    {'1': 'remaining', '3': 2, '4': 1, '5': 3, '10': 'remaining'},
    {'1': 'reset_at', '3': 3, '4': 1, '5': 3, '10': 'resetAt'},
    {'1': 'reason', '3': 4, '4': 1, '5': 9, '10': 'reason'},
    {'1': 'limit', '3': 5, '4': 1, '5': 3, '10': 'limit'},
    {'1': 'scope', '3': 6, '4': 1, '5': 9, '10': 'scope'},
    {'1': 'identifier', '3': 7, '4': 1, '5': 9, '10': 'identifier'},
    {'1': 'used', '3': 8, '4': 1, '5': 3, '10': 'used'},
    {'1': 'rule_id', '3': 9, '4': 1, '5': 9, '10': 'ruleId'},
  ],
};

/// Descriptor for `CheckRateLimitResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List checkRateLimitResponseDescriptor = $convert.base64Decode(
    'ChZDaGVja1JhdGVMaW1pdFJlc3BvbnNlEhgKB2FsbG93ZWQYASABKAhSB2FsbG93ZWQSHAoJcm'
    'VtYWluaW5nGAIgASgDUglyZW1haW5pbmcSGQoIcmVzZXRfYXQYAyABKANSB3Jlc2V0QXQSFgoG'
    'cmVhc29uGAQgASgJUgZyZWFzb24SFAoFbGltaXQYBSABKANSBWxpbWl0EhQKBXNjb3BlGAYgAS'
    'gJUgVzY29wZRIeCgppZGVudGlmaWVyGAcgASgJUgppZGVudGlmaWVyEhIKBHVzZWQYCCABKANS'
    'BHVzZWQSFwoHcnVsZV9pZBgJIAEoCVIGcnVsZUlk');

@$core.Deprecated('Use createRuleRequestDescriptor instead')
const CreateRuleRequest$json = {
  '1': 'CreateRuleRequest',
  '2': [
    {'1': 'scope', '3': 1, '4': 1, '5': 9, '10': 'scope'},
    {
      '1': 'identifier_pattern',
      '3': 2,
      '4': 1,
      '5': 9,
      '10': 'identifierPattern'
    },
    {'1': 'limit', '3': 3, '4': 1, '5': 3, '10': 'limit'},
    {'1': 'window_seconds', '3': 4, '4': 1, '5': 3, '10': 'windowSeconds'},
    {'1': 'enabled', '3': 5, '4': 1, '5': 8, '10': 'enabled'},
  ],
};

/// Descriptor for `CreateRuleRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List createRuleRequestDescriptor = $convert.base64Decode(
    'ChFDcmVhdGVSdWxlUmVxdWVzdBIUCgVzY29wZRgBIAEoCVIFc2NvcGUSLQoSaWRlbnRpZmllcl'
    '9wYXR0ZXJuGAIgASgJUhFpZGVudGlmaWVyUGF0dGVybhIUCgVsaW1pdBgDIAEoA1IFbGltaXQS'
    'JQoOd2luZG93X3NlY29uZHMYBCABKANSDXdpbmRvd1NlY29uZHMSGAoHZW5hYmxlZBgFIAEoCF'
    'IHZW5hYmxlZA==');

@$core.Deprecated('Use createRuleResponseDescriptor instead')
const CreateRuleResponse$json = {
  '1': 'CreateRuleResponse',
  '2': [
    {
      '1': 'rule',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.ratelimit.v1.RateLimitRule',
      '10': 'rule'
    },
  ],
};

/// Descriptor for `CreateRuleResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List createRuleResponseDescriptor = $convert.base64Decode(
    'ChJDcmVhdGVSdWxlUmVzcG9uc2USOwoEcnVsZRgBIAEoCzInLmsxczAuc3lzdGVtLnJhdGVsaW'
    '1pdC52MS5SYXRlTGltaXRSdWxlUgRydWxl');

@$core.Deprecated('Use getRuleRequestDescriptor instead')
const GetRuleRequest$json = {
  '1': 'GetRuleRequest',
  '2': [
    {'1': 'rule_id', '3': 1, '4': 1, '5': 9, '10': 'ruleId'},
  ],
};

/// Descriptor for `GetRuleRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List getRuleRequestDescriptor = $convert
    .base64Decode('Cg5HZXRSdWxlUmVxdWVzdBIXCgdydWxlX2lkGAEgASgJUgZydWxlSWQ=');

@$core.Deprecated('Use getRuleResponseDescriptor instead')
const GetRuleResponse$json = {
  '1': 'GetRuleResponse',
  '2': [
    {
      '1': 'rule',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.ratelimit.v1.RateLimitRule',
      '10': 'rule'
    },
  ],
};

/// Descriptor for `GetRuleResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List getRuleResponseDescriptor = $convert.base64Decode(
    'Cg9HZXRSdWxlUmVzcG9uc2USOwoEcnVsZRgBIAEoCzInLmsxczAuc3lzdGVtLnJhdGVsaW1pdC'
    '52MS5SYXRlTGltaXRSdWxlUgRydWxl');

@$core.Deprecated('Use updateRuleRequestDescriptor instead')
const UpdateRuleRequest$json = {
  '1': 'UpdateRuleRequest',
  '2': [
    {'1': 'rule_id', '3': 1, '4': 1, '5': 9, '10': 'ruleId'},
    {'1': 'scope', '3': 2, '4': 1, '5': 9, '10': 'scope'},
    {
      '1': 'identifier_pattern',
      '3': 3,
      '4': 1,
      '5': 9,
      '10': 'identifierPattern'
    },
    {'1': 'limit', '3': 4, '4': 1, '5': 3, '10': 'limit'},
    {'1': 'window_seconds', '3': 5, '4': 1, '5': 3, '10': 'windowSeconds'},
    {'1': 'enabled', '3': 6, '4': 1, '5': 8, '10': 'enabled'},
  ],
};

/// Descriptor for `UpdateRuleRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List updateRuleRequestDescriptor = $convert.base64Decode(
    'ChFVcGRhdGVSdWxlUmVxdWVzdBIXCgdydWxlX2lkGAEgASgJUgZydWxlSWQSFAoFc2NvcGUYAi'
    'ABKAlSBXNjb3BlEi0KEmlkZW50aWZpZXJfcGF0dGVybhgDIAEoCVIRaWRlbnRpZmllclBhdHRl'
    'cm4SFAoFbGltaXQYBCABKANSBWxpbWl0EiUKDndpbmRvd19zZWNvbmRzGAUgASgDUg13aW5kb3'
    'dTZWNvbmRzEhgKB2VuYWJsZWQYBiABKAhSB2VuYWJsZWQ=');

@$core.Deprecated('Use updateRuleResponseDescriptor instead')
const UpdateRuleResponse$json = {
  '1': 'UpdateRuleResponse',
  '2': [
    {
      '1': 'rule',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.ratelimit.v1.RateLimitRule',
      '10': 'rule'
    },
  ],
};

/// Descriptor for `UpdateRuleResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List updateRuleResponseDescriptor = $convert.base64Decode(
    'ChJVcGRhdGVSdWxlUmVzcG9uc2USOwoEcnVsZRgBIAEoCzInLmsxczAuc3lzdGVtLnJhdGVsaW'
    '1pdC52MS5SYXRlTGltaXRSdWxlUgRydWxl');

@$core.Deprecated('Use deleteRuleRequestDescriptor instead')
const DeleteRuleRequest$json = {
  '1': 'DeleteRuleRequest',
  '2': [
    {'1': 'rule_id', '3': 1, '4': 1, '5': 9, '10': 'ruleId'},
  ],
};

/// Descriptor for `DeleteRuleRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List deleteRuleRequestDescriptor = $convert.base64Decode(
    'ChFEZWxldGVSdWxlUmVxdWVzdBIXCgdydWxlX2lkGAEgASgJUgZydWxlSWQ=');

@$core.Deprecated('Use deleteRuleResponseDescriptor instead')
const DeleteRuleResponse$json = {
  '1': 'DeleteRuleResponse',
  '2': [
    {'1': 'success', '3': 1, '4': 1, '5': 8, '10': 'success'},
  ],
};

/// Descriptor for `DeleteRuleResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List deleteRuleResponseDescriptor =
    $convert.base64Decode(
        'ChJEZWxldGVSdWxlUmVzcG9uc2USGAoHc3VjY2VzcxgBIAEoCFIHc3VjY2Vzcw==');

@$core.Deprecated('Use listRulesRequestDescriptor instead')
const ListRulesRequest$json = {
  '1': 'ListRulesRequest',
  '2': [
    {'1': 'scope', '3': 1, '4': 1, '5': 9, '10': 'scope'},
    {
      '1': 'enabled_only',
      '3': 2,
      '4': 1,
      '5': 8,
      '9': 0,
      '10': 'enabledOnly',
      '17': true
    },
    {
      '1': 'pagination',
      '3': 3,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.common.v1.Pagination',
      '10': 'pagination'
    },
  ],
  '8': [
    {'1': '_enabled_only'},
  ],
  '9': [
    {'1': 4, '2': 5},
  ],
  '10': ['page_size'],
};

/// Descriptor for `ListRulesRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List listRulesRequestDescriptor = $convert.base64Decode(
    'ChBMaXN0UnVsZXNSZXF1ZXN0EhQKBXNjb3BlGAEgASgJUgVzY29wZRImCgxlbmFibGVkX29ubH'
    'kYAiABKAhIAFILZW5hYmxlZE9ubHmIAQESQQoKcGFnaW5hdGlvbhgDIAEoCzIhLmsxczAuc3lz'
    'dGVtLmNvbW1vbi52MS5QYWdpbmF0aW9uUgpwYWdpbmF0aW9uQg8KDV9lbmFibGVkX29ubHlKBA'
    'gEEAVSCXBhZ2Vfc2l6ZQ==');

@$core.Deprecated('Use listRulesResponseDescriptor instead')
const ListRulesResponse$json = {
  '1': 'ListRulesResponse',
  '2': [
    {
      '1': 'rules',
      '3': 1,
      '4': 3,
      '5': 11,
      '6': '.k1s0.system.ratelimit.v1.RateLimitRule',
      '10': 'rules'
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

/// Descriptor for `ListRulesResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List listRulesResponseDescriptor = $convert.base64Decode(
    'ChFMaXN0UnVsZXNSZXNwb25zZRI9CgVydWxlcxgBIAMoCzInLmsxczAuc3lzdGVtLnJhdGVsaW'
    '1pdC52MS5SYXRlTGltaXRSdWxlUgVydWxlcxJHCgpwYWdpbmF0aW9uGAIgASgLMicuazFzMC5z'
    'eXN0ZW0uY29tbW9uLnYxLlBhZ2luYXRpb25SZXN1bHRSCnBhZ2luYXRpb24=');

@$core.Deprecated('Use rateLimitRuleDescriptor instead')
const RateLimitRule$json = {
  '1': 'RateLimitRule',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 9, '10': 'id'},
    {'1': 'scope', '3': 2, '4': 1, '5': 9, '10': 'scope'},
    {
      '1': 'identifier_pattern',
      '3': 3,
      '4': 1,
      '5': 9,
      '10': 'identifierPattern'
    },
    {'1': 'limit', '3': 4, '4': 1, '5': 3, '10': 'limit'},
    {'1': 'window_seconds', '3': 5, '4': 1, '5': 3, '10': 'windowSeconds'},
    {'1': 'algorithm', '3': 6, '4': 1, '5': 9, '10': 'algorithm'},
    {'1': 'enabled', '3': 7, '4': 1, '5': 8, '10': 'enabled'},
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
    {'1': 'name', '3': 10, '4': 1, '5': 9, '10': 'name'},
    {
      '1': 'algorithm_enum',
      '3': 11,
      '4': 1,
      '5': 14,
      '6': '.k1s0.system.ratelimit.v1.RateLimitAlgorithm',
      '10': 'algorithmEnum'
    },
  ],
};

/// Descriptor for `RateLimitRule`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List rateLimitRuleDescriptor = $convert.base64Decode(
    'Cg1SYXRlTGltaXRSdWxlEg4KAmlkGAEgASgJUgJpZBIUCgVzY29wZRgCIAEoCVIFc2NvcGUSLQ'
    'oSaWRlbnRpZmllcl9wYXR0ZXJuGAMgASgJUhFpZGVudGlmaWVyUGF0dGVybhIUCgVsaW1pdBgE'
    'IAEoA1IFbGltaXQSJQoOd2luZG93X3NlY29uZHMYBSABKANSDXdpbmRvd1NlY29uZHMSHAoJYW'
    'xnb3JpdGhtGAYgASgJUglhbGdvcml0aG0SGAoHZW5hYmxlZBgHIAEoCFIHZW5hYmxlZBI/Cgpj'
    'cmVhdGVkX2F0GAggASgLMiAuazFzMC5zeXN0ZW0uY29tbW9uLnYxLlRpbWVzdGFtcFIJY3JlYX'
    'RlZEF0Ej8KCnVwZGF0ZWRfYXQYCSABKAsyIC5rMXMwLnN5c3RlbS5jb21tb24udjEuVGltZXN0'
    'YW1wUgl1cGRhdGVkQXQSEgoEbmFtZRgKIAEoCVIEbmFtZRJTCg5hbGdvcml0aG1fZW51bRgLIA'
    'EoDjIsLmsxczAuc3lzdGVtLnJhdGVsaW1pdC52MS5SYXRlTGltaXRBbGdvcml0aG1SDWFsZ29y'
    'aXRobUVudW0=');

@$core.Deprecated('Use getUsageRequestDescriptor instead')
const GetUsageRequest$json = {
  '1': 'GetUsageRequest',
  '2': [
    {'1': 'rule_id', '3': 1, '4': 1, '5': 9, '10': 'ruleId'},
  ],
};

/// Descriptor for `GetUsageRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List getUsageRequestDescriptor = $convert
    .base64Decode('Cg9HZXRVc2FnZVJlcXVlc3QSFwoHcnVsZV9pZBgBIAEoCVIGcnVsZUlk');

@$core.Deprecated('Use getUsageResponseDescriptor instead')
const GetUsageResponse$json = {
  '1': 'GetUsageResponse',
  '2': [
    {'1': 'rule_id', '3': 1, '4': 1, '5': 9, '10': 'ruleId'},
    {'1': 'rule_name', '3': 2, '4': 1, '5': 9, '10': 'ruleName'},
    {'1': 'limit', '3': 3, '4': 1, '5': 3, '10': 'limit'},
    {'1': 'window_seconds', '3': 4, '4': 1, '5': 3, '10': 'windowSeconds'},
    {'1': 'algorithm', '3': 5, '4': 1, '5': 9, '10': 'algorithm'},
    {'1': 'enabled', '3': 6, '4': 1, '5': 8, '10': 'enabled'},
    {'1': 'used', '3': 7, '4': 1, '5': 3, '9': 0, '10': 'used', '17': true},
    {
      '1': 'remaining',
      '3': 8,
      '4': 1,
      '5': 3,
      '9': 1,
      '10': 'remaining',
      '17': true
    },
    {
      '1': 'reset_at',
      '3': 9,
      '4': 1,
      '5': 3,
      '9': 2,
      '10': 'resetAt',
      '17': true
    },
    {
      '1': 'algorithm_enum',
      '3': 10,
      '4': 1,
      '5': 14,
      '6': '.k1s0.system.ratelimit.v1.RateLimitAlgorithm',
      '10': 'algorithmEnum'
    },
  ],
  '8': [
    {'1': '_used'},
    {'1': '_remaining'},
    {'1': '_reset_at'},
  ],
};

/// Descriptor for `GetUsageResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List getUsageResponseDescriptor = $convert.base64Decode(
    'ChBHZXRVc2FnZVJlc3BvbnNlEhcKB3J1bGVfaWQYASABKAlSBnJ1bGVJZBIbCglydWxlX25hbW'
    'UYAiABKAlSCHJ1bGVOYW1lEhQKBWxpbWl0GAMgASgDUgVsaW1pdBIlCg53aW5kb3dfc2Vjb25k'
    'cxgEIAEoA1INd2luZG93U2Vjb25kcxIcCglhbGdvcml0aG0YBSABKAlSCWFsZ29yaXRobRIYCg'
    'dlbmFibGVkGAYgASgIUgdlbmFibGVkEhcKBHVzZWQYByABKANIAFIEdXNlZIgBARIhCglyZW1h'
    'aW5pbmcYCCABKANIAVIJcmVtYWluaW5niAEBEh4KCHJlc2V0X2F0GAkgASgDSAJSB3Jlc2V0QX'
    'SIAQESUwoOYWxnb3JpdGhtX2VudW0YCiABKA4yLC5rMXMwLnN5c3RlbS5yYXRlbGltaXQudjEu'
    'UmF0ZUxpbWl0QWxnb3JpdGhtUg1hbGdvcml0aG1FbnVtQgcKBV91c2VkQgwKCl9yZW1haW5pbm'
    'dCCwoJX3Jlc2V0X2F0');

@$core.Deprecated('Use resetLimitRequestDescriptor instead')
const ResetLimitRequest$json = {
  '1': 'ResetLimitRequest',
  '2': [
    {'1': 'scope', '3': 1, '4': 1, '5': 9, '10': 'scope'},
    {'1': 'identifier', '3': 2, '4': 1, '5': 9, '10': 'identifier'},
  ],
};

/// Descriptor for `ResetLimitRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List resetLimitRequestDescriptor = $convert.base64Decode(
    'ChFSZXNldExpbWl0UmVxdWVzdBIUCgVzY29wZRgBIAEoCVIFc2NvcGUSHgoKaWRlbnRpZmllch'
    'gCIAEoCVIKaWRlbnRpZmllcg==');

@$core.Deprecated('Use resetLimitResponseDescriptor instead')
const ResetLimitResponse$json = {
  '1': 'ResetLimitResponse',
  '2': [
    {'1': 'success', '3': 1, '4': 1, '5': 8, '10': 'success'},
  ],
};

/// Descriptor for `ResetLimitResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List resetLimitResponseDescriptor =
    $convert.base64Decode(
        'ChJSZXNldExpbWl0UmVzcG9uc2USGAoHc3VjY2VzcxgBIAEoCFIHc3VjY2Vzcw==');
