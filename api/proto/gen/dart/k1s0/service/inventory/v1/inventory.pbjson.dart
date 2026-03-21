// This is a generated file - do not edit.
//
// Generated from k1s0/service/inventory/v1/inventory.proto.

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

@$core.Deprecated('Use inventoryItemDescriptor instead')
const InventoryItem$json = {
  '1': 'InventoryItem',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 9, '10': 'id'},
    {'1': 'product_id', '3': 2, '4': 1, '5': 9, '10': 'productId'},
    {'1': 'warehouse_id', '3': 3, '4': 1, '5': 9, '10': 'warehouseId'},
    {'1': 'qty_available', '3': 4, '4': 1, '5': 5, '10': 'qtyAvailable'},
    {'1': 'qty_reserved', '3': 5, '4': 1, '5': 5, '10': 'qtyReserved'},
    {'1': 'version', '3': 6, '4': 1, '5': 5, '10': 'version'},
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
};

/// Descriptor for `InventoryItem`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List inventoryItemDescriptor = $convert.base64Decode(
    'Cg1JbnZlbnRvcnlJdGVtEg4KAmlkGAEgASgJUgJpZBIdCgpwcm9kdWN0X2lkGAIgASgJUglwcm'
    '9kdWN0SWQSIQoMd2FyZWhvdXNlX2lkGAMgASgJUgt3YXJlaG91c2VJZBIjCg1xdHlfYXZhaWxh'
    'YmxlGAQgASgFUgxxdHlBdmFpbGFibGUSIQoMcXR5X3Jlc2VydmVkGAUgASgFUgtxdHlSZXNlcn'
    'ZlZBIYCgd2ZXJzaW9uGAYgASgFUgd2ZXJzaW9uEj8KCmNyZWF0ZWRfYXQYByABKAsyIC5rMXMw'
    'LnN5c3RlbS5jb21tb24udjEuVGltZXN0YW1wUgljcmVhdGVkQXQSPwoKdXBkYXRlZF9hdBgIIA'
    'EoCzIgLmsxczAuc3lzdGVtLmNvbW1vbi52MS5UaW1lc3RhbXBSCXVwZGF0ZWRBdA==');

@$core.Deprecated('Use reserveStockRequestDescriptor instead')
const ReserveStockRequest$json = {
  '1': 'ReserveStockRequest',
  '2': [
    {'1': 'order_id', '3': 1, '4': 1, '5': 9, '10': 'orderId'},
    {'1': 'product_id', '3': 2, '4': 1, '5': 9, '10': 'productId'},
    {'1': 'warehouse_id', '3': 3, '4': 1, '5': 9, '10': 'warehouseId'},
    {'1': 'quantity', '3': 4, '4': 1, '5': 5, '10': 'quantity'},
  ],
};

/// Descriptor for `ReserveStockRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List reserveStockRequestDescriptor = $convert.base64Decode(
    'ChNSZXNlcnZlU3RvY2tSZXF1ZXN0EhkKCG9yZGVyX2lkGAEgASgJUgdvcmRlcklkEh0KCnByb2'
    'R1Y3RfaWQYAiABKAlSCXByb2R1Y3RJZBIhCgx3YXJlaG91c2VfaWQYAyABKAlSC3dhcmVob3Vz'
    'ZUlkEhoKCHF1YW50aXR5GAQgASgFUghxdWFudGl0eQ==');

@$core.Deprecated('Use reserveStockResponseDescriptor instead')
const ReserveStockResponse$json = {
  '1': 'ReserveStockResponse',
  '2': [
    {
      '1': 'item',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.k1s0.service.inventory.v1.InventoryItem',
      '10': 'item'
    },
  ],
};

/// Descriptor for `ReserveStockResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List reserveStockResponseDescriptor = $convert.base64Decode(
    'ChRSZXNlcnZlU3RvY2tSZXNwb25zZRI8CgRpdGVtGAEgASgLMiguazFzMC5zZXJ2aWNlLmludm'
    'VudG9yeS52MS5JbnZlbnRvcnlJdGVtUgRpdGVt');

