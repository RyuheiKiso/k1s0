// This is a generated file - do not edit.
//
// Generated from k1s0/system/common/v1/types.proto.

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

@$core.Deprecated('Use changeTypeDescriptor instead')
const ChangeType$json = {
  '1': 'ChangeType',
  '2': [
    {'1': 'CHANGE_TYPE_UNSPECIFIED', '2': 0},
    {'1': 'CHANGE_TYPE_CREATED', '2': 1},
    {'1': 'CHANGE_TYPE_UPDATED', '2': 2},
    {'1': 'CHANGE_TYPE_DELETED', '2': 3},
  ],
};

/// Descriptor for `ChangeType`. Decode as a `google.protobuf.EnumDescriptorProto`.
final $typed_data.Uint8List changeTypeDescriptor = $convert.base64Decode(
    'CgpDaGFuZ2VUeXBlEhsKF0NIQU5HRV9UWVBFX1VOU1BFQ0lGSUVEEAASFwoTQ0hBTkdFX1RZUE'
    'VfQ1JFQVRFRBABEhcKE0NIQU5HRV9UWVBFX1VQREFURUQQAhIXChNDSEFOR0VfVFlQRV9ERUxF'
    'VEVEEAM=');

@$core.Deprecated('Use paginationDescriptor instead')
const Pagination$json = {
  '1': 'Pagination',
  '2': [
    {'1': 'page', '3': 1, '4': 1, '5': 5, '10': 'page'},
    {'1': 'page_size', '3': 2, '4': 1, '5': 5, '10': 'pageSize'},
  ],
};

/// Descriptor for `Pagination`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List paginationDescriptor = $convert.base64Decode(
    'CgpQYWdpbmF0aW9uEhIKBHBhZ2UYASABKAVSBHBhZ2USGwoJcGFnZV9zaXplGAIgASgFUghwYW'
    'dlU2l6ZQ==');

@$core.Deprecated('Use paginationResultDescriptor instead')
const PaginationResult$json = {
  '1': 'PaginationResult',
  '2': [
    {'1': 'total_count', '3': 1, '4': 1, '5': 3, '10': 'totalCount'},
    {'1': 'page', '3': 2, '4': 1, '5': 5, '10': 'page'},
    {'1': 'page_size', '3': 3, '4': 1, '5': 5, '10': 'pageSize'},
    {'1': 'has_next', '3': 4, '4': 1, '5': 8, '10': 'hasNext'},
  ],
};

/// Descriptor for `PaginationResult`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List paginationResultDescriptor = $convert.base64Decode(
    'ChBQYWdpbmF0aW9uUmVzdWx0Eh8KC3RvdGFsX2NvdW50GAEgASgDUgp0b3RhbENvdW50EhIKBH'
    'BhZ2UYAiABKAVSBHBhZ2USGwoJcGFnZV9zaXplGAMgASgFUghwYWdlU2l6ZRIZCghoYXNfbmV4'
    'dBgEIAEoCFIHaGFzTmV4dA==');

@$core.Deprecated('Use timestampDescriptor instead')
const Timestamp$json = {
  '1': 'Timestamp',
  '2': [
    {'1': 'seconds', '3': 1, '4': 1, '5': 3, '10': 'seconds'},
    {'1': 'nanos', '3': 2, '4': 1, '5': 5, '10': 'nanos'},
  ],
};

/// Descriptor for `Timestamp`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List timestampDescriptor = $convert.base64Decode(
    'CglUaW1lc3RhbXASGAoHc2Vjb25kcxgBIAEoA1IHc2Vjb25kcxIUCgVuYW5vcxgCIAEoBVIFbm'
    'Fub3M=');
