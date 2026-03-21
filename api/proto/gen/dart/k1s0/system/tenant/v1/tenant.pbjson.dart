// This is a generated file - do not edit.
//
// Generated from k1s0/system/tenant/v1/tenant.proto.

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

@$core.Deprecated('Use createTenantRequestDescriptor instead')
const CreateTenantRequest$json = {
  '1': 'CreateTenantRequest',
  '2': [
    {'1': 'name', '3': 1, '4': 1, '5': 9, '10': 'name'},
    {'1': 'display_name', '3': 2, '4': 1, '5': 9, '10': 'displayName'},
    {'1': 'owner_id', '3': 3, '4': 1, '5': 9, '10': 'ownerId'},
    {'1': 'plan', '3': 4, '4': 1, '5': 9, '10': 'plan'},
  ],
};

/// Descriptor for `CreateTenantRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List createTenantRequestDescriptor = $convert.base64Decode(
    'ChNDcmVhdGVUZW5hbnRSZXF1ZXN0EhIKBG5hbWUYASABKAlSBG5hbWUSIQoMZGlzcGxheV9uYW'
    '1lGAIgASgJUgtkaXNwbGF5TmFtZRIZCghvd25lcl9pZBgDIAEoCVIHb3duZXJJZBISCgRwbGFu'
    'GAQgASgJUgRwbGFu');

@$core.Deprecated('Use createTenantResponseDescriptor instead')
const CreateTenantResponse$json = {
  '1': 'CreateTenantResponse',
  '2': [
    {
      '1': 'tenant',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.tenant.v1.Tenant',
      '10': 'tenant'
    },
  ],
};

/// Descriptor for `CreateTenantResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List createTenantResponseDescriptor = $convert.base64Decode(
    'ChRDcmVhdGVUZW5hbnRSZXNwb25zZRI1CgZ0ZW5hbnQYASABKAsyHS5rMXMwLnN5c3RlbS50ZW'
    '5hbnQudjEuVGVuYW50UgZ0ZW5hbnQ=');

@$core.Deprecated('Use getTenantRequestDescriptor instead')
const GetTenantRequest$json = {
  '1': 'GetTenantRequest',
  '2': [
    {'1': 'tenant_id', '3': 1, '4': 1, '5': 9, '10': 'tenantId'},
  ],
};

/// Descriptor for `GetTenantRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List getTenantRequestDescriptor = $convert.base64Decode(
    'ChBHZXRUZW5hbnRSZXF1ZXN0EhsKCXRlbmFudF9pZBgBIAEoCVIIdGVuYW50SWQ=');

@$core.Deprecated('Use getTenantResponseDescriptor instead')
const GetTenantResponse$json = {
  '1': 'GetTenantResponse',
  '2': [
    {
      '1': 'tenant',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.tenant.v1.Tenant',
      '10': 'tenant'
    },
  ],
};

/// Descriptor for `GetTenantResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List getTenantResponseDescriptor = $convert.base64Decode(
    'ChFHZXRUZW5hbnRSZXNwb25zZRI1CgZ0ZW5hbnQYASABKAsyHS5rMXMwLnN5c3RlbS50ZW5hbn'
    'QudjEuVGVuYW50UgZ0ZW5hbnQ=');

@$core.Deprecated('Use listTenantsRequestDescriptor instead')
const ListTenantsRequest$json = {
  '1': 'ListTenantsRequest',
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

/// Descriptor for `ListTenantsRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List listTenantsRequestDescriptor = $convert.base64Decode(
    'ChJMaXN0VGVuYW50c1JlcXVlc3QSQQoKcGFnaW5hdGlvbhgBIAEoCzIhLmsxczAuc3lzdGVtLm'
    'NvbW1vbi52MS5QYWdpbmF0aW9uUgpwYWdpbmF0aW9u');