@$core.Deprecated('Use releaseStockRequestDescriptor instead')
const ReleaseStockRequest$json = {
  '1': 'ReleaseStockRequest',
  '2': [
    {'1': 'order_id', '3': 1, '4': 1, '5': 9, '10': 'orderId'},
    {'1': 'product_id', '3': 2, '4': 1, '5': 9, '10': 'productId'},
    {'1': 'warehouse_id', '3': 3, '4': 1, '5': 9, '10': 'warehouseId'},
    {'1': 'quantity', '3': 4, '4': 1, '5': 5, '10': 'quantity'},
    {'1': 'reason', '3': 5, '4': 1, '5': 9, '10': 'reason'},
  ],
};

/// Descriptor for `ReleaseStockRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List releaseStockRequestDescriptor = $convert.base64Decode(
    'ChNSZWxlYXNlU3RvY2tSZXF1ZXN0EhkKCG9yZGVyX2lkGAEgASgJUgdvcmRlcklkEh0KCnByb2'
    'R1Y3RfaWQYAiABKAlSCXByb2R1Y3RJZBIhCgx3YXJlaG91c2VfaWQYAyABKAlSC3dhcmVob3Vz'
    'ZUlkEhoKCHF1YW50aXR5GAQgASgFUghxdWFudGl0eRIWCgZyZWFzb24YBSABKAlSBnJlYXNvbg'
    '==');

@$core.Deprecated('Use releaseStockResponseDescriptor instead')
const ReleaseStockResponse$json = {
  '1': 'ReleaseStockResponse',
  '2': [
    {
      '1': 'item',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.k1s0.service.inventory.v1.InventoryItem',
      '10': 'item'
    },
  ],
};

/// Descriptor for `ReleaseStockResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List releaseStockResponseDescriptor = $convert.base64Decode(
    'ChRSZWxlYXNlU3RvY2tSZXNwb25zZRI8CgRpdGVtGAEgASgLMiguazFzMC5zZXJ2aWNlLmludm'
    'VudG9yeS52MS5JbnZlbnRvcnlJdGVtUgRpdGVt');

@$core.Deprecated('Use getInventoryRequestDescriptor instead')
const GetInventoryRequest$json = {
  '1': 'GetInventoryRequest',
  '2': [
    {'1': 'inventory_id', '3': 1, '4': 1, '5': 9, '10': 'inventoryId'},
  ],
};

/// Descriptor for `GetInventoryRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List getInventoryRequestDescriptor = $convert.base64Decode(
    'ChNHZXRJbnZlbnRvcnlSZXF1ZXN0EiEKDGludmVudG9yeV9pZBgBIAEoCVILaW52ZW50b3J5SW'
    'Q=');

@$core.Deprecated('Use getInventoryResponseDescriptor instead')
const GetInventoryResponse$json = {
  '1': 'GetInventoryResponse',
  '2': [
    {
      '1': 'item',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.k1s0.service.inventory.v1.InventoryItem',
      '10': 'item'
    },
  ],
};

/// Descriptor for `GetInventoryResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List getInventoryResponseDescriptor = $convert.base64Decode(
    'ChRHZXRJbnZlbnRvcnlSZXNwb25zZRI8CgRpdGVtGAEgASgLMiguazFzMC5zZXJ2aWNlLmludm'
    'VudG9yeS52MS5JbnZlbnRvcnlJdGVtUgRpdGVt');

@$core.Deprecated('Use listInventoryRequestDescriptor instead')
const ListInventoryRequest$json = {
  '1': 'ListInventoryRequest',
  '2': [
    {
      '1': 'product_id',
      '3': 1,
      '4': 1,
      '5': 9,
      '9': 0,
      '10': 'productId',
      '17': true
    },
    {
      '1': 'warehouse_id',
      '3': 2,
      '4': 1,
      '5': 9,
      '9': 1,
      '10': 'warehouseId',
      '17': true
    },
    {
      '1': 'pagination',
      '3': 3,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.common.v1.Pagination',
      '10': 'pagination'
    },
  ],
  '8': [
    {'1': '_product_id'},
    {'1': '_warehouse_id'},
  ],
};

