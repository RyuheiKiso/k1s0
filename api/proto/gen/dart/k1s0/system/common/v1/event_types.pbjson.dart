// This is a generated file - do not edit.
//
// Generated from k1s0/system/common/v1/event_types.proto.

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

@$core.Deprecated('Use eventTypeDescriptor instead')
const EventType$json = {
  '1': 'EventType',
  '2': [
    {'1': 'EVENT_TYPE_UNSPECIFIED', '2': 0},
    {'1': 'EVENT_TYPE_AUTH_LOGIN', '2': 100},
    {'1': 'EVENT_TYPE_AUTH_TOKEN_VALIDATION', '2': 101},
    {'1': 'EVENT_TYPE_AUTH_PERMISSION_CHECK', '2': 102},
    {'1': 'EVENT_TYPE_AUTH_AUDIT_LOG_RECORDED', '2': 103},
    {'1': 'EVENT_TYPE_CONFIG_CHANGED', '2': 200},
    {'1': 'EVENT_TYPE_TASKMANAGEMENT_PROJECT_TYPE_CHANGED', '2': 300},
    {'1': 'EVENT_TYPE_TASKMANAGEMENT_STATUS_DEFINITION_CHANGED', '2': 301},
    {'1': 'EVENT_TYPE_TASK_CREATED', '2': 400},
    {'1': 'EVENT_TYPE_TASK_STATUS_CHANGED', '2': 401},
    {'1': 'EVENT_TYPE_TASK_CANCELLED', '2': 402},
    {'1': 'EVENT_TYPE_BOARD_COLUMN_INCREMENTED', '2': 500},
    {'1': 'EVENT_TYPE_BOARD_COLUMN_DECREMENTED', '2': 501},
    {'1': 'EVENT_TYPE_ACTIVITY_CREATED', '2': 600},
    {'1': 'EVENT_TYPE_ACTIVITY_APPROVED', '2': 601},
    {'1': 'EVENT_TYPE_ACTIVITY_REJECTED', '2': 602},
    {'1': 'EVENT_TYPE_ACTIVITY_DELETED', '2': 603},
  ],
};

/// Descriptor for `EventType`. Decode as a `google.protobuf.EnumDescriptorProto`.
final $typed_data.Uint8List eventTypeDescriptor = $convert.base64Decode(
    'CglFdmVudFR5cGUSGgoWRVZFTlRfVFlQRV9VTlNQRUNJRklFRBAAEhkKFUVWRU5UX1RZUEVfQVVU'
    'SF9MT0dJThBkEiQKIEVWRU5UX1RZUEVfQVVUSF9UT0tFTl9WQUxJREFUSU9OEGUSJAogRVZFTlRf'
    'VFlQRV9BVVRIX1BFUk1JU1NJT05fQ0hFQ0sQZhImCiJFVkVOVF9UWVBFX0FVVEhfQVVESVRfTE9H'
    'X1JFQ09SREVEEGcSHgoZRVZFTlRfVFlQRV9DT05GSUdfQ0hBTkdFRBDIARIzCi5FVkVOVF9UWVBF'
    'X1RBU0tNQU5BR0VNRU5UX1BST0pFQ1RfVFlQRV9DSEFOR0VEEKwCEjgKM0VWRU5UX1RZUEVfVEFT'
    'S01BTkFHRU1FTlRfU1RBVFVTX0RFRklOSVRJT05fQ0hBTkdFRBCtAhIcChdFVkVOVF9UWVBFX1RB'
    'U0tfQ1JFQVRFRBCQAxIjCh5FVkVOVF9UWVBFX1RBU0tfU1RBVFVTX0NIQU5HRUQQkQMSHgoZRVZF'
    'TlRfVFlQRV9UQVNLX0NBTkNFTExFRBCSAxIoCiNFVkVOVF9UWVBFX0JPQVJEX0NPTFVNTl9JTkNS'
    'RU1FTlRFRBD0AxIoCiNFVkVOVF9UWVBFX0JPQVJEX0NPTFVNTl9ERUNSRU1FTlRFRBD1AxIgChtF'
    'VkVOVF9UWVBFX0FDVElWSVRZX0NSRUFURUQQ2AQSIQocRVZFTlRfVFlQRV9BQ1RJVklUWV9BUFBS'
    'T1ZFRBDZBBIhChxFVkVOVF9UWVBFX0FDVElWSVRZX1JFSkVDVEVEENoEEiAKG0VWRU5UX1RZUEVf'
    'QUNUSVZJVFlfREVMRVRFRBDbBA==');