@$core.Deprecated('Use listTenantsResponseDescriptor instead')
const ListTenantsResponse$json = {
  '1': 'ListTenantsResponse',
  '2': [
    {
      '1': 'tenants',
      '3': 1,
      '4': 3,
      '5': 11,
      '6': '.k1s0.system.tenant.v1.Tenant',
      '10': 'tenants'
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

/// Descriptor for `ListTenantsResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List listTenantsResponseDescriptor = $convert.base64Decode(
    'ChNMaXN0VGVuYW50c1Jlc3BvbnNlEjcKB3RlbmFudHMYASADKAsyHS5rMXMwLnN5c3RlbS50ZW'
    '5hbnQudjEuVGVuYW50Ugd0ZW5hbnRzEkcKCnBhZ2luYXRpb24YAiABKAsyJy5rMXMwLnN5c3Rl'
    'bS5jb21tb24udjEuUGFnaW5hdGlvblJlc3VsdFIKcGFnaW5hdGlvbg==');

@$core.Deprecated('Use updateTenantRequestDescriptor instead')
const UpdateTenantRequest$json = {
  '1': 'UpdateTenantRequest',
  '2': [
    {'1': 'tenant_id', '3': 1, '4': 1, '5': 9, '10': 'tenantId'},
    {'1': 'display_name', '3': 2, '4': 1, '5': 9, '10': 'displayName'},
    {'1': 'plan', '3': 3, '4': 1, '5': 9, '10': 'plan'},
  ],
};

/// Descriptor for `UpdateTenantRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List updateTenantRequestDescriptor = $convert.base64Decode(
    'ChNVcGRhdGVUZW5hbnRSZXF1ZXN0EhsKCXRlbmFudF9pZBgBIAEoCVIIdGVuYW50SWQSIQoMZG'
    'lzcGxheV9uYW1lGAIgASgJUgtkaXNwbGF5TmFtZRISCgRwbGFuGAMgASgJUgRwbGFu');

@$core.Deprecated('Use updateTenantResponseDescriptor instead')
const UpdateTenantResponse$json = {
  '1': 'UpdateTenantResponse',
  '2': [
    {
      '1': 'tenant',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.tenant.v1.Tenant',
      '10': 'tenant'
    },
  ],
};

/// Descriptor for `UpdateTenantResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List updateTenantResponseDescriptor = $convert.base64Decode(
    'ChRVcGRhdGVUZW5hbnRSZXNwb25zZRI1CgZ0ZW5hbnQYASABKAsyHS5rMXMwLnN5c3RlbS50ZW'
    '5hbnQudjEuVGVuYW50UgZ0ZW5hbnQ=');

@$core.Deprecated('Use suspendTenantRequestDescriptor instead')
const SuspendTenantRequest$json = {
  '1': 'SuspendTenantRequest',
  '2': [
    {'1': 'tenant_id', '3': 1, '4': 1, '5': 9, '10': 'tenantId'},
  ],
};

/// Descriptor for `SuspendTenantRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List suspendTenantRequestDescriptor =
    $convert.base64Decode(
        'ChRTdXNwZW5kVGVuYW50UmVxdWVzdBIbCgl0ZW5hbnRfaWQYASABKAlSCHRlbmFudElk');

@$core.Deprecated('Use suspendTenantResponseDescriptor instead')
const SuspendTenantResponse$json = {
  '1': 'SuspendTenantResponse',
  '2': [
    {
      '1': 'tenant',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.tenant.v1.Tenant',
      '10': 'tenant'
    },
  ],
};

/// Descriptor for `SuspendTenantResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List suspendTenantResponseDescriptor = $convert.base64Decode(
    'ChVTdXNwZW5kVGVuYW50UmVzcG9uc2USNQoGdGVuYW50GAEgASgLMh0uazFzMC5zeXN0ZW0udG'
    'VuYW50LnYxLlRlbmFudFIGdGVuYW50');

@$core.Deprecated('Use activateTenantRequestDescriptor instead')
const ActivateTenantRequest$json = {
  '1': 'ActivateTenantRequest',
  '2': [
    {'1': 'tenant_id', '3': 1, '4': 1, '5': 9, '10': 'tenantId'},
  ],
};

