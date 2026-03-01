import 'dart:convert';

import 'package:http/http.dart' as http;

import 'client.dart';
import 'error.dart';
import 'job.dart';

// --- Wire format helpers ---

Map<String, dynamic> _scheduleToJson(Schedule schedule) {
  if (schedule is CronSchedule) {
    return {'type': 'cron', 'cron': schedule.expression};
  } else if (schedule is OneShotSchedule) {
    return {'type': 'one_shot', 'one_shot': schedule.runAt.toIso8601String()};
  } else if (schedule is IntervalSchedule) {
    return {
      'type': 'interval',
      'interval_secs': schedule.interval.inSeconds,
    };
  }
  throw ArgumentError('Unknown Schedule type: ${schedule.runtimeType}');
}

Schedule _scheduleFromJson(Map<String, dynamic> json) {
  final type = json['type'] as String;
  switch (type) {
    case 'cron':
      return Schedule.cron(json['cron'] as String? ?? '');
    case 'one_shot':
      return Schedule.oneShot(DateTime.parse(json['one_shot'] as String));
    case 'interval':
      final secs = (json['interval_secs'] as num?)?.toInt() ?? 0;
      return Schedule.interval(Duration(seconds: secs));
    default:
      return Schedule.cron('');
  }
}

Job _jobFromJson(Map<String, dynamic> json) {
  return Job(
    id: json['id'] as String,
    name: json['name'] as String,
    schedule: _scheduleFromJson(json['schedule'] as Map<String, dynamic>),
    status: JobStatus.values.firstWhere(
      (s) => s.name == (json['status'] as String),
      orElse: () => JobStatus.pending,
    ),
    payload: (json['payload'] as Map<String, dynamic>?) ?? {},
    maxRetries: (json['max_retries'] as num?)?.toInt() ?? 3,
    timeoutSecs: (json['timeout_secs'] as num?)?.toInt() ?? 60,
    createdAt: DateTime.parse(json['created_at'] as String),
    nextRunAt: json['next_run_at'] != null
        ? DateTime.parse(json['next_run_at'] as String)
        : null,
  );
}

JobExecution _executionFromJson(Map<String, dynamic> json) {
  return JobExecution(
    id: json['id'] as String,
    jobId: json['job_id'] as String,
    startedAt: DateTime.parse(json['started_at'] as String),
    finishedAt: json['finished_at'] != null
        ? DateTime.parse(json['finished_at'] as String)
        : null,
    result: json['result'] as String? ?? '',
    error: json['error'] as String?,
  );
}

SchedulerError _parseError(int statusCode, String body, String op) {
  final msg = body.trim().isNotEmpty ? body.trim() : 'status $statusCode';
  if (statusCode == 404) {
    return SchedulerError('Job not found: $msg', 'JOB_NOT_FOUND');
  }
  if (statusCode == 408 || statusCode == 504) {
    return SchedulerError('$op timed out', 'TIMEOUT');
  }
  return SchedulerError('$op failed ($statusCode): $msg', 'SERVER_ERROR');
}

/// GrpcSchedulerClient は scheduler-server への HTTP クライアント。
/// 実際の gRPC プロトコルではなく HTTP REST API を使用するが、
/// gRPC サーバーのエンドポイント（:8080）に接続する。
class GrpcSchedulerClient implements SchedulerClient {
  final String _baseUrl;
  final http.Client _http;

  GrpcSchedulerClient(String serverUrl, {http.Client? httpClient})
      : _baseUrl = serverUrl.startsWith('http')
            ? serverUrl.replaceAll(RegExp(r'/$'), '')
            : 'http://${serverUrl.replaceAll(RegExp(r'/$'), '')}',
        _http = httpClient ?? http.Client();

  Future<dynamic> _request(
    String method,
    String path, {
    Map<String, dynamic>? body,
  }) async {
    final uri = Uri.parse('$_baseUrl$path');
    http.Response response;
    final headers = <String, String>{
      if (body != null) 'Content-Type': 'application/json',
    };
    final encodedBody = body != null ? json.encode(body) : null;

    switch (method) {
      case 'GET':
        response = await _http.get(uri, headers: headers);
      case 'POST':
        response = await _http.post(
          uri,
          headers: headers,
          body: encodedBody ?? '{}',
        );
      default:
        throw ArgumentError('Unsupported HTTP method: $method');
    }

    if (response.statusCode >= 200 && response.statusCode < 300) {
      if (response.body.isEmpty) return null;
      return json.decode(response.body);
    }
    throw _parseError(response.statusCode, response.body, '$method $path');
  }

  @override
  Future<Job> createJob(JobRequest request) async {
    final body = {
      'name': request.name,
      'schedule': _scheduleToJson(request.schedule),
      'payload': request.payload,
      'max_retries': request.maxRetries,
      'timeout_secs': request.timeoutSecs,
    };
    final result = await _request('POST', '/api/v1/jobs', body: body)
        as Map<String, dynamic>;
    return _jobFromJson(result);
  }

  @override
  Future<void> cancelJob(String jobId) async {
    await _request(
      'POST',
      '/api/v1/jobs/${Uri.encodeComponent(jobId)}/cancel',
      body: {},
    );
  }

  @override
  Future<void> pauseJob(String jobId) async {
    await _request(
      'POST',
      '/api/v1/jobs/${Uri.encodeComponent(jobId)}/pause',
      body: {},
    );
  }

  @override
  Future<void> resumeJob(String jobId) async {
    await _request(
      'POST',
      '/api/v1/jobs/${Uri.encodeComponent(jobId)}/resume',
      body: {},
    );
  }

  @override
  Future<Job> getJob(String jobId) async {
    final result = await _request(
      'GET',
      '/api/v1/jobs/${Uri.encodeComponent(jobId)}',
    ) as Map<String, dynamic>;
    return _jobFromJson(result);
  }

  @override
  Future<List<Job>> listJobs(JobFilter filter) async {
    final params = <String, String>{};
    if (filter.status != null) params['status'] = filter.status!.name;
    if (filter.namePrefix != null && filter.namePrefix!.isNotEmpty) {
      params['name_prefix'] = filter.namePrefix!;
    }

    final uri = Uri.parse('$_baseUrl/api/v1/jobs')
        .replace(queryParameters: params.isNotEmpty ? params : null);
    final response = await _http.get(uri);

    if (response.statusCode >= 200 && response.statusCode < 300) {
      final list = json.decode(response.body) as List<dynamic>;
      return list
          .cast<Map<String, dynamic>>()
          .map(_jobFromJson)
          .toList();
    }
    throw _parseError(response.statusCode, response.body, 'GET /api/v1/jobs');
  }

  @override
  Future<List<JobExecution>> getExecutions(String jobId) async {
    final result = await _request(
      'GET',
      '/api/v1/jobs/${Uri.encodeComponent(jobId)}/executions',
    ) as List<dynamic>;
    return result
        .cast<Map<String, dynamic>>()
        .map(_executionFromJson)
        .toList();
  }

  /// 接続を閉じてリソースを解放する。
  Future<void> close() async {
    _http.close();
  }
}
