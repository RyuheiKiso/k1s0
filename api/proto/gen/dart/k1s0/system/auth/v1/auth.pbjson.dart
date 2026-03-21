// This is a generated file - do not edit.
//
// Generated from k1s0/system/auth/v1/auth.proto.

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

@$core.Deprecated('Use auditEventTypeDescriptor instead')
const AuditEventType$json = {
  '1': 'AuditEventType',
  '2': [
    {'1': 'AUDIT_EVENT_TYPE_UNSPECIFIED', '2': 0},
    {'1': 'AUDIT_EVENT_TYPE_LOGIN', '2': 1},
    {'1': 'AUDIT_EVENT_TYPE_LOGOUT', '2': 2},
    {'1': 'AUDIT_EVENT_TYPE_TOKEN_REFRESH', '2': 3},
    {'1': 'AUDIT_EVENT_TYPE_PERMISSION_CHECK', '2': 4},
    {'1': 'AUDIT_EVENT_TYPE_API_KEY_CREATED', '2': 5},
    {'1': 'AUDIT_EVENT_TYPE_API_KEY_REVOKED', '2': 6},
  ],
};

/// Descriptor for `AuditEventType`. Decode as a `google.protobuf.EnumDescriptorProto`.
final $typed_data.Uint8List auditEventTypeDescriptor = $convert.base64Decode(
    'Cg5BdWRpdEV2ZW50VHlwZRIgChxBVURJVF9FVkVOVF9UWVBFX1VOU1BFQ0lGSUVEEAASGgoWQV'
    'VESVRfRVZFTlRfVFlQRV9MT0dJThABEhsKF0FVRElUX0VWRU5UX1RZUEVfTE9HT1VUEAISIgoe'
    'QVVESVRfRVZFTlRfVFlQRV9UT0tFTl9SRUZSRVNIEAMSJQohQVVESVRfRVZFTlRfVFlQRV9QRV'
    'JNSVNTSU9OX0NIRUNLEAQSJAogQVVESVRfRVZFTlRfVFlQRV9BUElfS0VZX0NSRUFURUQQBRIk'
    'CiBBVURJVF9FVkVOVF9UWVBFX0FQSV9LRVlfUkVWT0tFRBAG');

@$core.Deprecated('Use auditResultDescriptor instead')
const AuditResult$json = {
  '1': 'AuditResult',
  '2': [
    {'1': 'AUDIT_RESULT_UNSPECIFIED', '2': 0},
    {'1': 'AUDIT_RESULT_SUCCESS', '2': 1},
    {'1': 'AUDIT_RESULT_FAILURE', '2': 2},
  ],
};

/// Descriptor for `AuditResult`. Decode as a `google.protobuf.EnumDescriptorProto`.
final $typed_data.Uint8List auditResultDescriptor = $convert.base64Decode(
    'CgtBdWRpdFJlc3VsdBIcChhBVURJVF9SRVNVTFRfVU5TUEVDSUZJRUQQABIYChRBVURJVF9SRV'
    'NVTFRfU1VDQ0VTUxABEhgKFEFVRElUX1JFU1VMVF9GQUlMVVJFEAI=');

@$core.Deprecated('Use validateTokenRequestDescriptor instead')
const ValidateTokenRequest$json = {
  '1': 'ValidateTokenRequest',
  '2': [
    {'1': 'token', '3': 1, '4': 1, '5': 9, '10': 'token'},
  ],
};

/// Descriptor for `ValidateTokenRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List validateTokenRequestDescriptor =
    $convert.base64Decode(
        'ChRWYWxpZGF0ZVRva2VuUmVxdWVzdBIUCgV0b2tlbhgBIAEoCVIFdG9rZW4=');

@$core.Deprecated('Use validateTokenResponseDescriptor instead')
const ValidateTokenResponse$json = {
  '1': 'ValidateTokenResponse',
  '2': [
    {'1': 'valid', '3': 1, '4': 1, '5': 8, '10': 'valid'},
    {
      '1': 'claims',
      '3': 2,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.auth.v1.TokenClaims',
      '10': 'claims'
    },
    {'1': 'error_message', '3': 3, '4': 1, '5': 9, '10': 'errorMessage'},
  ],
};

