// This is a generated file - do not edit.
//
// Generated from k1s0/system/ruleengine/v1/rule_engine.proto.

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

@$core.Deprecated('Use ruleDescriptor instead')
const Rule$json = {
  '1': 'Rule',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 9, '10': 'id'},
    {'1': 'name', '3': 2, '4': 1, '5': 9, '10': 'name'},
    {'1': 'description', '3': 3, '4': 1, '5': 9, '10': 'description'},
    {'1': 'priority', '3': 4, '4': 1, '5': 5, '10': 'priority'},
    {'1': 'when_json', '3': 5, '4': 1, '5': 12, '10': 'whenJson'},
    {'1': 'then_json', '3': 6, '4': 1, '5': 12, '10': 'thenJson'},
    {'1': 'enabled', '3': 7, '4': 1, '5': 8, '10': 'enabled'},
    {'1': 'version', '3': 8, '4': 1, '5': 13, '10': 'version'},
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

/// Descriptor for `Rule`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List ruleDescriptor = $convert.base64Decode(
    'CgRSdWxlEg4KAmlkGAEgASgJUgJpZBISCgRuYW1lGAIgASgJUgRuYW1lEiAKC2Rlc2NyaXB0aW'
    '9uGAMgASgJUgtkZXNjcmlwdGlvbhIaCghwcmlvcml0eRgEIAEoBVIIcHJpb3JpdHkSGwoJd2hl'
    'bl9qc29uGAUgASgMUgh3aGVuSnNvbhIbCgl0aGVuX2pzb24YBiABKAxSCHRoZW5Kc29uEhgKB2'
    'VuYWJsZWQYByABKAhSB2VuYWJsZWQSGAoHdmVyc2lvbhgIIAEoDVIHdmVyc2lvbhI/CgpjcmVh'
    'dGVkX2F0GAkgASgLMiAuazFzMC5zeXN0ZW0uY29tbW9uLnYxLlRpbWVzdGFtcFIJY3JlYXRlZE'
    'F0Ej8KCnVwZGF0ZWRfYXQYCiABKAsyIC5rMXMwLnN5c3RlbS5jb21tb24udjEuVGltZXN0YW1w'
    'Ugl1cGRhdGVkQXQ=');

@$core.Deprecated('Use listRulesRequestDescriptor instead')
const ListRulesRequest$json = {
  '1': 'ListRulesRequest',
  '2': [
    {
      '1': 'pagination',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.common.v1.Pagination',
      '10': 'pagination'
    },
    {
      '1': 'rule_set_id',
      '3': 2,
      '4': 1,
      '5': 9,
      '9': 0,
      '10': 'ruleSetId',
      '17': true
    },
    {'1': 'domain', '3': 3, '4': 1, '5': 9, '9': 1, '10': 'domain', '17': true},
  ],
  '8': [
    {'1': '_rule_set_id'},
    {'1': '_domain'},
  ],
};

/// Descriptor for `ListRulesRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List listRulesRequestDescriptor = $convert.base64Decode(
    'ChBMaXN0UnVsZXNSZXF1ZXN0EkEKCnBhZ2luYXRpb24YASABKAsyIS5rMXMwLnN5c3RlbS5jb2'
    '1tb24udjEuUGFnaW5hdGlvblIKcGFnaW5hdGlvbhIjCgtydWxlX3NldF9pZBgCIAEoCUgAUgly'
    'dWxlU2V0SWSIAQESGwoGZG9tYWluGAMgASgJSAFSBmRvbWFpbogBAUIOCgxfcnVsZV9zZXRfaW'
    'RCCQoHX2RvbWFpbg==');

@$core.Deprecated('Use listRulesResponseDescriptor instead')
const ListRulesResponse$json = {
  '1': 'ListRulesResponse',
  '2': [
    {
      '1': 'rules',
      '3': 1,
      '4': 3,
      '5': 11,
      '6': '.k1s0.system.ruleengine.v1.Rule',
      '10': 'rules'
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

/// Descriptor for `ListRulesResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List listRulesResponseDescriptor = $convert.base64Decode(
    'ChFMaXN0UnVsZXNSZXNwb25zZRI1CgVydWxlcxgBIAMoCzIfLmsxczAuc3lzdGVtLnJ1bGVlbm'
    'dpbmUudjEuUnVsZVIFcnVsZXMSRwoKcGFnaW5hdGlvbhgCIAEoCzInLmsxczAuc3lzdGVtLmNv'
    'bW1vbi52MS5QYWdpbmF0aW9uUmVzdWx0UgpwYWdpbmF0aW9u');

@$core.Deprecated('Use getRuleRequestDescriptor instead')
const GetRuleRequest$json = {
  '1': 'GetRuleRequest',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 9, '10': 'id'},
  ],
};

