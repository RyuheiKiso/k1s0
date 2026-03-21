// This is a generated file - do not edit.
//
// Generated from k1s0/system/servicecatalog/v1/service_catalog.proto.

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

@$core.Deprecated('Use serviceInfoDescriptor instead')
const ServiceInfo$json = {
  '1': 'ServiceInfo',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 9, '10': 'id'},
    {'1': 'name', '3': 2, '4': 1, '5': 9, '10': 'name'},
    {'1': 'display_name', '3': 3, '4': 1, '5': 9, '10': 'displayName'},
    {'1': 'description', '3': 4, '4': 1, '5': 9, '10': 'description'},
    {'1': 'tier', '3': 5, '4': 1, '5': 9, '10': 'tier'},
    {'1': 'version', '3': 6, '4': 1, '5': 9, '10': 'version'},
    {'1': 'base_url', '3': 7, '4': 1, '5': 9, '10': 'baseUrl'},
    {
      '1': 'grpc_endpoint',
      '3': 8,
      '4': 1,
      '5': 9,
      '9': 0,
      '10': 'grpcEndpoint',
      '17': true
    },
    {'1': 'health_url', '3': 9, '4': 1, '5': 9, '10': 'healthUrl'},
    {'1': 'status', '3': 10, '4': 1, '5': 9, '10': 'status'},
    {
      '1': 'metadata',
      '3': 11,
      '4': 3,
      '5': 11,
      '6': '.k1s0.system.servicecatalog.v1.ServiceInfo.MetadataEntry',
      '10': 'metadata'
    },
    {
      '1': 'created_at',
      '3': 12,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.common.v1.Timestamp',
      '10': 'createdAt'
    },
    {
      '1': 'updated_at',
      '3': 13,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.common.v1.Timestamp',
      '10': 'updatedAt'
    },
  ],
  '3': [ServiceInfo_MetadataEntry$json],
  '8': [
    {'1': '_grpc_endpoint'},
  ],
};

@$core.Deprecated('Use serviceInfoDescriptor instead')
const ServiceInfo_MetadataEntry$json = {
  '1': 'MetadataEntry',
  '2': [
    {'1': 'key', '3': 1, '4': 1, '5': 9, '10': 'key'},
    {'1': 'value', '3': 2, '4': 1, '5': 9, '10': 'value'},
  ],
  '7': {'7': true},
};

/// Descriptor for `ServiceInfo`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List serviceInfoDescriptor = $convert.base64Decode(
    'CgtTZXJ2aWNlSW5mbxIOCgJpZBgBIAEoCVICaWQSEgoEbmFtZRgCIAEoCVIEbmFtZRIhCgxkaX'
    'NwbGF5X25hbWUYAyABKAlSC2Rpc3BsYXlOYW1lEiAKC2Rlc2NyaXB0aW9uGAQgASgJUgtkZXNj'
    'cmlwdGlvbhISCgR0aWVyGAUgASgJUgR0aWVyEhgKB3ZlcnNpb24YBiABKAlSB3ZlcnNpb24SGQ'
    'oIYmFzZV91cmwYByABKAlSB2Jhc2VVcmwSKAoNZ3JwY19lbmRwb2ludBgIIAEoCUgAUgxncnBj'
    'RW5kcG9pbnSIAQESHQoKaGVhbHRoX3VybBgJIAEoCVIJaGVhbHRoVXJsEhYKBnN0YXR1cxgKIA'
    'EoCVIGc3RhdHVzElQKCG1ldGFkYXRhGAsgAygLMjguazFzMC5zeXN0ZW0uc2VydmljZWNhdGFs'
    'b2cudjEuU2VydmljZUluZm8uTWV0YWRhdGFFbnRyeVIIbWV0YWRhdGESPwoKY3JlYXRlZF9hdB'
    'gMIAEoCzIgLmsxczAuc3lzdGVtLmNvbW1vbi52MS5UaW1lc3RhbXBSCWNyZWF0ZWRBdBI/Cgp1'
    'cGRhdGVkX2F0GA0gASgLMiAuazFzMC5zeXN0ZW0uY29tbW9uLnYxLlRpbWVzdGFtcFIJdXBkYX'
    'RlZEF0GjsKDU1ldGFkYXRhRW50cnkSEAoDa2V5GAEgASgJUgNrZXkSFAoFdmFsdWUYAiABKAlS'
    'BXZhbHVlOgI4AUIQCg5fZ3JwY19lbmRwb2ludA==');

