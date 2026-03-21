// This is a generated file - do not edit.
//
// Generated from k1s0/system/navigation/v1/navigation.proto.

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

@$core.Deprecated('Use guardTypeDescriptor instead')
const GuardType$json = {
  '1': 'GuardType',
  '2': [
    {'1': 'GUARD_TYPE_UNSPECIFIED', '2': 0},
    {'1': 'GUARD_TYPE_AUTH_REQUIRED', '2': 1},
    {'1': 'GUARD_TYPE_ROLE_REQUIRED', '2': 2},
    {'1': 'GUARD_TYPE_REDIRECT_IF_AUTHENTICATED', '2': 3},
  ],
};

/// Descriptor for `GuardType`. Decode as a `google.protobuf.EnumDescriptorProto`.
final $typed_data.Uint8List guardTypeDescriptor = $convert.base64Decode(
    'CglHdWFyZFR5cGUSGgoWR1VBUkRfVFlQRV9VTlNQRUNJRklFRBAAEhwKGEdVQVJEX1RZUEVfQV'
    'VUSF9SRVFVSVJFRBABEhwKGEdVQVJEX1RZUEVfUk9MRV9SRVFVSVJFRBACEigKJEdVQVJEX1RZ'
    'UEVfUkVESVJFQ1RfSUZfQVVUSEVOVElDQVRFRBAD');

@$core.Deprecated('Use transitionTypeDescriptor instead')
const TransitionType$json = {
  '1': 'TransitionType',
  '2': [
    {'1': 'TRANSITION_TYPE_UNSPECIFIED', '2': 0},
    {'1': 'TRANSITION_TYPE_FADE', '2': 1},
    {'1': 'TRANSITION_TYPE_SLIDE', '2': 2},
    {'1': 'TRANSITION_TYPE_MODAL', '2': 3},
  ],
};

/// Descriptor for `TransitionType`. Decode as a `google.protobuf.EnumDescriptorProto`.
final $typed_data.Uint8List transitionTypeDescriptor = $convert.base64Decode(
    'Cg5UcmFuc2l0aW9uVHlwZRIfChtUUkFOU0lUSU9OX1RZUEVfVU5TUEVDSUZJRUQQABIYChRUUk'
    'FOU0lUSU9OX1RZUEVfRkFERRABEhkKFVRSQU5TSVRJT05fVFlQRV9TTElERRACEhkKFVRSQU5T'
    'SVRJT05fVFlQRV9NT0RBTBAD');

@$core.Deprecated('Use paramTypeDescriptor instead')
const ParamType$json = {
  '1': 'ParamType',
  '2': [
    {'1': 'PARAM_TYPE_UNSPECIFIED', '2': 0},
    {'1': 'PARAM_TYPE_STRING', '2': 1},
    {'1': 'PARAM_TYPE_INT', '2': 2},
    {'1': 'PARAM_TYPE_UUID', '2': 3},
  ],
};

/// Descriptor for `ParamType`. Decode as a `google.protobuf.EnumDescriptorProto`.
final $typed_data.Uint8List paramTypeDescriptor = $convert.base64Decode(
    'CglQYXJhbVR5cGUSGgoWUEFSQU1fVFlQRV9VTlNQRUNJRklFRBAAEhUKEVBBUkFNX1RZUEVfU1'
    'RSSU5HEAESEgoOUEFSQU1fVFlQRV9JTlQQAhITCg9QQVJBTV9UWVBFX1VVSUQQAw==');

@$core.Deprecated('Use getNavigationRequestDescriptor instead')
const GetNavigationRequest$json = {
  '1': 'GetNavigationRequest',
  '2': [
    {'1': 'bearer_token', '3': 1, '4': 1, '5': 9, '10': 'bearerToken'},
  ],
};

/// Descriptor for `GetNavigationRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List getNavigationRequestDescriptor = $convert.base64Decode(
    'ChRHZXROYXZpZ2F0aW9uUmVxdWVzdBIhCgxiZWFyZXJfdG9rZW4YASABKAlSC2JlYXJlclRva2'
    'Vu');

@$core.Deprecated('Use getNavigationResponseDescriptor instead')
const GetNavigationResponse$json = {
  '1': 'GetNavigationResponse',
  '2': [
    {
      '1': 'routes',
      '3': 1,
      '4': 3,
      '5': 11,
      '6': '.k1s0.system.navigation.v1.Route',
      '10': 'routes'
    },
    {
      '1': 'guards',
      '3': 2,
      '4': 3,
      '5': 11,
      '6': '.k1s0.system.navigation.v1.Guard',
      '10': 'guards'
    },
  ],
};

/// Descriptor for `GetNavigationResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List getNavigationResponseDescriptor = $convert.base64Decode(
    'ChVHZXROYXZpZ2F0aW9uUmVzcG9uc2USOAoGcm91dGVzGAEgAygLMiAuazFzMC5zeXN0ZW0ubm'
    'F2aWdhdGlvbi52MS5Sb3V0ZVIGcm91dGVzEjgKBmd1YXJkcxgCIAMoCzIgLmsxczAuc3lzdGVt'
    'Lm5hdmlnYXRpb24udjEuR3VhcmRSBmd1YXJkcw==');

