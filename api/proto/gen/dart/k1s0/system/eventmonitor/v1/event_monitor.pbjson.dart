// This is a generated file - do not edit.
//
// Generated from k1s0/system/eventmonitor/v1/event_monitor.proto.

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

@$core.Deprecated('Use eventRecordDescriptor instead')
const EventRecord$json = {
  '1': 'EventRecord',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 9, '10': 'id'},
    {'1': 'correlation_id', '3': 2, '4': 1, '5': 9, '10': 'correlationId'},
    {'1': 'event_type', '3': 3, '4': 1, '5': 9, '10': 'eventType'},
    {'1': 'source', '3': 4, '4': 1, '5': 9, '10': 'source'},
    {'1': 'domain', '3': 5, '4': 1, '5': 9, '10': 'domain'},
    {'1': 'trace_id', '3': 6, '4': 1, '5': 9, '10': 'traceId'},
    {
      '1': 'timestamp',
      '3': 7,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.common.v1.Timestamp',
      '10': 'timestamp'
    },
    {
      '1': 'flow_id',
      '3': 8,
      '4': 1,
      '5': 9,
      '9': 0,
      '10': 'flowId',
      '17': true
    },
    {
      '1': 'flow_step_index',
      '3': 9,
      '4': 1,
      '5': 5,
      '9': 1,
      '10': 'flowStepIndex',
      '17': true
    },
    {'1': 'status', '3': 10, '4': 1, '5': 9, '10': 'status'},
  ],
  '8': [
    {'1': '_flow_id'},
    {'1': '_flow_step_index'},
  ],
};

/// Descriptor for `EventRecord`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List eventRecordDescriptor = $convert.base64Decode(
    'CgtFdmVudFJlY29yZBIOCgJpZBgBIAEoCVICaWQSJQoOY29ycmVsYXRpb25faWQYAiABKAlSDW'
    'NvcnJlbGF0aW9uSWQSHQoKZXZlbnRfdHlwZRgDIAEoCVIJZXZlbnRUeXBlEhYKBnNvdXJjZRgE'
    'IAEoCVIGc291cmNlEhYKBmRvbWFpbhgFIAEoCVIGZG9tYWluEhkKCHRyYWNlX2lkGAYgASgJUg'
    'd0cmFjZUlkEj4KCXRpbWVzdGFtcBgHIAEoCzIgLmsxczAuc3lzdGVtLmNvbW1vbi52MS5UaW1l'
    'c3RhbXBSCXRpbWVzdGFtcBIcCgdmbG93X2lkGAggASgJSABSBmZsb3dJZIgBARIrCg9mbG93X3'
    'N0ZXBfaW5kZXgYCSABKAVIAVINZmxvd1N0ZXBJbmRleIgBARIWCgZzdGF0dXMYCiABKAlSBnN0'
    'YXR1c0IKCghfZmxvd19pZEISChBfZmxvd19zdGVwX2luZGV4');

@$core.Deprecated('Use listEventsRequestDescriptor instead')
const ListEventsRequest$json = {
  '1': 'ListEventsRequest',
  '2': [
    {
      '1': 'pagination',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.common.v1.Pagination',
      '10': 'pagination'
    },
    {'1': 'domain', '3': 2, '4': 1, '5': 9, '9': 0, '10': 'domain', '17': true},
    {
      '1': 'event_type',
      '3': 3,
      '4': 1,
      '5': 9,
      '9': 1,
      '10': 'eventType',
      '17': true
    },
    {'1': 'source', '3': 4, '4': 1, '5': 9, '9': 2, '10': 'source', '17': true},
    {
      '1': 'from',
      '3': 5,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.common.v1.Timestamp',
      '9': 3,
      '10': 'from',
      '17': true
    },
    {
      '1': 'to',
      '3': 6,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.common.v1.Timestamp',
      '9': 4,
      '10': 'to',
      '17': true
    },
    {'1': 'status', '3': 7, '4': 1, '5': 9, '9': 5, '10': 'status', '17': true},
  ],
  '8': [
    {'1': '_domain'},
    {'1': '_event_type'},
    {'1': '_source'},
    {'1': '_from'},
    {'1': '_to'},
    {'1': '_status'},
  ],
};

/// Descriptor for `ListEventsRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List listEventsRequestDescriptor = $convert.base64Decode(
    'ChFMaXN0RXZlbnRzUmVxdWVzdBJBCgpwYWdpbmF0aW9uGAEgASgLMiEuazFzMC5zeXN0ZW0uY2'
    '9tbW9uLnYxLlBhZ2luYXRpb25SCnBhZ2luYXRpb24SGwoGZG9tYWluGAIgASgJSABSBmRvbWFp'
    'bogBARIiCgpldmVudF90eXBlGAMgASgJSAFSCWV2ZW50VHlwZYgBARIbCgZzb3VyY2UYBCABKA'
    'lIAlIGc291cmNliAEBEjkKBGZyb20YBSABKAsyIC5rMXMwLnN5c3RlbS5jb21tb24udjEuVGlt'
    'ZXN0YW1wSANSBGZyb22IAQESNQoCdG8YBiABKAsyIC5rMXMwLnN5c3RlbS5jb21tb24udjEuVG'
    'ltZXN0YW1wSARSAnRviAEBEhsKBnN0YXR1cxgHIAEoCUgFUgZzdGF0dXOIAQFCCQoHX2RvbWFp'
    'bkINCgtfZXZlbnRfdHlwZUIJCgdfc291cmNlQgcKBV9mcm9tQgUKA190b0IJCgdfc3RhdHVz');

@$core.Deprecated('Use listEventsResponseDescriptor instead')
const ListEventsResponse$json = {
  '1': 'ListEventsResponse',
  '2': [
    {
      '1': 'events',
      '3': 1,
      '4': 3,
      '5': 11,
      '6': '.k1s0.system.eventmonitor.v1.EventRecord',
      '10': 'events'
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

/// Descriptor for `ListEventsResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List listEventsResponseDescriptor = $convert.base64Decode(
    'ChJMaXN0RXZlbnRzUmVzcG9uc2USQAoGZXZlbnRzGAEgAygLMiguazFzMC5zeXN0ZW0uZXZlbn'
    'Rtb25pdG9yLnYxLkV2ZW50UmVjb3JkUgZldmVudHMSRwoKcGFnaW5hdGlvbhgCIAEoCzInLmsx'
    'czAuc3lzdGVtLmNvbW1vbi52MS5QYWdpbmF0aW9uUmVzdWx0UgpwYWdpbmF0aW9u');

@$core.Deprecated('Use traceByCorrelationRequestDescriptor instead')
const TraceByCorrelationRequest$json = {
  '1': 'TraceByCorrelationRequest',
  '2': [
    {'1': 'correlation_id', '3': 1, '4': 1, '5': 9, '10': 'correlationId'},
  ],
};

/// Descriptor for `TraceByCorrelationRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List traceByCorrelationRequestDescriptor =
    $convert.base64Decode(
        'ChlUcmFjZUJ5Q29ycmVsYXRpb25SZXF1ZXN0EiUKDmNvcnJlbGF0aW9uX2lkGAEgASgJUg1jb3'
        'JyZWxhdGlvbklk');

@$core.Deprecated('Use traceEventDescriptor instead')
const TraceEvent$json = {
  '1': 'TraceEvent',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 9, '10': 'id'},
    {'1': 'event_type', '3': 2, '4': 1, '5': 9, '10': 'eventType'},
    {'1': 'source', '3': 3, '4': 1, '5': 9, '10': 'source'},
    {
      '1': 'timestamp',
      '3': 4,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.common.v1.Timestamp',
      '10': 'timestamp'
    },
    {'1': 'step_index', '3': 5, '4': 1, '5': 5, '10': 'stepIndex'},
    {'1': 'status', '3': 6, '4': 1, '5': 9, '10': 'status'},
    {
      '1': 'duration_from_previous_ms',
      '3': 7,
      '4': 1,
      '5': 3,
      '10': 'durationFromPreviousMs'
    },
  ],
};

