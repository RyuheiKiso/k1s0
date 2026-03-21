// This is a generated file - do not edit.
//
// Generated from k1s0/system/file/v1/file.proto.

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

@$core.Deprecated('Use fileMetadataDescriptor instead')
const FileMetadata$json = {
  '1': 'FileMetadata',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 9, '10': 'id'},
    {'1': 'filename', '3': 2, '4': 1, '5': 9, '10': 'filename'},
    {'1': 'content_type', '3': 3, '4': 1, '5': 9, '10': 'contentType'},
    {'1': 'size_bytes', '3': 4, '4': 1, '5': 3, '10': 'sizeBytes'},
    {'1': 'tenant_id', '3': 5, '4': 1, '5': 9, '10': 'tenantId'},
    {'1': 'uploaded_by', '3': 6, '4': 1, '5': 9, '10': 'uploadedBy'},
    {'1': 'status', '3': 7, '4': 1, '5': 9, '10': 'status'},
    {'1': 'created_at', '3': 8, '4': 1, '5': 9, '10': 'createdAt'},
    {'1': 'updated_at', '3': 9, '4': 1, '5': 9, '10': 'updatedAt'},
    {
      '1': 'tags',
      '3': 10,
      '4': 3,
      '5': 11,
      '6': '.k1s0.system.file.v1.FileMetadata.TagsEntry',
      '10': 'tags'
    },
    {'1': 'storage_key', '3': 11, '4': 1, '5': 9, '10': 'storageKey'},
    {
      '1': 'checksum_sha256',
      '3': 12,
      '4': 1,
      '5': 9,
      '9': 0,
      '10': 'checksumSha256',
      '17': true
    },
  ],
  '3': [FileMetadata_TagsEntry$json],
  '8': [
    {'1': '_checksum_sha256'},
  ],
};

@$core.Deprecated('Use fileMetadataDescriptor instead')
const FileMetadata_TagsEntry$json = {
  '1': 'TagsEntry',
  '2': [
    {'1': 'key', '3': 1, '4': 1, '5': 9, '10': 'key'},
    {'1': 'value', '3': 2, '4': 1, '5': 9, '10': 'value'},
  ],
  '7': {'7': true},
};

/// Descriptor for `FileMetadata`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List fileMetadataDescriptor = $convert.base64Decode(
    'CgxGaWxlTWV0YWRhdGESDgoCaWQYASABKAlSAmlkEhoKCGZpbGVuYW1lGAIgASgJUghmaWxlbm'
    'FtZRIhCgxjb250ZW50X3R5cGUYAyABKAlSC2NvbnRlbnRUeXBlEh0KCnNpemVfYnl0ZXMYBCAB'
    'KANSCXNpemVCeXRlcxIbCgl0ZW5hbnRfaWQYBSABKAlSCHRlbmFudElkEh8KC3VwbG9hZGVkX2'
    'J5GAYgASgJUgp1cGxvYWRlZEJ5EhYKBnN0YXR1cxgHIAEoCVIGc3RhdHVzEh0KCmNyZWF0ZWRf'
    'YXQYCCABKAlSCWNyZWF0ZWRBdBIdCgp1cGRhdGVkX2F0GAkgASgJUgl1cGRhdGVkQXQSPwoEdG'
    'FncxgKIAMoCzIrLmsxczAuc3lzdGVtLmZpbGUudjEuRmlsZU1ldGFkYXRhLlRhZ3NFbnRyeVIE'
    'dGFncxIfCgtzdG9yYWdlX2tleRgLIAEoCVIKc3RvcmFnZUtleRIsCg9jaGVja3N1bV9zaGEyNT'
    'YYDCABKAlIAFIOY2hlY2tzdW1TaGEyNTaIAQEaNwoJVGFnc0VudHJ5EhAKA2tleRgBIAEoCVID'
    'a2V5EhQKBXZhbHVlGAIgASgJUgV2YWx1ZToCOAFCEgoQX2NoZWNrc3VtX3NoYTI1Ng==');

@$core.Deprecated('Use getFileMetadataRequestDescriptor instead')
const GetFileMetadataRequest$json = {
  '1': 'GetFileMetadataRequest',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 9, '10': 'id'},
  ],
};

/// Descriptor for `GetFileMetadataRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List getFileMetadataRequestDescriptor = $convert
    .base64Decode('ChZHZXRGaWxlTWV0YWRhdGFSZXF1ZXN0Eg4KAmlkGAEgASgJUgJpZA==');

@$core.Deprecated('Use getFileMetadataResponseDescriptor instead')
const GetFileMetadataResponse$json = {
  '1': 'GetFileMetadataResponse',
  '2': [
    {
      '1': 'metadata',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.file.v1.FileMetadata',
      '10': 'metadata'
    },
  ],
};

