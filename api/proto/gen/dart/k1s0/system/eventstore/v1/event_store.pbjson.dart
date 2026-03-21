// This is a generated file - do not edit.
//
// Generated from k1s0/system/eventstore/v1/event_store.proto.

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

@$core.Deprecated('Use listStreamsRequestDescriptor instead')
const ListStreamsRequest$json = {
  '1': 'ListStreamsRequest',
  '2': [
    {
      '1': 'pagination',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.common.v1.Pagination',
      '10': 'pagination'
    },
  ],
};

/// Descriptor for `ListStreamsRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List listStreamsRequestDescriptor = $convert.base64Decode(
    'ChJMaXN0U3RyZWFtc1JlcXVlc3QSQQoKcGFnaW5hdGlvbhgBIAEoCzIhLmsxczAuc3lzdGVtLm'
    'NvbW1vbi52MS5QYWdpbmF0aW9uUgpwYWdpbmF0aW9u');

@$core.Deprecated('Use listStreamsResponseDescriptor instead')
const ListStreamsResponse$json = {
  '1': 'ListStreamsResponse',
  '2': [
    {
      '1': 'streams',
      '3': 1,
      '4': 3,
      '5': 11,
      '6': '.k1s0.system.eventstore.v1.StreamInfo',
      '10': 'streams'
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

/// Descriptor for `ListStreamsResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List listStreamsResponseDescriptor = $convert.base64Decode(
    'ChNMaXN0U3RyZWFtc1Jlc3BvbnNlEj8KB3N0cmVhbXMYASADKAsyJS5rMXMwLnN5c3RlbS5ldm'
    'VudHN0b3JlLnYxLlN0cmVhbUluZm9SB3N0cmVhbXMSRwoKcGFnaW5hdGlvbhgCIAEoCzInLmsx'
    'czAuc3lzdGVtLmNvbW1vbi52MS5QYWdpbmF0aW9uUmVzdWx0UgpwYWdpbmF0aW9u');

@$core.Deprecated('Use streamInfoDescriptor instead')
const StreamInfo$json = {
  '1': 'StreamInfo',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 9, '10': 'id'},
    {'1': 'aggregate_type', '3': 2, '4': 1, '5': 9, '10': 'aggregateType'},
    {'1': 'current_version', '3': 3, '4': 1, '5': 3, '10': 'currentVersion'},
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
  ],
};

/// Descriptor for `StreamInfo`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List streamInfoDescriptor = $convert.base64Decode(
    'CgpTdHJlYW1JbmZvEg4KAmlkGAEgASgJUgJpZBIlCg5hZ2dyZWdhdGVfdHlwZRgCIAEoCVINYW'
    'dncmVnYXRlVHlwZRInCg9jdXJyZW50X3ZlcnNpb24YAyABKANSDmN1cnJlbnRWZXJzaW9uEj8K'
    'CmNyZWF0ZWRfYXQYBCABKAsyIC5rMXMwLnN5c3RlbS5jb21tb24udjEuVGltZXN0YW1wUgljcm'
    'VhdGVkQXQSPwoKdXBkYXRlZF9hdBgFIAEoCzIgLmsxczAuc3lzdGVtLmNvbW1vbi52MS5UaW1l'
    'c3RhbXBSCXVwZGF0ZWRBdA==');

@$core.Deprecated('Use appendEventsRequestDescriptor instead')
const AppendEventsRequest$json = {
  '1': 'AppendEventsRequest',
  '2': [
    {'1': 'stream_id', '3': 1, '4': 1, '5': 9, '10': 'streamId'},
    {
      '1': 'events',
      '3': 2,
      '4': 3,
      '5': 11,
      '6': '.k1s0.system.eventstore.v1.EventData',
      '10': 'events'
    },
    {'1': 'expected_version', '3': 3, '4': 1, '5': 3, '10': 'expectedVersion'},
    {'1': 'aggregate_type', '3': 4, '4': 1, '5': 9, '10': 'aggregateType'},
  ],
};

