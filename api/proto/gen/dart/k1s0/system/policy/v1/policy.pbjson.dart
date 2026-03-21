// This is a generated file - do not edit.
//
// Generated from k1s0/system/policy/v1/policy.proto.

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

@$core.Deprecated('Use evaluatePolicyRequestDescriptor instead')
const EvaluatePolicyRequest$json = {
  '1': 'EvaluatePolicyRequest',
  '2': [
    {'1': 'policy_id', '3': 1, '4': 1, '5': 9, '10': 'policyId'},
    {'1': 'input_json', '3': 2, '4': 1, '5': 12, '10': 'inputJson'},
  ],
};

/// Descriptor for `EvaluatePolicyRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List evaluatePolicyRequestDescriptor = $convert.base64Decode(
    'ChVFdmFsdWF0ZVBvbGljeVJlcXVlc3QSGwoJcG9saWN5X2lkGAEgASgJUghwb2xpY3lJZBIdCg'
    'ppbnB1dF9qc29uGAIgASgMUglpbnB1dEpzb24=');

@$core.Deprecated('Use evaluatePolicyResponseDescriptor instead')
const EvaluatePolicyResponse$json = {
  '1': 'EvaluatePolicyResponse',
  '2': [
    {'1': 'allowed', '3': 1, '4': 1, '5': 8, '10': 'allowed'},
    {'1': 'package_path', '3': 2, '4': 1, '5': 9, '10': 'packagePath'},
    {'1': 'decision_id', '3': 3, '4': 1, '5': 9, '10': 'decisionId'},
    {'1': 'cached', '3': 4, '4': 1, '5': 8, '10': 'cached'},
  ],
};

/// Descriptor for `EvaluatePolicyResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List evaluatePolicyResponseDescriptor = $convert.base64Decode(
    'ChZFdmFsdWF0ZVBvbGljeVJlc3BvbnNlEhgKB2FsbG93ZWQYASABKAhSB2FsbG93ZWQSIQoMcG'
    'Fja2FnZV9wYXRoGAIgASgJUgtwYWNrYWdlUGF0aBIfCgtkZWNpc2lvbl9pZBgDIAEoCVIKZGVj'
    'aXNpb25JZBIWCgZjYWNoZWQYBCABKAhSBmNhY2hlZA==');

@$core.Deprecated('Use getPolicyRequestDescriptor instead')
const GetPolicyRequest$json = {
  '1': 'GetPolicyRequest',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 9, '10': 'id'},
  ],
};

/// Descriptor for `GetPolicyRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List getPolicyRequestDescriptor =
    $convert.base64Decode('ChBHZXRQb2xpY3lSZXF1ZXN0Eg4KAmlkGAEgASgJUgJpZA==');

@$core.Deprecated('Use getPolicyResponseDescriptor instead')
const GetPolicyResponse$json = {
  '1': 'GetPolicyResponse',
  '2': [
    {
      '1': 'policy',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.policy.v1.Policy',
      '10': 'policy'
    },
  ],
};

/// Descriptor for `GetPolicyResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List getPolicyResponseDescriptor = $convert.base64Decode(
    'ChFHZXRQb2xpY3lSZXNwb25zZRI1CgZwb2xpY3kYASABKAsyHS5rMXMwLnN5c3RlbS5wb2xpY3'
    'kudjEuUG9saWN5UgZwb2xpY3k=');

@$core.Deprecated('Use listPoliciesRequestDescriptor instead')
const ListPoliciesRequest$json = {
  '1': 'ListPoliciesRequest',
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
      '1': 'bundle_id',
      '3': 2,
      '4': 1,
      '5': 9,
      '9': 0,
      '10': 'bundleId',
      '17': true
    },
    {'1': 'enabled_only', '3': 3, '4': 1, '5': 8, '10': 'enabledOnly'},
  ],
  '8': [
    {'1': '_bundle_id'},
  ],
};