/// Descriptor for `GetRuleRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List getRuleRequestDescriptor =
    $convert.base64Decode('Cg5HZXRSdWxlUmVxdWVzdBIOCgJpZBgBIAEoCVICaWQ=');

@$core.Deprecated('Use getRuleResponseDescriptor instead')
const GetRuleResponse$json = {
  '1': 'GetRuleResponse',
  '2': [
    {
      '1': 'rule',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.ruleengine.v1.Rule',
      '10': 'rule'
    },
  ],
};

/// Descriptor for `GetRuleResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List getRuleResponseDescriptor = $convert.base64Decode(
    'Cg9HZXRSdWxlUmVzcG9uc2USMwoEcnVsZRgBIAEoCzIfLmsxczAuc3lzdGVtLnJ1bGVlbmdpbm'
    'UudjEuUnVsZVIEcnVsZQ==');

@$core.Deprecated('Use createRuleRequestDescriptor instead')
const CreateRuleRequest$json = {
  '1': 'CreateRuleRequest',
  '2': [
    {'1': 'name', '3': 1, '4': 1, '5': 9, '10': 'name'},
    {'1': 'description', '3': 2, '4': 1, '5': 9, '10': 'description'},
    {'1': 'priority', '3': 3, '4': 1, '5': 5, '10': 'priority'},
    {'1': 'when_json', '3': 4, '4': 1, '5': 12, '10': 'whenJson'},
    {'1': 'then_json', '3': 5, '4': 1, '5': 12, '10': 'thenJson'},
  ],
};

/// Descriptor for `CreateRuleRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List createRuleRequestDescriptor = $convert.base64Decode(
    'ChFDcmVhdGVSdWxlUmVxdWVzdBISCgRuYW1lGAEgASgJUgRuYW1lEiAKC2Rlc2NyaXB0aW9uGA'
    'IgASgJUgtkZXNjcmlwdGlvbhIaCghwcmlvcml0eRgDIAEoBVIIcHJpb3JpdHkSGwoJd2hlbl9q'
    'c29uGAQgASgMUgh3aGVuSnNvbhIbCgl0aGVuX2pzb24YBSABKAxSCHRoZW5Kc29u');

@$core.Deprecated('Use createRuleResponseDescriptor instead')
const CreateRuleResponse$json = {
  '1': 'CreateRuleResponse',
  '2': [
    {
      '1': 'rule',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.ruleengine.v1.Rule',
      '10': 'rule'
    },
  ],
};

/// Descriptor for `CreateRuleResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List createRuleResponseDescriptor = $convert.base64Decode(
    'ChJDcmVhdGVSdWxlUmVzcG9uc2USMwoEcnVsZRgBIAEoCzIfLmsxczAuc3lzdGVtLnJ1bGVlbm'
    'dpbmUudjEuUnVsZVIEcnVsZQ==');

@$core.Deprecated('Use updateRuleRequestDescriptor instead')
const UpdateRuleRequest$json = {
  '1': 'UpdateRuleRequest',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 9, '10': 'id'},
    {
      '1': 'description',
      '3': 2,
      '4': 1,
      '5': 9,
      '9': 0,
      '10': 'description',
      '17': true
    },
    {
      '1': 'priority',
      '3': 3,
      '4': 1,
      '5': 5,
      '9': 1,
      '10': 'priority',
      '17': true
    },
    {
      '1': 'when_json',
      '3': 4,
      '4': 1,
      '5': 12,
      '9': 2,
      '10': 'whenJson',
      '17': true
    },
    {
      '1': 'then_json',
      '3': 5,
      '4': 1,
      '5': 12,
      '9': 3,
      '10': 'thenJson',
      '17': true
    },
    {
      '1': 'enabled',
      '3': 6,
      '4': 1,
      '5': 8,
      '9': 4,
      '10': 'enabled',
      '17': true
    },
  ],
  '8': [
    {'1': '_description'},
    {'1': '_priority'},
    {'1': '_when_json'},
    {'1': '_then_json'},
    {'1': '_enabled'},
  ],
};

