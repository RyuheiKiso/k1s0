// This is a generated file - do not edit.
//
// Generated from k1s0/system/featureflag/v1/featureflag.proto.

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

@$core.Deprecated('Use evaluateFlagRequestDescriptor instead')
const EvaluateFlagRequest$json = {
  '1': 'EvaluateFlagRequest',
  '2': [
    {'1': 'flag_key', '3': 1, '4': 1, '5': 9, '10': 'flagKey'},
    {
      '1': 'context',
      '3': 2,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.featureflag.v1.EvaluationContext',
      '10': 'context'
    },
  ],
};

/// Descriptor for `EvaluateFlagRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List evaluateFlagRequestDescriptor = $convert.base64Decode(
    'ChNFdmFsdWF0ZUZsYWdSZXF1ZXN0EhkKCGZsYWdfa2V5GAEgASgJUgdmbGFnS2V5EkcKB2Nvbn'
    'RleHQYAiABKAsyLS5rMXMwLnN5c3RlbS5mZWF0dXJlZmxhZy52MS5FdmFsdWF0aW9uQ29udGV4'
    'dFIHY29udGV4dA==');

@$core.Deprecated('Use evaluateFlagResponseDescriptor instead')
const EvaluateFlagResponse$json = {
  '1': 'EvaluateFlagResponse',
  '2': [
    {'1': 'flag_key', '3': 1, '4': 1, '5': 9, '10': 'flagKey'},
    {'1': 'enabled', '3': 2, '4': 1, '5': 8, '10': 'enabled'},
    {
      '1': 'variant',
      '3': 3,
      '4': 1,
      '5': 9,
      '9': 0,
      '10': 'variant',
      '17': true
    },
    {'1': 'reason', '3': 4, '4': 1, '5': 9, '10': 'reason'},
  ],
  '8': [
    {'1': '_variant'},
  ],
};

/// Descriptor for `EvaluateFlagResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List evaluateFlagResponseDescriptor = $convert.base64Decode(
    'ChRFdmFsdWF0ZUZsYWdSZXNwb25zZRIZCghmbGFnX2tleRgBIAEoCVIHZmxhZ0tleRIYCgdlbm'
    'FibGVkGAIgASgIUgdlbmFibGVkEh0KB3ZhcmlhbnQYAyABKAlIAFIHdmFyaWFudIgBARIWCgZy'
    'ZWFzb24YBCABKAlSBnJlYXNvbkIKCghfdmFyaWFudA==');

@$core.Deprecated('Use evaluationContextDescriptor instead')
const EvaluationContext$json = {
  '1': 'EvaluationContext',
  '2': [
    {
      '1': 'user_id',
      '3': 1,
      '4': 1,
      '5': 9,
      '9': 0,
      '10': 'userId',
      '17': true
    },
    {
      '1': 'tenant_id',
      '3': 2,
      '4': 1,
      '5': 9,
      '9': 1,
      '10': 'tenantId',
      '17': true
    },
    {
      '1': 'attributes',
      '3': 3,
      '4': 3,
      '5': 11,
      '6': '.k1s0.system.featureflag.v1.EvaluationContext.AttributesEntry',
      '10': 'attributes'
    },
  ],
  '3': [EvaluationContext_AttributesEntry$json],
  '8': [
    {'1': '_user_id'},
    {'1': '_tenant_id'},
  ],
};

@$core.Deprecated('Use evaluationContextDescriptor instead')
const EvaluationContext_AttributesEntry$json = {
  '1': 'AttributesEntry',
  '2': [
    {'1': 'key', '3': 1, '4': 1, '5': 9, '10': 'key'},
    {'1': 'value', '3': 2, '4': 1, '5': 9, '10': 'value'},
  ],
  '7': {'7': true},
};

