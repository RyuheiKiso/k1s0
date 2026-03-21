// This is a generated file - do not edit.
//
// Generated from k1s0/system/ai_agent/v1/ai_agent.proto.

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

@$core.Deprecated('Use executeRequestDescriptor instead')
const ExecuteRequest$json = {
  '1': 'ExecuteRequest',
  '2': [
    {'1': 'agent_id', '3': 1, '4': 1, '5': 9, '10': 'agentId'},
    {'1': 'input', '3': 2, '4': 1, '5': 9, '10': 'input'},
    {'1': 'session_id', '3': 3, '4': 1, '5': 9, '10': 'sessionId'},
    {'1': 'tenant_id', '3': 4, '4': 1, '5': 9, '10': 'tenantId'},
    {
      '1': 'context',
      '3': 5,
      '4': 3,
      '5': 11,
      '6': '.k1s0.system.aiagent.v1.ExecuteRequest.ContextEntry',
      '10': 'context'
    },
  ],
  '3': [ExecuteRequest_ContextEntry$json],
};

@$core.Deprecated('Use executeRequestDescriptor instead')
const ExecuteRequest_ContextEntry$json = {
  '1': 'ContextEntry',
  '2': [
    {'1': 'key', '3': 1, '4': 1, '5': 9, '10': 'key'},
    {'1': 'value', '3': 2, '4': 1, '5': 9, '10': 'value'},
  ],
  '7': {'7': true},
};

/// Descriptor for `ExecuteRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List executeRequestDescriptor = $convert.base64Decode(
    'Cg5FeGVjdXRlUmVxdWVzdBIZCghhZ2VudF9pZBgBIAEoCVIHYWdlbnRJZBIUCgVpbnB1dBgCIA'
    'EoCVIFaW5wdXQSHQoKc2Vzc2lvbl9pZBgDIAEoCVIJc2Vzc2lvbklkEhsKCXRlbmFudF9pZBgE'
    'IAEoCVIIdGVuYW50SWQSTQoHY29udGV4dBgFIAMoCzIzLmsxczAuc3lzdGVtLmFpYWdlbnQudj'
    'EuRXhlY3V0ZVJlcXVlc3QuQ29udGV4dEVudHJ5Ugdjb250ZXh0GjoKDENvbnRleHRFbnRyeRIQ'
    'CgNrZXkYASABKAlSA2tleRIUCgV2YWx1ZRgCIAEoCVIFdmFsdWU6AjgB');

@$core.Deprecated('Use executeResponseDescriptor instead')
const ExecuteResponse$json = {
  '1': 'ExecuteResponse',
  '2': [
    {'1': 'execution_id', '3': 1, '4': 1, '5': 9, '10': 'executionId'},
    {'1': 'status', '3': 2, '4': 1, '5': 9, '10': 'status'},
    {'1': 'output', '3': 3, '4': 1, '5': 9, '10': 'output'},
    {
      '1': 'steps',
      '3': 4,
      '4': 3,
      '5': 11,
      '6': '.k1s0.system.aiagent.v1.ExecutionStep',
      '10': 'steps'
    },
  ],
};

/// Descriptor for `ExecuteResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List executeResponseDescriptor = $convert.base64Decode(
    'Cg9FeGVjdXRlUmVzcG9uc2USIQoMZXhlY3V0aW9uX2lkGAEgASgJUgtleGVjdXRpb25JZBIWCg'
    'ZzdGF0dXMYAiABKAlSBnN0YXR1cxIWCgZvdXRwdXQYAyABKAlSBm91dHB1dBI7CgVzdGVwcxgE'
    'IAMoCzIlLmsxczAuc3lzdGVtLmFpYWdlbnQudjEuRXhlY3V0aW9uU3RlcFIFc3RlcHM=');