/// Descriptor for `ValidateTokenResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List validateTokenResponseDescriptor = $convert.base64Decode(
    'ChVWYWxpZGF0ZVRva2VuUmVzcG9uc2USFAoFdmFsaWQYASABKAhSBXZhbGlkEjgKBmNsYWltcx'
    'gCIAEoCzIgLmsxczAuc3lzdGVtLmF1dGgudjEuVG9rZW5DbGFpbXNSBmNsYWltcxIjCg1lcnJv'
    'cl9tZXNzYWdlGAMgASgJUgxlcnJvck1lc3NhZ2U=');

@$core.Deprecated('Use tokenClaimsDescriptor instead')
const TokenClaims$json = {
  '1': 'TokenClaims',
  '2': [
    {'1': 'sub', '3': 1, '4': 1, '5': 9, '10': 'sub'},
    {'1': 'iss', '3': 2, '4': 1, '5': 9, '10': 'iss'},
    {'1': 'aud', '3': 3, '4': 1, '5': 9, '10': 'aud'},
    {'1': 'exp', '3': 4, '4': 1, '5': 3, '10': 'exp'},
    {'1': 'iat', '3': 5, '4': 1, '5': 3, '10': 'iat'},
    {'1': 'jti', '3': 6, '4': 1, '5': 9, '10': 'jti'},
    {
      '1': 'preferred_username',
      '3': 7,
      '4': 1,
      '5': 9,
      '10': 'preferredUsername'
    },
    {'1': 'email', '3': 8, '4': 1, '5': 9, '10': 'email'},
    {
      '1': 'realm_access',
      '3': 9,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.auth.v1.RealmAccess',
      '10': 'realmAccess'
    },
    {
      '1': 'resource_access',
      '3': 10,
      '4': 3,
      '5': 11,
      '6': '.k1s0.system.auth.v1.TokenClaims.ResourceAccessEntry',
      '10': 'resourceAccess'
    },
    {'1': 'tier_access', '3': 11, '4': 3, '5': 9, '10': 'tierAccess'},
    {'1': 'scope', '3': 12, '4': 1, '5': 9, '10': 'scope'},
    {'1': 'typ', '3': 13, '4': 1, '5': 9, '9': 0, '10': 'typ', '17': true},
    {'1': 'azp', '3': 14, '4': 1, '5': 9, '9': 1, '10': 'azp', '17': true},
  ],
  '3': [TokenClaims_ResourceAccessEntry$json],
  '8': [
    {'1': '_typ'},
    {'1': '_azp'},
  ],
};

@$core.Deprecated('Use tokenClaimsDescriptor instead')
const TokenClaims_ResourceAccessEntry$json = {
  '1': 'ResourceAccessEntry',
  '2': [
    {'1': 'key', '3': 1, '4': 1, '5': 9, '10': 'key'},
    {
      '1': 'value',
      '3': 2,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.auth.v1.ClientRoles',
      '10': 'value'
    },
  ],
  '7': {'7': true},
};

/// Descriptor for `TokenClaims`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List tokenClaimsDescriptor = $convert.base64Decode(
    'CgtUb2tlbkNsYWltcxIQCgNzdWIYASABKAlSA3N1YhIQCgNpc3MYAiABKAlSA2lzcxIQCgNhdW'
    'QYAyABKAlSA2F1ZBIQCgNleHAYBCABKANSA2V4cBIQCgNpYXQYBSABKANSA2lhdBIQCgNqdGkY'
    'BiABKAlSA2p0aRItChJwcmVmZXJyZWRfdXNlcm5hbWUYByABKAlSEXByZWZlcnJlZFVzZXJuYW'
    '1lEhQKBWVtYWlsGAggASgJUgVlbWFpbBJDCgxyZWFsbV9hY2Nlc3MYCSABKAsyIC5rMXMwLnN5'
    'c3RlbS5hdXRoLnYxLlJlYWxtQWNjZXNzUgtyZWFsbUFjY2VzcxJdCg9yZXNvdXJjZV9hY2Nlc3'
    'MYCiADKAsyNC5rMXMwLnN5c3RlbS5hdXRoLnYxLlRva2VuQ2xhaW1zLlJlc291cmNlQWNjZXNz'
    'RW50cnlSDnJlc291cmNlQWNjZXNzEh8KC3RpZXJfYWNjZXNzGAsgAygJUgp0aWVyQWNjZXNzEh'
    'QKBXNjb3BlGAwgASgJUgVzY29wZRIVCgN0eXAYDSABKAlIAFIDdHlwiAEBEhUKA2F6cBgOIAEo'
    'CUgBUgNhenCIAQEaYwoTUmVzb3VyY2VBY2Nlc3NFbnRyeRIQCgNrZXkYASABKAlSA2tleRI2Cg'
    'V2YWx1ZRgCIAEoCzIgLmsxczAuc3lzdGVtLmF1dGgudjEuQ2xpZW50Um9sZXNSBXZhbHVlOgI4'
    'AUIGCgRfdHlwQgYKBF9henA=');