/// Descriptor for `EvaluationContext`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List evaluationContextDescriptor = $convert.base64Decode(
    'ChFFdmFsdWF0aW9uQ29udGV4dBIcCgd1c2VyX2lkGAEgASgJSABSBnVzZXJJZIgBARIgCgl0ZW'
    '5hbnRfaWQYAiABKAlIAVIIdGVuYW50SWSIAQESXQoKYXR0cmlidXRlcxgDIAMoCzI9LmsxczAu'
    'c3lzdGVtLmZlYXR1cmVmbGFnLnYxLkV2YWx1YXRpb25Db250ZXh0LkF0dHJpYnV0ZXNFbnRyeV'
    'IKYXR0cmlidXRlcxo9Cg9BdHRyaWJ1dGVzRW50cnkSEAoDa2V5GAEgASgJUgNrZXkSFAoFdmFs'
    'dWUYAiABKAlSBXZhbHVlOgI4AUIKCghfdXNlcl9pZEIMCgpfdGVuYW50X2lk');

@$core.Deprecated('Use getFlagRequestDescriptor instead')
const GetFlagRequest$json = {
  '1': 'GetFlagRequest',
  '2': [
    {'1': 'flag_key', '3': 1, '4': 1, '5': 9, '10': 'flagKey'},
  ],
};

/// Descriptor for `GetFlagRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List getFlagRequestDescriptor = $convert.base64Decode(
    'Cg5HZXRGbGFnUmVxdWVzdBIZCghmbGFnX2tleRgBIAEoCVIHZmxhZ0tleQ==');

@$core.Deprecated('Use getFlagResponseDescriptor instead')
const GetFlagResponse$json = {
  '1': 'GetFlagResponse',
  '2': [
    {
      '1': 'flag',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.featureflag.v1.FeatureFlag',
      '10': 'flag'
    },
  ],
};

/// Descriptor for `GetFlagResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List getFlagResponseDescriptor = $convert.base64Decode(
    'Cg9HZXRGbGFnUmVzcG9uc2USOwoEZmxhZxgBIAEoCzInLmsxczAuc3lzdGVtLmZlYXR1cmVmbG'
    'FnLnYxLkZlYXR1cmVGbGFnUgRmbGFn');

@$core.Deprecated('Use listFlagsRequestDescriptor instead')
const ListFlagsRequest$json = {
  '1': 'ListFlagsRequest',
};

/// Descriptor for `ListFlagsRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List listFlagsRequestDescriptor =
    $convert.base64Decode('ChBMaXN0RmxhZ3NSZXF1ZXN0');

@$core.Deprecated('Use listFlagsResponseDescriptor instead')
const ListFlagsResponse$json = {
  '1': 'ListFlagsResponse',
  '2': [
    {
      '1': 'flags',
      '3': 1,
      '4': 3,
      '5': 11,
      '6': '.k1s0.system.featureflag.v1.FeatureFlag',
      '10': 'flags'
    },
  ],
};

/// Descriptor for `ListFlagsResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List listFlagsResponseDescriptor = $convert.base64Decode(
    'ChFMaXN0RmxhZ3NSZXNwb25zZRI9CgVmbGFncxgBIAMoCzInLmsxczAuc3lzdGVtLmZlYXR1cm'
    'VmbGFnLnYxLkZlYXR1cmVGbGFnUgVmbGFncw==');

@$core.Deprecated('Use createFlagRequestDescriptor instead')
const CreateFlagRequest$json = {
  '1': 'CreateFlagRequest',
  '2': [
    {'1': 'flag_key', '3': 1, '4': 1, '5': 9, '10': 'flagKey'},
    {'1': 'description', '3': 2, '4': 1, '5': 9, '10': 'description'},
    {'1': 'enabled', '3': 3, '4': 1, '5': 8, '10': 'enabled'},
    {
      '1': 'variants',
      '3': 4,
      '4': 3,
      '5': 11,
      '6': '.k1s0.system.featureflag.v1.FlagVariant',
      '10': 'variants'
    },
  ],
};

/// Descriptor for `CreateFlagRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List createFlagRequestDescriptor = $convert.base64Decode(
    'ChFDcmVhdGVGbGFnUmVxdWVzdBIZCghmbGFnX2tleRgBIAEoCVIHZmxhZ0tleRIgCgtkZXNjcm'
    'lwdGlvbhgCIAEoCVILZGVzY3JpcHRpb24SGAoHZW5hYmxlZBgDIAEoCFIHZW5hYmxlZBJDCgh2'
    'YXJpYW50cxgEIAMoCzInLmsxczAuc3lzdGVtLmZlYXR1cmVmbGFnLnYxLkZsYWdWYXJpYW50Ug'
    'h2YXJpYW50cw==');

