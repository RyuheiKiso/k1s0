// This is a generated file - do not edit.
//
// Generated from k1s0/system/vault/v1/vault.proto.

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

@$core.Deprecated('Use getSecretRequestDescriptor instead')
const GetSecretRequest$json = {
  '1': 'GetSecretRequest',
  '2': [
    {'1': 'path', '3': 1, '4': 1, '5': 9, '10': 'path'},
    {'1': 'version', '3': 2, '4': 1, '5': 3, '10': 'version'},
  ],
};

/// Descriptor for `GetSecretRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List getSecretRequestDescriptor = $convert.base64Decode(
    'ChBHZXRTZWNyZXRSZXF1ZXN0EhIKBHBhdGgYASABKAlSBHBhdGgSGAoHdmVyc2lvbhgCIAEoA1'
    'IHdmVyc2lvbg==');

@$core.Deprecated('Use getSecretResponseDescriptor instead')
const GetSecretResponse$json = {
  '1': 'GetSecretResponse',
  '2': [
    {
      '1': 'data',
      '3': 1,
      '4': 3,
      '5': 11,
      '6': '.k1s0.system.vault.v1.GetSecretResponse.DataEntry',
      '10': 'data'
    },
    {'1': 'version', '3': 2, '4': 1, '5': 3, '10': 'version'},
    {
      '1': 'created_at',
      '3': 3,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.common.v1.Timestamp',
      '10': 'createdAt'
    },
    {
      '1': 'updated_at',
      '3': 4,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.common.v1.Timestamp',
      '10': 'updatedAt'
    },
    {'1': 'path', '3': 5, '4': 1, '5': 9, '10': 'path'},
  ],
  '3': [GetSecretResponse_DataEntry$json],
};

@$core.Deprecated('Use getSecretResponseDescriptor instead')
const GetSecretResponse_DataEntry$json = {
  '1': 'DataEntry',
  '2': [
    {'1': 'key', '3': 1, '4': 1, '5': 9, '10': 'key'},
    {'1': 'value', '3': 2, '4': 1, '5': 9, '10': 'value'},
  ],
  '7': {'7': true},
};

/// Descriptor for `GetSecretResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List getSecretResponseDescriptor = $convert.base64Decode(
    'ChFHZXRTZWNyZXRSZXNwb25zZRJFCgRkYXRhGAEgAygLMjEuazFzMC5zeXN0ZW0udmF1bHQudj'
    'EuR2V0U2VjcmV0UmVzcG9uc2UuRGF0YUVudHJ5UgRkYXRhEhgKB3ZlcnNpb24YAiABKANSB3Zl'
    'cnNpb24SPwoKY3JlYXRlZF9hdBgDIAEoCzIgLmsxczAuc3lzdGVtLmNvbW1vbi52MS5UaW1lc3'
    'RhbXBSCWNyZWF0ZWRBdBI/Cgp1cGRhdGVkX2F0GAQgASgLMiAuazFzMC5zeXN0ZW0uY29tbW9u'
    'LnYxLlRpbWVzdGFtcFIJdXBkYXRlZEF0EhIKBHBhdGgYBSABKAlSBHBhdGgaNwoJRGF0YUVudH'
    'J5EhAKA2tleRgBIAEoCVIDa2V5EhQKBXZhbHVlGAIgASgJUgV2YWx1ZToCOAE=');

@$core.Deprecated('Use setSecretRequestDescriptor instead')
const SetSecretRequest$json = {
  '1': 'SetSecretRequest',
  '2': [
    {'1': 'path', '3': 1, '4': 1, '5': 9, '10': 'path'},
    {
      '1': 'data',
      '3': 2,
      '4': 3,
      '5': 11,
      '6': '.k1s0.system.vault.v1.SetSecretRequest.DataEntry',
      '10': 'data'
    },
  ],
  '3': [SetSecretRequest_DataEntry$json],
};

@$core.Deprecated('Use setSecretRequestDescriptor instead')
const SetSecretRequest_DataEntry$json = {
  '1': 'DataEntry',
  '2': [
    {'1': 'key', '3': 1, '4': 1, '5': 9, '10': 'key'},
    {'1': 'value', '3': 2, '4': 1, '5': 9, '10': 'value'},
  ],
  '7': {'7': true},
};

