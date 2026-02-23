using K1s0.GraphQlClient;

namespace K1s0.GraphQlClient.Tests;

public class GraphQlClientTests
{
    [Fact]
    public void GraphQlQuery_CreatesWithRequiredFields()
    {
        var query = new GraphQlQuery("{ users { id } }");
        Assert.Equal("{ users { id } }", query.Query);
        Assert.Null(query.Variables);
        Assert.Null(query.OperationName);
    }

    [Fact]
    public void GraphQlQuery_CreatesWithAllFields()
    {
        var vars = new Dictionary<string, object> { ["id"] = "1" };
        var query = new GraphQlQuery("query GetUser($id: ID!)", vars, "GetUser");
        Assert.Equal("GetUser", query.OperationName);
        Assert.NotNull(query.Variables);
    }

    [Fact]
    public void GraphQlResponse_HasErrors_FalseWhenNoErrors()
    {
        var resp = new GraphQlResponse<string>("ok");
        Assert.False(resp.HasErrors);
    }

    [Fact]
    public void GraphQlResponse_HasErrors_TrueWithErrors()
    {
        var resp = new GraphQlResponse<string>(null, new List<GraphQlError> { new("fail") });
        Assert.True(resp.HasErrors);
    }

    [Fact]
    public void GraphQlResponse_HasErrors_FalseWithEmptyList()
    {
        var resp = new GraphQlResponse<string>(null, new List<GraphQlError>());
        Assert.False(resp.HasErrors);
    }

    [Fact]
    public void ErrorLocation_StoresLineAndColumn()
    {
        var loc = new ErrorLocation(3, 5);
        Assert.Equal(3, loc.Line);
        Assert.Equal(5, loc.Column);
    }

    [Fact]
    public void GraphQlError_CreatesWithMessage()
    {
        var err = new GraphQlError("Not found");
        Assert.Equal("Not found", err.Message);
        Assert.Null(err.Locations);
        Assert.Null(err.Path);
    }

    [Fact]
    public async Task ExecuteAsync_ReturnsConfiguredResponse()
    {
        var client = new InMemoryGraphQlClient();
        client.SetResponse("GetUser", "Alice");

        var result = await client.ExecuteAsync<string>(
            new GraphQlQuery("query GetUser { user }", OperationName: "GetUser"));

        Assert.False(result.HasErrors);
        Assert.Equal("Alice", result.Data);
    }

    [Fact]
    public async Task ExecuteAsync_ReturnsErrorForUnconfigured()
    {
        var client = new InMemoryGraphQlClient();

        var result = await client.ExecuteAsync<string>(
            new GraphQlQuery("{ unknown }", OperationName: "Unknown"));

        Assert.True(result.HasErrors);
        Assert.Null(result.Data);
    }

    [Fact]
    public async Task ExecuteMutationAsync_ReturnsConfiguredResponse()
    {
        var client = new InMemoryGraphQlClient();
        client.SetResponse("CreateUser", "Bob");

        var result = await client.ExecuteMutationAsync<string>(
            new GraphQlQuery("mutation CreateUser { createUser }", OperationName: "CreateUser"));

        Assert.False(result.HasErrors);
        Assert.Equal("Bob", result.Data);
    }

    [Fact]
    public async Task ExecuteAsync_FallsBackToQueryText()
    {
        var client = new InMemoryGraphQlClient();
        client.SetResponse("{ me { id } }", "me-data");

        var result = await client.ExecuteAsync<string>(
            new GraphQlQuery("{ me { id } }"));

        Assert.False(result.HasErrors);
        Assert.Equal("me-data", result.Data);
    }

    [Fact]
    public async Task ExecuteAsync_TypeMismatchReturnsError()
    {
        var client = new InMemoryGraphQlClient();
        client.SetResponse("Op", 42);

        var result = await client.ExecuteAsync<string>(
            new GraphQlQuery("query", OperationName: "Op"));

        Assert.True(result.HasErrors);
    }
}