/// Descriptor for `UpdateRuleRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List updateRuleRequestDescriptor = $convert.base64Decode(
    'ChFVcGRhdGVSdWxlUmVxdWVzdBIOCgJpZBgBIAEoCVICaWQSJQoLZGVzY3JpcHRpb24YAiABKA'
    'lIAFILZGVzY3JpcHRpb26IAQESHwoIcHJpb3JpdHkYAyABKAVIAVIIcHJpb3JpdHmIAQESIAoJ'
    'd2hlbl9qc29uGAQgASgMSAJSCHdoZW5Kc29uiAEBEiAKCXRoZW5fanNvbhgFIAEoDEgDUgh0aG'
    'VuSnNvbogBARIdCgdlbmFibGVkGAYgASgISARSB2VuYWJsZWSIAQFCDgoMX2Rlc2NyaXB0aW9u'
    'QgsKCV9wcmlvcml0eUIMCgpfd2hlbl9qc29uQgwKCl90aGVuX2pzb25CCgoIX2VuYWJsZWQ=');

@$core.Deprecated('Use updateRuleResponseDescriptor instead')
const UpdateRuleResponse$json = {
  '1': 'UpdateRuleResponse',
  '2': [
    {
      '1': 'rule',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.ruleengine.v1.Rule',
      '10': 'rule'
    },
  ],
};

/// Descriptor for `UpdateRuleResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List updateRuleResponseDescriptor = $convert.base64Decode(
    'ChJVcGRhdGVSdWxlUmVzcG9uc2USMwoEcnVsZRgBIAEoCzIfLmsxczAuc3lzdGVtLnJ1bGVlbm'
    'dpbmUudjEuUnVsZVIEcnVsZQ==');

@$core.Deprecated('Use deleteRuleRequestDescriptor instead')
const DeleteRuleRequest$json = {
  '1': 'DeleteRuleRequest',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 9, '10': 'id'},
  ],
};

/// Descriptor for `DeleteRuleRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List deleteRuleRequestDescriptor =
    $convert.base64Decode('ChFEZWxldGVSdWxlUmVxdWVzdBIOCgJpZBgBIAEoCVICaWQ=');

@$core.Deprecated('Use deleteRuleResponseDescriptor instead')
const DeleteRuleResponse$json = {
  '1': 'DeleteRuleResponse',
  '2': [
    {'1': 'success', '3': 1, '4': 1, '5': 8, '10': 'success'},
    {'1': 'message', '3': 2, '4': 1, '5': 9, '10': 'message'},
  ],
};

/// Descriptor for `DeleteRuleResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List deleteRuleResponseDescriptor = $convert.base64Decode(
    'ChJEZWxldGVSdWxlUmVzcG9uc2USGAoHc3VjY2VzcxgBIAEoCFIHc3VjY2VzcxIYCgdtZXNzYW'
    'dlGAIgASgJUgdtZXNzYWdl');

@$core.Deprecated('Use ruleSetDescriptor instead')
const RuleSet$json = {
  '1': 'RuleSet',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 9, '10': 'id'},
    {'1': 'name', '3': 2, '4': 1, '5': 9, '10': 'name'},
    {'1': 'description', '3': 3, '4': 1, '5': 9, '10': 'description'},
    {'1': 'domain', '3': 4, '4': 1, '5': 9, '10': 'domain'},
    {'1': 'evaluation_mode', '3': 5, '4': 1, '5': 9, '10': 'evaluationMode'},
    {
      '1': 'default_result_json',
      '3': 6,
      '4': 1,
      '5': 12,
      '10': 'defaultResultJson'
    },
    {'1': 'rule_ids', '3': 7, '4': 3, '5': 9, '10': 'ruleIds'},
    {'1': 'current_version', '3': 8, '4': 1, '5': 13, '10': 'currentVersion'},
    {'1': 'enabled', '3': 9, '4': 1, '5': 8, '10': 'enabled'},
    {
      '1': 'created_at',
      '3': 10,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.common.v1.Timestamp',
      '10': 'createdAt'
    },
    {
      '1': 'updated_at',
      '3': 11,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.common.v1.Timestamp',
      '10': 'updatedAt'
    },
  ],
};

