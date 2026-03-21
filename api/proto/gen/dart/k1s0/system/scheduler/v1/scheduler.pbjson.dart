// This is a generated file - do not edit.
//
// Generated from k1s0/system/scheduler/v1/scheduler.proto.

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

@$core.Deprecated('Use jobDescriptor instead')
const Job$json = {
  '1': 'Job',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 9, '10': 'id'},
    {'1': 'name', '3': 2, '4': 1, '5': 9, '10': 'name'},
    {'1': 'description', '3': 3, '4': 1, '5': 9, '10': 'description'},
    {'1': 'cron_expression', '3': 4, '4': 1, '5': 9, '10': 'cronExpression'},
    {'1': 'timezone', '3': 5, '4': 1, '5': 9, '10': 'timezone'},
    {'1': 'target_type', '3': 6, '4': 1, '5': 9, '10': 'targetType'},
    {'1': 'target', '3': 7, '4': 1, '5': 9, '10': 'target'},
    {
      '1': 'payload',
      '3': 8,
      '4': 1,
      '5': 11,
      '6': '.google.protobuf.Struct',
      '10': 'payload'
    },
    {'1': 'status', '3': 9, '4': 1, '5': 9, '10': 'status'},
    {
      '1': 'next_run_at',
      '3': 10,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.common.v1.Timestamp',
      '9': 0,
      '10': 'nextRunAt',
      '17': true
    },
    {
      '1': 'last_run_at',
      '3': 11,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.common.v1.Timestamp',
      '9': 1,
      '10': 'lastRunAt',
      '17': true
    },
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
  '8': [
    {'1': '_next_run_at'},
    {'1': '_last_run_at'},
  ],
};

/// Descriptor for `Job`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List jobDescriptor = $convert.base64Decode(
    'CgNKb2ISDgoCaWQYASABKAlSAmlkEhIKBG5hbWUYAiABKAlSBG5hbWUSIAoLZGVzY3JpcHRpb2'
    '4YAyABKAlSC2Rlc2NyaXB0aW9uEicKD2Nyb25fZXhwcmVzc2lvbhgEIAEoCVIOY3JvbkV4cHJl'
    'c3Npb24SGgoIdGltZXpvbmUYBSABKAlSCHRpbWV6b25lEh8KC3RhcmdldF90eXBlGAYgASgJUg'
    'p0YXJnZXRUeXBlEhYKBnRhcmdldBgHIAEoCVIGdGFyZ2V0EjEKB3BheWxvYWQYCCABKAsyFy5n'
    'b29nbGUucHJvdG9idWYuU3RydWN0UgdwYXlsb2FkEhYKBnN0YXR1cxgJIAEoCVIGc3RhdHVzEk'
    'UKC25leHRfcnVuX2F0GAogASgLMiAuazFzMC5zeXN0ZW0uY29tbW9uLnYxLlRpbWVzdGFtcEgA'
    'UgluZXh0UnVuQXSIAQESRQoLbGFzdF9ydW5fYXQYCyABKAsyIC5rMXMwLnN5c3RlbS5jb21tb2'
    '4udjEuVGltZXN0YW1wSAFSCWxhc3RSdW5BdIgBARI/CgpjcmVhdGVkX2F0GAwgASgLMiAuazFz'
    'MC5zeXN0ZW0uY29tbW9uLnYxLlRpbWVzdGFtcFIJY3JlYXRlZEF0Ej8KCnVwZGF0ZWRfYXQYDS'
    'ABKAsyIC5rMXMwLnN5c3RlbS5jb21tb24udjEuVGltZXN0YW1wUgl1cGRhdGVkQXRCDgoMX25l'
    'eHRfcnVuX2F0Qg4KDF9sYXN0X3J1bl9hdA==');

