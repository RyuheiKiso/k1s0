// This is a generated file - do not edit.
//
// Generated from k1s0/system/saga/v1/saga.proto.

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

@$core.Deprecated('Use sagaStatusDescriptor instead')
const SagaStatus$json = {
  '1': 'SagaStatus',
  '2': [
    {'1': 'SAGA_STATUS_UNSPECIFIED', '2': 0},
    {'1': 'SAGA_STATUS_RUNNING', '2': 1},
    {'1': 'SAGA_STATUS_COMPLETED', '2': 2},
    {'1': 'SAGA_STATUS_FAILED', '2': 3},
    {'1': 'SAGA_STATUS_COMPENSATING', '2': 4},
    {'1': 'SAGA_STATUS_COMPENSATED', '2': 5},
  ],
};

/// Descriptor for `SagaStatus`. Decode as a `google.protobuf.EnumDescriptorProto`.
final $typed_data.Uint8List sagaStatusDescriptor = $convert.base64Decode(
    'CgpTYWdhU3RhdHVzEhsKF1NBR0FfU1RBVFVTX1VOU1BFQ0lGSUVEEAASFwoTU0FHQV9TVEFUVV'
    'NfUlVOTklORxABEhkKFVNBR0FfU1RBVFVTX0NPTVBMRVRFRBACEhYKElNBR0FfU1RBVFVTX0ZB'
    'SUxFRBADEhwKGFNBR0FfU1RBVFVTX0NPTVBFTlNBVElORxAEEhsKF1NBR0FfU1RBVFVTX0NPTV'
    'BFTlNBVEVEEAU=');

@$core.Deprecated('Use sagaStateProtoDescriptor instead')
const SagaStateProto$json = {
  '1': 'SagaStateProto',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 9, '10': 'id'},
    {'1': 'workflow_name', '3': 2, '4': 1, '5': 9, '10': 'workflowName'},
    {'1': 'current_step', '3': 3, '4': 1, '5': 5, '10': 'currentStep'},
    {'1': 'status', '3': 4, '4': 1, '5': 9, '10': 'status'},
    {
      '1': 'payload',
      '3': 5,
      '4': 1,
      '5': 11,
      '6': '.google.protobuf.Struct',
      '10': 'payload'
    },
    {
      '1': 'correlation_id',
      '3': 6,
      '4': 1,
      '5': 9,
      '9': 0,
      '10': 'correlationId',
      '17': true
    },
    {
      '1': 'initiated_by',
      '3': 7,
      '4': 1,
      '5': 9,
      '9': 1,
      '10': 'initiatedBy',
      '17': true
    },
    {
      '1': 'error_message',
      '3': 8,
      '4': 1,
      '5': 9,
      '9': 2,
      '10': 'errorMessage',
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
    {
      '1': 'status_enum',
      '3': 11,
      '4': 1,
      '5': 14,
      '6': '.k1s0.system.saga.v1.SagaStatus',
      '10': 'statusEnum'
    },
  ],
  '8': [
    {'1': '_correlation_id'},
    {'1': '_initiated_by'},
    {'1': '_error_message'},
  ],
};

/// Descriptor for `SagaStateProto`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List sagaStateProtoDescriptor = $convert.base64Decode(
    'Cg5TYWdhU3RhdGVQcm90bxIOCgJpZBgBIAEoCVICaWQSIwoNd29ya2Zsb3dfbmFtZRgCIAEoCV'
    'IMd29ya2Zsb3dOYW1lEiEKDGN1cnJlbnRfc3RlcBgDIAEoBVILY3VycmVudFN0ZXASFgoGc3Rh'
    'dHVzGAQgASgJUgZzdGF0dXMSMQoHcGF5bG9hZBgFIAEoCzIXLmdvb2dsZS5wcm90b2J1Zi5TdH'
    'J1Y3RSB3BheWxvYWQSKgoOY29ycmVsYXRpb25faWQYBiABKAlIAFINY29ycmVsYXRpb25JZIgB'
    'ARImCgxpbml0aWF0ZWRfYnkYByABKAlIAVILaW5pdGlhdGVkQnmIAQESKAoNZXJyb3JfbWVzc2'
    'FnZRgIIAEoCUgCUgxlcnJvck1lc3NhZ2WIAQESPwoKY3JlYXRlZF9hdBgJIAEoCzIgLmsxczAu'
    'c3lzdGVtLmNvbW1vbi52MS5UaW1lc3RhbXBSCWNyZWF0ZWRBdBI/Cgp1cGRhdGVkX2F0GAogAS'
    'gLMiAuazFzMC5zeXN0ZW0uY29tbW9uLnYxLlRpbWVzdGFtcFIJdXBkYXRlZEF0EkAKC3N0YXR1'
    'c19lbnVtGAsgASgOMh8uazFzMC5zeXN0ZW0uc2FnYS52MS5TYWdhU3RhdHVzUgpzdGF0dXNFbn'
    'VtQhEKD19jb3JyZWxhdGlvbl9pZEIPCg1faW5pdGlhdGVkX2J5QhAKDl9lcnJvcl9tZXNzYWdl');

