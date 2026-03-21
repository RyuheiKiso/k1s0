// This is a generated file - do not edit.
//
// Generated from k1s0/system/workflow/v1/workflow.proto.

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

@$core.Deprecated('Use workflowStepTypeDescriptor instead')
const WorkflowStepType$json = {
  '1': 'WorkflowStepType',
  '2': [
    {'1': 'WORKFLOW_STEP_TYPE_UNSPECIFIED', '2': 0},
    {'1': 'WORKFLOW_STEP_TYPE_APPROVAL', '2': 1},
    {'1': 'WORKFLOW_STEP_TYPE_AUTOMATED', '2': 2},
    {'1': 'WORKFLOW_STEP_TYPE_NOTIFICATION', '2': 3},
  ],
};

/// Descriptor for `WorkflowStepType`. Decode as a `google.protobuf.EnumDescriptorProto`.
final $typed_data.Uint8List workflowStepTypeDescriptor = $convert.base64Decode(
    'ChBXb3JrZmxvd1N0ZXBUeXBlEiIKHldPUktGTE9XX1NURVBfVFlQRV9VTlNQRUNJRklFRBAAEh'
    '8KG1dPUktGTE9XX1NURVBfVFlQRV9BUFBST1ZBTBABEiAKHFdPUktGTE9XX1NURVBfVFlQRV9B'
    'VVRPTUFURUQQAhIjCh9XT1JLRkxPV19TVEVQX1RZUEVfTk9USUZJQ0FUSU9OEAM=');

@$core.Deprecated('Use workflowStepDescriptor instead')
const WorkflowStep$json = {
  '1': 'WorkflowStep',
  '2': [
    {'1': 'step_id', '3': 1, '4': 1, '5': 9, '10': 'stepId'},
    {'1': 'name', '3': 2, '4': 1, '5': 9, '10': 'name'},
    {'1': 'step_type', '3': 3, '4': 1, '5': 9, '10': 'stepType'},
    {
      '1': 'assignee_role',
      '3': 4,
      '4': 1,
      '5': 9,
      '9': 0,
      '10': 'assigneeRole',
      '17': true
    },
    {
      '1': 'timeout_hours',
      '3': 5,
      '4': 1,
      '5': 13,
      '9': 1,
      '10': 'timeoutHours',
      '17': true
    },
    {
      '1': 'on_approve',
      '3': 6,
      '4': 1,
      '5': 9,
      '9': 2,
      '10': 'onApprove',
      '17': true
    },
    {
      '1': 'on_reject',
      '3': 7,
      '4': 1,
      '5': 9,
      '9': 3,
      '10': 'onReject',
      '17': true
    },
    {
      '1': 'step_type_enum',
      '3': 8,
      '4': 1,
      '5': 14,
      '6': '.k1s0.system.workflow.v1.WorkflowStepType',
      '10': 'stepTypeEnum'
    },
  ],
  '8': [
    {'1': '_assignee_role'},
    {'1': '_timeout_hours'},
    {'1': '_on_approve'},
    {'1': '_on_reject'},
  ],
};

/// Descriptor for `WorkflowStep`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List workflowStepDescriptor = $convert.base64Decode(
    'CgxXb3JrZmxvd1N0ZXASFwoHc3RlcF9pZBgBIAEoCVIGc3RlcElkEhIKBG5hbWUYAiABKAlSBG'
    '5hbWUSGwoJc3RlcF90eXBlGAMgASgJUghzdGVwVHlwZRIoCg1hc3NpZ25lZV9yb2xlGAQgASgJ'
    'SABSDGFzc2lnbmVlUm9sZYgBARIoCg10aW1lb3V0X2hvdXJzGAUgASgNSAFSDHRpbWVvdXRIb3'
    'Vyc4gBARIiCgpvbl9hcHByb3ZlGAYgASgJSAJSCW9uQXBwcm92ZYgBARIgCglvbl9yZWplY3QY'
    'ByABKAlIA1IIb25SZWplY3SIAQESTwoOc3RlcF90eXBlX2VudW0YCCABKA4yKS5rMXMwLnN5c3'
    'RlbS53b3JrZmxvdy52MS5Xb3JrZmxvd1N0ZXBUeXBlUgxzdGVwVHlwZUVudW1CEAoOX2Fzc2ln'
    'bmVlX3JvbGVCEAoOX3RpbWVvdXRfaG91cnNCDQoLX29uX2FwcHJvdmVCDAoKX29uX3JlamVjdA'
    '==');

@$core.Deprecated('Use workflowDefinitionDescriptor instead')
const WorkflowDefinition$json = {
  '1': 'WorkflowDefinition',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 9, '10': 'id'},
    {'1': 'name', '3': 2, '4': 1, '5': 9, '10': 'name'},
    {'1': 'description', '3': 3, '4': 1, '5': 9, '10': 'description'},
    {'1': 'version', '3': 4, '4': 1, '5': 13, '10': 'version'},
    {'1': 'enabled', '3': 5, '4': 1, '5': 8, '10': 'enabled'},
    {
      '1': 'steps',
      '3': 6,
      '4': 3,
      '5': 11,
      '6': '.k1s0.system.workflow.v1.WorkflowStep',
      '10': 'steps'
    },
    {
      '1': 'created_at',
      '3': 7,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.common.v1.Timestamp',
      '10': 'createdAt'
    },
    {
      '1': 'updated_at',
      '3': 8,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.common.v1.Timestamp',
      '10': 'updatedAt'
    },
  ],
};

