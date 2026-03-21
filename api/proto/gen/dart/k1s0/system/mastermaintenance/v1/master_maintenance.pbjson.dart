// This is a generated file - do not edit.
//
// Generated from k1s0/system/mastermaintenance/v1/master_maintenance.proto.

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

@$core.Deprecated('Use createTableDefinitionRequestDescriptor instead')
const CreateTableDefinitionRequest$json = {
  '1': 'CreateTableDefinitionRequest',
  '2': [
    {
      '1': 'data',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.google.protobuf.Struct',
      '10': 'data'
    },
  ],
};

/// Descriptor for `CreateTableDefinitionRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List createTableDefinitionRequestDescriptor =
    $convert.base64Decode(
        'ChxDcmVhdGVUYWJsZURlZmluaXRpb25SZXF1ZXN0EisKBGRhdGEYASABKAsyFy5nb29nbGUucH'
        'JvdG9idWYuU3RydWN0UgRkYXRh');

@$core.Deprecated('Use createTableDefinitionResponseDescriptor instead')
const CreateTableDefinitionResponse$json = {
  '1': 'CreateTableDefinitionResponse',
  '2': [
    {
      '1': 'table',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.mastermaintenance.v1.GetTableDefinitionResponse',
      '10': 'table'
    },
  ],
};

/// Descriptor for `CreateTableDefinitionResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List createTableDefinitionResponseDescriptor =
    $convert.base64Decode(
        'Ch1DcmVhdGVUYWJsZURlZmluaXRpb25SZXNwb25zZRJSCgV0YWJsZRgBIAEoCzI8LmsxczAuc3'
        'lzdGVtLm1hc3Rlcm1haW50ZW5hbmNlLnYxLkdldFRhYmxlRGVmaW5pdGlvblJlc3BvbnNlUgV0'
        'YWJsZQ==');

@$core.Deprecated('Use updateTableDefinitionRequestDescriptor instead')
const UpdateTableDefinitionRequest$json = {
  '1': 'UpdateTableDefinitionRequest',
  '2': [
    {'1': 'table_name', '3': 1, '4': 1, '5': 9, '10': 'tableName'},
    {
      '1': 'data',
      '3': 2,
      '4': 1,
      '5': 11,
      '6': '.google.protobuf.Struct',
      '10': 'data'
    },
    {'1': 'domain_scope', '3': 3, '4': 1, '5': 9, '10': 'domainScope'},
  ],
};

/// Descriptor for `UpdateTableDefinitionRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List updateTableDefinitionRequestDescriptor =
    $convert.base64Decode(
        'ChxVcGRhdGVUYWJsZURlZmluaXRpb25SZXF1ZXN0Eh0KCnRhYmxlX25hbWUYASABKAlSCXRhYm'
        'xlTmFtZRIrCgRkYXRhGAIgASgLMhcuZ29vZ2xlLnByb3RvYnVmLlN0cnVjdFIEZGF0YRIhCgxk'
        'b21haW5fc2NvcGUYAyABKAlSC2RvbWFpblNjb3Bl');

@$core.Deprecated('Use updateTableDefinitionResponseDescriptor instead')
const UpdateTableDefinitionResponse$json = {
  '1': 'UpdateTableDefinitionResponse',
  '2': [
    {
      '1': 'table',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.mastermaintenance.v1.GetTableDefinitionResponse',
      '10': 'table'
    },
  ],
};

/// Descriptor for `UpdateTableDefinitionResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List updateTableDefinitionResponseDescriptor =
    $convert.base64Decode(
        'Ch1VcGRhdGVUYWJsZURlZmluaXRpb25SZXNwb25zZRJSCgV0YWJsZRgBIAEoCzI8LmsxczAuc3'
        'lzdGVtLm1hc3Rlcm1haW50ZW5hbmNlLnYxLkdldFRhYmxlRGVmaW5pdGlvblJlc3BvbnNlUgV0'
        'YWJsZQ==');

@$core.Deprecated('Use deleteTableDefinitionRequestDescriptor instead')
const DeleteTableDefinitionRequest$json = {
  '1': 'DeleteTableDefinitionRequest',
  '2': [
    {'1': 'table_name', '3': 1, '4': 1, '5': 9, '10': 'tableName'},
    {'1': 'domain_scope', '3': 2, '4': 1, '5': 9, '10': 'domainScope'},
  ],
};

/// Descriptor for `DeleteTableDefinitionRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List deleteTableDefinitionRequestDescriptor =
    $convert.base64Decode(
        'ChxEZWxldGVUYWJsZURlZmluaXRpb25SZXF1ZXN0Eh0KCnRhYmxlX25hbWUYASABKAlSCXRhYm'
        'xlTmFtZRIhCgxkb21haW5fc2NvcGUYAiABKAlSC2RvbWFpblNjb3Bl');

@$core.Deprecated('Use deleteTableDefinitionResponseDescriptor instead')
const DeleteTableDefinitionResponse$json = {
  '1': 'DeleteTableDefinitionResponse',
  '2': [
    {'1': 'success', '3': 1, '4': 1, '5': 8, '10': 'success'},
  ],
};

/// Descriptor for `DeleteTableDefinitionResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List deleteTableDefinitionResponseDescriptor =
    $convert.base64Decode(
        'Ch1EZWxldGVUYWJsZURlZmluaXRpb25SZXNwb25zZRIYCgdzdWNjZXNzGAEgASgIUgdzdWNjZX'
        'Nz');

@$core.Deprecated('Use getTableDefinitionRequestDescriptor instead')
const GetTableDefinitionRequest$json = {
  '1': 'GetTableDefinitionRequest',
  '2': [
    {'1': 'table_name', '3': 1, '4': 1, '5': 9, '10': 'tableName'},
    {'1': 'domain_scope', '3': 2, '4': 1, '5': 9, '10': 'domainScope'},
  ],
};

/// Descriptor for `GetTableDefinitionRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List getTableDefinitionRequestDescriptor =
    $convert.base64Decode(
        'ChlHZXRUYWJsZURlZmluaXRpb25SZXF1ZXN0Eh0KCnRhYmxlX25hbWUYASABKAlSCXRhYmxlTm'
        'FtZRIhCgxkb21haW5fc2NvcGUYAiABKAlSC2RvbWFpblNjb3Bl');

@$core.Deprecated('Use getTableDefinitionResponseDescriptor instead')
const GetTableDefinitionResponse$json = {
  '1': 'GetTableDefinitionResponse',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 9, '10': 'id'},
    {'1': 'name', '3': 2, '4': 1, '5': 9, '10': 'name'},
    {'1': 'schema_name', '3': 3, '4': 1, '5': 9, '10': 'schemaName'},
    {'1': 'display_name', '3': 4, '4': 1, '5': 9, '10': 'displayName'},
    {'1': 'description', '3': 5, '4': 1, '5': 9, '10': 'description'},
    {'1': 'allow_create', '3': 6, '4': 1, '5': 8, '10': 'allowCreate'},
    {'1': 'allow_update', '3': 7, '4': 1, '5': 8, '10': 'allowUpdate'},
    {'1': 'allow_delete', '3': 8, '4': 1, '5': 8, '10': 'allowDelete'},
    {
      '1': 'columns',
      '3': 9,
      '4': 3,
      '5': 11,
      '6': '.k1s0.system.mastermaintenance.v1.ColumnDefinition',
      '10': 'columns'
    },
    {
      '1': 'relationships',
      '3': 10,
      '4': 3,
      '5': 11,
      '6': '.k1s0.system.mastermaintenance.v1.TableRelationship',
      '10': 'relationships'
    },
    {'1': 'database_name', '3': 11, '4': 1, '5': 9, '10': 'databaseName'},
    {'1': 'category', '3': 12, '4': 1, '5': 9, '10': 'category'},
    {'1': 'is_active', '3': 13, '4': 1, '5': 8, '10': 'isActive'},
    {'1': 'sort_order', '3': 14, '4': 1, '5': 5, '10': 'sortOrder'},
    {'1': 'created_by', '3': 15, '4': 1, '5': 9, '10': 'createdBy'},
    {
      '1': 'created_at',
      '3': 16,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.common.v1.Timestamp',
      '10': 'createdAt'
    },
    {
      '1': 'updated_at',
      '3': 17,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.common.v1.Timestamp',
      '10': 'updatedAt'
    },
    {'1': 'domain_scope', '3': 18, '4': 1, '5': 9, '10': 'domainScope'},
    {'1': 'read_roles', '3': 19, '4': 3, '5': 9, '10': 'readRoles'},
    {'1': 'write_roles', '3': 20, '4': 3, '5': 9, '10': 'writeRoles'},
    {'1': 'admin_roles', '3': 21, '4': 3, '5': 9, '10': 'adminRoles'},
  ],
};

/// Descriptor for `GetTableDefinitionResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List getTableDefinitionResponseDescriptor = $convert.base64Decode(
    'ChpHZXRUYWJsZURlZmluaXRpb25SZXNwb25zZRIOCgJpZBgBIAEoCVICaWQSEgoEbmFtZRgCIA'
    'EoCVIEbmFtZRIfCgtzY2hlbWFfbmFtZRgDIAEoCVIKc2NoZW1hTmFtZRIhCgxkaXNwbGF5X25h'
    'bWUYBCABKAlSC2Rpc3BsYXlOYW1lEiAKC2Rlc2NyaXB0aW9uGAUgASgJUgtkZXNjcmlwdGlvbh'
    'IhCgxhbGxvd19jcmVhdGUYBiABKAhSC2FsbG93Q3JlYXRlEiEKDGFsbG93X3VwZGF0ZRgHIAEo'
    'CFILYWxsb3dVcGRhdGUSIQoMYWxsb3dfZGVsZXRlGAggASgIUgthbGxvd0RlbGV0ZRJMCgdjb2'
    'x1bW5zGAkgAygLMjIuazFzMC5zeXN0ZW0ubWFzdGVybWFpbnRlbmFuY2UudjEuQ29sdW1uRGVm'
    'aW5pdGlvblIHY29sdW1ucxJZCg1yZWxhdGlvbnNoaXBzGAogAygLMjMuazFzMC5zeXN0ZW0ubW'
    'FzdGVybWFpbnRlbmFuY2UudjEuVGFibGVSZWxhdGlvbnNoaXBSDXJlbGF0aW9uc2hpcHMSIwoN'
    'ZGF0YWJhc2VfbmFtZRgLIAEoCVIMZGF0YWJhc2VOYW1lEhoKCGNhdGVnb3J5GAwgASgJUghjYX'
    'RlZ29yeRIbCglpc19hY3RpdmUYDSABKAhSCGlzQWN0aXZlEh0KCnNvcnRfb3JkZXIYDiABKAVS'
    'CXNvcnRPcmRlchIdCgpjcmVhdGVkX2J5GA8gASgJUgljcmVhdGVkQnkSPwoKY3JlYXRlZF9hdB'
    'gQIAEoCzIgLmsxczAuc3lzdGVtLmNvbW1vbi52MS5UaW1lc3RhbXBSCWNyZWF0ZWRBdBI/Cgp1'
    'cGRhdGVkX2F0GBEgASgLMiAuazFzMC5zeXN0ZW0uY29tbW9uLnYxLlRpbWVzdGFtcFIJdXBkYX'
    'RlZEF0EiEKDGRvbWFpbl9zY29wZRgSIAEoCVILZG9tYWluU2NvcGUSHQoKcmVhZF9yb2xlcxgT'
    'IAMoCVIJcmVhZFJvbGVzEh8KC3dyaXRlX3JvbGVzGBQgAygJUgp3cml0ZVJvbGVzEh8KC2FkbW'
    'luX3JvbGVzGBUgAygJUgphZG1pblJvbGVz');

