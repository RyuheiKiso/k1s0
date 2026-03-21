// This is a generated file - do not edit.
//
// Generated from k1s0/system/session/v1/session.proto.

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

@$core.Deprecated('Use createSessionRequestDescriptor instead')
const CreateSessionRequest$json = {
  '1': 'CreateSessionRequest',
  '2': [
    {'1': 'user_id', '3': 1, '4': 1, '5': 9, '10': 'userId'},
    {'1': 'device_id', '3': 2, '4': 1, '5': 9, '10': 'deviceId'},
    {
      '1': 'device_name',
      '3': 3,
      '4': 1,
      '5': 9,
      '9': 0,
      '10': 'deviceName',
      '17': true
    },
    {
      '1': 'device_type',
      '3': 4,
      '4': 1,
      '5': 9,
      '9': 1,
      '10': 'deviceType',
      '17': true
    },
    {
      '1': 'user_agent',
      '3': 5,
      '4': 1,
      '5': 9,
      '9': 2,
      '10': 'userAgent',
      '17': true
    },
    {
      '1': 'ip_address',
      '3': 6,
      '4': 1,
      '5': 9,
      '9': 3,
      '10': 'ipAddress',
      '17': true
    },
    {
      '1': 'ttl_seconds',
      '3': 7,
      '4': 1,
      '5': 13,
      '9': 4,
      '10': 'ttlSeconds',
      '17': true
    },
    {
      '1': 'max_devices',
      '3': 8,
      '4': 1,
      '5': 5,
      '9': 5,
      '10': 'maxDevices',
      '17': true
    },
    {
      '1': 'metadata',
      '3': 9,
      '4': 3,
      '5': 11,
      '6': '.k1s0.system.session.v1.CreateSessionRequest.MetadataEntry',
      '10': 'metadata'
    },
  ],
  '3': [CreateSessionRequest_MetadataEntry$json],
  '8': [
    {'1': '_device_name'},
    {'1': '_device_type'},
    {'1': '_user_agent'},
    {'1': '_ip_address'},
    {'1': '_ttl_seconds'},
    {'1': '_max_devices'},
  ],
};

@$core.Deprecated('Use createSessionRequestDescriptor instead')
const CreateSessionRequest_MetadataEntry$json = {
  '1': 'MetadataEntry',
  '2': [
    {'1': 'key', '3': 1, '4': 1, '5': 9, '10': 'key'},
    {'1': 'value', '3': 2, '4': 1, '5': 9, '10': 'value'},
  ],
  '7': {'7': true},
};

/// Descriptor for `CreateSessionRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List createSessionRequestDescriptor = $convert.base64Decode(
    'ChRDcmVhdGVTZXNzaW9uUmVxdWVzdBIXCgd1c2VyX2lkGAEgASgJUgZ1c2VySWQSGwoJZGV2aW'
    'NlX2lkGAIgASgJUghkZXZpY2VJZBIkCgtkZXZpY2VfbmFtZRgDIAEoCUgAUgpkZXZpY2VOYW1l'
    'iAEBEiQKC2RldmljZV90eXBlGAQgASgJSAFSCmRldmljZVR5cGWIAQESIgoKdXNlcl9hZ2VudB'
    'gFIAEoCUgCUgl1c2VyQWdlbnSIAQESIgoKaXBfYWRkcmVzcxgGIAEoCUgDUglpcEFkZHJlc3OI'
    'AQESJAoLdHRsX3NlY29uZHMYByABKA1IBFIKdHRsU2Vjb25kc4gBARIkCgttYXhfZGV2aWNlcx'
    'gIIAEoBUgFUgptYXhEZXZpY2VziAEBElYKCG1ldGFkYXRhGAkgAygLMjouazFzMC5zeXN0ZW0u'
    'c2Vzc2lvbi52MS5DcmVhdGVTZXNzaW9uUmVxdWVzdC5NZXRhZGF0YUVudHJ5UghtZXRhZGF0YR'
    'o7Cg1NZXRhZGF0YUVudHJ5EhAKA2tleRgBIAEoCVIDa2V5EhQKBXZhbHVlGAIgASgJUgV2YWx1'
    'ZToCOAFCDgoMX2RldmljZV9uYW1lQg4KDF9kZXZpY2VfdHlwZUINCgtfdXNlcl9hZ2VudEINCg'
    'tfaXBfYWRkcmVzc0IOCgxfdHRsX3NlY29uZHNCDgoMX21heF9kZXZpY2Vz');

