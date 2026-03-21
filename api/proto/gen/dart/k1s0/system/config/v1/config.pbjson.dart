// This is a generated file - do not edit.
//
// Generated from k1s0/system/config/v1/config.proto.

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

@$core.Deprecated('Use configFieldTypeDescriptor instead')
const ConfigFieldType$json = {
  '1': 'ConfigFieldType',
  '2': [
    {'1': 'CONFIG_FIELD_TYPE_UNSPECIFIED', '2': 0},
    {'1': 'CONFIG_FIELD_TYPE_STRING', '2': 1},
    {'1': 'CONFIG_FIELD_TYPE_INTEGER', '2': 2},
    {'1': 'CONFIG_FIELD_TYPE_FLOAT', '2': 3},
    {'1': 'CONFIG_FIELD_TYPE_BOOLEAN', '2': 4},
    {'1': 'CONFIG_FIELD_TYPE_ENUM', '2': 5},
    {'1': 'CONFIG_FIELD_TYPE_OBJECT', '2': 6},
    {'1': 'CONFIG_FIELD_TYPE_ARRAY', '2': 7},
  ],
};

/// Descriptor for `ConfigFieldType`. Decode as a `google.protobuf.EnumDescriptorProto`.
final $typed_data.Uint8List configFieldTypeDescriptor = $convert.base64Decode(
    'Cg9Db25maWdGaWVsZFR5cGUSIQodQ09ORklHX0ZJRUxEX1RZUEVfVU5TUEVDSUZJRUQQABIcCh'
    'hDT05GSUdfRklFTERfVFlQRV9TVFJJTkcQARIdChlDT05GSUdfRklFTERfVFlQRV9JTlRFR0VS'
    'EAISGwoXQ09ORklHX0ZJRUxEX1RZUEVfRkxPQVQQAxIdChlDT05GSUdfRklFTERfVFlQRV9CT0'
    '9MRUFOEAQSGgoWQ09ORklHX0ZJRUxEX1RZUEVfRU5VTRAFEhwKGENPTkZJR19GSUVMRF9UWVBF'
    'X09CSkVDVBAGEhsKF0NPTkZJR19GSUVMRF9UWVBFX0FSUkFZEAc=');

@$core.Deprecated('Use configEntryDescriptor instead')
const ConfigEntry$json = {
  '1': 'ConfigEntry',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 9, '10': 'id'},
    {'1': 'namespace', '3': 2, '4': 1, '5': 9, '10': 'namespace'},
    {'1': 'key', '3': 3, '4': 1, '5': 9, '10': 'key'},
    {'1': 'value', '3': 4, '4': 1, '5': 12, '10': 'value'},
    {'1': 'version', '3': 5, '4': 1, '5': 5, '10': 'version'},
    {'1': 'description', '3': 6, '4': 1, '5': 9, '10': 'description'},
    {'1': 'created_by', '3': 7, '4': 1, '5': 9, '10': 'createdBy'},
    {'1': 'updated_by', '3': 8, '4': 1, '5': 9, '10': 'updatedBy'},
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

/// Descriptor for `ConfigEntry`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List configEntryDescriptor = $convert.base64Decode(
    'CgtDb25maWdFbnRyeRIOCgJpZBgBIAEoCVICaWQSHAoJbmFtZXNwYWNlGAIgASgJUgluYW1lc3'
    'BhY2USEAoDa2V5GAMgASgJUgNrZXkSFAoFdmFsdWUYBCABKAxSBXZhbHVlEhgKB3ZlcnNpb24Y'
    'BSABKAVSB3ZlcnNpb24SIAoLZGVzY3JpcHRpb24YBiABKAlSC2Rlc2NyaXB0aW9uEh0KCmNyZW'
    'F0ZWRfYnkYByABKAlSCWNyZWF0ZWRCeRIdCgp1cGRhdGVkX2J5GAggASgJUgl1cGRhdGVkQnkS'
    'PwoKY3JlYXRlZF9hdBgJIAEoCzIgLmsxczAuc3lzdGVtLmNvbW1vbi52MS5UaW1lc3RhbXBSCW'
    'NyZWF0ZWRBdBI/Cgp1cGRhdGVkX2F0GAogASgLMiAuazFzMC5zeXN0ZW0uY29tbW9uLnYxLlRp'
    'bWVzdGFtcFIJdXBkYXRlZEF0');