@$core.Deprecated('Use executeStreamRequestDescriptor instead')
const ExecuteStreamRequest$json = {
  '1': 'ExecuteStreamRequest',
  '2': [
    {'1': 'agent_id', '3': 1, '4': 1, '5': 9, '10': 'agentId'},
    {'1': 'input', '3': 2, '4': 1, '5': 9, '10': 'input'},
    {'1': 'session_id', '3': 3, '4': 1, '5': 9, '10': 'sessionId'},
    {'1': 'tenant_id', '3': 4, '4': 1, '5': 9, '10': 'tenantId'},
    {
      '1': 'context',
      '3': 5,
      '4': 3,
      '5': 11,
      '6': '.k1s0.system.aiagent.v1.ExecuteStreamRequest.ContextEntry',
      '10': 'context'
    },
  ],
  '3': [ExecuteStreamRequest_ContextEntry$json],
};

@$core.Deprecated('Use executeStreamRequestDescriptor instead')
const ExecuteStreamRequest_ContextEntry$json = {
  '1': 'ContextEntry',
  '2': [
    {'1': 'key', '3': 1, '4': 1, '5': 9, '10': 'key'},
    {'1': 'value', '3': 2, '4': 1, '5': 9, '10': 'value'},
  ],
  '7': {'7': true},
};

/// Descriptor for `ExecuteStreamRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List executeStreamRequestDescriptor = $convert.base64Decode(
    'ChRFeGVjdXRlU3RyZWFtUmVxdWVzdBIZCghhZ2VudF9pZBgBIAEoCVIHYWdlbnRJZBIUCgVpbn'
    'B1dBgCIAEoCVIFaW5wdXQSHQoKc2Vzc2lvbl9pZBgDIAEoCVIJc2Vzc2lvbklkEhsKCXRlbmFu'
    'dF9pZBgEIAEoCVIIdGVuYW50SWQSUwoHY29udGV4dBgFIAMoCzI5LmsxczAuc3lzdGVtLmFpYW'
    'dlbnQudjEuRXhlY3V0ZVN0cmVhbVJlcXVlc3QuQ29udGV4dEVudHJ5Ugdjb250ZXh0GjoKDENv'
    'bnRleHRFbnRyeRIQCgNrZXkYASABKAlSA2tleRIUCgV2YWx1ZRgCIAEoCVIFdmFsdWU6AjgB');

@$core.Deprecated('Use executeStreamResponseDescriptor instead')
const ExecuteStreamResponse$json = {
  '1': 'ExecuteStreamResponse',
  '2': [
    {'1': 'execution_id', '3': 1, '4': 1, '5': 9, '10': 'executionId'},
    {'1': 'event_type', '3': 2, '4': 1, '5': 9, '10': 'eventType'},
    {'1': 'data', '3': 3, '4': 1, '5': 9, '10': 'data'},
    {'1': 'step_index', '3': 4, '4': 1, '5': 5, '10': 'stepIndex'},
  ],
};

/// Descriptor for `ExecuteStreamResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List executeStreamResponseDescriptor = $convert.base64Decode(
    'ChVFeGVjdXRlU3RyZWFtUmVzcG9uc2USIQoMZXhlY3V0aW9uX2lkGAEgASgJUgtleGVjdXRpb2'
    '5JZBIdCgpldmVudF90eXBlGAIgASgJUglldmVudFR5cGUSEgoEZGF0YRgDIAEoCVIEZGF0YRId'
    'CgpzdGVwX2luZGV4GAQgASgFUglzdGVwSW5kZXg=');

@$core.Deprecated('Use executionStepDescriptor instead')
const ExecutionStep$json = {
  '1': 'ExecutionStep',
  '2': [
    {'1': 'index', '3': 1, '4': 1, '5': 5, '10': 'index'},
    {'1': 'step_type', '3': 2, '4': 1, '5': 9, '10': 'stepType'},
    {'1': 'input', '3': 3, '4': 1, '5': 9, '10': 'input'},
    {'1': 'output', '3': 4, '4': 1, '5': 9, '10': 'output'},
    {'1': 'tool_name', '3': 5, '4': 1, '5': 9, '10': 'toolName'},
    {'1': 'status', '3': 6, '4': 1, '5': 9, '10': 'status'},
  ],
};

