// This is a generated file - do not edit.
//
// Generated from k1s0/system/notification/v1/notification.proto.

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

@$core.Deprecated('Use notificationStatusDescriptor instead')
const NotificationStatus$json = {
  '1': 'NotificationStatus',
  '2': [
    {'1': 'NOTIFICATION_STATUS_UNSPECIFIED', '2': 0},
    {'1': 'NOTIFICATION_STATUS_PENDING', '2': 1},
    {'1': 'NOTIFICATION_STATUS_SENT', '2': 2},
    {'1': 'NOTIFICATION_STATUS_FAILED', '2': 3},
    {'1': 'NOTIFICATION_STATUS_RETRYING', '2': 4},
  ],
};

/// Descriptor for `NotificationStatus`. Decode as a `google.protobuf.EnumDescriptorProto`.
final $typed_data.Uint8List notificationStatusDescriptor = $convert.base64Decode(
    'ChJOb3RpZmljYXRpb25TdGF0dXMSIwofTk9USUZJQ0FUSU9OX1NUQVRVU19VTlNQRUNJRklFRB'
    'AAEh8KG05PVElGSUNBVElPTl9TVEFUVVNfUEVORElORxABEhwKGE5PVElGSUNBVElPTl9TVEFU'
    'VVNfU0VOVBACEh4KGk5PVElGSUNBVElPTl9TVEFUVVNfRkFJTEVEEAMSIAocTk9USUZJQ0FUSU'
    '9OX1NUQVRVU19SRVRSWUlORxAE');

@$core.Deprecated('Use sendNotificationRequestDescriptor instead')
const SendNotificationRequest$json = {
  '1': 'SendNotificationRequest',
  '2': [
    {'1': 'channel_id', '3': 1, '4': 1, '5': 9, '10': 'channelId'},
    {
      '1': 'template_id',
      '3': 2,
      '4': 1,
      '5': 9,
      '9': 0,
      '10': 'templateId',
      '17': true
    },
    {
      '1': 'template_variables',
      '3': 3,
      '4': 3,
      '5': 11,
      '6':
          '.k1s0.system.notification.v1.SendNotificationRequest.TemplateVariablesEntry',
      '10': 'templateVariables'
    },
    {'1': 'recipient', '3': 4, '4': 1, '5': 9, '10': 'recipient'},
    {
      '1': 'subject',
      '3': 5,
      '4': 1,
      '5': 9,
      '9': 1,
      '10': 'subject',
      '17': true
    },
    {'1': 'body', '3': 6, '4': 1, '5': 9, '9': 2, '10': 'body', '17': true},
  ],
  '3': [SendNotificationRequest_TemplateVariablesEntry$json],
  '8': [
    {'1': '_template_id'},
    {'1': '_subject'},
    {'1': '_body'},
  ],
};

@$core.Deprecated('Use sendNotificationRequestDescriptor instead')
const SendNotificationRequest_TemplateVariablesEntry$json = {
  '1': 'TemplateVariablesEntry',
  '2': [
    {'1': 'key', '3': 1, '4': 1, '5': 9, '10': 'key'},
    {'1': 'value', '3': 2, '4': 1, '5': 9, '10': 'value'},
  ],
  '7': {'7': true},
};

/// Descriptor for `SendNotificationRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List sendNotificationRequestDescriptor = $convert.base64Decode(
    'ChdTZW5kTm90aWZpY2F0aW9uUmVxdWVzdBIdCgpjaGFubmVsX2lkGAEgASgJUgljaGFubmVsSW'
    'QSJAoLdGVtcGxhdGVfaWQYAiABKAlIAFIKdGVtcGxhdGVJZIgBARJ6ChJ0ZW1wbGF0ZV92YXJp'
    'YWJsZXMYAyADKAsySy5rMXMwLnN5c3RlbS5ub3RpZmljYXRpb24udjEuU2VuZE5vdGlmaWNhdG'
    'lvblJlcXVlc3QuVGVtcGxhdGVWYXJpYWJsZXNFbnRyeVIRdGVtcGxhdGVWYXJpYWJsZXMSHAoJ'
    'cmVjaXBpZW50GAQgASgJUglyZWNpcGllbnQSHQoHc3ViamVjdBgFIAEoCUgBUgdzdWJqZWN0iA'
    'EBEhcKBGJvZHkYBiABKAlIAlIEYm9keYgBARpEChZUZW1wbGF0ZVZhcmlhYmxlc0VudHJ5EhAK'
    'A2tleRgBIAEoCVIDa2V5EhQKBXZhbHVlGAIgASgJUgV2YWx1ZToCOAFCDgoMX3RlbXBsYXRlX2'
    'lkQgoKCF9zdWJqZWN0QgcKBV9ib2R5');