@$core.Deprecated('Use createJobRequestDescriptor instead')
const CreateJobRequest$json = {
  '1': 'CreateJobRequest',
  '2': [
    {'1': 'name', '3': 1, '4': 1, '5': 9, '10': 'name'},
    {'1': 'description', '3': 2, '4': 1, '5': 9, '10': 'description'},
    {'1': 'cron_expression', '3': 3, '4': 1, '5': 9, '10': 'cronExpression'},
    {'1': 'timezone', '3': 4, '4': 1, '5': 9, '10': 'timezone'},
    {'1': 'target_type', '3': 5, '4': 1, '5': 9, '10': 'targetType'},
    {'1': 'target', '3': 6, '4': 1, '5': 9, '10': 'target'},
    {
      '1': 'payload',
      '3': 7,
      '4': 1,
      '5': 11,
      '6': '.google.protobuf.Struct',
      '10': 'payload'
    },
  ],
};

/// Descriptor for `CreateJobRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List createJobRequestDescriptor = $convert.base64Decode(
    'ChBDcmVhdGVKb2JSZXF1ZXN0EhIKBG5hbWUYASABKAlSBG5hbWUSIAoLZGVzY3JpcHRpb24YAi'
    'ABKAlSC2Rlc2NyaXB0aW9uEicKD2Nyb25fZXhwcmVzc2lvbhgDIAEoCVIOY3JvbkV4cHJlc3Np'
    'b24SGgoIdGltZXpvbmUYBCABKAlSCHRpbWV6b25lEh8KC3RhcmdldF90eXBlGAUgASgJUgp0YX'
    'JnZXRUeXBlEhYKBnRhcmdldBgGIAEoCVIGdGFyZ2V0EjEKB3BheWxvYWQYByABKAsyFy5nb29n'
    'bGUucHJvdG9idWYuU3RydWN0UgdwYXlsb2Fk');

@$core.Deprecated('Use createJobResponseDescriptor instead')
const CreateJobResponse$json = {
  '1': 'CreateJobResponse',
  '2': [
    {
      '1': 'job',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.scheduler.v1.Job',
      '10': 'job'
    },
  ],
};

/// Descriptor for `CreateJobResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List createJobResponseDescriptor = $convert.base64Decode(
    'ChFDcmVhdGVKb2JSZXNwb25zZRIvCgNqb2IYASABKAsyHS5rMXMwLnN5c3RlbS5zY2hlZHVsZX'
    'IudjEuSm9iUgNqb2I=');

@$core.Deprecated('Use getJobRequestDescriptor instead')
const GetJobRequest$json = {
  '1': 'GetJobRequest',
  '2': [
    {'1': 'job_id', '3': 1, '4': 1, '5': 9, '10': 'jobId'},
  ],
};

/// Descriptor for `GetJobRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List getJobRequestDescriptor = $convert
    .base64Decode('Cg1HZXRKb2JSZXF1ZXN0EhUKBmpvYl9pZBgBIAEoCVIFam9iSWQ=');

@$core.Deprecated('Use getJobResponseDescriptor instead')
const GetJobResponse$json = {
  '1': 'GetJobResponse',
  '2': [
    {
      '1': 'job',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.scheduler.v1.Job',
      '10': 'job'
    },
  ],
};

/// Descriptor for `GetJobResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List getJobResponseDescriptor = $convert.base64Decode(
    'Cg5HZXRKb2JSZXNwb25zZRIvCgNqb2IYASABKAsyHS5rMXMwLnN5c3RlbS5zY2hlZHVsZXIudj'
    'EuSm9iUgNqb2I=');

@$core.Deprecated('Use listJobsRequestDescriptor instead')
const ListJobsRequest$json = {
  '1': 'ListJobsRequest',
  '2': [
    {'1': 'status', '3': 1, '4': 1, '5': 9, '10': 'status'},
    {
      '1': 'pagination',
      '3': 2,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.common.v1.Pagination',
      '10': 'pagination'
    },
  ],
};

/// Descriptor for `ListJobsRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List listJobsRequestDescriptor = $convert.base64Decode(
    'Cg9MaXN0Sm9ic1JlcXVlc3QSFgoGc3RhdHVzGAEgASgJUgZzdGF0dXMSQQoKcGFnaW5hdGlvbh'
    'gCIAEoCzIhLmsxczAuc3lzdGVtLmNvbW1vbi52MS5QYWdpbmF0aW9uUgpwYWdpbmF0aW9u');