/// Descriptor for `GetFileMetadataResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List getFileMetadataResponseDescriptor =
    $convert.base64Decode(
        'ChdHZXRGaWxlTWV0YWRhdGFSZXNwb25zZRI9CghtZXRhZGF0YRgBIAEoCzIhLmsxczAuc3lzdG'
        'VtLmZpbGUudjEuRmlsZU1ldGFkYXRhUghtZXRhZGF0YQ==');

@$core.Deprecated('Use listFilesRequestDescriptor instead')
const ListFilesRequest$json = {
  '1': 'ListFilesRequest',
  '2': [
    {'1': 'tenant_id', '3': 1, '4': 1, '5': 9, '10': 'tenantId'},
    {
      '1': 'pagination',
      '3': 2,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.common.v1.Pagination',
      '10': 'pagination'
    },
    {
      '1': 'uploaded_by',
      '3': 4,
      '4': 1,
      '5': 9,
      '9': 0,
      '10': 'uploadedBy',
      '17': true
    },
    {
      '1': 'mime_type',
      '3': 5,
      '4': 1,
      '5': 9,
      '9': 1,
      '10': 'mimeType',
      '17': true
    },
    {'1': 'tag', '3': 6, '4': 1, '5': 9, '9': 2, '10': 'tag', '17': true},
  ],
  '8': [
    {'1': '_uploaded_by'},
    {'1': '_mime_type'},
    {'1': '_tag'},
  ],
  '9': [
    {'1': 3, '2': 4},
  ],
  '10': ['page_size'],
};

/// Descriptor for `ListFilesRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List listFilesRequestDescriptor = $convert.base64Decode(
    'ChBMaXN0RmlsZXNSZXF1ZXN0EhsKCXRlbmFudF9pZBgBIAEoCVIIdGVuYW50SWQSQQoKcGFnaW'
    '5hdGlvbhgCIAEoCzIhLmsxczAuc3lzdGVtLmNvbW1vbi52MS5QYWdpbmF0aW9uUgpwYWdpbmF0'
    'aW9uEiQKC3VwbG9hZGVkX2J5GAQgASgJSABSCnVwbG9hZGVkQnmIAQESIAoJbWltZV90eXBlGA'
    'UgASgJSAFSCG1pbWVUeXBliAEBEhUKA3RhZxgGIAEoCUgCUgN0YWeIAQFCDgoMX3VwbG9hZGVk'
    'X2J5QgwKCl9taW1lX3R5cGVCBgoEX3RhZ0oECAMQBFIJcGFnZV9zaXpl');