@$core.Deprecated('Use getConfigRequestDescriptor instead')
const GetConfigRequest$json = {
  '1': 'GetConfigRequest',
  '2': [
    {'1': 'namespace', '3': 1, '4': 1, '5': 9, '10': 'namespace'},
    {'1': 'key', '3': 2, '4': 1, '5': 9, '10': 'key'},
  ],
};

/// Descriptor for `GetConfigRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List getConfigRequestDescriptor = $convert.base64Decode(
    'ChBHZXRDb25maWdSZXF1ZXN0EhwKCW5hbWVzcGFjZRgBIAEoCVIJbmFtZXNwYWNlEhAKA2tleR'
    'gCIAEoCVIDa2V5');

@$core.Deprecated('Use getConfigResponseDescriptor instead')
const GetConfigResponse$json = {
  '1': 'GetConfigResponse',
  '2': [
    {
      '1': 'entry',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.config.v1.ConfigEntry',
      '10': 'entry'
    },
  ],
};

/// Descriptor for `GetConfigResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List getConfigResponseDescriptor = $convert.base64Decode(
    'ChFHZXRDb25maWdSZXNwb25zZRI4CgVlbnRyeRgBIAEoCzIiLmsxczAuc3lzdGVtLmNvbmZpZy'
    '52MS5Db25maWdFbnRyeVIFZW50cnk=');

@$core.Deprecated('Use listConfigsRequestDescriptor instead')
const ListConfigsRequest$json = {
  '1': 'ListConfigsRequest',
  '2': [
    {'1': 'namespace', '3': 1, '4': 1, '5': 9, '10': 'namespace'},
    {
      '1': 'pagination',
      '3': 2,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.common.v1.Pagination',
      '10': 'pagination'
    },
    {'1': 'search', '3': 3, '4': 1, '5': 9, '10': 'search'},
  ],
};

/// Descriptor for `ListConfigsRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List listConfigsRequestDescriptor = $convert.base64Decode(
    'ChJMaXN0Q29uZmlnc1JlcXVlc3QSHAoJbmFtZXNwYWNlGAEgASgJUgluYW1lc3BhY2USQQoKcG'
    'FnaW5hdGlvbhgCIAEoCzIhLmsxczAuc3lzdGVtLmNvbW1vbi52MS5QYWdpbmF0aW9uUgpwYWdp'
    'bmF0aW9uEhYKBnNlYXJjaBgDIAEoCVIGc2VhcmNo');

