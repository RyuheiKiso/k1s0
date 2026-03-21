// This is a generated file - do not edit.
//
// Generated from k1s0/service/order/v1/order.proto.

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

@$core.Deprecated('Use orderStatusDescriptor instead')
const OrderStatus$json = {
  '1': 'OrderStatus',
  '2': [
    {'1': 'ORDER_STATUS_UNSPECIFIED', '2': 0},
    {'1': 'ORDER_STATUS_PENDING', '2': 1},
    {'1': 'ORDER_STATUS_CONFIRMED', '2': 2},
    {'1': 'ORDER_STATUS_PROCESSING', '2': 3},
    {'1': 'ORDER_STATUS_SHIPPED', '2': 4},
    {'1': 'ORDER_STATUS_DELIVERED', '2': 5},
    {'1': 'ORDER_STATUS_CANCELLED', '2': 6},
    {'1': 'ORDER_STATUS_REFUNDED', '2': 7},
  ],
};

/// Descriptor for `OrderStatus`. Decode as a `google.protobuf.EnumDescriptorProto`.
final $typed_data.Uint8List orderStatusDescriptor = $convert.base64Decode(
    'CgtPcmRlclN0YXR1cxIcChhPUkRFUl9TVEFUVVNfVU5TUEVDSUZJRUQQABIYChRPUkRFUl9TVE'
    'FUVVNfUEVORElORxABEhoKFk9SREVSX1NUQVRVU19DT05GSVJNRUQQAhIbChdPUkRFUl9TVEFU'
    'VVNfUFJPQ0VTU0lORxADEhgKFE9SREVSX1NUQVRVU19TSElQUEVEEAQSGgoWT1JERVJfU1RBVF'
    'VTX0RFTElWRVJFRBAFEhoKFk9SREVSX1NUQVRVU19DQU5DRUxMRUQQBhIZChVPUkRFUl9TVEFU'
    'VVNfUkVGVU5ERUQQBw==');

@$core.Deprecated('Use orderDescriptor instead')
const Order$json = {
  '1': 'Order',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 9, '10': 'id'},
    {'1': 'customer_id', '3': 2, '4': 1, '5': 9, '10': 'customerId'},
    {
      '1': 'status',
      '3': 3,
      '4': 1,
      '5': 9,
      '8': {'3': true},
      '10': 'status',
    },
    {'1': 'total_amount', '3': 4, '4': 1, '5': 3, '10': 'totalAmount'},
    {'1': 'currency', '3': 5, '4': 1, '5': 9, '10': 'currency'},
    {'1': 'notes', '3': 6, '4': 1, '5': 9, '9': 0, '10': 'notes', '17': true},
    {'1': 'created_by', '3': 7, '4': 1, '5': 9, '10': 'createdBy'},
    {
      '1': 'updated_by',
      '3': 8,
      '4': 1,
      '5': 9,
      '9': 1,
      '10': 'updatedBy',
      '17': true
    },
    {'1': 'version', '3': 9, '4': 1, '5': 5, '10': 'version'},
    {
      '1': 'items',
      '3': 10,
      '4': 3,
      '5': 11,
      '6': '.k1s0.service.order.v1.OrderItem',
      '10': 'items'
    },
    {
      '1': 'created_at',
      '3': 11,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.common.v1.Timestamp',
      '10': 'createdAt'
    },
    {
      '1': 'updated_at',
      '3': 12,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.common.v1.Timestamp',
      '10': 'updatedAt'
    },
    {
      '1': 'status_enum',
      '3': 13,
      '4': 1,
      '5': 14,
      '6': '.k1s0.service.order.v1.OrderStatus',
      '10': 'statusEnum'
    },
  ],
  '8': [
    {'1': '_notes'},
    {'1': '_updated_by'},
  ],
};