@$core.Deprecated('Use sendNotificationResponseDescriptor instead')
const SendNotificationResponse$json = {
  '1': 'SendNotificationResponse',
  '2': [
    {'1': 'notification_id', '3': 1, '4': 1, '5': 9, '10': 'notificationId'},
    {'1': 'status', '3': 2, '4': 1, '5': 9, '10': 'status'},
    {'1': 'created_at', '3': 3, '4': 1, '5': 9, '10': 'createdAt'},
    {
      '1': 'created_at_ts',
      '3': 4,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.common.v1.Timestamp',
      '10': 'createdAtTs'
    },
  ],
};

/// Descriptor for `SendNotificationResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List sendNotificationResponseDescriptor = $convert.base64Decode(
    'ChhTZW5kTm90aWZpY2F0aW9uUmVzcG9uc2USJwoPbm90aWZpY2F0aW9uX2lkGAEgASgJUg5ub3'
    'RpZmljYXRpb25JZBIWCgZzdGF0dXMYAiABKAlSBnN0YXR1cxIdCgpjcmVhdGVkX2F0GAMgASgJ'
    'UgljcmVhdGVkQXQSRAoNY3JlYXRlZF9hdF90cxgEIAEoCzIgLmsxczAuc3lzdGVtLmNvbW1vbi'
    '52MS5UaW1lc3RhbXBSC2NyZWF0ZWRBdFRz');

@$core.Deprecated('Use getNotificationRequestDescriptor instead')
const GetNotificationRequest$json = {
  '1': 'GetNotificationRequest',
  '2': [
    {'1': 'notification_id', '3': 1, '4': 1, '5': 9, '10': 'notificationId'},
  ],
};

/// Descriptor for `GetNotificationRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List getNotificationRequestDescriptor =
    $convert.base64Decode(
        'ChZHZXROb3RpZmljYXRpb25SZXF1ZXN0EicKD25vdGlmaWNhdGlvbl9pZBgBIAEoCVIObm90aW'
        'ZpY2F0aW9uSWQ=');

@$core.Deprecated('Use getNotificationResponseDescriptor instead')
const GetNotificationResponse$json = {
  '1': 'GetNotificationResponse',
  '2': [
    {
      '1': 'notification',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.notification.v1.NotificationLog',
      '10': 'notification'
    },
  ],
};

/// Descriptor for `GetNotificationResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List getNotificationResponseDescriptor = $convert.base64Decode(
    'ChdHZXROb3RpZmljYXRpb25SZXNwb25zZRJQCgxub3RpZmljYXRpb24YASABKAsyLC5rMXMwLn'
    'N5c3RlbS5ub3RpZmljYXRpb24udjEuTm90aWZpY2F0aW9uTG9nUgxub3RpZmljYXRpb24=');

@$core.Deprecated('Use retryNotificationRequestDescriptor instead')
const RetryNotificationRequest$json = {
  '1': 'RetryNotificationRequest',
  '2': [
    {'1': 'notification_id', '3': 1, '4': 1, '5': 9, '10': 'notificationId'},
  ],
};

/// Descriptor for `RetryNotificationRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List retryNotificationRequestDescriptor =
    $convert.base64Decode(
        'ChhSZXRyeU5vdGlmaWNhdGlvblJlcXVlc3QSJwoPbm90aWZpY2F0aW9uX2lkGAEgASgJUg5ub3'
        'RpZmljYXRpb25JZA==');

@$core.Deprecated('Use retryNotificationResponseDescriptor instead')
const RetryNotificationResponse$json = {
  '1': 'RetryNotificationResponse',
  '2': [
    {
      '1': 'notification',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.notification.v1.NotificationLog',
      '10': 'notification'
    },
  ],
};

/// Descriptor for `RetryNotificationResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List retryNotificationResponseDescriptor = $convert.base64Decode(
    'ChlSZXRyeU5vdGlmaWNhdGlvblJlc3BvbnNlElAKDG5vdGlmaWNhdGlvbhgBIAEoCzIsLmsxcz'
    'Auc3lzdGVtLm5vdGlmaWNhdGlvbi52MS5Ob3RpZmljYXRpb25Mb2dSDG5vdGlmaWNhdGlvbg==');

@$core.Deprecated('Use listNotificationsRequestDescriptor instead')
const ListNotificationsRequest$json = {
  '1': 'ListNotificationsRequest',
  '2': [
    {
      '1': 'channel_id',
      '3': 1,
      '4': 1,
      '5': 9,
      '9': 0,
      '10': 'channelId',
      '17': true
    },
    {'1': 'status', '3': 2, '4': 1, '5': 9, '9': 1, '10': 'status', '17': true},
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
    {'1': '_channel_id'},
    {'1': '_status'},
  ],
  '9': [
    {'1': 4, '2': 5},
  ],
  '10': ['page_size'],
};

