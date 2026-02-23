using K1s0.System.TestHelper.Assertions;
using Xunit;

namespace K1s0.System.TestHelper.Tests;

public class AssertionHelperTests
{
    [Fact]
    public void AssertJsonContains_PartialMatch_Passes()
    {
        AssertionHelper.AssertJsonContains(
            "{\"id\":\"1\",\"status\":\"ok\",\"extra\":\"ignored\"}",
            "{\"id\":\"1\",\"status\":\"ok\"}");
    }

    [Fact]
    public void AssertJsonContains_NestedMatch_Passes()
    {
        AssertionHelper.AssertJsonContains(
            "{\"user\":{\"id\":\"1\",\"name\":\"test\"},\"status\":\"ok\"}",
            "{\"user\":{\"id\":\"1\"}}");
    }

    [Fact]
    public void AssertJsonContains_Mismatch_Throws()
    {
        Assert.Throws<InvalidOperationException>(() =>
            AssertionHelper.AssertJsonContains("{\"id\":\"1\"}", "{\"id\":\"2\"}"));
    }

    [Fact]
    public void AssertEventEmitted_Found()
    {
        var json = global::System.Text.Json.JsonDocument.Parse(
            "[{\"type\":\"created\",\"id\":\"1\"},{\"type\":\"updated\",\"id\":\"2\"}]");
        var events = json.RootElement.EnumerateArray().ToList();
        AssertionHelper.AssertEventEmitted(events, "created");
        AssertionHelper.AssertEventEmitted(events, "updated");
    }

    [Fact]
    public void AssertEventEmitted_NotFound_Throws()
    {
        var json = global::System.Text.Json.JsonDocument.Parse("[{\"type\":\"created\"}]");
        var events = json.RootElement.EnumerateArray().ToList();
        Assert.Throws<InvalidOperationException>(() =>
            AssertionHelper.AssertEventEmitted(events, "deleted"));
    }
}
