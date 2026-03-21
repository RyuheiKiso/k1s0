// This is a generated file - do not edit.
//
// Generated from k1s0/event/system/auth/v1/auth_events.proto.

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

@$core.Deprecated('Use loginEventDescriptor instead')
const LoginEvent$json = {
  '1': 'LoginEvent',
  '2': [
    {
      '1': 'metadata',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.common.v1.EventMetadata',
      '10': 'metadata'
    },
    {'1': 'user_id', '3': 2, '4': 1, '5': 9, '10': 'userId'},
    {'1': 'username', '3': 3, '4': 1, '5': 9, '10': 'username'},
    {'1': 'client_id', '3': 4, '4': 1, '5': 9, '10': 'clientId'},
    {'1': 'ip_address', '3': 5, '4': 1, '5': 9, '10': 'ipAddress'},
    {'1': 'user_agent', '3': 6, '4': 1, '5': 9, '10': 'userAgent'},
    {'1': 'result', '3': 7, '4': 1, '5': 9, '10': 'result'},
    {'1': 'failure_reason', '3': 8, '4': 1, '5': 9, '10': 'failureReason'},
  ],
};

/// Descriptor for `LoginEvent`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List loginEventDescriptor = $convert.base64Decode(
    'CgpMb2dpbkV2ZW50EkAKCG1ldGFkYXRhGAEgASgLMiQuazFzMC5zeXN0ZW0uY29tbW9uLnYxLk'
    'V2ZW50TWV0YWRhdGFSCG1ldGFkYXRhEhcKB3VzZXJfaWQYAiABKAlSBnVzZXJJZBIaCgh1c2Vy'
    'bmFtZRgDIAEoCVIIdXNlcm5hbWUSGwoJY2xpZW50X2lkGAQgASgJUghjbGllbnRJZBIdCgppcF'
    '9hZGRyZXNzGAUgASgJUglpcEFkZHJlc3MSHQoKdXNlcl9hZ2VudBgGIAEoCVIJdXNlckFnZW50'
    'EhYKBnJlc3VsdBgHIAEoCVIGcmVzdWx0EiUKDmZhaWx1cmVfcmVhc29uGAggASgJUg1mYWlsdX'
    'JlUmVhc29u');

@$core.Deprecated('Use tokenValidationEventDescriptor instead')
const TokenValidationEvent$json = {
  '1': 'TokenValidationEvent',
  '2': [
    {
      '1': 'metadata',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.common.v1.EventMetadata',
      '10': 'metadata'
    },
    {'1': 'user_id', '3': 2, '4': 1, '5': 9, '10': 'userId'},
    {'1': 'token_jti', '3': 3, '4': 1, '5': 9, '10': 'tokenJti'},
    {'1': 'valid', '3': 4, '4': 1, '5': 8, '10': 'valid'},
    {'1': 'error_message', '3': 5, '4': 1, '5': 9, '10': 'errorMessage'},
  ],
};

/// Descriptor for `TokenValidationEvent`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List tokenValidationEventDescriptor = $convert.base64Decode(
    'ChRUb2tlblZhbGlkYXRpb25FdmVudBJACghtZXRhZGF0YRgBIAEoCzIkLmsxczAuc3lzdGVtLm'
    'NvbW1vbi52MS5FdmVudE1ldGFkYXRhUghtZXRhZGF0YRIXCgd1c2VyX2lkGAIgASgJUgZ1c2Vy'
    'SWQSGwoJdG9rZW5fanRpGAMgASgJUgh0b2tlbkp0aRIUCgV2YWxpZBgEIAEoCFIFdmFsaWQSIw'
    'oNZXJyb3JfbWVzc2FnZRgFIAEoCVIMZXJyb3JNZXNzYWdl');

@$core.Deprecated('Use permissionCheckEventDescriptor instead')
const PermissionCheckEvent$json = {
  '1': 'PermissionCheckEvent',
  '2': [
    {
      '1': 'metadata',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.common.v1.EventMetadata',
      '10': 'metadata'
    },
    {'1': 'user_id', '3': 2, '4': 1, '5': 9, '10': 'userId'},
    {'1': 'permission', '3': 3, '4': 1, '5': 9, '10': 'permission'},
    {'1': 'resource', '3': 4, '4': 1, '5': 9, '10': 'resource'},
    {'1': 'roles', '3': 5, '4': 3, '5': 9, '10': 'roles'},
    {'1': 'allowed', '3': 6, '4': 1, '5': 8, '10': 'allowed'},
    {'1': 'reason', '3': 7, '4': 1, '5': 9, '10': 'reason'},
  ],
};

/// Descriptor for `PermissionCheckEvent`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List permissionCheckEventDescriptor = $convert.base64Decode(
    'ChRQZXJtaXNzaW9uQ2hlY2tFdmVudBJACghtZXRhZGF0YRgBIAEoCzIkLmsxczAuc3lzdGVtLm'
    'NvbW1vbi52MS5FdmVudE1ldGFkYXRhUghtZXRhZGF0YRIXCgd1c2VyX2lkGAIgASgJUgZ1c2Vy'
    'SWQSHgoKcGVybWlzc2lvbhgDIAEoCVIKcGVybWlzc2lvbhIaCghyZXNvdXJjZRgEIAEoCVIIcm'
    'Vzb3VyY2USFAoFcm9sZXMYBSADKAlSBXJvbGVzEhgKB2FsbG93ZWQYBiABKAhSB2FsbG93ZWQS'
    'FgoGcmVhc29uGAcgASgJUgZyZWFzb24=');

@$core.Deprecated('Use auditLogRecordedEventDescriptor instead')
const AuditLogRecordedEvent$json = {
  '1': 'AuditLogRecordedEvent',
  '2': [
    {
      '1': 'metadata',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.common.v1.EventMetadata',
      '10': 'metadata'
    },
    {'1': 'audit_log_id', '3': 2, '4': 1, '5': 9, '10': 'auditLogId'},
    {'1': 'event_type', '3': 3, '4': 1, '5': 9, '10': 'eventType'},
    {'1': 'user_id', '3': 4, '4': 1, '5': 9, '10': 'userId'},
    {'1': 'ip_address', '3': 5, '4': 1, '5': 9, '10': 'ipAddress'},
    {'1': 'resource', '3': 6, '4': 1, '5': 9, '10': 'resource'},
    {'1': 'action', '3': 7, '4': 1, '5': 9, '10': 'action'},
    {'1': 'result', '3': 8, '4': 1, '5': 9, '10': 'result'},
  ],
};

/// Descriptor for `AuditLogRecordedEvent`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List auditLogRecordedEventDescriptor = $convert.base64Decode(
    'ChVBdWRpdExvZ1JlY29yZGVkRXZlbnQSQAoIbWV0YWRhdGEYASABKAsyJC5rMXMwLnN5c3RlbS'
    '5jb21tb24udjEuRXZlbnRNZXRhZGF0YVIIbWV0YWRhdGESIAoMYXVkaXRfbG9nX2lkGAIgASgJ'
    'UgphdWRpdExvZ0lkEh0KCmV2ZW50X3R5cGUYAyABKAlSCWV2ZW50VHlwZRIXCgd1c2VyX2lkGA'
    'QgASgJUgZ1c2VySWQSHQoKaXBfYWRkcmVzcxgFIAEoCVIJaXBBZGRyZXNzEhoKCHJlc291cmNl'
    'GAYgASgJUghyZXNvdXJjZRIWCgZhY3Rpb24YByABKAlSBmFjdGlvbhIWCgZyZXN1bHQYCCABKA'
    'lSBnJlc3VsdA==');
