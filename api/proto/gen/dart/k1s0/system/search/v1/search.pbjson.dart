// This is a generated file - do not edit.
//
// Generated from k1s0/system/search/v1/search.proto.

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

@$core.Deprecated('Use searchIndexDescriptor instead')
const SearchIndex$json = {
  '1': 'SearchIndex',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 9, '10': 'id'},
    {'1': 'name', '3': 2, '4': 1, '5': 9, '10': 'name'},
    {'1': 'mapping_json', '3': 3, '4': 1, '5': 12, '10': 'mappingJson'},
    {'1': 'created_at', '3': 4, '4': 1, '5': 9, '10': 'createdAt'},
  ],
};

/// Descriptor for `SearchIndex`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List searchIndexDescriptor = $convert.base64Decode(
    'CgtTZWFyY2hJbmRleBIOCgJpZBgBIAEoCVICaWQSEgoEbmFtZRgCIAEoCVIEbmFtZRIhCgxtYX'
    'BwaW5nX2pzb24YAyABKAxSC21hcHBpbmdKc29uEh0KCmNyZWF0ZWRfYXQYBCABKAlSCWNyZWF0'
    'ZWRBdA==');

@$core.Deprecated('Use createIndexRequestDescriptor instead')
const CreateIndexRequest$json = {
  '1': 'CreateIndexRequest',
  '2': [
    {'1': 'name', '3': 1, '4': 1, '5': 9, '10': 'name'},
    {'1': 'mapping_json', '3': 2, '4': 1, '5': 12, '10': 'mappingJson'},
  ],
};

/// Descriptor for `CreateIndexRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List createIndexRequestDescriptor = $convert.base64Decode(
    'ChJDcmVhdGVJbmRleFJlcXVlc3QSEgoEbmFtZRgBIAEoCVIEbmFtZRIhCgxtYXBwaW5nX2pzb2'
    '4YAiABKAxSC21hcHBpbmdKc29u');

@$core.Deprecated('Use createIndexResponseDescriptor instead')
const CreateIndexResponse$json = {
  '1': 'CreateIndexResponse',
  '2': [
    {
      '1': 'index',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.search.v1.SearchIndex',
      '10': 'index'
    },
  ],
};

/// Descriptor for `CreateIndexResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List createIndexResponseDescriptor = $convert.base64Decode(
    'ChNDcmVhdGVJbmRleFJlc3BvbnNlEjgKBWluZGV4GAEgASgLMiIuazFzMC5zeXN0ZW0uc2Vhcm'
    'NoLnYxLlNlYXJjaEluZGV4UgVpbmRleA==');

@$core.Deprecated('Use listIndicesRequestDescriptor instead')
const ListIndicesRequest$json = {
  '1': 'ListIndicesRequest',
};

/// Descriptor for `ListIndicesRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List listIndicesRequestDescriptor =
    $convert.base64Decode('ChJMaXN0SW5kaWNlc1JlcXVlc3Q=');

@$core.Deprecated('Use listIndicesResponseDescriptor instead')
const ListIndicesResponse$json = {
  '1': 'ListIndicesResponse',
  '2': [
    {
      '1': 'indices',
      '3': 1,
      '4': 3,
      '5': 11,
      '6': '.k1s0.system.search.v1.SearchIndex',
      '10': 'indices'
    },
  ],
};

/// Descriptor for `ListIndicesResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List listIndicesResponseDescriptor = $convert.base64Decode(
    'ChNMaXN0SW5kaWNlc1Jlc3BvbnNlEjwKB2luZGljZXMYASADKAsyIi5rMXMwLnN5c3RlbS5zZW'
    'FyY2gudjEuU2VhcmNoSW5kZXhSB2luZGljZXM=');

@$core.Deprecated('Use indexDocumentRequestDescriptor instead')
const IndexDocumentRequest$json = {
  '1': 'IndexDocumentRequest',
  '2': [
    {'1': 'index', '3': 1, '4': 1, '5': 9, '10': 'index'},
    {'1': 'document_id', '3': 2, '4': 1, '5': 9, '10': 'documentId'},
    {'1': 'document_json', '3': 3, '4': 1, '5': 12, '10': 'documentJson'},
  ],
};

