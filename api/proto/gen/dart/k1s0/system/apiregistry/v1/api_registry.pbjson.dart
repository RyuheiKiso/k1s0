// This is a generated file - do not edit.
//
// Generated from k1s0/system/apiregistry/v1/api_registry.proto.

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

@$core.Deprecated('Use getSchemaRequestDescriptor instead')
const GetSchemaRequest$json = {
  '1': 'GetSchemaRequest',
  '2': [
    {'1': 'name', '3': 1, '4': 1, '5': 9, '10': 'name'},
  ],
};

/// Descriptor for `GetSchemaRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List getSchemaRequestDescriptor = $convert
    .base64Decode('ChBHZXRTY2hlbWFSZXF1ZXN0EhIKBG5hbWUYASABKAlSBG5hbWU=');

@$core.Deprecated('Use listSchemasRequestDescriptor instead')
const ListSchemasRequest$json = {
  '1': 'ListSchemasRequest',
  '2': [
    {'1': 'schema_type', '3': 1, '4': 1, '5': 9, '10': 'schemaType'},
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

/// Descriptor for `ListSchemasRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List listSchemasRequestDescriptor = $convert.base64Decode(
    'ChJMaXN0U2NoZW1hc1JlcXVlc3QSHwoLc2NoZW1hX3R5cGUYASABKAlSCnNjaGVtYVR5cGUSQQ'
    'oKcGFnaW5hdGlvbhgCIAEoCzIhLmsxczAuc3lzdGVtLmNvbW1vbi52MS5QYWdpbmF0aW9uUgpw'
    'YWdpbmF0aW9u');

@$core.Deprecated('Use listSchemasResponseDescriptor instead')
const ListSchemasResponse$json = {
  '1': 'ListSchemasResponse',
  '2': [
    {
      '1': 'schemas',
      '3': 1,
      '4': 3,
      '5': 11,
      '6': '.k1s0.system.apiregistry.v1.ApiSchemaProto',
      '10': 'schemas'
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

/// Descriptor for `ListSchemasResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List listSchemasResponseDescriptor = $convert.base64Decode(
    'ChNMaXN0U2NoZW1hc1Jlc3BvbnNlEkQKB3NjaGVtYXMYASADKAsyKi5rMXMwLnN5c3RlbS5hcG'
    'lyZWdpc3RyeS52MS5BcGlTY2hlbWFQcm90b1IHc2NoZW1hcxJHCgpwYWdpbmF0aW9uGAIgASgL'
    'MicuazFzMC5zeXN0ZW0uY29tbW9uLnYxLlBhZ2luYXRpb25SZXN1bHRSCnBhZ2luYXRpb24=');

@$core.Deprecated('Use registerSchemaRequestDescriptor instead')
const RegisterSchemaRequest$json = {
  '1': 'RegisterSchemaRequest',
  '2': [
    {'1': 'name', '3': 1, '4': 1, '5': 9, '10': 'name'},
    {'1': 'description', '3': 2, '4': 1, '5': 9, '10': 'description'},
    {'1': 'schema_type', '3': 3, '4': 1, '5': 9, '10': 'schemaType'},
    {'1': 'content', '3': 4, '4': 1, '5': 9, '10': 'content'},
    {'1': 'registered_by', '3': 5, '4': 1, '5': 9, '10': 'registeredBy'},
  ],
};

/// Descriptor for `RegisterSchemaRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List registerSchemaRequestDescriptor = $convert.base64Decode(
    'ChVSZWdpc3RlclNjaGVtYVJlcXVlc3QSEgoEbmFtZRgBIAEoCVIEbmFtZRIgCgtkZXNjcmlwdG'
    'lvbhgCIAEoCVILZGVzY3JpcHRpb24SHwoLc2NoZW1hX3R5cGUYAyABKAlSCnNjaGVtYVR5cGUS'
    'GAoHY29udGVudBgEIAEoCVIHY29udGVudBIjCg1yZWdpc3RlcmVkX2J5GAUgASgJUgxyZWdpc3'
    'RlcmVkQnk=');

@$core.Deprecated('Use registerSchemaResponseDescriptor instead')
const RegisterSchemaResponse$json = {
  '1': 'RegisterSchemaResponse',
  '2': [
    {
      '1': 'version',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.apiregistry.v1.ApiSchemaVersionProto',
      '10': 'version'
    },
  ],
};

/// Descriptor for `RegisterSchemaResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List registerSchemaResponseDescriptor =
    $convert.base64Decode(
        'ChZSZWdpc3RlclNjaGVtYVJlc3BvbnNlEksKB3ZlcnNpb24YASABKAsyMS5rMXMwLnN5c3RlbS'
        '5hcGlyZWdpc3RyeS52MS5BcGlTY2hlbWFWZXJzaW9uUHJvdG9SB3ZlcnNpb24=');

@$core.Deprecated('Use getSchemaResponseDescriptor instead')
const GetSchemaResponse$json = {
  '1': 'GetSchemaResponse',
  '2': [
    {
      '1': 'schema',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.apiregistry.v1.ApiSchemaProto',
      '10': 'schema'
    },
    {'1': 'latest_content', '3': 2, '4': 1, '5': 9, '10': 'latestContent'},
  ],
};

/// Descriptor for `GetSchemaResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List getSchemaResponseDescriptor = $convert.base64Decode(
    'ChFHZXRTY2hlbWFSZXNwb25zZRJCCgZzY2hlbWEYASABKAsyKi5rMXMwLnN5c3RlbS5hcGlyZW'
    'dpc3RyeS52MS5BcGlTY2hlbWFQcm90b1IGc2NoZW1hEiUKDmxhdGVzdF9jb250ZW50GAIgASgJ'
    'Ug1sYXRlc3RDb250ZW50');

@$core.Deprecated('Use listVersionsRequestDescriptor instead')
const ListVersionsRequest$json = {
  '1': 'ListVersionsRequest',
  '2': [
    {'1': 'name', '3': 1, '4': 1, '5': 9, '10': 'name'},
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

/// Descriptor for `ListVersionsRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List listVersionsRequestDescriptor = $convert.base64Decode(
    'ChNMaXN0VmVyc2lvbnNSZXF1ZXN0EhIKBG5hbWUYASABKAlSBG5hbWUSQQoKcGFnaW5hdGlvbh'
    'gCIAEoCzIhLmsxczAuc3lzdGVtLmNvbW1vbi52MS5QYWdpbmF0aW9uUgpwYWdpbmF0aW9u');

@$core.Deprecated('Use listVersionsResponseDescriptor instead')
const ListVersionsResponse$json = {
  '1': 'ListVersionsResponse',
  '2': [
    {'1': 'name', '3': 1, '4': 1, '5': 9, '10': 'name'},
    {
      '1': 'versions',
      '3': 2,
      '4': 3,
      '5': 11,
      '6': '.k1s0.system.apiregistry.v1.ApiSchemaVersionProto',
      '10': 'versions'
    },
    {
      '1': 'pagination',
      '3': 3,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.common.v1.PaginationResult',
      '10': 'pagination'
    },
  ],
};

/// Descriptor for `ListVersionsResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List listVersionsResponseDescriptor = $convert.base64Decode(
    'ChRMaXN0VmVyc2lvbnNSZXNwb25zZRISCgRuYW1lGAEgASgJUgRuYW1lEk0KCHZlcnNpb25zGA'
    'IgAygLMjEuazFzMC5zeXN0ZW0uYXBpcmVnaXN0cnkudjEuQXBpU2NoZW1hVmVyc2lvblByb3Rv'
    'Ugh2ZXJzaW9ucxJHCgpwYWdpbmF0aW9uGAMgASgLMicuazFzMC5zeXN0ZW0uY29tbW9uLnYxLl'
    'BhZ2luYXRpb25SZXN1bHRSCnBhZ2luYXRpb24=');

@$core.Deprecated('Use registerVersionRequestDescriptor instead')
const RegisterVersionRequest$json = {
  '1': 'RegisterVersionRequest',
  '2': [
    {'1': 'name', '3': 1, '4': 1, '5': 9, '10': 'name'},
    {'1': 'content', '3': 2, '4': 1, '5': 9, '10': 'content'},
    {'1': 'registered_by', '3': 3, '4': 1, '5': 9, '10': 'registeredBy'},
  ],
};

/// Descriptor for `RegisterVersionRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List registerVersionRequestDescriptor = $convert.base64Decode(
    'ChZSZWdpc3RlclZlcnNpb25SZXF1ZXN0EhIKBG5hbWUYASABKAlSBG5hbWUSGAoHY29udGVudB'
    'gCIAEoCVIHY29udGVudBIjCg1yZWdpc3RlcmVkX2J5GAMgASgJUgxyZWdpc3RlcmVkQnk=');