@$core.Deprecated('Use createSessionResponseDescriptor instead')
const CreateSessionResponse$json = {
  '1': 'CreateSessionResponse',
  '2': [
    {'1': 'session_id', '3': 1, '4': 1, '5': 9, '10': 'sessionId'},
    {'1': 'user_id', '3': 2, '4': 1, '5': 9, '10': 'userId'},
    {'1': 'device_id', '3': 3, '4': 1, '5': 9, '10': 'deviceId'},
    {
      '1': 'expires_at',
      '3': 4,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.common.v1.Timestamp',
      '10': 'expiresAt'
    },
    {
      '1': 'created_at',
      '3': 5,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.common.v1.Timestamp',
      '10': 'createdAt'
    },
    {'1': 'token', '3': 6, '4': 1, '5': 9, '10': 'token'},
    {
      '1': 'metadata',
      '3': 7,
      '4': 3,
      '5': 11,
      '6': '.k1s0.system.session.v1.CreateSessionResponse.MetadataEntry',
      '10': 'metadata'
    },
    {
      '1': 'device_name',
      '3': 8,
      '4': 1,
      '5': 9,
      '9': 0,
      '10': 'deviceName',
      '17': true
    },
    {
      '1': 'device_type',
      '3': 9,
      '4': 1,
      '5': 9,
      '9': 1,
      '10': 'deviceType',
      '17': true
    },
    {
      '1': 'user_agent',
      '3': 10,
      '4': 1,
      '5': 9,
      '9': 2,
      '10': 'userAgent',
      '17': true
    },
    {
      '1': 'ip_address',
      '3': 11,
      '4': 1,
      '5': 9,
      '9': 3,
      '10': 'ipAddress',
      '17': true
    },
    {'1': 'status', '3': 12, '4': 1, '5': 9, '10': 'status'},
  ],
  '3': [CreateSessionResponse_MetadataEntry$json],
  '8': [
    {'1': '_device_name'},
    {'1': '_device_type'},
    {'1': '_user_agent'},
    {'1': '_ip_address'},
  ],
};

@$core.Deprecated('Use createSessionResponseDescriptor instead')
const CreateSessionResponse_MetadataEntry$json = {
  '1': 'MetadataEntry',
  '2': [
    {'1': 'key', '3': 1, '4': 1, '5': 9, '10': 'key'},
    {'1': 'value', '3': 2, '4': 1, '5': 9, '10': 'value'},
  ],
  '7': {'7': true},
};

/// Descriptor for `CreateSessionResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List createSessionResponseDescriptor = $convert.base64Decode(
    'ChVDcmVhdGVTZXNzaW9uUmVzcG9uc2USHQoKc2Vzc2lvbl9pZBgBIAEoCVIJc2Vzc2lvbklkEh'
    'cKB3VzZXJfaWQYAiABKAlSBnVzZXJJZBIbCglkZXZpY2VfaWQYAyABKAlSCGRldmljZUlkEj8K'
    'CmV4cGlyZXNfYXQYBCABKAsyIC5rMXMwLnN5c3RlbS5jb21tb24udjEuVGltZXN0YW1wUglleH'
    'BpcmVzQXQSPwoKY3JlYXRlZF9hdBgFIAEoCzIgLmsxczAuc3lzdGVtLmNvbW1vbi52MS5UaW1l'
    'c3RhbXBSCWNyZWF0ZWRBdBIUCgV0b2tlbhgGIAEoCVIFdG9rZW4SVwoIbWV0YWRhdGEYByADKA'
    'syOy5rMXMwLnN5c3RlbS5zZXNzaW9uLnYxLkNyZWF0ZVNlc3Npb25SZXNwb25zZS5NZXRhZGF0'
    'YUVudHJ5UghtZXRhZGF0YRIkCgtkZXZpY2VfbmFtZRgIIAEoCUgAUgpkZXZpY2VOYW1liAEBEi'
    'QKC2RldmljZV90eXBlGAkgASgJSAFSCmRldmljZVR5cGWIAQESIgoKdXNlcl9hZ2VudBgKIAEo'
    'CUgCUgl1c2VyQWdlbnSIAQESIgoKaXBfYWRkcmVzcxgLIAEoCUgDUglpcEFkZHJlc3OIAQESFg'
    'oGc3RhdHVzGAwgASgJUgZzdGF0dXMaOwoNTWV0YWRhdGFFbnRyeRIQCgNrZXkYASABKAlSA2tl'
    'eRIUCgV2YWx1ZRgCIAEoCVIFdmFsdWU6AjgBQg4KDF9kZXZpY2VfbmFtZUIOCgxfZGV2aWNlX3'
    'R5cGVCDQoLX3VzZXJfYWdlbnRCDQoLX2lwX2FkZHJlc3M=');