/// Descriptor for `RuleSet`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List ruleSetDescriptor = $convert.base64Decode(
    'CgdSdWxlU2V0Eg4KAmlkGAEgASgJUgJpZBISCgRuYW1lGAIgASgJUgRuYW1lEiAKC2Rlc2NyaX'
    'B0aW9uGAMgASgJUgtkZXNjcmlwdGlvbhIWCgZkb21haW4YBCABKAlSBmRvbWFpbhInCg9ldmFs'
    'dWF0aW9uX21vZGUYBSABKAlSDmV2YWx1YXRpb25Nb2RlEi4KE2RlZmF1bHRfcmVzdWx0X2pzb2'
    '4YBiABKAxSEWRlZmF1bHRSZXN1bHRKc29uEhkKCHJ1bGVfaWRzGAcgAygJUgdydWxlSWRzEicK'
    'D2N1cnJlbnRfdmVyc2lvbhgIIAEoDVIOY3VycmVudFZlcnNpb24SGAoHZW5hYmxlZBgJIAEoCF'
    'IHZW5hYmxlZBI/CgpjcmVhdGVkX2F0GAogASgLMiAuazFzMC5zeXN0ZW0uY29tbW9uLnYxLlRp'
    'bWVzdGFtcFIJY3JlYXRlZEF0Ej8KCnVwZGF0ZWRfYXQYCyABKAsyIC5rMXMwLnN5c3RlbS5jb2'
    '1tb24udjEuVGltZXN0YW1wUgl1cGRhdGVkQXQ=');

@$core.Deprecated('Use listRuleSetsRequestDescriptor instead')
const ListRuleSetsRequest$json = {
  '1': 'ListRuleSetsRequest',
  '2': [
    {
      '1': 'pagination',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.common.v1.Pagination',
      '10': 'pagination'
    },
    {'1': 'domain', '3': 2, '4': 1, '5': 9, '9': 0, '10': 'domain', '17': true},
  ],
  '8': [
    {'1': '_domain'},
  ],
};

/// Descriptor for `ListRuleSetsRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List listRuleSetsRequestDescriptor = $convert.base64Decode(
    'ChNMaXN0UnVsZVNldHNSZXF1ZXN0EkEKCnBhZ2luYXRpb24YASABKAsyIS5rMXMwLnN5c3RlbS'
    '5jb21tb24udjEuUGFnaW5hdGlvblIKcGFnaW5hdGlvbhIbCgZkb21haW4YAiABKAlIAFIGZG9t'
    'YWluiAEBQgkKB19kb21haW4=');

@$core.Deprecated('Use listRuleSetsResponseDescriptor instead')
const ListRuleSetsResponse$json = {
  '1': 'ListRuleSetsResponse',
  '2': [
    {
      '1': 'rule_sets',
      '3': 1,
      '4': 3,
      '5': 11,
      '6': '.k1s0.system.ruleengine.v1.RuleSet',
      '10': 'ruleSets'
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

/// Descriptor for `ListRuleSetsResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List listRuleSetsResponseDescriptor = $convert.base64Decode(
    'ChRMaXN0UnVsZVNldHNSZXNwb25zZRI/CglydWxlX3NldHMYASADKAsyIi5rMXMwLnN5c3RlbS'
    '5ydWxlZW5naW5lLnYxLlJ1bGVTZXRSCHJ1bGVTZXRzEkcKCnBhZ2luYXRpb24YAiABKAsyJy5r'
    'MXMwLnN5c3RlbS5jb21tb24udjEuUGFnaW5hdGlvblJlc3VsdFIKcGFnaW5hdGlvbg==');

@$core.Deprecated('Use getRuleSetRequestDescriptor instead')
const GetRuleSetRequest$json = {
  '1': 'GetRuleSetRequest',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 9, '10': 'id'},
  ],
};

/// Descriptor for `GetRuleSetRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List getRuleSetRequestDescriptor =
    $convert.base64Decode('ChFHZXRSdWxlU2V0UmVxdWVzdBIOCgJpZBgBIAEoCVICaWQ=');

@$core.Deprecated('Use getRuleSetResponseDescriptor instead')
const GetRuleSetResponse$json = {
  '1': 'GetRuleSetResponse',
  '2': [
    {
      '1': 'rule_set',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.ruleengine.v1.RuleSet',
      '10': 'ruleSet'
    },
  ],
};

/// Descriptor for `GetRuleSetResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List getRuleSetResponseDescriptor = $convert.base64Decode(
    'ChJHZXRSdWxlU2V0UmVzcG9uc2USPQoIcnVsZV9zZXQYASABKAsyIi5rMXMwLnN5c3RlbS5ydW'
    'xlZW5naW5lLnYxLlJ1bGVTZXRSB3J1bGVTZXQ=');

@$core.Deprecated('Use createRuleSetRequestDescriptor instead')
const CreateRuleSetRequest$json = {
  '1': 'CreateRuleSetRequest',
  '2': [
    {'1': 'name', '3': 1, '4': 1, '5': 9, '10': 'name'},
    {'1': 'description', '3': 2, '4': 1, '5': 9, '10': 'description'},
    {'1': 'domain', '3': 3, '4': 1, '5': 9, '10': 'domain'},
    {'1': 'evaluation_mode', '3': 4, '4': 1, '5': 9, '10': 'evaluationMode'},
    {
      '1': 'default_result_json',
      '3': 5,
      '4': 1,
      '5': 12,
      '10': 'defaultResultJson'
    },
    {'1': 'rule_ids', '3': 6, '4': 3, '5': 9, '10': 'ruleIds'},
  ],
};

