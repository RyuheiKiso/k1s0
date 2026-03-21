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
    {'1': 'EVENT_TYPE_ACCOUNTING_ENTRY_CREATED', '2': 300},
    {'1': 'EVENT_TYPE_ACCOUNTING_ENTRY_APPROVED', '2': 301},
    {'1': 'EVENT_TYPE_ORDER_CREATED', '2': 400},
    {'1': 'EVENT_TYPE_ORDER_UPDATED', '2': 401},
    {'1': 'EVENT_TYPE_ORDER_CANCELLED', '2': 402},
    {'1': 'EVENT_TYPE_INVENTORY_RESERVED', '2': 500},
    {'1': 'EVENT_TYPE_INVENTORY_RELEASED', '2': 501},
    {'1': 'EVENT_TYPE_PAYMENT_INITIATED', '2': 600},
    {'1': 'EVENT_TYPE_PAYMENT_COMPLETED', '2': 601},
    {'1': 'EVENT_TYPE_PAYMENT_FAILED', '2': 602},
    {'1': 'EVENT_TYPE_PAYMENT_REFUNDED', '2': 603},
  ],
};

/// Descriptor for `EventType`. Decode as a `google.protobuf.EnumDescriptorProto`.
final $typed_data.Uint8List eventTypeDescriptor = $convert.base64Decode(
    'CglFdmVudFR5cGUSGgoWRVZFTlRfVFlQRV9VTlNQRUNJRklFRBAAEhkKFUVWRU5UX1RZUEVfQV'
    'VUSF9MT0dJThBkEiQKIEVWRU5UX1RZUEVfQVVUSF9UT0tFTl9WQUxJREFUSU9OEGUSJAogRVZF'
    'TlRfVFlQRV9BVVRIX1BFUk1JU1NJT05fQ0hFQ0sQZhImCiJFVkVOVF9UWVBFX0FVVEhfQVVESV'
    'RfTE9HX1JFQ09SREVEEGcSHgoZRVZFTlRfVFlQRV9DT05GSUdfQ0hBTkdFRBDIARIoCiNFVkVO'
    'VF9UWVBFX0FDQ09VTlRJTkdfRU5UUllfQ1JFQVRFRBCsAhIpCiRFVkVOVF9UWVBFX0FDQ09VTl'
    'RJTkdfRU5UUllfQVBQUk9WRUQQrQISHQoYRVZFTlRfVFlQRV9PUkRFUl9DUkVBVEVEEJADEh0K'
    'GEVWRU5UX1RZUEVfT1JERVJfVVBEQVRFRBCRAxIfChpFVkVOVF9UWVBFX09SREVSX0NBTkNFTE'
    'xFRBCSAxIiCh1FVkVOVF9UWVBFX0lOVkVOVE9SWV9SRVNFUlZFRBD0AxIiCh1FVkVOVF9UWVBF'
    'X0lOVkVOVE9SWV9SRUxFQVNFRBD1AxIhChxFVkVOVF9UWVBFX1BBWU1FTlRfSU5JVElBVEVEEN'
    'gEEiEKHEVWRU5UX1RZUEVfUEFZTUVOVF9DT01QTEVURUQQ2QQSHgoZRVZFTlRfVFlQRV9QQVlN'
    'RU5UX0ZBSUxFRBDaBBIgChtFVkVOVF9UWVBFX1BBWU1FTlRfUkVGVU5ERUQQ2wQ=');
