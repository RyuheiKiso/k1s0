// 本ファイルは Temporal SDK の Client を temporalClient narrow interface に
// 適合させる shim。DescribeWorkflowExecution の戻り値型を minimal subset に
// 詰め替える役割を担う。

package temporal

import (
	"context"

	tclient "go.temporal.io/sdk/client"
	"go.temporal.io/sdk/converter"
)

// sdkShim は実 Temporal SDK の Client を保持し、temporalClient interface に変換する。
type sdkShim struct {
	c tclient.Client
}

// newSDKShim は SDK Client を shim でラップする。
func newSDKShim(c tclient.Client) temporalClient {
	return &sdkShim{c: c}
}

func (s *sdkShim) ExecuteWorkflow(ctx context.Context, options tclient.StartWorkflowOptions, workflow interface{}, args ...interface{}) (tclient.WorkflowRun, error) {
	return s.c.ExecuteWorkflow(ctx, options, workflow, args...)
}

func (s *sdkShim) SignalWorkflow(ctx context.Context, workflowID, runID, signalName string, arg interface{}) error {
	return s.c.SignalWorkflow(ctx, workflowID, runID, signalName, arg)
}

func (s *sdkShim) QueryWorkflow(ctx context.Context, workflowID, runID, queryType string, args ...interface{}) (converter.EncodedValue, error) {
	return s.c.QueryWorkflow(ctx, workflowID, runID, queryType, args...)
}

func (s *sdkShim) CancelWorkflow(ctx context.Context, workflowID, runID string) error {
	return s.c.CancelWorkflow(ctx, workflowID, runID)
}

func (s *sdkShim) TerminateWorkflow(ctx context.Context, workflowID, runID, reason string, details ...interface{}) error {
	return s.c.TerminateWorkflow(ctx, workflowID, runID, reason, details...)
}

func (s *sdkShim) DescribeWorkflowExecution(ctx context.Context, workflowID, runID string) (*describeResponse, error) {
	resp, err := s.c.DescribeWorkflowExecution(ctx, workflowID, runID)
	if err != nil {
		return nil, err
	}
	out := &describeResponse{}
	if info := resp.GetWorkflowExecutionInfo(); info != nil {
		out.StatusCode = int32(info.GetStatus())
		if exec := info.GetExecution(); exec != nil {
			out.RunID = exec.GetRunId()
		}
	}
	return out, nil
}