/// Descriptor for `Order`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List orderDescriptor = $convert.base64Decode(
    'CgVPcmRlchIOCgJpZBgBIAEoCVICaWQSHwoLY3VzdG9tZXJfaWQYAiABKAlSCmN1c3RvbWVySW'
    'QSGgoGc3RhdHVzGAMgASgJQgIYAVIGc3RhdHVzEiEKDHRvdGFsX2Ftb3VudBgEIAEoA1ILdG90'
    'YWxBbW91bnQSGgoIY3VycmVuY3kYBSABKAlSCGN1cnJlbmN5EhkKBW5vdGVzGAYgASgJSABSBW'
    '5vdGVziAEBEh0KCmNyZWF0ZWRfYnkYByABKAlSCWNyZWF0ZWRCeRIiCgp1cGRhdGVkX2J5GAgg'
    'ASgJSAFSCXVwZGF0ZWRCeYgBARIYCgd2ZXJzaW9uGAkgASgFUgd2ZXJzaW9uEjYKBWl0ZW1zGA'
    'ogAygLMiAuazFzMC5zZXJ2aWNlLm9yZGVyLnYxLk9yZGVySXRlbVIFaXRlbXMSPwoKY3JlYXRl'
    'ZF9hdBgLIAEoCzIgLmsxczAuc3lzdGVtLmNvbW1vbi52MS5UaW1lc3RhbXBSCWNyZWF0ZWRBdB'
    'I/Cgp1cGRhdGVkX2F0GAwgASgLMiAuazFzMC5zeXN0ZW0uY29tbW9uLnYxLlRpbWVzdGFtcFIJ'
    'dXBkYXRlZEF0EkMKC3N0YXR1c19lbnVtGA0gASgOMiIuazFzMC5zZXJ2aWNlLm9yZGVyLnYxLk'
    '9yZGVyU3RhdHVzUgpzdGF0dXNFbnVtQggKBl9ub3Rlc0INCgtfdXBkYXRlZF9ieQ==');

@$core.Deprecated('Use orderItemDescriptor instead')
const OrderItem$json = {
  '1': 'OrderItem',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 9, '10': 'id'},
    {'1': 'order_id', '3': 2, '4': 1, '5': 9, '10': 'orderId'},
    {'1': 'product_id', '3': 3, '4': 1, '5': 9, '10': 'productId'},
    {'1': 'product_name', '3': 4, '4': 1, '5': 9, '10': 'productName'},
    {'1': 'quantity', '3': 5, '4': 1, '5': 5, '10': 'quantity'},
    {'1': 'unit_price', '3': 6, '4': 1, '5': 3, '10': 'unitPrice'},
    {'1': 'subtotal', '3': 7, '4': 1, '5': 3, '10': 'subtotal'},
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

/// Descriptor for `OrderItem`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List orderItemDescriptor = $convert.base64Decode(
    'CglPcmRlckl0ZW0SDgoCaWQYASABKAlSAmlkEhkKCG9yZGVyX2lkGAIgASgJUgdvcmRlcklkEh'
    '0KCnByb2R1Y3RfaWQYAyABKAlSCXByb2R1Y3RJZBIhCgxwcm9kdWN0X25hbWUYBCABKAlSC3By'
    'b2R1Y3ROYW1lEhoKCHF1YW50aXR5GAUgASgFUghxdWFudGl0eRIdCgp1bml0X3ByaWNlGAYgAS'
    'gDUgl1bml0UHJpY2USGgoIc3VidG90YWwYByABKANSCHN1YnRvdGFsEj8KCmNyZWF0ZWRfYXQY'
    'CCABKAsyIC5rMXMwLnN5c3RlbS5jb21tb24udjEuVGltZXN0YW1wUgljcmVhdGVkQXQ=');

@$core.Deprecated('Use createOrderRequestDescriptor instead')
const CreateOrderRequest$json = {
  '1': 'CreateOrderRequest',
  '2': [
    {'1': 'customer_id', '3': 1, '4': 1, '5': 9, '10': 'customerId'},
    {'1': 'currency', '3': 2, '4': 1, '5': 9, '10': 'currency'},
    {'1': 'notes', '3': 3, '4': 1, '5': 9, '9': 0, '10': 'notes', '17': true},
    {
      '1': 'items',
      '3': 4,
      '4': 3,
      '5': 11,
      '6': '.k1s0.service.order.v1.CreateOrderItemRequest',
      '10': 'items'
    },
  ],
  '8': [
    {'1': '_notes'},
  ],
};