/// Descriptor for `AppendEventsRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List appendEventsRequestDescriptor = $convert.base64Decode(
    'ChNBcHBlbmRFdmVudHNSZXF1ZXN0EhsKCXN0cmVhbV9pZBgBIAEoCVIIc3RyZWFtSWQSPAoGZX'
    'ZlbnRzGAIgAygLMiQuazFzMC5zeXN0ZW0uZXZlbnRzdG9yZS52MS5FdmVudERhdGFSBmV2ZW50'
    'cxIpChBleHBlY3RlZF92ZXJzaW9uGAMgASgDUg9leHBlY3RlZFZlcnNpb24SJQoOYWdncmVnYX'
    'RlX3R5cGUYBCABKAlSDWFnZ3JlZ2F0ZVR5cGU=');

@$core.Deprecated('Use appendEventsResponseDescriptor instead')
const AppendEventsResponse$json = {
  '1': 'AppendEventsResponse',
  '2': [
    {'1': 'stream_id', '3': 1, '4': 1, '5': 9, '10': 'streamId'},
    {
      '1': 'events',
      '3': 2,
      '4': 3,
      '5': 11,
      '6': '.k1s0.system.eventstore.v1.StoredEvent',
      '10': 'events'
    },
    {'1': 'current_version', '3': 3, '4': 1, '5': 3, '10': 'currentVersion'},
  ],
};

/// Descriptor for `AppendEventsResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List appendEventsResponseDescriptor = $convert.base64Decode(
    'ChRBcHBlbmRFdmVudHNSZXNwb25zZRIbCglzdHJlYW1faWQYASABKAlSCHN0cmVhbUlkEj4KBm'
    'V2ZW50cxgCIAMoCzImLmsxczAuc3lzdGVtLmV2ZW50c3RvcmUudjEuU3RvcmVkRXZlbnRSBmV2'
    'ZW50cxInCg9jdXJyZW50X3ZlcnNpb24YAyABKANSDmN1cnJlbnRWZXJzaW9u');

@$core.Deprecated('Use readEventsRequestDescriptor instead')
const ReadEventsRequest$json = {
  '1': 'ReadEventsRequest',
  '2': [
    {'1': 'stream_id', '3': 1, '4': 1, '5': 9, '10': 'streamId'},
    {'1': 'from_version', '3': 2, '4': 1, '5': 3, '10': 'fromVersion'},
    {
      '1': 'to_version',
      '3': 3,
      '4': 1,
      '5': 3,
      '9': 0,
      '10': 'toVersion',
      '17': true
    },
    {
      '1': 'pagination',
      '3': 4,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.common.v1.Pagination',
      '10': 'pagination'
    },
    {
      '1': 'event_type',
      '3': 6,
      '4': 1,
      '5': 9,
      '9': 1,
      '10': 'eventType',
      '17': true
    },
  ],
  '8': [
    {'1': '_to_version'},
    {'1': '_event_type'},
  ],
  '9': [
    {'1': 5, '2': 6},
  ],
  '10': ['page_size'],
};

/// Descriptor for `ReadEventsRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List readEventsRequestDescriptor = $convert.base64Decode(
    'ChFSZWFkRXZlbnRzUmVxdWVzdBIbCglzdHJlYW1faWQYASABKAlSCHN0cmVhbUlkEiEKDGZyb2'
    '1fdmVyc2lvbhgCIAEoA1ILZnJvbVZlcnNpb24SIgoKdG9fdmVyc2lvbhgDIAEoA0gAUgl0b1Zl'
    'cnNpb26IAQESQQoKcGFnaW5hdGlvbhgEIAEoCzIhLmsxczAuc3lzdGVtLmNvbW1vbi52MS5QYW'
    'dpbmF0aW9uUgpwYWdpbmF0aW9uEiIKCmV2ZW50X3R5cGUYBiABKAlIAVIJZXZlbnRUeXBliAEB'
    'Qg0KC190b192ZXJzaW9uQg0KC19ldmVudF90eXBlSgQIBRAGUglwYWdlX3NpemU=');

