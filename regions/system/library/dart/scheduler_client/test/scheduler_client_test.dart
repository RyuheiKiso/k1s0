import 'package:k1s0_scheduler_client/scheduler_client.dart';
import 'package:test/test.dart';

void main() {
  group('InMemorySchedulerClient', () {
    test('creates a job', () async {
      final client = InMemorySchedulerClient();
      final job = await client.createJob(JobRequest(
        name: 'daily-report',
        schedule: Schedule.cron('0 2 * * *'),
        payload: {'report_type': 'daily'},
        maxRetries: 3,
        timeoutSecs: 300,
      ));
      expect(job.id, equals('job-001'));
      expect(job.name, equals('daily-report'));
      expect(job.status, equals(JobStatus.pending));
    });

    test('cancels a job', () async {
      final client = InMemorySchedulerClient();
      final job = await client.createJob(JobRequest(
        name: 'test',
        schedule: Schedule.cron('* * * * *'),
      ));
      await client.cancelJob(job.id);
      final got = await client.getJob(job.id);
      expect(got.status, equals(JobStatus.cancelled));
    });

    test('pauses and resumes a job', () async {
      final client = InMemorySchedulerClient();
      final job = await client.createJob(JobRequest(
        name: 'test',
        schedule: Schedule.cron('* * * * *'),
      ));
      await client.pauseJob(job.id);
      expect((await client.getJob(job.id)).status, equals(JobStatus.paused));

      await client.resumeJob(job.id);
      expect((await client.getJob(job.id)).status, equals(JobStatus.pending));
    });

    test('throws on get non-existent job', () async {
      final client = InMemorySchedulerClient();
      expect(() => client.getJob('nonexistent'), throwsA(isA<SchedulerError>()));
    });

    test('lists jobs with status filter', () async {
      final client = InMemorySchedulerClient();
      await client.createJob(JobRequest(
        name: 'job-a',
        schedule: Schedule.cron('* * * * *'),
      ));
      final jobB = await client.createJob(JobRequest(
        name: 'job-b',
        schedule: Schedule.cron('* * * * *'),
      ));
      await client.pauseJob(jobB.id);

      final paused =
          await client.listJobs(const JobFilter(status: JobStatus.paused));
      expect(paused.length, equals(1));
      expect(paused[0].status, equals(JobStatus.paused));
    });

    test('lists jobs with name prefix filter', () async {
      final client = InMemorySchedulerClient();
      await client.createJob(JobRequest(
        name: 'daily-report',
        schedule: Schedule.cron('0 2 * * *'),
      ));
      await client.createJob(JobRequest(
        name: 'weekly-report',
        schedule: Schedule.cron('0 2 * * 0'),
      ));

      final daily =
          await client.listJobs(const JobFilter(namePrefix: 'daily'));
      expect(daily.length, equals(1));
      expect(daily[0].name, equals('daily-report'));
    });

    test('returns empty executions', () async {
      final client = InMemorySchedulerClient();
      final execs = await client.getExecutions('job-001');
      expect(execs, isEmpty);
    });

    test('schedule variants', () {
      final cron = Schedule.cron('0 * * * *');
      expect(cron, isA<CronSchedule>());

      final oneShot = Schedule.oneShot(DateTime.now());
      expect(oneShot, isA<OneShotSchedule>());

      final interval = Schedule.interval(const Duration(minutes: 10));
      expect(interval, isA<IntervalSchedule>());
    });

    test('throws on cancel non-existent job', () async {
      final client = InMemorySchedulerClient();
      expect(() => client.cancelJob('none'), throwsA(isA<SchedulerError>()));
    });

    test('job completed event', () {
      final event = JobCompletedEvent(
        jobId: 'job-1',
        executionId: 'exec-1',
        completedAt: DateTime.now(),
        result: 'success',
      );
      expect(event.jobId, equals('job-1'));
      expect(event.result, equals('success'));
    });
  });
}