/// Descriptor for `ExecutionStep`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List executionStepDescriptor = $convert.base64Decode(
    'Cg1FeGVjdXRpb25TdGVwEhQKBWluZGV4GAEgASgFUgVpbmRleBIbCglzdGVwX3R5cGUYAiABKA'
    'lSCHN0ZXBUeXBlEhQKBWlucHV0GAMgASgJUgVpbnB1dBIWCgZvdXRwdXQYBCABKAlSBm91dHB1'
    'dBIbCgl0b29sX25hbWUYBSABKAlSCHRvb2xOYW1lEhYKBnN0YXR1cxgGIAEoCVIGc3RhdHVz');

@$core.Deprecated('Use cancelExecutionRequestDescriptor instead')
const CancelExecutionRequest$json = {
  '1': 'CancelExecutionRequest',
  '2': [
    {'1': 'execution_id', '3': 1, '4': 1, '5': 9, '10': 'executionId'},
  ],
};

/// Descriptor for `CancelExecutionRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List cancelExecutionRequestDescriptor =
    $convert.base64Decode(
        'ChZDYW5jZWxFeGVjdXRpb25SZXF1ZXN0EiEKDGV4ZWN1dGlvbl9pZBgBIAEoCVILZXhlY3V0aW'
        '9uSWQ=');

@$core.Deprecated('Use cancelExecutionResponseDescriptor instead')
const CancelExecutionResponse$json = {
  '1': 'CancelExecutionResponse',
  '2': [
    {'1': 'success', '3': 1, '4': 1, '5': 8, '10': 'success'},
  ],
};

/// Descriptor for `CancelExecutionResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List cancelExecutionResponseDescriptor =
    $convert.base64Decode(
        'ChdDYW5jZWxFeGVjdXRpb25SZXNwb25zZRIYCgdzdWNjZXNzGAEgASgIUgdzdWNjZXNz');

@$core.Deprecated('Use reviewStepRequestDescriptor instead')
const ReviewStepRequest$json = {
  '1': 'ReviewStepRequest',
  '2': [
    {'1': 'execution_id', '3': 1, '4': 1, '5': 9, '10': 'executionId'},
    {'1': 'step_index', '3': 2, '4': 1, '5': 5, '10': 'stepIndex'},
    {'1': 'approved', '3': 3, '4': 1, '5': 8, '10': 'approved'},
    {'1': 'feedback', '3': 4, '4': 1, '5': 9, '10': 'feedback'},
  ],
};

/// Descriptor for `ReviewStepRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List reviewStepRequestDescriptor = $convert.base64Decode(
    'ChFSZXZpZXdTdGVwUmVxdWVzdBIhCgxleGVjdXRpb25faWQYASABKAlSC2V4ZWN1dGlvbklkEh'
    '0KCnN0ZXBfaW5kZXgYAiABKAVSCXN0ZXBJbmRleBIaCghhcHByb3ZlZBgDIAEoCFIIYXBwcm92'
    'ZWQSGgoIZmVlZGJhY2sYBCABKAlSCGZlZWRiYWNr');

@$core.Deprecated('Use reviewStepResponseDescriptor instead')
const ReviewStepResponse$json = {
  '1': 'ReviewStepResponse',
  '2': [
    {'1': 'execution_id', '3': 1, '4': 1, '5': 9, '10': 'executionId'},
    {'1': 'resumed', '3': 2, '4': 1, '5': 8, '10': 'resumed'},
  ],
};

/// Descriptor for `ReviewStepResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List reviewStepResponseDescriptor = $convert.base64Decode(
    'ChJSZXZpZXdTdGVwUmVzcG9uc2USIQoMZXhlY3V0aW9uX2lkGAEgASgJUgtleGVjdXRpb25JZB'
    'IYCgdyZXN1bWVkGAIgASgIUgdyZXN1bWVk');