/// Descriptor for `TraceEvent`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List traceEventDescriptor = $convert.base64Decode(
    'CgpUcmFjZUV2ZW50Eg4KAmlkGAEgASgJUgJpZBIdCgpldmVudF90eXBlGAIgASgJUglldmVudF'
    'R5cGUSFgoGc291cmNlGAMgASgJUgZzb3VyY2USPgoJdGltZXN0YW1wGAQgASgLMiAuazFzMC5z'
    'eXN0ZW0uY29tbW9uLnYxLlRpbWVzdGFtcFIJdGltZXN0YW1wEh0KCnN0ZXBfaW5kZXgYBSABKA'
    'VSCXN0ZXBJbmRleBIWCgZzdGF0dXMYBiABKAlSBnN0YXR1cxI5ChlkdXJhdGlvbl9mcm9tX3By'
    'ZXZpb3VzX21zGAcgASgDUhZkdXJhdGlvbkZyb21QcmV2aW91c01z');

@$core.Deprecated('Use pendingStepDescriptor instead')
const PendingStep$json = {
  '1': 'PendingStep',
  '2': [
    {'1': 'event_type', '3': 1, '4': 1, '5': 9, '10': 'eventType'},
    {'1': 'source', '3': 2, '4': 1, '5': 9, '10': 'source'},
    {'1': 'step_index', '3': 3, '4': 1, '5': 5, '10': 'stepIndex'},
    {'1': 'timeout_seconds', '3': 4, '4': 1, '5': 5, '10': 'timeoutSeconds'},
    {
      '1': 'waiting_since_seconds',
      '3': 5,
      '4': 1,
      '5': 3,
      '10': 'waitingSinceSeconds'
    },
  ],
};

/// Descriptor for `PendingStep`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List pendingStepDescriptor = $convert.base64Decode(
    'CgtQZW5kaW5nU3RlcBIdCgpldmVudF90eXBlGAEgASgJUglldmVudFR5cGUSFgoGc291cmNlGA'
    'IgASgJUgZzb3VyY2USHQoKc3RlcF9pbmRleBgDIAEoBVIJc3RlcEluZGV4EicKD3RpbWVvdXRf'
    'c2Vjb25kcxgEIAEoBVIOdGltZW91dFNlY29uZHMSMgoVd2FpdGluZ19zaW5jZV9zZWNvbmRzGA'
    'UgASgDUhN3YWl0aW5nU2luY2VTZWNvbmRz');

@$core.Deprecated('Use flowSummaryDescriptor instead')
const FlowSummary$json = {
  '1': 'FlowSummary',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 9, '10': 'id'},
    {'1': 'name', '3': 2, '4': 1, '5': 9, '10': 'name'},
    {'1': 'status', '3': 3, '4': 1, '5': 9, '10': 'status'},
    {
      '1': 'started_at',
      '3': 4,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.common.v1.Timestamp',
      '10': 'startedAt'
    },
    {'1': 'elapsed_seconds', '3': 5, '4': 1, '5': 3, '10': 'elapsedSeconds'},
  ],
};

/// Descriptor for `FlowSummary`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List flowSummaryDescriptor = $convert.base64Decode(
    'CgtGbG93U3VtbWFyeRIOCgJpZBgBIAEoCVICaWQSEgoEbmFtZRgCIAEoCVIEbmFtZRIWCgZzdG'
    'F0dXMYAyABKAlSBnN0YXR1cxI/CgpzdGFydGVkX2F0GAQgASgLMiAuazFzMC5zeXN0ZW0uY29t'
    'bW9uLnYxLlRpbWVzdGFtcFIJc3RhcnRlZEF0EicKD2VsYXBzZWRfc2Vjb25kcxgFIAEoA1IOZW'
    'xhcHNlZFNlY29uZHM=');

@$core.Deprecated('Use traceByCorrelationResponseDescriptor instead')
const TraceByCorrelationResponse$json = {
  '1': 'TraceByCorrelationResponse',
  '2': [
    {'1': 'correlation_id', '3': 1, '4': 1, '5': 9, '10': 'correlationId'},
    {
      '1': 'flow',
      '3': 2,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.eventmonitor.v1.FlowSummary',
      '10': 'flow'
    },
    {
      '1': 'events',
      '3': 3,
      '4': 3,
      '5': 11,
      '6': '.k1s0.system.eventmonitor.v1.TraceEvent',
      '10': 'events'
    },
    {
      '1': 'pending_steps',
      '3': 4,
      '4': 3,
      '5': 11,
      '6': '.k1s0.system.eventmonitor.v1.PendingStep',
      '10': 'pendingSteps'
    },
  ],
};

/// Descriptor for `TraceByCorrelationResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List traceByCorrelationResponseDescriptor = $convert.base64Decode(
    'ChpUcmFjZUJ5Q29ycmVsYXRpb25SZXNwb25zZRIlCg5jb3JyZWxhdGlvbl9pZBgBIAEoCVINY2'
    '9ycmVsYXRpb25JZBI8CgRmbG93GAIgASgLMiguazFzMC5zeXN0ZW0uZXZlbnRtb25pdG9yLnYx'
    'LkZsb3dTdW1tYXJ5UgRmbG93Ej8KBmV2ZW50cxgDIAMoCzInLmsxczAuc3lzdGVtLmV2ZW50bW'
    '9uaXRvci52MS5UcmFjZUV2ZW50UgZldmVudHMSTQoNcGVuZGluZ19zdGVwcxgEIAMoCzIoLmsx'
    'czAuc3lzdGVtLmV2ZW50bW9uaXRvci52MS5QZW5kaW5nU3RlcFIMcGVuZGluZ1N0ZXBz');