/// Descriptor for `ListNotificationsRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List listNotificationsRequestDescriptor = $convert.base64Decode(
    'ChhMaXN0Tm90aWZpY2F0aW9uc1JlcXVlc3QSIgoKY2hhbm5lbF9pZBgBIAEoCUgAUgljaGFubm'
    'VsSWSIAQESGwoGc3RhdHVzGAIgASgJSAFSBnN0YXR1c4gBARJBCgpwYWdpbmF0aW9uGAMgASgL'
    'MiEuazFzMC5zeXN0ZW0uY29tbW9uLnYxLlBhZ2luYXRpb25SCnBhZ2luYXRpb25CDQoLX2NoYW'
    '5uZWxfaWRCCQoHX3N0YXR1c0oECAQQBVIJcGFnZV9zaXpl');

@$core.Deprecated('Use listNotificationsResponseDescriptor instead')
const ListNotificationsResponse$json = {
  '1': 'ListNotificationsResponse',
  '2': [
    {
      '1': 'notifications',
      '3': 1,
      '4': 3,
      '5': 11,
      '6': '.k1s0.system.notification.v1.NotificationLog',
      '10': 'notifications'
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

/// Descriptor for `ListNotificationsResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List listNotificationsResponseDescriptor = $convert.base64Decode(
    'ChlMaXN0Tm90aWZpY2F0aW9uc1Jlc3BvbnNlElIKDW5vdGlmaWNhdGlvbnMYASADKAsyLC5rMX'
    'MwLnN5c3RlbS5ub3RpZmljYXRpb24udjEuTm90aWZpY2F0aW9uTG9nUg1ub3RpZmljYXRpb25z'
    'EkcKCnBhZ2luYXRpb24YAiABKAsyJy5rMXMwLnN5c3RlbS5jb21tb24udjEuUGFnaW5hdGlvbl'
    'Jlc3VsdFIKcGFnaW5hdGlvbg==');

@$core.Deprecated('Use notificationLogDescriptor instead')
const NotificationLog$json = {
  '1': 'NotificationLog',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 9, '10': 'id'},
    {'1': 'channel_id', '3': 2, '4': 1, '5': 9, '10': 'channelId'},
    {'1': 'channel_type', '3': 3, '4': 1, '5': 9, '10': 'channelType'},
    {
      '1': 'template_id',
      '3': 4,
      '4': 1,
      '5': 9,
      '9': 0,
      '10': 'templateId',
      '17': true
    },
    {'1': 'recipient', '3': 5, '4': 1, '5': 9, '10': 'recipient'},
    {
      '1': 'subject',
      '3': 6,
      '4': 1,
      '5': 9,
      '9': 1,
      '10': 'subject',
      '17': true
    },
    {'1': 'body', '3': 7, '4': 1, '5': 9, '10': 'body'},
    {'1': 'status', '3': 8, '4': 1, '5': 9, '10': 'status'},
    {'1': 'retry_count', '3': 9, '4': 1, '5': 13, '10': 'retryCount'},
    {
      '1': 'error_message',
      '3': 10,
      '4': 1,
      '5': 9,
      '9': 2,
      '10': 'errorMessage',
      '17': true
    },
    {
      '1': 'sent_at',
      '3': 11,
      '4': 1,
      '5': 9,
      '9': 3,
      '10': 'sentAt',
      '17': true
    },
    {'1': 'created_at', '3': 12, '4': 1, '5': 9, '10': 'createdAt'},
    {
      '1': 'status_enum',
      '3': 13,
      '4': 1,
      '5': 14,
      '6': '.k1s0.system.notification.v1.NotificationStatus',
      '10': 'statusEnum'
    },
    {
      '1': 'sent_at_ts',
      '3': 14,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.common.v1.Timestamp',
      '9': 4,
      '10': 'sentAtTs',
      '17': true
    },
    {
      '1': 'created_at_ts',
      '3': 15,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.common.v1.Timestamp',
      '10': 'createdAtTs'
    },
  ],
  '8': [
    {'1': '_template_id'},
    {'1': '_subject'},
    {'1': '_error_message'},
    {'1': '_sent_at'},
    {'1': '_sent_at_ts'},
  ],
};