@$core.Deprecated('Use sagaStepLogProtoDescriptor instead')
const SagaStepLogProto$json = {
  '1': 'SagaStepLogProto',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 9, '10': 'id'},
    {'1': 'saga_id', '3': 2, '4': 1, '5': 9, '10': 'sagaId'},
    {'1': 'step_index', '3': 3, '4': 1, '5': 5, '10': 'stepIndex'},
    {'1': 'step_name', '3': 4, '4': 1, '5': 9, '10': 'stepName'},
    {'1': 'action', '3': 5, '4': 1, '5': 9, '10': 'action'},
    {'1': 'status', '3': 6, '4': 1, '5': 9, '10': 'status'},
    {
      '1': 'request_payload',
      '3': 7,
      '4': 1,
      '5': 11,
      '6': '.google.protobuf.Struct',
      '10': 'requestPayload'
    },
    {
      '1': 'response_payload',
      '3': 8,
      '4': 1,
      '5': 11,
      '6': '.google.protobuf.Struct',
      '10': 'responsePayload'
    },
    {
      '1': 'error_message',
      '3': 9,
      '4': 1,
      '5': 9,
      '9': 0,
      '10': 'errorMessage',
      '17': true
    },
    {
      '1': 'started_at',
      '3': 10,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.common.v1.Timestamp',
      '10': 'startedAt'
    },
    {
      '1': 'completed_at',
      '3': 11,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.common.v1.Timestamp',
      '9': 1,
      '10': 'completedAt',
      '17': true
    },
  ],
  '8': [
    {'1': '_error_message'},
    {'1': '_completed_at'},
  ],
};

/// Descriptor for `SagaStepLogProto`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List sagaStepLogProtoDescriptor = $convert.base64Decode(
    'ChBTYWdhU3RlcExvZ1Byb3RvEg4KAmlkGAEgASgJUgJpZBIXCgdzYWdhX2lkGAIgASgJUgZzYW'
    'dhSWQSHQoKc3RlcF9pbmRleBgDIAEoBVIJc3RlcEluZGV4EhsKCXN0ZXBfbmFtZRgEIAEoCVII'
    'c3RlcE5hbWUSFgoGYWN0aW9uGAUgASgJUgZhY3Rpb24SFgoGc3RhdHVzGAYgASgJUgZzdGF0dX'
    'MSQAoPcmVxdWVzdF9wYXlsb2FkGAcgASgLMhcuZ29vZ2xlLnByb3RvYnVmLlN0cnVjdFIOcmVx'
    'dWVzdFBheWxvYWQSQgoQcmVzcG9uc2VfcGF5bG9hZBgIIAEoCzIXLmdvb2dsZS5wcm90b2J1Zi'
    '5TdHJ1Y3RSD3Jlc3BvbnNlUGF5bG9hZBIoCg1lcnJvcl9tZXNzYWdlGAkgASgJSABSDGVycm9y'
    'TWVzc2FnZYgBARI/CgpzdGFydGVkX2F0GAogASgLMiAuazFzMC5zeXN0ZW0uY29tbW9uLnYxLl'
    'RpbWVzdGFtcFIJc3RhcnRlZEF0EkgKDGNvbXBsZXRlZF9hdBgLIAEoCzIgLmsxczAuc3lzdGVt'
    'LmNvbW1vbi52MS5UaW1lc3RhbXBIAVILY29tcGxldGVkQXSIAQFCEAoOX2Vycm9yX21lc3NhZ2'
    'VCDwoNX2NvbXBsZXRlZF9hdA==');