@$core.Deprecated('Use flowStepDescriptor instead')
const FlowStep$json = {
  '1': 'FlowStep',
  '2': [
    {'1': 'event_type', '3': 1, '4': 1, '5': 9, '10': 'eventType'},
    {'1': 'source', '3': 2, '4': 1, '5': 9, '10': 'source'},
    {'1': 'timeout_seconds', '3': 3, '4': 1, '5': 5, '10': 'timeoutSeconds'},
    {'1': 'description', '3': 4, '4': 1, '5': 9, '10': 'description'},
  ],
};

/// Descriptor for `FlowStep`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List flowStepDescriptor = $convert.base64Decode(
    'CghGbG93U3RlcBIdCgpldmVudF90eXBlGAEgASgJUglldmVudFR5cGUSFgoGc291cmNlGAIgAS'
    'gJUgZzb3VyY2USJwoPdGltZW91dF9zZWNvbmRzGAMgASgFUg50aW1lb3V0U2Vjb25kcxIgCgtk'
    'ZXNjcmlwdGlvbhgEIAEoCVILZGVzY3JpcHRpb24=');

@$core.Deprecated('Use flowSloDescriptor instead')
const FlowSlo$json = {
  '1': 'FlowSlo',
  '2': [
    {
      '1': 'target_completion_seconds',
      '3': 1,
      '4': 1,
      '5': 5,
      '10': 'targetCompletionSeconds'
    },
    {
      '1': 'target_success_rate',
      '3': 2,
      '4': 1,
      '5': 1,
      '10': 'targetSuccessRate'
    },
    {
      '1': 'alert_on_violation',
      '3': 3,
      '4': 1,
      '5': 8,
      '10': 'alertOnViolation'
    },
  ],
};

/// Descriptor for `FlowSlo`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List flowSloDescriptor = $convert.base64Decode(
    'CgdGbG93U2xvEjoKGXRhcmdldF9jb21wbGV0aW9uX3NlY29uZHMYASABKAVSF3RhcmdldENvbX'
    'BsZXRpb25TZWNvbmRzEi4KE3RhcmdldF9zdWNjZXNzX3JhdGUYAiABKAFSEXRhcmdldFN1Y2Nl'
    'c3NSYXRlEiwKEmFsZXJ0X29uX3Zpb2xhdGlvbhgDIAEoCFIQYWxlcnRPblZpb2xhdGlvbg==');

@$core.Deprecated('Use flowDefinitionDescriptor instead')
const FlowDefinition$json = {
  '1': 'FlowDefinition',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 9, '10': 'id'},
    {'1': 'name', '3': 2, '4': 1, '5': 9, '10': 'name'},
    {'1': 'description', '3': 3, '4': 1, '5': 9, '10': 'description'},
    {'1': 'domain', '3': 4, '4': 1, '5': 9, '10': 'domain'},
    {
      '1': 'steps',
      '3': 5,
      '4': 3,
      '5': 11,
      '6': '.k1s0.system.eventmonitor.v1.FlowStep',
      '10': 'steps'
    },
    {
      '1': 'slo',
      '3': 6,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.eventmonitor.v1.FlowSlo',
      '10': 'slo'
    },
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
  ],
};

/// Descriptor for `FlowDefinition`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List flowDefinitionDescriptor = $convert.base64Decode(
    'Cg5GbG93RGVmaW5pdGlvbhIOCgJpZBgBIAEoCVICaWQSEgoEbmFtZRgCIAEoCVIEbmFtZRIgCg'
    'tkZXNjcmlwdGlvbhgDIAEoCVILZGVzY3JpcHRpb24SFgoGZG9tYWluGAQgASgJUgZkb21haW4S'
    'OwoFc3RlcHMYBSADKAsyJS5rMXMwLnN5c3RlbS5ldmVudG1vbml0b3IudjEuRmxvd1N0ZXBSBX'
    'N0ZXBzEjYKA3NsbxgGIAEoCzIkLmsxczAuc3lzdGVtLmV2ZW50bW9uaXRvci52MS5GbG93U2xv'
    'UgNzbG8SGAoHZW5hYmxlZBgHIAEoCFIHZW5hYmxlZBI/CgpjcmVhdGVkX2F0GAggASgLMiAuaz'
    'FzMC5zeXN0ZW0uY29tbW9uLnYxLlRpbWVzdGFtcFIJY3JlYXRlZEF0Ej8KCnVwZGF0ZWRfYXQY'
    'CSABKAsyIC5rMXMwLnN5c3RlbS5jb21tb24udjEuVGltZXN0YW1wUgl1cGRhdGVkQXQ=');

@$core.Deprecated('Use listFlowsRequestDescriptor instead')
const ListFlowsRequest$json = {
  '1': 'ListFlowsRequest',
  '2': [
    {
      '1': 'pagination',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.common.v1.Pagination',
      '10': 'pagination'
    },
    {'1': 'domain', '3': 2, '4': 1, '5': 9, '9': 0, '10': 'domain', '17': true},
  ],
  '8': [
    {'1': '_domain'},
  ],
};

/// Descriptor for `ListFlowsRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List listFlowsRequestDescriptor = $convert.base64Decode(
    'ChBMaXN0Rmxvd3NSZXF1ZXN0EkEKCnBhZ2luYXRpb24YASABKAsyIS5rMXMwLnN5c3RlbS5jb2'
    '1tb24udjEuUGFnaW5hdGlvblIKcGFnaW5hdGlvbhIbCgZkb21haW4YAiABKAlIAFIGZG9tYWlu'
    'iAEBQgkKB19kb21haW4=');

@$core.Deprecated('Use listFlowsResponseDescriptor instead')
const ListFlowsResponse$json = {
  '1': 'ListFlowsResponse',
  '2': [
    {
      '1': 'flows',
      '3': 1,
      '4': 3,
      '5': 11,
      '6': '.k1s0.system.eventmonitor.v1.FlowDefinition',
      '10': 'flows'
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

/// Descriptor for `ListFlowsResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List listFlowsResponseDescriptor = $convert.base64Decode(
    'ChFMaXN0Rmxvd3NSZXNwb25zZRJBCgVmbG93cxgBIAMoCzIrLmsxczAuc3lzdGVtLmV2ZW50bW'
    '9uaXRvci52MS5GbG93RGVmaW5pdGlvblIFZmxvd3MSRwoKcGFnaW5hdGlvbhgCIAEoCzInLmsx'
    'czAuc3lzdGVtLmNvbW1vbi52MS5QYWdpbmF0aW9uUmVzdWx0UgpwYWdpbmF0aW9u');

@$core.Deprecated('Use getFlowRequestDescriptor instead')
const GetFlowRequest$json = {
  '1': 'GetFlowRequest',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 9, '10': 'id'},
  ],
};

/// Descriptor for `GetFlowRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List getFlowRequestDescriptor =
    $convert.base64Decode('Cg5HZXRGbG93UmVxdWVzdBIOCgJpZBgBIAEoCVICaWQ=');

