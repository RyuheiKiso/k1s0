namespace K1s0.System.Retry;

public static class RetryPolicy
{
    public static async Task<T> WithRetryAsync<T>(RetryConfig config, Func<Task<T>> operation)
    {
        Exception? lastError = null;

        for (int attempt = 0; attempt < config.MaxAttempts; attempt++)
        {
            try
            {
                return await operation().ConfigureAwait(false);
            }
            catch (Exception ex)
            {
                lastError = ex;

                if (attempt < config.MaxAttempts - 1)
                {
                    var delay = config.ComputeDelay(attempt);
                    await Task.Delay(delay).ConfigureAwait(false);
                }
            }
        }

        throw new RetryExhaustedException(config.MaxAttempts, lastError!);
    }

    public static async Task WithRetryAsync(RetryConfig config, Func<Task> operation)
    {
        await WithRetryAsync(config, async () =>
        {
            await operation().ConfigureAwait(false);
            return 0;
        }).ConfigureAwait(false);
    }
}