@$core.Deprecated('Use columnDefinitionDescriptor instead')
const ColumnDefinition$json = {
  '1': 'ColumnDefinition',
  '2': [
    {'1': 'column_name', '3': 1, '4': 1, '5': 9, '10': 'columnName'},
    {'1': 'display_name', '3': 2, '4': 1, '5': 9, '10': 'displayName'},
    {'1': 'data_type', '3': 3, '4': 1, '5': 9, '10': 'dataType'},
    {'1': 'is_primary_key', '3': 4, '4': 1, '5': 8, '10': 'isPrimaryKey'},
    {'1': 'is_nullable', '3': 5, '4': 1, '5': 8, '10': 'isNullable'},
    {'1': 'is_searchable', '3': 6, '4': 1, '5': 8, '10': 'isSearchable'},
    {'1': 'is_sortable', '3': 7, '4': 1, '5': 8, '10': 'isSortable'},
    {'1': 'is_filterable', '3': 8, '4': 1, '5': 8, '10': 'isFilterable'},
    {
      '1': 'is_visible_in_list',
      '3': 9,
      '4': 1,
      '5': 8,
      '10': 'isVisibleInList'
    },
    {
      '1': 'is_visible_in_form',
      '3': 10,
      '4': 1,
      '5': 8,
      '10': 'isVisibleInForm'
    },
    {'1': 'is_readonly', '3': 11, '4': 1, '5': 8, '10': 'isReadonly'},
    {'1': 'input_type', '3': 12, '4': 1, '5': 9, '10': 'inputType'},
    {'1': 'display_order', '3': 13, '4': 1, '5': 5, '10': 'displayOrder'},
    {'1': 'is_unique', '3': 14, '4': 1, '5': 8, '10': 'isUnique'},
    {'1': 'default_value', '3': 15, '4': 1, '5': 9, '10': 'defaultValue'},
    {
      '1': 'max_length',
      '3': 16,
      '4': 1,
      '5': 5,
      '9': 0,
      '10': 'maxLength',
      '17': true
    },
    {
      '1': 'min_value',
      '3': 17,
      '4': 1,
      '5': 1,
      '9': 1,
      '10': 'minValue',
      '17': true
    },
    {
      '1': 'max_value',
      '3': 18,
      '4': 1,
      '5': 1,
      '9': 2,
      '10': 'maxValue',
      '17': true
    },
    {'1': 'regex_pattern', '3': 19, '4': 1, '5': 9, '10': 'regexPattern'},
    {
      '1': 'select_options_json',
      '3': 20,
      '4': 1,
      '5': 9,
      '10': 'selectOptionsJson'
    },
  ],
  '8': [
    {'1': '_max_length'},
    {'1': '_min_value'},
    {'1': '_max_value'},
  ],
};

/// Descriptor for `ColumnDefinition`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List columnDefinitionDescriptor = $convert.base64Decode(
    'ChBDb2x1bW5EZWZpbml0aW9uEh8KC2NvbHVtbl9uYW1lGAEgASgJUgpjb2x1bW5OYW1lEiEKDG'
    'Rpc3BsYXlfbmFtZRgCIAEoCVILZGlzcGxheU5hbWUSGwoJZGF0YV90eXBlGAMgASgJUghkYXRh'
    'VHlwZRIkCg5pc19wcmltYXJ5X2tleRgEIAEoCFIMaXNQcmltYXJ5S2V5Eh8KC2lzX251bGxhYm'
    'xlGAUgASgIUgppc051bGxhYmxlEiMKDWlzX3NlYXJjaGFibGUYBiABKAhSDGlzU2VhcmNoYWJs'
    'ZRIfCgtpc19zb3J0YWJsZRgHIAEoCFIKaXNTb3J0YWJsZRIjCg1pc19maWx0ZXJhYmxlGAggAS'
    'gIUgxpc0ZpbHRlcmFibGUSKwoSaXNfdmlzaWJsZV9pbl9saXN0GAkgASgIUg9pc1Zpc2libGVJ'
    'bkxpc3QSKwoSaXNfdmlzaWJsZV9pbl9mb3JtGAogASgIUg9pc1Zpc2libGVJbkZvcm0SHwoLaX'
    'NfcmVhZG9ubHkYCyABKAhSCmlzUmVhZG9ubHkSHQoKaW5wdXRfdHlwZRgMIAEoCVIJaW5wdXRU'
    'eXBlEiMKDWRpc3BsYXlfb3JkZXIYDSABKAVSDGRpc3BsYXlPcmRlchIbCglpc191bmlxdWUYDi'
    'ABKAhSCGlzVW5pcXVlEiMKDWRlZmF1bHRfdmFsdWUYDyABKAlSDGRlZmF1bHRWYWx1ZRIiCgpt'
    'YXhfbGVuZ3RoGBAgASgFSABSCW1heExlbmd0aIgBARIgCgltaW5fdmFsdWUYESABKAFIAVIIbW'
    'luVmFsdWWIAQESIAoJbWF4X3ZhbHVlGBIgASgBSAJSCG1heFZhbHVliAEBEiMKDXJlZ2V4X3Bh'
    'dHRlcm4YEyABKAlSDHJlZ2V4UGF0dGVybhIuChNzZWxlY3Rfb3B0aW9uc19qc29uGBQgASgJUh'
    'FzZWxlY3RPcHRpb25zSnNvbkINCgtfbWF4X2xlbmd0aEIMCgpfbWluX3ZhbHVlQgwKCl9tYXhf'
    'dmFsdWU=');

@$core.Deprecated('Use tableRelationshipDescriptor instead')
const TableRelationship$json = {
  '1': 'TableRelationship',
  '2': [
    {'1': 'source_column', '3': 1, '4': 1, '5': 9, '10': 'sourceColumn'},
    {'1': 'target_table', '3': 2, '4': 1, '5': 9, '10': 'targetTable'},
    {'1': 'target_column', '3': 3, '4': 1, '5': 9, '10': 'targetColumn'},
    {
      '1': 'relationship_type',
      '3': 4,
      '4': 1,
      '5': 9,
      '10': 'relationshipType'
    },
    {'1': 'display_name', '3': 5, '4': 1, '5': 9, '10': 'displayName'},
    {'1': 'id', '3': 6, '4': 1, '5': 9, '10': 'id'},
    {'1': 'is_cascade_delete', '3': 7, '4': 1, '5': 8, '10': 'isCascadeDelete'},
    {'1': 'created_at', '3': 8, '4': 1, '5': 9, '10': 'createdAt'},
  ],
};

/// Descriptor for `TableRelationship`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List tableRelationshipDescriptor = $convert.base64Decode(
    'ChFUYWJsZVJlbGF0aW9uc2hpcBIjCg1zb3VyY2VfY29sdW1uGAEgASgJUgxzb3VyY2VDb2x1bW'
    '4SIQoMdGFyZ2V0X3RhYmxlGAIgASgJUgt0YXJnZXRUYWJsZRIjCg10YXJnZXRfY29sdW1uGAMg'
    'ASgJUgx0YXJnZXRDb2x1bW4SKwoRcmVsYXRpb25zaGlwX3R5cGUYBCABKAlSEHJlbGF0aW9uc2'
    'hpcFR5cGUSIQoMZGlzcGxheV9uYW1lGAUgASgJUgtkaXNwbGF5TmFtZRIOCgJpZBgGIAEoCVIC'
    'aWQSKgoRaXNfY2FzY2FkZV9kZWxldGUYByABKAhSD2lzQ2FzY2FkZURlbGV0ZRIdCgpjcmVhdG'
    'VkX2F0GAggASgJUgljcmVhdGVkQXQ=');

@$core.Deprecated('Use listTableDefinitionsRequestDescriptor instead')
const ListTableDefinitionsRequest$json = {
  '1': 'ListTableDefinitionsRequest',
  '2': [
    {'1': 'category', '3': 1, '4': 1, '5': 9, '10': 'category'},
    {'1': 'active_only', '3': 2, '4': 1, '5': 8, '10': 'activeOnly'},
    {
      '1': 'pagination',
      '3': 3,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.common.v1.Pagination',
      '10': 'pagination'
    },
    {'1': 'domain_scope', '3': 4, '4': 1, '5': 9, '10': 'domainScope'},
  ],
};

/// Descriptor for `ListTableDefinitionsRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List listTableDefinitionsRequestDescriptor = $convert.base64Decode(
    'ChtMaXN0VGFibGVEZWZpbml0aW9uc1JlcXVlc3QSGgoIY2F0ZWdvcnkYASABKAlSCGNhdGVnb3'
    'J5Eh8KC2FjdGl2ZV9vbmx5GAIgASgIUgphY3RpdmVPbmx5EkEKCnBhZ2luYXRpb24YAyABKAsy'
    'IS5rMXMwLnN5c3RlbS5jb21tb24udjEuUGFnaW5hdGlvblIKcGFnaW5hdGlvbhIhCgxkb21haW'
    '5fc2NvcGUYBCABKAlSC2RvbWFpblNjb3Bl');

@$core.Deprecated('Use listTableDefinitionsResponseDescriptor instead')
const ListTableDefinitionsResponse$json = {
  '1': 'ListTableDefinitionsResponse',
  '2': [
    {
      '1': 'tables',
      '3': 1,
      '4': 3,
      '5': 11,
      '6': '.k1s0.system.mastermaintenance.v1.GetTableDefinitionResponse',
      '10': 'tables'
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

/// Descriptor for `ListTableDefinitionsResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List listTableDefinitionsResponseDescriptor = $convert.base64Decode(
    'ChxMaXN0VGFibGVEZWZpbml0aW9uc1Jlc3BvbnNlElQKBnRhYmxlcxgBIAMoCzI8LmsxczAuc3'
    'lzdGVtLm1hc3Rlcm1haW50ZW5hbmNlLnYxLkdldFRhYmxlRGVmaW5pdGlvblJlc3BvbnNlUgZ0'
    'YWJsZXMSRwoKcGFnaW5hdGlvbhgCIAEoCzInLmsxczAuc3lzdGVtLmNvbW1vbi52MS5QYWdpbm'
    'F0aW9uUmVzdWx0UgpwYWdpbmF0aW9u');

@$core.Deprecated('Use getRecordRequestDescriptor instead')
const GetRecordRequest$json = {
  '1': 'GetRecordRequest',
  '2': [
    {'1': 'table_name', '3': 1, '4': 1, '5': 9, '10': 'tableName'},
    {'1': 'record_id', '3': 2, '4': 1, '5': 9, '10': 'recordId'},
    {'1': 'domain_scope', '3': 3, '4': 1, '5': 9, '10': 'domainScope'},
  ],
};

/// Descriptor for `GetRecordRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List getRecordRequestDescriptor = $convert.base64Decode(
    'ChBHZXRSZWNvcmRSZXF1ZXN0Eh0KCnRhYmxlX25hbWUYASABKAlSCXRhYmxlTmFtZRIbCglyZW'
    'NvcmRfaWQYAiABKAlSCHJlY29yZElkEiEKDGRvbWFpbl9zY29wZRgDIAEoCVILZG9tYWluU2Nv'
    'cGU=');

@$core.Deprecated('Use getRecordResponseDescriptor instead')
const GetRecordResponse$json = {
  '1': 'GetRecordResponse',
  '2': [
    {
      '1': 'data',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.google.protobuf.Struct',
      '10': 'data'
    },
    {
      '1': 'warnings',
      '3': 2,
      '4': 3,
      '5': 11,
      '6': '.k1s0.system.mastermaintenance.v1.ValidationWarning',
      '10': 'warnings'
    },
  ],
};

/// Descriptor for `GetRecordResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List getRecordResponseDescriptor = $convert.base64Decode(
    'ChFHZXRSZWNvcmRSZXNwb25zZRIrCgRkYXRhGAEgASgLMhcuZ29vZ2xlLnByb3RvYnVmLlN0cn'
    'VjdFIEZGF0YRJPCgh3YXJuaW5ncxgCIAMoCzIzLmsxczAuc3lzdGVtLm1hc3Rlcm1haW50ZW5h'
    'bmNlLnYxLlZhbGlkYXRpb25XYXJuaW5nUgh3YXJuaW5ncw==');