/// Descriptor for `CreateRuleSetRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List createRuleSetRequestDescriptor = $convert.base64Decode(
    'ChRDcmVhdGVSdWxlU2V0UmVxdWVzdBISCgRuYW1lGAEgASgJUgRuYW1lEiAKC2Rlc2NyaXB0aW'
    '9uGAIgASgJUgtkZXNjcmlwdGlvbhIWCgZkb21haW4YAyABKAlSBmRvbWFpbhInCg9ldmFsdWF0'
    'aW9uX21vZGUYBCABKAlSDmV2YWx1YXRpb25Nb2RlEi4KE2RlZmF1bHRfcmVzdWx0X2pzb24YBS'
    'ABKAxSEWRlZmF1bHRSZXN1bHRKc29uEhkKCHJ1bGVfaWRzGAYgAygJUgdydWxlSWRz');

@$core.Deprecated('Use createRuleSetResponseDescriptor instead')
const CreateRuleSetResponse$json = {
  '1': 'CreateRuleSetResponse',
  '2': [
    {
      '1': 'rule_set',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.ruleengine.v1.RuleSet',
      '10': 'ruleSet'
    },
  ],
};

/// Descriptor for `CreateRuleSetResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List createRuleSetResponseDescriptor = $convert.base64Decode(
    'ChVDcmVhdGVSdWxlU2V0UmVzcG9uc2USPQoIcnVsZV9zZXQYASABKAsyIi5rMXMwLnN5c3RlbS'
    '5ydWxlZW5naW5lLnYxLlJ1bGVTZXRSB3J1bGVTZXQ=');

@$core.Deprecated('Use updateRuleSetRequestDescriptor instead')
const UpdateRuleSetRequest$json = {
  '1': 'UpdateRuleSetRequest',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 9, '10': 'id'},
    {
      '1': 'description',
      '3': 2,
      '4': 1,
      '5': 9,
      '9': 0,
      '10': 'description',
      '17': true
    },
    {
      '1': 'evaluation_mode',
      '3': 3,
      '4': 1,
      '5': 9,
      '9': 1,
      '10': 'evaluationMode',
      '17': true
    },
    {
      '1': 'default_result_json',
      '3': 4,
      '4': 1,
      '5': 12,
      '9': 2,
      '10': 'defaultResultJson',
      '17': true
    },
    {'1': 'rule_ids', '3': 5, '4': 3, '5': 9, '10': 'ruleIds'},
    {
      '1': 'enabled',
      '3': 6,
      '4': 1,
      '5': 8,
      '9': 3,
      '10': 'enabled',
      '17': true
    },
  ],
  '8': [
    {'1': '_description'},
    {'1': '_evaluation_mode'},
    {'1': '_default_result_json'},
    {'1': '_enabled'},
  ],
};

/// Descriptor for `UpdateRuleSetRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List updateRuleSetRequestDescriptor = $convert.base64Decode(
    'ChRVcGRhdGVSdWxlU2V0UmVxdWVzdBIOCgJpZBgBIAEoCVICaWQSJQoLZGVzY3JpcHRpb24YAi'
    'ABKAlIAFILZGVzY3JpcHRpb26IAQESLAoPZXZhbHVhdGlvbl9tb2RlGAMgASgJSAFSDmV2YWx1'
    'YXRpb25Nb2RliAEBEjMKE2RlZmF1bHRfcmVzdWx0X2pzb24YBCABKAxIAlIRZGVmYXVsdFJlc3'
    'VsdEpzb26IAQESGQoIcnVsZV9pZHMYBSADKAlSB3J1bGVJZHMSHQoHZW5hYmxlZBgGIAEoCEgD'
    'UgdlbmFibGVkiAEBQg4KDF9kZXNjcmlwdGlvbkISChBfZXZhbHVhdGlvbl9tb2RlQhYKFF9kZW'
    'ZhdWx0X3Jlc3VsdF9qc29uQgoKCF9lbmFibGVk');

@$core.Deprecated('Use updateRuleSetResponseDescriptor instead')
const UpdateRuleSetResponse$json = {
  '1': 'UpdateRuleSetResponse',
  '2': [
    {
      '1': 'rule_set',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.ruleengine.v1.RuleSet',
      '10': 'ruleSet'
    },
  ],
};