@$core.Deprecated('Use registerVersionResponseDescriptor instead')
const RegisterVersionResponse$json = {
  '1': 'RegisterVersionResponse',
  '2': [
    {
      '1': 'version',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.apiregistry.v1.ApiSchemaVersionProto',
      '10': 'version'
    },
  ],
};

/// Descriptor for `RegisterVersionResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List registerVersionResponseDescriptor =
    $convert.base64Decode(
        'ChdSZWdpc3RlclZlcnNpb25SZXNwb25zZRJLCgd2ZXJzaW9uGAEgASgLMjEuazFzMC5zeXN0ZW'
        '0uYXBpcmVnaXN0cnkudjEuQXBpU2NoZW1hVmVyc2lvblByb3RvUgd2ZXJzaW9u');

@$core.Deprecated('Use getSchemaVersionRequestDescriptor instead')
const GetSchemaVersionRequest$json = {
  '1': 'GetSchemaVersionRequest',
  '2': [
    {'1': 'name', '3': 1, '4': 1, '5': 9, '10': 'name'},
    {'1': 'version', '3': 2, '4': 1, '5': 13, '10': 'version'},
  ],
};

/// Descriptor for `GetSchemaVersionRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List getSchemaVersionRequestDescriptor =
    $convert.base64Decode(
        'ChdHZXRTY2hlbWFWZXJzaW9uUmVxdWVzdBISCgRuYW1lGAEgASgJUgRuYW1lEhgKB3ZlcnNpb2'
        '4YAiABKA1SB3ZlcnNpb24=');

