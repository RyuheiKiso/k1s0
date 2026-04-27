// 本ファイルは k1s0 .NET SDK の高水準 Client 型。
//
// 利用例:
//   using var client = new K1s0Client(new K1s0Config {
//       Target = "https://tier1.k1s0.example.com",
//       TenantId = "tenant-A",
//       Subject = "svc-foo",
//   });
//   var (data, etag, found) = await client.State.GetAsync("valkey-default", "user/123");

using Grpc.Net.Client;
using K1s0.Sdk.Generated.K1s0.Tier1.Audit.V1;
using K1s0.Sdk.Generated.K1s0.Tier1.Binding.V1;
using K1s0.Sdk.Generated.K1s0.Tier1.Common.V1;
using K1s0.Sdk.Generated.K1s0.Tier1.Decision.V1;
using K1s0.Sdk.Generated.K1s0.Tier1.Feature.V1;
using K1s0.Sdk.Generated.K1s0.Tier1.Log.V1;
using K1s0.Sdk.Generated.K1s0.Tier1.Pii.V1;
using K1s0.Sdk.Generated.K1s0.Tier1.Pubsub.V1;
using K1s0.Sdk.Generated.K1s0.Tier1.Secrets.V1;
using K1s0.Sdk.Generated.K1s0.Tier1.Serviceinvoke.V1;
using K1s0.Sdk.Generated.K1s0.Tier1.State.V1;
using K1s0.Sdk.Generated.K1s0.Tier1.Telemetry.V1;
using K1s0.Sdk.Generated.K1s0.Tier1.Workflow.V1;

namespace K1s0.Sdk;

// K1s0Config は Client 初期化時に渡す設定。
public sealed class K1s0Config
{
    // gRPC 接続先（例: "https://tier1.k1s0.example.com"）。
    public required string Target { get; init; }

    // テナント ID（全 RPC の TenantContext.TenantId に自動付与）。
    public required string TenantId { get; init; }

    // 主体識別子（subject）。
    public required string Subject { get; init; }
}

// K1s0Client は 12 service へのアクセス起点。Dispose で gRPC channel を解放する。
public sealed class K1s0Client : IDisposable
{
    // 内部に GrpcChannel を保持する。
    private readonly GrpcChannel _channel;

    // 親 Config への参照。
    public K1s0Config Config { get; }

    // 動詞統一 facade（12 service すべて）。
    public StateFacade State { get; }
    public PubSubFacade PubSub { get; }
    public SecretsFacade Secrets { get; }
    public LogFacade Log { get; }
    public WorkflowFacade Workflow { get; }
    public DecisionFacade Decision { get; }
    public AuditFacade Audit { get; }
    public PiiFacade Pii { get; }
    public FeatureFacade Feature { get; }
    public BindingFacade Binding { get; }
    public InvokeFacade Invoke { get; }
    public TelemetryFacade Telemetry { get; }
    public DecisionAdminFacade DecisionAdmin { get; }
    public FeatureAdminFacade FeatureAdmin { get; }

    // 14 service の生成 stub client への直接アクセス。
    public RawClients Raw { get; }

    public K1s0Client(K1s0Config config)
    {
        // Config を保持する。
        Config = config;
        // gRPC channel を確立する。
        _channel = GrpcChannel.ForAddress(config.Target);
        // 12 service の生成 client を構築する。
        Raw = new RawClients(_channel);
        // 動詞統一 facade を初期化する。
        State = new StateFacade(this);
        PubSub = new PubSubFacade(this);
        Secrets = new SecretsFacade(this);
        Log = new LogFacade(this);
        Workflow = new WorkflowFacade(this);
        Decision = new DecisionFacade(this);
        Audit = new AuditFacade(this);
        Pii = new PiiFacade(this);
        Feature = new FeatureFacade(this);
        Binding = new BindingFacade(this);
        Invoke = new InvokeFacade(this);
        Telemetry = new TelemetryFacade(this);
        DecisionAdmin = new DecisionAdminFacade(this);
        FeatureAdmin = new FeatureAdminFacade(this);
    }

    // 内部用: TenantContext proto を生成する。
    internal TenantContext TenantContext() => new()
    {
        TenantId = Config.TenantId,
        Subject = Config.Subject,
        // CorrelationId は OTel interceptor 後段付与。
        CorrelationId = string.Empty,
    };

    // 内部用: 各 facade 用の生成 client を取得する。
    internal StateService.StateServiceClient RawState() => Raw.State;
    internal PubSubService.PubSubServiceClient RawPubSub() => Raw.PubSub;
    internal SecretsService.SecretsServiceClient RawSecrets() => Raw.Secrets;

    public void Dispose()
    {
        _channel.Dispose();
    }
}

// RawClients は 12 service すべての生成 stub クライアントを保持する。
public sealed class RawClients
{
    public AuditService.AuditServiceClient Audit { get; }
    public BindingService.BindingServiceClient Binding { get; }
    public DecisionService.DecisionServiceClient Decision { get; }
    public DecisionAdminService.DecisionAdminServiceClient DecisionAdmin { get; }
    public FeatureService.FeatureServiceClient Feature { get; }
    public FeatureAdminService.FeatureAdminServiceClient FeatureAdmin { get; }
    public LogService.LogServiceClient Log { get; }
    public PiiService.PiiServiceClient Pii { get; }
    public PubSubService.PubSubServiceClient PubSub { get; }
    public SecretsService.SecretsServiceClient Secrets { get; }
    public InvokeService.InvokeServiceClient ServiceInvoke { get; }
    public StateService.StateServiceClient State { get; }
    public TelemetryService.TelemetryServiceClient Telemetry { get; }
    public WorkflowService.WorkflowServiceClient Workflow { get; }

    internal RawClients(GrpcChannel channel)
    {
        Audit = new AuditService.AuditServiceClient(channel);
        Binding = new BindingService.BindingServiceClient(channel);
        Decision = new DecisionService.DecisionServiceClient(channel);
        DecisionAdmin = new DecisionAdminService.DecisionAdminServiceClient(channel);
        Feature = new FeatureService.FeatureServiceClient(channel);
        FeatureAdmin = new FeatureAdminService.FeatureAdminServiceClient(channel);
        Log = new LogService.LogServiceClient(channel);
        Pii = new PiiService.PiiServiceClient(channel);
        PubSub = new PubSubService.PubSubServiceClient(channel);
        Secrets = new SecretsService.SecretsServiceClient(channel);
        ServiceInvoke = new InvokeService.InvokeServiceClient(channel);
        State = new StateService.StateServiceClient(channel);
        Telemetry = new TelemetryService.TelemetryServiceClient(channel);
        Workflow = new WorkflowService.WorkflowServiceClient(channel);
    }
}
