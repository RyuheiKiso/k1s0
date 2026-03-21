// This is a generated file - do not edit.
//
// Generated from k1s0/business/accounting/domainmaster/v1/domain_master.proto.

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

@$core.Deprecated('Use masterCategoryDescriptor instead')
const MasterCategory$json = {
  '1': 'MasterCategory',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 9, '10': 'id'},
    {'1': 'code', '3': 2, '4': 1, '5': 9, '10': 'code'},
    {'1': 'display_name', '3': 3, '4': 1, '5': 9, '10': 'displayName'},
    {'1': 'description', '3': 4, '4': 1, '5': 9, '10': 'description'},
    {
      '1': 'validation_schema',
      '3': 5,
      '4': 1,
      '5': 11,
      '6': '.google.protobuf.Struct',
      '10': 'validationSchema'
    },
    {'1': 'is_active', '3': 6, '4': 1, '5': 8, '10': 'isActive'},
    {'1': 'sort_order', '3': 7, '4': 1, '5': 5, '10': 'sortOrder'},
    {'1': 'created_by', '3': 8, '4': 1, '5': 9, '10': 'createdBy'},
    {
      '1': 'created_at',
      '3': 9,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.common.v1.Timestamp',
      '10': 'createdAt'
    },
    {
      '1': 'updated_at',
      '3': 10,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.common.v1.Timestamp',
      '10': 'updatedAt'
    },
  ],
};

/// Descriptor for `MasterCategory`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List masterCategoryDescriptor = $convert.base64Decode(
    'Cg5NYXN0ZXJDYXRlZ29yeRIOCgJpZBgBIAEoCVICaWQSEgoEY29kZRgCIAEoCVIEY29kZRIhCg'
    'xkaXNwbGF5X25hbWUYAyABKAlSC2Rpc3BsYXlOYW1lEiAKC2Rlc2NyaXB0aW9uGAQgASgJUgtk'
    'ZXNjcmlwdGlvbhJEChF2YWxpZGF0aW9uX3NjaGVtYRgFIAEoCzIXLmdvb2dsZS5wcm90b2J1Zi'
    '5TdHJ1Y3RSEHZhbGlkYXRpb25TY2hlbWESGwoJaXNfYWN0aXZlGAYgASgIUghpc0FjdGl2ZRId'
    'Cgpzb3J0X29yZGVyGAcgASgFUglzb3J0T3JkZXISHQoKY3JlYXRlZF9ieRgIIAEoCVIJY3JlYX'
    'RlZEJ5Ej8KCmNyZWF0ZWRfYXQYCSABKAsyIC5rMXMwLnN5c3RlbS5jb21tb24udjEuVGltZXN0'
    'YW1wUgljcmVhdGVkQXQSPwoKdXBkYXRlZF9hdBgKIAEoCzIgLmsxczAuc3lzdGVtLmNvbW1vbi'
    '52MS5UaW1lc3RhbXBSCXVwZGF0ZWRBdA==');

@$core.Deprecated('Use masterItemDescriptor instead')
const MasterItem$json = {
  '1': 'MasterItem',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 9, '10': 'id'},
    {'1': 'category_id', '3': 2, '4': 1, '5': 9, '10': 'categoryId'},
    {'1': 'code', '3': 3, '4': 1, '5': 9, '10': 'code'},
    {'1': 'display_name', '3': 4, '4': 1, '5': 9, '10': 'displayName'},
    {'1': 'description', '3': 5, '4': 1, '5': 9, '10': 'description'},
    {
      '1': 'attributes',
      '3': 6,
      '4': 1,
      '5': 11,
      '6': '.google.protobuf.Struct',
      '10': 'attributes'
    },
    {
      '1': 'parent_item_id',
      '3': 7,
      '4': 1,
      '5': 9,
      '9': 0,
      '10': 'parentItemId',
      '17': true
    },
    {
      '1': 'effective_from',
      '3': 8,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.common.v1.Timestamp',
      '10': 'effectiveFrom'
    },
    {
      '1': 'effective_until',
      '3': 9,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.common.v1.Timestamp',
      '9': 1,
      '10': 'effectiveUntil',
      '17': true
    },
    {'1': 'is_active', '3': 10, '4': 1, '5': 8, '10': 'isActive'},
    {'1': 'sort_order', '3': 11, '4': 1, '5': 5, '10': 'sortOrder'},
    {'1': 'created_by', '3': 12, '4': 1, '5': 9, '10': 'createdBy'},
    {
      '1': 'created_at',
      '3': 13,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.common.v1.Timestamp',
      '10': 'createdAt'
    },
    {
      '1': 'updated_at',
      '3': 14,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.common.v1.Timestamp',
      '10': 'updatedAt'
    },
  ],
  '8': [
    {'1': '_parent_item_id'},
    {'1': '_effective_until'},
  ],
};

/// Descriptor for `MasterItem`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List masterItemDescriptor = $convert.base64Decode(
    'CgpNYXN0ZXJJdGVtEg4KAmlkGAEgASgJUgJpZBIfCgtjYXRlZ29yeV9pZBgCIAEoCVIKY2F0ZW'
    'dvcnlJZBISCgRjb2RlGAMgASgJUgRjb2RlEiEKDGRpc3BsYXlfbmFtZRgEIAEoCVILZGlzcGxh'
    'eU5hbWUSIAoLZGVzY3JpcHRpb24YBSABKAlSC2Rlc2NyaXB0aW9uEjcKCmF0dHJpYnV0ZXMYBi'
    'ABKAsyFy5nb29nbGUucHJvdG9idWYuU3RydWN0UgphdHRyaWJ1dGVzEikKDnBhcmVudF9pdGVt'
    'X2lkGAcgASgJSABSDHBhcmVudEl0ZW1JZIgBARJHCg5lZmZlY3RpdmVfZnJvbRgIIAEoCzIgLm'
    'sxczAuc3lzdGVtLmNvbW1vbi52MS5UaW1lc3RhbXBSDWVmZmVjdGl2ZUZyb20STgoPZWZmZWN0'
    'aXZlX3VudGlsGAkgASgLMiAuazFzMC5zeXN0ZW0uY29tbW9uLnYxLlRpbWVzdGFtcEgBUg5lZm'
    'ZlY3RpdmVVbnRpbIgBARIbCglpc19hY3RpdmUYCiABKAhSCGlzQWN0aXZlEh0KCnNvcnRfb3Jk'
    'ZXIYCyABKAVSCXNvcnRPcmRlchIdCgpjcmVhdGVkX2J5GAwgASgJUgljcmVhdGVkQnkSPwoKY3'
    'JlYXRlZF9hdBgNIAEoCzIgLmsxczAuc3lzdGVtLmNvbW1vbi52MS5UaW1lc3RhbXBSCWNyZWF0'
    'ZWRBdBI/Cgp1cGRhdGVkX2F0GA4gASgLMiAuazFzMC5zeXN0ZW0uY29tbW9uLnYxLlRpbWVzdG'
    'FtcFIJdXBkYXRlZEF0QhEKD19wYXJlbnRfaXRlbV9pZEISChBfZWZmZWN0aXZlX3VudGls');