@$core.Deprecated('Use routeDescriptor instead')
const Route$json = {
  '1': 'Route',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 9, '10': 'id'},
    {'1': 'path', '3': 2, '4': 1, '5': 9, '10': 'path'},
    {
      '1': 'component_id',
      '3': 3,
      '4': 1,
      '5': 9,
      '9': 0,
      '10': 'componentId',
      '17': true
    },
    {'1': 'guard_ids', '3': 4, '4': 3, '5': 9, '10': 'guardIds'},
    {
      '1': 'children',
      '3': 5,
      '4': 3,
      '5': 11,
      '6': '.k1s0.system.navigation.v1.Route',
      '10': 'children'
    },
    {
      '1': 'transition',
      '3': 6,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.navigation.v1.TransitionConfig',
      '10': 'transition'
    },
    {
      '1': 'params',
      '3': 7,
      '4': 3,
      '5': 11,
      '6': '.k1s0.system.navigation.v1.Param',
      '10': 'params'
    },
    {'1': 'redirect_to', '3': 8, '4': 1, '5': 9, '10': 'redirectTo'},
  ],
  '8': [
    {'1': '_component_id'},
  ],
};

/// Descriptor for `Route`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List routeDescriptor = $convert.base64Decode(
    'CgVSb3V0ZRIOCgJpZBgBIAEoCVICaWQSEgoEcGF0aBgCIAEoCVIEcGF0aBImCgxjb21wb25lbn'
    'RfaWQYAyABKAlIAFILY29tcG9uZW50SWSIAQESGwoJZ3VhcmRfaWRzGAQgAygJUghndWFyZElk'
    'cxI8CghjaGlsZHJlbhgFIAMoCzIgLmsxczAuc3lzdGVtLm5hdmlnYXRpb24udjEuUm91dGVSCG'
    'NoaWxkcmVuEksKCnRyYW5zaXRpb24YBiABKAsyKy5rMXMwLnN5c3RlbS5uYXZpZ2F0aW9uLnYx'
    'LlRyYW5zaXRpb25Db25maWdSCnRyYW5zaXRpb24SOAoGcGFyYW1zGAcgAygLMiAuazFzMC5zeX'
    'N0ZW0ubmF2aWdhdGlvbi52MS5QYXJhbVIGcGFyYW1zEh8KC3JlZGlyZWN0X3RvGAggASgJUgpy'
    'ZWRpcmVjdFRvQg8KDV9jb21wb25lbnRfaWQ=');

@$core.Deprecated('Use guardDescriptor instead')
const Guard$json = {
  '1': 'Guard',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 9, '10': 'id'},
    {
      '1': 'type',
      '3': 2,
      '4': 1,
      '5': 14,
      '6': '.k1s0.system.navigation.v1.GuardType',
      '10': 'type'
    },
    {'1': 'redirect_to', '3': 3, '4': 1, '5': 9, '10': 'redirectTo'},
    {'1': 'roles', '3': 4, '4': 3, '5': 9, '10': 'roles'},
  ],
};

/// Descriptor for `Guard`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List guardDescriptor = $convert.base64Decode(
    'CgVHdWFyZBIOCgJpZBgBIAEoCVICaWQSOAoEdHlwZRgCIAEoDjIkLmsxczAuc3lzdGVtLm5hdm'
    'lnYXRpb24udjEuR3VhcmRUeXBlUgR0eXBlEh8KC3JlZGlyZWN0X3RvGAMgASgJUgpyZWRpcmVj'
    'dFRvEhQKBXJvbGVzGAQgAygJUgVyb2xlcw==');

@$core.Deprecated('Use paramDescriptor instead')
const Param$json = {
  '1': 'Param',
  '2': [
    {'1': 'name', '3': 1, '4': 1, '5': 9, '10': 'name'},
    {
      '1': 'type',
      '3': 2,
      '4': 1,
      '5': 14,
      '6': '.k1s0.system.navigation.v1.ParamType',
      '10': 'type'
    },
  ],
};

/// Descriptor for `Param`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List paramDescriptor = $convert.base64Decode(
    'CgVQYXJhbRISCgRuYW1lGAEgASgJUgRuYW1lEjgKBHR5cGUYAiABKA4yJC5rMXMwLnN5c3RlbS'
    '5uYXZpZ2F0aW9uLnYxLlBhcmFtVHlwZVIEdHlwZQ==');

@$core.Deprecated('Use transitionConfigDescriptor instead')
const TransitionConfig$json = {
  '1': 'TransitionConfig',
  '2': [
    {
      '1': 'type',
      '3': 1,
      '4': 1,
      '5': 14,
      '6': '.k1s0.system.navigation.v1.TransitionType',
      '10': 'type'
    },
    {'1': 'duration_ms', '3': 2, '4': 1, '5': 13, '10': 'durationMs'},
  ],
};

/// Descriptor for `TransitionConfig`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List transitionConfigDescriptor = $convert.base64Decode(
    'ChBUcmFuc2l0aW9uQ29uZmlnEj0KBHR5cGUYASABKA4yKS5rMXMwLnN5c3RlbS5uYXZpZ2F0aW'
    '9uLnYxLlRyYW5zaXRpb25UeXBlUgR0eXBlEh8KC2R1cmF0aW9uX21zGAIgASgNUgpkdXJhdGlv'
    'bk1z');