@$core.Deprecated('Use listConfigsResponseDescriptor instead')
const ListConfigsResponse$json = {
  '1': 'ListConfigsResponse',
  '2': [
    {
      '1': 'entries',
      '3': 1,
      '4': 3,
      '5': 11,
      '6': '.k1s0.system.config.v1.ConfigEntry',
      '10': 'entries'
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

/// Descriptor for `ListConfigsResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List listConfigsResponseDescriptor = $convert.base64Decode(
    'ChNMaXN0Q29uZmlnc1Jlc3BvbnNlEjwKB2VudHJpZXMYASADKAsyIi5rMXMwLnN5c3RlbS5jb2'
    '5maWcudjEuQ29uZmlnRW50cnlSB2VudHJpZXMSRwoKcGFnaW5hdGlvbhgCIAEoCzInLmsxczAu'
    'c3lzdGVtLmNvbW1vbi52MS5QYWdpbmF0aW9uUmVzdWx0UgpwYWdpbmF0aW9u');

@$core.Deprecated('Use updateConfigRequestDescriptor instead')
const UpdateConfigRequest$json = {
  '1': 'UpdateConfigRequest',
  '2': [
    {'1': 'namespace', '3': 1, '4': 1, '5': 9, '10': 'namespace'},
    {'1': 'key', '3': 2, '4': 1, '5': 9, '10': 'key'},
    {'1': 'value', '3': 3, '4': 1, '5': 12, '10': 'value'},
    {'1': 'version', '3': 4, '4': 1, '5': 5, '10': 'version'},
    {'1': 'description', '3': 5, '4': 1, '5': 9, '10': 'description'},
    {'1': 'updated_by', '3': 6, '4': 1, '5': 9, '10': 'updatedBy'},
  ],
};

/// Descriptor for `UpdateConfigRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List updateConfigRequestDescriptor = $convert.base64Decode(
    'ChNVcGRhdGVDb25maWdSZXF1ZXN0EhwKCW5hbWVzcGFjZRgBIAEoCVIJbmFtZXNwYWNlEhAKA2'
    'tleRgCIAEoCVIDa2V5EhQKBXZhbHVlGAMgASgMUgV2YWx1ZRIYCgd2ZXJzaW9uGAQgASgFUgd2'
    'ZXJzaW9uEiAKC2Rlc2NyaXB0aW9uGAUgASgJUgtkZXNjcmlwdGlvbhIdCgp1cGRhdGVkX2J5GA'
    'YgASgJUgl1cGRhdGVkQnk=');

@$core.Deprecated('Use updateConfigResponseDescriptor instead')
const UpdateConfigResponse$json = {
  '1': 'UpdateConfigResponse',
  '2': [
    {
      '1': 'entry',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.config.v1.ConfigEntry',
      '10': 'entry'
    },
  ],
};

/// Descriptor for `UpdateConfigResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List updateConfigResponseDescriptor = $convert.base64Decode(
    'ChRVcGRhdGVDb25maWdSZXNwb25zZRI4CgVlbnRyeRgBIAEoCzIiLmsxczAuc3lzdGVtLmNvbm'
    'ZpZy52MS5Db25maWdFbnRyeVIFZW50cnk=');

@$core.Deprecated('Use deleteConfigRequestDescriptor instead')
const DeleteConfigRequest$json = {
  '1': 'DeleteConfigRequest',
  '2': [
    {'1': 'namespace', '3': 1, '4': 1, '5': 9, '10': 'namespace'},
    {'1': 'key', '3': 2, '4': 1, '5': 9, '10': 'key'},
    {'1': 'deleted_by', '3': 3, '4': 1, '5': 9, '10': 'deletedBy'},
  ],
};

/// Descriptor for `DeleteConfigRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List deleteConfigRequestDescriptor = $convert.base64Decode(
    'ChNEZWxldGVDb25maWdSZXF1ZXN0EhwKCW5hbWVzcGFjZRgBIAEoCVIJbmFtZXNwYWNlEhAKA2'
    'tleRgCIAEoCVIDa2V5Eh0KCmRlbGV0ZWRfYnkYAyABKAlSCWRlbGV0ZWRCeQ==');

@$core.Deprecated('Use deleteConfigResponseDescriptor instead')
const DeleteConfigResponse$json = {
  '1': 'DeleteConfigResponse',
  '2': [
    {'1': 'success', '3': 1, '4': 1, '5': 8, '10': 'success'},
  ],
};

/// Descriptor for `DeleteConfigResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List deleteConfigResponseDescriptor =
    $convert.base64Decode(
        'ChREZWxldGVDb25maWdSZXNwb25zZRIYCgdzdWNjZXNzGAEgASgIUgdzdWNjZXNz');

@$core.Deprecated('Use getServiceConfigRequestDescriptor instead')
const GetServiceConfigRequest$json = {
  '1': 'GetServiceConfigRequest',
  '2': [
    {'1': 'service_name', '3': 1, '4': 1, '5': 9, '10': 'serviceName'},
    {'1': 'environment', '3': 2, '4': 1, '5': 9, '10': 'environment'},
  ],
};