/// Descriptor for `SetSecretRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List setSecretRequestDescriptor = $convert.base64Decode(
    'ChBTZXRTZWNyZXRSZXF1ZXN0EhIKBHBhdGgYASABKAlSBHBhdGgSRAoEZGF0YRgCIAMoCzIwLm'
    'sxczAuc3lzdGVtLnZhdWx0LnYxLlNldFNlY3JldFJlcXVlc3QuRGF0YUVudHJ5UgRkYXRhGjcK'
    'CURhdGFFbnRyeRIQCgNrZXkYASABKAlSA2tleRIUCgV2YWx1ZRgCIAEoCVIFdmFsdWU6AjgB');

@$core.Deprecated('Use setSecretResponseDescriptor instead')
const SetSecretResponse$json = {
  '1': 'SetSecretResponse',
  '2': [
    {'1': 'version', '3': 1, '4': 1, '5': 3, '10': 'version'},
    {
      '1': 'created_at',
      '3': 2,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.common.v1.Timestamp',
      '10': 'createdAt'
    },
    {'1': 'path', '3': 3, '4': 1, '5': 9, '10': 'path'},
  ],
};

/// Descriptor for `SetSecretResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List setSecretResponseDescriptor = $convert.base64Decode(
    'ChFTZXRTZWNyZXRSZXNwb25zZRIYCgd2ZXJzaW9uGAEgASgDUgd2ZXJzaW9uEj8KCmNyZWF0ZW'
    'RfYXQYAiABKAsyIC5rMXMwLnN5c3RlbS5jb21tb24udjEuVGltZXN0YW1wUgljcmVhdGVkQXQS'
    'EgoEcGF0aBgDIAEoCVIEcGF0aA==');

@$core.Deprecated('Use rotateSecretRequestDescriptor instead')
const RotateSecretRequest$json = {
  '1': 'RotateSecretRequest',
  '2': [
    {'1': 'path', '3': 1, '4': 1, '5': 9, '10': 'path'},
    {
      '1': 'data',
      '3': 2,
      '4': 3,
      '5': 11,
      '6': '.k1s0.system.vault.v1.RotateSecretRequest.DataEntry',
      '10': 'data'
    },
  ],
  '3': [RotateSecretRequest_DataEntry$json],
};

@$core.Deprecated('Use rotateSecretRequestDescriptor instead')
const RotateSecretRequest_DataEntry$json = {
  '1': 'DataEntry',
  '2': [
    {'1': 'key', '3': 1, '4': 1, '5': 9, '10': 'key'},
    {'1': 'value', '3': 2, '4': 1, '5': 9, '10': 'value'},
  ],
  '7': {'7': true},
};

/// Descriptor for `RotateSecretRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List rotateSecretRequestDescriptor = $convert.base64Decode(
    'ChNSb3RhdGVTZWNyZXRSZXF1ZXN0EhIKBHBhdGgYASABKAlSBHBhdGgSRwoEZGF0YRgCIAMoCz'
    'IzLmsxczAuc3lzdGVtLnZhdWx0LnYxLlJvdGF0ZVNlY3JldFJlcXVlc3QuRGF0YUVudHJ5UgRk'
    'YXRhGjcKCURhdGFFbnRyeRIQCgNrZXkYASABKAlSA2tleRIUCgV2YWx1ZRgCIAEoCVIFdmFsdW'
    'U6AjgB');

@$core.Deprecated('Use rotateSecretResponseDescriptor instead')
const RotateSecretResponse$json = {
  '1': 'RotateSecretResponse',
  '2': [
    {'1': 'path', '3': 1, '4': 1, '5': 9, '10': 'path'},
    {'1': 'new_version', '3': 2, '4': 1, '5': 3, '10': 'newVersion'},
    {'1': 'rotated', '3': 3, '4': 1, '5': 8, '10': 'rotated'},
  ],
};

/// Descriptor for `RotateSecretResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List rotateSecretResponseDescriptor = $convert.base64Decode(
    'ChRSb3RhdGVTZWNyZXRSZXNwb25zZRISCgRwYXRoGAEgASgJUgRwYXRoEh8KC25ld192ZXJzaW'
    '9uGAIgASgDUgpuZXdWZXJzaW9uEhgKB3JvdGF0ZWQYAyABKAhSB3JvdGF0ZWQ=');

@$core.Deprecated('Use deleteSecretRequestDescriptor instead')
const DeleteSecretRequest$json = {
  '1': 'DeleteSecretRequest',
  '2': [
    {'1': 'path', '3': 1, '4': 1, '5': 9, '10': 'path'},
    {'1': 'versions', '3': 2, '4': 3, '5': 3, '10': 'versions'},
  ],
};