/// Descriptor for `WorkflowDefinition`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List workflowDefinitionDescriptor = $convert.base64Decode(
    'ChJXb3JrZmxvd0RlZmluaXRpb24SDgoCaWQYASABKAlSAmlkEhIKBG5hbWUYAiABKAlSBG5hbW'
    'USIAoLZGVzY3JpcHRpb24YAyABKAlSC2Rlc2NyaXB0aW9uEhgKB3ZlcnNpb24YBCABKA1SB3Zl'
    'cnNpb24SGAoHZW5hYmxlZBgFIAEoCFIHZW5hYmxlZBI7CgVzdGVwcxgGIAMoCzIlLmsxczAuc3'
    'lzdGVtLndvcmtmbG93LnYxLldvcmtmbG93U3RlcFIFc3RlcHMSPwoKY3JlYXRlZF9hdBgHIAEo'
    'CzIgLmsxczAuc3lzdGVtLmNvbW1vbi52MS5UaW1lc3RhbXBSCWNyZWF0ZWRBdBI/Cgp1cGRhdG'
    'VkX2F0GAggASgLMiAuazFzMC5zeXN0ZW0uY29tbW9uLnYxLlRpbWVzdGFtcFIJdXBkYXRlZEF0');

@$core.Deprecated('Use listWorkflowsRequestDescriptor instead')
const ListWorkflowsRequest$json = {
  '1': 'ListWorkflowsRequest',
  '2': [
    {'1': 'enabled_only', '3': 1, '4': 1, '5': 8, '10': 'enabledOnly'},
    {
      '1': 'pagination',
      '3': 2,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.common.v1.Pagination',
      '10': 'pagination'
    },
  ],
};

/// Descriptor for `ListWorkflowsRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List listWorkflowsRequestDescriptor = $convert.base64Decode(
    'ChRMaXN0V29ya2Zsb3dzUmVxdWVzdBIhCgxlbmFibGVkX29ubHkYASABKAhSC2VuYWJsZWRPbm'
    'x5EkEKCnBhZ2luYXRpb24YAiABKAsyIS5rMXMwLnN5c3RlbS5jb21tb24udjEuUGFnaW5hdGlv'
    'blIKcGFnaW5hdGlvbg==');

@$core.Deprecated('Use listWorkflowsResponseDescriptor instead')
const ListWorkflowsResponse$json = {
  '1': 'ListWorkflowsResponse',
  '2': [
    {
      '1': 'workflows',
      '3': 1,
      '4': 3,
      '5': 11,
      '6': '.k1s0.system.workflow.v1.WorkflowDefinition',
      '10': 'workflows'
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

/// Descriptor for `ListWorkflowsResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List listWorkflowsResponseDescriptor = $convert.base64Decode(
    'ChVMaXN0V29ya2Zsb3dzUmVzcG9uc2USSQoJd29ya2Zsb3dzGAEgAygLMisuazFzMC5zeXN0ZW'
    '0ud29ya2Zsb3cudjEuV29ya2Zsb3dEZWZpbml0aW9uUgl3b3JrZmxvd3MSRwoKcGFnaW5hdGlv'
    'bhgCIAEoCzInLmsxczAuc3lzdGVtLmNvbW1vbi52MS5QYWdpbmF0aW9uUmVzdWx0UgpwYWdpbm'
    'F0aW9u');

@$core.Deprecated('Use createWorkflowRequestDescriptor instead')
const CreateWorkflowRequest$json = {
  '1': 'CreateWorkflowRequest',
  '2': [
    {'1': 'name', '3': 1, '4': 1, '5': 9, '10': 'name'},
    {'1': 'description', '3': 2, '4': 1, '5': 9, '10': 'description'},
    {'1': 'enabled', '3': 3, '4': 1, '5': 8, '10': 'enabled'},
    {
      '1': 'steps',
      '3': 4,
      '4': 3,
      '5': 11,
      '6': '.k1s0.system.workflow.v1.WorkflowStep',
      '10': 'steps'
    },
  ],
};

/// Descriptor for `CreateWorkflowRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List createWorkflowRequestDescriptor = $convert.base64Decode(
    'ChVDcmVhdGVXb3JrZmxvd1JlcXVlc3QSEgoEbmFtZRgBIAEoCVIEbmFtZRIgCgtkZXNjcmlwdG'
    'lvbhgCIAEoCVILZGVzY3JpcHRpb24SGAoHZW5hYmxlZBgDIAEoCFIHZW5hYmxlZBI7CgVzdGVw'
    'cxgEIAMoCzIlLmsxczAuc3lzdGVtLndvcmtmbG93LnYxLldvcmtmbG93U3RlcFIFc3RlcHM=');

@$core.Deprecated('Use createWorkflowResponseDescriptor instead')
const CreateWorkflowResponse$json = {
  '1': 'CreateWorkflowResponse',
  '2': [
    {
      '1': 'workflow',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.workflow.v1.WorkflowDefinition',
      '10': 'workflow'
    },
  ],
};

/// Descriptor for `CreateWorkflowResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List createWorkflowResponseDescriptor =
    $convert.base64Decode(
        'ChZDcmVhdGVXb3JrZmxvd1Jlc3BvbnNlEkcKCHdvcmtmbG93GAEgASgLMisuazFzMC5zeXN0ZW'
        '0ud29ya2Zsb3cudjEuV29ya2Zsb3dEZWZpbml0aW9uUgh3b3JrZmxvdw==');

@$core.Deprecated('Use getWorkflowRequestDescriptor instead')
const GetWorkflowRequest$json = {
  '1': 'GetWorkflowRequest',
  '2': [
    {'1': 'workflow_id', '3': 1, '4': 1, '5': 9, '10': 'workflowId'},
  ],
};

/// Descriptor for `GetWorkflowRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List getWorkflowRequestDescriptor = $convert.base64Decode(
    'ChJHZXRXb3JrZmxvd1JlcXVlc3QSHwoLd29ya2Zsb3dfaWQYASABKAlSCndvcmtmbG93SWQ=');