/// Descriptor for `NotificationLog`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List notificationLogDescriptor = $convert.base64Decode(
    'Cg9Ob3RpZmljYXRpb25Mb2cSDgoCaWQYASABKAlSAmlkEh0KCmNoYW5uZWxfaWQYAiABKAlSCW'
    'NoYW5uZWxJZBIhCgxjaGFubmVsX3R5cGUYAyABKAlSC2NoYW5uZWxUeXBlEiQKC3RlbXBsYXRl'
    'X2lkGAQgASgJSABSCnRlbXBsYXRlSWSIAQESHAoJcmVjaXBpZW50GAUgASgJUglyZWNpcGllbn'
    'QSHQoHc3ViamVjdBgGIAEoCUgBUgdzdWJqZWN0iAEBEhIKBGJvZHkYByABKAlSBGJvZHkSFgoG'
    'c3RhdHVzGAggASgJUgZzdGF0dXMSHwoLcmV0cnlfY291bnQYCSABKA1SCnJldHJ5Q291bnQSKA'
    'oNZXJyb3JfbWVzc2FnZRgKIAEoCUgCUgxlcnJvck1lc3NhZ2WIAQESHAoHc2VudF9hdBgLIAEo'
    'CUgDUgZzZW50QXSIAQESHQoKY3JlYXRlZF9hdBgMIAEoCVIJY3JlYXRlZEF0ElAKC3N0YXR1c1'
    '9lbnVtGA0gASgOMi8uazFzMC5zeXN0ZW0ubm90aWZpY2F0aW9uLnYxLk5vdGlmaWNhdGlvblN0'
    'YXR1c1IKc3RhdHVzRW51bRJDCgpzZW50X2F0X3RzGA4gASgLMiAuazFzMC5zeXN0ZW0uY29tbW'
    '9uLnYxLlRpbWVzdGFtcEgEUghzZW50QXRUc4gBARJECg1jcmVhdGVkX2F0X3RzGA8gASgLMiAu'
    'azFzMC5zeXN0ZW0uY29tbW9uLnYxLlRpbWVzdGFtcFILY3JlYXRlZEF0VHNCDgoMX3RlbXBsYX'
    'RlX2lkQgoKCF9zdWJqZWN0QhAKDl9lcnJvcl9tZXNzYWdlQgoKCF9zZW50X2F0Qg0KC19zZW50'
    'X2F0X3Rz');

@$core.Deprecated('Use channelDescriptor instead')
const Channel$json = {
  '1': 'Channel',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 9, '10': 'id'},
    {'1': 'name', '3': 2, '4': 1, '5': 9, '10': 'name'},
    {'1': 'channel_type', '3': 3, '4': 1, '5': 9, '10': 'channelType'},
    {'1': 'config_json', '3': 4, '4': 1, '5': 9, '10': 'configJson'},
    {'1': 'enabled', '3': 5, '4': 1, '5': 8, '10': 'enabled'},
    {'1': 'created_at', '3': 6, '4': 1, '5': 9, '10': 'createdAt'},
    {'1': 'updated_at', '3': 7, '4': 1, '5': 9, '10': 'updatedAt'},
    {
      '1': 'created_at_ts',
      '3': 8,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.common.v1.Timestamp',
      '10': 'createdAtTs'
    },
    {
      '1': 'updated_at_ts',
      '3': 9,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.common.v1.Timestamp',
      '10': 'updatedAtTs'
    },
  ],
};

/// Descriptor for `Channel`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List channelDescriptor = $convert.base64Decode(
    'CgdDaGFubmVsEg4KAmlkGAEgASgJUgJpZBISCgRuYW1lGAIgASgJUgRuYW1lEiEKDGNoYW5uZW'
    'xfdHlwZRgDIAEoCVILY2hhbm5lbFR5cGUSHwoLY29uZmlnX2pzb24YBCABKAlSCmNvbmZpZ0pz'
    'b24SGAoHZW5hYmxlZBgFIAEoCFIHZW5hYmxlZBIdCgpjcmVhdGVkX2F0GAYgASgJUgljcmVhdG'
    'VkQXQSHQoKdXBkYXRlZF9hdBgHIAEoCVIJdXBkYXRlZEF0EkQKDWNyZWF0ZWRfYXRfdHMYCCAB'
    'KAsyIC5rMXMwLnN5c3RlbS5jb21tb24udjEuVGltZXN0YW1wUgtjcmVhdGVkQXRUcxJECg11cG'
    'RhdGVkX2F0X3RzGAkgASgLMiAuazFzMC5zeXN0ZW0uY29tbW9uLnYxLlRpbWVzdGFtcFILdXBk'
    'YXRlZEF0VHM=');

@$core.Deprecated('Use listChannelsRequestDescriptor instead')
const ListChannelsRequest$json = {
  '1': 'ListChannelsRequest',
  '2': [
    {
      '1': 'channel_type',
      '3': 1,
      '4': 1,
      '5': 9,
      '9': 0,
      '10': 'channelType',
      '17': true
    },
    {'1': 'enabled_only', '3': 2, '4': 1, '5': 8, '10': 'enabledOnly'},
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
    {'1': '_channel_type'},
  ],
  '9': [
    {'1': 4, '2': 5},
  ],
  '10': ['page_size'],
};

/// Descriptor for `ListChannelsRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List listChannelsRequestDescriptor = $convert.base64Decode(
    'ChNMaXN0Q2hhbm5lbHNSZXF1ZXN0EiYKDGNoYW5uZWxfdHlwZRgBIAEoCUgAUgtjaGFubmVsVH'
    'lwZYgBARIhCgxlbmFibGVkX29ubHkYAiABKAhSC2VuYWJsZWRPbmx5EkEKCnBhZ2luYXRpb24Y'
    'AyABKAsyIS5rMXMwLnN5c3RlbS5jb21tb24udjEuUGFnaW5hdGlvblIKcGFnaW5hdGlvbkIPCg'
    '1fY2hhbm5lbF90eXBlSgQIBBAFUglwYWdlX3NpemU=');