@$core.Deprecated('Use listFilesResponseDescriptor instead')
const ListFilesResponse$json = {
  '1': 'ListFilesResponse',
  '2': [
    {
      '1': 'files',
      '3': 1,
      '4': 3,
      '5': 11,
      '6': '.k1s0.system.file.v1.FileMetadata',
      '10': 'files'
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

/// Descriptor for `ListFilesResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List listFilesResponseDescriptor = $convert.base64Decode(
    'ChFMaXN0RmlsZXNSZXNwb25zZRI3CgVmaWxlcxgBIAMoCzIhLmsxczAuc3lzdGVtLmZpbGUudj'
    'EuRmlsZU1ldGFkYXRhUgVmaWxlcxJHCgpwYWdpbmF0aW9uGAIgASgLMicuazFzMC5zeXN0ZW0u'
    'Y29tbW9uLnYxLlBhZ2luYXRpb25SZXN1bHRSCnBhZ2luYXRpb24=');

@$core.Deprecated('Use generateUploadUrlRequestDescriptor instead')
const GenerateUploadUrlRequest$json = {
  '1': 'GenerateUploadUrlRequest',
  '2': [
    {'1': 'filename', '3': 1, '4': 1, '5': 9, '10': 'filename'},
    {'1': 'content_type', '3': 2, '4': 1, '5': 9, '10': 'contentType'},
    {'1': 'tenant_id', '3': 3, '4': 1, '5': 9, '10': 'tenantId'},
    {'1': 'uploaded_by', '3': 4, '4': 1, '5': 9, '10': 'uploadedBy'},
    {
      '1': 'tags',
      '3': 5,
      '4': 3,
      '5': 11,
      '6': '.k1s0.system.file.v1.GenerateUploadUrlRequest.TagsEntry',
      '10': 'tags'
    },
    {
      '1': 'expires_in_seconds',
      '3': 6,
      '4': 1,
      '5': 13,
      '9': 0,
      '10': 'expiresInSeconds',
      '17': true
    },
    {'1': 'size_bytes', '3': 7, '4': 1, '5': 3, '10': 'sizeBytes'},
  ],
  '3': [GenerateUploadUrlRequest_TagsEntry$json],
  '8': [
    {'1': '_expires_in_seconds'},
  ],
};

@$core.Deprecated('Use generateUploadUrlRequestDescriptor instead')
const GenerateUploadUrlRequest_TagsEntry$json = {
  '1': 'TagsEntry',
  '2': [
    {'1': 'key', '3': 1, '4': 1, '5': 9, '10': 'key'},
    {'1': 'value', '3': 2, '4': 1, '5': 9, '10': 'value'},
  ],
  '7': {'7': true},
};

/// Descriptor for `GenerateUploadUrlRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List generateUploadUrlRequestDescriptor = $convert.base64Decode(
    'ChhHZW5lcmF0ZVVwbG9hZFVybFJlcXVlc3QSGgoIZmlsZW5hbWUYASABKAlSCGZpbGVuYW1lEi'
    'EKDGNvbnRlbnRfdHlwZRgCIAEoCVILY29udGVudFR5cGUSGwoJdGVuYW50X2lkGAMgASgJUgh0'
    'ZW5hbnRJZBIfCgt1cGxvYWRlZF9ieRgEIAEoCVIKdXBsb2FkZWRCeRJLCgR0YWdzGAUgAygLMj'
    'cuazFzMC5zeXN0ZW0uZmlsZS52MS5HZW5lcmF0ZVVwbG9hZFVybFJlcXVlc3QuVGFnc0VudHJ5'
    'UgR0YWdzEjEKEmV4cGlyZXNfaW5fc2Vjb25kcxgGIAEoDUgAUhBleHBpcmVzSW5TZWNvbmRziA'
    'EBEh0KCnNpemVfYnl0ZXMYByABKANSCXNpemVCeXRlcxo3CglUYWdzRW50cnkSEAoDa2V5GAEg'
    'ASgJUgNrZXkSFAoFdmFsdWUYAiABKAlSBXZhbHVlOgI4AUIVChNfZXhwaXJlc19pbl9zZWNvbm'
    'Rz');

@$core.Deprecated('Use generateUploadUrlResponseDescriptor instead')
const GenerateUploadUrlResponse$json = {
  '1': 'GenerateUploadUrlResponse',
  '2': [
    {'1': 'file_id', '3': 1, '4': 1, '5': 9, '10': 'fileId'},
    {'1': 'upload_url', '3': 2, '4': 1, '5': 9, '10': 'uploadUrl'},
    {
      '1': 'expires_in_seconds',
      '3': 3,
      '4': 1,
      '5': 13,
      '10': 'expiresInSeconds'
    },
  ],
};

/// Descriptor for `GenerateUploadUrlResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List generateUploadUrlResponseDescriptor = $convert.base64Decode(
    'ChlHZW5lcmF0ZVVwbG9hZFVybFJlc3BvbnNlEhcKB2ZpbGVfaWQYASABKAlSBmZpbGVJZBIdCg'
    'p1cGxvYWRfdXJsGAIgASgJUgl1cGxvYWRVcmwSLAoSZXhwaXJlc19pbl9zZWNvbmRzGAMgASgN'
    'UhBleHBpcmVzSW5TZWNvbmRz');

@$core.Deprecated('Use completeUploadRequestDescriptor instead')
const CompleteUploadRequest$json = {
  '1': 'CompleteUploadRequest',
  '2': [
    {'1': 'file_id', '3': 1, '4': 1, '5': 9, '10': 'fileId'},
    {
      '1': 'checksum_sha256',
      '3': 3,
      '4': 1,
      '5': 9,
      '9': 0,
      '10': 'checksumSha256',
      '17': true
    },
  ],
  '8': [
    {'1': '_checksum_sha256'},
  ],
};

/// Descriptor for `CompleteUploadRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List completeUploadRequestDescriptor = $convert.base64Decode(
    'ChVDb21wbGV0ZVVwbG9hZFJlcXVlc3QSFwoHZmlsZV9pZBgBIAEoCVIGZmlsZUlkEiwKD2NoZW'
    'Nrc3VtX3NoYTI1NhgDIAEoCUgAUg5jaGVja3N1bVNoYTI1NogBAUISChBfY2hlY2tzdW1fc2hh'
    'MjU2');

@$core.Deprecated('Use completeUploadResponseDescriptor instead')
const CompleteUploadResponse$json = {
  '1': 'CompleteUploadResponse',
  '2': [
    {
      '1': 'metadata',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.file.v1.FileMetadata',
      '10': 'metadata'
    },
  ],
};

/// Descriptor for `CompleteUploadResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List completeUploadResponseDescriptor =
    $convert.base64Decode(
        'ChZDb21wbGV0ZVVwbG9hZFJlc3BvbnNlEj0KCG1ldGFkYXRhGAEgASgLMiEuazFzMC5zeXN0ZW'
        '0uZmlsZS52MS5GaWxlTWV0YWRhdGFSCG1ldGFkYXRh');