@$core.Deprecated('Use readEventsResponseDescriptor instead')
const ReadEventsResponse$json = {
  '1': 'ReadEventsResponse',
  '2': [
    {'1': 'stream_id', '3': 1, '4': 1, '5': 9, '10': 'streamId'},
    {
      '1': 'events',
      '3': 2,
      '4': 3,
      '5': 11,
      '6': '.k1s0.system.eventstore.v1.StoredEvent',
      '10': 'events'
    },
    {'1': 'current_version', '3': 3, '4': 1, '5': 3, '10': 'currentVersion'},
    {
      '1': 'pagination',
      '3': 4,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.common.v1.PaginationResult',
      '10': 'pagination'
    },
  ],
};

/// Descriptor for `ReadEventsResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List readEventsResponseDescriptor = $convert.base64Decode(
    'ChJSZWFkRXZlbnRzUmVzcG9uc2USGwoJc3RyZWFtX2lkGAEgASgJUghzdHJlYW1JZBI+CgZldm'
    'VudHMYAiADKAsyJi5rMXMwLnN5c3RlbS5ldmVudHN0b3JlLnYxLlN0b3JlZEV2ZW50UgZldmVu'
    'dHMSJwoPY3VycmVudF92ZXJzaW9uGAMgASgDUg5jdXJyZW50VmVyc2lvbhJHCgpwYWdpbmF0aW'
    '9uGAQgASgLMicuazFzMC5zeXN0ZW0uY29tbW9uLnYxLlBhZ2luYXRpb25SZXN1bHRSCnBhZ2lu'
    'YXRpb24=');

@$core.Deprecated('Use readEventBySequenceRequestDescriptor instead')
const ReadEventBySequenceRequest$json = {
  '1': 'ReadEventBySequenceRequest',
  '2': [
    {'1': 'stream_id', '3': 1, '4': 1, '5': 9, '10': 'streamId'},
    {'1': 'sequence', '3': 2, '4': 1, '5': 4, '10': 'sequence'},
  ],
};

/// Descriptor for `ReadEventBySequenceRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List readEventBySequenceRequestDescriptor =
    $convert.base64Decode(
        'ChpSZWFkRXZlbnRCeVNlcXVlbmNlUmVxdWVzdBIbCglzdHJlYW1faWQYASABKAlSCHN0cmVhbU'
        'lkEhoKCHNlcXVlbmNlGAIgASgEUghzZXF1ZW5jZQ==');

@$core.Deprecated('Use readEventBySequenceResponseDescriptor instead')
const ReadEventBySequenceResponse$json = {
  '1': 'ReadEventBySequenceResponse',
  '2': [
    {
      '1': 'event',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.eventstore.v1.StoredEvent',
      '10': 'event'
    },
  ],
};

/// Descriptor for `ReadEventBySequenceResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List readEventBySequenceResponseDescriptor =
    $convert.base64Decode(
        'ChtSZWFkRXZlbnRCeVNlcXVlbmNlUmVzcG9uc2USPAoFZXZlbnQYASABKAsyJi5rMXMwLnN5c3'
        'RlbS5ldmVudHN0b3JlLnYxLlN0b3JlZEV2ZW50UgVldmVudA==');

@$core.Deprecated('Use createSnapshotRequestDescriptor instead')
const CreateSnapshotRequest$json = {
  '1': 'CreateSnapshotRequest',
  '2': [
    {'1': 'stream_id', '3': 1, '4': 1, '5': 9, '10': 'streamId'},
    {'1': 'snapshot_version', '3': 2, '4': 1, '5': 3, '10': 'snapshotVersion'},
    {'1': 'aggregate_type', '3': 3, '4': 1, '5': 9, '10': 'aggregateType'},
    {'1': 'state', '3': 4, '4': 1, '5': 12, '10': 'state'},
  ],
};

