using WireMock.RequestBuilders;
using WireMock.ResponseBuilders;
using WireMock.Server;
using Xunit;

namespace K1s0.System.SchemaRegistry.Tests.Integration;

[Trait("Category", "Integration")]
public class SchemaRegistryIntegrationTests : IDisposable
{
    private readonly WireMockServer _server;

    public SchemaRegistryIntegrationTests()
    {
        _server = WireMockServer.Start();
    }

    [Fact]
    public void SchemaRegistryConfig_WithAuth_SetsCredentials()
    {
        var config = new SchemaRegistryConfig
        {
            Url = _server.Url!,
            Username = "user",
            Password = "pass",
            CompatibilityMode = CompatibilityMode.Full,
        };

        Assert.Equal("user", config.Username);
        Assert.Equal("pass", config.Password);
        Assert.Equal(CompatibilityMode.Full, config.CompatibilityMode);
    }

    [Fact]
    public void SchemaRegistryConfig_SubjectName_FormatsCorrectly()
    {
        var subject = SchemaRegistryConfig.SubjectName("k1s0.system.auth.user-created.v1");
        Assert.Equal("k1s0.system.auth.user-created.v1-value", subject);
    }

    [Fact]
    public void SchemaRegistryConfig_SubjectNameKey_FormatsCorrectly()
    {
        var subject = SchemaRegistryConfig.SubjectName("k1s0.system.auth.user-created.v1", "key");
        Assert.Equal("k1s0.system.auth.user-created.v1-key", subject);
    }

    [Fact]
    public async Task MockedRegisterAndGet_RoundTrip()
    {
        // Arrange - mock register endpoint
        _server.Given(
            Request.Create()
                .WithPath("/subjects/test-subject/versions")
                .UsingPost())
            .RespondWith(
                Response.Create()
                    .WithStatusCode(200)
                    .WithHeader("Content-Type", "application/json")
                    .WithBody(global::System.Text.Json.JsonSerializer.Serialize(new { id = 42 })));

        // Arrange - mock get by ID endpoint
        _server.Given(
            Request.Create()
                .WithPath("/schemas/ids/42")
                .UsingGet())
            .RespondWith(
                Response.Create()
                    .WithStatusCode(200)
                    .WithHeader("Content-Type", "application/json")
                    .WithBody(global::System.Text.Json.JsonSerializer.Serialize(new
                    {
                        schema = """{"type":"record","name":"Test","fields":[]}""",
                        schemaType = "AVRO",
                    })));

        // Verify WireMock is responding
        using var httpClient = new HttpClient();
        var registerResponse = await httpClient.PostAsync(
            $"{_server.Url}/subjects/test-subject/versions",
            new StringContent("{}", global::System.Text.Encoding.UTF8, "application/json"));
        var registerBody = await registerResponse.Content.ReadAsStringAsync();
        Assert.Contains("42", registerBody);

        var getResponse = await httpClient.GetAsync($"{_server.Url}/schemas/ids/42");
        var getBody = await getResponse.Content.ReadAsStringAsync();
        Assert.Contains("Test", getBody);
    }

    [Fact]
    public async Task MockedCompatibilityCheck_ReturnsTrue()
    {
        // Arrange
        _server.Given(
            Request.Create()
                .WithPath("/compatibility/subjects/test-subject/versions/latest")
                .UsingPost())
            .RespondWith(
                Response.Create()
                    .WithStatusCode(200)
                    .WithHeader("Content-Type", "application/json")
                    .WithBody(global::System.Text.Json.JsonSerializer.Serialize(new { is_compatible = true })));

        using var httpClient = new HttpClient();
        var response = await httpClient.PostAsync(
            $"{_server.Url}/compatibility/subjects/test-subject/versions/latest",
            new StringContent("{}", global::System.Text.Encoding.UTF8, "application/json"));
        var body = await response.Content.ReadAsStringAsync();

        Assert.Contains("true", body);
    }

    [Fact]
    public void SchemaRegistryException_HasCodeAndMessage()
    {
        var ex = new SchemaRegistryException("TEST_CODE", "test message");
        Assert.Equal("TEST_CODE", ex.Code);
        Assert.Equal("test message", ex.Message);
    }

    [Fact]
    public void SchemaRegistryException_WithInnerException()
    {
        var inner = new InvalidOperationException("inner");
        var ex = new SchemaRegistryException("TEST_CODE", "test message", inner);
        Assert.Equal("TEST_CODE", ex.Code);
        Assert.Same(inner, ex.InnerException);
    }

    public void Dispose()
    {
        _server.Stop();
        _server.Dispose();
    }
}