@$core.Deprecated('Use listJobsResponseDescriptor instead')
const ListJobsResponse$json = {
  '1': 'ListJobsResponse',
  '2': [
    {
      '1': 'jobs',
      '3': 1,
      '4': 3,
      '5': 11,
      '6': '.k1s0.system.scheduler.v1.Job',
      '10': 'jobs'
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

/// Descriptor for `ListJobsResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List listJobsResponseDescriptor = $convert.base64Decode(
    'ChBMaXN0Sm9ic1Jlc3BvbnNlEjEKBGpvYnMYASADKAsyHS5rMXMwLnN5c3RlbS5zY2hlZHVsZX'
    'IudjEuSm9iUgRqb2JzEkcKCnBhZ2luYXRpb24YAiABKAsyJy5rMXMwLnN5c3RlbS5jb21tb24u'
    'djEuUGFnaW5hdGlvblJlc3VsdFIKcGFnaW5hdGlvbg==');

@$core.Deprecated('Use updateJobRequestDescriptor instead')
const UpdateJobRequest$json = {
  '1': 'UpdateJobRequest',
  '2': [
    {'1': 'job_id', '3': 1, '4': 1, '5': 9, '10': 'jobId'},
    {'1': 'name', '3': 2, '4': 1, '5': 9, '10': 'name'},
    {'1': 'description', '3': 3, '4': 1, '5': 9, '10': 'description'},
    {'1': 'cron_expression', '3': 4, '4': 1, '5': 9, '10': 'cronExpression'},
    {'1': 'timezone', '3': 5, '4': 1, '5': 9, '10': 'timezone'},
    {'1': 'target_type', '3': 6, '4': 1, '5': 9, '10': 'targetType'},
    {'1': 'target', '3': 7, '4': 1, '5': 9, '10': 'target'},
    {
      '1': 'payload',
      '3': 8,
      '4': 1,
      '5': 11,
      '6': '.google.protobuf.Struct',
      '10': 'payload'
    },
  ],
};

/// Descriptor for `UpdateJobRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List updateJobRequestDescriptor = $convert.base64Decode(
    'ChBVcGRhdGVKb2JSZXF1ZXN0EhUKBmpvYl9pZBgBIAEoCVIFam9iSWQSEgoEbmFtZRgCIAEoCV'
    'IEbmFtZRIgCgtkZXNjcmlwdGlvbhgDIAEoCVILZGVzY3JpcHRpb24SJwoPY3Jvbl9leHByZXNz'
    'aW9uGAQgASgJUg5jcm9uRXhwcmVzc2lvbhIaCgh0aW1lem9uZRgFIAEoCVIIdGltZXpvbmUSHw'
    'oLdGFyZ2V0X3R5cGUYBiABKAlSCnRhcmdldFR5cGUSFgoGdGFyZ2V0GAcgASgJUgZ0YXJnZXQS'
    'MQoHcGF5bG9hZBgIIAEoCzIXLmdvb2dsZS5wcm90b2J1Zi5TdHJ1Y3RSB3BheWxvYWQ=');

@$core.Deprecated('Use updateJobResponseDescriptor instead')
const UpdateJobResponse$json = {
  '1': 'UpdateJobResponse',
  '2': [
    {
      '1': 'job',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.scheduler.v1.Job',
      '10': 'job'
    },
  ],
};

/// Descriptor for `UpdateJobResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List updateJobResponseDescriptor = $convert.base64Decode(
    'ChFVcGRhdGVKb2JSZXNwb25zZRIvCgNqb2IYASABKAsyHS5rMXMwLnN5c3RlbS5zY2hlZHVsZX'
    'IudjEuSm9iUgNqb2I=');

@$core.Deprecated('Use deleteJobRequestDescriptor instead')
const DeleteJobRequest$json = {
  '1': 'DeleteJobRequest',
  '2': [
    {'1': 'job_id', '3': 1, '4': 1, '5': 9, '10': 'jobId'},
  ],
};

