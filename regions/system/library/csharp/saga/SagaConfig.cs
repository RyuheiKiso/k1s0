namespace K1s0.System.Saga;

public enum SagaProtocol
{
    Http,
    Grpc,
}

public sealed record SagaConfig(
    string RestBaseUrl,
    string? GrpcEndpoint,
    SagaProtocol Protocol = SagaProtocol.Http,
    int TimeoutSeconds = 30);
