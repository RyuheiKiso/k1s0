package schedulerclient

import (
	"bytes"
	"context"
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"net/url"
	"strings"
	"time"
)

// grpcJobResponse は scheduler-server のジョブレスポンス形式。
type grpcJobResponse struct {
	ID          string          `json:"id"`
	Name        string          `json:"name"`
	Schedule    scheduleJSON    `json:"schedule"`
	Status      string          `json:"status"`
	Payload     json.RawMessage `json:"payload"`
	MaxRetries  uint32          `json:"max_retries"`
	TimeoutSecs uint64          `json:"timeout_secs"`
	CreatedAt   time.Time       `json:"created_at"`
	NextRunAt   *time.Time      `json:"next_run_at,omitempty"`
}

// scheduleJSON は scheduler-server との JSON 交換用スケジュール形式。
type scheduleJSON struct {
	Type     string     `json:"type"`
	Cron     string     `json:"cron,omitempty"`
	OneShot  *time.Time `json:"one_shot,omitempty"`
	Interval *int64     `json:"interval_secs,omitempty"`
}

// grpcJobRequest は scheduler-server へのジョブ登録リクエスト形式。
type grpcJobRequest struct {
	Name        string          `json:"name"`
	Schedule    scheduleJSON    `json:"schedule"`
	Payload     json.RawMessage `json:"payload"`
	MaxRetries  uint32          `json:"max_retries"`
	TimeoutSecs uint64          `json:"timeout_secs"`
}

// grpcJobExecutionResponse は scheduler-server の実行履歴レスポンス形式。
type grpcJobExecutionResponse struct {
	ID         string     `json:"id"`
	JobID      string     `json:"job_id"`
	StartedAt  time.Time  `json:"started_at"`
	FinishedAt *time.Time `json:"finished_at,omitempty"`
	Result     string     `json:"result"`
	Error      string     `json:"error,omitempty"`
}

func toScheduleJSON(s Schedule) scheduleJSON {
	sj := scheduleJSON{Type: s.Type, Cron: s.Cron}
	if s.OneShot != nil {
		sj.OneShot = s.OneShot
	}
	if s.Interval != nil {
		secs := int64(s.Interval.Seconds())
		sj.Interval = &secs
	}
	return sj
}

func fromScheduleJSON(sj scheduleJSON) Schedule {
	s := Schedule{Type: sj.Type, Cron: sj.Cron, OneShot: sj.OneShot}
	if sj.Interval != nil {
		d := time.Duration(*sj.Interval) * time.Second
		s.Interval = &d
	}
	return s
}

func fromJobResponse(r grpcJobResponse) Job {
	return Job{
		ID:          r.ID,
		Name:        r.Name,
		Schedule:    fromScheduleJSON(r.Schedule),
		Status:      JobStatus(r.Status),
		Payload:     r.Payload,
		MaxRetries:  r.MaxRetries,
		TimeoutSecs: r.TimeoutSecs,
		CreatedAt:   r.CreatedAt,
		NextRunAt:   r.NextRunAt,
	}
}

// GrpcSchedulerClient は scheduler-server への HTTP/gRPC クライアント。
// 実際の gRPC プロトコルではなく HTTP REST API を使用するが、
// gRPC サーバーのエンドポイント（:8080）に接続する。
type GrpcSchedulerClient struct {
	baseURL    string
	httpClient *http.Client
}

// NewGrpcSchedulerClient は新しい GrpcSchedulerClient を生成する。
// addr は "scheduler-server:8080" または "http://scheduler-server:8080" の形式。
func NewGrpcSchedulerClient(addr string) (*GrpcSchedulerClient, error) {
	base := addr
	if !strings.HasPrefix(base, "http://") && !strings.HasPrefix(base, "https://") {
		base = "http://" + base
	}
	base = strings.TrimRight(base, "/")
	return &GrpcSchedulerClient{
		baseURL:    base,
		httpClient: &http.Client{Timeout: 30 * time.Second},
	}, nil
}

func (c *GrpcSchedulerClient) doRequest(ctx context.Context, method, path string, body interface{}) (*http.Response, error) {
	var reqBody io.Reader
	if body != nil {
		data, err := json.Marshal(body)
		if err != nil {
			return nil, fmt.Errorf("marshal request: %w", err)
		}
		reqBody = bytes.NewReader(data)
	}

	req, err := http.NewRequestWithContext(ctx, method, c.baseURL+path, reqBody)
	if err != nil {
		return nil, fmt.Errorf("create request: %w", err)
	}
	if body != nil {
		req.Header.Set("Content-Type", "application/json")
	}

	return c.httpClient.Do(req)
}

func parseSchedulerError(resp *http.Response, op string) error {
	bodyBytes, _ := io.ReadAll(resp.Body)
	msg := strings.TrimSpace(string(bodyBytes))
	if msg == "" {
		msg = fmt.Sprintf("status %d", resp.StatusCode)
	}
	if resp.StatusCode == http.StatusNotFound {
		return fmt.Errorf("job not found: %s", msg)
	}
	return fmt.Errorf("%s failed (status %d): %s", op, resp.StatusCode, msg)
}