@$core.Deprecated('Use getWorkflowResponseDescriptor instead')
const GetWorkflowResponse$json = {
  '1': 'GetWorkflowResponse',
  '2': [
    {
      '1': 'workflow',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.workflow.v1.WorkflowDefinition',
      '10': 'workflow'
    },
  ],
};

/// Descriptor for `GetWorkflowResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List getWorkflowResponseDescriptor = $convert.base64Decode(
    'ChNHZXRXb3JrZmxvd1Jlc3BvbnNlEkcKCHdvcmtmbG93GAEgASgLMisuazFzMC5zeXN0ZW0ud2'
    '9ya2Zsb3cudjEuV29ya2Zsb3dEZWZpbml0aW9uUgh3b3JrZmxvdw==');

@$core.Deprecated('Use workflowStepsDescriptor instead')
const WorkflowSteps$json = {
  '1': 'WorkflowSteps',
  '2': [
    {
      '1': 'items',
      '3': 1,
      '4': 3,
      '5': 11,
      '6': '.k1s0.system.workflow.v1.WorkflowStep',
      '10': 'items'
    },
  ],
};

/// Descriptor for `WorkflowSteps`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List workflowStepsDescriptor = $convert.base64Decode(
    'Cg1Xb3JrZmxvd1N0ZXBzEjsKBWl0ZW1zGAEgAygLMiUuazFzMC5zeXN0ZW0ud29ya2Zsb3cudj'
    'EuV29ya2Zsb3dTdGVwUgVpdGVtcw==');

@$core.Deprecated('Use updateWorkflowRequestDescriptor instead')
const UpdateWorkflowRequest$json = {
  '1': 'UpdateWorkflowRequest',
  '2': [
    {'1': 'workflow_id', '3': 1, '4': 1, '5': 9, '10': 'workflowId'},
    {'1': 'name', '3': 2, '4': 1, '5': 9, '9': 0, '10': 'name', '17': true},
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
      '1': 'enabled',
      '3': 4,
      '4': 1,
      '5': 8,
      '9': 2,
      '10': 'enabled',
      '17': true
    },
    {
      '1': 'steps',
      '3': 5,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.workflow.v1.WorkflowSteps',
      '9': 3,
      '10': 'steps',
      '17': true
    },
  ],
  '8': [
    {'1': '_name'},
    {'1': '_description'},
    {'1': '_enabled'},
    {'1': '_steps'},
  ],
};

/// Descriptor for `UpdateWorkflowRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List updateWorkflowRequestDescriptor = $convert.base64Decode(
    'ChVVcGRhdGVXb3JrZmxvd1JlcXVlc3QSHwoLd29ya2Zsb3dfaWQYASABKAlSCndvcmtmbG93SW'
    'QSFwoEbmFtZRgCIAEoCUgAUgRuYW1liAEBEiUKC2Rlc2NyaXB0aW9uGAMgASgJSAFSC2Rlc2Ny'
    'aXB0aW9uiAEBEh0KB2VuYWJsZWQYBCABKAhIAlIHZW5hYmxlZIgBARJBCgVzdGVwcxgFIAEoCz'
    'ImLmsxczAuc3lzdGVtLndvcmtmbG93LnYxLldvcmtmbG93U3RlcHNIA1IFc3RlcHOIAQFCBwoF'
    'X25hbWVCDgoMX2Rlc2NyaXB0aW9uQgoKCF9lbmFibGVkQggKBl9zdGVwcw==');

@$core.Deprecated('Use updateWorkflowResponseDescriptor instead')
const UpdateWorkflowResponse$json = {
  '1': 'UpdateWorkflowResponse',
  '2': [
    {
      '1': 'workflow',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.workflow.v1.WorkflowDefinition',
      '10': 'workflow'
    },
  ],
};

/// Descriptor for `UpdateWorkflowResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List updateWorkflowResponseDescriptor =
    $convert.base64Decode(
        'ChZVcGRhdGVXb3JrZmxvd1Jlc3BvbnNlEkcKCHdvcmtmbG93GAEgASgLMisuazFzMC5zeXN0ZW'
        '0ud29ya2Zsb3cudjEuV29ya2Zsb3dEZWZpbml0aW9uUgh3b3JrZmxvdw==');

@$core.Deprecated('Use deleteWorkflowRequestDescriptor instead')
const DeleteWorkflowRequest$json = {
  '1': 'DeleteWorkflowRequest',
  '2': [
    {'1': 'workflow_id', '3': 1, '4': 1, '5': 9, '10': 'workflowId'},
  ],
};

/// Descriptor for `DeleteWorkflowRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List deleteWorkflowRequestDescriptor = $convert.base64Decode(
    'ChVEZWxldGVXb3JrZmxvd1JlcXVlc3QSHwoLd29ya2Zsb3dfaWQYASABKAlSCndvcmtmbG93SW'
    'Q=');