/// Descriptor for `ListPoliciesRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List listPoliciesRequestDescriptor = $convert.base64Decode(
    'ChNMaXN0UG9saWNpZXNSZXF1ZXN0EkEKCnBhZ2luYXRpb24YASABKAsyIS5rMXMwLnN5c3RlbS'
    '5jb21tb24udjEuUGFnaW5hdGlvblIKcGFnaW5hdGlvbhIgCglidW5kbGVfaWQYAiABKAlIAFII'
    'YnVuZGxlSWSIAQESIQoMZW5hYmxlZF9vbmx5GAMgASgIUgtlbmFibGVkT25seUIMCgpfYnVuZG'
    'xlX2lk');

@$core.Deprecated('Use listPoliciesResponseDescriptor instead')
const ListPoliciesResponse$json = {
  '1': 'ListPoliciesResponse',
  '2': [
    {
      '1': 'policies',
      '3': 1,
      '4': 3,
      '5': 11,
      '6': '.k1s0.system.policy.v1.Policy',
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

/// Descriptor for `ListPoliciesResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List listPoliciesResponseDescriptor = $convert.base64Decode(
    'ChRMaXN0UG9saWNpZXNSZXNwb25zZRI5Cghwb2xpY2llcxgBIAMoCzIdLmsxczAuc3lzdGVtLn'
    'BvbGljeS52MS5Qb2xpY3lSCHBvbGljaWVzEkcKCnBhZ2luYXRpb24YAiABKAsyJy5rMXMwLnN5'
    'c3RlbS5jb21tb24udjEuUGFnaW5hdGlvblJlc3VsdFIKcGFnaW5hdGlvbg==');

@$core.Deprecated('Use createPolicyRequestDescriptor instead')
const CreatePolicyRequest$json = {
  '1': 'CreatePolicyRequest',
  '2': [
    {'1': 'name', '3': 1, '4': 1, '5': 9, '10': 'name'},
    {'1': 'description', '3': 2, '4': 1, '5': 9, '10': 'description'},
    {'1': 'rego_content', '3': 3, '4': 1, '5': 9, '10': 'regoContent'},
    {'1': 'package_path', '3': 4, '4': 1, '5': 9, '10': 'packagePath'},
    {
      '1': 'bundle_id',
      '3': 5,
      '4': 1,
      '5': 9,
      '9': 0,
      '10': 'bundleId',
      '17': true
    },
  ],
  '8': [
    {'1': '_bundle_id'},
  ],
};

/// Descriptor for `CreatePolicyRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List createPolicyRequestDescriptor = $convert.base64Decode(
    'ChNDcmVhdGVQb2xpY3lSZXF1ZXN0EhIKBG5hbWUYASABKAlSBG5hbWUSIAoLZGVzY3JpcHRpb2'
    '4YAiABKAlSC2Rlc2NyaXB0aW9uEiEKDHJlZ29fY29udGVudBgDIAEoCVILcmVnb0NvbnRlbnQS'
    'IQoMcGFja2FnZV9wYXRoGAQgASgJUgtwYWNrYWdlUGF0aBIgCglidW5kbGVfaWQYBSABKAlIAF'
    'IIYnVuZGxlSWSIAQFCDAoKX2J1bmRsZV9pZA==');

@$core.Deprecated('Use createPolicyResponseDescriptor instead')
const CreatePolicyResponse$json = {
  '1': 'CreatePolicyResponse',
  '2': [
    {
      '1': 'policy',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.policy.v1.Policy',
      '10': 'policy'
    },
  ],
};

/// Descriptor for `CreatePolicyResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List createPolicyResponseDescriptor = $convert.base64Decode(
    'ChRDcmVhdGVQb2xpY3lSZXNwb25zZRI1CgZwb2xpY3kYASABKAsyHS5rMXMwLnN5c3RlbS5wb2'
    'xpY3kudjEuUG9saWN5UgZwb2xpY3k=');