@$core.Deprecated('Use masterItemVersionDescriptor instead')
const MasterItemVersion$json = {
  '1': 'MasterItemVersion',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 9, '10': 'id'},
    {'1': 'item_id', '3': 2, '4': 1, '5': 9, '10': 'itemId'},
    {'1': 'version_number', '3': 3, '4': 1, '5': 5, '10': 'versionNumber'},
    {
      '1': 'before_data',
      '3': 4,
      '4': 1,
      '5': 11,
      '6': '.google.protobuf.Struct',
      '10': 'beforeData'
    },
    {
      '1': 'after_data',
      '3': 5,
      '4': 1,
      '5': 11,
      '6': '.google.protobuf.Struct',
      '10': 'afterData'
    },
    {'1': 'changed_by', '3': 6, '4': 1, '5': 9, '10': 'changedBy'},
    {'1': 'change_reason', '3': 7, '4': 1, '5': 9, '10': 'changeReason'},
    {
      '1': 'created_at',
      '3': 8,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.common.v1.Timestamp',
      '10': 'createdAt'
    },
  ],
};

/// Descriptor for `MasterItemVersion`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List masterItemVersionDescriptor = $convert.base64Decode(
    'ChFNYXN0ZXJJdGVtVmVyc2lvbhIOCgJpZBgBIAEoCVICaWQSFwoHaXRlbV9pZBgCIAEoCVIGaX'
    'RlbUlkEiUKDnZlcnNpb25fbnVtYmVyGAMgASgFUg12ZXJzaW9uTnVtYmVyEjgKC2JlZm9yZV9k'
    'YXRhGAQgASgLMhcuZ29vZ2xlLnByb3RvYnVmLlN0cnVjdFIKYmVmb3JlRGF0YRI2CgphZnRlcl'
    '9kYXRhGAUgASgLMhcuZ29vZ2xlLnByb3RvYnVmLlN0cnVjdFIJYWZ0ZXJEYXRhEh0KCmNoYW5n'
    'ZWRfYnkYBiABKAlSCWNoYW5nZWRCeRIjCg1jaGFuZ2VfcmVhc29uGAcgASgJUgxjaGFuZ2VSZW'
    'Fzb24SPwoKY3JlYXRlZF9hdBgIIAEoCzIgLmsxczAuc3lzdGVtLmNvbW1vbi52MS5UaW1lc3Rh'
    'bXBSCWNyZWF0ZWRBdA==');

@$core.Deprecated('Use tenantMasterExtensionDescriptor instead')
const TenantMasterExtension$json = {
  '1': 'TenantMasterExtension',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 9, '10': 'id'},
    {'1': 'tenant_id', '3': 2, '4': 1, '5': 9, '10': 'tenantId'},
    {'1': 'item_id', '3': 3, '4': 1, '5': 9, '10': 'itemId'},
    {
      '1': 'display_name_override',
      '3': 4,
      '4': 1,
      '5': 9,
      '9': 0,
      '10': 'displayNameOverride',
      '17': true
    },
    {
      '1': 'attributes_override',
      '3': 5,
      '4': 1,
      '5': 11,
      '6': '.google.protobuf.Struct',
      '10': 'attributesOverride'
    },
    {'1': 'is_enabled', '3': 6, '4': 1, '5': 8, '10': 'isEnabled'},
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
  '8': [
    {'1': '_display_name_override'},
  ],
};

/// Descriptor for `TenantMasterExtension`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List tenantMasterExtensionDescriptor = $convert.base64Decode(
    'ChVUZW5hbnRNYXN0ZXJFeHRlbnNpb24SDgoCaWQYASABKAlSAmlkEhsKCXRlbmFudF9pZBgCIA'
    'EoCVIIdGVuYW50SWQSFwoHaXRlbV9pZBgDIAEoCVIGaXRlbUlkEjcKFWRpc3BsYXlfbmFtZV9v'
    'dmVycmlkZRgEIAEoCUgAUhNkaXNwbGF5TmFtZU92ZXJyaWRliAEBEkgKE2F0dHJpYnV0ZXNfb3'
    'ZlcnJpZGUYBSABKAsyFy5nb29nbGUucHJvdG9idWYuU3RydWN0UhJhdHRyaWJ1dGVzT3ZlcnJp'
    'ZGUSHQoKaXNfZW5hYmxlZBgGIAEoCFIJaXNFbmFibGVkEj8KCmNyZWF0ZWRfYXQYByABKAsyIC'
    '5rMXMwLnN5c3RlbS5jb21tb24udjEuVGltZXN0YW1wUgljcmVhdGVkQXQSPwoKdXBkYXRlZF9h'
    'dBgIIAEoCzIgLmsxczAuc3lzdGVtLmNvbW1vbi52MS5UaW1lc3RhbXBSCXVwZGF0ZWRBdEIYCh'
    'ZfZGlzcGxheV9uYW1lX292ZXJyaWRl');