@$core.Deprecated('Use getFlowResponseDescriptor instead')
const GetFlowResponse$json = {
  '1': 'GetFlowResponse',
  '2': [
    {
      '1': 'flow',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.eventmonitor.v1.FlowDefinition',
      '10': 'flow'
    },
  ],
};

/// Descriptor for `GetFlowResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List getFlowResponseDescriptor = $convert.base64Decode(
    'Cg9HZXRGbG93UmVzcG9uc2USPwoEZmxvdxgBIAEoCzIrLmsxczAuc3lzdGVtLmV2ZW50bW9uaX'
    'Rvci52MS5GbG93RGVmaW5pdGlvblIEZmxvdw==');

@$core.Deprecated('Use createFlowRequestDescriptor instead')
const CreateFlowRequest$json = {
  '1': 'CreateFlowRequest',
  '2': [
    {'1': 'name', '3': 1, '4': 1, '5': 9, '10': 'name'},
    {'1': 'description', '3': 2, '4': 1, '5': 9, '10': 'description'},
    {'1': 'domain', '3': 3, '4': 1, '5': 9, '10': 'domain'},
    {
      '1': 'steps',
      '3': 4,
      '4': 3,
      '5': 11,
      '6': '.k1s0.system.eventmonitor.v1.FlowStep',
      '10': 'steps'
    },
    {
      '1': 'slo',
      '3': 5,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.eventmonitor.v1.FlowSlo',
      '10': 'slo'
    },
  ],
};

/// Descriptor for `CreateFlowRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List createFlowRequestDescriptor = $convert.base64Decode(
    'ChFDcmVhdGVGbG93UmVxdWVzdBISCgRuYW1lGAEgASgJUgRuYW1lEiAKC2Rlc2NyaXB0aW9uGA'
    'IgASgJUgtkZXNjcmlwdGlvbhIWCgZkb21haW4YAyABKAlSBmRvbWFpbhI7CgVzdGVwcxgEIAMo'
    'CzIlLmsxczAuc3lzdGVtLmV2ZW50bW9uaXRvci52MS5GbG93U3RlcFIFc3RlcHMSNgoDc2xvGA'
    'UgASgLMiQuazFzMC5zeXN0ZW0uZXZlbnRtb25pdG9yLnYxLkZsb3dTbG9SA3Nsbw==');

@$core.Deprecated('Use createFlowResponseDescriptor instead')
const CreateFlowResponse$json = {
  '1': 'CreateFlowResponse',
  '2': [
    {
      '1': 'flow',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.eventmonitor.v1.FlowDefinition',
      '10': 'flow'
    },
  ],
};

/// Descriptor for `CreateFlowResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List createFlowResponseDescriptor = $convert.base64Decode(
    'ChJDcmVhdGVGbG93UmVzcG9uc2USPwoEZmxvdxgBIAEoCzIrLmsxczAuc3lzdGVtLmV2ZW50bW'
    '9uaXRvci52MS5GbG93RGVmaW5pdGlvblIEZmxvdw==');

@$core.Deprecated('Use updateFlowRequestDescriptor instead')
const UpdateFlowRequest$json = {
  '1': 'UpdateFlowRequest',
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
      '1': 'steps',
      '3': 3,
      '4': 3,
      '5': 11,
      '6': '.k1s0.system.eventmonitor.v1.FlowStep',
      '10': 'steps'
    },
    {
      '1': 'slo',
      '3': 4,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.eventmonitor.v1.FlowSlo',
      '9': 1,
      '10': 'slo',
      '17': true
    },
    {
      '1': 'enabled',
      '3': 5,
      '4': 1,
      '5': 8,
      '9': 2,
      '10': 'enabled',
      '17': true
    },
  ],
  '8': [
    {'1': '_description'},
    {'1': '_slo'},
    {'1': '_enabled'},
  ],
};

/// Descriptor for `UpdateFlowRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List updateFlowRequestDescriptor = $convert.base64Decode(
    'ChFVcGRhdGVGbG93UmVxdWVzdBIOCgJpZBgBIAEoCVICaWQSJQoLZGVzY3JpcHRpb24YAiABKA'
    'lIAFILZGVzY3JpcHRpb26IAQESOwoFc3RlcHMYAyADKAsyJS5rMXMwLnN5c3RlbS5ldmVudG1v'
    'bml0b3IudjEuRmxvd1N0ZXBSBXN0ZXBzEjsKA3NsbxgEIAEoCzIkLmsxczAuc3lzdGVtLmV2ZW'
    '50bW9uaXRvci52MS5GbG93U2xvSAFSA3Nsb4gBARIdCgdlbmFibGVkGAUgASgISAJSB2VuYWJs'
    'ZWSIAQFCDgoMX2Rlc2NyaXB0aW9uQgYKBF9zbG9CCgoIX2VuYWJsZWQ=');

@$core.Deprecated('Use updateFlowResponseDescriptor instead')
const UpdateFlowResponse$json = {
  '1': 'UpdateFlowResponse',
  '2': [
    {
      '1': 'flow',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.eventmonitor.v1.FlowDefinition',
      '10': 'flow'
    },
  ],
};

/// Descriptor for `UpdateFlowResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List updateFlowResponseDescriptor = $convert.base64Decode(
    'ChJVcGRhdGVGbG93UmVzcG9uc2USPwoEZmxvdxgBIAEoCzIrLmsxczAuc3lzdGVtLmV2ZW50bW'
    '9uaXRvci52MS5GbG93RGVmaW5pdGlvblIEZmxvdw==');

@$core.Deprecated('Use deleteFlowRequestDescriptor instead')
const DeleteFlowRequest$json = {
  '1': 'DeleteFlowRequest',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 9, '10': 'id'},
  ],
};

/// Descriptor for `DeleteFlowRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List deleteFlowRequestDescriptor =
    $convert.base64Decode('ChFEZWxldGVGbG93UmVxdWVzdBIOCgJpZBgBIAEoCVICaWQ=');

@$core.Deprecated('Use deleteFlowResponseDescriptor instead')
const DeleteFlowResponse$json = {
  '1': 'DeleteFlowResponse',
  '2': [
    {'1': 'success', '3': 1, '4': 1, '5': 8, '10': 'success'},
    {'1': 'message', '3': 2, '4': 1, '5': 9, '10': 'message'},
  ],
};

/// Descriptor for `DeleteFlowResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List deleteFlowResponseDescriptor = $convert.base64Decode(
    'ChJEZWxldGVGbG93UmVzcG9uc2USGAoHc3VjY2VzcxgBIAEoCFIHc3VjY2VzcxIYCgdtZXNzYW'
    'dlGAIgASgJUgdtZXNzYWdl');