@$core.Deprecated('Use createRecordResponseDescriptor instead')
const CreateRecordResponse$json = {
  '1': 'CreateRecordResponse',
  '2': [
    {
      '1': 'data',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.google.protobuf.Struct',
      '10': 'data'
    },
    {
      '1': 'warnings',
      '3': 2,
      '4': 3,
      '5': 11,
      '6': '.k1s0.system.mastermaintenance.v1.ValidationWarning',
      '10': 'warnings'
    },
  ],
};

/// Descriptor for `CreateRecordResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List createRecordResponseDescriptor = $convert.base64Decode(
    'ChRDcmVhdGVSZWNvcmRSZXNwb25zZRIrCgRkYXRhGAEgASgLMhcuZ29vZ2xlLnByb3RvYnVmLl'
    'N0cnVjdFIEZGF0YRJPCgh3YXJuaW5ncxgCIAMoCzIzLmsxczAuc3lzdGVtLm1hc3Rlcm1haW50'
    'ZW5hbmNlLnYxLlZhbGlkYXRpb25XYXJuaW5nUgh3YXJuaW5ncw==');

@$core.Deprecated('Use updateRecordResponseDescriptor instead')
const UpdateRecordResponse$json = {
  '1': 'UpdateRecordResponse',
  '2': [
    {
      '1': 'data',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.google.protobuf.Struct',
      '10': 'data'
    },
    {
      '1': 'warnings',
      '3': 2,
      '4': 3,
      '5': 11,
      '6': '.k1s0.system.mastermaintenance.v1.ValidationWarning',
      '10': 'warnings'
    },
  ],
};

/// Descriptor for `UpdateRecordResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List updateRecordResponseDescriptor = $convert.base64Decode(
    'ChRVcGRhdGVSZWNvcmRSZXNwb25zZRIrCgRkYXRhGAEgASgLMhcuZ29vZ2xlLnByb3RvYnVmLl'
    'N0cnVjdFIEZGF0YRJPCgh3YXJuaW5ncxgCIAMoCzIzLmsxczAuc3lzdGVtLm1hc3Rlcm1haW50'
    'ZW5hbmNlLnYxLlZhbGlkYXRpb25XYXJuaW5nUgh3YXJuaW5ncw==');

@$core.Deprecated('Use listRecordsRequestDescriptor instead')
const ListRecordsRequest$json = {
  '1': 'ListRecordsRequest',
  '2': [
    {'1': 'table_name', '3': 1, '4': 1, '5': 9, '10': 'tableName'},
    {
      '1': 'pagination',
      '3': 2,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.common.v1.Pagination',
      '10': 'pagination'
    },
    {'1': 'sort', '3': 3, '4': 1, '5': 9, '10': 'sort'},
    {'1': 'filter', '3': 4, '4': 1, '5': 9, '10': 'filter'},
    {'1': 'search', '3': 5, '4': 1, '5': 9, '10': 'search'},
    {'1': 'domain_scope', '3': 6, '4': 1, '5': 9, '10': 'domainScope'},
  ],
};

/// Descriptor for `ListRecordsRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List listRecordsRequestDescriptor = $convert.base64Decode(
    'ChJMaXN0UmVjb3Jkc1JlcXVlc3QSHQoKdGFibGVfbmFtZRgBIAEoCVIJdGFibGVOYW1lEkEKCn'
    'BhZ2luYXRpb24YAiABKAsyIS5rMXMwLnN5c3RlbS5jb21tb24udjEuUGFnaW5hdGlvblIKcGFn'
    'aW5hdGlvbhISCgRzb3J0GAMgASgJUgRzb3J0EhYKBmZpbHRlchgEIAEoCVIGZmlsdGVyEhYKBn'
    'NlYXJjaBgFIAEoCVIGc2VhcmNoEiEKDGRvbWFpbl9zY29wZRgGIAEoCVILZG9tYWluU2NvcGU=');

@$core.Deprecated('Use listRecordsResponseDescriptor instead')
const ListRecordsResponse$json = {
  '1': 'ListRecordsResponse',
  '2': [
    {
      '1': 'records',
      '3': 1,
      '4': 3,
      '5': 11,
      '6': '.google.protobuf.Struct',
      '10': 'records'
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

/// Descriptor for `ListRecordsResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List listRecordsResponseDescriptor = $convert.base64Decode(
    'ChNMaXN0UmVjb3Jkc1Jlc3BvbnNlEjEKB3JlY29yZHMYASADKAsyFy5nb29nbGUucHJvdG9idW'
    'YuU3RydWN0UgdyZWNvcmRzEkcKCnBhZ2luYXRpb24YAiABKAsyJy5rMXMwLnN5c3RlbS5jb21t'
    'b24udjEuUGFnaW5hdGlvblJlc3VsdFIKcGFnaW5hdGlvbg==');

@$core.Deprecated('Use createRecordRequestDescriptor instead')
const CreateRecordRequest$json = {
  '1': 'CreateRecordRequest',
  '2': [
    {'1': 'table_name', '3': 1, '4': 1, '5': 9, '10': 'tableName'},
    {
      '1': 'data',
      '3': 2,
      '4': 1,
      '5': 11,
      '6': '.google.protobuf.Struct',
      '10': 'data'
    },
    {'1': 'domain_scope', '3': 3, '4': 1, '5': 9, '10': 'domainScope'},
  ],
};

/// Descriptor for `CreateRecordRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List createRecordRequestDescriptor = $convert.base64Decode(
    'ChNDcmVhdGVSZWNvcmRSZXF1ZXN0Eh0KCnRhYmxlX25hbWUYASABKAlSCXRhYmxlTmFtZRIrCg'
    'RkYXRhGAIgASgLMhcuZ29vZ2xlLnByb3RvYnVmLlN0cnVjdFIEZGF0YRIhCgxkb21haW5fc2Nv'
    'cGUYAyABKAlSC2RvbWFpblNjb3Bl');

@$core.Deprecated('Use updateRecordRequestDescriptor instead')
const UpdateRecordRequest$json = {
  '1': 'UpdateRecordRequest',
  '2': [
    {'1': 'table_name', '3': 1, '4': 1, '5': 9, '10': 'tableName'},
    {'1': 'record_id', '3': 2, '4': 1, '5': 9, '10': 'recordId'},
    {
      '1': 'data',
      '3': 3,
      '4': 1,
      '5': 11,
      '6': '.google.protobuf.Struct',
      '10': 'data'
    },
    {'1': 'domain_scope', '3': 4, '4': 1, '5': 9, '10': 'domainScope'},
  ],
};

/// Descriptor for `UpdateRecordRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List updateRecordRequestDescriptor = $convert.base64Decode(
    'ChNVcGRhdGVSZWNvcmRSZXF1ZXN0Eh0KCnRhYmxlX25hbWUYASABKAlSCXRhYmxlTmFtZRIbCg'
    'lyZWNvcmRfaWQYAiABKAlSCHJlY29yZElkEisKBGRhdGEYAyABKAsyFy5nb29nbGUucHJvdG9i'
    'dWYuU3RydWN0UgRkYXRhEiEKDGRvbWFpbl9zY29wZRgEIAEoCVILZG9tYWluU2NvcGU=');

@$core.Deprecated('Use deleteRecordRequestDescriptor instead')
const DeleteRecordRequest$json = {
  '1': 'DeleteRecordRequest',
  '2': [
    {'1': 'table_name', '3': 1, '4': 1, '5': 9, '10': 'tableName'},
    {'1': 'record_id', '3': 2, '4': 1, '5': 9, '10': 'recordId'},
    {'1': 'domain_scope', '3': 3, '4': 1, '5': 9, '10': 'domainScope'},
  ],
};

/// Descriptor for `DeleteRecordRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List deleteRecordRequestDescriptor = $convert.base64Decode(
    'ChNEZWxldGVSZWNvcmRSZXF1ZXN0Eh0KCnRhYmxlX25hbWUYASABKAlSCXRhYmxlTmFtZRIbCg'
    'lyZWNvcmRfaWQYAiABKAlSCHJlY29yZElkEiEKDGRvbWFpbl9zY29wZRgDIAEoCVILZG9tYWlu'
    'U2NvcGU=');

@$core.Deprecated('Use deleteRecordResponseDescriptor instead')
const DeleteRecordResponse$json = {
  '1': 'DeleteRecordResponse',
  '2': [
    {'1': 'success', '3': 1, '4': 1, '5': 8, '10': 'success'},
  ],
};

/// Descriptor for `DeleteRecordResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List deleteRecordResponseDescriptor =
    $convert.base64Decode(
        'ChREZWxldGVSZWNvcmRSZXNwb25zZRIYCgdzdWNjZXNzGAEgASgIUgdzdWNjZXNz');

@$core.Deprecated('Use checkConsistencyRequestDescriptor instead')
const CheckConsistencyRequest$json = {
  '1': 'CheckConsistencyRequest',
  '2': [
    {'1': 'table_name', '3': 1, '4': 1, '5': 9, '10': 'tableName'},
    {'1': 'rule_ids', '3': 2, '4': 3, '5': 9, '10': 'ruleIds'},
    {'1': 'domain_scope', '3': 3, '4': 1, '5': 9, '10': 'domainScope'},
  ],
};

/// Descriptor for `CheckConsistencyRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List checkConsistencyRequestDescriptor = $convert.base64Decode(
    'ChdDaGVja0NvbnNpc3RlbmN5UmVxdWVzdBIdCgp0YWJsZV9uYW1lGAEgASgJUgl0YWJsZU5hbW'
    'USGQoIcnVsZV9pZHMYAiADKAlSB3J1bGVJZHMSIQoMZG9tYWluX3Njb3BlGAMgASgJUgtkb21h'
    'aW5TY29wZQ==');

@$core.Deprecated('Use checkConsistencyResponseDescriptor instead')
const CheckConsistencyResponse$json = {
  '1': 'CheckConsistencyResponse',
  '2': [
    {
      '1': 'results',
      '3': 1,
      '4': 3,
      '5': 11,
      '6': '.k1s0.system.mastermaintenance.v1.ConsistencyResult',
      '10': 'results'
    },
    {'1': 'total_checked', '3': 2, '4': 1, '5': 5, '10': 'totalChecked'},
    {'1': 'error_count', '3': 3, '4': 1, '5': 5, '10': 'errorCount'},
    {'1': 'warning_count', '3': 4, '4': 1, '5': 5, '10': 'warningCount'},
  ],
};