/// Descriptor for `CreateOrderRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List createOrderRequestDescriptor = $convert.base64Decode(
    'ChJDcmVhdGVPcmRlclJlcXVlc3QSHwoLY3VzdG9tZXJfaWQYASABKAlSCmN1c3RvbWVySWQSGg'
    'oIY3VycmVuY3kYAiABKAlSCGN1cnJlbmN5EhkKBW5vdGVzGAMgASgJSABSBW5vdGVziAEBEkMK'
    'BWl0ZW1zGAQgAygLMi0uazFzMC5zZXJ2aWNlLm9yZGVyLnYxLkNyZWF0ZU9yZGVySXRlbVJlcX'
    'Vlc3RSBWl0ZW1zQggKBl9ub3Rlcw==');

@$core.Deprecated('Use createOrderItemRequestDescriptor instead')
const CreateOrderItemRequest$json = {
  '1': 'CreateOrderItemRequest',
  '2': [
    {'1': 'product_id', '3': 1, '4': 1, '5': 9, '10': 'productId'},
    {'1': 'product_name', '3': 2, '4': 1, '5': 9, '10': 'productName'},
    {'1': 'quantity', '3': 3, '4': 1, '5': 5, '10': 'quantity'},
    {'1': 'unit_price', '3': 4, '4': 1, '5': 3, '10': 'unitPrice'},
  ],
};

/// Descriptor for `CreateOrderItemRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List createOrderItemRequestDescriptor = $convert.base64Decode(
    'ChZDcmVhdGVPcmRlckl0ZW1SZXF1ZXN0Eh0KCnByb2R1Y3RfaWQYASABKAlSCXByb2R1Y3RJZB'
    'IhCgxwcm9kdWN0X25hbWUYAiABKAlSC3Byb2R1Y3ROYW1lEhoKCHF1YW50aXR5GAMgASgFUghx'
    'dWFudGl0eRIdCgp1bml0X3ByaWNlGAQgASgDUgl1bml0UHJpY2U=');

@$core.Deprecated('Use createOrderResponseDescriptor instead')
const CreateOrderResponse$json = {
  '1': 'CreateOrderResponse',
  '2': [
    {
      '1': 'order',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.k1s0.service.order.v1.Order',
      '10': 'order'
    },
  ],
};

/// Descriptor for `CreateOrderResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List createOrderResponseDescriptor = $convert.base64Decode(
    'ChNDcmVhdGVPcmRlclJlc3BvbnNlEjIKBW9yZGVyGAEgASgLMhwuazFzMC5zZXJ2aWNlLm9yZG'
    'VyLnYxLk9yZGVyUgVvcmRlcg==');

@$core.Deprecated('Use getOrderRequestDescriptor instead')
const GetOrderRequest$json = {
  '1': 'GetOrderRequest',
  '2': [
    {'1': 'order_id', '3': 1, '4': 1, '5': 9, '10': 'orderId'},
  ],
};

/// Descriptor for `GetOrderRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List getOrderRequestDescriptor = $convert.base64Decode(
    'Cg9HZXRPcmRlclJlcXVlc3QSGQoIb3JkZXJfaWQYASABKAlSB29yZGVySWQ=');

@$core.Deprecated('Use getOrderResponseDescriptor instead')
const GetOrderResponse$json = {
  '1': 'GetOrderResponse',
  '2': [
    {
      '1': 'order',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.k1s0.service.order.v1.Order',
      '10': 'order'
    },
  ],
};

/// Descriptor for `GetOrderResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List getOrderResponseDescriptor = $convert.base64Decode(
    'ChBHZXRPcmRlclJlc3BvbnNlEjIKBW9yZGVyGAEgASgLMhwuazFzMC5zZXJ2aWNlLm9yZGVyLn'
    'YxLk9yZGVyUgVvcmRlcg==');

@$core.Deprecated('Use listOrdersRequestDescriptor instead')
const ListOrdersRequest$json = {
  '1': 'ListOrdersRequest',
  '2': [
    {
      '1': 'customer_id',
      '3': 1,
      '4': 1,
      '5': 9,
      '9': 0,
      '10': 'customerId',
      '17': true
    },
    {
      '1': 'status',
      '3': 2,
      '4': 1,
      '5': 9,
      '8': {'3': true},
      '9': 1,
      '10': 'status',
      '17': true,
    },
    {
      '1': 'pagination',
      '3': 3,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.common.v1.Pagination',
      '10': 'pagination'
    },
    {
      '1': 'status_enum',
      '3': 4,
      '4': 1,
      '5': 14,
      '6': '.k1s0.service.order.v1.OrderStatus',
      '9': 2,
      '10': 'statusEnum',
      '17': true
    },
  ],
  '8': [
    {'1': '_customer_id'},
    {'1': '_status'},
    {'1': '_status_enum'},
  ],
};