@$core.Deprecated('Use getSchemaVersionResponseDescriptor instead')
const GetSchemaVersionResponse$json = {
  '1': 'GetSchemaVersionResponse',
  '2': [
    {
      '1': 'version',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.apiregistry.v1.ApiSchemaVersionProto',
      '10': 'version'
    },
  ],
};

/// Descriptor for `GetSchemaVersionResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List getSchemaVersionResponseDescriptor =
    $convert.base64Decode(
        'ChhHZXRTY2hlbWFWZXJzaW9uUmVzcG9uc2USSwoHdmVyc2lvbhgBIAEoCzIxLmsxczAuc3lzdG'
        'VtLmFwaXJlZ2lzdHJ5LnYxLkFwaVNjaGVtYVZlcnNpb25Qcm90b1IHdmVyc2lvbg==');

@$core.Deprecated('Use deleteVersionRequestDescriptor instead')
const DeleteVersionRequest$json = {
  '1': 'DeleteVersionRequest',
  '2': [
    {'1': 'name', '3': 1, '4': 1, '5': 9, '10': 'name'},
    {'1': 'version', '3': 2, '4': 1, '5': 13, '10': 'version'},
  ],
};

/// Descriptor for `DeleteVersionRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List deleteVersionRequestDescriptor = $convert.base64Decode(
    'ChREZWxldGVWZXJzaW9uUmVxdWVzdBISCgRuYW1lGAEgASgJUgRuYW1lEhgKB3ZlcnNpb24YAi'
    'ABKA1SB3ZlcnNpb24=');

@$core.Deprecated('Use deleteVersionResponseDescriptor instead')
const DeleteVersionResponse$json = {
  '1': 'DeleteVersionResponse',
  '2': [
    {'1': 'success', '3': 1, '4': 1, '5': 8, '10': 'success'},
    {'1': 'message', '3': 2, '4': 1, '5': 9, '10': 'message'},
  ],
};