/// Descriptor for `CheckConsistencyResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List checkConsistencyResponseDescriptor = $convert.base64Decode(
    'ChhDaGVja0NvbnNpc3RlbmN5UmVzcG9uc2USTQoHcmVzdWx0cxgBIAMoCzIzLmsxczAuc3lzdG'
    'VtLm1hc3Rlcm1haW50ZW5hbmNlLnYxLkNvbnNpc3RlbmN5UmVzdWx0UgdyZXN1bHRzEiMKDXRv'
    'dGFsX2NoZWNrZWQYAiABKAVSDHRvdGFsQ2hlY2tlZBIfCgtlcnJvcl9jb3VudBgDIAEoBVIKZX'
    'Jyb3JDb3VudBIjCg13YXJuaW5nX2NvdW50GAQgASgFUgx3YXJuaW5nQ291bnQ=');

@$core.Deprecated('Use createRuleRequestDescriptor instead')
const CreateRuleRequest$json = {
  '1': 'CreateRuleRequest',
  '2': [
    {
      '1': 'data',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.google.protobuf.Struct',
      '10': 'data'
    },
  ],
};

/// Descriptor for `CreateRuleRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List createRuleRequestDescriptor = $convert.base64Decode(
    'ChFDcmVhdGVSdWxlUmVxdWVzdBIrCgRkYXRhGAEgASgLMhcuZ29vZ2xlLnByb3RvYnVmLlN0cn'
    'VjdFIEZGF0YQ==');

@$core.Deprecated('Use createRuleResponseDescriptor instead')
const CreateRuleResponse$json = {
  '1': 'CreateRuleResponse',
  '2': [
    {
      '1': 'rule',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.mastermaintenance.v1.ConsistencyRule',
      '10': 'rule'
    },
  ],
};

/// Descriptor for `CreateRuleResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List createRuleResponseDescriptor = $convert.base64Decode(
    'ChJDcmVhdGVSdWxlUmVzcG9uc2USRQoEcnVsZRgBIAEoCzIxLmsxczAuc3lzdGVtLm1hc3Rlcm'
    '1haW50ZW5hbmNlLnYxLkNvbnNpc3RlbmN5UnVsZVIEcnVsZQ==');

@$core.Deprecated('Use getRuleRequestDescriptor instead')
const GetRuleRequest$json = {
  '1': 'GetRuleRequest',
  '2': [
    {'1': 'rule_id', '3': 1, '4': 1, '5': 9, '10': 'ruleId'},
  ],
};

/// Descriptor for `GetRuleRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List getRuleRequestDescriptor = $convert
    .base64Decode('Cg5HZXRSdWxlUmVxdWVzdBIXCgdydWxlX2lkGAEgASgJUgZydWxlSWQ=');

@$core.Deprecated('Use getRuleResponseDescriptor instead')
const GetRuleResponse$json = {
  '1': 'GetRuleResponse',
  '2': [
    {
      '1': 'rule',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.mastermaintenance.v1.ConsistencyRule',
      '10': 'rule'
    },
  ],
};

/// Descriptor for `GetRuleResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List getRuleResponseDescriptor = $convert.base64Decode(
    'Cg9HZXRSdWxlUmVzcG9uc2USRQoEcnVsZRgBIAEoCzIxLmsxczAuc3lzdGVtLm1hc3Rlcm1haW'
    '50ZW5hbmNlLnYxLkNvbnNpc3RlbmN5UnVsZVIEcnVsZQ==');

@$core.Deprecated('Use updateRuleRequestDescriptor instead')
const UpdateRuleRequest$json = {
  '1': 'UpdateRuleRequest',
  '2': [
    {'1': 'rule_id', '3': 1, '4': 1, '5': 9, '10': 'ruleId'},
    {
      '1': 'data',
      '3': 2,
      '4': 1,
      '5': 11,
      '6': '.google.protobuf.Struct',
      '10': 'data'
    },
  ],
};

/// Descriptor for `UpdateRuleRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List updateRuleRequestDescriptor = $convert.base64Decode(
    'ChFVcGRhdGVSdWxlUmVxdWVzdBIXCgdydWxlX2lkGAEgASgJUgZydWxlSWQSKwoEZGF0YRgCIA'
    'EoCzIXLmdvb2dsZS5wcm90b2J1Zi5TdHJ1Y3RSBGRhdGE=');

@$core.Deprecated('Use updateRuleResponseDescriptor instead')
const UpdateRuleResponse$json = {
  '1': 'UpdateRuleResponse',
  '2': [
    {
      '1': 'rule',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.mastermaintenance.v1.ConsistencyRule',
      '10': 'rule'
    },
  ],
};

/// Descriptor for `UpdateRuleResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List updateRuleResponseDescriptor = $convert.base64Decode(
    'ChJVcGRhdGVSdWxlUmVzcG9uc2USRQoEcnVsZRgBIAEoCzIxLmsxczAuc3lzdGVtLm1hc3Rlcm'
    '1haW50ZW5hbmNlLnYxLkNvbnNpc3RlbmN5UnVsZVIEcnVsZQ==');

@$core.Deprecated('Use deleteRuleRequestDescriptor instead')
const DeleteRuleRequest$json = {
  '1': 'DeleteRuleRequest',
  '2': [
    {'1': 'rule_id', '3': 1, '4': 1, '5': 9, '10': 'ruleId'},
  ],
};

/// Descriptor for `DeleteRuleRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List deleteRuleRequestDescriptor = $convert.base64Decode(
    'ChFEZWxldGVSdWxlUmVxdWVzdBIXCgdydWxlX2lkGAEgASgJUgZydWxlSWQ=');

@$core.Deprecated('Use deleteRuleResponseDescriptor instead')
const DeleteRuleResponse$json = {
  '1': 'DeleteRuleResponse',
  '2': [
    {'1': 'success', '3': 1, '4': 1, '5': 8, '10': 'success'},
  ],
};

/// Descriptor for `DeleteRuleResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List deleteRuleResponseDescriptor =
    $convert.base64Decode(
        'ChJEZWxldGVSdWxlUmVzcG9uc2USGAoHc3VjY2VzcxgBIAEoCFIHc3VjY2Vzcw==');

@$core.Deprecated('Use listRulesRequestDescriptor instead')
const ListRulesRequest$json = {
  '1': 'ListRulesRequest',
  '2': [
    {'1': 'table_name', '3': 1, '4': 1, '5': 9, '10': 'tableName'},
    {'1': 'rule_type', '3': 2, '4': 1, '5': 9, '10': 'ruleType'},
    {'1': 'severity', '3': 3, '4': 1, '5': 9, '10': 'severity'},
    {
      '1': 'pagination',
      '3': 4,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.common.v1.Pagination',
      '10': 'pagination'
    },
  ],
};

/// Descriptor for `ListRulesRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List listRulesRequestDescriptor = $convert.base64Decode(
    'ChBMaXN0UnVsZXNSZXF1ZXN0Eh0KCnRhYmxlX25hbWUYASABKAlSCXRhYmxlTmFtZRIbCglydW'
    'xlX3R5cGUYAiABKAlSCHJ1bGVUeXBlEhoKCHNldmVyaXR5GAMgASgJUghzZXZlcml0eRJBCgpw'
    'YWdpbmF0aW9uGAQgASgLMiEuazFzMC5zeXN0ZW0uY29tbW9uLnYxLlBhZ2luYXRpb25SCnBhZ2'
    'luYXRpb24=');

@$core.Deprecated('Use listRulesResponseDescriptor instead')
const ListRulesResponse$json = {
  '1': 'ListRulesResponse',
  '2': [
    {
      '1': 'rules',
      '3': 1,
      '4': 3,
      '5': 11,
      '6': '.k1s0.system.mastermaintenance.v1.ConsistencyRule',
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
    'ChFMaXN0UnVsZXNSZXNwb25zZRJHCgVydWxlcxgBIAMoCzIxLmsxczAuc3lzdGVtLm1hc3Rlcm'
    '1haW50ZW5hbmNlLnYxLkNvbnNpc3RlbmN5UnVsZVIFcnVsZXMSRwoKcGFnaW5hdGlvbhgCIAEo'
    'CzInLmsxczAuc3lzdGVtLmNvbW1vbi52MS5QYWdpbmF0aW9uUmVzdWx0UgpwYWdpbmF0aW9u');

@$core.Deprecated('Use executeRuleRequestDescriptor instead')
const ExecuteRuleRequest$json = {
  '1': 'ExecuteRuleRequest',
  '2': [
    {'1': 'rule_id', '3': 1, '4': 1, '5': 9, '10': 'ruleId'},
  ],
};

/// Descriptor for `ExecuteRuleRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List executeRuleRequestDescriptor =
    $convert.base64Decode(
        'ChJFeGVjdXRlUnVsZVJlcXVlc3QSFwoHcnVsZV9pZBgBIAEoCVIGcnVsZUlk');

@$core.Deprecated('Use executeRuleResponseDescriptor instead')
const ExecuteRuleResponse$json = {
  '1': 'ExecuteRuleResponse',
  '2': [
    {
      '1': 'results',
      '3': 1,
      '4': 3,
      '5': 11,
      '6': '.k1s0.system.mastermaintenance.v1.ConsistencyResult',
      '10': 'results'
    },
    {'1': 'total_checked', '3': 2, '4': 1, '5': 5, '10': 'totalChecked'},
    {'1': 'error_count', '3': 3, '4': 1, '5': 5, '10': 'errorCount'},
    {'1': 'warning_count', '3': 4, '4': 1, '5': 5, '10': 'warningCount'},
  ],
};

/// Descriptor for `ExecuteRuleResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List executeRuleResponseDescriptor = $convert.base64Decode(
    'ChNFeGVjdXRlUnVsZVJlc3BvbnNlEk0KB3Jlc3VsdHMYASADKAsyMy5rMXMwLnN5c3RlbS5tYX'
    'N0ZXJtYWludGVuYW5jZS52MS5Db25zaXN0ZW5jeVJlc3VsdFIHcmVzdWx0cxIjCg10b3RhbF9j'
    'aGVja2VkGAIgASgFUgx0b3RhbENoZWNrZWQSHwoLZXJyb3JfY291bnQYAyABKAVSCmVycm9yQ2'
    '91bnQSIwoNd2FybmluZ19jb3VudBgEIAEoBVIMd2FybmluZ0NvdW50');

@$core.Deprecated('Use consistencyResultDescriptor instead')
const ConsistencyResult$json = {
  '1': 'ConsistencyResult',
  '2': [
    {'1': 'rule_id', '3': 1, '4': 1, '5': 9, '10': 'ruleId'},
    {'1': 'rule_name', '3': 2, '4': 1, '5': 9, '10': 'ruleName'},
    {'1': 'severity', '3': 3, '4': 1, '5': 9, '10': 'severity'},
    {'1': 'passed', '3': 4, '4': 1, '5': 8, '10': 'passed'},
    {'1': 'message', '3': 5, '4': 1, '5': 9, '10': 'message'},
    {
      '1': 'affected_record_ids',
      '3': 6,
      '4': 3,
      '5': 9,
      '10': 'affectedRecordIds'
    },
  ],
};

/// Descriptor for `ConsistencyResult`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List consistencyResultDescriptor = $convert.base64Decode(
    'ChFDb25zaXN0ZW5jeVJlc3VsdBIXCgdydWxlX2lkGAEgASgJUgZydWxlSWQSGwoJcnVsZV9uYW'
    '1lGAIgASgJUghydWxlTmFtZRIaCghzZXZlcml0eRgDIAEoCVIIc2V2ZXJpdHkSFgoGcGFzc2Vk'
    'GAQgASgIUgZwYXNzZWQSGAoHbWVzc2FnZRgFIAEoCVIHbWVzc2FnZRIuChNhZmZlY3RlZF9yZW'
    'NvcmRfaWRzGAYgAygJUhFhZmZlY3RlZFJlY29yZElkcw==');

