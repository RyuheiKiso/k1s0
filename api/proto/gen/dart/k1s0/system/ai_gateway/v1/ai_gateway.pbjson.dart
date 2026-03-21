// This is a generated file - do not edit.
//
// Generated from k1s0/system/ai_gateway/v1/ai_gateway.proto.

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

@$core.Deprecated('Use completeRequestDescriptor instead')
const CompleteRequest$json = {
  '1': 'CompleteRequest',
  '2': [
    {'1': 'model', '3': 1, '4': 1, '5': 9, '10': 'model'},
    {
      '1': 'messages',
      '3': 2,
      '4': 3,
      '5': 11,
      '6': '.k1s0.system.aigateway.v1.Message',
      '10': 'messages'
    },
    {'1': 'max_tokens', '3': 3, '4': 1, '5': 5, '10': 'maxTokens'},
    {'1': 'temperature', '3': 4, '4': 1, '5': 2, '10': 'temperature'},
    {'1': 'stream', '3': 5, '4': 1, '5': 8, '10': 'stream'},
    {'1': 'tenant_id', '3': 6, '4': 1, '5': 9, '10': 'tenantId'},
  ],
};

/// Descriptor for `CompleteRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List completeRequestDescriptor = $convert.base64Decode(
    'Cg9Db21wbGV0ZVJlcXVlc3QSFAoFbW9kZWwYASABKAlSBW1vZGVsEj0KCG1lc3NhZ2VzGAIgAy'
    'gLMiEuazFzMC5zeXN0ZW0uYWlnYXRld2F5LnYxLk1lc3NhZ2VSCG1lc3NhZ2VzEh0KCm1heF90'
    'b2tlbnMYAyABKAVSCW1heFRva2VucxIgCgt0ZW1wZXJhdHVyZRgEIAEoAlILdGVtcGVyYXR1cm'
    'USFgoGc3RyZWFtGAUgASgIUgZzdHJlYW0SGwoJdGVuYW50X2lkGAYgASgJUgh0ZW5hbnRJZA==');

@$core.Deprecated('Use messageDescriptor instead')
const Message$json = {
  '1': 'Message',
  '2': [
    {'1': 'role', '3': 1, '4': 1, '5': 9, '10': 'role'},
    {'1': 'content', '3': 2, '4': 1, '5': 9, '10': 'content'},
  ],
};

/// Descriptor for `Message`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List messageDescriptor = $convert.base64Decode(
    'CgdNZXNzYWdlEhIKBHJvbGUYASABKAlSBHJvbGUSGAoHY29udGVudBgCIAEoCVIHY29udGVudA'
    '==');

@$core.Deprecated('Use completeResponseDescriptor instead')
const CompleteResponse$json = {
  '1': 'CompleteResponse',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 9, '10': 'id'},
    {'1': 'model', '3': 2, '4': 1, '5': 9, '10': 'model'},
    {'1': 'content', '3': 3, '4': 1, '5': 9, '10': 'content'},
    {'1': 'prompt_tokens', '3': 4, '4': 1, '5': 5, '10': 'promptTokens'},
    {
      '1': 'completion_tokens',
      '3': 5,
      '4': 1,
      '5': 5,
      '10': 'completionTokens'
    },
  ],
};

/// Descriptor for `CompleteResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List completeResponseDescriptor = $convert.base64Decode(
    'ChBDb21wbGV0ZVJlc3BvbnNlEg4KAmlkGAEgASgJUgJpZBIUCgVtb2RlbBgCIAEoCVIFbW9kZW'
    'wSGAoHY29udGVudBgDIAEoCVIHY29udGVudBIjCg1wcm9tcHRfdG9rZW5zGAQgASgFUgxwcm9t'
    'cHRUb2tlbnMSKwoRY29tcGxldGlvbl90b2tlbnMYBSABKAVSEGNvbXBsZXRpb25Ub2tlbnM=');

@$core.Deprecated('Use completeStreamRequestDescriptor instead')
const CompleteStreamRequest$json = {
  '1': 'CompleteStreamRequest',
  '2': [
    {'1': 'model', '3': 1, '4': 1, '5': 9, '10': 'model'},
    {
      '1': 'messages',
      '3': 2,
      '4': 3,
      '5': 11,
      '6': '.k1s0.system.aigateway.v1.Message',
      '10': 'messages'
    },
    {'1': 'max_tokens', '3': 3, '4': 1, '5': 5, '10': 'maxTokens'},
    {'1': 'temperature', '3': 4, '4': 1, '5': 2, '10': 'temperature'},
    {'1': 'tenant_id', '3': 5, '4': 1, '5': 9, '10': 'tenantId'},
  ],
};