@$core.Deprecated('Use deleteWorkflowResponseDescriptor instead')
const DeleteWorkflowResponse$json = {
  '1': 'DeleteWorkflowResponse',
  '2': [
    {'1': 'success', '3': 1, '4': 1, '5': 8, '10': 'success'},
    {'1': 'message', '3': 2, '4': 1, '5': 9, '10': 'message'},
  ],
};

/// Descriptor for `DeleteWorkflowResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List deleteWorkflowResponseDescriptor =
    $convert.base64Decode(
        'ChZEZWxldGVXb3JrZmxvd1Jlc3BvbnNlEhgKB3N1Y2Nlc3MYASABKAhSB3N1Y2Nlc3MSGAoHbW'
        'Vzc2FnZRgCIAEoCVIHbWVzc2FnZQ==');

@$core.Deprecated('Use startInstanceRequestDescriptor instead')
const StartInstanceRequest$json = {
  '1': 'StartInstanceRequest',
  '2': [
    {'1': 'workflow_id', '3': 1, '4': 1, '5': 9, '10': 'workflowId'},
    {'1': 'title', '3': 2, '4': 1, '5': 9, '10': 'title'},
    {'1': 'initiator_id', '3': 3, '4': 1, '5': 9, '10': 'initiatorId'},
    {'1': 'context_json', '3': 4, '4': 1, '5': 12, '10': 'contextJson'},
  ],
};

/// Descriptor for `StartInstanceRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List startInstanceRequestDescriptor = $convert.base64Decode(
    'ChRTdGFydEluc3RhbmNlUmVxdWVzdBIfCgt3b3JrZmxvd19pZBgBIAEoCVIKd29ya2Zsb3dJZB'
    'IUCgV0aXRsZRgCIAEoCVIFdGl0bGUSIQoMaW5pdGlhdG9yX2lkGAMgASgJUgtpbml0aWF0b3JJ'
    'ZBIhCgxjb250ZXh0X2pzb24YBCABKAxSC2NvbnRleHRKc29u');

@$core.Deprecated('Use startInstanceResponseDescriptor instead')
const StartInstanceResponse$json = {
  '1': 'StartInstanceResponse',
  '2': [
    {'1': 'instance_id', '3': 1, '4': 1, '5': 9, '10': 'instanceId'},
    {'1': 'status', '3': 2, '4': 1, '5': 9, '10': 'status'},
    {
      '1': 'current_step_id',
      '3': 3,
      '4': 1,
      '5': 9,
      '9': 0,
      '10': 'currentStepId',
      '17': true
    },
    {
      '1': 'started_at',
      '3': 4,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.common.v1.Timestamp',
      '10': 'startedAt'
    },
    {'1': 'workflow_id', '3': 5, '4': 1, '5': 9, '10': 'workflowId'},
    {'1': 'workflow_name', '3': 6, '4': 1, '5': 9, '10': 'workflowName'},
    {'1': 'title', '3': 7, '4': 1, '5': 9, '10': 'title'},
    {'1': 'initiator_id', '3': 8, '4': 1, '5': 9, '10': 'initiatorId'},
    {'1': 'context_json', '3': 9, '4': 1, '5': 12, '10': 'contextJson'},
  ],
  '8': [
    {'1': '_current_step_id'},
  ],
};

/// Descriptor for `StartInstanceResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List startInstanceResponseDescriptor = $convert.base64Decode(
    'ChVTdGFydEluc3RhbmNlUmVzcG9uc2USHwoLaW5zdGFuY2VfaWQYASABKAlSCmluc3RhbmNlSW'
    'QSFgoGc3RhdHVzGAIgASgJUgZzdGF0dXMSKwoPY3VycmVudF9zdGVwX2lkGAMgASgJSABSDWN1'
    'cnJlbnRTdGVwSWSIAQESPwoKc3RhcnRlZF9hdBgEIAEoCzIgLmsxczAuc3lzdGVtLmNvbW1vbi'
    '52MS5UaW1lc3RhbXBSCXN0YXJ0ZWRBdBIfCgt3b3JrZmxvd19pZBgFIAEoCVIKd29ya2Zsb3dJ'
    'ZBIjCg13b3JrZmxvd19uYW1lGAYgASgJUgx3b3JrZmxvd05hbWUSFAoFdGl0bGUYByABKAlSBX'
    'RpdGxlEiEKDGluaXRpYXRvcl9pZBgIIAEoCVILaW5pdGlhdG9ySWQSIQoMY29udGV4dF9qc29u'
    'GAkgASgMUgtjb250ZXh0SnNvbkISChBfY3VycmVudF9zdGVwX2lk');

@$core.Deprecated('Use getInstanceRequestDescriptor instead')
const GetInstanceRequest$json = {
  '1': 'GetInstanceRequest',
  '2': [
    {'1': 'instance_id', '3': 1, '4': 1, '5': 9, '10': 'instanceId'},
  ],
};

/// Descriptor for `GetInstanceRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List getInstanceRequestDescriptor = $convert.base64Decode(
    'ChJHZXRJbnN0YW5jZVJlcXVlc3QSHwoLaW5zdGFuY2VfaWQYASABKAlSCmluc3RhbmNlSWQ=');

@$core.Deprecated('Use getInstanceResponseDescriptor instead')
const GetInstanceResponse$json = {
  '1': 'GetInstanceResponse',
  '2': [
    {
      '1': 'instance',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.workflow.v1.WorkflowInstance',
      '10': 'instance'
    },
  ],
};