@$core.Deprecated('Use tenantMergedItemDescriptor instead')
const TenantMergedItem$json = {
  '1': 'TenantMergedItem',
  '2': [
    {
      '1': 'base_item',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.k1s0.business.accounting.domainmaster.v1.MasterItem',
      '10': 'baseItem'
    },
    {
      '1': 'extension',
      '3': 2,
      '4': 1,
      '5': 11,
      '6': '.k1s0.business.accounting.domainmaster.v1.TenantMasterExtension',
      '9': 0,
      '10': 'extension',
      '17': true
    },
    {
      '1': 'effective_display_name',
      '3': 3,
      '4': 1,
      '5': 9,
      '10': 'effectiveDisplayName'
    },
    {
      '1': 'effective_attributes',
      '3': 4,
      '4': 1,
      '5': 11,
      '6': '.google.protobuf.Struct',
      '10': 'effectiveAttributes'
    },
  ],
  '8': [
    {'1': '_extension'},
  ],
};

/// Descriptor for `TenantMergedItem`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List tenantMergedItemDescriptor = $convert.base64Decode(
    'ChBUZW5hbnRNZXJnZWRJdGVtElEKCWJhc2VfaXRlbRgBIAEoCzI0LmsxczAuYnVzaW5lc3MuYW'
    'Njb3VudGluZy5kb21haW5tYXN0ZXIudjEuTWFzdGVySXRlbVIIYmFzZUl0ZW0SYgoJZXh0ZW5z'
    'aW9uGAIgASgLMj8uazFzMC5idXNpbmVzcy5hY2NvdW50aW5nLmRvbWFpbm1hc3Rlci52MS5UZW'
    '5hbnRNYXN0ZXJFeHRlbnNpb25IAFIJZXh0ZW5zaW9uiAEBEjQKFmVmZmVjdGl2ZV9kaXNwbGF5'
    'X25hbWUYAyABKAlSFGVmZmVjdGl2ZURpc3BsYXlOYW1lEkoKFGVmZmVjdGl2ZV9hdHRyaWJ1dG'
    'VzGAQgASgLMhcuZ29vZ2xlLnByb3RvYnVmLlN0cnVjdFITZWZmZWN0aXZlQXR0cmlidXRlc0IM'
    'CgpfZXh0ZW5zaW9u');

@$core.Deprecated('Use listCategoriesRequestDescriptor instead')
const ListCategoriesRequest$json = {
  '1': 'ListCategoriesRequest',
  '2': [
    {'1': 'active_only', '3': 1, '4': 1, '5': 8, '10': 'activeOnly'},
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

/// Descriptor for `ListCategoriesRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List listCategoriesRequestDescriptor = $convert.base64Decode(
    'ChVMaXN0Q2F0ZWdvcmllc1JlcXVlc3QSHwoLYWN0aXZlX29ubHkYASABKAhSCmFjdGl2ZU9ubH'
    'kSQQoKcGFnaW5hdGlvbhgCIAEoCzIhLmsxczAuc3lzdGVtLmNvbW1vbi52MS5QYWdpbmF0aW9u'
    'UgpwYWdpbmF0aW9u');

@$core.Deprecated('Use listCategoriesResponseDescriptor instead')
const ListCategoriesResponse$json = {
  '1': 'ListCategoriesResponse',
  '2': [
    {
      '1': 'categories',
      '3': 1,
      '4': 3,
      '5': 11,
      '6': '.k1s0.business.accounting.domainmaster.v1.MasterCategory',
      '10': 'categories'
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

/// Descriptor for `ListCategoriesResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List listCategoriesResponseDescriptor = $convert.base64Decode(
    'ChZMaXN0Q2F0ZWdvcmllc1Jlc3BvbnNlElgKCmNhdGVnb3JpZXMYASADKAsyOC5rMXMwLmJ1c2'
    'luZXNzLmFjY291bnRpbmcuZG9tYWlubWFzdGVyLnYxLk1hc3RlckNhdGVnb3J5UgpjYXRlZ29y'
    'aWVzEkcKCnBhZ2luYXRpb24YAiABKAsyJy5rMXMwLnN5c3RlbS5jb21tb24udjEuUGFnaW5hdG'
    'lvblJlc3VsdFIKcGFnaW5hdGlvbg==');

@$core.Deprecated('Use getCategoryRequestDescriptor instead')
const GetCategoryRequest$json = {
  '1': 'GetCategoryRequest',
  '2': [
    {'1': 'category_id', '3': 1, '4': 1, '5': 9, '10': 'categoryId'},
  ],
};

/// Descriptor for `GetCategoryRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List getCategoryRequestDescriptor = $convert.base64Decode(
    'ChJHZXRDYXRlZ29yeVJlcXVlc3QSHwoLY2F0ZWdvcnlfaWQYASABKAlSCmNhdGVnb3J5SWQ=');

@$core.Deprecated('Use getCategoryResponseDescriptor instead')
const GetCategoryResponse$json = {
  '1': 'GetCategoryResponse',
  '2': [
    {
      '1': 'category',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.k1s0.business.accounting.domainmaster.v1.MasterCategory',
      '10': 'category'
    },
  ],
};

/// Descriptor for `GetCategoryResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List getCategoryResponseDescriptor = $convert.base64Decode(
    'ChNHZXRDYXRlZ29yeVJlc3BvbnNlElQKCGNhdGVnb3J5GAEgASgLMjguazFzMC5idXNpbmVzcy'
    '5hY2NvdW50aW5nLmRvbWFpbm1hc3Rlci52MS5NYXN0ZXJDYXRlZ29yeVIIY2F0ZWdvcnk=');

@$core.Deprecated('Use createCategoryRequestDescriptor instead')
const CreateCategoryRequest$json = {
  '1': 'CreateCategoryRequest',
  '2': [
    {'1': 'code', '3': 1, '4': 1, '5': 9, '10': 'code'},
    {'1': 'display_name', '3': 2, '4': 1, '5': 9, '10': 'displayName'},
    {
      '1': 'description',
      '3': 3,
      '4': 1,
      '5': 9,
      '9': 0,
      '10': 'description',
      '17': true
    },
    {
      '1': 'validation_schema',
      '3': 4,
      '4': 1,
      '5': 11,
      '6': '.google.protobuf.Struct',
      '10': 'validationSchema'
    },
    {
      '1': 'is_active',
      '3': 5,
      '4': 1,
      '5': 8,
      '9': 1,
      '10': 'isActive',
      '17': true
    },
    {
      '1': 'sort_order',
      '3': 6,
      '4': 1,
      '5': 5,
      '9': 2,
      '10': 'sortOrder',
      '17': true
    },
  ],
  '8': [
    {'1': '_description'},
    {'1': '_is_active'},
    {'1': '_sort_order'},
  ],
};

/// Descriptor for `CreateCategoryRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List createCategoryRequestDescriptor = $convert.base64Decode(
    'ChVDcmVhdGVDYXRlZ29yeVJlcXVlc3QSEgoEY29kZRgBIAEoCVIEY29kZRIhCgxkaXNwbGF5X2'
    '5hbWUYAiABKAlSC2Rpc3BsYXlOYW1lEiUKC2Rlc2NyaXB0aW9uGAMgASgJSABSC2Rlc2NyaXB0'
    'aW9uiAEBEkQKEXZhbGlkYXRpb25fc2NoZW1hGAQgASgLMhcuZ29vZ2xlLnByb3RvYnVmLlN0cn'
    'VjdFIQdmFsaWRhdGlvblNjaGVtYRIgCglpc19hY3RpdmUYBSABKAhIAVIIaXNBY3RpdmWIAQES'
    'IgoKc29ydF9vcmRlchgGIAEoBUgCUglzb3J0T3JkZXKIAQFCDgoMX2Rlc2NyaXB0aW9uQgwKCl'
    '9pc19hY3RpdmVCDQoLX3NvcnRfb3JkZXI=');