/// Descriptor for `IndexDocumentRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List indexDocumentRequestDescriptor = $convert.base64Decode(
    'ChRJbmRleERvY3VtZW50UmVxdWVzdBIUCgVpbmRleBgBIAEoCVIFaW5kZXgSHwoLZG9jdW1lbn'
    'RfaWQYAiABKAlSCmRvY3VtZW50SWQSIwoNZG9jdW1lbnRfanNvbhgDIAEoDFIMZG9jdW1lbnRK'
    'c29u');

@$core.Deprecated('Use indexDocumentResponseDescriptor instead')
const IndexDocumentResponse$json = {
  '1': 'IndexDocumentResponse',
  '2': [
    {'1': 'document_id', '3': 1, '4': 1, '5': 9, '10': 'documentId'},
    {'1': 'index', '3': 2, '4': 1, '5': 9, '10': 'index'},
    {'1': 'result', '3': 3, '4': 1, '5': 9, '10': 'result'},
  ],
};

/// Descriptor for `IndexDocumentResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List indexDocumentResponseDescriptor = $convert.base64Decode(
    'ChVJbmRleERvY3VtZW50UmVzcG9uc2USHwoLZG9jdW1lbnRfaWQYASABKAlSCmRvY3VtZW50SW'
    'QSFAoFaW5kZXgYAiABKAlSBWluZGV4EhYKBnJlc3VsdBgDIAEoCVIGcmVzdWx0');

@$core.Deprecated('Use searchRequestDescriptor instead')
const SearchRequest$json = {
  '1': 'SearchRequest',
  '2': [
    {'1': 'index', '3': 1, '4': 1, '5': 9, '10': 'index'},
    {'1': 'query', '3': 2, '4': 1, '5': 9, '10': 'query'},
    {'1': 'filters_json', '3': 3, '4': 1, '5': 12, '10': 'filtersJson'},
    {'1': 'from', '3': 4, '4': 1, '5': 13, '10': 'from'},
    {'1': 'size', '3': 5, '4': 1, '5': 13, '10': 'size'},
    {'1': 'facets', '3': 6, '4': 3, '5': 9, '10': 'facets'},
  ],
};

/// Descriptor for `SearchRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List searchRequestDescriptor = $convert.base64Decode(
    'Cg1TZWFyY2hSZXF1ZXN0EhQKBWluZGV4GAEgASgJUgVpbmRleBIUCgVxdWVyeRgCIAEoCVIFcX'
    'VlcnkSIQoMZmlsdGVyc19qc29uGAMgASgMUgtmaWx0ZXJzSnNvbhISCgRmcm9tGAQgASgNUgRm'
    'cm9tEhIKBHNpemUYBSABKA1SBHNpemUSFgoGZmFjZXRzGAYgAygJUgZmYWNldHM=');

@$core.Deprecated('Use searchResponseDescriptor instead')
const SearchResponse$json = {
  '1': 'SearchResponse',
  '2': [
    {
      '1': 'hits',
      '3': 1,
      '4': 3,
      '5': 11,
      '6': '.k1s0.system.search.v1.SearchHit',
      '10': 'hits'
    },
    {
      '1': 'pagination',
      '3': 2,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.common.v1.PaginationResult',
      '10': 'pagination'
    },
    {
      '1': 'facets',
      '3': 3,
      '4': 3,
      '5': 11,
      '6': '.k1s0.system.search.v1.SearchResponse.FacetsEntry',
      '10': 'facets'
    },
  ],
  '3': [SearchResponse_FacetsEntry$json],
};

@$core.Deprecated('Use searchResponseDescriptor instead')
const SearchResponse_FacetsEntry$json = {
  '1': 'FacetsEntry',
  '2': [
    {'1': 'key', '3': 1, '4': 1, '5': 9, '10': 'key'},
    {
      '1': 'value',
      '3': 2,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.search.v1.FacetCounts',
      '10': 'value'
    },
  ],
  '7': {'7': true},
};

