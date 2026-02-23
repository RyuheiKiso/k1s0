enum JobStatus { pending, running, completed, failed, paused, cancelled }

sealed class Schedule {
  const Schedule();
  factory Schedule.cron(String expression) = CronSchedule;
  factory Schedule.oneShot(DateTime runAt) = OneShotSchedule;
  factory Schedule.interval(Duration interval) = IntervalSchedule;
}

class CronSchedule extends Schedule {
  final String expression;
  const CronSchedule(this.expression);
}

class OneShotSchedule extends Schedule {
  final DateTime runAt;
  const OneShotSchedule(this.runAt);
}

class IntervalSchedule extends Schedule {
  final Duration interval;
  const IntervalSchedule(this.interval);
}

class JobRequest {
  final String name;
  final Schedule schedule;
  final Map<String, dynamic> payload;
  final int maxRetries;
  final int timeoutSecs;

  const JobRequest({
    required this.name,
    required this.schedule,
    this.payload = const {},
    this.maxRetries = 3,
    this.timeoutSecs = 60,
  });
}

class Job {
  final String id;
  final String name;
  final Schedule schedule;
  final JobStatus status;
  final Map<String, dynamic> payload;
  final int maxRetries;
  final int timeoutSecs;
  final DateTime createdAt;
  final DateTime? nextRunAt;

  const Job({
    required this.id,
    required this.name,
    required this.schedule,
    required this.status,
    this.payload = const {},
    this.maxRetries = 3,
    this.timeoutSecs = 60,
    required this.createdAt,
    this.nextRunAt,
  });

  Job copyWith({JobStatus? status}) {
    return Job(
      id: id,
      name: name,
      schedule: schedule,
      status: status ?? this.status,
      payload: payload,
      maxRetries: maxRetries,
      timeoutSecs: timeoutSecs,
      createdAt: createdAt,
      nextRunAt: nextRunAt,
    );
  }
}

class JobFilter {
  final JobStatus? status;
  final String? namePrefix;

  const JobFilter({this.status, this.namePrefix});
}

class JobExecution {
  final String id;
  final String jobId;
  final DateTime startedAt;
  final DateTime? finishedAt;
  final String result;
  final String? error;

  const JobExecution({
    required this.id,
    required this.jobId,
    required this.startedAt,
    this.finishedAt,
    required this.result,
    this.error,
  });
}