/// Descriptor for `GetInstanceResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List getInstanceResponseDescriptor = $convert.base64Decode(
    'ChNHZXRJbnN0YW5jZVJlc3BvbnNlEkUKCGluc3RhbmNlGAEgASgLMikuazFzMC5zeXN0ZW0ud2'
    '9ya2Zsb3cudjEuV29ya2Zsb3dJbnN0YW5jZVIIaW5zdGFuY2U=');

@$core.Deprecated('Use listInstancesRequestDescriptor instead')
const ListInstancesRequest$json = {
  '1': 'ListInstancesRequest',
  '2': [
    {'1': 'status', '3': 1, '4': 1, '5': 9, '10': 'status'},
    {'1': 'workflow_id', '3': 2, '4': 1, '5': 9, '10': 'workflowId'},
    {'1': 'initiator_id', '3': 3, '4': 1, '5': 9, '10': 'initiatorId'},
    {
      '1': 'pagination',
      '3': 4,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.common.v1.Pagination',
      '10': 'pagination'
    },
  ],
};

/// Descriptor for `ListInstancesRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List listInstancesRequestDescriptor = $convert.base64Decode(
    'ChRMaXN0SW5zdGFuY2VzUmVxdWVzdBIWCgZzdGF0dXMYASABKAlSBnN0YXR1cxIfCgt3b3JrZm'
    'xvd19pZBgCIAEoCVIKd29ya2Zsb3dJZBIhCgxpbml0aWF0b3JfaWQYAyABKAlSC2luaXRpYXRv'
    'cklkEkEKCnBhZ2luYXRpb24YBCABKAsyIS5rMXMwLnN5c3RlbS5jb21tb24udjEuUGFnaW5hdG'
    'lvblIKcGFnaW5hdGlvbg==');

@$core.Deprecated('Use listInstancesResponseDescriptor instead')
const ListInstancesResponse$json = {
  '1': 'ListInstancesResponse',
  '2': [
    {
      '1': 'instances',
      '3': 1,
      '4': 3,
      '5': 11,
      '6': '.k1s0.system.workflow.v1.WorkflowInstance',
      '10': 'instances'
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

/// Descriptor for `ListInstancesResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List listInstancesResponseDescriptor = $convert.base64Decode(
    'ChVMaXN0SW5zdGFuY2VzUmVzcG9uc2USRwoJaW5zdGFuY2VzGAEgAygLMikuazFzMC5zeXN0ZW'
    '0ud29ya2Zsb3cudjEuV29ya2Zsb3dJbnN0YW5jZVIJaW5zdGFuY2VzEkcKCnBhZ2luYXRpb24Y'
    'AiABKAsyJy5rMXMwLnN5c3RlbS5jb21tb24udjEuUGFnaW5hdGlvblJlc3VsdFIKcGFnaW5hdG'
    'lvbg==');

@$core.Deprecated('Use cancelInstanceRequestDescriptor instead')
const CancelInstanceRequest$json = {
  '1': 'CancelInstanceRequest',
  '2': [
    {'1': 'instance_id', '3': 1, '4': 1, '5': 9, '10': 'instanceId'},
    {'1': 'reason', '3': 2, '4': 1, '5': 9, '9': 0, '10': 'reason', '17': true},
  ],
  '8': [
    {'1': '_reason'},
  ],
};

/// Descriptor for `CancelInstanceRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List cancelInstanceRequestDescriptor = $convert.base64Decode(
    'ChVDYW5jZWxJbnN0YW5jZVJlcXVlc3QSHwoLaW5zdGFuY2VfaWQYASABKAlSCmluc3RhbmNlSW'
    'QSGwoGcmVhc29uGAIgASgJSABSBnJlYXNvbogBAUIJCgdfcmVhc29u');

@$core.Deprecated('Use cancelInstanceResponseDescriptor instead')
const CancelInstanceResponse$json = {
  '1': 'CancelInstanceResponse',
  '2': [
    {
      '1': 'instance',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.workflow.v1.WorkflowInstance',
      '10': 'instance'
    },
  ],
};

/// Descriptor for `CancelInstanceResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List cancelInstanceResponseDescriptor =
    $convert.base64Decode(
        'ChZDYW5jZWxJbnN0YW5jZVJlc3BvbnNlEkUKCGluc3RhbmNlGAEgASgLMikuazFzMC5zeXN0ZW'
        '0ud29ya2Zsb3cudjEuV29ya2Zsb3dJbnN0YW5jZVIIaW5zdGFuY2U=');

@$core.Deprecated('Use workflowTaskDescriptor instead')
const WorkflowTask$json = {
  '1': 'WorkflowTask',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 9, '10': 'id'},
    {'1': 'instance_id', '3': 2, '4': 1, '5': 9, '10': 'instanceId'},
    {'1': 'step_id', '3': 3, '4': 1, '5': 9, '10': 'stepId'},
    {'1': 'step_name', '3': 4, '4': 1, '5': 9, '10': 'stepName'},
    {
      '1': 'assignee_id',
      '3': 5,
      '4': 1,
      '5': 9,
      '9': 0,
      '10': 'assigneeId',
      '17': true
    },
    {'1': 'status', '3': 6, '4': 1, '5': 9, '10': 'status'},
    {
      '1': 'due_at',
      '3': 7,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.common.v1.Timestamp',
      '9': 1,
      '10': 'dueAt',
      '17': true
    },
    {
      '1': 'comment',
      '3': 8,
      '4': 1,
      '5': 9,
      '9': 2,
      '10': 'comment',
      '17': true
    },
    {
      '1': 'actor_id',
      '3': 9,
      '4': 1,
      '5': 9,
      '9': 3,
      '10': 'actorId',
      '17': true
    },
    {
      '1': 'decided_at',
      '3': 10,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.common.v1.Timestamp',
      '9': 4,
      '10': 'decidedAt',
      '17': true
    },
    {
      '1': 'created_at',
      '3': 11,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.common.v1.Timestamp',
      '10': 'createdAt'
    },
    {
      '1': 'updated_at',
      '3': 12,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.common.v1.Timestamp',
      '10': 'updatedAt'
    },
  ],
  '8': [
    {'1': '_assignee_id'},
    {'1': '_due_at'},
    {'1': '_comment'},
    {'1': '_actor_id'},
    {'1': '_decided_at'},
  ],
};