/// Descriptor for `UpdateRuleSetResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List updateRuleSetResponseDescriptor = $convert.base64Decode(
    'ChVVcGRhdGVSdWxlU2V0UmVzcG9uc2USPQoIcnVsZV9zZXQYASABKAsyIi5rMXMwLnN5c3RlbS'
    '5ydWxlZW5naW5lLnYxLlJ1bGVTZXRSB3J1bGVTZXQ=');

@$core.Deprecated('Use deleteRuleSetRequestDescriptor instead')
const DeleteRuleSetRequest$json = {
  '1': 'DeleteRuleSetRequest',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 9, '10': 'id'},
  ],
};

/// Descriptor for `DeleteRuleSetRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List deleteRuleSetRequestDescriptor = $convert
    .base64Decode('ChREZWxldGVSdWxlU2V0UmVxdWVzdBIOCgJpZBgBIAEoCVICaWQ=');

@$core.Deprecated('Use deleteRuleSetResponseDescriptor instead')
const DeleteRuleSetResponse$json = {
  '1': 'DeleteRuleSetResponse',
  '2': [
    {'1': 'success', '3': 1, '4': 1, '5': 8, '10': 'success'},
    {'1': 'message', '3': 2, '4': 1, '5': 9, '10': 'message'},
  ],
};

/// Descriptor for `DeleteRuleSetResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List deleteRuleSetResponseDescriptor = $convert.base64Decode(
    'ChVEZWxldGVSdWxlU2V0UmVzcG9uc2USGAoHc3VjY2VzcxgBIAEoCFIHc3VjY2VzcxIYCgdtZX'
    'NzYWdlGAIgASgJUgdtZXNzYWdl');

@$core.Deprecated('Use publishRuleSetRequestDescriptor instead')
const PublishRuleSetRequest$json = {
  '1': 'PublishRuleSetRequest',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 9, '10': 'id'},
  ],
};

/// Descriptor for `PublishRuleSetRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List publishRuleSetRequestDescriptor = $convert
    .base64Decode('ChVQdWJsaXNoUnVsZVNldFJlcXVlc3QSDgoCaWQYASABKAlSAmlk');

@$core.Deprecated('Use publishRuleSetResponseDescriptor instead')
const PublishRuleSetResponse$json = {
  '1': 'PublishRuleSetResponse',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 9, '10': 'id'},
    {
      '1': 'published_version',
      '3': 2,
      '4': 1,
      '5': 13,
      '10': 'publishedVersion'
    },
    {'1': 'previous_version', '3': 3, '4': 1, '5': 13, '10': 'previousVersion'},
    {
      '1': 'published_at',
      '3': 4,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.common.v1.Timestamp',
      '10': 'publishedAt'
    },
  ],
};

/// Descriptor for `PublishRuleSetResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List publishRuleSetResponseDescriptor = $convert.base64Decode(
    'ChZQdWJsaXNoUnVsZVNldFJlc3BvbnNlEg4KAmlkGAEgASgJUgJpZBIrChFwdWJsaXNoZWRfdm'
    'Vyc2lvbhgCIAEoDVIQcHVibGlzaGVkVmVyc2lvbhIpChBwcmV2aW91c192ZXJzaW9uGAMgASgN'
    'Ug9wcmV2aW91c1ZlcnNpb24SQwoMcHVibGlzaGVkX2F0GAQgASgLMiAuazFzMC5zeXN0ZW0uY2'
    '9tbW9uLnYxLlRpbWVzdGFtcFILcHVibGlzaGVkQXQ=');

@$core.Deprecated('Use rollbackRuleSetRequestDescriptor instead')
const RollbackRuleSetRequest$json = {
  '1': 'RollbackRuleSetRequest',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 9, '10': 'id'},
  ],
};

/// Descriptor for `RollbackRuleSetRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List rollbackRuleSetRequestDescriptor = $convert
    .base64Decode('ChZSb2xsYmFja1J1bGVTZXRSZXF1ZXN0Eg4KAmlkGAEgASgJUgJpZA==');

@$core.Deprecated('Use rollbackRuleSetResponseDescriptor instead')
const RollbackRuleSetResponse$json = {
  '1': 'RollbackRuleSetResponse',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 9, '10': 'id'},
    {
      '1': 'rolled_back_to_version',
      '3': 2,
      '4': 1,
      '5': 13,
      '10': 'rolledBackToVersion'
    },
    {'1': 'previous_version', '3': 3, '4': 1, '5': 13, '10': 'previousVersion'},
    {
      '1': 'rolled_back_at',
      '3': 4,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.common.v1.Timestamp',
      '10': 'rolledBackAt'
    },
  ],
};