// CreateJob はジョブを登録する。
func (c *GrpcSchedulerClient) CreateJob(ctx context.Context, req JobRequest) (Job, error) {
	body := grpcJobRequest{
		Name:        req.Name,
		Schedule:    toScheduleJSON(req.Schedule),
		Payload:     req.Payload,
		MaxRetries:  req.MaxRetries,
		TimeoutSecs: req.TimeoutSecs,
	}

	resp, err := c.doRequest(ctx, http.MethodPost, "/api/v1/jobs", body)
	if err != nil {
		return Job{}, fmt.Errorf("create_job: %w", err)
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK && resp.StatusCode != http.StatusCreated {
		return Job{}, parseSchedulerError(resp, "create_job")
	}

	var result grpcJobResponse
	if err := json.NewDecoder(resp.Body).Decode(&result); err != nil {
		return Job{}, fmt.Errorf("create_job: decode response: %w", err)
	}
	return fromJobResponse(result), nil
}

// CancelJob はジョブをキャンセルする。
func (c *GrpcSchedulerClient) CancelJob(ctx context.Context, jobID string) error {
	path := fmt.Sprintf("/api/v1/jobs/%s/cancel", url.PathEscape(jobID))
	resp, err := c.doRequest(ctx, http.MethodPost, path, struct{}{})
	if err != nil {
		return fmt.Errorf("cancel_job: %w", err)
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK && resp.StatusCode != http.StatusNoContent {
		return parseSchedulerError(resp, "cancel_job")
	}
	return nil
}

// PauseJob はジョブを一時停止する。
func (c *GrpcSchedulerClient) PauseJob(ctx context.Context, jobID string) error {
	path := fmt.Sprintf("/api/v1/jobs/%s/pause", url.PathEscape(jobID))
	resp, err := c.doRequest(ctx, http.MethodPost, path, struct{}{})
	if err != nil {
		return fmt.Errorf("pause_job: %w", err)
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK && resp.StatusCode != http.StatusNoContent {
		return parseSchedulerError(resp, "pause_job")
	}
	return nil
}

// ResumeJob はジョブを再開する。
func (c *GrpcSchedulerClient) ResumeJob(ctx context.Context, jobID string) error {
	path := fmt.Sprintf("/api/v1/jobs/%s/resume", url.PathEscape(jobID))
	resp, err := c.doRequest(ctx, http.MethodPost, path, struct{}{})
	if err != nil {
		return fmt.Errorf("resume_job: %w", err)
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK && resp.StatusCode != http.StatusNoContent {
		return parseSchedulerError(resp, "resume_job")
	}
	return nil
}

// GetJob はジョブ情報を取得する。
func (c *GrpcSchedulerClient) GetJob(ctx context.Context, jobID string) (Job, error) {
	path := fmt.Sprintf("/api/v1/jobs/%s", url.PathEscape(jobID))
	resp, err := c.doRequest(ctx, http.MethodGet, path, nil)
	if err != nil {
		return Job{}, fmt.Errorf("get_job: %w", err)
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		return Job{}, parseSchedulerError(resp, "get_job")
	}

	var result grpcJobResponse
	if err := json.NewDecoder(resp.Body).Decode(&result); err != nil {
		return Job{}, fmt.Errorf("get_job: decode response: %w", err)
	}
	return fromJobResponse(result), nil
}

// ListJobs はジョブ一覧を取得する。
func (c *GrpcSchedulerClient) ListJobs(ctx context.Context, filter JobFilter) ([]Job, error) {
	reqURL := c.baseURL + "/api/v1/jobs"
	q := url.Values{}
	if filter.Status != nil {
		q.Set("status", string(*filter.Status))
	}
	if filter.NamePrefix != nil && *filter.NamePrefix != "" {
		q.Set("name_prefix", *filter.NamePrefix)
	}
	if len(q) > 0 {
		reqURL += "?" + q.Encode()
	}

	req, err := http.NewRequestWithContext(ctx, http.MethodGet, reqURL, nil)
	if err != nil {
		return nil, fmt.Errorf("list_jobs: create request: %w", err)
	}

	resp, err := c.httpClient.Do(req)
	if err != nil {
		return nil, fmt.Errorf("list_jobs: %w", err)
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		return nil, parseSchedulerError(resp, "list_jobs")
	}

	var results []grpcJobResponse
	if err := json.NewDecoder(resp.Body).Decode(&results); err != nil {
		return nil, fmt.Errorf("list_jobs: decode response: %w", err)
	}

	jobs := make([]Job, len(results))
	for i, r := range results {
		jobs[i] = fromJobResponse(r)
	}
	return jobs, nil
}

// GetExecutions はジョブの実行履歴を取得する。
func (c *GrpcSchedulerClient) GetExecutions(ctx context.Context, jobID string) ([]JobExecution, error) {
	path := fmt.Sprintf("/api/v1/jobs/%s/executions", url.PathEscape(jobID))
	resp, err := c.doRequest(ctx, http.MethodGet, path, nil)
	if err != nil {
		return nil, fmt.Errorf("get_executions: %w", err)
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		return nil, parseSchedulerError(resp, "get_executions")
	}

	var results []grpcJobExecutionResponse
	if err := json.NewDecoder(resp.Body).Decode(&results); err != nil {
		return nil, fmt.Errorf("get_executions: decode response: %w", err)
	}

	executions := make([]JobExecution, len(results))
	for i, r := range results {
		executions[i] = JobExecution{
			ID:         r.ID,
			JobID:      r.JobID,
			StartedAt:  r.StartedAt,
			FinishedAt: r.FinishedAt,
			Result:     r.Result,
			Error:      r.Error,
		}
	}
	return executions, nil
}
