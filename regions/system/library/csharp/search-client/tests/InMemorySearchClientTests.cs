using System.Text.Json;
using K1s0.System.SearchClient;

namespace K1s0.System.SearchClient.Tests;

public class InMemorySearchClientTests
{
    [Fact]
    public async Task CreateIndex_Succeeds()
    {
        var client = new InMemorySearchClient();
        await client.CreateIndexAsync("products", new IndexMapping().WithField("name", "text"));
        Assert.Equal(0, client.DocumentCount("products"));
    }

    [Fact]
    public async Task IndexDocument_ReturnsResult()
    {
        var client = new InMemorySearchClient();
        await client.CreateIndexAsync("products", new IndexMapping());

        var doc = new IndexDocument("p-1", new Dictionary<string, object> { ["name"] = "Rust Programming" });
        var result = await client.IndexDocumentAsync("products", doc);

        Assert.Equal("p-1", result.Id);
        Assert.Equal(1, result.Version);
    }

    [Fact]
    public async Task IndexDocument_ThrowsOnMissingIndex()
    {
        var client = new InMemorySearchClient();
        var doc = new IndexDocument("1", new Dictionary<string, object>());
        await Assert.ThrowsAsync<SearchException>(
            () => client.IndexDocumentAsync("nonexistent", doc));
    }

    [Fact]
    public async Task BulkIndex_ReturnsSuccessCount()
    {
        var client = new InMemorySearchClient();
        await client.CreateIndexAsync("items", new IndexMapping());

        var docs = new[]
        {
            new IndexDocument("i-1", new Dictionary<string, object> { ["name"] = "Item 1" }),
            new IndexDocument("i-2", new Dictionary<string, object> { ["name"] = "Item 2" }),
        };
        var result = await client.BulkIndexAsync("items", docs);

        Assert.Equal(2, result.SuccessCount);
        Assert.Equal(0, result.FailedCount);
        Assert.Empty(result.Failures);
    }

    [Fact]
    public async Task Search_FindsMatchingDocuments()
    {
        var client = new InMemorySearchClient();
        await client.CreateIndexAsync("products", new IndexMapping());
        await client.IndexDocumentAsync("products",
            new IndexDocument("p-1", new Dictionary<string, object> { ["name"] = "Rust Programming" }));
        await client.IndexDocumentAsync("products",
            new IndexDocument("p-2", new Dictionary<string, object> { ["name"] = "Go Language" }));

        var query = new SearchQuery("Rust", Facets: new[] { "name" });
        var result = await client.SearchAsync<JsonElement>("products", query);

        Assert.Equal(1UL, result.Total);
        Assert.Single(result.Hits);
        Assert.True(result.Facets.ContainsKey("name"));
    }

    [Fact]
    public async Task Search_ThrowsOnMissingIndex()
    {
        var client = new InMemorySearchClient();
        await Assert.ThrowsAsync<SearchException>(
            () => client.SearchAsync<object>("nonexistent", new SearchQuery("test")));
    }

    [Fact]
    public async Task DeleteDocument_RemovesDocument()
    {
        var client = new InMemorySearchClient();
        await client.CreateIndexAsync("products", new IndexMapping());
        await client.IndexDocumentAsync("products",
            new IndexDocument("p-1", new Dictionary<string, object> { ["name"] = "Test" }));

        await client.DeleteDocumentAsync("products", "p-1");
        Assert.Equal(0, client.DocumentCount("products"));
    }

    [Fact]
    public async Task Search_EmptyQuery_ReturnsAll()
    {
        var client = new InMemorySearchClient();
        await client.CreateIndexAsync("items", new IndexMapping());
        await client.IndexDocumentAsync("items",
            new IndexDocument("i-1", new Dictionary<string, object> { ["name"] = "Item" }));

        var result = await client.SearchAsync<JsonElement>("items", new SearchQuery(""));
        Assert.Equal(1UL, result.Total);
    }

    [Fact]
    public void SearchException_HasCorrectCode()
    {
        var ex = new SearchException("test", SearchErrorCode.IndexNotFound);
        Assert.Equal(SearchErrorCode.IndexNotFound, ex.Code);
        Assert.Equal("test", ex.Message);
    }

    [Fact]
    public void IndexMapping_WithField_AddsField()
    {
        var mapping = new IndexMapping()
            .WithField("name", "text")
            .WithField("price", "integer");
        Assert.Equal(2, mapping.Fields.Count);
        Assert.Equal("text", mapping.Fields["name"].FieldType);
    }

    [Fact]
    public async Task DisposeAsync_ClearsState()
    {
        var client = new InMemorySearchClient();
        await client.CreateIndexAsync("test", new IndexMapping());
        await client.DisposeAsync();
        Assert.Equal(0, client.DocumentCount("test"));
    }
}
