package schedulerclient_test

import (
	"context"
	"encoding/json"
	"testing"

	schedulerclient "github.com/k1s0-platform/system-library-go-scheduler-client"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestCreateJob_ReturnsJob(t *testing.T) {
	c := schedulerclient.NewInMemoryClient()
	job, err := c.CreateJob(context.Background(), schedulerclient.JobRequest{
		Name:        "daily-report",
		Schedule:    schedulerclient.Schedule{Type: "cron", Cron: "0 2 * * *"},
		Payload:     json.RawMessage(`{"report_type":"daily"}`),
		MaxRetries:  3,
		TimeoutSecs: 300,
	})
	require.NoError(t, err)
	assert.Equal(t, "job-001", job.ID)
	assert.Equal(t, "daily-report", job.Name)
	assert.Equal(t, schedulerclient.JobStatusPending, job.Status)
}

func TestCancelJob_UpdatesStatus(t *testing.T) {
	c := schedulerclient.NewInMemoryClient()
	ctx := context.Background()
	job, _ := c.CreateJob(ctx, schedulerclient.JobRequest{
		Name:     "test-job",
		Schedule: schedulerclient.Schedule{Type: "cron", Cron: "* * * * *"},
		Payload:  json.RawMessage(`{}`),
	})
	err := c.CancelJob(ctx, job.ID)
	require.NoError(t, err)

	got, _ := c.GetJob(ctx, job.ID)
	assert.Equal(t, schedulerclient.JobStatusCancelled, got.Status)
}

func TestPauseAndResumeJob(t *testing.T) {
	c := schedulerclient.NewInMemoryClient()
	ctx := context.Background()
	job, _ := c.CreateJob(ctx, schedulerclient.JobRequest{
		Name:     "pause-test",
		Schedule: schedulerclient.Schedule{Type: "cron", Cron: "0 * * * *"},
		Payload:  json.RawMessage(`{}`),
	})

	err := c.PauseJob(ctx, job.ID)
	require.NoError(t, err)
	got, _ := c.GetJob(ctx, job.ID)
	assert.Equal(t, schedulerclient.JobStatusPaused, got.Status)

	err = c.ResumeJob(ctx, job.ID)
	require.NoError(t, err)
	got, _ = c.GetJob(ctx, job.ID)
	assert.Equal(t, schedulerclient.JobStatusPending, got.Status)
}

func TestGetJob_NotFound(t *testing.T) {
	c := schedulerclient.NewInMemoryClient()
	_, err := c.GetJob(context.Background(), "nonexistent")
	assert.Error(t, err)
}

func TestListJobs_Empty(t *testing.T) {
	c := schedulerclient.NewInMemoryClient()
	jobs, err := c.ListJobs(context.Background(), schedulerclient.JobFilter{})
	require.NoError(t, err)
	assert.Empty(t, jobs)
}

func TestListJobs_WithStatusFilter(t *testing.T) {
	c := schedulerclient.NewInMemoryClient()
	ctx := context.Background()

	_, _ = c.CreateJob(ctx, schedulerclient.JobRequest{
		Name: "job-a", Schedule: schedulerclient.Schedule{Type: "cron", Cron: "* * * * *"},
		Payload: json.RawMessage(`{}`),
	})
	job2, _ := c.CreateJob(ctx, schedulerclient.JobRequest{
		Name: "job-b", Schedule: schedulerclient.Schedule{Type: "cron", Cron: "* * * * *"},
		Payload: json.RawMessage(`{}`),
	})
	_ = c.PauseJob(ctx, job2.ID)

	paused := schedulerclient.JobStatusPaused
	jobs, err := c.ListJobs(ctx, schedulerclient.JobFilter{Status: &paused})
	require.NoError(t, err)
	assert.Len(t, jobs, 1)
	assert.Equal(t, schedulerclient.JobStatusPaused, jobs[0].Status)
}

func TestGetExecutions_ReturnsEmpty(t *testing.T) {
	c := schedulerclient.NewInMemoryClient()
	execs, err := c.GetExecutions(context.Background(), "job-001")
	require.NoError(t, err)
	assert.Empty(t, execs)
}

func TestCancelJob_NotFound(t *testing.T) {
	c := schedulerclient.NewInMemoryClient()
	err := c.CancelJob(context.Background(), "nonexistent")
	assert.Error(t, err)
}

func TestJobs_ReturnsCopy(t *testing.T) {
	c := schedulerclient.NewInMemoryClient()
	ctx := context.Background()
	_, _ = c.CreateJob(ctx, schedulerclient.JobRequest{
		Name: "test", Schedule: schedulerclient.Schedule{Type: "cron", Cron: "* * * * *"},
		Payload: json.RawMessage(`{}`),
	})
	jobs := c.Jobs()
	assert.Len(t, jobs, 1)
}