/// Descriptor for `ListOrdersRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List listOrdersRequestDescriptor = $convert.base64Decode(
    'ChFMaXN0T3JkZXJzUmVxdWVzdBIkCgtjdXN0b21lcl9pZBgBIAEoCUgAUgpjdXN0b21lcklkiA'
    'EBEh8KBnN0YXR1cxgCIAEoCUICGAFIAVIGc3RhdHVziAEBEkEKCnBhZ2luYXRpb24YAyABKAsy'
    'IS5rMXMwLnN5c3RlbS5jb21tb24udjEuUGFnaW5hdGlvblIKcGFnaW5hdGlvbhJICgtzdGF0dX'
    'NfZW51bRgEIAEoDjIiLmsxczAuc2VydmljZS5vcmRlci52MS5PcmRlclN0YXR1c0gCUgpzdGF0'
    'dXNFbnVtiAEBQg4KDF9jdXN0b21lcl9pZEIJCgdfc3RhdHVzQg4KDF9zdGF0dXNfZW51bQ==');

@$core.Deprecated('Use listOrdersResponseDescriptor instead')
const ListOrdersResponse$json = {
  '1': 'ListOrdersResponse',
  '2': [
    {
      '1': 'orders',
      '3': 1,
      '4': 3,
      '5': 11,
      '6': '.k1s0.service.order.v1.Order',
      '10': 'orders'
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

/// Descriptor for `ListOrdersResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List listOrdersResponseDescriptor = $convert.base64Decode(
    'ChJMaXN0T3JkZXJzUmVzcG9uc2USNAoGb3JkZXJzGAEgAygLMhwuazFzMC5zZXJ2aWNlLm9yZG'
    'VyLnYxLk9yZGVyUgZvcmRlcnMSRwoKcGFnaW5hdGlvbhgDIAEoCzInLmsxczAuc3lzdGVtLmNv'
    'bW1vbi52MS5QYWdpbmF0aW9uUmVzdWx0UgpwYWdpbmF0aW9uSgQIAhADUgt0b3RhbF9jb3VudA'
    '==');

@$core.Deprecated('Use updateOrderStatusRequestDescriptor instead')
const UpdateOrderStatusRequest$json = {
  '1': 'UpdateOrderStatusRequest',
  '2': [
    {'1': 'order_id', '3': 1, '4': 1, '5': 9, '10': 'orderId'},
    {
      '1': 'status',
      '3': 2,
      '4': 1,
      '5': 9,
      '8': {'3': true},
      '10': 'status',
    },
    {
      '1': 'status_enum',
      '3': 3,
      '4': 1,
      '5': 14,
      '6': '.k1s0.service.order.v1.OrderStatus',
      '10': 'statusEnum'
    },
  ],
};

/// Descriptor for `UpdateOrderStatusRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List updateOrderStatusRequestDescriptor = $convert.base64Decode(
    'ChhVcGRhdGVPcmRlclN0YXR1c1JlcXVlc3QSGQoIb3JkZXJfaWQYASABKAlSB29yZGVySWQSGg'
    'oGc3RhdHVzGAIgASgJQgIYAVIGc3RhdHVzEkMKC3N0YXR1c19lbnVtGAMgASgOMiIuazFzMC5z'
    'ZXJ2aWNlLm9yZGVyLnYxLk9yZGVyU3RhdHVzUgpzdGF0dXNFbnVt');

@$core.Deprecated('Use updateOrderStatusResponseDescriptor instead')
const UpdateOrderStatusResponse$json = {
  '1': 'UpdateOrderStatusResponse',
  '2': [
    {
      '1': 'order',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.k1s0.service.order.v1.Order',
      '10': 'order'
    },
  ],
};

/// Descriptor for `UpdateOrderStatusResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List updateOrderStatusResponseDescriptor =
    $convert.base64Decode(
        'ChlVcGRhdGVPcmRlclN0YXR1c1Jlc3BvbnNlEjIKBW9yZGVyGAEgASgLMhwuazFzMC5zZXJ2aW'
        'NlLm9yZGVyLnYxLk9yZGVyUgVvcmRlcg==');