@$core.Deprecated('Use createFlagResponseDescriptor instead')
const CreateFlagResponse$json = {
  '1': 'CreateFlagResponse',
  '2': [
    {
      '1': 'flag',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.featureflag.v1.FeatureFlag',
      '10': 'flag'
    },
  ],
};

/// Descriptor for `CreateFlagResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List createFlagResponseDescriptor = $convert.base64Decode(
    'ChJDcmVhdGVGbGFnUmVzcG9uc2USOwoEZmxhZxgBIAEoCzInLmsxczAuc3lzdGVtLmZlYXR1cm'
    'VmbGFnLnYxLkZlYXR1cmVGbGFnUgRmbGFn');

@$core.Deprecated('Use updateFlagRequestDescriptor instead')
const UpdateFlagRequest$json = {
  '1': 'UpdateFlagRequest',
  '2': [
    {'1': 'flag_key', '3': 1, '4': 1, '5': 9, '10': 'flagKey'},
    {
      '1': 'enabled',
      '3': 2,
      '4': 1,
      '5': 8,
      '9': 0,
      '10': 'enabled',
      '17': true
    },
    {
      '1': 'description',
      '3': 3,
      '4': 1,
      '5': 9,
      '9': 1,
      '10': 'description',
      '17': true
    },
    {
      '1': 'variants',
      '3': 4,
      '4': 3,
      '5': 11,
      '6': '.k1s0.system.featureflag.v1.FlagVariant',
      '10': 'variants'
    },
    {
      '1': 'rules',
      '3': 5,
      '4': 3,
      '5': 11,
      '6': '.k1s0.system.featureflag.v1.FlagRule',
      '10': 'rules'
    },
  ],
  '8': [
    {'1': '_enabled'},
    {'1': '_description'},
  ],
};

/// Descriptor for `UpdateFlagRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List updateFlagRequestDescriptor = $convert.base64Decode(
    'ChFVcGRhdGVGbGFnUmVxdWVzdBIZCghmbGFnX2tleRgBIAEoCVIHZmxhZ0tleRIdCgdlbmFibG'
    'VkGAIgASgISABSB2VuYWJsZWSIAQESJQoLZGVzY3JpcHRpb24YAyABKAlIAVILZGVzY3JpcHRp'
    'b26IAQESQwoIdmFyaWFudHMYBCADKAsyJy5rMXMwLnN5c3RlbS5mZWF0dXJlZmxhZy52MS5GbG'
    'FnVmFyaWFudFIIdmFyaWFudHMSOgoFcnVsZXMYBSADKAsyJC5rMXMwLnN5c3RlbS5mZWF0dXJl'
    'ZmxhZy52MS5GbGFnUnVsZVIFcnVsZXNCCgoIX2VuYWJsZWRCDgoMX2Rlc2NyaXB0aW9u');

@$core.Deprecated('Use updateFlagResponseDescriptor instead')
const UpdateFlagResponse$json = {
  '1': 'UpdateFlagResponse',
  '2': [
    {
      '1': 'flag',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.featureflag.v1.FeatureFlag',
      '10': 'flag'
    },
  ],
};

/// Descriptor for `UpdateFlagResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List updateFlagResponseDescriptor = $convert.base64Decode(
    'ChJVcGRhdGVGbGFnUmVzcG9uc2USOwoEZmxhZxgBIAEoCzInLmsxczAuc3lzdGVtLmZlYXR1cm'
    'VmbGFnLnYxLkZlYXR1cmVGbGFnUgRmbGFn');

@$core.Deprecated('Use deleteFlagRequestDescriptor instead')
const DeleteFlagRequest$json = {
  '1': 'DeleteFlagRequest',
  '2': [
    {'1': 'flag_key', '3': 1, '4': 1, '5': 9, '10': 'flagKey'},
  ],
};