@$core.Deprecated('Use realmAccessDescriptor instead')
const RealmAccess$json = {
  '1': 'RealmAccess',
  '2': [
    {'1': 'roles', '3': 1, '4': 3, '5': 9, '10': 'roles'},
  ],
};

/// Descriptor for `RealmAccess`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List realmAccessDescriptor =
    $convert.base64Decode('CgtSZWFsbUFjY2VzcxIUCgVyb2xlcxgBIAMoCVIFcm9sZXM=');

@$core.Deprecated('Use clientRolesDescriptor instead')
const ClientRoles$json = {
  '1': 'ClientRoles',
  '2': [
    {'1': 'roles', '3': 1, '4': 3, '5': 9, '10': 'roles'},
  ],
};

/// Descriptor for `ClientRoles`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List clientRolesDescriptor =
    $convert.base64Decode('CgtDbGllbnRSb2xlcxIUCgVyb2xlcxgBIAMoCVIFcm9sZXM=');

@$core.Deprecated('Use getUserRequestDescriptor instead')
const GetUserRequest$json = {
  '1': 'GetUserRequest',
  '2': [
    {'1': 'user_id', '3': 1, '4': 1, '5': 9, '10': 'userId'},
  ],
};

/// Descriptor for `GetUserRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List getUserRequestDescriptor = $convert
    .base64Decode('Cg5HZXRVc2VyUmVxdWVzdBIXCgd1c2VyX2lkGAEgASgJUgZ1c2VySWQ=');

@$core.Deprecated('Use getUserResponseDescriptor instead')
const GetUserResponse$json = {
  '1': 'GetUserResponse',
  '2': [
    {
      '1': 'user',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.auth.v1.User',
      '10': 'user'
    },
  ],
};

/// Descriptor for `GetUserResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List getUserResponseDescriptor = $convert.base64Decode(
    'Cg9HZXRVc2VyUmVzcG9uc2USLQoEdXNlchgBIAEoCzIZLmsxczAuc3lzdGVtLmF1dGgudjEuVX'
    'NlclIEdXNlcg==');

@$core.Deprecated('Use listUsersRequestDescriptor instead')
const ListUsersRequest$json = {
  '1': 'ListUsersRequest',
  '2': [
    {
      '1': 'pagination',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.common.v1.Pagination',
      '10': 'pagination'
    },
    {'1': 'search', '3': 2, '4': 1, '5': 9, '10': 'search'},
    {
      '1': 'enabled',
      '3': 3,
      '4': 1,
      '5': 8,
      '9': 0,
      '10': 'enabled',
      '17': true
    },
  ],
  '8': [
    {'1': '_enabled'},
  ],
};

/// Descriptor for `ListUsersRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List listUsersRequestDescriptor = $convert.base64Decode(
    'ChBMaXN0VXNlcnNSZXF1ZXN0EkEKCnBhZ2luYXRpb24YASABKAsyIS5rMXMwLnN5c3RlbS5jb2'
    '1tb24udjEuUGFnaW5hdGlvblIKcGFnaW5hdGlvbhIWCgZzZWFyY2gYAiABKAlSBnNlYXJjaBId'
    'CgdlbmFibGVkGAMgASgISABSB2VuYWJsZWSIAQFCCgoIX2VuYWJsZWQ=');