/// Descriptor for `CreateSnapshotRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List createSnapshotRequestDescriptor = $convert.base64Decode(
    'ChVDcmVhdGVTbmFwc2hvdFJlcXVlc3QSGwoJc3RyZWFtX2lkGAEgASgJUghzdHJlYW1JZBIpCh'
    'BzbmFwc2hvdF92ZXJzaW9uGAIgASgDUg9zbmFwc2hvdFZlcnNpb24SJQoOYWdncmVnYXRlX3R5'
    'cGUYAyABKAlSDWFnZ3JlZ2F0ZVR5cGUSFAoFc3RhdGUYBCABKAxSBXN0YXRl');

@$core.Deprecated('Use createSnapshotResponseDescriptor instead')
const CreateSnapshotResponse$json = {
  '1': 'CreateSnapshotResponse',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 9, '10': 'id'},
    {'1': 'stream_id', '3': 2, '4': 1, '5': 9, '10': 'streamId'},
    {'1': 'snapshot_version', '3': 3, '4': 1, '5': 3, '10': 'snapshotVersion'},
    {
      '1': 'created_at',
      '3': 4,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.common.v1.Timestamp',
      '10': 'createdAt'
    },
    {'1': 'aggregate_type', '3': 5, '4': 1, '5': 9, '10': 'aggregateType'},
  ],
};

/// Descriptor for `CreateSnapshotResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List createSnapshotResponseDescriptor = $convert.base64Decode(
    'ChZDcmVhdGVTbmFwc2hvdFJlc3BvbnNlEg4KAmlkGAEgASgJUgJpZBIbCglzdHJlYW1faWQYAi'
    'ABKAlSCHN0cmVhbUlkEikKEHNuYXBzaG90X3ZlcnNpb24YAyABKANSD3NuYXBzaG90VmVyc2lv'
    'bhI/CgpjcmVhdGVkX2F0GAQgASgLMiAuazFzMC5zeXN0ZW0uY29tbW9uLnYxLlRpbWVzdGFtcF'
    'IJY3JlYXRlZEF0EiUKDmFnZ3JlZ2F0ZV90eXBlGAUgASgJUg1hZ2dyZWdhdGVUeXBl');

@$core.Deprecated('Use getLatestSnapshotRequestDescriptor instead')
const GetLatestSnapshotRequest$json = {
  '1': 'GetLatestSnapshotRequest',
  '2': [
    {'1': 'stream_id', '3': 1, '4': 1, '5': 9, '10': 'streamId'},
  ],
};

/// Descriptor for `GetLatestSnapshotRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List getLatestSnapshotRequestDescriptor =
    $convert.base64Decode(
        'ChhHZXRMYXRlc3RTbmFwc2hvdFJlcXVlc3QSGwoJc3RyZWFtX2lkGAEgASgJUghzdHJlYW1JZA'
        '==');

@$core.Deprecated('Use getLatestSnapshotResponseDescriptor instead')
const GetLatestSnapshotResponse$json = {
  '1': 'GetLatestSnapshotResponse',
  '2': [
    {
      '1': 'snapshot',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.eventstore.v1.Snapshot',
      '10': 'snapshot'
    },
  ],
};

/// Descriptor for `GetLatestSnapshotResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List getLatestSnapshotResponseDescriptor =
    $convert.base64Decode(
        'ChlHZXRMYXRlc3RTbmFwc2hvdFJlc3BvbnNlEj8KCHNuYXBzaG90GAEgASgLMiMuazFzMC5zeX'
        'N0ZW0uZXZlbnRzdG9yZS52MS5TbmFwc2hvdFIIc25hcHNob3Q=');

