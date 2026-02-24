using K1s0.System.NotificationClient;

namespace K1s0.System.NotificationClient.Tests;

public class InMemoryNotificationClientTests
{
    [Fact]
    public async Task Send_ReturnsResponse()
    {
        var client = new InMemoryNotificationClient();
        var req = new NotificationRequest("1", NotificationChannel.Email, "user@example.com", "Hello", "Body text");

        var resp = await client.SendAsync(req);

        Assert.Equal("1", resp.Id);
        Assert.Equal("sent", resp.Status);
        Assert.NotNull(resp.MessageId);
    }

    [Fact]
    public async Task Send_RecordsInSentList()
    {
        var client = new InMemoryNotificationClient();
        var req = new NotificationRequest("1", NotificationChannel.Sms, "+1234567890", null, "OTP: 1234");

        await client.SendAsync(req);

        Assert.Single(client.Sent);
        Assert.Equal(NotificationChannel.Sms, client.Sent[0].Channel);
    }

    [Fact]
    public async Task Send_MultipleTimes_TracksAll()
    {
        var client = new InMemoryNotificationClient();
        await client.SendAsync(new NotificationRequest("1", NotificationChannel.Email, "a@b.com", "S1", "B1"));
        await client.SendAsync(new NotificationRequest("2", NotificationChannel.Push, "device-1", null, "Push body"));

        Assert.Equal(2, client.Sent.Count);
    }

    [Fact]
    public void Sent_InitiallyEmpty()
    {
        var client = new InMemoryNotificationClient();
        Assert.Empty(client.Sent);
    }

    [Fact]
    public async Task Send_AllChannels_Succeed()
    {
        var client = new InMemoryNotificationClient();
        foreach (var channel in Enum.GetValues<NotificationChannel>())
        {
            var resp = await client.SendAsync(
                new NotificationRequest(channel.ToString(), channel, "recipient", null, "body"));
            Assert.Equal("sent", resp.Status);
        }
        Assert.Equal(4, client.Sent.Count);
    }

    [Fact]
    public async Task SendBatch_ReturnsMultipleResponses()
    {
        var client = new InMemoryNotificationClient();
        var requests = new List<NotificationRequest>
        {
            new("1", NotificationChannel.Email, "a@b.com", "Subject1", "Body1"),
            new("2", NotificationChannel.Sms, "+1234567890", null, "OTP"),
            new("3", NotificationChannel.Push, "device-1", null, "Push body"),
        };

        var responses = await client.SendBatchAsync(requests);

        Assert.Equal(3, responses.Count);
        Assert.All(responses, r => Assert.Equal("sent", r.Status));
        Assert.Equal(3, client.Sent.Count);
    }

    [Fact]
    public async Task SendBatch_EmptyList_ReturnsEmpty()
    {
        var client = new InMemoryNotificationClient();
        var responses = await client.SendBatchAsync(new List<NotificationRequest>());
        Assert.Empty(responses);
        Assert.Empty(client.Sent);
    }
}
