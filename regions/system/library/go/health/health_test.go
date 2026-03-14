package health_test

import (
	"context"
	"errors"
	"testing"

	"github.com/k1s0-platform/system-library-go-health"
	"github.com/stretchr/testify/assert"
)

type alwaysHealthy struct{}

func (a *alwaysHealthy) Name() string                        { return "healthy-check" }
func (a *alwaysHealthy) Check(_ context.Context) error       { return nil }

type alwaysUnhealthy struct{}

func (a *alwaysUnhealthy) Name() string                      { return "unhealthy-check" }
func (a *alwaysUnhealthy) Check(_ context.Context) error     { return errors.New("down") }

// Checker_AllHealthyが全チェックが正常な場合にStatusHealthyを返すことを検証する。
func TestChecker_AllHealthy(t *testing.T) {
	c := health.NewChecker()
	c.Add(&alwaysHealthy{})
	resp := c.RunAll(context.Background())
	assert.Equal(t, health.StatusHealthy, resp.Status)
	assert.Equal(t, health.StatusHealthy, resp.Checks["healthy-check"].Status)
}

// Checker_OneUnhealthyが1つでも異常なチェックがある場合に全体をStatusUnhealthyとすることを検証する。
func TestChecker_OneUnhealthy(t *testing.T) {
	c := health.NewChecker()
	c.Add(&alwaysHealthy{})
	c.Add(&alwaysUnhealthy{})
	resp := c.RunAll(context.Background())
	assert.Equal(t, health.StatusUnhealthy, resp.Status)
	assert.Equal(t, health.StatusHealthy, resp.Checks["healthy-check"].Status)
	assert.Equal(t, health.StatusUnhealthy, resp.Checks["unhealthy-check"].Status)
}

// Checker_NoChecksがチェックが登録されていない場合にStatusHealthyと空のチェックマップを返すことを検証する。
func TestChecker_NoChecks(t *testing.T) {
	c := health.NewChecker()
	resp := c.RunAll(context.Background())
	assert.Equal(t, health.StatusHealthy, resp.Status)
	assert.Empty(t, resp.Checks)
}

// Checker_TimestampSetがRunAllの結果にゼロでないタイムスタンプが含まれることを検証する。
func TestChecker_TimestampSet(t *testing.T) {
	c := health.NewChecker()
	resp := c.RunAll(context.Background())
	assert.False(t, resp.Timestamp.IsZero())
}