@$core.Deprecated('Use listUsersResponseDescriptor instead')
const ListUsersResponse$json = {
  '1': 'ListUsersResponse',
  '2': [
    {
      '1': 'users',
      '3': 1,
      '4': 3,
      '5': 11,
      '6': '.k1s0.system.auth.v1.User',
      '10': 'users'
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

/// Descriptor for `ListUsersResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List listUsersResponseDescriptor = $convert.base64Decode(
    'ChFMaXN0VXNlcnNSZXNwb25zZRIvCgV1c2VycxgBIAMoCzIZLmsxczAuc3lzdGVtLmF1dGgudj'
    'EuVXNlclIFdXNlcnMSRwoKcGFnaW5hdGlvbhgCIAEoCzInLmsxczAuc3lzdGVtLmNvbW1vbi52'
    'MS5QYWdpbmF0aW9uUmVzdWx0UgpwYWdpbmF0aW9u');

@$core.Deprecated('Use userDescriptor instead')
const User$json = {
  '1': 'User',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 9, '10': 'id'},
    {'1': 'username', '3': 2, '4': 1, '5': 9, '10': 'username'},
    {'1': 'email', '3': 3, '4': 1, '5': 9, '10': 'email'},
    {'1': 'first_name', '3': 4, '4': 1, '5': 9, '10': 'firstName'},
    {'1': 'last_name', '3': 5, '4': 1, '5': 9, '10': 'lastName'},
    {'1': 'enabled', '3': 6, '4': 1, '5': 8, '10': 'enabled'},
    {'1': 'email_verified', '3': 7, '4': 1, '5': 8, '10': 'emailVerified'},
    {
      '1': 'created_at',
      '3': 8,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.common.v1.Timestamp',
      '10': 'createdAt'
    },
    {
      '1': 'attributes',
      '3': 9,
      '4': 3,
      '5': 11,
      '6': '.k1s0.system.auth.v1.User.AttributesEntry',
      '10': 'attributes'
    },
  ],
  '3': [User_AttributesEntry$json],
};

@$core.Deprecated('Use userDescriptor instead')
const User_AttributesEntry$json = {
  '1': 'AttributesEntry',
  '2': [
    {'1': 'key', '3': 1, '4': 1, '5': 9, '10': 'key'},
    {
      '1': 'value',
      '3': 2,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.auth.v1.StringList',
      '10': 'value'
    },
  ],
  '7': {'7': true},
};

/// Descriptor for `User`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List userDescriptor = $convert.base64Decode(
    'CgRVc2VyEg4KAmlkGAEgASgJUgJpZBIaCgh1c2VybmFtZRgCIAEoCVIIdXNlcm5hbWUSFAoFZW'
    '1haWwYAyABKAlSBWVtYWlsEh0KCmZpcnN0X25hbWUYBCABKAlSCWZpcnN0TmFtZRIbCglsYXN0'
    'X25hbWUYBSABKAlSCGxhc3ROYW1lEhgKB2VuYWJsZWQYBiABKAhSB2VuYWJsZWQSJQoOZW1haW'
    'xfdmVyaWZpZWQYByABKAhSDWVtYWlsVmVyaWZpZWQSPwoKY3JlYXRlZF9hdBgIIAEoCzIgLmsx'
    'czAuc3lzdGVtLmNvbW1vbi52MS5UaW1lc3RhbXBSCWNyZWF0ZWRBdBJJCgphdHRyaWJ1dGVzGA'
    'kgAygLMikuazFzMC5zeXN0ZW0uYXV0aC52MS5Vc2VyLkF0dHJpYnV0ZXNFbnRyeVIKYXR0cmli'
    'dXRlcxpeCg9BdHRyaWJ1dGVzRW50cnkSEAoDa2V5GAEgASgJUgNrZXkSNQoFdmFsdWUYAiABKA'
    'syHy5rMXMwLnN5c3RlbS5hdXRoLnYxLlN0cmluZ0xpc3RSBXZhbHVlOgI4AQ==');

@$core.Deprecated('Use stringListDescriptor instead')
const StringList$json = {
  '1': 'StringList',
  '2': [
    {'1': 'values', '3': 1, '4': 3, '5': 9, '10': 'values'},
  ],
};

/// Descriptor for `StringList`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List stringListDescriptor =
    $convert.base64Decode('CgpTdHJpbmdMaXN0EhYKBnZhbHVlcxgBIAMoCVIGdmFsdWVz');

@$core.Deprecated('Use getUserRolesRequestDescriptor instead')
const GetUserRolesRequest$json = {
  '1': 'GetUserRolesRequest',
  '2': [
    {'1': 'user_id', '3': 1, '4': 1, '5': 9, '10': 'userId'},
  ],
};

/// Descriptor for `GetUserRolesRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List getUserRolesRequestDescriptor =
    $convert.base64Decode(
        'ChNHZXRVc2VyUm9sZXNSZXF1ZXN0EhcKB3VzZXJfaWQYASABKAlSBnVzZXJJZA==');