/// Descriptor for `CompleteStreamRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List completeStreamRequestDescriptor = $convert.base64Decode(
    'ChVDb21wbGV0ZVN0cmVhbVJlcXVlc3QSFAoFbW9kZWwYASABKAlSBW1vZGVsEj0KCG1lc3NhZ2'
    'VzGAIgAygLMiEuazFzMC5zeXN0ZW0uYWlnYXRld2F5LnYxLk1lc3NhZ2VSCG1lc3NhZ2VzEh0K'
    'Cm1heF90b2tlbnMYAyABKAVSCW1heFRva2VucxIgCgt0ZW1wZXJhdHVyZRgEIAEoAlILdGVtcG'
    'VyYXR1cmUSGwoJdGVuYW50X2lkGAUgASgJUgh0ZW5hbnRJZA==');

@$core.Deprecated('Use completeStreamResponseDescriptor instead')
const CompleteStreamResponse$json = {
  '1': 'CompleteStreamResponse',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 9, '10': 'id'},
    {'1': 'delta', '3': 2, '4': 1, '5': 9, '10': 'delta'},
    {'1': 'finished', '3': 3, '4': 1, '5': 8, '10': 'finished'},
  ],
};

/// Descriptor for `CompleteStreamResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List completeStreamResponseDescriptor =
    $convert.base64Decode(
        'ChZDb21wbGV0ZVN0cmVhbVJlc3BvbnNlEg4KAmlkGAEgASgJUgJpZBIUCgVkZWx0YRgCIAEoCV'
        'IFZGVsdGESGgoIZmluaXNoZWQYAyABKAhSCGZpbmlzaGVk');

@$core.Deprecated('Use embedRequestDescriptor instead')
const EmbedRequest$json = {
  '1': 'EmbedRequest',
  '2': [
    {'1': 'model', '3': 1, '4': 1, '5': 9, '10': 'model'},
    {'1': 'inputs', '3': 2, '4': 3, '5': 9, '10': 'inputs'},
    {'1': 'tenant_id', '3': 3, '4': 1, '5': 9, '10': 'tenantId'},
  ],
};

/// Descriptor for `EmbedRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List embedRequestDescriptor = $convert.base64Decode(
    'CgxFbWJlZFJlcXVlc3QSFAoFbW9kZWwYASABKAlSBW1vZGVsEhYKBmlucHV0cxgCIAMoCVIGaW'
    '5wdXRzEhsKCXRlbmFudF9pZBgDIAEoCVIIdGVuYW50SWQ=');

@$core.Deprecated('Use embedResponseDescriptor instead')
const EmbedResponse$json = {
  '1': 'EmbedResponse',
  '2': [
    {'1': 'model', '3': 1, '4': 1, '5': 9, '10': 'model'},
    {
      '1': 'embeddings',
      '3': 2,
      '4': 3,
      '5': 11,
      '6': '.k1s0.system.aigateway.v1.Embedding',
      '10': 'embeddings'
    },
  ],
};

/// Descriptor for `EmbedResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List embedResponseDescriptor = $convert.base64Decode(
    'Cg1FbWJlZFJlc3BvbnNlEhQKBW1vZGVsGAEgASgJUgVtb2RlbBJDCgplbWJlZGRpbmdzGAIgAy'
    'gLMiMuazFzMC5zeXN0ZW0uYWlnYXRld2F5LnYxLkVtYmVkZGluZ1IKZW1iZWRkaW5ncw==');

@$core.Deprecated('Use embeddingDescriptor instead')
const Embedding$json = {
  '1': 'Embedding',
  '2': [
    {'1': 'index', '3': 1, '4': 1, '5': 5, '10': 'index'},
    {'1': 'values', '3': 2, '4': 3, '5': 2, '10': 'values'},
  ],
};

/// Descriptor for `Embedding`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List embeddingDescriptor = $convert.base64Decode(
    'CglFbWJlZGRpbmcSFAoFaW5kZXgYASABKAVSBWluZGV4EhYKBnZhbHVlcxgCIAMoAlIGdmFsdW'
    'Vz');

@$core.Deprecated('Use aiModelDescriptor instead')
const AiModel$json = {
  '1': 'AiModel',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 9, '10': 'id'},
    {'1': 'name', '3': 2, '4': 1, '5': 9, '10': 'name'},
    {'1': 'provider', '3': 3, '4': 1, '5': 9, '10': 'provider'},
    {'1': 'context_window', '3': 4, '4': 1, '5': 5, '10': 'contextWindow'},
    {'1': 'enabled', '3': 5, '4': 1, '5': 8, '10': 'enabled'},
  ],
};