/// Descriptor for `WorkflowTask`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List workflowTaskDescriptor = $convert.base64Decode(
    'CgxXb3JrZmxvd1Rhc2sSDgoCaWQYASABKAlSAmlkEh8KC2luc3RhbmNlX2lkGAIgASgJUgppbn'
    'N0YW5jZUlkEhcKB3N0ZXBfaWQYAyABKAlSBnN0ZXBJZBIbCglzdGVwX25hbWUYBCABKAlSCHN0'
    'ZXBOYW1lEiQKC2Fzc2lnbmVlX2lkGAUgASgJSABSCmFzc2lnbmVlSWSIAQESFgoGc3RhdHVzGA'
    'YgASgJUgZzdGF0dXMSPAoGZHVlX2F0GAcgASgLMiAuazFzMC5zeXN0ZW0uY29tbW9uLnYxLlRp'
    'bWVzdGFtcEgBUgVkdWVBdIgBARIdCgdjb21tZW50GAggASgJSAJSB2NvbW1lbnSIAQESHgoIYW'
    'N0b3JfaWQYCSABKAlIA1IHYWN0b3JJZIgBARJECgpkZWNpZGVkX2F0GAogASgLMiAuazFzMC5z'
    'eXN0ZW0uY29tbW9uLnYxLlRpbWVzdGFtcEgEUglkZWNpZGVkQXSIAQESPwoKY3JlYXRlZF9hdB'
    'gLIAEoCzIgLmsxczAuc3lzdGVtLmNvbW1vbi52MS5UaW1lc3RhbXBSCWNyZWF0ZWRBdBI/Cgp1'
    'cGRhdGVkX2F0GAwgASgLMiAuazFzMC5zeXN0ZW0uY29tbW9uLnYxLlRpbWVzdGFtcFIJdXBkYX'
    'RlZEF0Qg4KDF9hc3NpZ25lZV9pZEIJCgdfZHVlX2F0QgoKCF9jb21tZW50QgsKCV9hY3Rvcl9p'
    'ZEINCgtfZGVjaWRlZF9hdA==');

@$core.Deprecated('Use listTasksRequestDescriptor instead')
const ListTasksRequest$json = {
  '1': 'ListTasksRequest',
  '2': [
    {'1': 'assignee_id', '3': 1, '4': 1, '5': 9, '10': 'assigneeId'},
    {'1': 'status', '3': 2, '4': 1, '5': 9, '10': 'status'},
    {'1': 'instance_id', '3': 3, '4': 1, '5': 9, '10': 'instanceId'},
    {'1': 'overdue_only', '3': 4, '4': 1, '5': 8, '10': 'overdueOnly'},
    {
      '1': 'pagination',
      '3': 5,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.common.v1.Pagination',
      '10': 'pagination'
    },
  ],
};

/// Descriptor for `ListTasksRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List listTasksRequestDescriptor = $convert.base64Decode(
    'ChBMaXN0VGFza3NSZXF1ZXN0Eh8KC2Fzc2lnbmVlX2lkGAEgASgJUgphc3NpZ25lZUlkEhYKBn'
    'N0YXR1cxgCIAEoCVIGc3RhdHVzEh8KC2luc3RhbmNlX2lkGAMgASgJUgppbnN0YW5jZUlkEiEK'
    'DG92ZXJkdWVfb25seRgEIAEoCFILb3ZlcmR1ZU9ubHkSQQoKcGFnaW5hdGlvbhgFIAEoCzIhLm'
    'sxczAuc3lzdGVtLmNvbW1vbi52MS5QYWdpbmF0aW9uUgpwYWdpbmF0aW9u');