@$core.Deprecated('Use registerServiceRequestDescriptor instead')
const RegisterServiceRequest$json = {
  '1': 'RegisterServiceRequest',
  '2': [
    {'1': 'name', '3': 1, '4': 1, '5': 9, '10': 'name'},
    {'1': 'display_name', '3': 2, '4': 1, '5': 9, '10': 'displayName'},
    {'1': 'description', '3': 3, '4': 1, '5': 9, '10': 'description'},
    {'1': 'tier', '3': 4, '4': 1, '5': 9, '10': 'tier'},
    {'1': 'version', '3': 5, '4': 1, '5': 9, '10': 'version'},
    {'1': 'base_url', '3': 6, '4': 1, '5': 9, '10': 'baseUrl'},
    {
      '1': 'grpc_endpoint',
      '3': 7,
      '4': 1,
      '5': 9,
      '9': 0,
      '10': 'grpcEndpoint',
      '17': true
    },
    {'1': 'health_url', '3': 8, '4': 1, '5': 9, '10': 'healthUrl'},
    {
      '1': 'metadata',
      '3': 9,
      '4': 3,
      '5': 11,
      '6':
          '.k1s0.system.servicecatalog.v1.RegisterServiceRequest.MetadataEntry',
      '10': 'metadata'
    },
  ],
  '3': [RegisterServiceRequest_MetadataEntry$json],
  '8': [
    {'1': '_grpc_endpoint'},
  ],
};

@$core.Deprecated('Use registerServiceRequestDescriptor instead')
const RegisterServiceRequest_MetadataEntry$json = {
  '1': 'MetadataEntry',
  '2': [
    {'1': 'key', '3': 1, '4': 1, '5': 9, '10': 'key'},
    {'1': 'value', '3': 2, '4': 1, '5': 9, '10': 'value'},
  ],
  '7': {'7': true},
};

/// Descriptor for `RegisterServiceRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List registerServiceRequestDescriptor = $convert.base64Decode(
    'ChZSZWdpc3RlclNlcnZpY2VSZXF1ZXN0EhIKBG5hbWUYASABKAlSBG5hbWUSIQoMZGlzcGxheV'
    '9uYW1lGAIgASgJUgtkaXNwbGF5TmFtZRIgCgtkZXNjcmlwdGlvbhgDIAEoCVILZGVzY3JpcHRp'
    'b24SEgoEdGllchgEIAEoCVIEdGllchIYCgd2ZXJzaW9uGAUgASgJUgd2ZXJzaW9uEhkKCGJhc2'
    'VfdXJsGAYgASgJUgdiYXNlVXJsEigKDWdycGNfZW5kcG9pbnQYByABKAlIAFIMZ3JwY0VuZHBv'
    'aW50iAEBEh0KCmhlYWx0aF91cmwYCCABKAlSCWhlYWx0aFVybBJfCghtZXRhZGF0YRgJIAMoCz'
    'JDLmsxczAuc3lzdGVtLnNlcnZpY2VjYXRhbG9nLnYxLlJlZ2lzdGVyU2VydmljZVJlcXVlc3Qu'
    'TWV0YWRhdGFFbnRyeVIIbWV0YWRhdGEaOwoNTWV0YWRhdGFFbnRyeRIQCgNrZXkYASABKAlSA2'
    'tleRIUCgV2YWx1ZRgCIAEoCVIFdmFsdWU6AjgBQhAKDl9ncnBjX2VuZHBvaW50');

@$core.Deprecated('Use registerServiceResponseDescriptor instead')
const RegisterServiceResponse$json = {
  '1': 'RegisterServiceResponse',
  '2': [
    {
      '1': 'service',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.servicecatalog.v1.ServiceInfo',
      '10': 'service'
    },
  ],
};

/// Descriptor for `RegisterServiceResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List registerServiceResponseDescriptor =
    $convert.base64Decode(
        'ChdSZWdpc3RlclNlcnZpY2VSZXNwb25zZRJECgdzZXJ2aWNlGAEgASgLMiouazFzMC5zeXN0ZW'
        '0uc2VydmljZWNhdGFsb2cudjEuU2VydmljZUluZm9SB3NlcnZpY2U=');

@$core.Deprecated('Use getServiceRequestDescriptor instead')
const GetServiceRequest$json = {
  '1': 'GetServiceRequest',
  '2': [
    {'1': 'service_id', '3': 1, '4': 1, '5': 9, '10': 'serviceId'},
  ],
};