/// Descriptor for `DeleteSecretRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List deleteSecretRequestDescriptor = $convert.base64Decode(
    'ChNEZWxldGVTZWNyZXRSZXF1ZXN0EhIKBHBhdGgYASABKAlSBHBhdGgSGgoIdmVyc2lvbnMYAi'
    'ADKANSCHZlcnNpb25z');

@$core.Deprecated('Use deleteSecretResponseDescriptor instead')
const DeleteSecretResponse$json = {
  '1': 'DeleteSecretResponse',
  '2': [
    {'1': 'success', '3': 1, '4': 1, '5': 8, '10': 'success'},
  ],
};

/// Descriptor for `DeleteSecretResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List deleteSecretResponseDescriptor =
    $convert.base64Decode(
        'ChREZWxldGVTZWNyZXRSZXNwb25zZRIYCgdzdWNjZXNzGAEgASgIUgdzdWNjZXNz');

@$core.Deprecated('Use getSecretMetadataRequestDescriptor instead')
const GetSecretMetadataRequest$json = {
  '1': 'GetSecretMetadataRequest',
  '2': [
    {'1': 'path', '3': 1, '4': 1, '5': 9, '10': 'path'},
  ],
};

/// Descriptor for `GetSecretMetadataRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List getSecretMetadataRequestDescriptor =
    $convert.base64Decode(
        'ChhHZXRTZWNyZXRNZXRhZGF0YVJlcXVlc3QSEgoEcGF0aBgBIAEoCVIEcGF0aA==');