@$core.Deprecated('Use listTasksResponseDescriptor instead')
const ListTasksResponse$json = {
  '1': 'ListTasksResponse',
  '2': [
    {
      '1': 'tasks',
      '3': 1,
      '4': 3,
      '5': 11,
      '6': '.k1s0.system.workflow.v1.WorkflowTask',
      '10': 'tasks'
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

/// Descriptor for `ListTasksResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List listTasksResponseDescriptor = $convert.base64Decode(
    'ChFMaXN0VGFza3NSZXNwb25zZRI7CgV0YXNrcxgBIAMoCzIlLmsxczAuc3lzdGVtLndvcmtmbG'
    '93LnYxLldvcmtmbG93VGFza1IFdGFza3MSRwoKcGFnaW5hdGlvbhgCIAEoCzInLmsxczAuc3lz'
    'dGVtLmNvbW1vbi52MS5QYWdpbmF0aW9uUmVzdWx0UgpwYWdpbmF0aW9u');

@$core.Deprecated('Use reassignTaskRequestDescriptor instead')
const ReassignTaskRequest$json = {
  '1': 'ReassignTaskRequest',
  '2': [
    {'1': 'task_id', '3': 1, '4': 1, '5': 9, '10': 'taskId'},
    {'1': 'new_assignee_id', '3': 2, '4': 1, '5': 9, '10': 'newAssigneeId'},
    {'1': 'reason', '3': 3, '4': 1, '5': 9, '9': 0, '10': 'reason', '17': true},
    {'1': 'actor_id', '3': 4, '4': 1, '5': 9, '10': 'actorId'},
  ],
  '8': [
    {'1': '_reason'},
  ],
};

/// Descriptor for `ReassignTaskRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List reassignTaskRequestDescriptor = $convert.base64Decode(
    'ChNSZWFzc2lnblRhc2tSZXF1ZXN0EhcKB3Rhc2tfaWQYASABKAlSBnRhc2tJZBImCg9uZXdfYX'
    'NzaWduZWVfaWQYAiABKAlSDW5ld0Fzc2lnbmVlSWQSGwoGcmVhc29uGAMgASgJSABSBnJlYXNv'
    'bogBARIZCghhY3Rvcl9pZBgEIAEoCVIHYWN0b3JJZEIJCgdfcmVhc29u');

@$core.Deprecated('Use reassignTaskResponseDescriptor instead')
const ReassignTaskResponse$json = {
  '1': 'ReassignTaskResponse',
  '2': [
    {
      '1': 'task',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.workflow.v1.WorkflowTask',
      '10': 'task'
    },
    {
      '1': 'previous_assignee_id',
      '3': 2,
      '4': 1,
      '5': 9,
      '9': 0,
      '10': 'previousAssigneeId',
      '17': true
    },
  ],
  '8': [
    {'1': '_previous_assignee_id'},
  ],
};

/// Descriptor for `ReassignTaskResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List reassignTaskResponseDescriptor = $convert.base64Decode(
    'ChRSZWFzc2lnblRhc2tSZXNwb25zZRI5CgR0YXNrGAEgASgLMiUuazFzMC5zeXN0ZW0ud29ya2'
    'Zsb3cudjEuV29ya2Zsb3dUYXNrUgR0YXNrEjUKFHByZXZpb3VzX2Fzc2lnbmVlX2lkGAIgASgJ'
    'SABSEnByZXZpb3VzQXNzaWduZWVJZIgBAUIXChVfcHJldmlvdXNfYXNzaWduZWVfaWQ=');

@$core.Deprecated('Use workflowInstanceDescriptor instead')
const WorkflowInstance$json = {
  '1': 'WorkflowInstance',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 9, '10': 'id'},
    {'1': 'workflow_id', '3': 2, '4': 1, '5': 9, '10': 'workflowId'},
    {'1': 'workflow_name', '3': 3, '4': 1, '5': 9, '10': 'workflowName'},
    {'1': 'title', '3': 4, '4': 1, '5': 9, '10': 'title'},
    {'1': 'initiator_id', '3': 5, '4': 1, '5': 9, '10': 'initiatorId'},
    {
      '1': 'current_step_id',
      '3': 6,
      '4': 1,
      '5': 9,
      '9': 0,
      '10': 'currentStepId',
      '17': true
    },
    {'1': 'status', '3': 7, '4': 1, '5': 9, '10': 'status'},
    {'1': 'context_json', '3': 8, '4': 1, '5': 12, '10': 'contextJson'},
    {
      '1': 'started_at',
      '3': 9,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.common.v1.Timestamp',
      '10': 'startedAt'
    },
    {
      '1': 'completed_at',
      '3': 10,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.common.v1.Timestamp',
      '9': 1,
      '10': 'completedAt',
      '17': true
    },
    {
      '1': 'created_at',
      '3': 11,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.common.v1.Timestamp',
      '9': 2,
      '10': 'createdAt',
      '17': true
    },
  ],
  '8': [
    {'1': '_current_step_id'},
    {'1': '_completed_at'},
    {'1': '_created_at'},
  ],
};

/// Descriptor for `WorkflowInstance`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List workflowInstanceDescriptor = $convert.base64Decode(
    'ChBXb3JrZmxvd0luc3RhbmNlEg4KAmlkGAEgASgJUgJpZBIfCgt3b3JrZmxvd19pZBgCIAEoCV'
    'IKd29ya2Zsb3dJZBIjCg13b3JrZmxvd19uYW1lGAMgASgJUgx3b3JrZmxvd05hbWUSFAoFdGl0'
    'bGUYBCABKAlSBXRpdGxlEiEKDGluaXRpYXRvcl9pZBgFIAEoCVILaW5pdGlhdG9ySWQSKwoPY3'
    'VycmVudF9zdGVwX2lkGAYgASgJSABSDWN1cnJlbnRTdGVwSWSIAQESFgoGc3RhdHVzGAcgASgJ'
    'UgZzdGF0dXMSIQoMY29udGV4dF9qc29uGAggASgMUgtjb250ZXh0SnNvbhI/CgpzdGFydGVkX2'
    'F0GAkgASgLMiAuazFzMC5zeXN0ZW0uY29tbW9uLnYxLlRpbWVzdGFtcFIJc3RhcnRlZEF0EkgK'
    'DGNvbXBsZXRlZF9hdBgKIAEoCzIgLmsxczAuc3lzdGVtLmNvbW1vbi52MS5UaW1lc3RhbXBIAV'
    'ILY29tcGxldGVkQXSIAQESRAoKY3JlYXRlZF9hdBgLIAEoCzIgLmsxczAuc3lzdGVtLmNvbW1v'
    'bi52MS5UaW1lc3RhbXBIAlIJY3JlYXRlZEF0iAEBQhIKEF9jdXJyZW50X3N0ZXBfaWRCDwoNX2'
    'NvbXBsZXRlZF9hdEINCgtfY3JlYXRlZF9hdA==');