/// Descriptor for `GetServiceConfigRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List getServiceConfigRequestDescriptor =
    $convert.base64Decode(
        'ChdHZXRTZXJ2aWNlQ29uZmlnUmVxdWVzdBIhCgxzZXJ2aWNlX25hbWUYASABKAlSC3NlcnZpY2'
        'VOYW1lEiAKC2Vudmlyb25tZW50GAIgASgJUgtlbnZpcm9ubWVudA==');

@$core.Deprecated('Use serviceConfigEntryDescriptor instead')
const ServiceConfigEntry$json = {
  '1': 'ServiceConfigEntry',
  '2': [
    {'1': 'namespace', '3': 1, '4': 1, '5': 9, '10': 'namespace'},
    {'1': 'key', '3': 2, '4': 1, '5': 9, '10': 'key'},
    {'1': 'value', '3': 3, '4': 1, '5': 9, '10': 'value'},
    {'1': 'version', '3': 4, '4': 1, '5': 5, '10': 'version'},
  ],
};

/// Descriptor for `ServiceConfigEntry`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List serviceConfigEntryDescriptor = $convert.base64Decode(
    'ChJTZXJ2aWNlQ29uZmlnRW50cnkSHAoJbmFtZXNwYWNlGAEgASgJUgluYW1lc3BhY2USEAoDa2'
    'V5GAIgASgJUgNrZXkSFAoFdmFsdWUYAyABKAlSBXZhbHVlEhgKB3ZlcnNpb24YBCABKAVSB3Zl'
    'cnNpb24=');

@$core.Deprecated('Use getServiceConfigResponseDescriptor instead')
const GetServiceConfigResponse$json = {
  '1': 'GetServiceConfigResponse',
  '2': [
    {
      '1': 'entries',
      '3': 1,
      '4': 3,
      '5': 11,
      '6': '.k1s0.system.config.v1.ServiceConfigEntry',
      '10': 'entries'
    },
  ],
};

/// Descriptor for `GetServiceConfigResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List getServiceConfigResponseDescriptor =
    $convert.base64Decode(
        'ChhHZXRTZXJ2aWNlQ29uZmlnUmVzcG9uc2USQwoHZW50cmllcxgBIAMoCzIpLmsxczAuc3lzdG'
        'VtLmNvbmZpZy52MS5TZXJ2aWNlQ29uZmlnRW50cnlSB2VudHJpZXM=');

@$core.Deprecated('Use watchConfigRequestDescriptor instead')
const WatchConfigRequest$json = {
  '1': 'WatchConfigRequest',
  '2': [
    {'1': 'namespaces', '3': 1, '4': 3, '5': 9, '10': 'namespaces'},
  ],
};

/// Descriptor for `WatchConfigRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List watchConfigRequestDescriptor = $convert.base64Decode(
    'ChJXYXRjaENvbmZpZ1JlcXVlc3QSHgoKbmFtZXNwYWNlcxgBIAMoCVIKbmFtZXNwYWNlcw==');

@$core.Deprecated('Use watchConfigResponseDescriptor instead')
const WatchConfigResponse$json = {
  '1': 'WatchConfigResponse',
  '2': [
    {'1': 'namespace', '3': 1, '4': 1, '5': 9, '10': 'namespace'},
    {'1': 'key', '3': 2, '4': 1, '5': 9, '10': 'key'},
    {'1': 'old_value', '3': 3, '4': 1, '5': 12, '10': 'oldValue'},
    {'1': 'new_value', '3': 4, '4': 1, '5': 12, '10': 'newValue'},
    {'1': 'old_version', '3': 5, '4': 1, '5': 5, '10': 'oldVersion'},
    {'1': 'new_version', '3': 6, '4': 1, '5': 5, '10': 'newVersion'},
    {'1': 'changed_by', '3': 7, '4': 1, '5': 9, '10': 'changedBy'},
    {'1': 'change_type', '3': 8, '4': 1, '5': 9, '10': 'changeType'},
    {
      '1': 'changed_at',
      '3': 9,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.common.v1.Timestamp',
      '10': 'changedAt'
    },
    {
      '1': 'change_type_enum',
      '3': 10,
      '4': 1,
      '5': 14,
      '6': '.k1s0.system.common.v1.ChangeType',
      '10': 'changeTypeEnum'
    },
  ],
};