@$core.Deprecated('Use deleteStreamRequestDescriptor instead')
const DeleteStreamRequest$json = {
  '1': 'DeleteStreamRequest',
  '2': [
    {'1': 'stream_id', '3': 1, '4': 1, '5': 9, '10': 'streamId'},
  ],
};

/// Descriptor for `DeleteStreamRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List deleteStreamRequestDescriptor =
    $convert.base64Decode(
        'ChNEZWxldGVTdHJlYW1SZXF1ZXN0EhsKCXN0cmVhbV9pZBgBIAEoCVIIc3RyZWFtSWQ=');

@$core.Deprecated('Use deleteStreamResponseDescriptor instead')
const DeleteStreamResponse$json = {
  '1': 'DeleteStreamResponse',
  '2': [
    {'1': 'success', '3': 1, '4': 1, '5': 8, '10': 'success'},
    {'1': 'message', '3': 2, '4': 1, '5': 9, '10': 'message'},
  ],
};

/// Descriptor for `DeleteStreamResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List deleteStreamResponseDescriptor = $convert.base64Decode(
    'ChREZWxldGVTdHJlYW1SZXNwb25zZRIYCgdzdWNjZXNzGAEgASgIUgdzdWNjZXNzEhgKB21lc3'
    'NhZ2UYAiABKAlSB21lc3NhZ2U=');

@$core.Deprecated('Use eventDataDescriptor instead')
const EventData$json = {
  '1': 'EventData',
  '2': [
    {'1': 'event_type', '3': 1, '4': 1, '5': 9, '10': 'eventType'},
    {'1': 'payload', '3': 2, '4': 1, '5': 12, '10': 'payload'},
    {
      '1': 'metadata',
      '3': 3,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.eventstore.v1.EventStoreMetadata',
      '10': 'metadata'
    },
  ],
};

/// Descriptor for `EventData`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List eventDataDescriptor = $convert.base64Decode(
    'CglFdmVudERhdGESHQoKZXZlbnRfdHlwZRgBIAEoCVIJZXZlbnRUeXBlEhgKB3BheWxvYWQYAi'
    'ABKAxSB3BheWxvYWQSSQoIbWV0YWRhdGEYAyABKAsyLS5rMXMwLnN5c3RlbS5ldmVudHN0b3Jl'
    'LnYxLkV2ZW50U3RvcmVNZXRhZGF0YVIIbWV0YWRhdGE=');

@$core.Deprecated('Use storedEventDescriptor instead')
const StoredEvent$json = {
  '1': 'StoredEvent',
  '2': [
    {'1': 'stream_id', '3': 1, '4': 1, '5': 9, '10': 'streamId'},
    {'1': 'sequence', '3': 2, '4': 1, '5': 4, '10': 'sequence'},
    {'1': 'event_type', '3': 3, '4': 1, '5': 9, '10': 'eventType'},
    {'1': 'version', '3': 4, '4': 1, '5': 3, '10': 'version'},
    {'1': 'payload', '3': 5, '4': 1, '5': 12, '10': 'payload'},
    {
      '1': 'metadata',
      '3': 6,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.eventstore.v1.EventStoreMetadata',
      '10': 'metadata'
    },
    {
      '1': 'occurred_at',
      '3': 7,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.common.v1.Timestamp',
      '10': 'occurredAt'
    },
    {
      '1': 'stored_at',
      '3': 8,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.common.v1.Timestamp',
      '10': 'storedAt'
    },
  ],
};