@$core.Deprecated('Use createCategoryResponseDescriptor instead')
const CreateCategoryResponse$json = {
  '1': 'CreateCategoryResponse',
  '2': [
    {
      '1': 'category',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.k1s0.business.accounting.domainmaster.v1.MasterCategory',
      '10': 'category'
    },
  ],
};

/// Descriptor for `CreateCategoryResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List createCategoryResponseDescriptor = $convert.base64Decode(
    'ChZDcmVhdGVDYXRlZ29yeVJlc3BvbnNlElQKCGNhdGVnb3J5GAEgASgLMjguazFzMC5idXNpbm'
    'Vzcy5hY2NvdW50aW5nLmRvbWFpbm1hc3Rlci52MS5NYXN0ZXJDYXRlZ29yeVIIY2F0ZWdvcnk=');

@$core.Deprecated('Use updateCategoryRequestDescriptor instead')
const UpdateCategoryRequest$json = {
  '1': 'UpdateCategoryRequest',
  '2': [
    {'1': 'category_id', '3': 1, '4': 1, '5': 9, '10': 'categoryId'},
    {
      '1': 'display_name',
      '3': 2,
      '4': 1,
      '5': 9,
      '9': 0,
      '10': 'displayName',
      '17': true
    },
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
      '1': 'validation_schema',
      '3': 4,
      '4': 1,
      '5': 11,
      '6': '.google.protobuf.Struct',
      '10': 'validationSchema'
    },
    {
      '1': 'is_active',
      '3': 5,
      '4': 1,
      '5': 8,
      '9': 2,
      '10': 'isActive',
      '17': true
    },
    {
      '1': 'sort_order',
      '3': 6,
      '4': 1,
      '5': 5,
      '9': 3,
      '10': 'sortOrder',
      '17': true
    },
  ],
  '8': [
    {'1': '_display_name'},
    {'1': '_description'},
    {'1': '_is_active'},
    {'1': '_sort_order'},
  ],
};

/// Descriptor for `UpdateCategoryRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List updateCategoryRequestDescriptor = $convert.base64Decode(
    'ChVVcGRhdGVDYXRlZ29yeVJlcXVlc3QSHwoLY2F0ZWdvcnlfaWQYASABKAlSCmNhdGVnb3J5SW'
    'QSJgoMZGlzcGxheV9uYW1lGAIgASgJSABSC2Rpc3BsYXlOYW1liAEBEiUKC2Rlc2NyaXB0aW9u'
    'GAMgASgJSAFSC2Rlc2NyaXB0aW9uiAEBEkQKEXZhbGlkYXRpb25fc2NoZW1hGAQgASgLMhcuZ2'
    '9vZ2xlLnByb3RvYnVmLlN0cnVjdFIQdmFsaWRhdGlvblNjaGVtYRIgCglpc19hY3RpdmUYBSAB'
    'KAhIAlIIaXNBY3RpdmWIAQESIgoKc29ydF9vcmRlchgGIAEoBUgDUglzb3J0T3JkZXKIAQFCDw'
    'oNX2Rpc3BsYXlfbmFtZUIOCgxfZGVzY3JpcHRpb25CDAoKX2lzX2FjdGl2ZUINCgtfc29ydF9v'
    'cmRlcg==');

@$core.Deprecated('Use updateCategoryResponseDescriptor instead')
const UpdateCategoryResponse$json = {
  '1': 'UpdateCategoryResponse',
  '2': [
    {
      '1': 'category',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.k1s0.business.accounting.domainmaster.v1.MasterCategory',
      '10': 'category'
    },
  ],
};

/// Descriptor for `UpdateCategoryResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List updateCategoryResponseDescriptor = $convert.base64Decode(
    'ChZVcGRhdGVDYXRlZ29yeVJlc3BvbnNlElQKCGNhdGVnb3J5GAEgASgLMjguazFzMC5idXNpbm'
    'Vzcy5hY2NvdW50aW5nLmRvbWFpbm1hc3Rlci52MS5NYXN0ZXJDYXRlZ29yeVIIY2F0ZWdvcnk=');

@$core.Deprecated('Use deleteCategoryRequestDescriptor instead')
const DeleteCategoryRequest$json = {
  '1': 'DeleteCategoryRequest',
  '2': [
    {'1': 'category_id', '3': 1, '4': 1, '5': 9, '10': 'categoryId'},
  ],
};

/// Descriptor for `DeleteCategoryRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List deleteCategoryRequestDescriptor = $convert.base64Decode(
    'ChVEZWxldGVDYXRlZ29yeVJlcXVlc3QSHwoLY2F0ZWdvcnlfaWQYASABKAlSCmNhdGVnb3J5SW'
    'Q=');

@$core.Deprecated('Use deleteCategoryResponseDescriptor instead')
const DeleteCategoryResponse$json = {
  '1': 'DeleteCategoryResponse',
  '2': [
    {'1': 'success', '3': 1, '4': 1, '5': 8, '10': 'success'},
  ],
};