/// Descriptor for `DeleteJobRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List deleteJobRequestDescriptor = $convert
    .base64Decode('ChBEZWxldGVKb2JSZXF1ZXN0EhUKBmpvYl9pZBgBIAEoCVIFam9iSWQ=');

@$core.Deprecated('Use deleteJobResponseDescriptor instead')
const DeleteJobResponse$json = {
  '1': 'DeleteJobResponse',
  '2': [
    {'1': 'success', '3': 1, '4': 1, '5': 8, '10': 'success'},
    {'1': 'message', '3': 2, '4': 1, '5': 9, '10': 'message'},
  ],
};

/// Descriptor for `DeleteJobResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List deleteJobResponseDescriptor = $convert.base64Decode(
    'ChFEZWxldGVKb2JSZXNwb25zZRIYCgdzdWNjZXNzGAEgASgIUgdzdWNjZXNzEhgKB21lc3NhZ2'
    'UYAiABKAlSB21lc3NhZ2U=');

@$core.Deprecated('Use pauseJobRequestDescriptor instead')
const PauseJobRequest$json = {
  '1': 'PauseJobRequest',
  '2': [
    {'1': 'job_id', '3': 1, '4': 1, '5': 9, '10': 'jobId'},
  ],
};

/// Descriptor for `PauseJobRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List pauseJobRequestDescriptor = $convert
    .base64Decode('Cg9QYXVzZUpvYlJlcXVlc3QSFQoGam9iX2lkGAEgASgJUgVqb2JJZA==');

@$core.Deprecated('Use pauseJobResponseDescriptor instead')
const PauseJobResponse$json = {
  '1': 'PauseJobResponse',
  '2': [
    {
      '1': 'job',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.scheduler.v1.Job',
      '10': 'job'
    },
  ],
};

/// Descriptor for `PauseJobResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List pauseJobResponseDescriptor = $convert.base64Decode(
    'ChBQYXVzZUpvYlJlc3BvbnNlEi8KA2pvYhgBIAEoCzIdLmsxczAuc3lzdGVtLnNjaGVkdWxlci'
    '52MS5Kb2JSA2pvYg==');

@$core.Deprecated('Use resumeJobRequestDescriptor instead')
const ResumeJobRequest$json = {
  '1': 'ResumeJobRequest',
  '2': [
    {'1': 'job_id', '3': 1, '4': 1, '5': 9, '10': 'jobId'},
  ],
};

/// Descriptor for `ResumeJobRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List resumeJobRequestDescriptor = $convert
    .base64Decode('ChBSZXN1bWVKb2JSZXF1ZXN0EhUKBmpvYl9pZBgBIAEoCVIFam9iSWQ=');

@$core.Deprecated('Use resumeJobResponseDescriptor instead')
const ResumeJobResponse$json = {
  '1': 'ResumeJobResponse',
  '2': [
    {
      '1': 'job',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.scheduler.v1.Job',
      '10': 'job'
    },
  ],
};

/// Descriptor for `ResumeJobResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List resumeJobResponseDescriptor = $convert.base64Decode(
    'ChFSZXN1bWVKb2JSZXNwb25zZRIvCgNqb2IYASABKAsyHS5rMXMwLnN5c3RlbS5zY2hlZHVsZX'
    'IudjEuSm9iUgNqb2I=');

@$core.Deprecated('Use triggerJobRequestDescriptor instead')
const TriggerJobRequest$json = {
  '1': 'TriggerJobRequest',
  '2': [
    {'1': 'job_id', '3': 1, '4': 1, '5': 9, '10': 'jobId'},
  ],
};

/// Descriptor for `TriggerJobRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List triggerJobRequestDescriptor = $convert
    .base64Decode('ChFUcmlnZ2VySm9iUmVxdWVzdBIVCgZqb2JfaWQYASABKAlSBWpvYklk');

@$core.Deprecated('Use triggerJobResponseDescriptor instead')
const TriggerJobResponse$json = {
  '1': 'TriggerJobResponse',
  '2': [
    {'1': 'execution_id', '3': 1, '4': 1, '5': 9, '10': 'executionId'},
    {'1': 'job_id', '3': 2, '4': 1, '5': 9, '10': 'jobId'},
    {'1': 'status', '3': 3, '4': 1, '5': 9, '10': 'status'},
    {
      '1': 'triggered_at',
      '3': 4,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.common.v1.Timestamp',
      '10': 'triggeredAt'
    },
  ],
};