@$core.Deprecated('Use workflowSummaryDescriptor instead')
const WorkflowSummary$json = {
  '1': 'WorkflowSummary',
  '2': [
    {'1': 'name', '3': 1, '4': 1, '5': 9, '10': 'name'},
    {'1': 'step_count', '3': 2, '4': 1, '5': 5, '10': 'stepCount'},
    {'1': 'step_names', '3': 3, '4': 3, '5': 9, '10': 'stepNames'},
  ],
};

/// Descriptor for `WorkflowSummary`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List workflowSummaryDescriptor = $convert.base64Decode(
    'Cg9Xb3JrZmxvd1N1bW1hcnkSEgoEbmFtZRgBIAEoCVIEbmFtZRIdCgpzdGVwX2NvdW50GAIgAS'
    'gFUglzdGVwQ291bnQSHQoKc3RlcF9uYW1lcxgDIAMoCVIJc3RlcE5hbWVz');

@$core.Deprecated('Use startSagaRequestDescriptor instead')
const StartSagaRequest$json = {
  '1': 'StartSagaRequest',
  '2': [
    {'1': 'workflow_name', '3': 1, '4': 1, '5': 9, '10': 'workflowName'},
    {
      '1': 'payload',
      '3': 2,
      '4': 1,
      '5': 11,
      '6': '.google.protobuf.Struct',
      '10': 'payload'
    },
    {'1': 'correlation_id', '3': 3, '4': 1, '5': 9, '10': 'correlationId'},
    {'1': 'initiated_by', '3': 4, '4': 1, '5': 9, '10': 'initiatedBy'},
  ],
};

/// Descriptor for `StartSagaRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List startSagaRequestDescriptor = $convert.base64Decode(
    'ChBTdGFydFNhZ2FSZXF1ZXN0EiMKDXdvcmtmbG93X25hbWUYASABKAlSDHdvcmtmbG93TmFtZR'
    'IxCgdwYXlsb2FkGAIgASgLMhcuZ29vZ2xlLnByb3RvYnVmLlN0cnVjdFIHcGF5bG9hZBIlCg5j'
    'b3JyZWxhdGlvbl9pZBgDIAEoCVINY29ycmVsYXRpb25JZBIhCgxpbml0aWF0ZWRfYnkYBCABKA'
    'lSC2luaXRpYXRlZEJ5');

@$core.Deprecated('Use startSagaResponseDescriptor instead')
const StartSagaResponse$json = {
  '1': 'StartSagaResponse',
  '2': [
    {'1': 'saga_id', '3': 1, '4': 1, '5': 9, '10': 'sagaId'},
    {'1': 'status', '3': 2, '4': 1, '5': 9, '10': 'status'},
  ],
};

/// Descriptor for `StartSagaResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List startSagaResponseDescriptor = $convert.base64Decode(
    'ChFTdGFydFNhZ2FSZXNwb25zZRIXCgdzYWdhX2lkGAEgASgJUgZzYWdhSWQSFgoGc3RhdHVzGA'
    'IgASgJUgZzdGF0dXM=');

@$core.Deprecated('Use getSagaRequestDescriptor instead')
const GetSagaRequest$json = {
  '1': 'GetSagaRequest',
  '2': [
    {'1': 'saga_id', '3': 1, '4': 1, '5': 9, '10': 'sagaId'},
  ],
};

/// Descriptor for `GetSagaRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List getSagaRequestDescriptor = $convert
    .base64Decode('Cg5HZXRTYWdhUmVxdWVzdBIXCgdzYWdhX2lkGAEgASgJUgZzYWdhSWQ=');

@$core.Deprecated('Use getSagaResponseDescriptor instead')
const GetSagaResponse$json = {
  '1': 'GetSagaResponse',
  '2': [
    {
      '1': 'saga',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.saga.v1.SagaStateProto',
      '10': 'saga'
    },
    {
      '1': 'step_logs',
      '3': 2,
      '4': 3,
      '5': 11,
      '6': '.k1s0.system.saga.v1.SagaStepLogProto',
      '10': 'stepLogs'
    },
  ],
};