/// Descriptor for `DeleteVersionResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List deleteVersionResponseDescriptor = $convert.base64Decode(
    'ChVEZWxldGVWZXJzaW9uUmVzcG9uc2USGAoHc3VjY2VzcxgBIAEoCFIHc3VjY2VzcxIYCgdtZX'
    'NzYWdlGAIgASgJUgdtZXNzYWdl');

@$core.Deprecated('Use checkCompatibilityRequestDescriptor instead')
const CheckCompatibilityRequest$json = {
  '1': 'CheckCompatibilityRequest',
  '2': [
    {'1': 'name', '3': 1, '4': 1, '5': 9, '10': 'name'},
    {'1': 'content', '3': 2, '4': 1, '5': 9, '10': 'content'},
    {
      '1': 'base_version',
      '3': 3,
      '4': 1,
      '5': 13,
      '9': 0,
      '10': 'baseVersion',
      '17': true
    },
  ],
  '8': [
    {'1': '_base_version'},
  ],
};

/// Descriptor for `CheckCompatibilityRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List checkCompatibilityRequestDescriptor = $convert.base64Decode(
    'ChlDaGVja0NvbXBhdGliaWxpdHlSZXF1ZXN0EhIKBG5hbWUYASABKAlSBG5hbWUSGAoHY29udG'
    'VudBgCIAEoCVIHY29udGVudBImCgxiYXNlX3ZlcnNpb24YAyABKA1IAFILYmFzZVZlcnNpb26I'
    'AQFCDwoNX2Jhc2VfdmVyc2lvbg==');

@$core.Deprecated('Use checkCompatibilityResponseDescriptor instead')
const CheckCompatibilityResponse$json = {
  '1': 'CheckCompatibilityResponse',
  '2': [
    {'1': 'name', '3': 1, '4': 1, '5': 9, '10': 'name'},
    {'1': 'base_version', '3': 2, '4': 1, '5': 13, '10': 'baseVersion'},
    {
      '1': 'result',
      '3': 3,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.apiregistry.v1.CompatibilityResultProto',
      '10': 'result'
    },
  ],
};

/// Descriptor for `CheckCompatibilityResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List checkCompatibilityResponseDescriptor =
    $convert.base64Decode(
        'ChpDaGVja0NvbXBhdGliaWxpdHlSZXNwb25zZRISCgRuYW1lGAEgASgJUgRuYW1lEiEKDGJhc2'
        'VfdmVyc2lvbhgCIAEoDVILYmFzZVZlcnNpb24STAoGcmVzdWx0GAMgASgLMjQuazFzMC5zeXN0'
        'ZW0uYXBpcmVnaXN0cnkudjEuQ29tcGF0aWJpbGl0eVJlc3VsdFByb3RvUgZyZXN1bHQ=');

@$core.Deprecated('Use getDiffRequestDescriptor instead')
const GetDiffRequest$json = {
  '1': 'GetDiffRequest',
  '2': [
    {'1': 'name', '3': 1, '4': 1, '5': 9, '10': 'name'},
    {
      '1': 'from_version',
      '3': 2,
      '4': 1,
      '5': 13,
      '9': 0,
      '10': 'fromVersion',
      '17': true
    },
    {
      '1': 'to_version',
      '3': 3,
      '4': 1,
      '5': 13,
      '9': 1,
      '10': 'toVersion',
      '17': true
    },
  ],
  '8': [
    {'1': '_from_version'},
    {'1': '_to_version'},
  ],
};

/// Descriptor for `GetDiffRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List getDiffRequestDescriptor = $convert.base64Decode(
    'Cg5HZXREaWZmUmVxdWVzdBISCgRuYW1lGAEgASgJUgRuYW1lEiYKDGZyb21fdmVyc2lvbhgCIA'
    'EoDUgAUgtmcm9tVmVyc2lvbogBARIiCgp0b192ZXJzaW9uGAMgASgNSAFSCXRvVmVyc2lvbogB'
    'AUIPCg1fZnJvbV92ZXJzaW9uQg0KC190b192ZXJzaW9u');