/// Descriptor for `WatchConfigResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List watchConfigResponseDescriptor = $convert.base64Decode(
    'ChNXYXRjaENvbmZpZ1Jlc3BvbnNlEhwKCW5hbWVzcGFjZRgBIAEoCVIJbmFtZXNwYWNlEhAKA2'
    'tleRgCIAEoCVIDa2V5EhsKCW9sZF92YWx1ZRgDIAEoDFIIb2xkVmFsdWUSGwoJbmV3X3ZhbHVl'
    'GAQgASgMUghuZXdWYWx1ZRIfCgtvbGRfdmVyc2lvbhgFIAEoBVIKb2xkVmVyc2lvbhIfCgtuZX'
    'dfdmVyc2lvbhgGIAEoBVIKbmV3VmVyc2lvbhIdCgpjaGFuZ2VkX2J5GAcgASgJUgljaGFuZ2Vk'
    'QnkSHwoLY2hhbmdlX3R5cGUYCCABKAlSCmNoYW5nZVR5cGUSPwoKY2hhbmdlZF9hdBgJIAEoCz'
    'IgLmsxczAuc3lzdGVtLmNvbW1vbi52MS5UaW1lc3RhbXBSCWNoYW5nZWRBdBJLChBjaGFuZ2Vf'
    'dHlwZV9lbnVtGAogASgOMiEuazFzMC5zeXN0ZW0uY29tbW9uLnYxLkNoYW5nZVR5cGVSDmNoYW'
    '5nZVR5cGVFbnVt');

@$core.Deprecated('Use configFieldSchemaDescriptor instead')
const ConfigFieldSchema$json = {
  '1': 'ConfigFieldSchema',
  '2': [
    {'1': 'key', '3': 1, '4': 1, '5': 9, '10': 'key'},
    {'1': 'label', '3': 2, '4': 1, '5': 9, '10': 'label'},
    {'1': 'description', '3': 3, '4': 1, '5': 9, '10': 'description'},
    {
      '1': 'type',
      '3': 4,
      '4': 1,
      '5': 14,
      '6': '.k1s0.system.config.v1.ConfigFieldType',
      '10': 'type'
    },
    {'1': 'min', '3': 5, '4': 1, '5': 3, '10': 'min'},
    {'1': 'max', '3': 6, '4': 1, '5': 3, '10': 'max'},
    {'1': 'options', '3': 7, '4': 3, '5': 9, '10': 'options'},
    {'1': 'pattern', '3': 8, '4': 1, '5': 9, '10': 'pattern'},
    {'1': 'unit', '3': 9, '4': 1, '5': 9, '10': 'unit'},
    {'1': 'default_value', '3': 10, '4': 1, '5': 12, '10': 'defaultValue'},
  ],
};

/// Descriptor for `ConfigFieldSchema`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List configFieldSchemaDescriptor = $convert.base64Decode(
    'ChFDb25maWdGaWVsZFNjaGVtYRIQCgNrZXkYASABKAlSA2tleRIUCgVsYWJlbBgCIAEoCVIFbG'
    'FiZWwSIAoLZGVzY3JpcHRpb24YAyABKAlSC2Rlc2NyaXB0aW9uEjoKBHR5cGUYBCABKA4yJi5r'
    'MXMwLnN5c3RlbS5jb25maWcudjEuQ29uZmlnRmllbGRUeXBlUgR0eXBlEhAKA21pbhgFIAEoA1'
    'IDbWluEhAKA21heBgGIAEoA1IDbWF4EhgKB29wdGlvbnMYByADKAlSB29wdGlvbnMSGAoHcGF0'
    'dGVybhgIIAEoCVIHcGF0dGVybhISCgR1bml0GAkgASgJUgR1bml0EiMKDWRlZmF1bHRfdmFsdW'
    'UYCiABKAxSDGRlZmF1bHRWYWx1ZQ==');