/// Descriptor for `GetSagaResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List getSagaResponseDescriptor = $convert.base64Decode(
    'Cg9HZXRTYWdhUmVzcG9uc2USNwoEc2FnYRgBIAEoCzIjLmsxczAuc3lzdGVtLnNhZ2EudjEuU2'
    'FnYVN0YXRlUHJvdG9SBHNhZ2ESQgoJc3RlcF9sb2dzGAIgAygLMiUuazFzMC5zeXN0ZW0uc2Fn'
    'YS52MS5TYWdhU3RlcExvZ1Byb3RvUghzdGVwTG9ncw==');

@$core.Deprecated('Use listSagasRequestDescriptor instead')
const ListSagasRequest$json = {
  '1': 'ListSagasRequest',
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
      '1': 'workflow_name',
      '3': 2,
      '4': 1,
      '5': 9,
      '9': 0,
      '10': 'workflowName',
      '17': true
    },
    {'1': 'status', '3': 3, '4': 1, '5': 9, '9': 1, '10': 'status', '17': true},
    {
      '1': 'correlation_id',
      '3': 4,
      '4': 1,
      '5': 9,
      '9': 2,
      '10': 'correlationId',
      '17': true
    },
  ],
  '8': [
    {'1': '_workflow_name'},
    {'1': '_status'},
    {'1': '_correlation_id'},
  ],
};

/// Descriptor for `ListSagasRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List listSagasRequestDescriptor = $convert.base64Decode(
    'ChBMaXN0U2FnYXNSZXF1ZXN0EkEKCnBhZ2luYXRpb24YASABKAsyIS5rMXMwLnN5c3RlbS5jb2'
    '1tb24udjEuUGFnaW5hdGlvblIKcGFnaW5hdGlvbhIoCg13b3JrZmxvd19uYW1lGAIgASgJSABS'
    'DHdvcmtmbG93TmFtZYgBARIbCgZzdGF0dXMYAyABKAlIAVIGc3RhdHVziAEBEioKDmNvcnJlbG'
    'F0aW9uX2lkGAQgASgJSAJSDWNvcnJlbGF0aW9uSWSIAQFCEAoOX3dvcmtmbG93X25hbWVCCQoH'
    'X3N0YXR1c0IRCg9fY29ycmVsYXRpb25faWQ=');

@$core.Deprecated('Use listSagasResponseDescriptor instead')
const ListSagasResponse$json = {
  '1': 'ListSagasResponse',
  '2': [
    {
      '1': 'sagas',
      '3': 1,
      '4': 3,
      '5': 11,
      '6': '.k1s0.system.saga.v1.SagaStateProto',
      '10': 'sagas'
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

/// Descriptor for `ListSagasResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List listSagasResponseDescriptor = $convert.base64Decode(
    'ChFMaXN0U2FnYXNSZXNwb25zZRI5CgVzYWdhcxgBIAMoCzIjLmsxczAuc3lzdGVtLnNhZ2Eudj'
    'EuU2FnYVN0YXRlUHJvdG9SBXNhZ2FzEkcKCnBhZ2luYXRpb24YAiABKAsyJy5rMXMwLnN5c3Rl'
    'bS5jb21tb24udjEuUGFnaW5hdGlvblJlc3VsdFIKcGFnaW5hdGlvbg==');

@$core.Deprecated('Use cancelSagaRequestDescriptor instead')
const CancelSagaRequest$json = {
  '1': 'CancelSagaRequest',
  '2': [
    {'1': 'saga_id', '3': 1, '4': 1, '5': 9, '10': 'sagaId'},
  ],
};

/// Descriptor for `CancelSagaRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List cancelSagaRequestDescriptor = $convert.base64Decode(
    'ChFDYW5jZWxTYWdhUmVxdWVzdBIXCgdzYWdhX2lkGAEgASgJUgZzYWdhSWQ=');

@$core.Deprecated('Use cancelSagaResponseDescriptor instead')
const CancelSagaResponse$json = {
  '1': 'CancelSagaResponse',
  '2': [
    {'1': 'success', '3': 1, '4': 1, '5': 8, '10': 'success'},
    {'1': 'message', '3': 2, '4': 1, '5': 9, '10': 'message'},
  ],
};

/// Descriptor for `CancelSagaResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List cancelSagaResponseDescriptor = $convert.base64Decode(
    'ChJDYW5jZWxTYWdhUmVzcG9uc2USGAoHc3VjY2VzcxgBIAEoCFIHc3VjY2VzcxIYCgdtZXNzYW'
    'dlGAIgASgJUgdtZXNzYWdl');