@$core.Deprecated('Use updatePolicyRequestDescriptor instead')
const UpdatePolicyRequest$json = {
  '1': 'UpdatePolicyRequest',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 9, '10': 'id'},
    {
      '1': 'description',
      '3': 2,
      '4': 1,
      '5': 9,
      '9': 0,
      '10': 'description',
      '17': true
    },
    {
      '1': 'rego_content',
      '3': 3,
      '4': 1,
      '5': 9,
      '9': 1,
      '10': 'regoContent',
      '17': true
    },
    {
      '1': 'enabled',
      '3': 4,
      '4': 1,
      '5': 8,
      '9': 2,
      '10': 'enabled',
      '17': true
    },
  ],
  '8': [
    {'1': '_description'},
    {'1': '_rego_content'},
    {'1': '_enabled'},
  ],
};

/// Descriptor for `UpdatePolicyRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List updatePolicyRequestDescriptor = $convert.base64Decode(
    'ChNVcGRhdGVQb2xpY3lSZXF1ZXN0Eg4KAmlkGAEgASgJUgJpZBIlCgtkZXNjcmlwdGlvbhgCIA'
    'EoCUgAUgtkZXNjcmlwdGlvbogBARImCgxyZWdvX2NvbnRlbnQYAyABKAlIAVILcmVnb0NvbnRl'
    'bnSIAQESHQoHZW5hYmxlZBgEIAEoCEgCUgdlbmFibGVkiAEBQg4KDF9kZXNjcmlwdGlvbkIPCg'
    '1fcmVnb19jb250ZW50QgoKCF9lbmFibGVk');

@$core.Deprecated('Use updatePolicyResponseDescriptor instead')
const UpdatePolicyResponse$json = {
  '1': 'UpdatePolicyResponse',
  '2': [
    {
      '1': 'policy',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.policy.v1.Policy',
      '10': 'policy'
    },
  ],
};

/// Descriptor for `UpdatePolicyResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List updatePolicyResponseDescriptor = $convert.base64Decode(
    'ChRVcGRhdGVQb2xpY3lSZXNwb25zZRI1CgZwb2xpY3kYASABKAsyHS5rMXMwLnN5c3RlbS5wb2'
    'xpY3kudjEuUG9saWN5UgZwb2xpY3k=');

@$core.Deprecated('Use deletePolicyRequestDescriptor instead')
const DeletePolicyRequest$json = {
  '1': 'DeletePolicyRequest',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 9, '10': 'id'},
  ],
};

/// Descriptor for `DeletePolicyRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List deletePolicyRequestDescriptor = $convert
    .base64Decode('ChNEZWxldGVQb2xpY3lSZXF1ZXN0Eg4KAmlkGAEgASgJUgJpZA==');

@$core.Deprecated('Use deletePolicyResponseDescriptor instead')
const DeletePolicyResponse$json = {
  '1': 'DeletePolicyResponse',
  '2': [
    {'1': 'success', '3': 1, '4': 1, '5': 8, '10': 'success'},
    {'1': 'message', '3': 2, '4': 1, '5': 9, '10': 'message'},
  ],
};

/// Descriptor for `DeletePolicyResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List deletePolicyResponseDescriptor = $convert.base64Decode(
    'ChREZWxldGVQb2xpY3lSZXNwb25zZRIYCgdzdWNjZXNzGAEgASgIUgdzdWNjZXNzEhgKB21lc3'
    'NhZ2UYAiABKAlSB21lc3NhZ2U=');

@$core.Deprecated('Use createBundleRequestDescriptor instead')
const CreateBundleRequest$json = {
  '1': 'CreateBundleRequest',
  '2': [
    {'1': 'name', '3': 1, '4': 1, '5': 9, '10': 'name'},
    {'1': 'policy_ids', '3': 2, '4': 3, '5': 9, '10': 'policyIds'},
    {
      '1': 'description',
      '3': 3,
      '4': 1,
      '5': 9,
      '9': 0,
      '10': 'description',
      '17': true
    },
    {
      '1': 'enabled',
      '3': 4,
      '4': 1,
      '5': 8,
      '9': 1,
      '10': 'enabled',
      '17': true
    },
  ],
  '8': [
    {'1': '_description'},
    {'1': '_enabled'},
  ],
};