@$core.Deprecated('Use getDiffResponseDescriptor instead')
const GetDiffResponse$json = {
  '1': 'GetDiffResponse',
  '2': [
    {'1': 'name', '3': 1, '4': 1, '5': 9, '10': 'name'},
    {'1': 'from_version', '3': 2, '4': 1, '5': 13, '10': 'fromVersion'},
    {'1': 'to_version', '3': 3, '4': 1, '5': 13, '10': 'toVersion'},
    {'1': 'breaking_changes', '3': 4, '4': 1, '5': 8, '10': 'breakingChanges'},
    {
      '1': 'diff',
      '3': 5,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.apiregistry.v1.SchemaDiffProto',
      '10': 'diff'
    },
  ],
};

/// Descriptor for `GetDiffResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List getDiffResponseDescriptor = $convert.base64Decode(
    'Cg9HZXREaWZmUmVzcG9uc2USEgoEbmFtZRgBIAEoCVIEbmFtZRIhCgxmcm9tX3ZlcnNpb24YAi'
    'ABKA1SC2Zyb21WZXJzaW9uEh0KCnRvX3ZlcnNpb24YAyABKA1SCXRvVmVyc2lvbhIpChBicmVh'
    'a2luZ19jaGFuZ2VzGAQgASgIUg9icmVha2luZ0NoYW5nZXMSPwoEZGlmZhgFIAEoCzIrLmsxcz'
    'Auc3lzdGVtLmFwaXJlZ2lzdHJ5LnYxLlNjaGVtYURpZmZQcm90b1IEZGlmZg==');

@$core.Deprecated('Use apiSchemaProtoDescriptor instead')
const ApiSchemaProto$json = {
  '1': 'ApiSchemaProto',
  '2': [
    {'1': 'name', '3': 1, '4': 1, '5': 9, '10': 'name'},
    {'1': 'description', '3': 2, '4': 1, '5': 9, '10': 'description'},
    {'1': 'schema_type', '3': 3, '4': 1, '5': 9, '10': 'schemaType'},
    {'1': 'latest_version', '3': 4, '4': 1, '5': 13, '10': 'latestVersion'},
    {'1': 'version_count', '3': 5, '4': 1, '5': 13, '10': 'versionCount'},
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

/// Descriptor for `ApiSchemaProto`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List apiSchemaProtoDescriptor = $convert.base64Decode(
    'Cg5BcGlTY2hlbWFQcm90bxISCgRuYW1lGAEgASgJUgRuYW1lEiAKC2Rlc2NyaXB0aW9uGAIgAS'
    'gJUgtkZXNjcmlwdGlvbhIfCgtzY2hlbWFfdHlwZRgDIAEoCVIKc2NoZW1hVHlwZRIlCg5sYXRl'
    'c3RfdmVyc2lvbhgEIAEoDVINbGF0ZXN0VmVyc2lvbhIjCg12ZXJzaW9uX2NvdW50GAUgASgNUg'
    'x2ZXJzaW9uQ291bnQSPwoKY3JlYXRlZF9hdBgGIAEoCzIgLmsxczAuc3lzdGVtLmNvbW1vbi52'
    'MS5UaW1lc3RhbXBSCWNyZWF0ZWRBdBI/Cgp1cGRhdGVkX2F0GAcgASgLMiAuazFzMC5zeXN0ZW'
    '0uY29tbW9uLnYxLlRpbWVzdGFtcFIJdXBkYXRlZEF0');

@$core.Deprecated('Use apiSchemaVersionProtoDescriptor instead')
const ApiSchemaVersionProto$json = {
  '1': 'ApiSchemaVersionProto',
  '2': [
    {'1': 'name', '3': 1, '4': 1, '5': 9, '10': 'name'},
    {'1': 'version', '3': 2, '4': 1, '5': 13, '10': 'version'},
    {'1': 'schema_type', '3': 3, '4': 1, '5': 9, '10': 'schemaType'},
    {'1': 'content', '3': 4, '4': 1, '5': 9, '10': 'content'},
    {'1': 'content_hash', '3': 5, '4': 1, '5': 9, '10': 'contentHash'},
    {'1': 'breaking_changes', '3': 6, '4': 1, '5': 8, '10': 'breakingChanges'},
    {'1': 'registered_by', '3': 7, '4': 1, '5': 9, '10': 'registeredBy'},
    {
      '1': 'created_at',
      '3': 8,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.common.v1.Timestamp',
      '10': 'createdAt'
    },
    {
      '1': 'breaking_change_details',
      '3': 9,
      '4': 3,
      '5': 11,
      '6': '.k1s0.system.apiregistry.v1.SchemaChange',
      '10': 'breakingChangeDetails'
    },
  ],
};

/// Descriptor for `ApiSchemaVersionProto`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List apiSchemaVersionProtoDescriptor = $convert.base64Decode(
    'ChVBcGlTY2hlbWFWZXJzaW9uUHJvdG8SEgoEbmFtZRgBIAEoCVIEbmFtZRIYCgd2ZXJzaW9uGA'
    'IgASgNUgd2ZXJzaW9uEh8KC3NjaGVtYV90eXBlGAMgASgJUgpzY2hlbWFUeXBlEhgKB2NvbnRl'
    'bnQYBCABKAlSB2NvbnRlbnQSIQoMY29udGVudF9oYXNoGAUgASgJUgtjb250ZW50SGFzaBIpCh'
    'BicmVha2luZ19jaGFuZ2VzGAYgASgIUg9icmVha2luZ0NoYW5nZXMSIwoNcmVnaXN0ZXJlZF9i'
    'eRgHIAEoCVIMcmVnaXN0ZXJlZEJ5Ej8KCmNyZWF0ZWRfYXQYCCABKAsyIC5rMXMwLnN5c3RlbS'
    '5jb21tb24udjEuVGltZXN0YW1wUgljcmVhdGVkQXQSYAoXYnJlYWtpbmdfY2hhbmdlX2RldGFp'
    'bHMYCSADKAsyKC5rMXMwLnN5c3RlbS5hcGlyZWdpc3RyeS52MS5TY2hlbWFDaGFuZ2VSFWJyZW'
    'FraW5nQ2hhbmdlRGV0YWlscw==');