/// Descriptor for `GetServiceRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List getServiceRequestDescriptor = $convert.base64Decode(
    'ChFHZXRTZXJ2aWNlUmVxdWVzdBIdCgpzZXJ2aWNlX2lkGAEgASgJUglzZXJ2aWNlSWQ=');

@$core.Deprecated('Use getServiceResponseDescriptor instead')
const GetServiceResponse$json = {
  '1': 'GetServiceResponse',
  '2': [
    {
      '1': 'service',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.servicecatalog.v1.ServiceInfo',
      '10': 'service'
    },
  ],
};

/// Descriptor for `GetServiceResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List getServiceResponseDescriptor = $convert.base64Decode(
    'ChJHZXRTZXJ2aWNlUmVzcG9uc2USRAoHc2VydmljZRgBIAEoCzIqLmsxczAuc3lzdGVtLnNlcn'
    'ZpY2VjYXRhbG9nLnYxLlNlcnZpY2VJbmZvUgdzZXJ2aWNl');

@$core.Deprecated('Use listServicesRequestDescriptor instead')
const ListServicesRequest$json = {
  '1': 'ListServicesRequest',
  '2': [
    {
      '1': 'pagination',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.common.v1.Pagination',
      '10': 'pagination'
    },
    {'1': 'tier', '3': 2, '4': 1, '5': 9, '9': 0, '10': 'tier', '17': true},
    {'1': 'status', '3': 3, '4': 1, '5': 9, '9': 1, '10': 'status', '17': true},
    {'1': 'search', '3': 4, '4': 1, '5': 9, '9': 2, '10': 'search', '17': true},
  ],
  '8': [
    {'1': '_tier'},
    {'1': '_status'},
    {'1': '_search'},
  ],
};

/// Descriptor for `ListServicesRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List listServicesRequestDescriptor = $convert.base64Decode(
    'ChNMaXN0U2VydmljZXNSZXF1ZXN0EkEKCnBhZ2luYXRpb24YASABKAsyIS5rMXMwLnN5c3RlbS'
    '5jb21tb24udjEuUGFnaW5hdGlvblIKcGFnaW5hdGlvbhIXCgR0aWVyGAIgASgJSABSBHRpZXKI'
    'AQESGwoGc3RhdHVzGAMgASgJSAFSBnN0YXR1c4gBARIbCgZzZWFyY2gYBCABKAlIAlIGc2Vhcm'
    'NoiAEBQgcKBV90aWVyQgkKB19zdGF0dXNCCQoHX3NlYXJjaA==');

