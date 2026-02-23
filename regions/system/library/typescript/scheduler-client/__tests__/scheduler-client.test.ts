import { describe, it, expect } from 'vitest';
import {
  InMemorySchedulerClient,
  SchedulerError,
  type JobRequest,
} from '../src/index';

describe('InMemorySchedulerClient', () => {
  it('should create a job', async () => {
    const client = new InMemorySchedulerClient();
    const req: JobRequest = {
      name: 'daily-report',
      schedule: { type: 'cron', expression: '0 2 * * *' },
      payload: { reportType: 'daily' },
      maxRetries: 3,
      timeoutSecs: 300,
    };
    const job = await client.createJob(req);
    expect(job.id).toBe('job-001');
    expect(job.name).toBe('daily-report');
    expect(job.status).toBe('pending');
  });

  it('should cancel a job', async () => {
    const client = new InMemorySchedulerClient();
    const job = await client.createJob({
      name: 'test',
      schedule: { type: 'cron', expression: '* * * * *' },
      payload: {},
    });
    await client.cancelJob(job.id);
    const got = await client.getJob(job.id);
    expect(got.status).toBe('cancelled');
  });

  it('should pause and resume a job', async () => {
    const client = new InMemorySchedulerClient();
    const job = await client.createJob({
      name: 'test',
      schedule: { type: 'cron', expression: '* * * * *' },
      payload: {},
    });
    await client.pauseJob(job.id);
    expect((await client.getJob(job.id)).status).toBe('paused');

    await client.resumeJob(job.id);
    expect((await client.getJob(job.id)).status).toBe('pending');
  });

  it('should throw on get non-existent job', async () => {
    const client = new InMemorySchedulerClient();
    await expect(client.getJob('nonexistent')).rejects.toThrow(SchedulerError);
  });

  it('should list jobs with filter', async () => {
    const client = new InMemorySchedulerClient();
    await client.createJob({
      name: 'job-a',
      schedule: { type: 'cron', expression: '* * * * *' },
      payload: {},
    });
    const jobB = await client.createJob({
      name: 'job-b',
      schedule: { type: 'one_shot', runAt: new Date() },
      payload: {},
    });
    await client.pauseJob(jobB.id);

    const paused = await client.listJobs({ status: 'paused' });
    expect(paused).toHaveLength(1);
    expect(paused[0].status).toBe('paused');
  });

  it('should filter by name prefix', async () => {
    const client = new InMemorySchedulerClient();
    await client.createJob({
      name: 'daily-report',
      schedule: { type: 'cron', expression: '0 2 * * *' },
      payload: {},
    });
    await client.createJob({
      name: 'weekly-report',
      schedule: { type: 'cron', expression: '0 2 * * 0' },
      payload: {},
    });

    const daily = await client.listJobs({ namePrefix: 'daily' });
    expect(daily).toHaveLength(1);
    expect(daily[0].name).toBe('daily-report');
  });

  it('should return empty executions', async () => {
    const client = new InMemorySchedulerClient();
    const execs = await client.getExecutions('job-001');
    expect(execs).toHaveLength(0);
  });

  it('should return all jobs', async () => {
    const client = new InMemorySchedulerClient();
    await client.createJob({
      name: 'a',
      schedule: { type: 'interval', intervalMs: 60000 },
      payload: {},
    });
    expect(client.getAll()).toHaveLength(1);
  });

  it('should throw on cancel non-existent job', async () => {
    const client = new InMemorySchedulerClient();
    await expect(client.cancelJob('none')).rejects.toThrow(SchedulerError);
  });

  it('should use default maxRetries and timeoutSecs', async () => {
    const client = new InMemorySchedulerClient();
    const job = await client.createJob({
      name: 'defaults',
      schedule: { type: 'cron', expression: '* * * * *' },
      payload: {},
    });
    expect(job.maxRetries).toBe(3);
    expect(job.timeoutSecs).toBe(60);
  });
});