@$core.Deprecated('Use consistencyRuleDescriptor instead')
const ConsistencyRule$json = {
  '1': 'ConsistencyRule',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 9, '10': 'id'},
    {'1': 'name', '3': 2, '4': 1, '5': 9, '10': 'name'},
    {'1': 'description', '3': 3, '4': 1, '5': 9, '10': 'description'},
    {'1': 'rule_type', '3': 4, '4': 1, '5': 9, '10': 'ruleType'},
    {'1': 'severity', '3': 5, '4': 1, '5': 9, '10': 'severity'},
    {'1': 'is_active', '3': 6, '4': 1, '5': 8, '10': 'isActive'},
    {'1': 'source_table_id', '3': 7, '4': 1, '5': 9, '10': 'sourceTableId'},
    {
      '1': 'evaluation_timing',
      '3': 8,
      '4': 1,
      '5': 9,
      '10': 'evaluationTiming'
    },
    {
      '1': 'error_message_template',
      '3': 9,
      '4': 1,
      '5': 9,
      '10': 'errorMessageTemplate'
    },
    {'1': 'zen_rule_json', '3': 10, '4': 1, '5': 9, '10': 'zenRuleJson'},
    {'1': 'created_by', '3': 11, '4': 1, '5': 9, '10': 'createdBy'},
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
};

/// Descriptor for `ConsistencyRule`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List consistencyRuleDescriptor = $convert.base64Decode(
    'Cg9Db25zaXN0ZW5jeVJ1bGUSDgoCaWQYASABKAlSAmlkEhIKBG5hbWUYAiABKAlSBG5hbWUSIA'
    'oLZGVzY3JpcHRpb24YAyABKAlSC2Rlc2NyaXB0aW9uEhsKCXJ1bGVfdHlwZRgEIAEoCVIIcnVs'
    'ZVR5cGUSGgoIc2V2ZXJpdHkYBSABKAlSCHNldmVyaXR5EhsKCWlzX2FjdGl2ZRgGIAEoCFIIaX'
    'NBY3RpdmUSJgoPc291cmNlX3RhYmxlX2lkGAcgASgJUg1zb3VyY2VUYWJsZUlkEisKEWV2YWx1'
    'YXRpb25fdGltaW5nGAggASgJUhBldmFsdWF0aW9uVGltaW5nEjQKFmVycm9yX21lc3NhZ2VfdG'
    'VtcGxhdGUYCSABKAlSFGVycm9yTWVzc2FnZVRlbXBsYXRlEiIKDXplbl9ydWxlX2pzb24YCiAB'
    'KAlSC3plblJ1bGVKc29uEh0KCmNyZWF0ZWRfYnkYCyABKAlSCWNyZWF0ZWRCeRI/CgpjcmVhdG'
    'VkX2F0GAwgASgLMiAuazFzMC5zeXN0ZW0uY29tbW9uLnYxLlRpbWVzdGFtcFIJY3JlYXRlZEF0'
    'Ej8KCnVwZGF0ZWRfYXQYDSABKAsyIC5rMXMwLnN5c3RlbS5jb21tb24udjEuVGltZXN0YW1wUg'
    'l1cGRhdGVkQXQ=');

@$core.Deprecated('Use validationWarningDescriptor instead')
const ValidationWarning$json = {
  '1': 'ValidationWarning',
  '2': [
    {'1': 'rule_name', '3': 1, '4': 1, '5': 9, '10': 'ruleName'},
    {'1': 'message', '3': 2, '4': 1, '5': 9, '10': 'message'},
    {'1': 'severity', '3': 3, '4': 1, '5': 9, '10': 'severity'},
  ],
};

/// Descriptor for `ValidationWarning`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List validationWarningDescriptor = $convert.base64Decode(
    'ChFWYWxpZGF0aW9uV2FybmluZxIbCglydWxlX25hbWUYASABKAlSCHJ1bGVOYW1lEhgKB21lc3'
    'NhZ2UYAiABKAlSB21lc3NhZ2USGgoIc2V2ZXJpdHkYAyABKAlSCHNldmVyaXR5');

@$core.Deprecated('Use getTableSchemaRequestDescriptor instead')
const GetTableSchemaRequest$json = {
  '1': 'GetTableSchemaRequest',
  '2': [
    {'1': 'table_name', '3': 1, '4': 1, '5': 9, '10': 'tableName'},
  ],
};

/// Descriptor for `GetTableSchemaRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List getTableSchemaRequestDescriptor = $convert.base64Decode(
    'ChVHZXRUYWJsZVNjaGVtYVJlcXVlc3QSHQoKdGFibGVfbmFtZRgBIAEoCVIJdGFibGVOYW1l');

@$core.Deprecated('Use getTableSchemaResponseDescriptor instead')
const GetTableSchemaResponse$json = {
  '1': 'GetTableSchemaResponse',
  '2': [
    {'1': 'json_schema', '3': 1, '4': 1, '5': 9, '10': 'jsonSchema'},
  ],
};

/// Descriptor for `GetTableSchemaResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List getTableSchemaResponseDescriptor =
    $convert.base64Decode(
        'ChZHZXRUYWJsZVNjaGVtYVJlc3BvbnNlEh8KC2pzb25fc2NoZW1hGAEgASgJUgpqc29uU2NoZW'
        '1h');

@$core.Deprecated('Use listColumnsRequestDescriptor instead')
const ListColumnsRequest$json = {
  '1': 'ListColumnsRequest',
  '2': [
    {'1': 'table_name', '3': 1, '4': 1, '5': 9, '10': 'tableName'},
  ],
};

/// Descriptor for `ListColumnsRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List listColumnsRequestDescriptor =
    $convert.base64Decode(
        'ChJMaXN0Q29sdW1uc1JlcXVlc3QSHQoKdGFibGVfbmFtZRgBIAEoCVIJdGFibGVOYW1l');

@$core.Deprecated('Use listColumnsResponseDescriptor instead')
const ListColumnsResponse$json = {
  '1': 'ListColumnsResponse',
  '2': [
    {
      '1': 'columns',
      '3': 1,
      '4': 3,
      '5': 11,
      '6': '.k1s0.system.mastermaintenance.v1.ColumnDefinition',
      '10': 'columns'
    },
  ],
};

/// Descriptor for `ListColumnsResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List listColumnsResponseDescriptor = $convert.base64Decode(
    'ChNMaXN0Q29sdW1uc1Jlc3BvbnNlEkwKB2NvbHVtbnMYASADKAsyMi5rMXMwLnN5c3RlbS5tYX'
    'N0ZXJtYWludGVuYW5jZS52MS5Db2x1bW5EZWZpbml0aW9uUgdjb2x1bW5z');

@$core.Deprecated('Use createColumnsRequestDescriptor instead')
const CreateColumnsRequest$json = {
  '1': 'CreateColumnsRequest',
  '2': [
    {'1': 'table_name', '3': 1, '4': 1, '5': 9, '10': 'tableName'},
    {
      '1': 'columns',
      '3': 2,
      '4': 3,
      '5': 11,
      '6': '.google.protobuf.Struct',
      '10': 'columns'
    },
  ],
};

/// Descriptor for `CreateColumnsRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List createColumnsRequestDescriptor = $convert.base64Decode(
    'ChRDcmVhdGVDb2x1bW5zUmVxdWVzdBIdCgp0YWJsZV9uYW1lGAEgASgJUgl0YWJsZU5hbWUSMQ'
    'oHY29sdW1ucxgCIAMoCzIXLmdvb2dsZS5wcm90b2J1Zi5TdHJ1Y3RSB2NvbHVtbnM=');

@$core.Deprecated('Use createColumnsResponseDescriptor instead')
const CreateColumnsResponse$json = {
  '1': 'CreateColumnsResponse',
  '2': [
    {
      '1': 'columns',
      '3': 1,
      '4': 3,
      '5': 11,
      '6': '.k1s0.system.mastermaintenance.v1.ColumnDefinition',
      '10': 'columns'
    },
  ],
};

/// Descriptor for `CreateColumnsResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List createColumnsResponseDescriptor = $convert.base64Decode(
    'ChVDcmVhdGVDb2x1bW5zUmVzcG9uc2USTAoHY29sdW1ucxgBIAMoCzIyLmsxczAuc3lzdGVtLm'
    '1hc3Rlcm1haW50ZW5hbmNlLnYxLkNvbHVtbkRlZmluaXRpb25SB2NvbHVtbnM=');

@$core.Deprecated('Use updateColumnRequestDescriptor instead')
const UpdateColumnRequest$json = {
  '1': 'UpdateColumnRequest',
  '2': [
    {'1': 'table_name', '3': 1, '4': 1, '5': 9, '10': 'tableName'},
    {'1': 'column_name', '3': 2, '4': 1, '5': 9, '10': 'columnName'},
    {
      '1': 'data',
      '3': 3,
      '4': 1,
      '5': 11,
      '6': '.google.protobuf.Struct',
      '10': 'data'
    },
  ],
};

/// Descriptor for `UpdateColumnRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List updateColumnRequestDescriptor = $convert.base64Decode(
    'ChNVcGRhdGVDb2x1bW5SZXF1ZXN0Eh0KCnRhYmxlX25hbWUYASABKAlSCXRhYmxlTmFtZRIfCg'
    'tjb2x1bW5fbmFtZRgCIAEoCVIKY29sdW1uTmFtZRIrCgRkYXRhGAMgASgLMhcuZ29vZ2xlLnBy'
    'b3RvYnVmLlN0cnVjdFIEZGF0YQ==');

@$core.Deprecated('Use updateColumnResponseDescriptor instead')
const UpdateColumnResponse$json = {
  '1': 'UpdateColumnResponse',
  '2': [
    {
      '1': 'column',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.mastermaintenance.v1.ColumnDefinition',
      '10': 'column'
    },
  ],
};

/// Descriptor for `UpdateColumnResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List updateColumnResponseDescriptor = $convert.base64Decode(
    'ChRVcGRhdGVDb2x1bW5SZXNwb25zZRJKCgZjb2x1bW4YASABKAsyMi5rMXMwLnN5c3RlbS5tYX'
    'N0ZXJtYWludGVuYW5jZS52MS5Db2x1bW5EZWZpbml0aW9uUgZjb2x1bW4=');

@$core.Deprecated('Use deleteColumnRequestDescriptor instead')
const DeleteColumnRequest$json = {
  '1': 'DeleteColumnRequest',
  '2': [
    {'1': 'table_name', '3': 1, '4': 1, '5': 9, '10': 'tableName'},
    {'1': 'column_name', '3': 2, '4': 1, '5': 9, '10': 'columnName'},
  ],
};