/// Descriptor for `DeleteCategoryResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List deleteCategoryResponseDescriptor =
    $convert.base64Decode(
        'ChZEZWxldGVDYXRlZ29yeVJlc3BvbnNlEhgKB3N1Y2Nlc3MYASABKAhSB3N1Y2Nlc3M=');

@$core.Deprecated('Use listItemsRequestDescriptor instead')
const ListItemsRequest$json = {
  '1': 'ListItemsRequest',
  '2': [
    {'1': 'category_id', '3': 1, '4': 1, '5': 9, '10': 'categoryId'},
    {'1': 'active_only', '3': 2, '4': 1, '5': 8, '10': 'activeOnly'},
    {
      '1': 'pagination',
      '3': 3,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.common.v1.Pagination',
      '10': 'pagination'
    },
  ],
};

/// Descriptor for `ListItemsRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List listItemsRequestDescriptor = $convert.base64Decode(
    'ChBMaXN0SXRlbXNSZXF1ZXN0Eh8KC2NhdGVnb3J5X2lkGAEgASgJUgpjYXRlZ29yeUlkEh8KC2'
    'FjdGl2ZV9vbmx5GAIgASgIUgphY3RpdmVPbmx5EkEKCnBhZ2luYXRpb24YAyABKAsyIS5rMXMw'
    'LnN5c3RlbS5jb21tb24udjEuUGFnaW5hdGlvblIKcGFnaW5hdGlvbg==');

@$core.Deprecated('Use listItemsResponseDescriptor instead')
const ListItemsResponse$json = {
  '1': 'ListItemsResponse',
  '2': [
    {
      '1': 'items',
      '3': 1,
      '4': 3,
      '5': 11,
      '6': '.k1s0.business.accounting.domainmaster.v1.MasterItem',
      '10': 'items'
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

/// Descriptor for `ListItemsResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List listItemsResponseDescriptor = $convert.base64Decode(
    'ChFMaXN0SXRlbXNSZXNwb25zZRJKCgVpdGVtcxgBIAMoCzI0LmsxczAuYnVzaW5lc3MuYWNjb3'
    'VudGluZy5kb21haW5tYXN0ZXIudjEuTWFzdGVySXRlbVIFaXRlbXMSRwoKcGFnaW5hdGlvbhgC'
    'IAEoCzInLmsxczAuc3lzdGVtLmNvbW1vbi52MS5QYWdpbmF0aW9uUmVzdWx0UgpwYWdpbmF0aW'
    '9u');

@$core.Deprecated('Use getItemRequestDescriptor instead')
const GetItemRequest$json = {
  '1': 'GetItemRequest',
  '2': [
    {'1': 'item_id', '3': 1, '4': 1, '5': 9, '10': 'itemId'},
  ],
};

/// Descriptor for `GetItemRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List getItemRequestDescriptor = $convert
    .base64Decode('Cg5HZXRJdGVtUmVxdWVzdBIXCgdpdGVtX2lkGAEgASgJUgZpdGVtSWQ=');

@$core.Deprecated('Use getItemResponseDescriptor instead')
const GetItemResponse$json = {
  '1': 'GetItemResponse',
  '2': [
    {
      '1': 'item',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.k1s0.business.accounting.domainmaster.v1.MasterItem',
      '10': 'item'
    },
  ],
};

/// Descriptor for `GetItemResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List getItemResponseDescriptor = $convert.base64Decode(
    'Cg9HZXRJdGVtUmVzcG9uc2USSAoEaXRlbRgBIAEoCzI0LmsxczAuYnVzaW5lc3MuYWNjb3VudG'
    'luZy5kb21haW5tYXN0ZXIudjEuTWFzdGVySXRlbVIEaXRlbQ==');

@$core.Deprecated('Use createItemRequestDescriptor instead')
const CreateItemRequest$json = {
  '1': 'CreateItemRequest',
  '2': [
    {'1': 'category_id', '3': 1, '4': 1, '5': 9, '10': 'categoryId'},
    {'1': 'code', '3': 2, '4': 1, '5': 9, '10': 'code'},
    {'1': 'display_name', '3': 3, '4': 1, '5': 9, '10': 'displayName'},
    {
      '1': 'description',
      '3': 4,
      '4': 1,
      '5': 9,
      '9': 0,
      '10': 'description',
      '17': true
    },
    {
      '1': 'attributes',
      '3': 5,
      '4': 1,
      '5': 11,
      '6': '.google.protobuf.Struct',
      '10': 'attributes'
    },
    {
      '1': 'parent_item_id',
      '3': 6,
      '4': 1,
      '5': 9,
      '9': 1,
      '10': 'parentItemId',
      '17': true
    },
    {
      '1': 'effective_from',
      '3': 7,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.common.v1.Timestamp',
      '9': 2,
      '10': 'effectiveFrom',
      '17': true
    },
    {
      '1': 'effective_until',
      '3': 8,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.common.v1.Timestamp',
      '9': 3,
      '10': 'effectiveUntil',
      '17': true
    },
    {
      '1': 'is_active',
      '3': 9,
      '4': 1,
      '5': 8,
      '9': 4,
      '10': 'isActive',
      '17': true
    },
    {
      '1': 'sort_order',
      '3': 10,
      '4': 1,
      '5': 5,
      '9': 5,
      '10': 'sortOrder',
      '17': true
    },
  ],
  '8': [
    {'1': '_description'},
    {'1': '_parent_item_id'},
    {'1': '_effective_from'},
    {'1': '_effective_until'},
    {'1': '_is_active'},
    {'1': '_sort_order'},
  ],
};

/// Descriptor for `CreateItemRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List createItemRequestDescriptor = $convert.base64Decode(
    'ChFDcmVhdGVJdGVtUmVxdWVzdBIfCgtjYXRlZ29yeV9pZBgBIAEoCVIKY2F0ZWdvcnlJZBISCg'
    'Rjb2RlGAIgASgJUgRjb2RlEiEKDGRpc3BsYXlfbmFtZRgDIAEoCVILZGlzcGxheU5hbWUSJQoL'
    'ZGVzY3JpcHRpb24YBCABKAlIAFILZGVzY3JpcHRpb26IAQESNwoKYXR0cmlidXRlcxgFIAEoCz'
    'IXLmdvb2dsZS5wcm90b2J1Zi5TdHJ1Y3RSCmF0dHJpYnV0ZXMSKQoOcGFyZW50X2l0ZW1faWQY'
    'BiABKAlIAVIMcGFyZW50SXRlbUlkiAEBEkwKDmVmZmVjdGl2ZV9mcm9tGAcgASgLMiAuazFzMC'
    '5zeXN0ZW0uY29tbW9uLnYxLlRpbWVzdGFtcEgCUg1lZmZlY3RpdmVGcm9tiAEBEk4KD2VmZmVj'
    'dGl2ZV91bnRpbBgIIAEoCzIgLmsxczAuc3lzdGVtLmNvbW1vbi52MS5UaW1lc3RhbXBIA1IOZW'
    'ZmZWN0aXZlVW50aWyIAQESIAoJaXNfYWN0aXZlGAkgASgISARSCGlzQWN0aXZliAEBEiIKCnNv'
    'cnRfb3JkZXIYCiABKAVIBVIJc29ydE9yZGVyiAEBQg4KDF9kZXNjcmlwdGlvbkIRCg9fcGFyZW'
    '50X2l0ZW1faWRCEQoPX2VmZmVjdGl2ZV9mcm9tQhIKEF9lZmZlY3RpdmVfdW50aWxCDAoKX2lz'
    'X2FjdGl2ZUINCgtfc29ydF9vcmRlcg==');