@$core.Deprecated('Use getUserRolesResponseDescriptor instead')
const GetUserRolesResponse$json = {
  '1': 'GetUserRolesResponse',
  '2': [
    {'1': 'user_id', '3': 1, '4': 1, '5': 9, '10': 'userId'},
    {
      '1': 'realm_roles',
      '3': 2,
      '4': 3,
      '5': 11,
      '6': '.k1s0.system.auth.v1.Role',
      '10': 'realmRoles'
    },
    {
      '1': 'client_roles',
      '3': 3,
      '4': 3,
      '5': 11,
      '6': '.k1s0.system.auth.v1.GetUserRolesResponse.ClientRolesEntry',
      '10': 'clientRoles'
    },
  ],
  '3': [GetUserRolesResponse_ClientRolesEntry$json],
};

@$core.Deprecated('Use getUserRolesResponseDescriptor instead')
const GetUserRolesResponse_ClientRolesEntry$json = {
  '1': 'ClientRolesEntry',
  '2': [
    {'1': 'key', '3': 1, '4': 1, '5': 9, '10': 'key'},
    {
      '1': 'value',
      '3': 2,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.auth.v1.RoleList',
      '10': 'value'
    },
  ],
  '7': {'7': true},
};

/// Descriptor for `GetUserRolesResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List getUserRolesResponseDescriptor = $convert.base64Decode(
    'ChRHZXRVc2VyUm9sZXNSZXNwb25zZRIXCgd1c2VyX2lkGAEgASgJUgZ1c2VySWQSOgoLcmVhbG'
    '1fcm9sZXMYAiADKAsyGS5rMXMwLnN5c3RlbS5hdXRoLnYxLlJvbGVSCnJlYWxtUm9sZXMSXQoM'
    'Y2xpZW50X3JvbGVzGAMgAygLMjouazFzMC5zeXN0ZW0uYXV0aC52MS5HZXRVc2VyUm9sZXNSZX'
    'Nwb25zZS5DbGllbnRSb2xlc0VudHJ5UgtjbGllbnRSb2xlcxpdChBDbGllbnRSb2xlc0VudHJ5'
    'EhAKA2tleRgBIAEoCVIDa2V5EjMKBXZhbHVlGAIgASgLMh0uazFzMC5zeXN0ZW0uYXV0aC52MS'
    '5Sb2xlTGlzdFIFdmFsdWU6AjgB');

@$core.Deprecated('Use roleDescriptor instead')
const Role$json = {
  '1': 'Role',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 9, '10': 'id'},
    {'1': 'name', '3': 2, '4': 1, '5': 9, '10': 'name'},
    {'1': 'description', '3': 3, '4': 1, '5': 9, '10': 'description'},
  ],
};

/// Descriptor for `Role`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List roleDescriptor = $convert.base64Decode(
    'CgRSb2xlEg4KAmlkGAEgASgJUgJpZBISCgRuYW1lGAIgASgJUgRuYW1lEiAKC2Rlc2NyaXB0aW'
    '9uGAMgASgJUgtkZXNjcmlwdGlvbg==');

@$core.Deprecated('Use roleListDescriptor instead')
const RoleList$json = {
  '1': 'RoleList',
  '2': [
    {
      '1': 'roles',
      '3': 1,
      '4': 3,
      '5': 11,
      '6': '.k1s0.system.auth.v1.Role',
      '10': 'roles'
    },
  ],
};

/// Descriptor for `RoleList`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List roleListDescriptor = $convert.base64Decode(
    'CghSb2xlTGlzdBIvCgVyb2xlcxgBIAMoCzIZLmsxczAuc3lzdGVtLmF1dGgudjEuUm9sZVIFcm'
    '9sZXM=');

@$core.Deprecated('Use checkPermissionRequestDescriptor instead')
const CheckPermissionRequest$json = {
  '1': 'CheckPermissionRequest',
  '2': [
    {
      '1': 'user_id',
      '3': 1,
      '4': 1,
      '5': 9,
      '9': 0,
      '10': 'userId',
      '17': true
    },
    {'1': 'permission', '3': 2, '4': 1, '5': 9, '10': 'permission'},
    {'1': 'resource', '3': 3, '4': 1, '5': 9, '10': 'resource'},
    {'1': 'roles', '3': 4, '4': 3, '5': 9, '10': 'roles'},
  ],
  '8': [
    {'1': '_user_id'},
  ],
};