/// Descriptor for `ActivateTenantRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List activateTenantRequestDescriptor = $convert.base64Decode(
    'ChVBY3RpdmF0ZVRlbmFudFJlcXVlc3QSGwoJdGVuYW50X2lkGAEgASgJUgh0ZW5hbnRJZA==');

@$core.Deprecated('Use activateTenantResponseDescriptor instead')
const ActivateTenantResponse$json = {
  '1': 'ActivateTenantResponse',
  '2': [
    {
      '1': 'tenant',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.tenant.v1.Tenant',
      '10': 'tenant'
    },
  ],
};

/// Descriptor for `ActivateTenantResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List activateTenantResponseDescriptor =
    $convert.base64Decode(
        'ChZBY3RpdmF0ZVRlbmFudFJlc3BvbnNlEjUKBnRlbmFudBgBIAEoCzIdLmsxczAuc3lzdGVtLn'
        'RlbmFudC52MS5UZW5hbnRSBnRlbmFudA==');

@$core.Deprecated('Use deleteTenantRequestDescriptor instead')
const DeleteTenantRequest$json = {
  '1': 'DeleteTenantRequest',
  '2': [
    {'1': 'tenant_id', '3': 1, '4': 1, '5': 9, '10': 'tenantId'},
  ],
};

/// Descriptor for `DeleteTenantRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List deleteTenantRequestDescriptor =
    $convert.base64Decode(
        'ChNEZWxldGVUZW5hbnRSZXF1ZXN0EhsKCXRlbmFudF9pZBgBIAEoCVIIdGVuYW50SWQ=');

@$core.Deprecated('Use deleteTenantResponseDescriptor instead')
const DeleteTenantResponse$json = {
  '1': 'DeleteTenantResponse',
  '2': [
    {
      '1': 'tenant',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.tenant.v1.Tenant',
      '10': 'tenant'
    },
  ],
};

/// Descriptor for `DeleteTenantResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List deleteTenantResponseDescriptor = $convert.base64Decode(
    'ChREZWxldGVUZW5hbnRSZXNwb25zZRI1CgZ0ZW5hbnQYASABKAsyHS5rMXMwLnN5c3RlbS50ZW'
    '5hbnQudjEuVGVuYW50UgZ0ZW5hbnQ=');

@$core.Deprecated('Use addMemberRequestDescriptor instead')
const AddMemberRequest$json = {
  '1': 'AddMemberRequest',
  '2': [
    {'1': 'tenant_id', '3': 1, '4': 1, '5': 9, '10': 'tenantId'},
    {'1': 'user_id', '3': 2, '4': 1, '5': 9, '10': 'userId'},
    {'1': 'role', '3': 3, '4': 1, '5': 9, '10': 'role'},
  ],
};

/// Descriptor for `AddMemberRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List addMemberRequestDescriptor = $convert.base64Decode(
    'ChBBZGRNZW1iZXJSZXF1ZXN0EhsKCXRlbmFudF9pZBgBIAEoCVIIdGVuYW50SWQSFwoHdXNlcl'
    '9pZBgCIAEoCVIGdXNlcklkEhIKBHJvbGUYAyABKAlSBHJvbGU=');

@$core.Deprecated('Use addMemberResponseDescriptor instead')
const AddMemberResponse$json = {
  '1': 'AddMemberResponse',
  '2': [
    {
      '1': 'member',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.tenant.v1.TenantMember',
      '10': 'member'
    },
  ],
};

/// Descriptor for `AddMemberResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List addMemberResponseDescriptor = $convert.base64Decode(
    'ChFBZGRNZW1iZXJSZXNwb25zZRI7CgZtZW1iZXIYASABKAsyIy5rMXMwLnN5c3RlbS50ZW5hbn'
    'QudjEuVGVuYW50TWVtYmVyUgZtZW1iZXI=');

