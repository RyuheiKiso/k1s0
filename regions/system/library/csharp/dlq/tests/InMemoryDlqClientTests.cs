namespace K1s0.System.Dlq.Tests;

public class InMemoryDlqClientTests
{
    [Fact]
    public async Task Send_CreatesMessage()
    {
        var client = new InMemoryDlqClient();
        var msg = await client.SendAsync(new DlqSendRequest("orders.v1", "processing failed", "{\"orderId\":1}"));

        Assert.Equal("orders.v1", msg.OriginalTopic);
        Assert.Equal(DlqStatus.Pending, msg.Status);
        Assert.Equal(0, msg.RetryCount);
    }

    [Fact]
    public async Task ListMessages_FiltersByTopic()
    {
        var client = new InMemoryDlqClient();
        await client.SendAsync(new DlqSendRequest("orders.v1", "err", "{}"));
        await client.SendAsync(new DlqSendRequest("payments.v1", "err", "{}"));
        await client.SendAsync(new DlqSendRequest("orders.v1", "err2", "{}"));

        var result = await client.ListMessagesAsync("orders.v1");

        Assert.Equal(2, result.Total);
        Assert.Equal(2, result.Messages.Count);
    }

    [Fact]
    public async Task RetryMessage_UpdatesStatus()
    {
        var client = new InMemoryDlqClient();
        var msg = await client.SendAsync(new DlqSendRequest("orders.v1", "err", "{}"));

        var resp = await client.RetryMessageAsync(msg.Id);

        Assert.Equal(DlqStatus.Retrying, resp.Status);
        var updated = await client.GetMessageAsync(msg.Id);
        Assert.Equal(1, updated.RetryCount);
        Assert.Equal(DlqStatus.Retrying, updated.Status);
    }

    [Fact]
    public async Task DeleteMessage_RemovesMessage()
    {
        var client = new InMemoryDlqClient();
        var msg = await client.SendAsync(new DlqSendRequest("orders.v1", "err", "{}"));

        await client.DeleteMessageAsync(msg.Id);

        var ex = Assert.Throws<DlqException>(() => client.GetMessageAsync(msg.Id).GetAwaiter().GetResult());
        Assert.Equal(DlqErrorCodes.NotFound, ex.Code);
    }

    [Fact]
    public async Task GetMessage_NotFound_Throws()
    {
        var client = new InMemoryDlqClient();
        Assert.Throws<DlqException>(() => client.GetMessageAsync(Guid.NewGuid()).GetAwaiter().GetResult());
    }

    [Fact]
    public async Task RetryAll_RetriesAllPendingForTopic()
    {
        var client = new InMemoryDlqClient();
        await client.SendAsync(new DlqSendRequest("orders.v1", "err1", "{}"));
        await client.SendAsync(new DlqSendRequest("orders.v1", "err2", "{}"));
        await client.SendAsync(new DlqSendRequest("payments.v1", "err3", "{}"));

        await client.RetryAllAsync("orders.v1");

        var orders = await client.ListMessagesAsync("orders.v1");
        Assert.All(orders.Messages, m => Assert.Equal(DlqStatus.Retrying, m.Status));

        var payments = await client.ListMessagesAsync("payments.v1");
        Assert.All(payments.Messages, m => Assert.Equal(DlqStatus.Pending, m.Status));
    }
}