@$core.Deprecated('Use listChannelsResponseDescriptor instead')
const ListChannelsResponse$json = {
  '1': 'ListChannelsResponse',
  '2': [
    {
      '1': 'channels',
      '3': 1,
      '4': 3,
      '5': 11,
      '6': '.k1s0.system.notification.v1.Channel',
      '10': 'channels'
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

/// Descriptor for `ListChannelsResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List listChannelsResponseDescriptor = $convert.base64Decode(
    'ChRMaXN0Q2hhbm5lbHNSZXNwb25zZRJACghjaGFubmVscxgBIAMoCzIkLmsxczAuc3lzdGVtLm'
    '5vdGlmaWNhdGlvbi52MS5DaGFubmVsUghjaGFubmVscxJHCgpwYWdpbmF0aW9uGAIgASgLMicu'
    'azFzMC5zeXN0ZW0uY29tbW9uLnYxLlBhZ2luYXRpb25SZXN1bHRSCnBhZ2luYXRpb24=');

@$core.Deprecated('Use createChannelRequestDescriptor instead')
const CreateChannelRequest$json = {
  '1': 'CreateChannelRequest',
  '2': [
    {'1': 'name', '3': 1, '4': 1, '5': 9, '10': 'name'},
    {'1': 'channel_type', '3': 2, '4': 1, '5': 9, '10': 'channelType'},
    {'1': 'config_json', '3': 3, '4': 1, '5': 9, '10': 'configJson'},
    {'1': 'enabled', '3': 4, '4': 1, '5': 8, '10': 'enabled'},
  ],
};

/// Descriptor for `CreateChannelRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List createChannelRequestDescriptor = $convert.base64Decode(
    'ChRDcmVhdGVDaGFubmVsUmVxdWVzdBISCgRuYW1lGAEgASgJUgRuYW1lEiEKDGNoYW5uZWxfdH'
    'lwZRgCIAEoCVILY2hhbm5lbFR5cGUSHwoLY29uZmlnX2pzb24YAyABKAlSCmNvbmZpZ0pzb24S'
    'GAoHZW5hYmxlZBgEIAEoCFIHZW5hYmxlZA==');

@$core.Deprecated('Use createChannelResponseDescriptor instead')
const CreateChannelResponse$json = {
  '1': 'CreateChannelResponse',
  '2': [
    {
      '1': 'channel',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.notification.v1.Channel',
      '10': 'channel'
    },
  ],
};

/// Descriptor for `CreateChannelResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List createChannelResponseDescriptor = $convert.base64Decode(
    'ChVDcmVhdGVDaGFubmVsUmVzcG9uc2USPgoHY2hhbm5lbBgBIAEoCzIkLmsxczAuc3lzdGVtLm'
    '5vdGlmaWNhdGlvbi52MS5DaGFubmVsUgdjaGFubmVs');

@$core.Deprecated('Use getChannelRequestDescriptor instead')
const GetChannelRequest$json = {
  '1': 'GetChannelRequest',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 9, '10': 'id'},
  ],
};

/// Descriptor for `GetChannelRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List getChannelRequestDescriptor =
    $convert.base64Decode('ChFHZXRDaGFubmVsUmVxdWVzdBIOCgJpZBgBIAEoCVICaWQ=');

@$core.Deprecated('Use getChannelResponseDescriptor instead')
const GetChannelResponse$json = {
  '1': 'GetChannelResponse',
  '2': [
    {
      '1': 'channel',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.notification.v1.Channel',
      '10': 'channel'
    },
  ],
};

/// Descriptor for `GetChannelResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List getChannelResponseDescriptor = $convert.base64Decode(
    'ChJHZXRDaGFubmVsUmVzcG9uc2USPgoHY2hhbm5lbBgBIAEoCzIkLmsxczAuc3lzdGVtLm5vdG'
    'lmaWNhdGlvbi52MS5DaGFubmVsUgdjaGFubmVs');

@$core.Deprecated('Use updateChannelRequestDescriptor instead')
const UpdateChannelRequest$json = {
  '1': 'UpdateChannelRequest',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 9, '10': 'id'},
    {'1': 'name', '3': 2, '4': 1, '5': 9, '9': 0, '10': 'name', '17': true},
    {
      '1': 'enabled',
      '3': 3,
      '4': 1,
      '5': 8,
      '9': 1,
      '10': 'enabled',
      '17': true
    },
    {
      '1': 'config_json',
      '3': 4,
      '4': 1,
      '5': 9,
      '9': 2,
      '10': 'configJson',
      '17': true
    },
  ],
  '8': [
    {'1': '_name'},
    {'1': '_enabled'},
    {'1': '_config_json'},
  ],
};