/// Descriptor for `CheckPermissionRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List checkPermissionRequestDescriptor = $convert.base64Decode(
    'ChZDaGVja1Blcm1pc3Npb25SZXF1ZXN0EhwKB3VzZXJfaWQYASABKAlIAFIGdXNlcklkiAEBEh'
    '4KCnBlcm1pc3Npb24YAiABKAlSCnBlcm1pc3Npb24SGgoIcmVzb3VyY2UYAyABKAlSCHJlc291'
    'cmNlEhQKBXJvbGVzGAQgAygJUgVyb2xlc0IKCghfdXNlcl9pZA==');

@$core.Deprecated('Use checkPermissionResponseDescriptor instead')
const CheckPermissionResponse$json = {
  '1': 'CheckPermissionResponse',
  '2': [
    {'1': 'allowed', '3': 1, '4': 1, '5': 8, '10': 'allowed'},
    {'1': 'reason', '3': 2, '4': 1, '5': 9, '10': 'reason'},
  ],
};

/// Descriptor for `CheckPermissionResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List checkPermissionResponseDescriptor =
    $convert.base64Decode(
        'ChdDaGVja1Blcm1pc3Npb25SZXNwb25zZRIYCgdhbGxvd2VkGAEgASgIUgdhbGxvd2VkEhYKBn'
        'JlYXNvbhgCIAEoCVIGcmVhc29u');

@$core.Deprecated('Use recordAuditLogRequestDescriptor instead')
const RecordAuditLogRequest$json = {
  '1': 'RecordAuditLogRequest',
  '2': [
    {'1': 'event_type', '3': 1, '4': 1, '5': 9, '10': 'eventType'},
    {'1': 'user_id', '3': 2, '4': 1, '5': 9, '10': 'userId'},
    {'1': 'ip_address', '3': 3, '4': 1, '5': 9, '10': 'ipAddress'},
    {'1': 'user_agent', '3': 4, '4': 1, '5': 9, '10': 'userAgent'},
    {'1': 'resource', '3': 5, '4': 1, '5': 9, '10': 'resource'},
    {'1': 'action', '3': 6, '4': 1, '5': 9, '10': 'action'},
    {'1': 'result', '3': 7, '4': 1, '5': 9, '10': 'result'},
    {
      '1': 'detail',
      '3': 8,
      '4': 1,
      '5': 11,
      '6': '.google.protobuf.Struct',
      '10': 'detail'
    },
    {'1': 'resource_id', '3': 9, '4': 1, '5': 9, '10': 'resourceId'},
    {'1': 'trace_id', '3': 10, '4': 1, '5': 9, '10': 'traceId'},
    {
      '1': 'event_type_enum',
      '3': 11,
      '4': 1,
      '5': 14,
      '6': '.k1s0.system.auth.v1.AuditEventType',
      '10': 'eventTypeEnum'
    },
    {
      '1': 'result_enum',
      '3': 12,
      '4': 1,
      '5': 14,
      '6': '.k1s0.system.auth.v1.AuditResult',
      '10': 'resultEnum'
    },
  ],
};

/// Descriptor for `RecordAuditLogRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List recordAuditLogRequestDescriptor = $convert.base64Decode(
    'ChVSZWNvcmRBdWRpdExvZ1JlcXVlc3QSHQoKZXZlbnRfdHlwZRgBIAEoCVIJZXZlbnRUeXBlEh'
    'cKB3VzZXJfaWQYAiABKAlSBnVzZXJJZBIdCgppcF9hZGRyZXNzGAMgASgJUglpcEFkZHJlc3MS'
    'HQoKdXNlcl9hZ2VudBgEIAEoCVIJdXNlckFnZW50EhoKCHJlc291cmNlGAUgASgJUghyZXNvdX'
    'JjZRIWCgZhY3Rpb24YBiABKAlSBmFjdGlvbhIWCgZyZXN1bHQYByABKAlSBnJlc3VsdBIvCgZk'
    'ZXRhaWwYCCABKAsyFy5nb29nbGUucHJvdG9idWYuU3RydWN0UgZkZXRhaWwSHwoLcmVzb3VyY2'
    'VfaWQYCSABKAlSCnJlc291cmNlSWQSGQoIdHJhY2VfaWQYCiABKAlSB3RyYWNlSWQSSwoPZXZl'
    'bnRfdHlwZV9lbnVtGAsgASgOMiMuazFzMC5zeXN0ZW0uYXV0aC52MS5BdWRpdEV2ZW50VHlwZV'
    'INZXZlbnRUeXBlRW51bRJBCgtyZXN1bHRfZW51bRgMIAEoDjIgLmsxczAuc3lzdGVtLmF1dGgu'
    'djEuQXVkaXRSZXN1bHRSCnJlc3VsdEVudW0=');