/// Descriptor for `DeleteColumnRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List deleteColumnRequestDescriptor = $convert.base64Decode(
    'ChNEZWxldGVDb2x1bW5SZXF1ZXN0Eh0KCnRhYmxlX25hbWUYASABKAlSCXRhYmxlTmFtZRIfCg'
    'tjb2x1bW5fbmFtZRgCIAEoCVIKY29sdW1uTmFtZQ==');

@$core.Deprecated('Use deleteColumnResponseDescriptor instead')
const DeleteColumnResponse$json = {
  '1': 'DeleteColumnResponse',
  '2': [
    {'1': 'success', '3': 1, '4': 1, '5': 8, '10': 'success'},
  ],
};

/// Descriptor for `DeleteColumnResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List deleteColumnResponseDescriptor =
    $convert.base64Decode(
        'ChREZWxldGVDb2x1bW5SZXNwb25zZRIYCgdzdWNjZXNzGAEgASgIUgdzdWNjZXNz');

@$core.Deprecated('Use listRelationshipsRequestDescriptor instead')
const ListRelationshipsRequest$json = {
  '1': 'ListRelationshipsRequest',
};

/// Descriptor for `ListRelationshipsRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List listRelationshipsRequestDescriptor =
    $convert.base64Decode('ChhMaXN0UmVsYXRpb25zaGlwc1JlcXVlc3Q=');

@$core.Deprecated('Use listRelationshipsResponseDescriptor instead')
const ListRelationshipsResponse$json = {
  '1': 'ListRelationshipsResponse',
  '2': [
    {
      '1': 'relationships',
      '3': 1,
      '4': 3,
      '5': 11,
      '6': '.k1s0.system.mastermaintenance.v1.TableRelationship',
      '10': 'relationships'
    },
  ],
};

/// Descriptor for `ListRelationshipsResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List listRelationshipsResponseDescriptor = $convert.base64Decode(
    'ChlMaXN0UmVsYXRpb25zaGlwc1Jlc3BvbnNlElkKDXJlbGF0aW9uc2hpcHMYASADKAsyMy5rMX'
    'MwLnN5c3RlbS5tYXN0ZXJtYWludGVuYW5jZS52MS5UYWJsZVJlbGF0aW9uc2hpcFINcmVsYXRp'
    'b25zaGlwcw==');

@$core.Deprecated('Use createRelationshipRequestDescriptor instead')
const CreateRelationshipRequest$json = {
  '1': 'CreateRelationshipRequest',
  '2': [
    {
      '1': 'data',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.google.protobuf.Struct',
      '10': 'data'
    },
  ],
};

/// Descriptor for `CreateRelationshipRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List createRelationshipRequestDescriptor =
    $convert.base64Decode(
        'ChlDcmVhdGVSZWxhdGlvbnNoaXBSZXF1ZXN0EisKBGRhdGEYASABKAsyFy5nb29nbGUucHJvdG'
        '9idWYuU3RydWN0UgRkYXRh');

@$core.Deprecated('Use createRelationshipResponseDescriptor instead')
const CreateRelationshipResponse$json = {
  '1': 'CreateRelationshipResponse',
  '2': [
    {
      '1': 'relationship',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.mastermaintenance.v1.TableRelationship',
      '10': 'relationship'
    },
  ],
};

/// Descriptor for `CreateRelationshipResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List createRelationshipResponseDescriptor =
    $convert.base64Decode(
        'ChpDcmVhdGVSZWxhdGlvbnNoaXBSZXNwb25zZRJXCgxyZWxhdGlvbnNoaXAYASABKAsyMy5rMX'
        'MwLnN5c3RlbS5tYXN0ZXJtYWludGVuYW5jZS52MS5UYWJsZVJlbGF0aW9uc2hpcFIMcmVsYXRp'
        'b25zaGlw');

@$core.Deprecated('Use updateRelationshipRequestDescriptor instead')
const UpdateRelationshipRequest$json = {
  '1': 'UpdateRelationshipRequest',
  '2': [
    {'1': 'relationship_id', '3': 1, '4': 1, '5': 9, '10': 'relationshipId'},
    {
      '1': 'data',
      '3': 2,
      '4': 1,
      '5': 11,
      '6': '.google.protobuf.Struct',
      '10': 'data'
    },
  ],
};

/// Descriptor for `UpdateRelationshipRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List updateRelationshipRequestDescriptor = $convert.base64Decode(
    'ChlVcGRhdGVSZWxhdGlvbnNoaXBSZXF1ZXN0EicKD3JlbGF0aW9uc2hpcF9pZBgBIAEoCVIOcm'
    'VsYXRpb25zaGlwSWQSKwoEZGF0YRgCIAEoCzIXLmdvb2dsZS5wcm90b2J1Zi5TdHJ1Y3RSBGRh'
    'dGE=');

@$core.Deprecated('Use updateRelationshipResponseDescriptor instead')
const UpdateRelationshipResponse$json = {
  '1': 'UpdateRelationshipResponse',
  '2': [
    {
      '1': 'relationship',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.mastermaintenance.v1.TableRelationship',
      '10': 'relationship'
    },
  ],
};

/// Descriptor for `UpdateRelationshipResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List updateRelationshipResponseDescriptor =
    $convert.base64Decode(
        'ChpVcGRhdGVSZWxhdGlvbnNoaXBSZXNwb25zZRJXCgxyZWxhdGlvbnNoaXAYASABKAsyMy5rMX'
        'MwLnN5c3RlbS5tYXN0ZXJtYWludGVuYW5jZS52MS5UYWJsZVJlbGF0aW9uc2hpcFIMcmVsYXRp'
        'b25zaGlw');

@$core.Deprecated('Use deleteRelationshipRequestDescriptor instead')
const DeleteRelationshipRequest$json = {
  '1': 'DeleteRelationshipRequest',
  '2': [
    {'1': 'relationship_id', '3': 1, '4': 1, '5': 9, '10': 'relationshipId'},
  ],
};

/// Descriptor for `DeleteRelationshipRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List deleteRelationshipRequestDescriptor =
    $convert.base64Decode(
        'ChlEZWxldGVSZWxhdGlvbnNoaXBSZXF1ZXN0EicKD3JlbGF0aW9uc2hpcF9pZBgBIAEoCVIOcm'
        'VsYXRpb25zaGlwSWQ=');

@$core.Deprecated('Use deleteRelationshipResponseDescriptor instead')
const DeleteRelationshipResponse$json = {
  '1': 'DeleteRelationshipResponse',
  '2': [
    {'1': 'success', '3': 1, '4': 1, '5': 8, '10': 'success'},
  ],
};

/// Descriptor for `DeleteRelationshipResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List deleteRelationshipResponseDescriptor =
    $convert.base64Decode(
        'ChpEZWxldGVSZWxhdGlvbnNoaXBSZXNwb25zZRIYCgdzdWNjZXNzGAEgASgIUgdzdWNjZXNz');

@$core.Deprecated('Use importRecordsRequestDescriptor instead')
const ImportRecordsRequest$json = {
  '1': 'ImportRecordsRequest',
  '2': [
    {'1': 'table_name', '3': 1, '4': 1, '5': 9, '10': 'tableName'},
    {
      '1': 'data',
      '3': 2,
      '4': 1,
      '5': 11,
      '6': '.google.protobuf.Struct',
      '10': 'data'
    },
  ],
};

/// Descriptor for `ImportRecordsRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List importRecordsRequestDescriptor = $convert.base64Decode(
    'ChRJbXBvcnRSZWNvcmRzUmVxdWVzdBIdCgp0YWJsZV9uYW1lGAEgASgJUgl0YWJsZU5hbWUSKw'
    'oEZGF0YRgCIAEoCzIXLmdvb2dsZS5wcm90b2J1Zi5TdHJ1Y3RSBGRhdGE=');

@$core.Deprecated('Use importRecordsResponseDescriptor instead')
const ImportRecordsResponse$json = {
  '1': 'ImportRecordsResponse',
  '2': [
    {
      '1': 'import_job',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.mastermaintenance.v1.ImportJob',
      '10': 'importJob'
    },
  ],
};

/// Descriptor for `ImportRecordsResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List importRecordsResponseDescriptor = $convert.base64Decode(
    'ChVJbXBvcnRSZWNvcmRzUmVzcG9uc2USSgoKaW1wb3J0X2pvYhgBIAEoCzIrLmsxczAuc3lzdG'
    'VtLm1hc3Rlcm1haW50ZW5hbmNlLnYxLkltcG9ydEpvYlIJaW1wb3J0Sm9i');

@$core.Deprecated('Use exportRecordsRequestDescriptor instead')
const ExportRecordsRequest$json = {
  '1': 'ExportRecordsRequest',
  '2': [
    {'1': 'table_name', '3': 1, '4': 1, '5': 9, '10': 'tableName'},
  ],
};

/// Descriptor for `ExportRecordsRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List exportRecordsRequestDescriptor = $convert.base64Decode(
    'ChRFeHBvcnRSZWNvcmRzUmVxdWVzdBIdCgp0YWJsZV9uYW1lGAEgASgJUgl0YWJsZU5hbWU=');

@$core.Deprecated('Use exportRecordsResponseDescriptor instead')
const ExportRecordsResponse$json = {
  '1': 'ExportRecordsResponse',
  '2': [
    {
      '1': 'data',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.google.protobuf.Struct',
      '10': 'data'
    },
  ],
};

/// Descriptor for `ExportRecordsResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List exportRecordsResponseDescriptor = $convert.base64Decode(
    'ChVFeHBvcnRSZWNvcmRzUmVzcG9uc2USKwoEZGF0YRgBIAEoCzIXLmdvb2dsZS5wcm90b2J1Zi'
    '5TdHJ1Y3RSBGRhdGE=');

@$core.Deprecated('Use getImportJobRequestDescriptor instead')
const GetImportJobRequest$json = {
  '1': 'GetImportJobRequest',
  '2': [
    {'1': 'import_job_id', '3': 1, '4': 1, '5': 9, '10': 'importJobId'},
  ],
};

/// Descriptor for `GetImportJobRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List getImportJobRequestDescriptor = $convert.base64Decode(
    'ChNHZXRJbXBvcnRKb2JSZXF1ZXN0EiIKDWltcG9ydF9qb2JfaWQYASABKAlSC2ltcG9ydEpvYk'
    'lk');

@$core.Deprecated('Use getImportJobResponseDescriptor instead')
const GetImportJobResponse$json = {
  '1': 'GetImportJobResponse',
  '2': [
    {
      '1': 'import_job',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.mastermaintenance.v1.ImportJob',
      '10': 'importJob'
    },
  ],
};

/// Descriptor for `GetImportJobResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List getImportJobResponseDescriptor = $convert.base64Decode(
    'ChRHZXRJbXBvcnRKb2JSZXNwb25zZRJKCgppbXBvcnRfam9iGAEgASgLMisuazFzMC5zeXN0ZW'
    '0ubWFzdGVybWFpbnRlbmFuY2UudjEuSW1wb3J0Sm9iUglpbXBvcnRKb2I=');