/// Descriptor for `SearchResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List searchResponseDescriptor = $convert.base64Decode(
    'Cg5TZWFyY2hSZXNwb25zZRI0CgRoaXRzGAEgAygLMiAuazFzMC5zeXN0ZW0uc2VhcmNoLnYxLl'
    'NlYXJjaEhpdFIEaGl0cxJHCgpwYWdpbmF0aW9uGAIgASgLMicuazFzMC5zeXN0ZW0uY29tbW9u'
    'LnYxLlBhZ2luYXRpb25SZXN1bHRSCnBhZ2luYXRpb24SSQoGZmFjZXRzGAMgAygLMjEuazFzMC'
    '5zeXN0ZW0uc2VhcmNoLnYxLlNlYXJjaFJlc3BvbnNlLkZhY2V0c0VudHJ5UgZmYWNldHMaXQoL'
    'RmFjZXRzRW50cnkSEAoDa2V5GAEgASgJUgNrZXkSOAoFdmFsdWUYAiABKAsyIi5rMXMwLnN5c3'
    'RlbS5zZWFyY2gudjEuRmFjZXRDb3VudHNSBXZhbHVlOgI4AQ==');

@$core.Deprecated('Use facetCountsDescriptor instead')
const FacetCounts$json = {
  '1': 'FacetCounts',
  '2': [
    {
      '1': 'buckets',
      '3': 1,
      '4': 3,
      '5': 11,
      '6': '.k1s0.system.search.v1.FacetCounts.BucketsEntry',
      '10': 'buckets'
    },
  ],
  '3': [FacetCounts_BucketsEntry$json],
};

@$core.Deprecated('Use facetCountsDescriptor instead')
const FacetCounts_BucketsEntry$json = {
  '1': 'BucketsEntry',
  '2': [
    {'1': 'key', '3': 1, '4': 1, '5': 9, '10': 'key'},
    {'1': 'value', '3': 2, '4': 1, '5': 4, '10': 'value'},
  ],
  '7': {'7': true},
};

/// Descriptor for `FacetCounts`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List facetCountsDescriptor = $convert.base64Decode(
    'CgtGYWNldENvdW50cxJJCgdidWNrZXRzGAEgAygLMi8uazFzMC5zeXN0ZW0uc2VhcmNoLnYxLk'
    'ZhY2V0Q291bnRzLkJ1Y2tldHNFbnRyeVIHYnVja2V0cxo6CgxCdWNrZXRzRW50cnkSEAoDa2V5'
    'GAEgASgJUgNrZXkSFAoFdmFsdWUYAiABKARSBXZhbHVlOgI4AQ==');

@$core.Deprecated('Use searchHitDescriptor instead')
const SearchHit$json = {
  '1': 'SearchHit',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 9, '10': 'id'},
    {'1': 'score', '3': 2, '4': 1, '5': 2, '10': 'score'},
    {'1': 'document_json', '3': 3, '4': 1, '5': 12, '10': 'documentJson'},
  ],
};

/// Descriptor for `SearchHit`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List searchHitDescriptor = $convert.base64Decode(
    'CglTZWFyY2hIaXQSDgoCaWQYASABKAlSAmlkEhQKBXNjb3JlGAIgASgCUgVzY29yZRIjCg1kb2'
    'N1bWVudF9qc29uGAMgASgMUgxkb2N1bWVudEpzb24=');

@$core.Deprecated('Use deleteDocumentRequestDescriptor instead')
const DeleteDocumentRequest$json = {
  '1': 'DeleteDocumentRequest',
  '2': [
    {'1': 'index', '3': 1, '4': 1, '5': 9, '10': 'index'},
    {'1': 'document_id', '3': 2, '4': 1, '5': 9, '10': 'documentId'},
  ],
};

/// Descriptor for `DeleteDocumentRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List deleteDocumentRequestDescriptor = $convert.base64Decode(
    'ChVEZWxldGVEb2N1bWVudFJlcXVlc3QSFAoFaW5kZXgYASABKAlSBWluZGV4Eh8KC2RvY3VtZW'
    '50X2lkGAIgASgJUgpkb2N1bWVudElk');

@$core.Deprecated('Use deleteDocumentResponseDescriptor instead')
const DeleteDocumentResponse$json = {
  '1': 'DeleteDocumentResponse',
  '2': [
    {'1': 'success', '3': 1, '4': 1, '5': 8, '10': 'success'},
    {'1': 'message', '3': 2, '4': 1, '5': 9, '10': 'message'},
  ],
};

/// Descriptor for `DeleteDocumentResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List deleteDocumentResponseDescriptor =
    $convert.base64Decode(
        'ChZEZWxldGVEb2N1bWVudFJlc3BvbnNlEhgKB3N1Y2Nlc3MYASABKAhSB3N1Y2Nlc3MSGAoHbW'
        'Vzc2FnZRgCIAEoCVIHbWVzc2FnZQ==');
