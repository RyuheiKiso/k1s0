using Microsoft.Extensions.Hosting;

namespace K1s0.System.Outbox;

public sealed class OutboxProcessor : BackgroundService
{
    private readonly OutboxConfig _config;
    private readonly IOutboxStore _store;
    private readonly IEventProducer _producer;

    public OutboxProcessor(OutboxConfig config, IOutboxStore store, IEventProducer producer)
    {
        _config = config ?? throw new ArgumentNullException(nameof(config));
        _store = store ?? throw new ArgumentNullException(nameof(store));
        _producer = producer ?? throw new ArgumentNullException(nameof(producer));
    }

    protected override async Task ExecuteAsync(CancellationToken stoppingToken)
    {
        while (!stoppingToken.IsCancellationRequested)
        {
            try
            {
                var messages = await _store.FetchPendingAsync(ct: stoppingToken);

                foreach (var message in messages)
                {
                    if (stoppingToken.IsCancellationRequested)
                    {
                        break;
                    }

                    await ProcessMessageAsync(message, stoppingToken);
                }
            }
            catch (OutboxException)
            {
                // Store fetch failure; wait and retry on next cycle
            }

            try
            {
                await Task.Delay(_config.PollingInterval, stoppingToken);
            }
            catch (OperationCanceledException)
            {
                break;
            }
        }
    }

    private async Task ProcessMessageAsync(OutboxMessage message, CancellationToken ct)
    {
        if (message.RetryCount >= _config.MaxRetries)
        {
            return;
        }

        try
        {
            await _producer.PublishAsync(message, ct);
            await _store.MarkPublishedAsync(message.Id, ct);
        }
        catch (Exception ex) when (ex is not OperationCanceledException)
        {
            var error = ex.Message;
            await _store.MarkFailedAsync(message.Id, error, ct);
        }
    }

    public static TimeSpan CalculateBackoff(int retryCount, TimeSpan backoffBase)
    {
        var multiplier = Math.Pow(2, retryCount);
        return TimeSpan.FromTicks((long)(backoffBase.Ticks * multiplier));
    }
}