@$core.Deprecated('Use importJobDescriptor instead')
const ImportJob$json = {
  '1': 'ImportJob',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 9, '10': 'id'},
    {'1': 'table_id', '3': 2, '4': 1, '5': 9, '10': 'tableId'},
    {'1': 'file_name', '3': 3, '4': 1, '5': 9, '10': 'fileName'},
    {'1': 'status', '3': 4, '4': 1, '5': 9, '10': 'status'},
    {'1': 'total_rows', '3': 5, '4': 1, '5': 5, '10': 'totalRows'},
    {'1': 'processed_rows', '3': 6, '4': 1, '5': 5, '10': 'processedRows'},
    {'1': 'error_rows', '3': 7, '4': 1, '5': 5, '10': 'errorRows'},
    {
      '1': 'error_details_json',
      '3': 8,
      '4': 1,
      '5': 9,
      '10': 'errorDetailsJson'
    },
    {'1': 'started_by', '3': 9, '4': 1, '5': 9, '10': 'startedBy'},
    {'1': 'started_at', '3': 10, '4': 1, '5': 9, '10': 'startedAt'},
    {
      '1': 'completed_at',
      '3': 11,
      '4': 1,
      '5': 9,
      '9': 0,
      '10': 'completedAt',
      '17': true
    },
  ],
  '8': [
    {'1': '_completed_at'},
  ],
};

/// Descriptor for `ImportJob`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List importJobDescriptor = $convert.base64Decode(
    'CglJbXBvcnRKb2ISDgoCaWQYASABKAlSAmlkEhkKCHRhYmxlX2lkGAIgASgJUgd0YWJsZUlkEh'
    'sKCWZpbGVfbmFtZRgDIAEoCVIIZmlsZU5hbWUSFgoGc3RhdHVzGAQgASgJUgZzdGF0dXMSHQoK'
    'dG90YWxfcm93cxgFIAEoBVIJdG90YWxSb3dzEiUKDnByb2Nlc3NlZF9yb3dzGAYgASgFUg1wcm'
    '9jZXNzZWRSb3dzEh0KCmVycm9yX3Jvd3MYByABKAVSCWVycm9yUm93cxIsChJlcnJvcl9kZXRh'
    'aWxzX2pzb24YCCABKAlSEGVycm9yRGV0YWlsc0pzb24SHQoKc3RhcnRlZF9ieRgJIAEoCVIJc3'
    'RhcnRlZEJ5Eh0KCnN0YXJ0ZWRfYXQYCiABKAlSCXN0YXJ0ZWRBdBImCgxjb21wbGV0ZWRfYXQY'
    'CyABKAlIAFILY29tcGxldGVkQXSIAQFCDwoNX2NvbXBsZXRlZF9hdA==');

@$core.Deprecated('Use listDisplayConfigsRequestDescriptor instead')
const ListDisplayConfigsRequest$json = {
  '1': 'ListDisplayConfigsRequest',
  '2': [
    {'1': 'table_name', '3': 1, '4': 1, '5': 9, '10': 'tableName'},
  ],
};

/// Descriptor for `ListDisplayConfigsRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List listDisplayConfigsRequestDescriptor =
    $convert.base64Decode(
        'ChlMaXN0RGlzcGxheUNvbmZpZ3NSZXF1ZXN0Eh0KCnRhYmxlX25hbWUYASABKAlSCXRhYmxlTm'
        'FtZQ==');

@$core.Deprecated('Use listDisplayConfigsResponseDescriptor instead')
const ListDisplayConfigsResponse$json = {
  '1': 'ListDisplayConfigsResponse',
  '2': [
    {
      '1': 'display_configs',
      '3': 1,
      '4': 3,
      '5': 11,
      '6': '.k1s0.system.mastermaintenance.v1.DisplayConfig',
      '10': 'displayConfigs'
    },
  ],
};

/// Descriptor for `ListDisplayConfigsResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List listDisplayConfigsResponseDescriptor =
    $convert.base64Decode(
        'ChpMaXN0RGlzcGxheUNvbmZpZ3NSZXNwb25zZRJYCg9kaXNwbGF5X2NvbmZpZ3MYASADKAsyLy'
        '5rMXMwLnN5c3RlbS5tYXN0ZXJtYWludGVuYW5jZS52MS5EaXNwbGF5Q29uZmlnUg5kaXNwbGF5'
        'Q29uZmlncw==');

@$core.Deprecated('Use getDisplayConfigRequestDescriptor instead')
const GetDisplayConfigRequest$json = {
  '1': 'GetDisplayConfigRequest',
  '2': [
    {'1': 'table_name', '3': 1, '4': 1, '5': 9, '10': 'tableName'},
    {'1': 'display_config_id', '3': 2, '4': 1, '5': 9, '10': 'displayConfigId'},
  ],
};

/// Descriptor for `GetDisplayConfigRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List getDisplayConfigRequestDescriptor =
    $convert.base64Decode(
        'ChdHZXREaXNwbGF5Q29uZmlnUmVxdWVzdBIdCgp0YWJsZV9uYW1lGAEgASgJUgl0YWJsZU5hbW'
        'USKgoRZGlzcGxheV9jb25maWdfaWQYAiABKAlSD2Rpc3BsYXlDb25maWdJZA==');

@$core.Deprecated('Use getDisplayConfigResponseDescriptor instead')
const GetDisplayConfigResponse$json = {
  '1': 'GetDisplayConfigResponse',
  '2': [
    {
      '1': 'display_config',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.mastermaintenance.v1.DisplayConfig',
      '10': 'displayConfig'
    },
  ],
};

/// Descriptor for `GetDisplayConfigResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List getDisplayConfigResponseDescriptor = $convert.base64Decode(
    'ChhHZXREaXNwbGF5Q29uZmlnUmVzcG9uc2USVgoOZGlzcGxheV9jb25maWcYASABKAsyLy5rMX'
    'MwLnN5c3RlbS5tYXN0ZXJtYWludGVuYW5jZS52MS5EaXNwbGF5Q29uZmlnUg1kaXNwbGF5Q29u'
    'Zmln');

@$core.Deprecated('Use createDisplayConfigRequestDescriptor instead')
const CreateDisplayConfigRequest$json = {
  '1': 'CreateDisplayConfigRequest',
  '2': [
    {'1': 'table_name', '3': 1, '4': 1, '5': 9, '10': 'tableName'},
    {
      '1': 'data',
      '3': 2,
      '4': 1,
      '5': 11,
      '6': '.google.protobuf.Struct',
      '10': 'data'
    },
  ],
};

/// Descriptor for `CreateDisplayConfigRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List createDisplayConfigRequestDescriptor =
    $convert.base64Decode(
        'ChpDcmVhdGVEaXNwbGF5Q29uZmlnUmVxdWVzdBIdCgp0YWJsZV9uYW1lGAEgASgJUgl0YWJsZU'
        '5hbWUSKwoEZGF0YRgCIAEoCzIXLmdvb2dsZS5wcm90b2J1Zi5TdHJ1Y3RSBGRhdGE=');

@$core.Deprecated('Use createDisplayConfigResponseDescriptor instead')
const CreateDisplayConfigResponse$json = {
  '1': 'CreateDisplayConfigResponse',
  '2': [
    {
      '1': 'display_config',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.mastermaintenance.v1.DisplayConfig',
      '10': 'displayConfig'
    },
  ],
};

/// Descriptor for `CreateDisplayConfigResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List createDisplayConfigResponseDescriptor =
    $convert.base64Decode(
        'ChtDcmVhdGVEaXNwbGF5Q29uZmlnUmVzcG9uc2USVgoOZGlzcGxheV9jb25maWcYASABKAsyLy'
        '5rMXMwLnN5c3RlbS5tYXN0ZXJtYWludGVuYW5jZS52MS5EaXNwbGF5Q29uZmlnUg1kaXNwbGF5'
        'Q29uZmln');

@$core.Deprecated('Use updateDisplayConfigRequestDescriptor instead')
const UpdateDisplayConfigRequest$json = {
  '1': 'UpdateDisplayConfigRequest',
  '2': [
    {'1': 'table_name', '3': 1, '4': 1, '5': 9, '10': 'tableName'},
    {'1': 'display_config_id', '3': 2, '4': 1, '5': 9, '10': 'displayConfigId'},
    {
      '1': 'data',
      '3': 3,
      '4': 1,
      '5': 11,
      '6': '.google.protobuf.Struct',
      '10': 'data'
    },
  ],
};

/// Descriptor for `UpdateDisplayConfigRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List updateDisplayConfigRequestDescriptor =
    $convert.base64Decode(
        'ChpVcGRhdGVEaXNwbGF5Q29uZmlnUmVxdWVzdBIdCgp0YWJsZV9uYW1lGAEgASgJUgl0YWJsZU'
        '5hbWUSKgoRZGlzcGxheV9jb25maWdfaWQYAiABKAlSD2Rpc3BsYXlDb25maWdJZBIrCgRkYXRh'
        'GAMgASgLMhcuZ29vZ2xlLnByb3RvYnVmLlN0cnVjdFIEZGF0YQ==');

@$core.Deprecated('Use updateDisplayConfigResponseDescriptor instead')
const UpdateDisplayConfigResponse$json = {
  '1': 'UpdateDisplayConfigResponse',
  '2': [
    {
      '1': 'display_config',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.mastermaintenance.v1.DisplayConfig',
      '10': 'displayConfig'
    },
  ],
};

/// Descriptor for `UpdateDisplayConfigResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List updateDisplayConfigResponseDescriptor =
    $convert.base64Decode(
        'ChtVcGRhdGVEaXNwbGF5Q29uZmlnUmVzcG9uc2USVgoOZGlzcGxheV9jb25maWcYASABKAsyLy'
        '5rMXMwLnN5c3RlbS5tYXN0ZXJtYWludGVuYW5jZS52MS5EaXNwbGF5Q29uZmlnUg1kaXNwbGF5'
        'Q29uZmln');

@$core.Deprecated('Use deleteDisplayConfigRequestDescriptor instead')
const DeleteDisplayConfigRequest$json = {
  '1': 'DeleteDisplayConfigRequest',
  '2': [
    {'1': 'table_name', '3': 1, '4': 1, '5': 9, '10': 'tableName'},
    {'1': 'display_config_id', '3': 2, '4': 1, '5': 9, '10': 'displayConfigId'},
  ],
};

/// Descriptor for `DeleteDisplayConfigRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List deleteDisplayConfigRequestDescriptor =
    $convert.base64Decode(
        'ChpEZWxldGVEaXNwbGF5Q29uZmlnUmVxdWVzdBIdCgp0YWJsZV9uYW1lGAEgASgJUgl0YWJsZU'
        '5hbWUSKgoRZGlzcGxheV9jb25maWdfaWQYAiABKAlSD2Rpc3BsYXlDb25maWdJZA==');

@$core.Deprecated('Use deleteDisplayConfigResponseDescriptor instead')
const DeleteDisplayConfigResponse$json = {
  '1': 'DeleteDisplayConfigResponse',
  '2': [
    {'1': 'success', '3': 1, '4': 1, '5': 8, '10': 'success'},
  ],
};

/// Descriptor for `DeleteDisplayConfigResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List deleteDisplayConfigResponseDescriptor =
    $convert.base64Decode(
        'ChtEZWxldGVEaXNwbGF5Q29uZmlnUmVzcG9uc2USGAoHc3VjY2VzcxgBIAEoCFIHc3VjY2Vzcw'
        '==');