@$core.Deprecated('Use getSessionRequestDescriptor instead')
const GetSessionRequest$json = {
  '1': 'GetSessionRequest',
  '2': [
    {'1': 'session_id', '3': 1, '4': 1, '5': 9, '10': 'sessionId'},
  ],
};

/// Descriptor for `GetSessionRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List getSessionRequestDescriptor = $convert.base64Decode(
    'ChFHZXRTZXNzaW9uUmVxdWVzdBIdCgpzZXNzaW9uX2lkGAEgASgJUglzZXNzaW9uSWQ=');

@$core.Deprecated('Use getSessionResponseDescriptor instead')
const GetSessionResponse$json = {
  '1': 'GetSessionResponse',
  '2': [
    {
      '1': 'session',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.session.v1.Session',
      '10': 'session'
    },
  ],
};

/// Descriptor for `GetSessionResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List getSessionResponseDescriptor = $convert.base64Decode(
    'ChJHZXRTZXNzaW9uUmVzcG9uc2USOQoHc2Vzc2lvbhgBIAEoCzIfLmsxczAuc3lzdGVtLnNlc3'
    'Npb24udjEuU2Vzc2lvblIHc2Vzc2lvbg==');

@$core.Deprecated('Use refreshSessionRequestDescriptor instead')
const RefreshSessionRequest$json = {
  '1': 'RefreshSessionRequest',
  '2': [
    {'1': 'session_id', '3': 1, '4': 1, '5': 9, '10': 'sessionId'},
    {
      '1': 'ttl_seconds',
      '3': 2,
      '4': 1,
      '5': 13,
      '9': 0,
      '10': 'ttlSeconds',
      '17': true
    },
  ],
  '8': [
    {'1': '_ttl_seconds'},
  ],
};

/// Descriptor for `RefreshSessionRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List refreshSessionRequestDescriptor = $convert.base64Decode(
    'ChVSZWZyZXNoU2Vzc2lvblJlcXVlc3QSHQoKc2Vzc2lvbl9pZBgBIAEoCVIJc2Vzc2lvbklkEi'
    'QKC3R0bF9zZWNvbmRzGAIgASgNSABSCnR0bFNlY29uZHOIAQFCDgoMX3R0bF9zZWNvbmRz');

