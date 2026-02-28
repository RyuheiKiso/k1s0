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

// --- Wire format helpers ---

interface ScheduleJson {
  type: string;
  cron?: string;
  one_shot?: string;
  interval_secs?: number;
}

interface JobResponseJson {
  id: string;
  name: string;
  schedule: ScheduleJson;
  status: JobStatus;
  payload: unknown;
  max_retries: number;
  timeout_secs: number;
  created_at: string;
  next_run_at?: string;
}

interface JobExecutionResponseJson {
  id: string;
  job_id: string;
  started_at: string;
  finished_at?: string;
  result: string;
  error?: string;
}

function toScheduleJson(s: Schedule): ScheduleJson {
  if (s.type === 'cron') return { type: 'cron', cron: s.expression };
  if (s.type === 'one_shot') return { type: 'one_shot', one_shot: s.runAt.toISOString() };
  return { type: 'interval', interval_secs: Math.floor(s.intervalMs / 1000) };
}

function fromScheduleJson(sj: ScheduleJson): Schedule {
  if (sj.type === 'cron') return { type: 'cron', expression: sj.cron ?? '' };
  if (sj.type === 'one_shot') return { type: 'one_shot', runAt: new Date(sj.one_shot!) };
  return { type: 'interval', intervalMs: (sj.interval_secs ?? 0) * 1000 };
}

function fromJobJson(r: JobResponseJson): Job {
  return {
    id: r.id,
    name: r.name,
    schedule: fromScheduleJson(r.schedule),
    status: r.status,
    payload: r.payload,
    maxRetries: r.max_retries,
    timeoutSecs: r.timeout_secs,
    createdAt: new Date(r.created_at),
    nextRunAt: r.next_run_at ? new Date(r.next_run_at) : undefined,
  };
}

function parseSchedulerError(status: number, body: string, op: string): SchedulerError {
  const msg = body.trim() || `status ${status}`;
  if (status === 404) return new SchedulerError(`Job not found: ${msg}`, 'JOB_NOT_FOUND');
  if (status === 408 || status === 504) return new SchedulerError(`${op} timed out`, 'TIMEOUT');
  return new SchedulerError(`${op} failed (${status}): ${msg}`, 'SERVER_ERROR');
}

/**
 * GrpcSchedulerClient は scheduler-server への HTTP クライアント。
 * 実際の gRPC プロトコルではなく HTTP REST API を使用するが、
 * gRPC サーバーのエンドポイント（:8080）に接続する。
 */
export class GrpcSchedulerClient implements SchedulerClient {
  private readonly baseUrl: string;

  constructor(serverUrl: string) {
    const url = serverUrl.startsWith('http') ? serverUrl : `http://${serverUrl}`;
    this.baseUrl = url.replace(/\/$/, '');
  }

  private async request<T>(
    method: string,
    path: string,
    body?: unknown,
  ): Promise<T> {
    const url = `${this.baseUrl}${path}`;
    const init: RequestInit = {
      method,
      headers: body !== undefined ? { 'Content-Type': 'application/json' } : {},
      body: body !== undefined ? JSON.stringify(body) : undefined,
    };
    const resp = await fetch(url, init);
    const text = await resp.text();
    if (!resp.ok) {
      throw parseSchedulerError(resp.status, text, `${method} ${path}`);
    }
    return text ? (JSON.parse(text) as T) : (undefined as unknown as T);
  }

  async createJob(req: JobRequest): Promise<Job> {
    const body = {
      name: req.name,
      schedule: toScheduleJson(req.schedule),
      payload: req.payload,
      max_retries: req.maxRetries ?? 3,
      timeout_secs: req.timeoutSecs ?? 60,
    };
    const result = await this.request<JobResponseJson>('POST', '/api/v1/jobs', body);
    return fromJobJson(result);
  }

  async cancelJob(jobId: string): Promise<void> {
    await this.request<void>('POST', `/api/v1/jobs/${encodeURIComponent(jobId)}/cancel`, {});
  }

  async pauseJob(jobId: string): Promise<void> {
    await this.request<void>('POST', `/api/v1/jobs/${encodeURIComponent(jobId)}/pause`, {});
  }

  async resumeJob(jobId: string): Promise<void> {
    await this.request<void>('POST', `/api/v1/jobs/${encodeURIComponent(jobId)}/resume`, {});
  }

  async getJob(jobId: string): Promise<Job> {
    const result = await this.request<JobResponseJson>('GET', `/api/v1/jobs/${encodeURIComponent(jobId)}`);
    return fromJobJson(result);
  }

  async listJobs(filter?: JobFilter): Promise<Job[]> {
    const params = new URLSearchParams();
    if (filter?.status) params.set('status', filter.status);
    if (filter?.namePrefix) params.set('name_prefix', filter.namePrefix);
    const qs = params.size > 0 ? `?${params.toString()}` : '';
    const results = await this.request<JobResponseJson[]>('GET', `/api/v1/jobs${qs}`);
    return results.map(fromJobJson);
  }

  async getExecutions(jobId: string): Promise<JobExecution[]> {
    const results = await this.request<JobExecutionResponseJson[]>(
      'GET',
      `/api/v1/jobs/${encodeURIComponent(jobId)}/executions`,
    );
    return results.map((r) => ({
      id: r.id,
      jobId: r.job_id,
      startedAt: new Date(r.started_at),
      finishedAt: r.finished_at ? new Date(r.finished_at) : undefined,
      result: r.result,
      error: r.error,
    }));
  }

  async close(): Promise<void> {
    // HTTP クライアントはステートレスなので明示的なクローズは不要。
  }
}