/// Descriptor for `UpdateChannelRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List updateChannelRequestDescriptor = $convert.base64Decode(
    'ChRVcGRhdGVDaGFubmVsUmVxdWVzdBIOCgJpZBgBIAEoCVICaWQSFwoEbmFtZRgCIAEoCUgAUg'
    'RuYW1liAEBEh0KB2VuYWJsZWQYAyABKAhIAVIHZW5hYmxlZIgBARIkCgtjb25maWdfanNvbhgE'
    'IAEoCUgCUgpjb25maWdKc29uiAEBQgcKBV9uYW1lQgoKCF9lbmFibGVkQg4KDF9jb25maWdfan'
    'Nvbg==');

@$core.Deprecated('Use updateChannelResponseDescriptor instead')
const UpdateChannelResponse$json = {
  '1': 'UpdateChannelResponse',
  '2': [
    {
      '1': 'channel',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.notification.v1.Channel',
      '10': 'channel'
    },
  ],
};

/// Descriptor for `UpdateChannelResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List updateChannelResponseDescriptor = $convert.base64Decode(
    'ChVVcGRhdGVDaGFubmVsUmVzcG9uc2USPgoHY2hhbm5lbBgBIAEoCzIkLmsxczAuc3lzdGVtLm'
    '5vdGlmaWNhdGlvbi52MS5DaGFubmVsUgdjaGFubmVs');

@$core.Deprecated('Use deleteChannelRequestDescriptor instead')
const DeleteChannelRequest$json = {
  '1': 'DeleteChannelRequest',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 9, '10': 'id'},
  ],
};

/// Descriptor for `DeleteChannelRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List deleteChannelRequestDescriptor = $convert
    .base64Decode('ChREZWxldGVDaGFubmVsUmVxdWVzdBIOCgJpZBgBIAEoCVICaWQ=');

@$core.Deprecated('Use deleteChannelResponseDescriptor instead')
const DeleteChannelResponse$json = {
  '1': 'DeleteChannelResponse',
  '2': [
    {'1': 'success', '3': 1, '4': 1, '5': 8, '10': 'success'},
    {'1': 'message', '3': 2, '4': 1, '5': 9, '10': 'message'},
  ],
};

/// Descriptor for `DeleteChannelResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List deleteChannelResponseDescriptor = $convert.base64Decode(
    'ChVEZWxldGVDaGFubmVsUmVzcG9uc2USGAoHc3VjY2VzcxgBIAEoCFIHc3VjY2VzcxIYCgdtZX'
    'NzYWdlGAIgASgJUgdtZXNzYWdl');

@$core.Deprecated('Use templateDescriptor instead')
const Template$json = {
  '1': 'Template',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 9, '10': 'id'},
    {'1': 'name', '3': 2, '4': 1, '5': 9, '10': 'name'},
    {'1': 'channel_type', '3': 3, '4': 1, '5': 9, '10': 'channelType'},
    {
      '1': 'subject_template',
      '3': 4,
      '4': 1,
      '5': 9,
      '9': 0,
      '10': 'subjectTemplate',
      '17': true
    },
    {'1': 'body_template', '3': 5, '4': 1, '5': 9, '10': 'bodyTemplate'},
    {'1': 'created_at', '3': 6, '4': 1, '5': 9, '10': 'createdAt'},
    {'1': 'updated_at', '3': 7, '4': 1, '5': 9, '10': 'updatedAt'},
    {
      '1': 'created_at_ts',
      '3': 8,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.common.v1.Timestamp',
      '10': 'createdAtTs'
    },
    {
      '1': 'updated_at_ts',
      '3': 9,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.common.v1.Timestamp',
      '10': 'updatedAtTs'
    },
  ],
  '8': [
    {'1': '_subject_template'},
  ],
};

/// Descriptor for `Template`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List templateDescriptor = $convert.base64Decode(
    'CghUZW1wbGF0ZRIOCgJpZBgBIAEoCVICaWQSEgoEbmFtZRgCIAEoCVIEbmFtZRIhCgxjaGFubm'
    'VsX3R5cGUYAyABKAlSC2NoYW5uZWxUeXBlEi4KEHN1YmplY3RfdGVtcGxhdGUYBCABKAlIAFIP'
    'c3ViamVjdFRlbXBsYXRliAEBEiMKDWJvZHlfdGVtcGxhdGUYBSABKAlSDGJvZHlUZW1wbGF0ZR'
    'IdCgpjcmVhdGVkX2F0GAYgASgJUgljcmVhdGVkQXQSHQoKdXBkYXRlZF9hdBgHIAEoCVIJdXBk'
    'YXRlZEF0EkQKDWNyZWF0ZWRfYXRfdHMYCCABKAsyIC5rMXMwLnN5c3RlbS5jb21tb24udjEuVG'
    'ltZXN0YW1wUgtjcmVhdGVkQXRUcxJECg11cGRhdGVkX2F0X3RzGAkgASgLMiAuazFzMC5zeXN0'
    'ZW0uY29tbW9uLnYxLlRpbWVzdGFtcFILdXBkYXRlZEF0VHNCEwoRX3N1YmplY3RfdGVtcGxhdG'
    'U=');