/// Descriptor for `RollbackRuleSetResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List rollbackRuleSetResponseDescriptor = $convert.base64Decode(
    'ChdSb2xsYmFja1J1bGVTZXRSZXNwb25zZRIOCgJpZBgBIAEoCVICaWQSMwoWcm9sbGVkX2JhY2'
    'tfdG9fdmVyc2lvbhgCIAEoDVITcm9sbGVkQmFja1RvVmVyc2lvbhIpChBwcmV2aW91c192ZXJz'
    'aW9uGAMgASgNUg9wcmV2aW91c1ZlcnNpb24SRgoOcm9sbGVkX2JhY2tfYXQYBCABKAsyIC5rMX'
    'MwLnN5c3RlbS5jb21tb24udjEuVGltZXN0YW1wUgxyb2xsZWRCYWNrQXQ=');

@$core.Deprecated('Use evaluateRequestDescriptor instead')
const EvaluateRequest$json = {
  '1': 'EvaluateRequest',
  '2': [
    {'1': 'rule_set', '3': 1, '4': 1, '5': 9, '10': 'ruleSet'},
    {'1': 'input_json', '3': 2, '4': 1, '5': 12, '10': 'inputJson'},
    {'1': 'context_json', '3': 3, '4': 1, '5': 12, '10': 'contextJson'},
  ],
};

/// Descriptor for `EvaluateRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List evaluateRequestDescriptor = $convert.base64Decode(
    'Cg9FdmFsdWF0ZVJlcXVlc3QSGQoIcnVsZV9zZXQYASABKAlSB3J1bGVTZXQSHQoKaW5wdXRfan'
    'NvbhgCIAEoDFIJaW5wdXRKc29uEiEKDGNvbnRleHRfanNvbhgDIAEoDFILY29udGV4dEpzb24=');

@$core.Deprecated('Use matchedRuleDescriptor instead')
const MatchedRule$json = {
  '1': 'MatchedRule',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 9, '10': 'id'},
    {'1': 'name', '3': 2, '4': 1, '5': 9, '10': 'name'},
    {'1': 'priority', '3': 3, '4': 1, '5': 5, '10': 'priority'},
    {'1': 'result_json', '3': 4, '4': 1, '5': 12, '10': 'resultJson'},
  ],
};

/// Descriptor for `MatchedRule`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List matchedRuleDescriptor = $convert.base64Decode(
    'CgtNYXRjaGVkUnVsZRIOCgJpZBgBIAEoCVICaWQSEgoEbmFtZRgCIAEoCVIEbmFtZRIaCghwcm'
    'lvcml0eRgDIAEoBVIIcHJpb3JpdHkSHwoLcmVzdWx0X2pzb24YBCABKAxSCnJlc3VsdEpzb24=');

@$core.Deprecated('Use evaluateResponseDescriptor instead')
const EvaluateResponse$json = {
  '1': 'EvaluateResponse',
  '2': [
    {'1': 'evaluation_id', '3': 1, '4': 1, '5': 9, '10': 'evaluationId'},
    {'1': 'rule_set', '3': 2, '4': 1, '5': 9, '10': 'ruleSet'},
    {'1': 'rule_set_version', '3': 3, '4': 1, '5': 13, '10': 'ruleSetVersion'},
    {
      '1': 'matched_rules',
      '3': 4,
      '4': 3,
      '5': 11,
      '6': '.k1s0.system.ruleengine.v1.MatchedRule',
      '10': 'matchedRules'
    },
    {'1': 'result_json', '3': 5, '4': 1, '5': 12, '10': 'resultJson'},
    {'1': 'default_applied', '3': 6, '4': 1, '5': 8, '10': 'defaultApplied'},
    {'1': 'cached', '3': 7, '4': 1, '5': 8, '10': 'cached'},
    {
      '1': 'evaluated_at',
      '3': 8,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.common.v1.Timestamp',
      '10': 'evaluatedAt'
    },
  ],
};