@$core.Deprecated('Use listServicesResponseDescriptor instead')
const ListServicesResponse$json = {
  '1': 'ListServicesResponse',
  '2': [
    {
      '1': 'services',
      '3': 1,
      '4': 3,
      '5': 11,
      '6': '.k1s0.system.servicecatalog.v1.ServiceInfo',
      '10': 'services'
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

/// Descriptor for `ListServicesResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List listServicesResponseDescriptor = $convert.base64Decode(
    'ChRMaXN0U2VydmljZXNSZXNwb25zZRJGCghzZXJ2aWNlcxgBIAMoCzIqLmsxczAuc3lzdGVtLn'
    'NlcnZpY2VjYXRhbG9nLnYxLlNlcnZpY2VJbmZvUghzZXJ2aWNlcxJHCgpwYWdpbmF0aW9uGAIg'
    'ASgLMicuazFzMC5zeXN0ZW0uY29tbW9uLnYxLlBhZ2luYXRpb25SZXN1bHRSCnBhZ2luYXRpb2'
    '4=');

@$core.Deprecated('Use updateServiceRequestDescriptor instead')
const UpdateServiceRequest$json = {
  '1': 'UpdateServiceRequest',
  '2': [
    {'1': 'service_id', '3': 1, '4': 1, '5': 9, '10': 'serviceId'},
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
      '1': 'version',
      '3': 4,
      '4': 1,
      '5': 9,
      '9': 2,
      '10': 'version',
      '17': true
    },
    {
      '1': 'base_url',
      '3': 5,
      '4': 1,
      '5': 9,
      '9': 3,
      '10': 'baseUrl',
      '17': true
    },
    {
      '1': 'grpc_endpoint',
      '3': 6,
      '4': 1,
      '5': 9,
      '9': 4,
      '10': 'grpcEndpoint',
      '17': true
    },
    {
      '1': 'health_url',
      '3': 7,
      '4': 1,
      '5': 9,
      '9': 5,
      '10': 'healthUrl',
      '17': true
    },
    {
      '1': 'metadata',
      '3': 8,
      '4': 3,
      '5': 11,
      '6': '.k1s0.system.servicecatalog.v1.UpdateServiceRequest.MetadataEntry',
      '10': 'metadata'
    },
  ],
  '3': [UpdateServiceRequest_MetadataEntry$json],
  '8': [
    {'1': '_display_name'},
    {'1': '_description'},
    {'1': '_version'},
    {'1': '_base_url'},
    {'1': '_grpc_endpoint'},
    {'1': '_health_url'},
  ],
};

@$core.Deprecated('Use updateServiceRequestDescriptor instead')
const UpdateServiceRequest_MetadataEntry$json = {
  '1': 'MetadataEntry',
  '2': [
    {'1': 'key', '3': 1, '4': 1, '5': 9, '10': 'key'},
    {'1': 'value', '3': 2, '4': 1, '5': 9, '10': 'value'},
  ],
  '7': {'7': true},
};

/// Descriptor for `UpdateServiceRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List updateServiceRequestDescriptor = $convert.base64Decode(
    'ChRVcGRhdGVTZXJ2aWNlUmVxdWVzdBIdCgpzZXJ2aWNlX2lkGAEgASgJUglzZXJ2aWNlSWQSJg'
    'oMZGlzcGxheV9uYW1lGAIgASgJSABSC2Rpc3BsYXlOYW1liAEBEiUKC2Rlc2NyaXB0aW9uGAMg'
    'ASgJSAFSC2Rlc2NyaXB0aW9uiAEBEh0KB3ZlcnNpb24YBCABKAlIAlIHdmVyc2lvbogBARIeCg'
    'hiYXNlX3VybBgFIAEoCUgDUgdiYXNlVXJsiAEBEigKDWdycGNfZW5kcG9pbnQYBiABKAlIBFIM'
    'Z3JwY0VuZHBvaW50iAEBEiIKCmhlYWx0aF91cmwYByABKAlIBVIJaGVhbHRoVXJsiAEBEl0KCG'
    '1ldGFkYXRhGAggAygLMkEuazFzMC5zeXN0ZW0uc2VydmljZWNhdGFsb2cudjEuVXBkYXRlU2Vy'
    'dmljZVJlcXVlc3QuTWV0YWRhdGFFbnRyeVIIbWV0YWRhdGEaOwoNTWV0YWRhdGFFbnRyeRIQCg'
    'NrZXkYASABKAlSA2tleRIUCgV2YWx1ZRgCIAEoCVIFdmFsdWU6AjgBQg8KDV9kaXNwbGF5X25h'
    'bWVCDgoMX2Rlc2NyaXB0aW9uQgoKCF92ZXJzaW9uQgsKCV9iYXNlX3VybEIQCg5fZ3JwY19lbm'
    'Rwb2ludEINCgtfaGVhbHRoX3VybA==');

@$core.Deprecated('Use updateServiceResponseDescriptor instead')
const UpdateServiceResponse$json = {
  '1': 'UpdateServiceResponse',
  '2': [
    {
      '1': 'service',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.servicecatalog.v1.ServiceInfo',
      '10': 'service'
    },
  ],
};

/// Descriptor for `UpdateServiceResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List updateServiceResponseDescriptor = $convert.base64Decode(
    'ChVVcGRhdGVTZXJ2aWNlUmVzcG9uc2USRAoHc2VydmljZRgBIAEoCzIqLmsxczAuc3lzdGVtLn'
    'NlcnZpY2VjYXRhbG9nLnYxLlNlcnZpY2VJbmZvUgdzZXJ2aWNl');

@$core.Deprecated('Use deleteServiceRequestDescriptor instead')
const DeleteServiceRequest$json = {
  '1': 'DeleteServiceRequest',
  '2': [
    {'1': 'service_id', '3': 1, '4': 1, '5': 9, '10': 'serviceId'},
  ],
};

/// Descriptor for `DeleteServiceRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List deleteServiceRequestDescriptor = $convert.base64Decode(
    'ChREZWxldGVTZXJ2aWNlUmVxdWVzdBIdCgpzZXJ2aWNlX2lkGAEgASgJUglzZXJ2aWNlSWQ=');

