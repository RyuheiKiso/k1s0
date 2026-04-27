// 本ファイルは k1s0 .NET SDK の Workflow 動詞統一 facade。
using Google.Protobuf;
using K1s0.Sdk.Generated.K1s0.Tier1.Workflow.V1;

namespace K1s0.Sdk;

public sealed class WorkflowFacade
{
    private readonly K1s0Client _client;
    internal WorkflowFacade(K1s0Client client) { _client = client; }

    /// StartAsync: ワークフロー開始。返り値は (workflowId, runId)。
    public async Task<(string WorkflowId, string RunId)> StartAsync(string workflowType, string workflowId, byte[] input, bool idempotent = false, CancellationToken ct = default)
    {
        var resp = await _client.Raw.Workflow.StartAsync(new StartRequest
        {
            WorkflowType = workflowType,
            WorkflowId = workflowId,
            Input = ByteString.CopyFrom(input),
            Idempotent = idempotent,
            Context = _client.TenantContext(),
        }, cancellationToken: ct);
        return (resp.WorkflowId, resp.RunId);
    }

    public async Task SignalAsync(string workflowId, string signalName, byte[] payload, CancellationToken ct = default)
        => await _client.Raw.Workflow.SignalAsync(new SignalRequest { WorkflowId = workflowId, SignalName = signalName, Payload = ByteString.CopyFrom(payload), Context = _client.TenantContext() }, cancellationToken: ct);

    public async Task<byte[]> QueryAsync(string workflowId, string queryName, byte[] payload, CancellationToken ct = default)
    {
        var resp = await _client.Raw.Workflow.QueryAsync(new QueryRequest { WorkflowId = workflowId, QueryName = queryName, Payload = ByteString.CopyFrom(payload), Context = _client.TenantContext() }, cancellationToken: ct);
        return resp.Result.ToByteArray();
    }

    public async Task CancelAsync(string workflowId, string reason, CancellationToken ct = default)
        => await _client.Raw.Workflow.CancelAsync(new CancelRequest { WorkflowId = workflowId, Reason = reason, Context = _client.TenantContext() }, cancellationToken: ct);

    public async Task TerminateAsync(string workflowId, string reason, CancellationToken ct = default)
        => await _client.Raw.Workflow.TerminateAsync(new TerminateRequest { WorkflowId = workflowId, Reason = reason, Context = _client.TenantContext() }, cancellationToken: ct);

    public async Task<GetStatusResponse> GetStatusAsync(string workflowId, CancellationToken ct = default)
        => await _client.Raw.Workflow.GetStatusAsync(new GetStatusRequest { WorkflowId = workflowId, Context = _client.TenantContext() }, cancellationToken: ct);
}
