// This is a generated file - do not edit.
//
// Generated from k1s0/event/system/config/v1/config_events.proto.

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

@$core.Deprecated('Use configChangedEventDescriptor instead')
const ConfigChangedEvent$json = {
  '1': 'ConfigChangedEvent',
  '2': [
    {
      '1': 'metadata',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.common.v1.EventMetadata',
      '10': 'metadata'
    },
    {'1': 'namespace', '3': 2, '4': 1, '5': 9, '10': 'namespace'},
    {'1': 'key', '3': 3, '4': 1, '5': 9, '10': 'key'},
    {'1': 'old_value', '3': 4, '4': 1, '5': 9, '10': 'oldValue'},
    {'1': 'new_value', '3': 5, '4': 1, '5': 9, '10': 'newValue'},
    {'1': 'old_version', '3': 6, '4': 1, '5': 5, '10': 'oldVersion'},
    {'1': 'new_version', '3': 7, '4': 1, '5': 5, '10': 'newVersion'},
    {'1': 'change_type', '3': 8, '4': 1, '5': 9, '10': 'changeType'},
    {'1': 'changed_by', '3': 9, '4': 1, '5': 9, '10': 'changedBy'},
  ],
};

/// Descriptor for `ConfigChangedEvent`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List configChangedEventDescriptor = $convert.base64Decode(
    'ChJDb25maWdDaGFuZ2VkRXZlbnQSQAoIbWV0YWRhdGEYASABKAsyJC5rMXMwLnN5c3RlbS5jb2'
    '1tb24udjEuRXZlbnRNZXRhZGF0YVIIbWV0YWRhdGESHAoJbmFtZXNwYWNlGAIgASgJUgluYW1l'
    'c3BhY2USEAoDa2V5GAMgASgJUgNrZXkSGwoJb2xkX3ZhbHVlGAQgASgJUghvbGRWYWx1ZRIbCg'
    'luZXdfdmFsdWUYBSABKAlSCG5ld1ZhbHVlEh8KC29sZF92ZXJzaW9uGAYgASgFUgpvbGRWZXJz'
    'aW9uEh8KC25ld192ZXJzaW9uGAcgASgFUgpuZXdWZXJzaW9uEh8KC2NoYW5nZV90eXBlGAggAS'
    'gJUgpjaGFuZ2VUeXBlEh0KCmNoYW5nZWRfYnkYCSABKAlSCWNoYW5nZWRCeQ==');