@$core.Deprecated('Use displayConfigDescriptor instead')
const DisplayConfig$json = {
  '1': 'DisplayConfig',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 9, '10': 'id'},
    {'1': 'table_id', '3': 2, '4': 1, '5': 9, '10': 'tableId'},
    {'1': 'config_type', '3': 3, '4': 1, '5': 9, '10': 'configType'},
    {'1': 'config_json', '3': 4, '4': 1, '5': 9, '10': 'configJson'},
    {'1': 'is_default', '3': 5, '4': 1, '5': 8, '10': 'isDefault'},
    {'1': 'created_by', '3': 6, '4': 1, '5': 9, '10': 'createdBy'},
    {'1': 'created_at', '3': 7, '4': 1, '5': 9, '10': 'createdAt'},
    {'1': 'updated_at', '3': 8, '4': 1, '5': 9, '10': 'updatedAt'},
  ],
};

/// Descriptor for `DisplayConfig`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List displayConfigDescriptor = $convert.base64Decode(
    'Cg1EaXNwbGF5Q29uZmlnEg4KAmlkGAEgASgJUgJpZBIZCgh0YWJsZV9pZBgCIAEoCVIHdGFibG'
    'VJZBIfCgtjb25maWdfdHlwZRgDIAEoCVIKY29uZmlnVHlwZRIfCgtjb25maWdfanNvbhgEIAEo'
    'CVIKY29uZmlnSnNvbhIdCgppc19kZWZhdWx0GAUgASgIUglpc0RlZmF1bHQSHQoKY3JlYXRlZF'
    '9ieRgGIAEoCVIJY3JlYXRlZEJ5Eh0KCmNyZWF0ZWRfYXQYByABKAlSCWNyZWF0ZWRBdBIdCgp1'
    'cGRhdGVkX2F0GAggASgJUgl1cGRhdGVkQXQ=');

@$core.Deprecated('Use listTableAuditLogsRequestDescriptor instead')
const ListTableAuditLogsRequest$json = {
  '1': 'ListTableAuditLogsRequest',
  '2': [
    {'1': 'table_name', '3': 1, '4': 1, '5': 9, '10': 'tableName'},
    {
      '1': 'pagination',
      '3': 2,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.common.v1.Pagination',
      '10': 'pagination'
    },
    {'1': 'domain_scope', '3': 3, '4': 1, '5': 9, '10': 'domainScope'},
  ],
};

/// Descriptor for `ListTableAuditLogsRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List listTableAuditLogsRequestDescriptor = $convert.base64Decode(
    'ChlMaXN0VGFibGVBdWRpdExvZ3NSZXF1ZXN0Eh0KCnRhYmxlX25hbWUYASABKAlSCXRhYmxlTm'
    'FtZRJBCgpwYWdpbmF0aW9uGAIgASgLMiEuazFzMC5zeXN0ZW0uY29tbW9uLnYxLlBhZ2luYXRp'
    'b25SCnBhZ2luYXRpb24SIQoMZG9tYWluX3Njb3BlGAMgASgJUgtkb21haW5TY29wZQ==');

@$core.Deprecated('Use listTableAuditLogsResponseDescriptor instead')
const ListTableAuditLogsResponse$json = {
  '1': 'ListTableAuditLogsResponse',
  '2': [
    {
      '1': 'logs',
      '3': 1,
      '4': 3,
      '5': 11,
      '6': '.k1s0.system.mastermaintenance.v1.AuditLogEntry',
      '10': 'logs'
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

/// Descriptor for `ListTableAuditLogsResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List listTableAuditLogsResponseDescriptor = $convert.base64Decode(
    'ChpMaXN0VGFibGVBdWRpdExvZ3NSZXNwb25zZRJDCgRsb2dzGAEgAygLMi8uazFzMC5zeXN0ZW'
    '0ubWFzdGVybWFpbnRlbmFuY2UudjEuQXVkaXRMb2dFbnRyeVIEbG9ncxJHCgpwYWdpbmF0aW9u'
    'GAIgASgLMicuazFzMC5zeXN0ZW0uY29tbW9uLnYxLlBhZ2luYXRpb25SZXN1bHRSCnBhZ2luYX'
    'Rpb24=');

@$core.Deprecated('Use listRecordAuditLogsRequestDescriptor instead')
const ListRecordAuditLogsRequest$json = {
  '1': 'ListRecordAuditLogsRequest',
  '2': [
    {'1': 'table_name', '3': 1, '4': 1, '5': 9, '10': 'tableName'},
    {'1': 'record_id', '3': 2, '4': 1, '5': 9, '10': 'recordId'},
    {
      '1': 'pagination',
      '3': 3,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.common.v1.Pagination',
      '10': 'pagination'
    },
    {'1': 'domain_scope', '3': 4, '4': 1, '5': 9, '10': 'domainScope'},
  ],
};

/// Descriptor for `ListRecordAuditLogsRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List listRecordAuditLogsRequestDescriptor = $convert.base64Decode(
    'ChpMaXN0UmVjb3JkQXVkaXRMb2dzUmVxdWVzdBIdCgp0YWJsZV9uYW1lGAEgASgJUgl0YWJsZU'
    '5hbWUSGwoJcmVjb3JkX2lkGAIgASgJUghyZWNvcmRJZBJBCgpwYWdpbmF0aW9uGAMgASgLMiEu'
    'azFzMC5zeXN0ZW0uY29tbW9uLnYxLlBhZ2luYXRpb25SCnBhZ2luYXRpb24SIQoMZG9tYWluX3'
    'Njb3BlGAQgASgJUgtkb21haW5TY29wZQ==');

@$core.Deprecated('Use listRecordAuditLogsResponseDescriptor instead')
const ListRecordAuditLogsResponse$json = {
  '1': 'ListRecordAuditLogsResponse',
  '2': [
    {
      '1': 'logs',
      '3': 1,
      '4': 3,
      '5': 11,
      '6': '.k1s0.system.mastermaintenance.v1.AuditLogEntry',
      '10': 'logs'
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

/// Descriptor for `ListRecordAuditLogsResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List listRecordAuditLogsResponseDescriptor = $convert.base64Decode(
    'ChtMaXN0UmVjb3JkQXVkaXRMb2dzUmVzcG9uc2USQwoEbG9ncxgBIAMoCzIvLmsxczAuc3lzdG'
    'VtLm1hc3Rlcm1haW50ZW5hbmNlLnYxLkF1ZGl0TG9nRW50cnlSBGxvZ3MSRwoKcGFnaW5hdGlv'
    'bhgCIAEoCzInLmsxczAuc3lzdGVtLmNvbW1vbi52MS5QYWdpbmF0aW9uUmVzdWx0UgpwYWdpbm'
    'F0aW9u');

@$core.Deprecated('Use auditLogEntryDescriptor instead')
const AuditLogEntry$json = {
  '1': 'AuditLogEntry',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 9, '10': 'id'},
    {'1': 'target_table', '3': 2, '4': 1, '5': 9, '10': 'targetTable'},
    {'1': 'target_record_id', '3': 3, '4': 1, '5': 9, '10': 'targetRecordId'},
    {'1': 'operation', '3': 4, '4': 1, '5': 9, '10': 'operation'},
    {'1': 'before_data_json', '3': 5, '4': 1, '5': 9, '10': 'beforeDataJson'},
    {'1': 'after_data_json', '3': 6, '4': 1, '5': 9, '10': 'afterDataJson'},
    {'1': 'changed_columns', '3': 7, '4': 3, '5': 9, '10': 'changedColumns'},
    {'1': 'changed_by', '3': 8, '4': 1, '5': 9, '10': 'changedBy'},
    {'1': 'change_reason', '3': 9, '4': 1, '5': 9, '10': 'changeReason'},
    {'1': 'trace_id', '3': 10, '4': 1, '5': 9, '10': 'traceId'},
    {'1': 'created_at', '3': 11, '4': 1, '5': 9, '10': 'createdAt'},
    {'1': 'domain_scope', '3': 12, '4': 1, '5': 9, '10': 'domainScope'},
  ],
};

/// Descriptor for `AuditLogEntry`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List auditLogEntryDescriptor = $convert.base64Decode(
    'Cg1BdWRpdExvZ0VudHJ5Eg4KAmlkGAEgASgJUgJpZBIhCgx0YXJnZXRfdGFibGUYAiABKAlSC3'
    'RhcmdldFRhYmxlEigKEHRhcmdldF9yZWNvcmRfaWQYAyABKAlSDnRhcmdldFJlY29yZElkEhwK'
    'CW9wZXJhdGlvbhgEIAEoCVIJb3BlcmF0aW9uEigKEGJlZm9yZV9kYXRhX2pzb24YBSABKAlSDm'
    'JlZm9yZURhdGFKc29uEiYKD2FmdGVyX2RhdGFfanNvbhgGIAEoCVINYWZ0ZXJEYXRhSnNvbhIn'
    'Cg9jaGFuZ2VkX2NvbHVtbnMYByADKAlSDmNoYW5nZWRDb2x1bW5zEh0KCmNoYW5nZWRfYnkYCC'
    'ABKAlSCWNoYW5nZWRCeRIjCg1jaGFuZ2VfcmVhc29uGAkgASgJUgxjaGFuZ2VSZWFzb24SGQoI'
    'dHJhY2VfaWQYCiABKAlSB3RyYWNlSWQSHQoKY3JlYXRlZF9hdBgLIAEoCVIJY3JlYXRlZEF0Ei'
    'EKDGRvbWFpbl9zY29wZRgMIAEoCVILZG9tYWluU2NvcGU=');

@$core.Deprecated('Use listDomainsRequestDescriptor instead')
const ListDomainsRequest$json = {
  '1': 'ListDomainsRequest',
};

/// Descriptor for `ListDomainsRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List listDomainsRequestDescriptor =
    $convert.base64Decode('ChJMaXN0RG9tYWluc1JlcXVlc3Q=');

@$core.Deprecated('Use listDomainsResponseDescriptor instead')
const ListDomainsResponse$json = {
  '1': 'ListDomainsResponse',
  '2': [
    {
      '1': 'domains',
      '3': 1,
      '4': 3,
      '5': 11,
      '6': '.k1s0.system.mastermaintenance.v1.DomainInfo',
      '10': 'domains'
    },
  ],
};

/// Descriptor for `ListDomainsResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List listDomainsResponseDescriptor = $convert.base64Decode(
    'ChNMaXN0RG9tYWluc1Jlc3BvbnNlEkYKB2RvbWFpbnMYASADKAsyLC5rMXMwLnN5c3RlbS5tYX'
    'N0ZXJtYWludGVuYW5jZS52MS5Eb21haW5JbmZvUgdkb21haW5z');

@$core.Deprecated('Use domainInfoDescriptor instead')
const DomainInfo$json = {
  '1': 'DomainInfo',
  '2': [
    {'1': 'domain_scope', '3': 1, '4': 1, '5': 9, '10': 'domainScope'},
    {'1': 'table_count', '3': 2, '4': 1, '5': 5, '10': 'tableCount'},
  ],
};

/// Descriptor for `DomainInfo`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List domainInfoDescriptor = $convert.base64Decode(
    'CgpEb21haW5JbmZvEiEKDGRvbWFpbl9zY29wZRgBIAEoCVILZG9tYWluU2NvcGUSHwoLdGFibG'
    'VfY291bnQYAiABKAVSCnRhYmxlQ291bnQ=');