@$core.Deprecated('Use listMembersRequestDescriptor instead')
const ListMembersRequest$json = {
  '1': 'ListMembersRequest',
  '2': [
    {'1': 'tenant_id', '3': 1, '4': 1, '5': 9, '10': 'tenantId'},
  ],
};

/// Descriptor for `ListMembersRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List listMembersRequestDescriptor =
    $convert.base64Decode(
        'ChJMaXN0TWVtYmVyc1JlcXVlc3QSGwoJdGVuYW50X2lkGAEgASgJUgh0ZW5hbnRJZA==');

@$core.Deprecated('Use listMembersResponseDescriptor instead')
const ListMembersResponse$json = {
  '1': 'ListMembersResponse',
  '2': [
    {
      '1': 'members',
      '3': 1,
      '4': 3,
      '5': 11,
      '6': '.k1s0.system.tenant.v1.TenantMember',
      '10': 'members'
    },
  ],
};

/// Descriptor for `ListMembersResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List listMembersResponseDescriptor = $convert.base64Decode(
    'ChNMaXN0TWVtYmVyc1Jlc3BvbnNlEj0KB21lbWJlcnMYASADKAsyIy5rMXMwLnN5c3RlbS50ZW'
    '5hbnQudjEuVGVuYW50TWVtYmVyUgdtZW1iZXJz');

@$core.Deprecated('Use removeMemberRequestDescriptor instead')
const RemoveMemberRequest$json = {
  '1': 'RemoveMemberRequest',
  '2': [
    {'1': 'tenant_id', '3': 1, '4': 1, '5': 9, '10': 'tenantId'},
    {'1': 'user_id', '3': 2, '4': 1, '5': 9, '10': 'userId'},
  ],
};

/// Descriptor for `RemoveMemberRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List removeMemberRequestDescriptor = $convert.base64Decode(
    'ChNSZW1vdmVNZW1iZXJSZXF1ZXN0EhsKCXRlbmFudF9pZBgBIAEoCVIIdGVuYW50SWQSFwoHdX'
    'Nlcl9pZBgCIAEoCVIGdXNlcklk');

@$core.Deprecated('Use removeMemberResponseDescriptor instead')
const RemoveMemberResponse$json = {
  '1': 'RemoveMemberResponse',
  '2': [
    {'1': 'success', '3': 1, '4': 1, '5': 8, '10': 'success'},
  ],
};

/// Descriptor for `RemoveMemberResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List removeMemberResponseDescriptor =
    $convert.base64Decode(
        'ChRSZW1vdmVNZW1iZXJSZXNwb25zZRIYCgdzdWNjZXNzGAEgASgIUgdzdWNjZXNz');

@$core.Deprecated('Use getProvisioningStatusRequestDescriptor instead')
const GetProvisioningStatusRequest$json = {
  '1': 'GetProvisioningStatusRequest',
  '2': [
    {'1': 'job_id', '3': 1, '4': 1, '5': 9, '10': 'jobId'},
  ],
};

/// Descriptor for `GetProvisioningStatusRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List getProvisioningStatusRequestDescriptor =
    $convert.base64Decode(
        'ChxHZXRQcm92aXNpb25pbmdTdGF0dXNSZXF1ZXN0EhUKBmpvYl9pZBgBIAEoCVIFam9iSWQ=');

@$core.Deprecated('Use getProvisioningStatusResponseDescriptor instead')
const GetProvisioningStatusResponse$json = {
  '1': 'GetProvisioningStatusResponse',
  '2': [
    {
      '1': 'job',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.tenant.v1.ProvisioningJob',
      '10': 'job'
    },
  ],
};

/// Descriptor for `GetProvisioningStatusResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List getProvisioningStatusResponseDescriptor =
    $convert.base64Decode(
        'Ch1HZXRQcm92aXNpb25pbmdTdGF0dXNSZXNwb25zZRI4CgNqb2IYASABKAsyJi5rMXMwLnN5c3'
        'RlbS50ZW5hbnQudjEuUHJvdmlzaW9uaW5nSm9iUgNqb2I=');