/// Descriptor for `TriggerJobResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List triggerJobResponseDescriptor = $convert.base64Decode(
    'ChJUcmlnZ2VySm9iUmVzcG9uc2USIQoMZXhlY3V0aW9uX2lkGAEgASgJUgtleGVjdXRpb25JZB'
    'IVCgZqb2JfaWQYAiABKAlSBWpvYklkEhYKBnN0YXR1cxgDIAEoCVIGc3RhdHVzEkMKDHRyaWdn'
    'ZXJlZF9hdBgEIAEoCzIgLmsxczAuc3lzdGVtLmNvbW1vbi52MS5UaW1lc3RhbXBSC3RyaWdnZX'
    'JlZEF0');

@$core.Deprecated('Use getJobExecutionRequestDescriptor instead')
const GetJobExecutionRequest$json = {
  '1': 'GetJobExecutionRequest',
  '2': [
    {'1': 'execution_id', '3': 1, '4': 1, '5': 9, '10': 'executionId'},
  ],
};

/// Descriptor for `GetJobExecutionRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List getJobExecutionRequestDescriptor =
    $convert.base64Decode(
        'ChZHZXRKb2JFeGVjdXRpb25SZXF1ZXN0EiEKDGV4ZWN1dGlvbl9pZBgBIAEoCVILZXhlY3V0aW'
        '9uSWQ=');

@$core.Deprecated('Use getJobExecutionResponseDescriptor instead')
const GetJobExecutionResponse$json = {
  '1': 'GetJobExecutionResponse',
  '2': [
    {
      '1': 'execution',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.scheduler.v1.JobExecution',
      '10': 'execution'
    },
  ],
};

/// Descriptor for `GetJobExecutionResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List getJobExecutionResponseDescriptor =
    $convert.base64Decode(
        'ChdHZXRKb2JFeGVjdXRpb25SZXNwb25zZRJECglleGVjdXRpb24YASABKAsyJi5rMXMwLnN5c3'
        'RlbS5zY2hlZHVsZXIudjEuSm9iRXhlY3V0aW9uUglleGVjdXRpb24=');

@$core.Deprecated('Use listExecutionsRequestDescriptor instead')
const ListExecutionsRequest$json = {
  '1': 'ListExecutionsRequest',
  '2': [
    {'1': 'job_id', '3': 1, '4': 1, '5': 9, '10': 'jobId'},
    {
      '1': 'pagination',
      '3': 2,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.common.v1.Pagination',
      '10': 'pagination'
    },
    {'1': 'status', '3': 3, '4': 1, '5': 9, '9': 0, '10': 'status', '17': true},
    {
      '1': 'from',
      '3': 4,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.common.v1.Timestamp',
      '9': 1,
      '10': 'from',
      '17': true
    },
    {
      '1': 'to',
      '3': 5,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.common.v1.Timestamp',
      '9': 2,
      '10': 'to',
      '17': true
    },
  ],
  '8': [
    {'1': '_status'},
    {'1': '_from'},
    {'1': '_to'},
  ],
};

/// Descriptor for `ListExecutionsRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List listExecutionsRequestDescriptor = $convert.base64Decode(
    'ChVMaXN0RXhlY3V0aW9uc1JlcXVlc3QSFQoGam9iX2lkGAEgASgJUgVqb2JJZBJBCgpwYWdpbm'
    'F0aW9uGAIgASgLMiEuazFzMC5zeXN0ZW0uY29tbW9uLnYxLlBhZ2luYXRpb25SCnBhZ2luYXRp'
    'b24SGwoGc3RhdHVzGAMgASgJSABSBnN0YXR1c4gBARI5CgRmcm9tGAQgASgLMiAuazFzMC5zeX'
    'N0ZW0uY29tbW9uLnYxLlRpbWVzdGFtcEgBUgRmcm9tiAEBEjUKAnRvGAUgASgLMiAuazFzMC5z'
    'eXN0ZW0uY29tbW9uLnYxLlRpbWVzdGFtcEgCUgJ0b4gBAUIJCgdfc3RhdHVzQgcKBV9mcm9tQg'
    'UKA190bw==');