@$core.Deprecated('Use generateDownloadUrlRequestDescriptor instead')
const GenerateDownloadUrlRequest$json = {
  '1': 'GenerateDownloadUrlRequest',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 9, '10': 'id'},
  ],
};

/// Descriptor for `GenerateDownloadUrlRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List generateDownloadUrlRequestDescriptor =
    $convert.base64Decode(
        'ChpHZW5lcmF0ZURvd25sb2FkVXJsUmVxdWVzdBIOCgJpZBgBIAEoCVICaWQ=');

@$core.Deprecated('Use generateDownloadUrlResponseDescriptor instead')
const GenerateDownloadUrlResponse$json = {
  '1': 'GenerateDownloadUrlResponse',
  '2': [
    {'1': 'download_url', '3': 1, '4': 1, '5': 9, '10': 'downloadUrl'},
    {
      '1': 'expires_in_seconds',
      '3': 2,
      '4': 1,
      '5': 5,
      '10': 'expiresInSeconds'
    },
  ],
};

/// Descriptor for `GenerateDownloadUrlResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List generateDownloadUrlResponseDescriptor =
    $convert.base64Decode(
        'ChtHZW5lcmF0ZURvd25sb2FkVXJsUmVzcG9uc2USIQoMZG93bmxvYWRfdXJsGAEgASgJUgtkb3'
        'dubG9hZFVybBIsChJleHBpcmVzX2luX3NlY29uZHMYAiABKAVSEGV4cGlyZXNJblNlY29uZHM=');

@$core.Deprecated('Use updateFileTagsRequestDescriptor instead')
const UpdateFileTagsRequest$json = {
  '1': 'UpdateFileTagsRequest',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 9, '10': 'id'},
    {
      '1': 'tags',
      '3': 2,
      '4': 3,
      '5': 11,
      '6': '.k1s0.system.file.v1.UpdateFileTagsRequest.TagsEntry',
      '10': 'tags'
    },
  ],
  '3': [UpdateFileTagsRequest_TagsEntry$json],
};

@$core.Deprecated('Use updateFileTagsRequestDescriptor instead')
const UpdateFileTagsRequest_TagsEntry$json = {
  '1': 'TagsEntry',
  '2': [
    {'1': 'key', '3': 1, '4': 1, '5': 9, '10': 'key'},
    {'1': 'value', '3': 2, '4': 1, '5': 9, '10': 'value'},
  ],
  '7': {'7': true},
};

/// Descriptor for `UpdateFileTagsRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List updateFileTagsRequestDescriptor = $convert.base64Decode(
    'ChVVcGRhdGVGaWxlVGFnc1JlcXVlc3QSDgoCaWQYASABKAlSAmlkEkgKBHRhZ3MYAiADKAsyNC'
    '5rMXMwLnN5c3RlbS5maWxlLnYxLlVwZGF0ZUZpbGVUYWdzUmVxdWVzdC5UYWdzRW50cnlSBHRh'
    'Z3MaNwoJVGFnc0VudHJ5EhAKA2tleRgBIAEoCVIDa2V5EhQKBXZhbHVlGAIgASgJUgV2YWx1ZT'
    'oCOAE=');

@$core.Deprecated('Use updateFileTagsResponseDescriptor instead')
const UpdateFileTagsResponse$json = {
  '1': 'UpdateFileTagsResponse',
  '2': [
    {
      '1': 'metadata',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.file.v1.FileMetadata',
      '10': 'metadata'
    },
  ],
};

/// Descriptor for `UpdateFileTagsResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List updateFileTagsResponseDescriptor =
    $convert.base64Decode(
        'ChZVcGRhdGVGaWxlVGFnc1Jlc3BvbnNlEj0KCG1ldGFkYXRhGAEgASgLMiEuazFzMC5zeXN0ZW'
        '0uZmlsZS52MS5GaWxlTWV0YWRhdGFSCG1ldGFkYXRh');

@$core.Deprecated('Use deleteFileRequestDescriptor instead')
const DeleteFileRequest$json = {
  '1': 'DeleteFileRequest',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 9, '10': 'id'},
  ],
};

/// Descriptor for `DeleteFileRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List deleteFileRequestDescriptor =
    $convert.base64Decode('ChFEZWxldGVGaWxlUmVxdWVzdBIOCgJpZBgBIAEoCVICaWQ=');

@$core.Deprecated('Use deleteFileResponseDescriptor instead')
const DeleteFileResponse$json = {
  '1': 'DeleteFileResponse',
};

/// Descriptor for `DeleteFileResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List deleteFileResponseDescriptor =
    $convert.base64Decode('ChJEZWxldGVGaWxlUmVzcG9uc2U=');