@$core.Deprecated('Use getSecretMetadataResponseDescriptor instead')
const GetSecretMetadataResponse$json = {
  '1': 'GetSecretMetadataResponse',
  '2': [
    {'1': 'path', '3': 1, '4': 1, '5': 9, '10': 'path'},
    {'1': 'current_version', '3': 2, '4': 1, '5': 3, '10': 'currentVersion'},
    {'1': 'version_count', '3': 3, '4': 1, '5': 5, '10': 'versionCount'},
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

/// Descriptor for `GetSecretMetadataResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List getSecretMetadataResponseDescriptor = $convert.base64Decode(
    'ChlHZXRTZWNyZXRNZXRhZGF0YVJlc3BvbnNlEhIKBHBhdGgYASABKAlSBHBhdGgSJwoPY3Vycm'
    'VudF92ZXJzaW9uGAIgASgDUg5jdXJyZW50VmVyc2lvbhIjCg12ZXJzaW9uX2NvdW50GAMgASgF'
    'Ugx2ZXJzaW9uQ291bnQSPwoKY3JlYXRlZF9hdBgEIAEoCzIgLmsxczAuc3lzdGVtLmNvbW1vbi'
    '52MS5UaW1lc3RhbXBSCWNyZWF0ZWRBdBI/Cgp1cGRhdGVkX2F0GAUgASgLMiAuazFzMC5zeXN0'
    'ZW0uY29tbW9uLnYxLlRpbWVzdGFtcFIJdXBkYXRlZEF0');

@$core.Deprecated('Use listSecretsRequestDescriptor instead')
const ListSecretsRequest$json = {
  '1': 'ListSecretsRequest',
  '2': [
    {'1': 'prefix', '3': 1, '4': 1, '5': 9, '10': 'prefix'},
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

/// Descriptor for `ListSecretsRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List listSecretsRequestDescriptor = $convert.base64Decode(
    'ChJMaXN0U2VjcmV0c1JlcXVlc3QSFgoGcHJlZml4GAEgASgJUgZwcmVmaXgSQQoKcGFnaW5hdG'
    'lvbhgCIAEoCzIhLmsxczAuc3lzdGVtLmNvbW1vbi52MS5QYWdpbmF0aW9uUgpwYWdpbmF0aW9u');

@$core.Deprecated('Use listSecretsResponseDescriptor instead')
const ListSecretsResponse$json = {
  '1': 'ListSecretsResponse',
  '2': [
    {'1': 'keys', '3': 1, '4': 3, '5': 9, '10': 'keys'},
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

/// Descriptor for `ListSecretsResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List listSecretsResponseDescriptor = $convert.base64Decode(
    'ChNMaXN0U2VjcmV0c1Jlc3BvbnNlEhIKBGtleXMYASADKAlSBGtleXMSRwoKcGFnaW5hdGlvbh'
    'gCIAEoCzInLmsxczAuc3lzdGVtLmNvbW1vbi52MS5QYWdpbmF0aW9uUmVzdWx0UgpwYWdpbmF0'
    'aW9u');

@$core.Deprecated('Use listAuditLogsRequestDescriptor instead')
const ListAuditLogsRequest$json = {
  '1': 'ListAuditLogsRequest',
  '2': [
    {
      '1': 'pagination',
      '3': 3,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.common.v1.Pagination',
      '10': 'pagination'
    },
  ],
  '9': [
    {'1': 1, '2': 2},
    {'1': 2, '2': 3},
  ],
  '10': ['offset', 'limit'],
};

/// Descriptor for `ListAuditLogsRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List listAuditLogsRequestDescriptor = $convert.base64Decode(
    'ChRMaXN0QXVkaXRMb2dzUmVxdWVzdBJBCgpwYWdpbmF0aW9uGAMgASgLMiEuazFzMC5zeXN0ZW'
    '0uY29tbW9uLnYxLlBhZ2luYXRpb25SCnBhZ2luYXRpb25KBAgBEAJKBAgCEANSBm9mZnNldFIF'
    'bGltaXQ=');

@$core.Deprecated('Use listAuditLogsResponseDescriptor instead')
const ListAuditLogsResponse$json = {
  '1': 'ListAuditLogsResponse',
  '2': [
    {
      '1': 'logs',
      '3': 1,
      '4': 3,
      '5': 11,
      '6': '.k1s0.system.vault.v1.AuditLogEntry',
      '10': 'logs'
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

/// Descriptor for `ListAuditLogsResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List listAuditLogsResponseDescriptor = $convert.base64Decode(
    'ChVMaXN0QXVkaXRMb2dzUmVzcG9uc2USNwoEbG9ncxgBIAMoCzIjLmsxczAuc3lzdGVtLnZhdW'
    'x0LnYxLkF1ZGl0TG9nRW50cnlSBGxvZ3MSRwoKcGFnaW5hdGlvbhgCIAEoCzInLmsxczAuc3lz'
    'dGVtLmNvbW1vbi52MS5QYWdpbmF0aW9uUmVzdWx0UgpwYWdpbmF0aW9u');

@$core.Deprecated('Use auditLogEntryDescriptor instead')
const AuditLogEntry$json = {
  '1': 'AuditLogEntry',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 9, '10': 'id'},
    {'1': 'key_path', '3': 2, '4': 1, '5': 9, '10': 'keyPath'},
    {'1': 'action', '3': 3, '4': 1, '5': 9, '10': 'action'},
    {'1': 'actor_id', '3': 4, '4': 1, '5': 9, '10': 'actorId'},
    {'1': 'ip_address', '3': 5, '4': 1, '5': 9, '10': 'ipAddress'},
    {'1': 'success', '3': 6, '4': 1, '5': 8, '10': 'success'},
    {
      '1': 'error_msg',
      '3': 7,
      '4': 1,
      '5': 9,
      '9': 0,
      '10': 'errorMsg',
      '17': true
    },
    {
      '1': 'created_at',
      '3': 8,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.common.v1.Timestamp',
      '10': 'createdAt'
    },
  ],
  '8': [
    {'1': '_error_msg'},
  ],
};

/// Descriptor for `AuditLogEntry`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List auditLogEntryDescriptor = $convert.base64Decode(
    'Cg1BdWRpdExvZ0VudHJ5Eg4KAmlkGAEgASgJUgJpZBIZCghrZXlfcGF0aBgCIAEoCVIHa2V5UG'
    'F0aBIWCgZhY3Rpb24YAyABKAlSBmFjdGlvbhIZCghhY3Rvcl9pZBgEIAEoCVIHYWN0b3JJZBId'
    'CgppcF9hZGRyZXNzGAUgASgJUglpcEFkZHJlc3MSGAoHc3VjY2VzcxgGIAEoCFIHc3VjY2Vzcx'
    'IgCgllcnJvcl9tc2cYByABKAlIAFIIZXJyb3JNc2eIAQESPwoKY3JlYXRlZF9hdBgIIAEoCzIg'
    'LmsxczAuc3lzdGVtLmNvbW1vbi52MS5UaW1lc3RhbXBSCWNyZWF0ZWRBdEIMCgpfZXJyb3JfbX'
    'Nn');