@$core.Deprecated('Use createItemResponseDescriptor instead')
const CreateItemResponse$json = {
  '1': 'CreateItemResponse',
  '2': [
    {
      '1': 'item',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.k1s0.business.accounting.domainmaster.v1.MasterItem',
      '10': 'item'
    },
  ],
};

/// Descriptor for `CreateItemResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List createItemResponseDescriptor = $convert.base64Decode(
    'ChJDcmVhdGVJdGVtUmVzcG9uc2USSAoEaXRlbRgBIAEoCzI0LmsxczAuYnVzaW5lc3MuYWNjb3'
    'VudGluZy5kb21haW5tYXN0ZXIudjEuTWFzdGVySXRlbVIEaXRlbQ==');

@$core.Deprecated('Use updateItemRequestDescriptor instead')
const UpdateItemRequest$json = {
  '1': 'UpdateItemRequest',
  '2': [
    {'1': 'item_id', '3': 1, '4': 1, '5': 9, '10': 'itemId'},
    {
      '1': 'display_name',
      '3': 2,
      '4': 1,
      '5': 9,
      '9': 0,
      '10': 'displayName',
      '17': true
    },
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
      '1': 'attributes',
      '3': 4,
      '4': 1,
      '5': 11,
      '6': '.google.protobuf.Struct',
      '10': 'attributes'
    },
    {
      '1': 'parent_item_id',
      '3': 5,
      '4': 1,
      '5': 9,
      '9': 2,
      '10': 'parentItemId',
      '17': true
    },
    {
      '1': 'effective_from',
      '3': 6,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.common.v1.Timestamp',
      '9': 3,
      '10': 'effectiveFrom',
      '17': true
    },
    {
      '1': 'effective_until',
      '3': 7,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.common.v1.Timestamp',
      '9': 4,
      '10': 'effectiveUntil',
      '17': true
    },
    {
      '1': 'is_active',
      '3': 8,
      '4': 1,
      '5': 8,
      '9': 5,
      '10': 'isActive',
      '17': true
    },
    {
      '1': 'sort_order',
      '3': 9,
      '4': 1,
      '5': 5,
      '9': 6,
      '10': 'sortOrder',
      '17': true
    },
  ],
  '8': [
    {'1': '_display_name'},
    {'1': '_description'},
    {'1': '_parent_item_id'},
    {'1': '_effective_from'},
    {'1': '_effective_until'},
    {'1': '_is_active'},
    {'1': '_sort_order'},
  ],
};

/// Descriptor for `UpdateItemRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List updateItemRequestDescriptor = $convert.base64Decode(
    'ChFVcGRhdGVJdGVtUmVxdWVzdBIXCgdpdGVtX2lkGAEgASgJUgZpdGVtSWQSJgoMZGlzcGxheV'
    '9uYW1lGAIgASgJSABSC2Rpc3BsYXlOYW1liAEBEiUKC2Rlc2NyaXB0aW9uGAMgASgJSAFSC2Rl'
    'c2NyaXB0aW9uiAEBEjcKCmF0dHJpYnV0ZXMYBCABKAsyFy5nb29nbGUucHJvdG9idWYuU3RydW'
    'N0UgphdHRyaWJ1dGVzEikKDnBhcmVudF9pdGVtX2lkGAUgASgJSAJSDHBhcmVudEl0ZW1JZIgB'
    'ARJMCg5lZmZlY3RpdmVfZnJvbRgGIAEoCzIgLmsxczAuc3lzdGVtLmNvbW1vbi52MS5UaW1lc3'
    'RhbXBIA1INZWZmZWN0aXZlRnJvbYgBARJOCg9lZmZlY3RpdmVfdW50aWwYByABKAsyIC5rMXMw'
    'LnN5c3RlbS5jb21tb24udjEuVGltZXN0YW1wSARSDmVmZmVjdGl2ZVVudGlsiAEBEiAKCWlzX2'
    'FjdGl2ZRgIIAEoCEgFUghpc0FjdGl2ZYgBARIiCgpzb3J0X29yZGVyGAkgASgFSAZSCXNvcnRP'
    'cmRlcogBAUIPCg1fZGlzcGxheV9uYW1lQg4KDF9kZXNjcmlwdGlvbkIRCg9fcGFyZW50X2l0ZW'
    '1faWRCEQoPX2VmZmVjdGl2ZV9mcm9tQhIKEF9lZmZlY3RpdmVfdW50aWxCDAoKX2lzX2FjdGl2'
    'ZUINCgtfc29ydF9vcmRlcg==');