/// Descriptor for `CreateBundleRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List createBundleRequestDescriptor = $convert.base64Decode(
    'ChNDcmVhdGVCdW5kbGVSZXF1ZXN0EhIKBG5hbWUYASABKAlSBG5hbWUSHQoKcG9saWN5X2lkcx'
    'gCIAMoCVIJcG9saWN5SWRzEiUKC2Rlc2NyaXB0aW9uGAMgASgJSABSC2Rlc2NyaXB0aW9uiAEB'
    'Eh0KB2VuYWJsZWQYBCABKAhIAVIHZW5hYmxlZIgBAUIOCgxfZGVzY3JpcHRpb25CCgoIX2VuYW'
    'JsZWQ=');

@$core.Deprecated('Use createBundleResponseDescriptor instead')
const CreateBundleResponse$json = {
  '1': 'CreateBundleResponse',
  '2': [
    {
      '1': 'bundle',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.policy.v1.PolicyBundle',
      '10': 'bundle'
    },
  ],
};

/// Descriptor for `CreateBundleResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List createBundleResponseDescriptor = $convert.base64Decode(
    'ChRDcmVhdGVCdW5kbGVSZXNwb25zZRI7CgZidW5kbGUYASABKAsyIy5rMXMwLnN5c3RlbS5wb2'
    'xpY3kudjEuUG9saWN5QnVuZGxlUgZidW5kbGU=');

@$core.Deprecated('Use listBundlesRequestDescriptor instead')
const ListBundlesRequest$json = {
  '1': 'ListBundlesRequest',
};

/// Descriptor for `ListBundlesRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List listBundlesRequestDescriptor =
    $convert.base64Decode('ChJMaXN0QnVuZGxlc1JlcXVlc3Q=');

@$core.Deprecated('Use listBundlesResponseDescriptor instead')
const ListBundlesResponse$json = {
  '1': 'ListBundlesResponse',
  '2': [
    {
      '1': 'bundles',
      '3': 1,
      '4': 3,
      '5': 11,
      '6': '.k1s0.system.policy.v1.PolicyBundle',
      '10': 'bundles'
    },
  ],
};

/// Descriptor for `ListBundlesResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List listBundlesResponseDescriptor = $convert.base64Decode(
    'ChNMaXN0QnVuZGxlc1Jlc3BvbnNlEj0KB2J1bmRsZXMYASADKAsyIy5rMXMwLnN5c3RlbS5wb2'
    'xpY3kudjEuUG9saWN5QnVuZGxlUgdidW5kbGVz');

@$core.Deprecated('Use getBundleRequestDescriptor instead')
const GetBundleRequest$json = {
  '1': 'GetBundleRequest',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 9, '10': 'id'},
  ],
};

/// Descriptor for `GetBundleRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List getBundleRequestDescriptor =
    $convert.base64Decode('ChBHZXRCdW5kbGVSZXF1ZXN0Eg4KAmlkGAEgASgJUgJpZA==');

@$core.Deprecated('Use getBundleResponseDescriptor instead')
const GetBundleResponse$json = {
  '1': 'GetBundleResponse',
  '2': [
    {
      '1': 'bundle',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.policy.v1.PolicyBundle',
      '10': 'bundle'
    },
  ],
};

/// Descriptor for `GetBundleResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List getBundleResponseDescriptor = $convert.base64Decode(
    'ChFHZXRCdW5kbGVSZXNwb25zZRI7CgZidW5kbGUYASABKAsyIy5rMXMwLnN5c3RlbS5wb2xpY3'
    'kudjEuUG9saWN5QnVuZGxlUgZidW5kbGU=');