/// Descriptor for `StoredEvent`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List storedEventDescriptor = $convert.base64Decode(
    'CgtTdG9yZWRFdmVudBIbCglzdHJlYW1faWQYASABKAlSCHN0cmVhbUlkEhoKCHNlcXVlbmNlGA'
    'IgASgEUghzZXF1ZW5jZRIdCgpldmVudF90eXBlGAMgASgJUglldmVudFR5cGUSGAoHdmVyc2lv'
    'bhgEIAEoA1IHdmVyc2lvbhIYCgdwYXlsb2FkGAUgASgMUgdwYXlsb2FkEkkKCG1ldGFkYXRhGA'
    'YgASgLMi0uazFzMC5zeXN0ZW0uZXZlbnRzdG9yZS52MS5FdmVudFN0b3JlTWV0YWRhdGFSCG1l'
    'dGFkYXRhEkEKC29jY3VycmVkX2F0GAcgASgLMiAuazFzMC5zeXN0ZW0uY29tbW9uLnYxLlRpbW'
    'VzdGFtcFIKb2NjdXJyZWRBdBI9CglzdG9yZWRfYXQYCCABKAsyIC5rMXMwLnN5c3RlbS5jb21t'
    'b24udjEuVGltZXN0YW1wUghzdG9yZWRBdA==');

@$core.Deprecated('Use eventStoreMetadataDescriptor instead')
const EventStoreMetadata$json = {
  '1': 'EventStoreMetadata',
  '2': [
    {
      '1': 'actor_id',
      '3': 1,
      '4': 1,
      '5': 9,
      '9': 0,
      '10': 'actorId',
      '17': true
    },
    {
      '1': 'correlation_id',
      '3': 2,
      '4': 1,
      '5': 9,
      '9': 1,
      '10': 'correlationId',
      '17': true
    },
    {
      '1': 'causation_id',
      '3': 3,
      '4': 1,
      '5': 9,
      '9': 2,
      '10': 'causationId',
      '17': true
    },
  ],
  '8': [
    {'1': '_actor_id'},
    {'1': '_correlation_id'},
    {'1': '_causation_id'},
  ],
};

/// Descriptor for `EventStoreMetadata`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List eventStoreMetadataDescriptor = $convert.base64Decode(
    'ChJFdmVudFN0b3JlTWV0YWRhdGESHgoIYWN0b3JfaWQYASABKAlIAFIHYWN0b3JJZIgBARIqCg'
    '5jb3JyZWxhdGlvbl9pZBgCIAEoCUgBUg1jb3JyZWxhdGlvbklkiAEBEiYKDGNhdXNhdGlvbl9p'
    'ZBgDIAEoCUgCUgtjYXVzYXRpb25JZIgBAUILCglfYWN0b3JfaWRCEQoPX2NvcnJlbGF0aW9uX2'
    'lkQg8KDV9jYXVzYXRpb25faWQ=');

@$core.Deprecated('Use snapshotDescriptor instead')
const Snapshot$json = {
  '1': 'Snapshot',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 9, '10': 'id'},
    {'1': 'stream_id', '3': 2, '4': 1, '5': 9, '10': 'streamId'},
    {'1': 'snapshot_version', '3': 3, '4': 1, '5': 3, '10': 'snapshotVersion'},
    {'1': 'aggregate_type', '3': 4, '4': 1, '5': 9, '10': 'aggregateType'},
    {'1': 'state', '3': 5, '4': 1, '5': 12, '10': 'state'},
    {
      '1': 'created_at',
      '3': 6,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.common.v1.Timestamp',
      '10': 'createdAt'
    },
  ],
};

/// Descriptor for `Snapshot`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List snapshotDescriptor = $convert.base64Decode(
    'CghTbmFwc2hvdBIOCgJpZBgBIAEoCVICaWQSGwoJc3RyZWFtX2lkGAIgASgJUghzdHJlYW1JZB'
    'IpChBzbmFwc2hvdF92ZXJzaW9uGAMgASgDUg9zbmFwc2hvdFZlcnNpb24SJQoOYWdncmVnYXRl'
    'X3R5cGUYBCABKAlSDWFnZ3JlZ2F0ZVR5cGUSFAoFc3RhdGUYBSABKAxSBXN0YXRlEj8KCmNyZW'
    'F0ZWRfYXQYBiABKAsyIC5rMXMwLnN5c3RlbS5jb21tb24udjEuVGltZXN0YW1wUgljcmVhdGVk'
    'QXQ=');