@$core.Deprecated('Use schemaChangeDescriptor instead')
const SchemaChange$json = {
  '1': 'SchemaChange',
  '2': [
    {'1': 'change_type', '3': 1, '4': 1, '5': 9, '10': 'changeType'},
    {'1': 'path', '3': 2, '4': 1, '5': 9, '10': 'path'},
    {'1': 'description', '3': 3, '4': 1, '5': 9, '10': 'description'},
  ],
};

/// Descriptor for `SchemaChange`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List schemaChangeDescriptor = $convert.base64Decode(
    'CgxTY2hlbWFDaGFuZ2USHwoLY2hhbmdlX3R5cGUYASABKAlSCmNoYW5nZVR5cGUSEgoEcGF0aB'
    'gCIAEoCVIEcGF0aBIgCgtkZXNjcmlwdGlvbhgDIAEoCVILZGVzY3JpcHRpb24=');

@$core.Deprecated('Use compatibilityResultProtoDescriptor instead')
const CompatibilityResultProto$json = {
  '1': 'CompatibilityResultProto',
  '2': [
    {'1': 'compatible', '3': 1, '4': 1, '5': 8, '10': 'compatible'},
    {
      '1': 'breaking_changes',
      '3': 2,
      '4': 3,
      '5': 11,
      '6': '.k1s0.system.apiregistry.v1.SchemaChange',
      '10': 'breakingChanges'
    },
    {
      '1': 'non_breaking_changes',
      '3': 3,
      '4': 3,
      '5': 11,
      '6': '.k1s0.system.apiregistry.v1.SchemaChange',
      '10': 'nonBreakingChanges'
    },
  ],
};