@$core.Deprecated('Use policyDescriptor instead')
const Policy$json = {
  '1': 'Policy',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 9, '10': 'id'},
    {'1': 'name', '3': 2, '4': 1, '5': 9, '10': 'name'},
    {'1': 'description', '3': 3, '4': 1, '5': 9, '10': 'description'},
    {'1': 'package_path', '3': 4, '4': 1, '5': 9, '10': 'packagePath'},
    {'1': 'rego_content', '3': 5, '4': 1, '5': 9, '10': 'regoContent'},
    {
      '1': 'bundle_id',
      '3': 6,
      '4': 1,
      '5': 9,
      '9': 0,
      '10': 'bundleId',
      '17': true
    },
    {'1': 'enabled', '3': 7, '4': 1, '5': 8, '10': 'enabled'},
    {'1': 'version', '3': 8, '4': 1, '5': 13, '10': 'version'},
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
    {'1': '_bundle_id'},
  ],
};

/// Descriptor for `Policy`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List policyDescriptor = $convert.base64Decode(
    'CgZQb2xpY3kSDgoCaWQYASABKAlSAmlkEhIKBG5hbWUYAiABKAlSBG5hbWUSIAoLZGVzY3JpcH'
    'Rpb24YAyABKAlSC2Rlc2NyaXB0aW9uEiEKDHBhY2thZ2VfcGF0aBgEIAEoCVILcGFja2FnZVBh'
    'dGgSIQoMcmVnb19jb250ZW50GAUgASgJUgtyZWdvQ29udGVudBIgCglidW5kbGVfaWQYBiABKA'
    'lIAFIIYnVuZGxlSWSIAQESGAoHZW5hYmxlZBgHIAEoCFIHZW5hYmxlZBIYCgd2ZXJzaW9uGAgg'
    'ASgNUgd2ZXJzaW9uEj8KCmNyZWF0ZWRfYXQYCSABKAsyIC5rMXMwLnN5c3RlbS5jb21tb24udj'
    'EuVGltZXN0YW1wUgljcmVhdGVkQXQSPwoKdXBkYXRlZF9hdBgKIAEoCzIgLmsxczAuc3lzdGVt'
    'LmNvbW1vbi52MS5UaW1lc3RhbXBSCXVwZGF0ZWRBdEIMCgpfYnVuZGxlX2lk');

@$core.Deprecated('Use policyBundleDescriptor instead')
const PolicyBundle$json = {
  '1': 'PolicyBundle',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 9, '10': 'id'},
    {'1': 'name', '3': 2, '4': 1, '5': 9, '10': 'name'},
    {'1': 'policy_ids', '3': 3, '4': 3, '5': 9, '10': 'policyIds'},
    {
      '1': 'created_at',
      '3': 4,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.common.v1.Timestamp',
      '10': 'createdAt'
    },
    {
      '1': 'updated_at',
      '3': 5,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.common.v1.Timestamp',
      '10': 'updatedAt'
    },
    {'1': 'description', '3': 6, '4': 1, '5': 9, '10': 'description'},
    {'1': 'enabled', '3': 7, '4': 1, '5': 8, '10': 'enabled'},
  ],
};

/// Descriptor for `PolicyBundle`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List policyBundleDescriptor = $convert.base64Decode(
    'CgxQb2xpY3lCdW5kbGUSDgoCaWQYASABKAlSAmlkEhIKBG5hbWUYAiABKAlSBG5hbWUSHQoKcG'
    '9saWN5X2lkcxgDIAMoCVIJcG9saWN5SWRzEj8KCmNyZWF0ZWRfYXQYBCABKAsyIC5rMXMwLnN5'
    'c3RlbS5jb21tb24udjEuVGltZXN0YW1wUgljcmVhdGVkQXQSPwoKdXBkYXRlZF9hdBgFIAEoCz'
    'IgLmsxczAuc3lzdGVtLmNvbW1vbi52MS5UaW1lc3RhbXBSCXVwZGF0ZWRBdBIgCgtkZXNjcmlw'
    'dGlvbhgGIAEoCVILZGVzY3JpcHRpb24SGAoHZW5hYmxlZBgHIAEoCFIHZW5hYmxlZA==');
