import 'job.dart';
import 'error.dart';

abstract class SchedulerClient {
  Future<Job> createJob(JobRequest request);
  Future<void> cancelJob(String jobId);
  Future<void> pauseJob(String jobId);
  Future<void> resumeJob(String jobId);
  Future<Job> getJob(String jobId);
  Future<List<Job>> listJobs(JobFilter filter);
  Future<List<JobExecution>> getExecutions(String jobId);
}

class InMemorySchedulerClient implements SchedulerClient {
  final Map<String, Job> _jobs = {};
  int _seq = 0;

  Map<String, Job> get jobs => Map.unmodifiable(_jobs);

  @override
  Future<Job> createJob(JobRequest request) async {
    _seq++;
    final id = 'job-${_seq.toString().padLeft(3, '0')}';
    final job = Job(
      id: id,
      name: request.name,
      schedule: request.schedule,
      status: JobStatus.pending,
      payload: request.payload,
      maxRetries: request.maxRetries,
      timeoutSecs: request.timeoutSecs,
      createdAt: DateTime.now(),
    );
    _jobs[id] = job;
    return job;
  }

  @override
  Future<void> cancelJob(String jobId) async {
    final job = _jobs[jobId];
    if (job == null) {
      throw SchedulerError('Job not found: $jobId', 'JOB_NOT_FOUND');
    }
    _jobs[jobId] = job.copyWith(status: JobStatus.cancelled);
  }

  @override
  Future<void> pauseJob(String jobId) async {
    final job = _jobs[jobId];
    if (job == null) {
      throw SchedulerError('Job not found: $jobId', 'JOB_NOT_FOUND');
    }
    _jobs[jobId] = job.copyWith(status: JobStatus.paused);
  }

  @override
  Future<void> resumeJob(String jobId) async {
    final job = _jobs[jobId];
    if (job == null) {
      throw SchedulerError('Job not found: $jobId', 'JOB_NOT_FOUND');
    }
    _jobs[jobId] = job.copyWith(status: JobStatus.pending);
  }

  @override
  Future<Job> getJob(String jobId) async {
    final job = _jobs[jobId];
    if (job == null) {
      throw SchedulerError('Job not found: $jobId', 'JOB_NOT_FOUND');
    }
    return job;
  }

  @override
  Future<List<Job>> listJobs(JobFilter filter) async {
    var result = _jobs.values.toList();
    if (filter.status != null) {
      result = result.where((j) => j.status == filter.status).toList();
    }
    if (filter.namePrefix != null && filter.namePrefix!.isNotEmpty) {
      result =
          result.where((j) => j.name.startsWith(filter.namePrefix!)).toList();
    }
    return result;
  }

  @override
  Future<List<JobExecution>> getExecutions(String jobId) async {
    return [];
  }
}
