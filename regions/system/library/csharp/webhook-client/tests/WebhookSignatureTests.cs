using System.Text;
using K1s0.System.WebhookClient;

namespace K1s0.System.WebhookClient.Tests;

public class WebhookSignatureTests
{
    [Fact]
    public void Generate_ReturnsDeterministicSignature()
    {
        var body = Encoding.UTF8.GetBytes("test body");
        var sig1 = WebhookSignature.Generate("secret", body);
        var sig2 = WebhookSignature.Generate("secret", body);

        Assert.Equal(sig1, sig2);
    }

    [Fact]
    public void Verify_CorrectSignature_ReturnsTrue()
    {
        var body = Encoding.UTF8.GetBytes("payload");
        var sig = WebhookSignature.Generate("mysecret", body);

        Assert.True(WebhookSignature.Verify("mysecret", body, sig));
    }

    [Fact]
    public void Verify_WrongSignature_ReturnsFalse()
    {
        var body = Encoding.UTF8.GetBytes("payload");
        Assert.False(WebhookSignature.Verify("mysecret", body, "wrongsig"));
    }

    [Fact]
    public void Verify_WrongSecret_ReturnsFalse()
    {
        var body = Encoding.UTF8.GetBytes("payload");
        var sig = WebhookSignature.Generate("secret1", body);

        Assert.False(WebhookSignature.Verify("secret2", body, sig));
    }

    [Fact]
    public void WebhookPayload_RecordEquality()
    {
        var a = new WebhookPayload("test.event", "2024-01-01T00:00:00Z", new Dictionary<string, object> { ["key"] = "val" });
        var b = new WebhookPayload("test.event", "2024-01-01T00:00:00Z", a.Data);

        Assert.Equal(a, b);
    }
}
