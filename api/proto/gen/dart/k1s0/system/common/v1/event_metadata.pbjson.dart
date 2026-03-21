// This is a generated file - do not edit.
//
// Generated from k1s0/system/common/v1/event_metadata.proto.

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

@$core.Deprecated('Use eventMetadataDescriptor instead')
const EventMetadata$json = {
  '1': 'EventMetadata',
  '2': [
    {'1': 'event_id', '3': 1, '4': 1, '5': 9, '10': 'eventId'},
    {'1': 'event_type', '3': 2, '4': 1, '5': 9, '10': 'eventType'},
    {'1': 'source', '3': 3, '4': 1, '5': 9, '10': 'source'},
    {'1': 'timestamp', '3': 4, '4': 1, '5': 3, '10': 'timestamp'},
    {'1': 'trace_id', '3': 5, '4': 1, '5': 9, '10': 'traceId'},
    {'1': 'correlation_id', '3': 6, '4': 1, '5': 9, '10': 'correlationId'},
    {'1': 'schema_version', '3': 7, '4': 1, '5': 5, '10': 'schemaVersion'},
    {'1': 'causation_id', '3': 8, '4': 1, '5': 9, '10': 'causationId'},
  ],
};

/// Descriptor for `EventMetadata`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List eventMetadataDescriptor = $convert.base64Decode(
    'Cg1FdmVudE1ldGFkYXRhEhkKCGV2ZW50X2lkGAEgASgJUgdldmVudElkEh0KCmV2ZW50X3R5cG'
    'UYAiABKAlSCWV2ZW50VHlwZRIWCgZzb3VyY2UYAyABKAlSBnNvdXJjZRIcCgl0aW1lc3RhbXAY'
    'BCABKANSCXRpbWVzdGFtcBIZCgh0cmFjZV9pZBgFIAEoCVIHdHJhY2VJZBIlCg5jb3JyZWxhdG'
    'lvbl9pZBgGIAEoCVINY29ycmVsYXRpb25JZBIlCg5zY2hlbWFfdmVyc2lvbhgHIAEoBVINc2No'
    'ZW1hVmVyc2lvbhIhCgxjYXVzYXRpb25faWQYCCABKAlSC2NhdXNhdGlvbklk');