@$core.Deprecated('Use bottleneckStepDescriptor instead')
const BottleneckStep$json = {
  '1': 'BottleneckStep',
  '2': [
    {'1': 'event_type', '3': 1, '4': 1, '5': 9, '10': 'eventType'},
    {'1': 'step_index', '3': 2, '4': 1, '5': 5, '10': 'stepIndex'},
    {
      '1': 'avg_duration_seconds',
      '3': 3,
      '4': 1,
      '5': 1,
      '10': 'avgDurationSeconds'
    },
    {'1': 'timeout_rate', '3': 4, '4': 1, '5': 1, '10': 'timeoutRate'},
  ],
};

/// Descriptor for `BottleneckStep`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List bottleneckStepDescriptor = $convert.base64Decode(
    'Cg5Cb3R0bGVuZWNrU3RlcBIdCgpldmVudF90eXBlGAEgASgJUglldmVudFR5cGUSHQoKc3RlcF'
    '9pbmRleBgCIAEoBVIJc3RlcEluZGV4EjAKFGF2Z19kdXJhdGlvbl9zZWNvbmRzGAMgASgBUhJh'
    'dmdEdXJhdGlvblNlY29uZHMSIQoMdGltZW91dF9yYXRlGAQgASgBUgt0aW1lb3V0UmF0ZQ==');

@$core.Deprecated('Use sloStatusDescriptor instead')
const SloStatus$json = {
  '1': 'SloStatus',
  '2': [
    {
      '1': 'target_completion_seconds',
      '3': 1,
      '4': 1,
      '5': 5,
      '10': 'targetCompletionSeconds'
    },
    {
      '1': 'target_success_rate',
      '3': 2,
      '4': 1,
      '5': 1,
      '10': 'targetSuccessRate'
    },
    {
      '1': 'current_success_rate',
      '3': 3,
      '4': 1,
      '5': 1,
      '10': 'currentSuccessRate'
    },
    {'1': 'is_violated', '3': 4, '4': 1, '5': 8, '10': 'isViolated'},
    {'1': 'burn_rate', '3': 5, '4': 1, '5': 1, '10': 'burnRate'},
    {
      '1': 'estimated_budget_exhaustion_hours',
      '3': 6,
      '4': 1,
      '5': 1,
      '10': 'estimatedBudgetExhaustionHours'
    },
  ],
};

/// Descriptor for `SloStatus`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List sloStatusDescriptor = $convert.base64Decode(
    'CglTbG9TdGF0dXMSOgoZdGFyZ2V0X2NvbXBsZXRpb25fc2Vjb25kcxgBIAEoBVIXdGFyZ2V0Q2'
    '9tcGxldGlvblNlY29uZHMSLgoTdGFyZ2V0X3N1Y2Nlc3NfcmF0ZRgCIAEoAVIRdGFyZ2V0U3Vj'
    'Y2Vzc1JhdGUSMAoUY3VycmVudF9zdWNjZXNzX3JhdGUYAyABKAFSEmN1cnJlbnRTdWNjZXNzUm'
    'F0ZRIfCgtpc192aW9sYXRlZBgEIAEoCFIKaXNWaW9sYXRlZBIbCglidXJuX3JhdGUYBSABKAFS'
    'CGJ1cm5SYXRlEkkKIWVzdGltYXRlZF9idWRnZXRfZXhoYXVzdGlvbl9ob3VycxgGIAEoAVIeZX'
    'N0aW1hdGVkQnVkZ2V0RXhoYXVzdGlvbkhvdXJz');

@$core.Deprecated('Use flowKpiDescriptor instead')
const FlowKpi$json = {
  '1': 'FlowKpi',
  '2': [
    {'1': 'total_started', '3': 1, '4': 1, '5': 3, '10': 'totalStarted'},
    {'1': 'total_completed', '3': 2, '4': 1, '5': 3, '10': 'totalCompleted'},
    {'1': 'total_failed', '3': 3, '4': 1, '5': 3, '10': 'totalFailed'},
    {'1': 'total_in_progress', '3': 4, '4': 1, '5': 3, '10': 'totalInProgress'},
    {'1': 'completion_rate', '3': 5, '4': 1, '5': 1, '10': 'completionRate'},
    {
      '1': 'avg_duration_seconds',
      '3': 6,
      '4': 1,
      '5': 1,
      '10': 'avgDurationSeconds'
    },
    {
      '1': 'p50_duration_seconds',
      '3': 7,
      '4': 1,
      '5': 1,
      '10': 'p50DurationSeconds'
    },
    {
      '1': 'p95_duration_seconds',
      '3': 8,
      '4': 1,
      '5': 1,
      '10': 'p95DurationSeconds'
    },
    {
      '1': 'p99_duration_seconds',
      '3': 9,
      '4': 1,
      '5': 1,
      '10': 'p99DurationSeconds'
    },
    {
      '1': 'bottleneck_step',
      '3': 10,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.eventmonitor.v1.BottleneckStep',
      '10': 'bottleneckStep'
    },
  ],
};

/// Descriptor for `FlowKpi`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List flowKpiDescriptor = $convert.base64Decode(
    'CgdGbG93S3BpEiMKDXRvdGFsX3N0YXJ0ZWQYASABKANSDHRvdGFsU3RhcnRlZBInCg90b3RhbF'
    '9jb21wbGV0ZWQYAiABKANSDnRvdGFsQ29tcGxldGVkEiEKDHRvdGFsX2ZhaWxlZBgDIAEoA1IL'
    'dG90YWxGYWlsZWQSKgoRdG90YWxfaW5fcHJvZ3Jlc3MYBCABKANSD3RvdGFsSW5Qcm9ncmVzcx'
    'InCg9jb21wbGV0aW9uX3JhdGUYBSABKAFSDmNvbXBsZXRpb25SYXRlEjAKFGF2Z19kdXJhdGlv'
    'bl9zZWNvbmRzGAYgASgBUhJhdmdEdXJhdGlvblNlY29uZHMSMAoUcDUwX2R1cmF0aW9uX3NlY2'
    '9uZHMYByABKAFSEnA1MER1cmF0aW9uU2Vjb25kcxIwChRwOTVfZHVyYXRpb25fc2Vjb25kcxgI'
    'IAEoAVIScDk1RHVyYXRpb25TZWNvbmRzEjAKFHA5OV9kdXJhdGlvbl9zZWNvbmRzGAkgASgBUh'
    'JwOTlEdXJhdGlvblNlY29uZHMSVAoPYm90dGxlbmVja19zdGVwGAogASgLMisuazFzMC5zeXN0'
    'ZW0uZXZlbnRtb25pdG9yLnYxLkJvdHRsZW5lY2tTdGVwUg5ib3R0bGVuZWNrU3RlcA==');

@$core.Deprecated('Use getFlowKpiRequestDescriptor instead')
const GetFlowKpiRequest$json = {
  '1': 'GetFlowKpiRequest',
  '2': [
    {'1': 'flow_id', '3': 1, '4': 1, '5': 9, '10': 'flowId'},
    {'1': 'period', '3': 2, '4': 1, '5': 9, '9': 0, '10': 'period', '17': true},
  ],
  '8': [
    {'1': '_period'},
  ],
};