@$core.Deprecated('Use deleteServiceResponseDescriptor instead')
const DeleteServiceResponse$json = {
  '1': 'DeleteServiceResponse',
  '2': [
    {'1': 'success', '3': 1, '4': 1, '5': 8, '10': 'success'},
  ],
};

/// Descriptor for `DeleteServiceResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List deleteServiceResponseDescriptor =
    $convert.base64Decode(
        'ChVEZWxldGVTZXJ2aWNlUmVzcG9uc2USGAoHc3VjY2VzcxgBIAEoCFIHc3VjY2Vzcw==');

@$core.Deprecated('Use healthCheckRequestDescriptor instead')
const HealthCheckRequest$json = {
  '1': 'HealthCheckRequest',
  '2': [
    {
      '1': 'service_id',
      '3': 1,
      '4': 1,
      '5': 9,
      '9': 0,
      '10': 'serviceId',
      '17': true
    },
  ],
  '8': [
    {'1': '_service_id'},
  ],
};

/// Descriptor for `HealthCheckRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List healthCheckRequestDescriptor = $convert.base64Decode(
    'ChJIZWFsdGhDaGVja1JlcXVlc3QSIgoKc2VydmljZV9pZBgBIAEoCUgAUglzZXJ2aWNlSWSIAQ'
    'FCDQoLX3NlcnZpY2VfaWQ=');

@$core.Deprecated('Use healthCheckResponseDescriptor instead')
const HealthCheckResponse$json = {
  '1': 'HealthCheckResponse',
  '2': [
    {
      '1': 'services',
      '3': 1,
      '4': 3,
      '5': 11,
      '6': '.k1s0.system.servicecatalog.v1.ServiceHealth',
      '10': 'services'
    },
  ],
};

/// Descriptor for `HealthCheckResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List healthCheckResponseDescriptor = $convert.base64Decode(
    'ChNIZWFsdGhDaGVja1Jlc3BvbnNlEkgKCHNlcnZpY2VzGAEgAygLMiwuazFzMC5zeXN0ZW0uc2'
    'VydmljZWNhdGFsb2cudjEuU2VydmljZUhlYWx0aFIIc2VydmljZXM=');

@$core.Deprecated('Use serviceHealthDescriptor instead')
const ServiceHealth$json = {
  '1': 'ServiceHealth',
  '2': [
    {'1': 'service_id', '3': 1, '4': 1, '5': 9, '10': 'serviceId'},
    {'1': 'service_name', '3': 2, '4': 1, '5': 9, '10': 'serviceName'},
    {'1': 'status', '3': 3, '4': 1, '5': 9, '10': 'status'},
    {
      '1': 'response_time_ms',
      '3': 4,
      '4': 1,
      '5': 3,
      '9': 0,
      '10': 'responseTimeMs',
      '17': true
    },
    {
      '1': 'error_message',
      '3': 5,
      '4': 1,
      '5': 9,
      '9': 1,
      '10': 'errorMessage',
      '17': true
    },
    {
      '1': 'checked_at',
      '3': 6,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.common.v1.Timestamp',
      '10': 'checkedAt'
    },
  ],
  '8': [
    {'1': '_response_time_ms'},
    {'1': '_error_message'},
  ],
};

/// Descriptor for `ServiceHealth`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List serviceHealthDescriptor = $convert.base64Decode(
    'Cg1TZXJ2aWNlSGVhbHRoEh0KCnNlcnZpY2VfaWQYASABKAlSCXNlcnZpY2VJZBIhCgxzZXJ2aW'
    'NlX25hbWUYAiABKAlSC3NlcnZpY2VOYW1lEhYKBnN0YXR1cxgDIAEoCVIGc3RhdHVzEi0KEHJl'
    'c3BvbnNlX3RpbWVfbXMYBCABKANIAFIOcmVzcG9uc2VUaW1lTXOIAQESKAoNZXJyb3JfbWVzc2'
    'FnZRgFIAEoCUgBUgxlcnJvck1lc3NhZ2WIAQESPwoKY2hlY2tlZF9hdBgGIAEoCzIgLmsxczAu'
    'c3lzdGVtLmNvbW1vbi52MS5UaW1lc3RhbXBSCWNoZWNrZWRBdEITChFfcmVzcG9uc2VfdGltZV'
    '9tc0IQCg5fZXJyb3JfbWVzc2FnZQ==');