/// Descriptor for `DeleteFlagRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List deleteFlagRequestDescriptor = $convert.base64Decode(
    'ChFEZWxldGVGbGFnUmVxdWVzdBIZCghmbGFnX2tleRgBIAEoCVIHZmxhZ0tleQ==');

@$core.Deprecated('Use deleteFlagResponseDescriptor instead')
const DeleteFlagResponse$json = {
  '1': 'DeleteFlagResponse',
  '2': [
    {'1': 'success', '3': 1, '4': 1, '5': 8, '10': 'success'},
    {'1': 'message', '3': 2, '4': 1, '5': 9, '10': 'message'},
  ],
};

/// Descriptor for `DeleteFlagResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List deleteFlagResponseDescriptor = $convert.base64Decode(
    'ChJEZWxldGVGbGFnUmVzcG9uc2USGAoHc3VjY2VzcxgBIAEoCFIHc3VjY2VzcxIYCgdtZXNzYW'
    'dlGAIgASgJUgdtZXNzYWdl');

@$core.Deprecated('Use featureFlagDescriptor instead')
const FeatureFlag$json = {
  '1': 'FeatureFlag',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 9, '10': 'id'},
    {'1': 'flag_key', '3': 2, '4': 1, '5': 9, '10': 'flagKey'},
    {'1': 'description', '3': 3, '4': 1, '5': 9, '10': 'description'},
    {'1': 'enabled', '3': 4, '4': 1, '5': 8, '10': 'enabled'},
    {
      '1': 'variants',
      '3': 5,
      '4': 3,
      '5': 11,
      '6': '.k1s0.system.featureflag.v1.FlagVariant',
      '10': 'variants'
    },
    {
      '1': 'created_at',
      '3': 6,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.common.v1.Timestamp',
      '10': 'createdAt'
    },
    {
      '1': 'updated_at',
      '3': 7,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.common.v1.Timestamp',
      '10': 'updatedAt'
    },
    {
      '1': 'rules',
      '3': 8,
      '4': 3,
      '5': 11,
      '6': '.k1s0.system.featureflag.v1.FlagRule',
      '10': 'rules'
    },
  ],
};

/// Descriptor for `FeatureFlag`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List featureFlagDescriptor = $convert.base64Decode(
    'CgtGZWF0dXJlRmxhZxIOCgJpZBgBIAEoCVICaWQSGQoIZmxhZ19rZXkYAiABKAlSB2ZsYWdLZX'
    'kSIAoLZGVzY3JpcHRpb24YAyABKAlSC2Rlc2NyaXB0aW9uEhgKB2VuYWJsZWQYBCABKAhSB2Vu'
    'YWJsZWQSQwoIdmFyaWFudHMYBSADKAsyJy5rMXMwLnN5c3RlbS5mZWF0dXJlZmxhZy52MS5GbG'
    'FnVmFyaWFudFIIdmFyaWFudHMSPwoKY3JlYXRlZF9hdBgGIAEoCzIgLmsxczAuc3lzdGVtLmNv'
    'bW1vbi52MS5UaW1lc3RhbXBSCWNyZWF0ZWRBdBI/Cgp1cGRhdGVkX2F0GAcgASgLMiAuazFzMC'
    '5zeXN0ZW0uY29tbW9uLnYxLlRpbWVzdGFtcFIJdXBkYXRlZEF0EjoKBXJ1bGVzGAggAygLMiQu'
    'azFzMC5zeXN0ZW0uZmVhdHVyZWZsYWcudjEuRmxhZ1J1bGVSBXJ1bGVz');

@$core.Deprecated('Use flagVariantDescriptor instead')
const FlagVariant$json = {
  '1': 'FlagVariant',
  '2': [
    {'1': 'name', '3': 1, '4': 1, '5': 9, '10': 'name'},
    {'1': 'value', '3': 2, '4': 1, '5': 9, '10': 'value'},
    {'1': 'weight', '3': 3, '4': 1, '5': 5, '10': 'weight'},
  ],
};