/// Descriptor for `GetFlowKpiRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List getFlowKpiRequestDescriptor = $convert.base64Decode(
    'ChFHZXRGbG93S3BpUmVxdWVzdBIXCgdmbG93X2lkGAEgASgJUgZmbG93SWQSGwoGcGVyaW9kGA'
    'IgASgJSABSBnBlcmlvZIgBAUIJCgdfcGVyaW9k');

@$core.Deprecated('Use getFlowKpiResponseDescriptor instead')
const GetFlowKpiResponse$json = {
  '1': 'GetFlowKpiResponse',
  '2': [
    {'1': 'flow_id', '3': 1, '4': 1, '5': 9, '10': 'flowId'},
    {'1': 'flow_name', '3': 2, '4': 1, '5': 9, '10': 'flowName'},
    {'1': 'period', '3': 3, '4': 1, '5': 9, '10': 'period'},
    {
      '1': 'kpi',
      '3': 4,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.eventmonitor.v1.FlowKpi',
      '10': 'kpi'
    },
    {
      '1': 'slo_status',
      '3': 5,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.eventmonitor.v1.SloStatus',
      '10': 'sloStatus'
    },
  ],
};

/// Descriptor for `GetFlowKpiResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List getFlowKpiResponseDescriptor = $convert.base64Decode(
    'ChJHZXRGbG93S3BpUmVzcG9uc2USFwoHZmxvd19pZBgBIAEoCVIGZmxvd0lkEhsKCWZsb3dfbm'
    'FtZRgCIAEoCVIIZmxvd05hbWUSFgoGcGVyaW9kGAMgASgJUgZwZXJpb2QSNgoDa3BpGAQgASgL'
    'MiQuazFzMC5zeXN0ZW0uZXZlbnRtb25pdG9yLnYxLkZsb3dLcGlSA2twaRJFCgpzbG9fc3RhdH'
    'VzGAUgASgLMiYuazFzMC5zeXN0ZW0uZXZlbnRtb25pdG9yLnYxLlNsb1N0YXR1c1IJc2xvU3Rh'
    'dHVz');

@$core.Deprecated('Use flowKpiSummaryDescriptor instead')
const FlowKpiSummary$json = {
  '1': 'FlowKpiSummary',
  '2': [
    {'1': 'flow_id', '3': 1, '4': 1, '5': 9, '10': 'flowId'},
    {'1': 'flow_name', '3': 2, '4': 1, '5': 9, '10': 'flowName'},
    {'1': 'domain', '3': 3, '4': 1, '5': 9, '10': 'domain'},
    {'1': 'total_started', '3': 4, '4': 1, '5': 3, '10': 'totalStarted'},
    {'1': 'completion_rate', '3': 5, '4': 1, '5': 1, '10': 'completionRate'},
    {
      '1': 'avg_duration_seconds',
      '3': 6,
      '4': 1,
      '5': 1,
      '10': 'avgDurationSeconds'
    },
    {'1': 'slo_violated', '3': 7, '4': 1, '5': 8, '10': 'sloViolated'},
  ],
};

/// Descriptor for `FlowKpiSummary`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List flowKpiSummaryDescriptor = $convert.base64Decode(
    'Cg5GbG93S3BpU3VtbWFyeRIXCgdmbG93X2lkGAEgASgJUgZmbG93SWQSGwoJZmxvd19uYW1lGA'
    'IgASgJUghmbG93TmFtZRIWCgZkb21haW4YAyABKAlSBmRvbWFpbhIjCg10b3RhbF9zdGFydGVk'
    'GAQgASgDUgx0b3RhbFN0YXJ0ZWQSJwoPY29tcGxldGlvbl9yYXRlGAUgASgBUg5jb21wbGV0aW'
    '9uUmF0ZRIwChRhdmdfZHVyYXRpb25fc2Vjb25kcxgGIAEoAVISYXZnRHVyYXRpb25TZWNvbmRz'
    'EiEKDHNsb192aW9sYXRlZBgHIAEoCFILc2xvVmlvbGF0ZWQ=');

@$core.Deprecated('Use getKpiSummaryRequestDescriptor instead')
const GetKpiSummaryRequest$json = {
  '1': 'GetKpiSummaryRequest',
  '2': [
    {'1': 'period', '3': 1, '4': 1, '5': 9, '9': 0, '10': 'period', '17': true},
  ],
  '8': [
    {'1': '_period'},
  ],
};

/// Descriptor for `GetKpiSummaryRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List getKpiSummaryRequestDescriptor = $convert.base64Decode(
    'ChRHZXRLcGlTdW1tYXJ5UmVxdWVzdBIbCgZwZXJpb2QYASABKAlIAFIGcGVyaW9kiAEBQgkKB1'
    '9wZXJpb2Q=');

@$core.Deprecated('Use getKpiSummaryResponseDescriptor instead')
const GetKpiSummaryResponse$json = {
  '1': 'GetKpiSummaryResponse',
  '2': [
    {'1': 'period', '3': 1, '4': 1, '5': 9, '10': 'period'},
    {
      '1': 'flows',
      '3': 2,
      '4': 3,
      '5': 11,
      '6': '.k1s0.system.eventmonitor.v1.FlowKpiSummary',
      '10': 'flows'
    },
    {'1': 'total_flows', '3': 3, '4': 1, '5': 5, '10': 'totalFlows'},
    {
      '1': 'flows_with_slo_violation',
      '3': 4,
      '4': 1,
      '5': 5,
      '10': 'flowsWithSloViolation'
    },
    {
      '1': 'overall_completion_rate',
      '3': 5,
      '4': 1,
      '5': 1,
      '10': 'overallCompletionRate'
    },
  ],
};

/// Descriptor for `GetKpiSummaryResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List getKpiSummaryResponseDescriptor = $convert.base64Decode(
    'ChVHZXRLcGlTdW1tYXJ5UmVzcG9uc2USFgoGcGVyaW9kGAEgASgJUgZwZXJpb2QSQQoFZmxvd3'
    'MYAiADKAsyKy5rMXMwLnN5c3RlbS5ldmVudG1vbml0b3IudjEuRmxvd0twaVN1bW1hcnlSBWZs'
    'b3dzEh8KC3RvdGFsX2Zsb3dzGAMgASgFUgp0b3RhbEZsb3dzEjcKGGZsb3dzX3dpdGhfc2xvX3'
    'Zpb2xhdGlvbhgEIAEoBVIVZmxvd3NXaXRoU2xvVmlvbGF0aW9uEjYKF292ZXJhbGxfY29tcGxl'
    'dGlvbl9yYXRlGAUgASgBUhVvdmVyYWxsQ29tcGxldGlvblJhdGU=');

@$core.Deprecated('Use getSloStatusRequestDescriptor instead')
const GetSloStatusRequest$json = {
  '1': 'GetSloStatusRequest',
};