@$core.Deprecated('Use updateItemResponseDescriptor instead')
const UpdateItemResponse$json = {
  '1': 'UpdateItemResponse',
  '2': [
    {
      '1': 'item',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.k1s0.business.accounting.domainmaster.v1.MasterItem',
      '10': 'item'
    },
  ],
};

/// Descriptor for `UpdateItemResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List updateItemResponseDescriptor = $convert.base64Decode(
    'ChJVcGRhdGVJdGVtUmVzcG9uc2USSAoEaXRlbRgBIAEoCzI0LmsxczAuYnVzaW5lc3MuYWNjb3'
    'VudGluZy5kb21haW5tYXN0ZXIudjEuTWFzdGVySXRlbVIEaXRlbQ==');

@$core.Deprecated('Use deleteItemRequestDescriptor instead')
const DeleteItemRequest$json = {
  '1': 'DeleteItemRequest',
  '2': [
    {'1': 'item_id', '3': 1, '4': 1, '5': 9, '10': 'itemId'},
  ],
};

/// Descriptor for `DeleteItemRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List deleteItemRequestDescriptor = $convert.base64Decode(
    'ChFEZWxldGVJdGVtUmVxdWVzdBIXCgdpdGVtX2lkGAEgASgJUgZpdGVtSWQ=');

@$core.Deprecated('Use deleteItemResponseDescriptor instead')
const DeleteItemResponse$json = {
  '1': 'DeleteItemResponse',
  '2': [
    {'1': 'success', '3': 1, '4': 1, '5': 8, '10': 'success'},
  ],
};

/// Descriptor for `DeleteItemResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List deleteItemResponseDescriptor =
    $convert.base64Decode(
        'ChJEZWxldGVJdGVtUmVzcG9uc2USGAoHc3VjY2VzcxgBIAEoCFIHc3VjY2Vzcw==');