@$core.Deprecated('Use configCategorySchemaDescriptor instead')
const ConfigCategorySchema$json = {
  '1': 'ConfigCategorySchema',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 9, '10': 'id'},
    {'1': 'label', '3': 2, '4': 1, '5': 9, '10': 'label'},
    {'1': 'icon', '3': 3, '4': 1, '5': 9, '10': 'icon'},
    {'1': 'namespaces', '3': 4, '4': 3, '5': 9, '10': 'namespaces'},
    {
      '1': 'fields',
      '3': 5,
      '4': 3,
      '5': 11,
      '6': '.k1s0.system.config.v1.ConfigFieldSchema',
      '10': 'fields'
    },
  ],
};

/// Descriptor for `ConfigCategorySchema`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List configCategorySchemaDescriptor = $convert.base64Decode(
    'ChRDb25maWdDYXRlZ29yeVNjaGVtYRIOCgJpZBgBIAEoCVICaWQSFAoFbGFiZWwYAiABKAlSBW'
    'xhYmVsEhIKBGljb24YAyABKAlSBGljb24SHgoKbmFtZXNwYWNlcxgEIAMoCVIKbmFtZXNwYWNl'
    'cxJACgZmaWVsZHMYBSADKAsyKC5rMXMwLnN5c3RlbS5jb25maWcudjEuQ29uZmlnRmllbGRTY2'
    'hlbWFSBmZpZWxkcw==');

@$core.Deprecated('Use configEditorSchemaDescriptor instead')
const ConfigEditorSchema$json = {
  '1': 'ConfigEditorSchema',
  '2': [
    {'1': 'service_name', '3': 1, '4': 1, '5': 9, '10': 'serviceName'},
    {'1': 'namespace_prefix', '3': 2, '4': 1, '5': 9, '10': 'namespacePrefix'},
    {
      '1': 'categories',
      '3': 3,
      '4': 3,
      '5': 11,
      '6': '.k1s0.system.config.v1.ConfigCategorySchema',
      '10': 'categories'
    },
    {
      '1': 'updated_at',
      '3': 4,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.common.v1.Timestamp',
      '10': 'updatedAt'
    },
  ],
};

/// Descriptor for `ConfigEditorSchema`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List configEditorSchemaDescriptor = $convert.base64Decode(
    'ChJDb25maWdFZGl0b3JTY2hlbWESIQoMc2VydmljZV9uYW1lGAEgASgJUgtzZXJ2aWNlTmFtZR'
    'IpChBuYW1lc3BhY2VfcHJlZml4GAIgASgJUg9uYW1lc3BhY2VQcmVmaXgSSwoKY2F0ZWdvcmll'
    'cxgDIAMoCzIrLmsxczAuc3lzdGVtLmNvbmZpZy52MS5Db25maWdDYXRlZ29yeVNjaGVtYVIKY2'
    'F0ZWdvcmllcxI/Cgp1cGRhdGVkX2F0GAQgASgLMiAuazFzMC5zeXN0ZW0uY29tbW9uLnYxLlRp'
    'bWVzdGFtcFIJdXBkYXRlZEF0');

@$core.Deprecated('Use getConfigSchemaRequestDescriptor instead')
const GetConfigSchemaRequest$json = {
  '1': 'GetConfigSchemaRequest',
  '2': [
    {'1': 'service_name', '3': 1, '4': 1, '5': 9, '10': 'serviceName'},
  ],
};

/// Descriptor for `GetConfigSchemaRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List getConfigSchemaRequestDescriptor =
    $convert.base64Decode(
        'ChZHZXRDb25maWdTY2hlbWFSZXF1ZXN0EiEKDHNlcnZpY2VfbmFtZRgBIAEoCVILc2VydmljZU'
        '5hbWU=');