/// Descriptor for `ListInventoryRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List listInventoryRequestDescriptor = $convert.base64Decode(
    'ChRMaXN0SW52ZW50b3J5UmVxdWVzdBIiCgpwcm9kdWN0X2lkGAEgASgJSABSCXByb2R1Y3RJZI'
    'gBARImCgx3YXJlaG91c2VfaWQYAiABKAlIAVILd2FyZWhvdXNlSWSIAQESQQoKcGFnaW5hdGlv'
    'bhgDIAEoCzIhLmsxczAuc3lzdGVtLmNvbW1vbi52MS5QYWdpbmF0aW9uUgpwYWdpbmF0aW9uQg'
    '0KC19wcm9kdWN0X2lkQg8KDV93YXJlaG91c2VfaWQ=');

@$core.Deprecated('Use listInventoryResponseDescriptor instead')
const ListInventoryResponse$json = {
  '1': 'ListInventoryResponse',
  '2': [
    {
      '1': 'items',
      '3': 1,
      '4': 3,
      '5': 11,
      '6': '.k1s0.service.inventory.v1.InventoryItem',
      '10': 'items'
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
  '9': [
    {'1': 2, '2': 3},
  ],
  '10': ['total_count'],
};

/// Descriptor for `ListInventoryResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List listInventoryResponseDescriptor = $convert.base64Decode(
    'ChVMaXN0SW52ZW50b3J5UmVzcG9uc2USPgoFaXRlbXMYASADKAsyKC5rMXMwLnNlcnZpY2UuaW'
    '52ZW50b3J5LnYxLkludmVudG9yeUl0ZW1SBWl0ZW1zEkcKCnBhZ2luYXRpb24YAyABKAsyJy5r'
    'MXMwLnN5c3RlbS5jb21tb24udjEuUGFnaW5hdGlvblJlc3VsdFIKcGFnaW5hdGlvbkoECAIQA1'
    'ILdG90YWxfY291bnQ=');

@$core.Deprecated('Use updateStockRequestDescriptor instead')
const UpdateStockRequest$json = {
  '1': 'UpdateStockRequest',
  '2': [
    {'1': 'inventory_id', '3': 1, '4': 1, '5': 9, '10': 'inventoryId'},
    {'1': 'qty_available', '3': 2, '4': 1, '5': 5, '10': 'qtyAvailable'},
    {'1': 'expected_version', '3': 3, '4': 1, '5': 5, '10': 'expectedVersion'},
  ],
};

/// Descriptor for `UpdateStockRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List updateStockRequestDescriptor = $convert.base64Decode(
    'ChJVcGRhdGVTdG9ja1JlcXVlc3QSIQoMaW52ZW50b3J5X2lkGAEgASgJUgtpbnZlbnRvcnlJZB'
    'IjCg1xdHlfYXZhaWxhYmxlGAIgASgFUgxxdHlBdmFpbGFibGUSKQoQZXhwZWN0ZWRfdmVyc2lv'
    'bhgDIAEoBVIPZXhwZWN0ZWRWZXJzaW9u');

@$core.Deprecated('Use updateStockResponseDescriptor instead')
const UpdateStockResponse$json = {
  '1': 'UpdateStockResponse',
  '2': [
    {
      '1': 'item',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.k1s0.service.inventory.v1.InventoryItem',
      '10': 'item'
    },
  ],
};

/// Descriptor for `UpdateStockResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List updateStockResponseDescriptor = $convert.base64Decode(
    'ChNVcGRhdGVTdG9ja1Jlc3BvbnNlEjwKBGl0ZW0YASABKAsyKC5rMXMwLnNlcnZpY2UuaW52ZW'
    '50b3J5LnYxLkludmVudG9yeUl0ZW1SBGl0ZW0=');