@$core.Deprecated('Use tenantDescriptor instead')
const Tenant$json = {
  '1': 'Tenant',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 9, '10': 'id'},
    {'1': 'name', '3': 2, '4': 1, '5': 9, '10': 'name'},
    {'1': 'display_name', '3': 3, '4': 1, '5': 9, '10': 'displayName'},
    {'1': 'status', '3': 4, '4': 1, '5': 9, '10': 'status'},
    {'1': 'plan', '3': 5, '4': 1, '5': 9, '10': 'plan'},
    {
      '1': 'created_at',
      '3': 6,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.common.v1.Timestamp',
      '10': 'createdAt'
    },
    {'1': 'owner_id', '3': 7, '4': 1, '5': 9, '10': 'ownerId'},
    {'1': 'settings', '3': 8, '4': 1, '5': 9, '10': 'settings'},
    {'1': 'db_schema', '3': 9, '4': 1, '5': 9, '10': 'dbSchema'},
    {
      '1': 'updated_at',
      '3': 10,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.common.v1.Timestamp',
      '10': 'updatedAt'
    },
    {'1': 'keycloak_realm', '3': 11, '4': 1, '5': 9, '10': 'keycloakRealm'},
  ],
};

/// Descriptor for `Tenant`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List tenantDescriptor = $convert.base64Decode(
    'CgZUZW5hbnQSDgoCaWQYASABKAlSAmlkEhIKBG5hbWUYAiABKAlSBG5hbWUSIQoMZGlzcGxheV'
    '9uYW1lGAMgASgJUgtkaXNwbGF5TmFtZRIWCgZzdGF0dXMYBCABKAlSBnN0YXR1cxISCgRwbGFu'
    'GAUgASgJUgRwbGFuEj8KCmNyZWF0ZWRfYXQYBiABKAsyIC5rMXMwLnN5c3RlbS5jb21tb24udj'
    'EuVGltZXN0YW1wUgljcmVhdGVkQXQSGQoIb3duZXJfaWQYByABKAlSB293bmVySWQSGgoIc2V0'
    'dGluZ3MYCCABKAlSCHNldHRpbmdzEhsKCWRiX3NjaGVtYRgJIAEoCVIIZGJTY2hlbWESPwoKdX'
    'BkYXRlZF9hdBgKIAEoCzIgLmsxczAuc3lzdGVtLmNvbW1vbi52MS5UaW1lc3RhbXBSCXVwZGF0'
    'ZWRBdBIlCg5rZXljbG9ha19yZWFsbRgLIAEoCVINa2V5Y2xvYWtSZWFsbQ==');

@$core.Deprecated('Use tenantMemberDescriptor instead')
const TenantMember$json = {
  '1': 'TenantMember',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 9, '10': 'id'},
    {'1': 'tenant_id', '3': 2, '4': 1, '5': 9, '10': 'tenantId'},
    {'1': 'user_id', '3': 3, '4': 1, '5': 9, '10': 'userId'},
    {'1': 'role', '3': 4, '4': 1, '5': 9, '10': 'role'},
    {
      '1': 'joined_at',
      '3': 5,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.common.v1.Timestamp',
      '10': 'joinedAt'
    },
  ],
};

/// Descriptor for `TenantMember`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List tenantMemberDescriptor = $convert.base64Decode(
    'CgxUZW5hbnRNZW1iZXISDgoCaWQYASABKAlSAmlkEhsKCXRlbmFudF9pZBgCIAEoCVIIdGVuYW'
    '50SWQSFwoHdXNlcl9pZBgDIAEoCVIGdXNlcklkEhIKBHJvbGUYBCABKAlSBHJvbGUSPQoJam9p'
    'bmVkX2F0GAUgASgLMiAuazFzMC5zeXN0ZW0uY29tbW9uLnYxLlRpbWVzdGFtcFIIam9pbmVkQX'
    'Q=');