@$core.Deprecated('Use refreshSessionResponseDescriptor instead')
const RefreshSessionResponse$json = {
  '1': 'RefreshSessionResponse',
  '2': [
    {'1': 'session_id', '3': 1, '4': 1, '5': 9, '10': 'sessionId'},
    {
      '1': 'expires_at',
      '3': 2,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.common.v1.Timestamp',
      '10': 'expiresAt'
    },
    {'1': 'user_id', '3': 3, '4': 1, '5': 9, '10': 'userId'},
    {'1': 'token', '3': 4, '4': 1, '5': 9, '10': 'token'},
    {'1': 'device_id', '3': 5, '4': 1, '5': 9, '10': 'deviceId'},
    {
      '1': 'device_name',
      '3': 6,
      '4': 1,
      '5': 9,
      '9': 0,
      '10': 'deviceName',
      '17': true
    },
    {
      '1': 'device_type',
      '3': 7,
      '4': 1,
      '5': 9,
      '9': 1,
      '10': 'deviceType',
      '17': true
    },
    {
      '1': 'user_agent',
      '3': 8,
      '4': 1,
      '5': 9,
      '9': 2,
      '10': 'userAgent',
      '17': true
    },
    {
      '1': 'ip_address',
      '3': 9,
      '4': 1,
      '5': 9,
      '9': 3,
      '10': 'ipAddress',
      '17': true
    },
    {
      '1': 'metadata',
      '3': 10,
      '4': 3,
      '5': 11,
      '6': '.k1s0.system.session.v1.RefreshSessionResponse.MetadataEntry',
      '10': 'metadata'
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
      '1': 'last_accessed_at',
      '3': 12,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.common.v1.Timestamp',
      '9': 4,
      '10': 'lastAccessedAt',
      '17': true
    },
    {'1': 'status', '3': 13, '4': 1, '5': 9, '10': 'status'},
  ],
  '3': [RefreshSessionResponse_MetadataEntry$json],
  '8': [
    {'1': '_device_name'},
    {'1': '_device_type'},
    {'1': '_user_agent'},
    {'1': '_ip_address'},
    {'1': '_last_accessed_at'},
  ],
};

@$core.Deprecated('Use refreshSessionResponseDescriptor instead')
const RefreshSessionResponse_MetadataEntry$json = {
  '1': 'MetadataEntry',
  '2': [
    {'1': 'key', '3': 1, '4': 1, '5': 9, '10': 'key'},
    {'1': 'value', '3': 2, '4': 1, '5': 9, '10': 'value'},
  ],
  '7': {'7': true},
};

/// Descriptor for `RefreshSessionResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List refreshSessionResponseDescriptor = $convert.base64Decode(
    'ChZSZWZyZXNoU2Vzc2lvblJlc3BvbnNlEh0KCnNlc3Npb25faWQYASABKAlSCXNlc3Npb25JZB'
    'I/CgpleHBpcmVzX2F0GAIgASgLMiAuazFzMC5zeXN0ZW0uY29tbW9uLnYxLlRpbWVzdGFtcFIJ'
    'ZXhwaXJlc0F0EhcKB3VzZXJfaWQYAyABKAlSBnVzZXJJZBIUCgV0b2tlbhgEIAEoCVIFdG9rZW'
    '4SGwoJZGV2aWNlX2lkGAUgASgJUghkZXZpY2VJZBIkCgtkZXZpY2VfbmFtZRgGIAEoCUgAUgpk'
    'ZXZpY2VOYW1liAEBEiQKC2RldmljZV90eXBlGAcgASgJSAFSCmRldmljZVR5cGWIAQESIgoKdX'
    'Nlcl9hZ2VudBgIIAEoCUgCUgl1c2VyQWdlbnSIAQESIgoKaXBfYWRkcmVzcxgJIAEoCUgDUglp'
    'cEFkZHJlc3OIAQESWAoIbWV0YWRhdGEYCiADKAsyPC5rMXMwLnN5c3RlbS5zZXNzaW9uLnYxLl'
    'JlZnJlc2hTZXNzaW9uUmVzcG9uc2UuTWV0YWRhdGFFbnRyeVIIbWV0YWRhdGESPwoKY3JlYXRl'
    'ZF9hdBgLIAEoCzIgLmsxczAuc3lzdGVtLmNvbW1vbi52MS5UaW1lc3RhbXBSCWNyZWF0ZWRBdB'
    'JPChBsYXN0X2FjY2Vzc2VkX2F0GAwgASgLMiAuazFzMC5zeXN0ZW0uY29tbW9uLnYxLlRpbWVz'
    'dGFtcEgEUg5sYXN0QWNjZXNzZWRBdIgBARIWCgZzdGF0dXMYDSABKAlSBnN0YXR1cxo7Cg1NZX'
    'RhZGF0YUVudHJ5EhAKA2tleRgBIAEoCVIDa2V5EhQKBXZhbHVlGAIgASgJUgV2YWx1ZToCOAFC'
    'DgoMX2RldmljZV9uYW1lQg4KDF9kZXZpY2VfdHlwZUINCgtfdXNlcl9hZ2VudEINCgtfaXBfYW'
    'RkcmVzc0ITChFfbGFzdF9hY2Nlc3NlZF9hdA==');