@$core.Deprecated('Use recordAuditLogResponseDescriptor instead')
const RecordAuditLogResponse$json = {
  '1': 'RecordAuditLogResponse',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 9, '10': 'id'},
    {
      '1': 'created_at',
      '3': 2,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.common.v1.Timestamp',
      '10': 'createdAt'
    },
  ],
};

/// Descriptor for `RecordAuditLogResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List recordAuditLogResponseDescriptor =
    $convert.base64Decode(
        'ChZSZWNvcmRBdWRpdExvZ1Jlc3BvbnNlEg4KAmlkGAEgASgJUgJpZBI/CgpjcmVhdGVkX2F0GA'
        'IgASgLMiAuazFzMC5zeXN0ZW0uY29tbW9uLnYxLlRpbWVzdGFtcFIJY3JlYXRlZEF0');

@$core.Deprecated('Use searchAuditLogsRequestDescriptor instead')
const SearchAuditLogsRequest$json = {
  '1': 'SearchAuditLogsRequest',
  '2': [
    {
      '1': 'pagination',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.common.v1.Pagination',
      '10': 'pagination'
    },
    {'1': 'user_id', '3': 2, '4': 1, '5': 9, '10': 'userId'},
    {'1': 'event_type', '3': 3, '4': 1, '5': 9, '10': 'eventType'},
    {
      '1': 'from',
      '3': 4,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.common.v1.Timestamp',
      '10': 'from'
    },
    {
      '1': 'to',
      '3': 5,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.common.v1.Timestamp',
      '10': 'to'
    },
    {'1': 'result', '3': 6, '4': 1, '5': 9, '10': 'result'},
    {
      '1': 'event_type_enum',
      '3': 7,
      '4': 1,
      '5': 14,
      '6': '.k1s0.system.auth.v1.AuditEventType',
      '10': 'eventTypeEnum'
    },
    {
      '1': 'result_enum',
      '3': 8,
      '4': 1,
      '5': 14,
      '6': '.k1s0.system.auth.v1.AuditResult',
      '10': 'resultEnum'
    },
  ],
};

/// Descriptor for `SearchAuditLogsRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List searchAuditLogsRequestDescriptor = $convert.base64Decode(
    'ChZTZWFyY2hBdWRpdExvZ3NSZXF1ZXN0EkEKCnBhZ2luYXRpb24YASABKAsyIS5rMXMwLnN5c3'
    'RlbS5jb21tb24udjEuUGFnaW5hdGlvblIKcGFnaW5hdGlvbhIXCgd1c2VyX2lkGAIgASgJUgZ1'
    'c2VySWQSHQoKZXZlbnRfdHlwZRgDIAEoCVIJZXZlbnRUeXBlEjQKBGZyb20YBCABKAsyIC5rMX'
    'MwLnN5c3RlbS5jb21tb24udjEuVGltZXN0YW1wUgRmcm9tEjAKAnRvGAUgASgLMiAuazFzMC5z'
    'eXN0ZW0uY29tbW9uLnYxLlRpbWVzdGFtcFICdG8SFgoGcmVzdWx0GAYgASgJUgZyZXN1bHQSSw'
    'oPZXZlbnRfdHlwZV9lbnVtGAcgASgOMiMuazFzMC5zeXN0ZW0uYXV0aC52MS5BdWRpdEV2ZW50'
    'VHlwZVINZXZlbnRUeXBlRW51bRJBCgtyZXN1bHRfZW51bRgIIAEoDjIgLmsxczAuc3lzdGVtLm'
    'F1dGgudjEuQXVkaXRSZXN1bHRSCnJlc3VsdEVudW0=');