@$core.Deprecated('Use provisioningJobDescriptor instead')
const ProvisioningJob$json = {
  '1': 'ProvisioningJob',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 9, '10': 'id'},
    {'1': 'tenant_id', '3': 2, '4': 1, '5': 9, '10': 'tenantId'},
    {'1': 'status', '3': 3, '4': 1, '5': 9, '10': 'status'},
    {'1': 'current_step', '3': 4, '4': 1, '5': 9, '10': 'currentStep'},
    {'1': 'error_message', '3': 5, '4': 1, '5': 9, '10': 'errorMessage'},
    {
      '1': 'created_at',
      '3': 6,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.common.v1.Timestamp',
      '10': 'createdAt'
    },
    {
      '1': 'updated_at',
      '3': 7,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.common.v1.Timestamp',
      '10': 'updatedAt'
    },
  ],
};

/// Descriptor for `ProvisioningJob`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List provisioningJobDescriptor = $convert.base64Decode(
    'Cg9Qcm92aXNpb25pbmdKb2ISDgoCaWQYASABKAlSAmlkEhsKCXRlbmFudF9pZBgCIAEoCVIIdG'
    'VuYW50SWQSFgoGc3RhdHVzGAMgASgJUgZzdGF0dXMSIQoMY3VycmVudF9zdGVwGAQgASgJUgtj'
    'dXJyZW50U3RlcBIjCg1lcnJvcl9tZXNzYWdlGAUgASgJUgxlcnJvck1lc3NhZ2USPwoKY3JlYX'
    'RlZF9hdBgGIAEoCzIgLmsxczAuc3lzdGVtLmNvbW1vbi52MS5UaW1lc3RhbXBSCWNyZWF0ZWRB'
    'dBI/Cgp1cGRhdGVkX2F0GAcgASgLMiAuazFzMC5zeXN0ZW0uY29tbW9uLnYxLlRpbWVzdGFtcF'
    'IJdXBkYXRlZEF0');

@$core.Deprecated('Use watchTenantRequestDescriptor instead')
const WatchTenantRequest$json = {
  '1': 'WatchTenantRequest',
  '2': [
    {'1': 'tenant_id', '3': 1, '4': 1, '5': 9, '10': 'tenantId'},
  ],
};

/// Descriptor for `WatchTenantRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List watchTenantRequestDescriptor =
    $convert.base64Decode(
        'ChJXYXRjaFRlbmFudFJlcXVlc3QSGwoJdGVuYW50X2lkGAEgASgJUgh0ZW5hbnRJZA==');

@$core.Deprecated('Use watchTenantResponseDescriptor instead')
const WatchTenantResponse$json = {
  '1': 'WatchTenantResponse',
  '2': [
    {'1': 'tenant_id', '3': 1, '4': 1, '5': 9, '10': 'tenantId'},
    {'1': 'change_type', '3': 2, '4': 1, '5': 9, '10': 'changeType'},
    {
      '1': 'tenant',
      '3': 3,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.tenant.v1.Tenant',
      '10': 'tenant'
    },
    {
      '1': 'changed_at',
      '3': 4,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.common.v1.Timestamp',
      '10': 'changedAt'
    },
  ],
};

/// Descriptor for `WatchTenantResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List watchTenantResponseDescriptor = $convert.base64Decode(
    'ChNXYXRjaFRlbmFudFJlc3BvbnNlEhsKCXRlbmFudF9pZBgBIAEoCVIIdGVuYW50SWQSHwoLY2'
    'hhbmdlX3R5cGUYAiABKAlSCmNoYW5nZVR5cGUSNQoGdGVuYW50GAMgASgLMh0uazFzMC5zeXN0'
    'ZW0udGVuYW50LnYxLlRlbmFudFIGdGVuYW50Ej8KCmNoYW5nZWRfYXQYBCABKAsyIC5rMXMwLn'
    'N5c3RlbS5jb21tb24udjEuVGltZXN0YW1wUgljaGFuZ2VkQXQ=');
