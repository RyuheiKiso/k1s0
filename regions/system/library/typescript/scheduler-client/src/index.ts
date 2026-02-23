export type JobStatus = 'pending' | 'running' | 'completed' | 'failed' | 'paused' | 'cancelled';

export type Schedule =
  | { type: 'cron'; expression: string }
  | { type: 'one_shot'; runAt: Date }
  | { type: 'interval'; intervalMs: number };

export interface JobRequest {
  name: string;
  schedule: Schedule;
  payload: unknown;
  maxRetries?: number;
  timeoutSecs?: number;
}

export interface Job {
  id: string;
  name: string;
  schedule: Schedule;
  status: JobStatus;
  payload: unknown;
  maxRetries: number;
  timeoutSecs: number;
  createdAt: Date;
  nextRunAt?: Date;
}

export interface JobFilter {
  status?: JobStatus;
  namePrefix?: string;
}

export interface JobExecution {
  id: string;
  jobId: string;
  startedAt: Date;
  finishedAt?: Date;
  result: string;
  error?: string;
}

export interface JobCompletedEvent {
  jobId: string;
  executionId: string;
  completedAt: Date;
  result: string;
}

export interface SchedulerClient {
  createJob(req: JobRequest): Promise<Job>;
  cancelJob(jobId: string): Promise<void>;
  pauseJob(jobId: string): Promise<void>;
  resumeJob(jobId: string): Promise<void>;
  getJob(jobId: string): Promise<Job>;
  listJobs(filter?: JobFilter): Promise<Job[]>;
  getExecutions(jobId: string): Promise<JobExecution[]>;
}

export class SchedulerError extends Error {
  constructor(
    message: string,
    public readonly code: 'JOB_NOT_FOUND' | 'INVALID_SCHEDULE' | 'SERVER_ERROR' | 'TIMEOUT',
  ) {
    super(message);
    this.name = 'SchedulerError';
  }
}

export class InMemorySchedulerClient implements SchedulerClient {
  private jobs = new Map<string, Job>();
  private seq = 0;

  async createJob(req: JobRequest): Promise<Job> {
    this.seq++;
    const job: Job = {
      id: `job-${String(this.seq).padStart(3, '0')}`,
      name: req.name,
      schedule: req.schedule,
      status: 'pending',
      payload: req.payload,
      maxRetries: req.maxRetries ?? 3,
      timeoutSecs: req.timeoutSecs ?? 60,
      createdAt: new Date(),
    };
    this.jobs.set(job.id, job);
    return job;
  }

  async cancelJob(jobId: string): Promise<void> {
    const job = this.jobs.get(jobId);
    if (!job) throw new SchedulerError(`Job not found: ${jobId}`, 'JOB_NOT_FOUND');
    job.status = 'cancelled';
  }

  async pauseJob(jobId: string): Promise<void> {
    const job = this.jobs.get(jobId);
    if (!job) throw new SchedulerError(`Job not found: ${jobId}`, 'JOB_NOT_FOUND');
    job.status = 'paused';
  }

  async resumeJob(jobId: string): Promise<void> {
    const job = this.jobs.get(jobId);
    if (!job) throw new SchedulerError(`Job not found: ${jobId}`, 'JOB_NOT_FOUND');
    job.status = 'pending';
  }

  async getJob(jobId: string): Promise<Job> {
    const job = this.jobs.get(jobId);
    if (!job) throw new SchedulerError(`Job not found: ${jobId}`, 'JOB_NOT_FOUND');
    return job;
  }

  async listJobs(filter?: JobFilter): Promise<Job[]> {
    let result = Array.from(this.jobs.values());
    if (filter?.status) {
      result = result.filter((j) => j.status === filter.status);
    }
    if (filter?.namePrefix) {
      const prefix = filter.namePrefix;
      result = result.filter((j) => j.name.startsWith(prefix));
    }
    return result;
  }

  async getExecutions(_jobId: string): Promise<JobExecution[]> {
    return [];
  }

  getAll(): Job[] {
    return Array.from(this.jobs.values());
  }
}