@$core.Deprecated('Use listTemplatesRequestDescriptor instead')
const ListTemplatesRequest$json = {
  '1': 'ListTemplatesRequest',
  '2': [
    {
      '1': 'channel_type',
      '3': 1,
      '4': 1,
      '5': 9,
      '9': 0,
      '10': 'channelType',
      '17': true
    },
    {
      '1': 'pagination',
      '3': 2,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.common.v1.Pagination',
      '10': 'pagination'
    },
  ],
  '8': [
    {'1': '_channel_type'},
  ],
  '9': [
    {'1': 3, '2': 4},
  ],
  '10': ['page_size'],
};

/// Descriptor for `ListTemplatesRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List listTemplatesRequestDescriptor = $convert.base64Decode(
    'ChRMaXN0VGVtcGxhdGVzUmVxdWVzdBImCgxjaGFubmVsX3R5cGUYASABKAlIAFILY2hhbm5lbF'
    'R5cGWIAQESQQoKcGFnaW5hdGlvbhgCIAEoCzIhLmsxczAuc3lzdGVtLmNvbW1vbi52MS5QYWdp'
    'bmF0aW9uUgpwYWdpbmF0aW9uQg8KDV9jaGFubmVsX3R5cGVKBAgDEARSCXBhZ2Vfc2l6ZQ==');

@$core.Deprecated('Use listTemplatesResponseDescriptor instead')
const ListTemplatesResponse$json = {
  '1': 'ListTemplatesResponse',
  '2': [
    {
      '1': 'templates',
      '3': 1,
      '4': 3,
      '5': 11,
      '6': '.k1s0.system.notification.v1.Template',
      '10': 'templates'
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

/// Descriptor for `ListTemplatesResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List listTemplatesResponseDescriptor = $convert.base64Decode(
    'ChVMaXN0VGVtcGxhdGVzUmVzcG9uc2USQwoJdGVtcGxhdGVzGAEgAygLMiUuazFzMC5zeXN0ZW'
    '0ubm90aWZpY2F0aW9uLnYxLlRlbXBsYXRlUgl0ZW1wbGF0ZXMSRwoKcGFnaW5hdGlvbhgCIAEo'
    'CzInLmsxczAuc3lzdGVtLmNvbW1vbi52MS5QYWdpbmF0aW9uUmVzdWx0UgpwYWdpbmF0aW9u');

@$core.Deprecated('Use createTemplateRequestDescriptor instead')
const CreateTemplateRequest$json = {
  '1': 'CreateTemplateRequest',
  '2': [
    {'1': 'name', '3': 1, '4': 1, '5': 9, '10': 'name'},
    {'1': 'channel_type', '3': 2, '4': 1, '5': 9, '10': 'channelType'},
    {
      '1': 'subject_template',
      '3': 3,
      '4': 1,
      '5': 9,
      '9': 0,
      '10': 'subjectTemplate',
      '17': true
    },
    {'1': 'body_template', '3': 4, '4': 1, '5': 9, '10': 'bodyTemplate'},
  ],
  '8': [
    {'1': '_subject_template'},
  ],
};

/// Descriptor for `CreateTemplateRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List createTemplateRequestDescriptor = $convert.base64Decode(
    'ChVDcmVhdGVUZW1wbGF0ZVJlcXVlc3QSEgoEbmFtZRgBIAEoCVIEbmFtZRIhCgxjaGFubmVsX3'
    'R5cGUYAiABKAlSC2NoYW5uZWxUeXBlEi4KEHN1YmplY3RfdGVtcGxhdGUYAyABKAlIAFIPc3Vi'
    'amVjdFRlbXBsYXRliAEBEiMKDWJvZHlfdGVtcGxhdGUYBCABKAlSDGJvZHlUZW1wbGF0ZUITCh'
    'Ffc3ViamVjdF90ZW1wbGF0ZQ==');

@$core.Deprecated('Use createTemplateResponseDescriptor instead')
const CreateTemplateResponse$json = {
  '1': 'CreateTemplateResponse',
  '2': [
    {
      '1': 'template',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.notification.v1.Template',
      '10': 'template'
    },
  ],
};

/// Descriptor for `CreateTemplateResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List createTemplateResponseDescriptor =
    $convert.base64Decode(
        'ChZDcmVhdGVUZW1wbGF0ZVJlc3BvbnNlEkEKCHRlbXBsYXRlGAEgASgLMiUuazFzMC5zeXN0ZW'
        '0ubm90aWZpY2F0aW9uLnYxLlRlbXBsYXRlUgh0ZW1wbGF0ZQ==');

@$core.Deprecated('Use getTemplateRequestDescriptor instead')
const GetTemplateRequest$json = {
  '1': 'GetTemplateRequest',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 9, '10': 'id'},
  ],
};