/// Descriptor for `EvaluateResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List evaluateResponseDescriptor = $convert.base64Decode(
    'ChBFdmFsdWF0ZVJlc3BvbnNlEiMKDWV2YWx1YXRpb25faWQYASABKAlSDGV2YWx1YXRpb25JZB'
    'IZCghydWxlX3NldBgCIAEoCVIHcnVsZVNldBIoChBydWxlX3NldF92ZXJzaW9uGAMgASgNUg5y'
    'dWxlU2V0VmVyc2lvbhJLCg1tYXRjaGVkX3J1bGVzGAQgAygLMiYuazFzMC5zeXN0ZW0ucnVsZW'
    'VuZ2luZS52MS5NYXRjaGVkUnVsZVIMbWF0Y2hlZFJ1bGVzEh8KC3Jlc3VsdF9qc29uGAUgASgM'
    'UgpyZXN1bHRKc29uEicKD2RlZmF1bHRfYXBwbGllZBgGIAEoCFIOZGVmYXVsdEFwcGxpZWQSFg'
    'oGY2FjaGVkGAcgASgIUgZjYWNoZWQSQwoMZXZhbHVhdGVkX2F0GAggASgLMiAuazFzMC5zeXN0'
    'ZW0uY29tbW9uLnYxLlRpbWVzdGFtcFILZXZhbHVhdGVkQXQ=');

@$core.Deprecated('Use evaluateDryRunRequestDescriptor instead')
const EvaluateDryRunRequest$json = {
  '1': 'EvaluateDryRunRequest',
  '2': [
    {'1': 'rule_set', '3': 1, '4': 1, '5': 9, '10': 'ruleSet'},
    {'1': 'input_json', '3': 2, '4': 1, '5': 12, '10': 'inputJson'},
    {'1': 'context_json', '3': 3, '4': 1, '5': 12, '10': 'contextJson'},
  ],
};

/// Descriptor for `EvaluateDryRunRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List evaluateDryRunRequestDescriptor = $convert.base64Decode(
    'ChVFdmFsdWF0ZURyeVJ1blJlcXVlc3QSGQoIcnVsZV9zZXQYASABKAlSB3J1bGVTZXQSHQoKaW'
    '5wdXRfanNvbhgCIAEoDFIJaW5wdXRKc29uEiEKDGNvbnRleHRfanNvbhgDIAEoDFILY29udGV4'
    'dEpzb24=');

@$core.Deprecated('Use evaluateDryRunResponseDescriptor instead')
const EvaluateDryRunResponse$json = {
  '1': 'EvaluateDryRunResponse',
  '2': [
    {'1': 'evaluation_id', '3': 1, '4': 1, '5': 9, '10': 'evaluationId'},
    {'1': 'rule_set', '3': 2, '4': 1, '5': 9, '10': 'ruleSet'},
    {'1': 'rule_set_version', '3': 3, '4': 1, '5': 13, '10': 'ruleSetVersion'},
    {
      '1': 'matched_rules',
      '3': 4,
      '4': 3,
      '5': 11,
      '6': '.k1s0.system.ruleengine.v1.MatchedRule',
      '10': 'matchedRules'
    },
    {'1': 'result_json', '3': 5, '4': 1, '5': 12, '10': 'resultJson'},
    {'1': 'default_applied', '3': 6, '4': 1, '5': 8, '10': 'defaultApplied'},
    {'1': 'cached', '3': 7, '4': 1, '5': 8, '10': 'cached'},
    {
      '1': 'evaluated_at',
      '3': 8,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.common.v1.Timestamp',
      '10': 'evaluatedAt'
    },
  ],
};

/// Descriptor for `EvaluateDryRunResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List evaluateDryRunResponseDescriptor = $convert.base64Decode(
    'ChZFdmFsdWF0ZURyeVJ1blJlc3BvbnNlEiMKDWV2YWx1YXRpb25faWQYASABKAlSDGV2YWx1YX'
    'Rpb25JZBIZCghydWxlX3NldBgCIAEoCVIHcnVsZVNldBIoChBydWxlX3NldF92ZXJzaW9uGAMg'
    'ASgNUg5ydWxlU2V0VmVyc2lvbhJLCg1tYXRjaGVkX3J1bGVzGAQgAygLMiYuazFzMC5zeXN0ZW'
    '0ucnVsZWVuZ2luZS52MS5NYXRjaGVkUnVsZVIMbWF0Y2hlZFJ1bGVzEh8KC3Jlc3VsdF9qc29u'
    'GAUgASgMUgpyZXN1bHRKc29uEicKD2RlZmF1bHRfYXBwbGllZBgGIAEoCFIOZGVmYXVsdEFwcG'
    'xpZWQSFgoGY2FjaGVkGAcgASgIUgZjYWNoZWQSQwoMZXZhbHVhdGVkX2F0GAggASgLMiAuazFz'
    'MC5zeXN0ZW0uY29tbW9uLnYxLlRpbWVzdGFtcFILZXZhbHVhdGVkQXQ=');