/// Descriptor for `GetSloStatusRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List getSloStatusRequestDescriptor =
    $convert.base64Decode('ChNHZXRTbG9TdGF0dXNSZXF1ZXN0');

@$core.Deprecated('Use sloFlowStatusDescriptor instead')
const SloFlowStatus$json = {
  '1': 'SloFlowStatus',
  '2': [
    {'1': 'flow_id', '3': 1, '4': 1, '5': 9, '10': 'flowId'},
    {'1': 'flow_name', '3': 2, '4': 1, '5': 9, '10': 'flowName'},
    {'1': 'is_violated', '3': 3, '4': 1, '5': 8, '10': 'isViolated'},
    {'1': 'burn_rate', '3': 4, '4': 1, '5': 1, '10': 'burnRate'},
    {
      '1': 'error_budget_remaining',
      '3': 5,
      '4': 1,
      '5': 1,
      '10': 'errorBudgetRemaining'
    },
  ],
};

/// Descriptor for `SloFlowStatus`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List sloFlowStatusDescriptor = $convert.base64Decode(
    'Cg1TbG9GbG93U3RhdHVzEhcKB2Zsb3dfaWQYASABKAlSBmZsb3dJZBIbCglmbG93X25hbWUYAi'
    'ABKAlSCGZsb3dOYW1lEh8KC2lzX3Zpb2xhdGVkGAMgASgIUgppc1Zpb2xhdGVkEhsKCWJ1cm5f'
    'cmF0ZRgEIAEoAVIIYnVyblJhdGUSNAoWZXJyb3JfYnVkZ2V0X3JlbWFpbmluZxgFIAEoAVIUZX'
    'Jyb3JCdWRnZXRSZW1haW5pbmc=');

@$core.Deprecated('Use getSloStatusResponseDescriptor instead')
const GetSloStatusResponse$json = {
  '1': 'GetSloStatusResponse',
  '2': [
    {
      '1': 'flows',
      '3': 1,
      '4': 3,
      '5': 11,
      '6': '.k1s0.system.eventmonitor.v1.SloFlowStatus',
      '10': 'flows'
    },
  ],
};

/// Descriptor for `GetSloStatusResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List getSloStatusResponseDescriptor = $convert.base64Decode(
    'ChRHZXRTbG9TdGF0dXNSZXNwb25zZRJACgVmbG93cxgBIAMoCzIqLmsxczAuc3lzdGVtLmV2ZW'
    '50bW9uaXRvci52MS5TbG9GbG93U3RhdHVzUgVmbG93cw==');

@$core.Deprecated('Use burnRateWindowDescriptor instead')
const BurnRateWindow$json = {
  '1': 'BurnRateWindow',
  '2': [
    {'1': 'window', '3': 1, '4': 1, '5': 9, '10': 'window'},
    {'1': 'burn_rate', '3': 2, '4': 1, '5': 1, '10': 'burnRate'},
    {
      '1': 'error_budget_remaining',
      '3': 3,
      '4': 1,
      '5': 1,
      '10': 'errorBudgetRemaining'
    },
  ],
};

/// Descriptor for `BurnRateWindow`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List burnRateWindowDescriptor = $convert.base64Decode(
    'Cg5CdXJuUmF0ZVdpbmRvdxIWCgZ3aW5kb3cYASABKAlSBndpbmRvdxIbCglidXJuX3JhdGUYAi'
    'ABKAFSCGJ1cm5SYXRlEjQKFmVycm9yX2J1ZGdldF9yZW1haW5pbmcYAyABKAFSFGVycm9yQnVk'
    'Z2V0UmVtYWluaW5n');

@$core.Deprecated('Use getSloBurnRateRequestDescriptor instead')
const GetSloBurnRateRequest$json = {
  '1': 'GetSloBurnRateRequest',
  '2': [
    {'1': 'flow_id', '3': 1, '4': 1, '5': 9, '10': 'flowId'},
  ],
};

/// Descriptor for `GetSloBurnRateRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List getSloBurnRateRequestDescriptor =
    $convert.base64Decode(
        'ChVHZXRTbG9CdXJuUmF0ZVJlcXVlc3QSFwoHZmxvd19pZBgBIAEoCVIGZmxvd0lk');

@$core.Deprecated('Use getSloBurnRateResponseDescriptor instead')
const GetSloBurnRateResponse$json = {
  '1': 'GetSloBurnRateResponse',
  '2': [
    {'1': 'flow_id', '3': 1, '4': 1, '5': 9, '10': 'flowId'},
    {'1': 'flow_name', '3': 2, '4': 1, '5': 9, '10': 'flowName'},
    {
      '1': 'windows',
      '3': 3,
      '4': 3,
      '5': 11,
      '6': '.k1s0.system.eventmonitor.v1.BurnRateWindow',
      '10': 'windows'
    },
    {'1': 'alert_status', '3': 4, '4': 1, '5': 9, '10': 'alertStatus'},
    {
      '1': 'alert_fired_at',
      '3': 5,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.common.v1.Timestamp',
      '9': 0,
      '10': 'alertFiredAt',
      '17': true
    },
  ],
  '8': [
    {'1': '_alert_fired_at'},
  ],
};

/// Descriptor for `GetSloBurnRateResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List getSloBurnRateResponseDescriptor = $convert.base64Decode(
    'ChZHZXRTbG9CdXJuUmF0ZVJlc3BvbnNlEhcKB2Zsb3dfaWQYASABKAlSBmZsb3dJZBIbCglmbG'
    '93X25hbWUYAiABKAlSCGZsb3dOYW1lEkUKB3dpbmRvd3MYAyADKAsyKy5rMXMwLnN5c3RlbS5l'
    'dmVudG1vbml0b3IudjEuQnVyblJhdGVXaW5kb3dSB3dpbmRvd3MSIQoMYWxlcnRfc3RhdHVzGA'
    'QgASgJUgthbGVydFN0YXR1cxJLCg5hbGVydF9maXJlZF9hdBgFIAEoCzIgLmsxczAuc3lzdGVt'
    'LmNvbW1vbi52MS5UaW1lc3RhbXBIAFIMYWxlcnRGaXJlZEF0iAEBQhEKD19hbGVydF9maXJlZF'
    '9hdA==');

@$core.Deprecated('Use replayFlowPreviewDescriptor instead')
const ReplayFlowPreview$json = {
  '1': 'ReplayFlowPreview',
  '2': [
    {'1': 'correlation_id', '3': 1, '4': 1, '5': 9, '10': 'correlationId'},
    {'1': 'flow_name', '3': 2, '4': 1, '5': 9, '10': 'flowName'},
    {'1': 'replay_from_step', '3': 3, '4': 1, '5': 5, '10': 'replayFromStep'},
    {'1': 'events_to_replay', '3': 4, '4': 1, '5': 5, '10': 'eventsToReplay'},
  ],
};