/// Descriptor for `GetTemplateRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List getTemplateRequestDescriptor =
    $convert.base64Decode('ChJHZXRUZW1wbGF0ZVJlcXVlc3QSDgoCaWQYASABKAlSAmlk');

@$core.Deprecated('Use getTemplateResponseDescriptor instead')
const GetTemplateResponse$json = {
  '1': 'GetTemplateResponse',
  '2': [
    {
      '1': 'template',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.notification.v1.Template',
      '10': 'template'
    },
  ],
};

/// Descriptor for `GetTemplateResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List getTemplateResponseDescriptor = $convert.base64Decode(
    'ChNHZXRUZW1wbGF0ZVJlc3BvbnNlEkEKCHRlbXBsYXRlGAEgASgLMiUuazFzMC5zeXN0ZW0ubm'
    '90aWZpY2F0aW9uLnYxLlRlbXBsYXRlUgh0ZW1wbGF0ZQ==');

@$core.Deprecated('Use updateTemplateRequestDescriptor instead')
const UpdateTemplateRequest$json = {
  '1': 'UpdateTemplateRequest',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 9, '10': 'id'},
    {'1': 'name', '3': 2, '4': 1, '5': 9, '9': 0, '10': 'name', '17': true},
    {
      '1': 'subject_template',
      '3': 3,
      '4': 1,
      '5': 9,
      '9': 1,
      '10': 'subjectTemplate',
      '17': true
    },
    {
      '1': 'body_template',
      '3': 4,
      '4': 1,
      '5': 9,
      '9': 2,
      '10': 'bodyTemplate',
      '17': true
    },
  ],
  '8': [
    {'1': '_name'},
    {'1': '_subject_template'},
    {'1': '_body_template'},
  ],
};

/// Descriptor for `UpdateTemplateRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List updateTemplateRequestDescriptor = $convert.base64Decode(
    'ChVVcGRhdGVUZW1wbGF0ZVJlcXVlc3QSDgoCaWQYASABKAlSAmlkEhcKBG5hbWUYAiABKAlIAF'
    'IEbmFtZYgBARIuChBzdWJqZWN0X3RlbXBsYXRlGAMgASgJSAFSD3N1YmplY3RUZW1wbGF0ZYgB'
    'ARIoCg1ib2R5X3RlbXBsYXRlGAQgASgJSAJSDGJvZHlUZW1wbGF0ZYgBAUIHCgVfbmFtZUITCh'
    'Ffc3ViamVjdF90ZW1wbGF0ZUIQCg5fYm9keV90ZW1wbGF0ZQ==');

@$core.Deprecated('Use updateTemplateResponseDescriptor instead')
const UpdateTemplateResponse$json = {
  '1': 'UpdateTemplateResponse',
  '2': [
    {
      '1': 'template',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.notification.v1.Template',
      '10': 'template'
    },
  ],
};

/// Descriptor for `UpdateTemplateResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List updateTemplateResponseDescriptor =
    $convert.base64Decode(
        'ChZVcGRhdGVUZW1wbGF0ZVJlc3BvbnNlEkEKCHRlbXBsYXRlGAEgASgLMiUuazFzMC5zeXN0ZW'
        '0ubm90aWZpY2F0aW9uLnYxLlRlbXBsYXRlUgh0ZW1wbGF0ZQ==');

@$core.Deprecated('Use deleteTemplateRequestDescriptor instead')
const DeleteTemplateRequest$json = {
  '1': 'DeleteTemplateRequest',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 9, '10': 'id'},
  ],
};

/// Descriptor for `DeleteTemplateRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List deleteTemplateRequestDescriptor = $convert
    .base64Decode('ChVEZWxldGVUZW1wbGF0ZVJlcXVlc3QSDgoCaWQYASABKAlSAmlk');

@$core.Deprecated('Use deleteTemplateResponseDescriptor instead')
const DeleteTemplateResponse$json = {
  '1': 'DeleteTemplateResponse',
  '2': [
    {'1': 'success', '3': 1, '4': 1, '5': 8, '10': 'success'},
    {'1': 'message', '3': 2, '4': 1, '5': 9, '10': 'message'},
  ],
};

/// Descriptor for `DeleteTemplateResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List deleteTemplateResponseDescriptor =
    $convert.base64Decode(
        'ChZEZWxldGVUZW1wbGF0ZVJlc3BvbnNlEhgKB3N1Y2Nlc3MYASABKAhSB3N1Y2Nlc3MSGAoHbW'
        'Vzc2FnZRgCIAEoCVIHbWVzc2FnZQ==');