/// Descriptor for `AiModel`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List aiModelDescriptor = $convert.base64Decode(
    'CgdBaU1vZGVsEg4KAmlkGAEgASgJUgJpZBISCgRuYW1lGAIgASgJUgRuYW1lEhoKCHByb3ZpZG'
    'VyGAMgASgJUghwcm92aWRlchIlCg5jb250ZXh0X3dpbmRvdxgEIAEoBVINY29udGV4dFdpbmRv'
    'dxIYCgdlbmFibGVkGAUgASgIUgdlbmFibGVk');

@$core.Deprecated('Use listModelsRequestDescriptor instead')
const ListModelsRequest$json = {
  '1': 'ListModelsRequest',
  '2': [
    {'1': 'provider_filter', '3': 1, '4': 1, '5': 9, '10': 'providerFilter'},
  ],
};

/// Descriptor for `ListModelsRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List listModelsRequestDescriptor = $convert.base64Decode(
    'ChFMaXN0TW9kZWxzUmVxdWVzdBInCg9wcm92aWRlcl9maWx0ZXIYASABKAlSDnByb3ZpZGVyRm'
    'lsdGVy');

@$core.Deprecated('Use listModelsResponseDescriptor instead')
const ListModelsResponse$json = {
  '1': 'ListModelsResponse',
  '2': [
    {
      '1': 'models',
      '3': 1,
      '4': 3,
      '5': 11,
      '6': '.k1s0.system.aigateway.v1.AiModel',
      '10': 'models'
    },
  ],
};

/// Descriptor for `ListModelsResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List listModelsResponseDescriptor = $convert.base64Decode(
    'ChJMaXN0TW9kZWxzUmVzcG9uc2USOQoGbW9kZWxzGAEgAygLMiEuazFzMC5zeXN0ZW0uYWlnYX'
    'Rld2F5LnYxLkFpTW9kZWxSBm1vZGVscw==');

@$core.Deprecated('Use getUsageRequestDescriptor instead')
const GetUsageRequest$json = {
  '1': 'GetUsageRequest',
  '2': [
    {'1': 'tenant_id', '3': 1, '4': 1, '5': 9, '10': 'tenantId'},
    {'1': 'start_date', '3': 2, '4': 1, '5': 9, '10': 'startDate'},
    {'1': 'end_date', '3': 3, '4': 1, '5': 9, '10': 'endDate'},
  ],
};

/// Descriptor for `GetUsageRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List getUsageRequestDescriptor = $convert.base64Decode(
    'Cg9HZXRVc2FnZVJlcXVlc3QSGwoJdGVuYW50X2lkGAEgASgJUgh0ZW5hbnRJZBIdCgpzdGFydF'
    '9kYXRlGAIgASgJUglzdGFydERhdGUSGQoIZW5kX2RhdGUYAyABKAlSB2VuZERhdGU=');

@$core.Deprecated('Use getUsageResponseDescriptor instead')
const GetUsageResponse$json = {
  '1': 'GetUsageResponse',
  '2': [
    {'1': 'tenant_id', '3': 1, '4': 1, '5': 9, '10': 'tenantId'},
    {
      '1': 'total_prompt_tokens',
      '3': 2,
      '4': 1,
      '5': 3,
      '10': 'totalPromptTokens'
    },
    {
      '1': 'total_completion_tokens',
      '3': 3,
      '4': 1,
      '5': 3,
      '10': 'totalCompletionTokens'
    },
    {'1': 'total_cost_usd', '3': 4, '4': 1, '5': 1, '10': 'totalCostUsd'},
  ],
};

/// Descriptor for `GetUsageResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List getUsageResponseDescriptor = $convert.base64Decode(
    'ChBHZXRVc2FnZVJlc3BvbnNlEhsKCXRlbmFudF9pZBgBIAEoCVIIdGVuYW50SWQSLgoTdG90YW'
    'xfcHJvbXB0X3Rva2VucxgCIAEoA1IRdG90YWxQcm9tcHRUb2tlbnMSNgoXdG90YWxfY29tcGxl'
    'dGlvbl90b2tlbnMYAyABKANSFXRvdGFsQ29tcGxldGlvblRva2VucxIkCg50b3RhbF9jb3N0X3'
    'VzZBgEIAEoAVIMdG90YWxDb3N0VXNk');