@$core.Deprecated('Use listItemVersionsRequestDescriptor instead')
const ListItemVersionsRequest$json = {
  '1': 'ListItemVersionsRequest',
  '2': [
    {'1': 'item_id', '3': 1, '4': 1, '5': 9, '10': 'itemId'},
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

/// Descriptor for `ListItemVersionsRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List listItemVersionsRequestDescriptor = $convert.base64Decode(
    'ChdMaXN0SXRlbVZlcnNpb25zUmVxdWVzdBIXCgdpdGVtX2lkGAEgASgJUgZpdGVtSWQSQQoKcG'
    'FnaW5hdGlvbhgCIAEoCzIhLmsxczAuc3lzdGVtLmNvbW1vbi52MS5QYWdpbmF0aW9uUgpwYWdp'
    'bmF0aW9u');

@$core.Deprecated('Use listItemVersionsResponseDescriptor instead')
const ListItemVersionsResponse$json = {
  '1': 'ListItemVersionsResponse',
  '2': [
    {
      '1': 'versions',
      '3': 1,
      '4': 3,
      '5': 11,
      '6': '.k1s0.business.accounting.domainmaster.v1.MasterItemVersion',
      '10': 'versions'
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

/// Descriptor for `ListItemVersionsResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List listItemVersionsResponseDescriptor = $convert.base64Decode(
    'ChhMaXN0SXRlbVZlcnNpb25zUmVzcG9uc2USVwoIdmVyc2lvbnMYASADKAsyOy5rMXMwLmJ1c2'
    'luZXNzLmFjY291bnRpbmcuZG9tYWlubWFzdGVyLnYxLk1hc3Rlckl0ZW1WZXJzaW9uUgh2ZXJz'
    'aW9ucxJHCgpwYWdpbmF0aW9uGAIgASgLMicuazFzMC5zeXN0ZW0uY29tbW9uLnYxLlBhZ2luYX'
    'Rpb25SZXN1bHRSCnBhZ2luYXRpb24=');

@$core.Deprecated('Use getTenantExtensionRequestDescriptor instead')
const GetTenantExtensionRequest$json = {
  '1': 'GetTenantExtensionRequest',
  '2': [
    {'1': 'tenant_id', '3': 1, '4': 1, '5': 9, '10': 'tenantId'},
    {'1': 'item_id', '3': 2, '4': 1, '5': 9, '10': 'itemId'},
  ],
};

/// Descriptor for `GetTenantExtensionRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List getTenantExtensionRequestDescriptor =
    $convert.base64Decode(
        'ChlHZXRUZW5hbnRFeHRlbnNpb25SZXF1ZXN0EhsKCXRlbmFudF9pZBgBIAEoCVIIdGVuYW50SW'
        'QSFwoHaXRlbV9pZBgCIAEoCVIGaXRlbUlk');

@$core.Deprecated('Use getTenantExtensionResponseDescriptor instead')
const GetTenantExtensionResponse$json = {
  '1': 'GetTenantExtensionResponse',
  '2': [
    {
      '1': 'extension',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.k1s0.business.accounting.domainmaster.v1.TenantMasterExtension',
      '10': 'extension'
    },
  ],
};

/// Descriptor for `GetTenantExtensionResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List getTenantExtensionResponseDescriptor =
    $convert.base64Decode(
        'ChpHZXRUZW5hbnRFeHRlbnNpb25SZXNwb25zZRJdCglleHRlbnNpb24YASABKAsyPy5rMXMwLm'
        'J1c2luZXNzLmFjY291bnRpbmcuZG9tYWlubWFzdGVyLnYxLlRlbmFudE1hc3RlckV4dGVuc2lv'
        'blIJZXh0ZW5zaW9u');

@$core.Deprecated('Use upsertTenantExtensionRequestDescriptor instead')
const UpsertTenantExtensionRequest$json = {
  '1': 'UpsertTenantExtensionRequest',
  '2': [
    {'1': 'tenant_id', '3': 1, '4': 1, '5': 9, '10': 'tenantId'},
    {'1': 'item_id', '3': 2, '4': 1, '5': 9, '10': 'itemId'},
    {
      '1': 'display_name_override',
      '3': 3,
      '4': 1,
      '5': 9,
      '9': 0,
      '10': 'displayNameOverride',
      '17': true
    },
    {
      '1': 'attributes_override',
      '3': 4,
      '4': 1,
      '5': 11,
      '6': '.google.protobuf.Struct',
      '10': 'attributesOverride'
    },
    {
      '1': 'is_enabled',
      '3': 5,
      '4': 1,
      '5': 8,
      '9': 1,
      '10': 'isEnabled',
      '17': true
    },
  ],
  '8': [
    {'1': '_display_name_override'},
    {'1': '_is_enabled'},
  ],
};

/// Descriptor for `UpsertTenantExtensionRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List upsertTenantExtensionRequestDescriptor = $convert.base64Decode(
    'ChxVcHNlcnRUZW5hbnRFeHRlbnNpb25SZXF1ZXN0EhsKCXRlbmFudF9pZBgBIAEoCVIIdGVuYW'
    '50SWQSFwoHaXRlbV9pZBgCIAEoCVIGaXRlbUlkEjcKFWRpc3BsYXlfbmFtZV9vdmVycmlkZRgD'
    'IAEoCUgAUhNkaXNwbGF5TmFtZU92ZXJyaWRliAEBEkgKE2F0dHJpYnV0ZXNfb3ZlcnJpZGUYBC'
    'ABKAsyFy5nb29nbGUucHJvdG9idWYuU3RydWN0UhJhdHRyaWJ1dGVzT3ZlcnJpZGUSIgoKaXNf'
    'ZW5hYmxlZBgFIAEoCEgBUglpc0VuYWJsZWSIAQFCGAoWX2Rpc3BsYXlfbmFtZV9vdmVycmlkZU'
    'INCgtfaXNfZW5hYmxlZA==');

@$core.Deprecated('Use upsertTenantExtensionResponseDescriptor instead')
const UpsertTenantExtensionResponse$json = {
  '1': 'UpsertTenantExtensionResponse',
  '2': [
    {
      '1': 'extension',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.k1s0.business.accounting.domainmaster.v1.TenantMasterExtension',
      '10': 'extension'
    },
  ],
};

/// Descriptor for `UpsertTenantExtensionResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List upsertTenantExtensionResponseDescriptor =
    $convert.base64Decode(
        'Ch1VcHNlcnRUZW5hbnRFeHRlbnNpb25SZXNwb25zZRJdCglleHRlbnNpb24YASABKAsyPy5rMX'
        'MwLmJ1c2luZXNzLmFjY291bnRpbmcuZG9tYWlubWFzdGVyLnYxLlRlbmFudE1hc3RlckV4dGVu'
        'c2lvblIJZXh0ZW5zaW9u');

@$core.Deprecated('Use deleteTenantExtensionRequestDescriptor instead')
const DeleteTenantExtensionRequest$json = {
  '1': 'DeleteTenantExtensionRequest',
  '2': [
    {'1': 'tenant_id', '3': 1, '4': 1, '5': 9, '10': 'tenantId'},
    {'1': 'item_id', '3': 2, '4': 1, '5': 9, '10': 'itemId'},
  ],
};

/// Descriptor for `DeleteTenantExtensionRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List deleteTenantExtensionRequestDescriptor =
    $convert.base64Decode(
        'ChxEZWxldGVUZW5hbnRFeHRlbnNpb25SZXF1ZXN0EhsKCXRlbmFudF9pZBgBIAEoCVIIdGVuYW'
        '50SWQSFwoHaXRlbV9pZBgCIAEoCVIGaXRlbUlk');

@$core.Deprecated('Use deleteTenantExtensionResponseDescriptor instead')
const DeleteTenantExtensionResponse$json = {
  '1': 'DeleteTenantExtensionResponse',
  '2': [
    {'1': 'success', '3': 1, '4': 1, '5': 8, '10': 'success'},
  ],
};

/// Descriptor for `DeleteTenantExtensionResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List deleteTenantExtensionResponseDescriptor =
    $convert.base64Decode(
        'Ch1EZWxldGVUZW5hbnRFeHRlbnNpb25SZXNwb25zZRIYCgdzdWNjZXNzGAEgASgIUgdzdWNjZX'
        'Nz');

@$core.Deprecated('Use listTenantItemsRequestDescriptor instead')
const ListTenantItemsRequest$json = {
  '1': 'ListTenantItemsRequest',
  '2': [
    {'1': 'tenant_id', '3': 1, '4': 1, '5': 9, '10': 'tenantId'},
    {'1': 'category_id', '3': 2, '4': 1, '5': 9, '10': 'categoryId'},
    {'1': 'active_only', '3': 3, '4': 1, '5': 8, '10': 'activeOnly'},
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

/// Descriptor for `ListTenantItemsRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List listTenantItemsRequestDescriptor = $convert.base64Decode(
    'ChZMaXN0VGVuYW50SXRlbXNSZXF1ZXN0EhsKCXRlbmFudF9pZBgBIAEoCVIIdGVuYW50SWQSHw'
    'oLY2F0ZWdvcnlfaWQYAiABKAlSCmNhdGVnb3J5SWQSHwoLYWN0aXZlX29ubHkYAyABKAhSCmFj'
    'dGl2ZU9ubHkSQQoKcGFnaW5hdGlvbhgEIAEoCzIhLmsxczAuc3lzdGVtLmNvbW1vbi52MS5QYW'
    'dpbmF0aW9uUgpwYWdpbmF0aW9u');

@$core.Deprecated('Use listTenantItemsResponseDescriptor instead')
const ListTenantItemsResponse$json = {
  '1': 'ListTenantItemsResponse',
  '2': [
    {
      '1': 'items',
      '3': 1,
      '4': 3,
      '5': 11,
      '6': '.k1s0.business.accounting.domainmaster.v1.TenantMergedItem',
      '10': 'items'
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

/// Descriptor for `ListTenantItemsResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List listTenantItemsResponseDescriptor = $convert.base64Decode(
    'ChdMaXN0VGVuYW50SXRlbXNSZXNwb25zZRJQCgVpdGVtcxgBIAMoCzI6LmsxczAuYnVzaW5lc3'
    'MuYWNjb3VudGluZy5kb21haW5tYXN0ZXIudjEuVGVuYW50TWVyZ2VkSXRlbVIFaXRlbXMSRwoK'
    'cGFnaW5hdGlvbhgCIAEoCzInLmsxczAuc3lzdGVtLmNvbW1vbi52MS5QYWdpbmF0aW9uUmVzdW'
    'x0UgpwYWdpbmF0aW9u');