/// Descriptor for `CompatibilityResultProto`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List compatibilityResultProtoDescriptor = $convert.base64Decode(
    'ChhDb21wYXRpYmlsaXR5UmVzdWx0UHJvdG8SHgoKY29tcGF0aWJsZRgBIAEoCFIKY29tcGF0aW'
    'JsZRJTChBicmVha2luZ19jaGFuZ2VzGAIgAygLMiguazFzMC5zeXN0ZW0uYXBpcmVnaXN0cnku'
    'djEuU2NoZW1hQ2hhbmdlUg9icmVha2luZ0NoYW5nZXMSWgoUbm9uX2JyZWFraW5nX2NoYW5nZX'
    'MYAyADKAsyKC5rMXMwLnN5c3RlbS5hcGlyZWdpc3RyeS52MS5TY2hlbWFDaGFuZ2VSEm5vbkJy'
    'ZWFraW5nQ2hhbmdlcw==');

@$core.Deprecated('Use schemaDiffProtoDescriptor instead')
const SchemaDiffProto$json = {
  '1': 'SchemaDiffProto',
  '2': [
    {
      '1': 'added',
      '3': 1,
      '4': 3,
      '5': 11,
      '6': '.k1s0.system.apiregistry.v1.DiffEntryProto',
      '10': 'added'
    },
    {
      '1': 'modified',
      '3': 2,
      '4': 3,
      '5': 11,
      '6': '.k1s0.system.apiregistry.v1.DiffModifiedEntryProto',
      '10': 'modified'
    },
    {
      '1': 'removed',
      '3': 3,
      '4': 3,
      '5': 11,
      '6': '.k1s0.system.apiregistry.v1.DiffEntryProto',
      '10': 'removed'
    },
  ],
};

/// Descriptor for `SchemaDiffProto`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List schemaDiffProtoDescriptor = $convert.base64Decode(
    'Cg9TY2hlbWFEaWZmUHJvdG8SQAoFYWRkZWQYASADKAsyKi5rMXMwLnN5c3RlbS5hcGlyZWdpc3'
    'RyeS52MS5EaWZmRW50cnlQcm90b1IFYWRkZWQSTgoIbW9kaWZpZWQYAiADKAsyMi5rMXMwLnN5'
    'c3RlbS5hcGlyZWdpc3RyeS52MS5EaWZmTW9kaWZpZWRFbnRyeVByb3RvUghtb2RpZmllZBJECg'
    'dyZW1vdmVkGAMgAygLMiouazFzMC5zeXN0ZW0uYXBpcmVnaXN0cnkudjEuRGlmZkVudHJ5UHJv'
    'dG9SB3JlbW92ZWQ=');

@$core.Deprecated('Use diffEntryProtoDescriptor instead')
const DiffEntryProto$json = {
  '1': 'DiffEntryProto',
  '2': [
    {'1': 'path', '3': 1, '4': 1, '5': 9, '10': 'path'},
    {'1': 'type', '3': 2, '4': 1, '5': 9, '10': 'type'},
    {'1': 'description', '3': 3, '4': 1, '5': 9, '10': 'description'},
  ],
};

/// Descriptor for `DiffEntryProto`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List diffEntryProtoDescriptor = $convert.base64Decode(
    'Cg5EaWZmRW50cnlQcm90bxISCgRwYXRoGAEgASgJUgRwYXRoEhIKBHR5cGUYAiABKAlSBHR5cG'
    'USIAoLZGVzY3JpcHRpb24YAyABKAlSC2Rlc2NyaXB0aW9u');

@$core.Deprecated('Use diffModifiedEntryProtoDescriptor instead')
const DiffModifiedEntryProto$json = {
  '1': 'DiffModifiedEntryProto',
  '2': [
    {'1': 'path', '3': 1, '4': 1, '5': 9, '10': 'path'},
    {'1': 'before', '3': 2, '4': 1, '5': 9, '10': 'before'},
    {'1': 'after', '3': 3, '4': 1, '5': 9, '10': 'after'},
  ],
};

/// Descriptor for `DiffModifiedEntryProto`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List diffModifiedEntryProtoDescriptor =
    $convert.base64Decode(
        'ChZEaWZmTW9kaWZpZWRFbnRyeVByb3RvEhIKBHBhdGgYASABKAlSBHBhdGgSFgoGYmVmb3JlGA'
        'IgASgJUgZiZWZvcmUSFAoFYWZ0ZXIYAyABKAlSBWFmdGVy');