@$core.Deprecated('Use revokeSessionRequestDescriptor instead')
const RevokeSessionRequest$json = {
  '1': 'RevokeSessionRequest',
  '2': [
    {'1': 'session_id', '3': 1, '4': 1, '5': 9, '10': 'sessionId'},
  ],
};

/// Descriptor for `RevokeSessionRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List revokeSessionRequestDescriptor = $convert.base64Decode(
    'ChRSZXZva2VTZXNzaW9uUmVxdWVzdBIdCgpzZXNzaW9uX2lkGAEgASgJUglzZXNzaW9uSWQ=');

@$core.Deprecated('Use revokeSessionResponseDescriptor instead')
const RevokeSessionResponse$json = {
  '1': 'RevokeSessionResponse',
  '2': [
    {'1': 'success', '3': 1, '4': 1, '5': 8, '10': 'success'},
  ],
};

/// Descriptor for `RevokeSessionResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List revokeSessionResponseDescriptor =
    $convert.base64Decode(
        'ChVSZXZva2VTZXNzaW9uUmVzcG9uc2USGAoHc3VjY2VzcxgBIAEoCFIHc3VjY2Vzcw==');

@$core.Deprecated('Use revokeAllSessionsRequestDescriptor instead')
const RevokeAllSessionsRequest$json = {
  '1': 'RevokeAllSessionsRequest',
  '2': [
    {'1': 'user_id', '3': 1, '4': 1, '5': 9, '10': 'userId'},
  ],
};

/// Descriptor for `RevokeAllSessionsRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List revokeAllSessionsRequestDescriptor =
    $convert.base64Decode(
        'ChhSZXZva2VBbGxTZXNzaW9uc1JlcXVlc3QSFwoHdXNlcl9pZBgBIAEoCVIGdXNlcklk');

@$core.Deprecated('Use revokeAllSessionsResponseDescriptor instead')
const RevokeAllSessionsResponse$json = {
  '1': 'RevokeAllSessionsResponse',
  '2': [
    {'1': 'revoked_count', '3': 1, '4': 1, '5': 13, '10': 'revokedCount'},
  ],
};

/// Descriptor for `RevokeAllSessionsResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List revokeAllSessionsResponseDescriptor =
    $convert.base64Decode(
        'ChlSZXZva2VBbGxTZXNzaW9uc1Jlc3BvbnNlEiMKDXJldm9rZWRfY291bnQYASABKA1SDHJldm'
        '9rZWRDb3VudA==');

@$core.Deprecated('Use listUserSessionsRequestDescriptor instead')
const ListUserSessionsRequest$json = {
  '1': 'ListUserSessionsRequest',
  '2': [
    {'1': 'user_id', '3': 1, '4': 1, '5': 9, '10': 'userId'},
  ],
};

/// Descriptor for `ListUserSessionsRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List listUserSessionsRequestDescriptor =
    $convert.base64Decode(
        'ChdMaXN0VXNlclNlc3Npb25zUmVxdWVzdBIXCgd1c2VyX2lkGAEgASgJUgZ1c2VySWQ=');

@$core.Deprecated('Use listUserSessionsResponseDescriptor instead')
const ListUserSessionsResponse$json = {
  '1': 'ListUserSessionsResponse',
  '2': [
    {
      '1': 'sessions',
      '3': 1,
      '4': 3,
      '5': 11,
      '6': '.k1s0.system.session.v1.Session',
      '10': 'sessions'
    },
    {'1': 'total_count', '3': 2, '4': 1, '5': 13, '10': 'totalCount'},
  ],
};

/// Descriptor for `ListUserSessionsResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List listUserSessionsResponseDescriptor = $convert.base64Decode(
    'ChhMaXN0VXNlclNlc3Npb25zUmVzcG9uc2USOwoIc2Vzc2lvbnMYASADKAsyHy5rMXMwLnN5c3'
    'RlbS5zZXNzaW9uLnYxLlNlc3Npb25SCHNlc3Npb25zEh8KC3RvdGFsX2NvdW50GAIgASgNUgp0'
    'b3RhbENvdW50');