/// Descriptor for `ReplayFlowPreview`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List replayFlowPreviewDescriptor = $convert.base64Decode(
    'ChFSZXBsYXlGbG93UHJldmlldxIlCg5jb3JyZWxhdGlvbl9pZBgBIAEoCVINY29ycmVsYXRpb2'
    '5JZBIbCglmbG93X25hbWUYAiABKAlSCGZsb3dOYW1lEigKEHJlcGxheV9mcm9tX3N0ZXAYAyAB'
    'KAVSDnJlcGxheUZyb21TdGVwEigKEGV2ZW50c190b19yZXBsYXkYBCABKAVSDmV2ZW50c1RvUm'
    'VwbGF5');

@$core.Deprecated('Use previewReplayRequestDescriptor instead')
const PreviewReplayRequest$json = {
  '1': 'PreviewReplayRequest',
  '2': [
    {'1': 'correlation_ids', '3': 1, '4': 3, '5': 9, '10': 'correlationIds'},
    {'1': 'from_step_index', '3': 2, '4': 1, '5': 5, '10': 'fromStepIndex'},
    {
      '1': 'include_downstream',
      '3': 3,
      '4': 1,
      '5': 8,
      '10': 'includeDownstream'
    },
  ],
};

/// Descriptor for `PreviewReplayRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List previewReplayRequestDescriptor = $convert.base64Decode(
    'ChRQcmV2aWV3UmVwbGF5UmVxdWVzdBInCg9jb3JyZWxhdGlvbl9pZHMYASADKAlSDmNvcnJlbG'
    'F0aW9uSWRzEiYKD2Zyb21fc3RlcF9pbmRleBgCIAEoBVINZnJvbVN0ZXBJbmRleBItChJpbmNs'
    'dWRlX2Rvd25zdHJlYW0YAyABKAhSEWluY2x1ZGVEb3duc3RyZWFt');

@$core.Deprecated('Use previewReplayResponseDescriptor instead')
const PreviewReplayResponse$json = {
  '1': 'PreviewReplayResponse',
  '2': [
    {
      '1': 'total_events_to_replay',
      '3': 1,
      '4': 1,
      '5': 5,
      '10': 'totalEventsToReplay'
    },
    {
      '1': 'affected_services',
      '3': 2,
      '4': 3,
      '5': 9,
      '10': 'affectedServices'
    },
    {
      '1': 'affected_flows',
      '3': 3,
      '4': 3,
      '5': 11,
      '6': '.k1s0.system.eventmonitor.v1.ReplayFlowPreview',
      '10': 'affectedFlows'
    },
    {
      '1': 'dlq_messages_found',
      '3': 4,
      '4': 1,
      '5': 5,
      '10': 'dlqMessagesFound'
    },
    {
      '1': 'estimated_duration_seconds',
      '3': 5,
      '4': 1,
      '5': 5,
      '10': 'estimatedDurationSeconds'
    },
  ],
};

/// Descriptor for `PreviewReplayResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List previewReplayResponseDescriptor = $convert.base64Decode(
    'ChVQcmV2aWV3UmVwbGF5UmVzcG9uc2USMwoWdG90YWxfZXZlbnRzX3RvX3JlcGxheRgBIAEoBV'
    'ITdG90YWxFdmVudHNUb1JlcGxheRIrChFhZmZlY3RlZF9zZXJ2aWNlcxgCIAMoCVIQYWZmZWN0'
    'ZWRTZXJ2aWNlcxJVCg5hZmZlY3RlZF9mbG93cxgDIAMoCzIuLmsxczAuc3lzdGVtLmV2ZW50bW'
    '9uaXRvci52MS5SZXBsYXlGbG93UHJldmlld1INYWZmZWN0ZWRGbG93cxIsChJkbHFfbWVzc2Fn'
    'ZXNfZm91bmQYBCABKAVSEGRscU1lc3NhZ2VzRm91bmQSPAoaZXN0aW1hdGVkX2R1cmF0aW9uX3'
    'NlY29uZHMYBSABKAVSGGVzdGltYXRlZER1cmF0aW9uU2Vjb25kcw==');

@$core.Deprecated('Use executeReplayRequestDescriptor instead')
const ExecuteReplayRequest$json = {
  '1': 'ExecuteReplayRequest',
  '2': [
    {'1': 'correlation_ids', '3': 1, '4': 3, '5': 9, '10': 'correlationIds'},
    {'1': 'from_step_index', '3': 2, '4': 1, '5': 5, '10': 'fromStepIndex'},
    {
      '1': 'include_downstream',
      '3': 3,
      '4': 1,
      '5': 8,
      '10': 'includeDownstream'
    },
    {'1': 'dry_run', '3': 4, '4': 1, '5': 8, '10': 'dryRun'},
  ],
};

/// Descriptor for `ExecuteReplayRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List executeReplayRequestDescriptor = $convert.base64Decode(
    'ChRFeGVjdXRlUmVwbGF5UmVxdWVzdBInCg9jb3JyZWxhdGlvbl9pZHMYASADKAlSDmNvcnJlbG'
    'F0aW9uSWRzEiYKD2Zyb21fc3RlcF9pbmRleBgCIAEoBVINZnJvbVN0ZXBJbmRleBItChJpbmNs'
    'dWRlX2Rvd25zdHJlYW0YAyABKAhSEWluY2x1ZGVEb3duc3RyZWFtEhcKB2RyeV9ydW4YBCABKA'
    'hSBmRyeVJ1bg==');

@$core.Deprecated('Use executeReplayResponseDescriptor instead')
const ExecuteReplayResponse$json = {
  '1': 'ExecuteReplayResponse',
  '2': [
    {'1': 'replay_id', '3': 1, '4': 1, '5': 9, '10': 'replayId'},
    {'1': 'status', '3': 2, '4': 1, '5': 9, '10': 'status'},
    {'1': 'total_events', '3': 3, '4': 1, '5': 5, '10': 'totalEvents'},
    {'1': 'replayed_events', '3': 4, '4': 1, '5': 5, '10': 'replayedEvents'},
    {
      '1': 'started_at',
      '3': 5,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.common.v1.Timestamp',
      '10': 'startedAt'
    },
  ],
};

/// Descriptor for `ExecuteReplayResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List executeReplayResponseDescriptor = $convert.base64Decode(
    'ChVFeGVjdXRlUmVwbGF5UmVzcG9uc2USGwoJcmVwbGF5X2lkGAEgASgJUghyZXBsYXlJZBIWCg'
    'ZzdGF0dXMYAiABKAlSBnN0YXR1cxIhCgx0b3RhbF9ldmVudHMYAyABKAVSC3RvdGFsRXZlbnRz'
    'EicKD3JlcGxheWVkX2V2ZW50cxgEIAEoBVIOcmVwbGF5ZWRFdmVudHMSPwoKc3RhcnRlZF9hdB'
    'gFIAEoCzIgLmsxczAuc3lzdGVtLmNvbW1vbi52MS5UaW1lc3RhbXBSCXN0YXJ0ZWRBdA==');