@$core.Deprecated('Use compensateSagaRequestDescriptor instead')
const CompensateSagaRequest$json = {
  '1': 'CompensateSagaRequest',
  '2': [
    {'1': 'saga_id', '3': 1, '4': 1, '5': 9, '10': 'sagaId'},
  ],
};

/// Descriptor for `CompensateSagaRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List compensateSagaRequestDescriptor =
    $convert.base64Decode(
        'ChVDb21wZW5zYXRlU2FnYVJlcXVlc3QSFwoHc2FnYV9pZBgBIAEoCVIGc2FnYUlk');

@$core.Deprecated('Use compensateSagaResponseDescriptor instead')
const CompensateSagaResponse$json = {
  '1': 'CompensateSagaResponse',
  '2': [
    {'1': 'success', '3': 1, '4': 1, '5': 8, '10': 'success'},
    {'1': 'status', '3': 2, '4': 1, '5': 9, '10': 'status'},
    {'1': 'message', '3': 3, '4': 1, '5': 9, '10': 'message'},
    {'1': 'saga_id', '3': 4, '4': 1, '5': 9, '10': 'sagaId'},
  ],
};

/// Descriptor for `CompensateSagaResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List compensateSagaResponseDescriptor = $convert.base64Decode(
    'ChZDb21wZW5zYXRlU2FnYVJlc3BvbnNlEhgKB3N1Y2Nlc3MYASABKAhSB3N1Y2Nlc3MSFgoGc3'
    'RhdHVzGAIgASgJUgZzdGF0dXMSGAoHbWVzc2FnZRgDIAEoCVIHbWVzc2FnZRIXCgdzYWdhX2lk'
    'GAQgASgJUgZzYWdhSWQ=');

@$core.Deprecated('Use registerWorkflowRequestDescriptor instead')
const RegisterWorkflowRequest$json = {
  '1': 'RegisterWorkflowRequest',
  '2': [
    {'1': 'workflow_yaml', '3': 1, '4': 1, '5': 9, '10': 'workflowYaml'},
  ],
};

/// Descriptor for `RegisterWorkflowRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List registerWorkflowRequestDescriptor =
    $convert.base64Decode(
        'ChdSZWdpc3RlcldvcmtmbG93UmVxdWVzdBIjCg13b3JrZmxvd195YW1sGAEgASgJUgx3b3JrZm'
        'xvd1lhbWw=');

@$core.Deprecated('Use registerWorkflowResponseDescriptor instead')
const RegisterWorkflowResponse$json = {
  '1': 'RegisterWorkflowResponse',
  '2': [
    {'1': 'name', '3': 1, '4': 1, '5': 9, '10': 'name'},
    {'1': 'step_count', '3': 2, '4': 1, '5': 5, '10': 'stepCount'},
  ],
};

/// Descriptor for `RegisterWorkflowResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List registerWorkflowResponseDescriptor =
    $convert.base64Decode(
        'ChhSZWdpc3RlcldvcmtmbG93UmVzcG9uc2USEgoEbmFtZRgBIAEoCVIEbmFtZRIdCgpzdGVwX2'
        'NvdW50GAIgASgFUglzdGVwQ291bnQ=');

@$core.Deprecated('Use listWorkflowsRequestDescriptor instead')
const ListWorkflowsRequest$json = {
  '1': 'ListWorkflowsRequest',
};

/// Descriptor for `ListWorkflowsRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List listWorkflowsRequestDescriptor =
    $convert.base64Decode('ChRMaXN0V29ya2Zsb3dzUmVxdWVzdA==');

@$core.Deprecated('Use listWorkflowsResponseDescriptor instead')
const ListWorkflowsResponse$json = {
  '1': 'ListWorkflowsResponse',
  '2': [
    {
      '1': 'workflows',
      '3': 1,
      '4': 3,
      '5': 11,
      '6': '.k1s0.system.saga.v1.WorkflowSummary',
      '10': 'workflows'
    },
  ],
};

/// Descriptor for `ListWorkflowsResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List listWorkflowsResponseDescriptor = $convert.base64Decode(
    'ChVMaXN0V29ya2Zsb3dzUmVzcG9uc2USQgoJd29ya2Zsb3dzGAEgAygLMiQuazFzMC5zeXN0ZW'
    '0uc2FnYS52MS5Xb3JrZmxvd1N1bW1hcnlSCXdvcmtmbG93cw==');
