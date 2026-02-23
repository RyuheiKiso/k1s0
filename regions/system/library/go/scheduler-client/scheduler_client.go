package schedulerclient

import (
	"context"
	"encoding/json"
	"fmt"
	"sync"
	"time"
)

// JobStatus はジョブの状態。
type JobStatus string

const (
	JobStatusPending   JobStatus = "pending"
	JobStatusRunning   JobStatus = "running"
	JobStatusCompleted JobStatus = "completed"
	JobStatusFailed    JobStatus = "failed"
	JobStatusPaused    JobStatus = "paused"
	JobStatusCancelled JobStatus = "cancelled"
)

// Schedule はジョブのスケジュール。
type Schedule struct {
	Type     string         `json:"type"` // "cron", "one_shot", "interval"
	Cron     string         `json:"cron,omitempty"`
	OneShot  *time.Time     `json:"one_shot,omitempty"`
	Interval *time.Duration `json:"interval,omitempty"`
}

// JobRequest はジョブ登録リクエスト。
type JobRequest struct {
	Name        string          `json:"name"`
	Schedule    Schedule        `json:"schedule"`
	Payload     json.RawMessage `json:"payload"`
	MaxRetries  uint32          `json:"max_retries"`
	TimeoutSecs uint64          `json:"timeout_secs"`
}

// Job はジョブ情報。
type Job struct {
	ID          string          `json:"id"`
	Name        string          `json:"name"`
	Schedule    Schedule        `json:"schedule"`
	Status      JobStatus       `json:"status"`
	Payload     json.RawMessage `json:"payload"`
	MaxRetries  uint32          `json:"max_retries"`
	TimeoutSecs uint64          `json:"timeout_secs"`
	CreatedAt   time.Time       `json:"created_at"`
	NextRunAt   *time.Time      `json:"next_run_at,omitempty"`
}

// JobFilter はジョブ一覧取得フィルター。
type JobFilter struct {
	Status     *JobStatus `json:"status,omitempty"`
	NamePrefix *string    `json:"name_prefix,omitempty"`
}

// JobExecution は実行履歴。
type JobExecution struct {
	ID         string     `json:"id"`
	JobID      string     `json:"job_id"`
	StartedAt  time.Time  `json:"started_at"`
	FinishedAt *time.Time `json:"finished_at,omitempty"`
	Result     string     `json:"result"`
	Error      string     `json:"error,omitempty"`
}

// JobCompletedEvent はジョブ完了イベント。
type JobCompletedEvent struct {
	JobID       string    `json:"job_id"`
	ExecutionID string    `json:"execution_id"`
	CompletedAt time.Time `json:"completed_at"`
	Result      string    `json:"result"`
}

// SchedulerClient はスケジューラークライアントのインターフェース。
type SchedulerClient interface {
	CreateJob(ctx context.Context, req JobRequest) (Job, error)
	CancelJob(ctx context.Context, jobID string) error
	PauseJob(ctx context.Context, jobID string) error
	ResumeJob(ctx context.Context, jobID string) error
	GetJob(ctx context.Context, jobID string) (Job, error)
	ListJobs(ctx context.Context, filter JobFilter) ([]Job, error)
	GetExecutions(ctx context.Context, jobID string) ([]JobExecution, error)
}

// InMemoryClient はメモリ内のスケジューラークライアント。
type InMemoryClient struct {
	mu   sync.Mutex
	jobs map[string]Job
	seq  int
}

// NewInMemoryClient は新しい InMemoryClient を生成する。
func NewInMemoryClient() *InMemoryClient {
	return &InMemoryClient{
		jobs: make(map[string]Job),
	}
}

func (c *InMemoryClient) CreateJob(_ context.Context, req JobRequest) (Job, error) {
	c.mu.Lock()
	defer c.mu.Unlock()
	c.seq++
	job := Job{
		ID:          fmt.Sprintf("job-%03d", c.seq),
		Name:        req.Name,
		Schedule:    req.Schedule,
		Status:      JobStatusPending,
		Payload:     req.Payload,
		MaxRetries:  req.MaxRetries,
		TimeoutSecs: req.TimeoutSecs,
		CreatedAt:   time.Now(),
	}
	c.jobs[job.ID] = job
	return job, nil
}

func (c *InMemoryClient) CancelJob(_ context.Context, jobID string) error {
	c.mu.Lock()
	defer c.mu.Unlock()
	job, ok := c.jobs[jobID]
	if !ok {
		return fmt.Errorf("job not found: %s", jobID)
	}
	job.Status = JobStatusCancelled
	c.jobs[jobID] = job
	return nil
}

func (c *InMemoryClient) PauseJob(_ context.Context, jobID string) error {
	c.mu.Lock()
	defer c.mu.Unlock()
	job, ok := c.jobs[jobID]
	if !ok {
		return fmt.Errorf("job not found: %s", jobID)
	}
	job.Status = JobStatusPaused
	c.jobs[jobID] = job
	return nil
}

func (c *InMemoryClient) ResumeJob(_ context.Context, jobID string) error {
	c.mu.Lock()
	defer c.mu.Unlock()
	job, ok := c.jobs[jobID]
	if !ok {
		return fmt.Errorf("job not found: %s", jobID)
	}
	job.Status = JobStatusPending
	c.jobs[jobID] = job
	return nil
}

func (c *InMemoryClient) GetJob(_ context.Context, jobID string) (Job, error) {
	c.mu.Lock()
	defer c.mu.Unlock()
	job, ok := c.jobs[jobID]
	if !ok {
		return Job{}, fmt.Errorf("job not found: %s", jobID)
	}
	return job, nil
}

func (c *InMemoryClient) ListJobs(_ context.Context, filter JobFilter) ([]Job, error) {
	c.mu.Lock()
	defer c.mu.Unlock()
	var result []Job
	for _, job := range c.jobs {
		if filter.Status != nil && job.Status != *filter.Status {
			continue
		}
		if filter.NamePrefix != nil && len(*filter.NamePrefix) > 0 {
			if len(job.Name) < len(*filter.NamePrefix) || job.Name[:len(*filter.NamePrefix)] != *filter.NamePrefix {
				continue
			}
		}
		result = append(result, job)
	}
	return result, nil
}

func (c *InMemoryClient) GetExecutions(_ context.Context, _ string) ([]JobExecution, error) {
	return []JobExecution{}, nil
}

// Jobs は登録済みジョブ一覧を返す。
func (c *InMemoryClient) Jobs() map[string]Job {
	c.mu.Lock()
	defer c.mu.Unlock()
	result := make(map[string]Job, len(c.jobs))
	for k, v := range c.jobs {
		result[k] = v
	}
	return result
}