@$core.Deprecated('Use searchAuditLogsResponseDescriptor instead')
const SearchAuditLogsResponse$json = {
  '1': 'SearchAuditLogsResponse',
  '2': [
    {
      '1': 'logs',
      '3': 1,
      '4': 3,
      '5': 11,
      '6': '.k1s0.system.auth.v1.AuditLog',
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

/// Descriptor for `SearchAuditLogsResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List searchAuditLogsResponseDescriptor = $convert.base64Decode(
    'ChdTZWFyY2hBdWRpdExvZ3NSZXNwb25zZRIxCgRsb2dzGAEgAygLMh0uazFzMC5zeXN0ZW0uYX'
    'V0aC52MS5BdWRpdExvZ1IEbG9ncxJHCgpwYWdpbmF0aW9uGAIgASgLMicuazFzMC5zeXN0ZW0u'
    'Y29tbW9uLnYxLlBhZ2luYXRpb25SZXN1bHRSCnBhZ2luYXRpb24=');

@$core.Deprecated('Use auditLogDescriptor instead')
const AuditLog$json = {
  '1': 'AuditLog',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 9, '10': 'id'},
    {'1': 'event_type', '3': 2, '4': 1, '5': 9, '10': 'eventType'},
    {'1': 'user_id', '3': 3, '4': 1, '5': 9, '10': 'userId'},
    {'1': 'ip_address', '3': 4, '4': 1, '5': 9, '10': 'ipAddress'},
    {'1': 'user_agent', '3': 5, '4': 1, '5': 9, '10': 'userAgent'},
    {'1': 'resource', '3': 6, '4': 1, '5': 9, '10': 'resource'},
    {'1': 'action', '3': 7, '4': 1, '5': 9, '10': 'action'},
    {'1': 'result', '3': 8, '4': 1, '5': 9, '10': 'result'},
    {
      '1': 'detail',
      '3': 9,
      '4': 1,
      '5': 11,
      '6': '.google.protobuf.Struct',
      '10': 'detail'
    },
    {
      '1': 'created_at',
      '3': 10,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.common.v1.Timestamp',
      '10': 'createdAt'
    },
    {'1': 'resource_id', '3': 11, '4': 1, '5': 9, '10': 'resourceId'},
    {'1': 'trace_id', '3': 12, '4': 1, '5': 9, '10': 'traceId'},
    {
      '1': 'event_type_enum',
      '3': 13,
      '4': 1,
      '5': 14,
      '6': '.k1s0.system.auth.v1.AuditEventType',
      '10': 'eventTypeEnum'
    },
    {
      '1': 'result_enum',
      '3': 14,
      '4': 1,
      '5': 14,
      '6': '.k1s0.system.auth.v1.AuditResult',
      '10': 'resultEnum'
    },
  ],
};

/// Descriptor for `AuditLog`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List auditLogDescriptor = $convert.base64Decode(
    'CghBdWRpdExvZxIOCgJpZBgBIAEoCVICaWQSHQoKZXZlbnRfdHlwZRgCIAEoCVIJZXZlbnRUeX'
    'BlEhcKB3VzZXJfaWQYAyABKAlSBnVzZXJJZBIdCgppcF9hZGRyZXNzGAQgASgJUglpcEFkZHJl'
    'c3MSHQoKdXNlcl9hZ2VudBgFIAEoCVIJdXNlckFnZW50EhoKCHJlc291cmNlGAYgASgJUghyZX'
    'NvdXJjZRIWCgZhY3Rpb24YByABKAlSBmFjdGlvbhIWCgZyZXN1bHQYCCABKAlSBnJlc3VsdBIv'
    'CgZkZXRhaWwYCSABKAsyFy5nb29nbGUucHJvdG9idWYuU3RydWN0UgZkZXRhaWwSPwoKY3JlYX'
    'RlZF9hdBgKIAEoCzIgLmsxczAuc3lzdGVtLmNvbW1vbi52MS5UaW1lc3RhbXBSCWNyZWF0ZWRB'
    'dBIfCgtyZXNvdXJjZV9pZBgLIAEoCVIKcmVzb3VyY2VJZBIZCgh0cmFjZV9pZBgMIAEoCVIHdH'
    'JhY2VJZBJLCg9ldmVudF90eXBlX2VudW0YDSABKA4yIy5rMXMwLnN5c3RlbS5hdXRoLnYxLkF1'
    'ZGl0RXZlbnRUeXBlUg1ldmVudFR5cGVFbnVtEkEKC3Jlc3VsdF9lbnVtGA4gASgOMiAuazFzMC'
    '5zeXN0ZW0uYXV0aC52MS5BdWRpdFJlc3VsdFIKcmVzdWx0RW51bQ==');