/// Descriptor for `FlagVariant`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List flagVariantDescriptor = $convert.base64Decode(
    'CgtGbGFnVmFyaWFudBISCgRuYW1lGAEgASgJUgRuYW1lEhQKBXZhbHVlGAIgASgJUgV2YWx1ZR'
    'IWCgZ3ZWlnaHQYAyABKAVSBndlaWdodA==');

@$core.Deprecated('Use flagRuleDescriptor instead')
const FlagRule$json = {
  '1': 'FlagRule',
  '2': [
    {'1': 'attribute', '3': 1, '4': 1, '5': 9, '10': 'attribute'},
    {'1': 'operator', '3': 2, '4': 1, '5': 9, '10': 'operator'},
    {'1': 'value', '3': 3, '4': 1, '5': 9, '10': 'value'},
    {'1': 'variant', '3': 4, '4': 1, '5': 9, '10': 'variant'},
  ],
};

/// Descriptor for `FlagRule`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List flagRuleDescriptor = $convert.base64Decode(
    'CghGbGFnUnVsZRIcCglhdHRyaWJ1dGUYASABKAlSCWF0dHJpYnV0ZRIaCghvcGVyYXRvchgCIA'
    'EoCVIIb3BlcmF0b3ISFAoFdmFsdWUYAyABKAlSBXZhbHVlEhgKB3ZhcmlhbnQYBCABKAlSB3Zh'
    'cmlhbnQ=');

@$core.Deprecated('Use watchFeatureFlagRequestDescriptor instead')
const WatchFeatureFlagRequest$json = {
  '1': 'WatchFeatureFlagRequest',
  '2': [
    {'1': 'flag_key', '3': 1, '4': 1, '5': 9, '10': 'flagKey'},
  ],
};

/// Descriptor for `WatchFeatureFlagRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List watchFeatureFlagRequestDescriptor =
    $convert.base64Decode(
        'ChdXYXRjaEZlYXR1cmVGbGFnUmVxdWVzdBIZCghmbGFnX2tleRgBIAEoCVIHZmxhZ0tleQ==');

@$core.Deprecated('Use watchFeatureFlagResponseDescriptor instead')
const WatchFeatureFlagResponse$json = {
  '1': 'WatchFeatureFlagResponse',
  '2': [
    {'1': 'flag_key', '3': 1, '4': 1, '5': 9, '10': 'flagKey'},
    {'1': 'change_type', '3': 2, '4': 1, '5': 9, '10': 'changeType'},
    {
      '1': 'flag',
      '3': 3,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.featureflag.v1.FeatureFlag',
      '10': 'flag'
    },
    {
      '1': 'changed_at',
      '3': 4,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.common.v1.Timestamp',
      '10': 'changedAt'
    },
    {
      '1': 'change_type_enum',
      '3': 5,
      '4': 1,
      '5': 14,
      '6': '.k1s0.system.common.v1.ChangeType',
      '10': 'changeTypeEnum'
    },
  ],
};

/// Descriptor for `WatchFeatureFlagResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List watchFeatureFlagResponseDescriptor = $convert.base64Decode(
    'ChhXYXRjaEZlYXR1cmVGbGFnUmVzcG9uc2USGQoIZmxhZ19rZXkYASABKAlSB2ZsYWdLZXkSHw'
    'oLY2hhbmdlX3R5cGUYAiABKAlSCmNoYW5nZVR5cGUSOwoEZmxhZxgDIAEoCzInLmsxczAuc3lz'
    'dGVtLmZlYXR1cmVmbGFnLnYxLkZlYXR1cmVGbGFnUgRmbGFnEj8KCmNoYW5nZWRfYXQYBCABKA'
    'syIC5rMXMwLnN5c3RlbS5jb21tb24udjEuVGltZXN0YW1wUgljaGFuZ2VkQXQSSwoQY2hhbmdl'
    'X3R5cGVfZW51bRgFIAEoDjIhLmsxczAuc3lzdGVtLmNvbW1vbi52MS5DaGFuZ2VUeXBlUg5jaG'
    'FuZ2VUeXBlRW51bQ==');