@$core.Deprecated('Use approveTaskRequestDescriptor instead')
const ApproveTaskRequest$json = {
  '1': 'ApproveTaskRequest',
  '2': [
    {'1': 'task_id', '3': 1, '4': 1, '5': 9, '10': 'taskId'},
    {'1': 'actor_id', '3': 2, '4': 1, '5': 9, '10': 'actorId'},
    {
      '1': 'comment',
      '3': 3,
      '4': 1,
      '5': 9,
      '9': 0,
      '10': 'comment',
      '17': true
    },
  ],
  '8': [
    {'1': '_comment'},
  ],
};

/// Descriptor for `ApproveTaskRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List approveTaskRequestDescriptor = $convert.base64Decode(
    'ChJBcHByb3ZlVGFza1JlcXVlc3QSFwoHdGFza19pZBgBIAEoCVIGdGFza0lkEhkKCGFjdG9yX2'
    'lkGAIgASgJUgdhY3RvcklkEh0KB2NvbW1lbnQYAyABKAlIAFIHY29tbWVudIgBAUIKCghfY29t'
    'bWVudA==');

@$core.Deprecated('Use approveTaskResponseDescriptor instead')
const ApproveTaskResponse$json = {
  '1': 'ApproveTaskResponse',
  '2': [
    {'1': 'task_id', '3': 1, '4': 1, '5': 9, '10': 'taskId'},
    {'1': 'status', '3': 2, '4': 1, '5': 9, '10': 'status'},
    {
      '1': 'next_task_id',
      '3': 3,
      '4': 1,
      '5': 9,
      '9': 0,
      '10': 'nextTaskId',
      '17': true
    },
    {'1': 'instance_status', '3': 4, '4': 1, '5': 9, '10': 'instanceStatus'},
  ],
  '8': [
    {'1': '_next_task_id'},
  ],
};

/// Descriptor for `ApproveTaskResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List approveTaskResponseDescriptor = $convert.base64Decode(
    'ChNBcHByb3ZlVGFza1Jlc3BvbnNlEhcKB3Rhc2tfaWQYASABKAlSBnRhc2tJZBIWCgZzdGF0dX'
    'MYAiABKAlSBnN0YXR1cxIlCgxuZXh0X3Rhc2tfaWQYAyABKAlIAFIKbmV4dFRhc2tJZIgBARIn'
    'Cg9pbnN0YW5jZV9zdGF0dXMYBCABKAlSDmluc3RhbmNlU3RhdHVzQg8KDV9uZXh0X3Rhc2tfaW'
    'Q=');

@$core.Deprecated('Use rejectTaskRequestDescriptor instead')
const RejectTaskRequest$json = {
  '1': 'RejectTaskRequest',
  '2': [
    {'1': 'task_id', '3': 1, '4': 1, '5': 9, '10': 'taskId'},
    {'1': 'actor_id', '3': 2, '4': 1, '5': 9, '10': 'actorId'},
    {
      '1': 'comment',
      '3': 3,
      '4': 1,
      '5': 9,
      '9': 0,
      '10': 'comment',
      '17': true
    },
  ],
  '8': [
    {'1': '_comment'},
  ],
};

/// Descriptor for `RejectTaskRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List rejectTaskRequestDescriptor = $convert.base64Decode(
    'ChFSZWplY3RUYXNrUmVxdWVzdBIXCgd0YXNrX2lkGAEgASgJUgZ0YXNrSWQSGQoIYWN0b3JfaW'
    'QYAiABKAlSB2FjdG9ySWQSHQoHY29tbWVudBgDIAEoCUgAUgdjb21tZW50iAEBQgoKCF9jb21t'
    'ZW50');

@$core.Deprecated('Use rejectTaskResponseDescriptor instead')
const RejectTaskResponse$json = {
  '1': 'RejectTaskResponse',
  '2': [
    {'1': 'task_id', '3': 1, '4': 1, '5': 9, '10': 'taskId'},
    {'1': 'status', '3': 2, '4': 1, '5': 9, '10': 'status'},
    {
      '1': 'next_task_id',
      '3': 3,
      '4': 1,
      '5': 9,
      '9': 0,
      '10': 'nextTaskId',
      '17': true
    },
    {'1': 'instance_status', '3': 4, '4': 1, '5': 9, '10': 'instanceStatus'},
  ],
  '8': [
    {'1': '_next_task_id'},
  ],
};

/// Descriptor for `RejectTaskResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List rejectTaskResponseDescriptor = $convert.base64Decode(
    'ChJSZWplY3RUYXNrUmVzcG9uc2USFwoHdGFza19pZBgBIAEoCVIGdGFza0lkEhYKBnN0YXR1cx'
    'gCIAEoCVIGc3RhdHVzEiUKDG5leHRfdGFza19pZBgDIAEoCUgAUgpuZXh0VGFza0lkiAEBEicK'
    'D2luc3RhbmNlX3N0YXR1cxgEIAEoCVIOaW5zdGFuY2VTdGF0dXNCDwoNX25leHRfdGFza19pZA'
    '==');