@$core.Deprecated('Use listExecutionsResponseDescriptor instead')
const ListExecutionsResponse$json = {
  '1': 'ListExecutionsResponse',
  '2': [
    {
      '1': 'executions',
      '3': 1,
      '4': 3,
      '5': 11,
      '6': '.k1s0.system.scheduler.v1.JobExecution',
      '10': 'executions'
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

/// Descriptor for `ListExecutionsResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List listExecutionsResponseDescriptor = $convert.base64Decode(
    'ChZMaXN0RXhlY3V0aW9uc1Jlc3BvbnNlEkYKCmV4ZWN1dGlvbnMYASADKAsyJi5rMXMwLnN5c3'
    'RlbS5zY2hlZHVsZXIudjEuSm9iRXhlY3V0aW9uUgpleGVjdXRpb25zEkcKCnBhZ2luYXRpb24Y'
    'AiABKAsyJy5rMXMwLnN5c3RlbS5jb21tb24udjEuUGFnaW5hdGlvblJlc3VsdFIKcGFnaW5hdG'
    'lvbg==');

@$core.Deprecated('Use jobExecutionDescriptor instead')
const JobExecution$json = {
  '1': 'JobExecution',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 9, '10': 'id'},
    {'1': 'job_id', '3': 2, '4': 1, '5': 9, '10': 'jobId'},
    {'1': 'status', '3': 3, '4': 1, '5': 9, '10': 'status'},
    {'1': 'triggered_by', '3': 4, '4': 1, '5': 9, '10': 'triggeredBy'},
    {
      '1': 'started_at',
      '3': 5,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.common.v1.Timestamp',
      '10': 'startedAt'
    },
    {
      '1': 'finished_at',
      '3': 6,
      '4': 1,
      '5': 11,
      '6': '.k1s0.system.common.v1.Timestamp',
      '9': 0,
      '10': 'finishedAt',
      '17': true
    },
    {
      '1': 'duration_ms',
      '3': 7,
      '4': 1,
      '5': 4,
      '9': 1,
      '10': 'durationMs',
      '17': true
    },
    {
      '1': 'error_message',
      '3': 8,
      '4': 1,
      '5': 9,
      '9': 2,
      '10': 'errorMessage',
      '17': true
    },
  ],
  '8': [
    {'1': '_finished_at'},
    {'1': '_duration_ms'},
    {'1': '_error_message'},
  ],
};

/// Descriptor for `JobExecution`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List jobExecutionDescriptor = $convert.base64Decode(
    'CgxKb2JFeGVjdXRpb24SDgoCaWQYASABKAlSAmlkEhUKBmpvYl9pZBgCIAEoCVIFam9iSWQSFg'
    'oGc3RhdHVzGAMgASgJUgZzdGF0dXMSIQoMdHJpZ2dlcmVkX2J5GAQgASgJUgt0cmlnZ2VyZWRC'
    'eRI/CgpzdGFydGVkX2F0GAUgASgLMiAuazFzMC5zeXN0ZW0uY29tbW9uLnYxLlRpbWVzdGFtcF'
    'IJc3RhcnRlZEF0EkYKC2ZpbmlzaGVkX2F0GAYgASgLMiAuazFzMC5zeXN0ZW0uY29tbW9uLnYx'
    'LlRpbWVzdGFtcEgAUgpmaW5pc2hlZEF0iAEBEiQKC2R1cmF0aW9uX21zGAcgASgESAFSCmR1cm'
    'F0aW9uTXOIAQESKAoNZXJyb3JfbWVzc2FnZRgIIAEoCUgCUgxlcnJvck1lc3NhZ2WIAQFCDgoM'
    'X2ZpbmlzaGVkX2F0Qg4KDF9kdXJhdGlvbl9tc0IQCg5fZXJyb3JfbWVzc2FnZQ==');