@$core.Deprecated('Use sessionDescriptor instead')
const Session$json = {
  '1': 'Session',
  '2': [
    {'1': 'session_id', '3': 1, '4': 1, '5': 9, '10': 'sessionId'},
    {'1': 'user_id', '3': 2, '4': 1, '5': 9, '10': 'userId'},
    {'1': 'device_id', '3': 3, '4': 1, '5': 9, '10': 'deviceId'},
    {
      '1': 'device_name',
      '3': 4,
      '4': 1,
      '5': 9,
      '9': 0,
      '10': 'deviceName',
      '17': true
    },
    {
      '1': 'device_type',
      '3': 5,
      '4': 1,
      '5': 9,
      '9': 1,
      '10': 'deviceType',
      '17': true
    },
    {
      '1': 'user_agent',
      '3': 6,
      '4': 1,
      '5': 9,
      '9': 2,
      '10': 'userAgent',
      '17': true
    },
    {
      '1': 'ip_address',
      '3': 7,
      '4': 1,
      '5': 9,
      '9': 3,
      '10': 'ipAddress',
      '17': true
    },
    {'1': 'status', '3': 8, '4': 1, '5': 9, '10': 'status'},
    {
      '1': 'expires_at',
      '3': 9,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.common.v1.Timestamp',
      '10': 'expiresAt'
    },
    {
      '1': 'created_at',
      '3': 10,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.common.v1.Timestamp',
      '10': 'createdAt'
    },
    {
      '1': 'last_accessed_at',
      '3': 11,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.common.v1.Timestamp',
      '9': 4,
      '10': 'lastAccessedAt',
      '17': true
    },
    {'1': 'token', '3': 12, '4': 1, '5': 9, '10': 'token'},
  ],
  '8': [
    {'1': '_device_name'},
    {'1': '_device_type'},
    {'1': '_user_agent'},
    {'1': '_ip_address'},
    {'1': '_last_accessed_at'},
  ],
};

/// Descriptor for `Session`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List sessionDescriptor = $convert.base64Decode(
    'CgdTZXNzaW9uEh0KCnNlc3Npb25faWQYASABKAlSCXNlc3Npb25JZBIXCgd1c2VyX2lkGAIgAS'
    'gJUgZ1c2VySWQSGwoJZGV2aWNlX2lkGAMgASgJUghkZXZpY2VJZBIkCgtkZXZpY2VfbmFtZRgE'
    'IAEoCUgAUgpkZXZpY2VOYW1liAEBEiQKC2RldmljZV90eXBlGAUgASgJSAFSCmRldmljZVR5cG'
    'WIAQESIgoKdXNlcl9hZ2VudBgGIAEoCUgCUgl1c2VyQWdlbnSIAQESIgoKaXBfYWRkcmVzcxgH'
    'IAEoCUgDUglpcEFkZHJlc3OIAQESFgoGc3RhdHVzGAggASgJUgZzdGF0dXMSPwoKZXhwaXJlc1'
    '9hdBgJIAEoCzIgLmsxczAuc3lzdGVtLmNvbW1vbi52MS5UaW1lc3RhbXBSCWV4cGlyZXNBdBI/'
    'CgpjcmVhdGVkX2F0GAogASgLMiAuazFzMC5zeXN0ZW0uY29tbW9uLnYxLlRpbWVzdGFtcFIJY3'
    'JlYXRlZEF0Ek8KEGxhc3RfYWNjZXNzZWRfYXQYCyABKAsyIC5rMXMwLnN5c3RlbS5jb21tb24u'
    'djEuVGltZXN0YW1wSARSDmxhc3RBY2Nlc3NlZEF0iAEBEhQKBXRva2VuGAwgASgJUgV0b2tlbk'
    'IOCgxfZGV2aWNlX25hbWVCDgoMX2RldmljZV90eXBlQg0KC191c2VyX2FnZW50Qg0KC19pcF9h'
    'ZGRyZXNzQhMKEV9sYXN0X2FjY2Vzc2VkX2F0');