@$core.Deprecated('Use getConfigSchemaResponseDescriptor instead')
const GetConfigSchemaResponse$json = {
  '1': 'GetConfigSchemaResponse',
  '2': [
    {
      '1': 'schema',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.config.v1.ConfigEditorSchema',
      '10': 'schema'
    },
  ],
};

/// Descriptor for `GetConfigSchemaResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List getConfigSchemaResponseDescriptor =
    $convert.base64Decode(
        'ChdHZXRDb25maWdTY2hlbWFSZXNwb25zZRJBCgZzY2hlbWEYASABKAsyKS5rMXMwLnN5c3RlbS'
        '5jb25maWcudjEuQ29uZmlnRWRpdG9yU2NoZW1hUgZzY2hlbWE=');

@$core.Deprecated('Use upsertConfigSchemaRequestDescriptor instead')
const UpsertConfigSchemaRequest$json = {
  '1': 'UpsertConfigSchemaRequest',
  '2': [
    {
      '1': 'schema',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.config.v1.ConfigEditorSchema',
      '10': 'schema'
    },
    {'1': 'updated_by', '3': 2, '4': 1, '5': 9, '10': 'updatedBy'},
  ],
};

/// Descriptor for `UpsertConfigSchemaRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List upsertConfigSchemaRequestDescriptor = $convert.base64Decode(
    'ChlVcHNlcnRDb25maWdTY2hlbWFSZXF1ZXN0EkEKBnNjaGVtYRgBIAEoCzIpLmsxczAuc3lzdG'
    'VtLmNvbmZpZy52MS5Db25maWdFZGl0b3JTY2hlbWFSBnNjaGVtYRIdCgp1cGRhdGVkX2J5GAIg'
    'ASgJUgl1cGRhdGVkQnk=');

@$core.Deprecated('Use upsertConfigSchemaResponseDescriptor instead')
const UpsertConfigSchemaResponse$json = {
  '1': 'UpsertConfigSchemaResponse',
  '2': [
    {
      '1': 'schema',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.config.v1.ConfigEditorSchema',
      '10': 'schema'
    },
  ],
};

/// Descriptor for `UpsertConfigSchemaResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List upsertConfigSchemaResponseDescriptor =
    $convert.base64Decode(
        'ChpVcHNlcnRDb25maWdTY2hlbWFSZXNwb25zZRJBCgZzY2hlbWEYASABKAsyKS5rMXMwLnN5c3'
        'RlbS5jb25maWcudjEuQ29uZmlnRWRpdG9yU2NoZW1hUgZzY2hlbWE=');

@$core.Deprecated('Use listConfigSchemasRequestDescriptor instead')
const ListConfigSchemasRequest$json = {
  '1': 'ListConfigSchemasRequest',
};

/// Descriptor for `ListConfigSchemasRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List listConfigSchemasRequestDescriptor =
    $convert.base64Decode('ChhMaXN0Q29uZmlnU2NoZW1hc1JlcXVlc3Q=');

@$core.Deprecated('Use listConfigSchemasResponseDescriptor instead')
const ListConfigSchemasResponse$json = {
  '1': 'ListConfigSchemasResponse',
  '2': [
    {
      '1': 'schemas',
      '3': 1,
      '4': 3,
      '5': 11,
      '6': '.k1s0.system.config.v1.ConfigEditorSchema',
      '10': 'schemas'
    },
  ],
};

/// Descriptor for `ListConfigSchemasResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List listConfigSchemasResponseDescriptor =
    $convert.base64Decode(
        'ChlMaXN0Q29uZmlnU2NoZW1hc1Jlc3BvbnNlEkMKB3NjaGVtYXMYASADKAsyKS5rMXMwLnN5c3'
        'RlbS5jb25maWcudjEuQ29uZmlnRWRpdG9yU2NoZW1hUgdzY2hlbWFz');
