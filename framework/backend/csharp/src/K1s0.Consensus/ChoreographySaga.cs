using Microsoft.Extensions.Hosting;
using Microsoft.Extensions.Logging;
using Microsoft.Extensions.Options;
using Npgsql;

namespace K1s0.Consensus;

/// <summary>
/// Handles an event-driven saga step in a choreography-based saga.
/// </summary>
/// <typeparam name="TEvent">The event type that triggers this step.</typeparam>
/// <typeparam name="TContext">The saga context type.</typeparam>
public interface IEventStepHandler<in TEvent, TContext>
    where TEvent : class
    where TContext : class
{
    /// <summary>
    /// The step name.
    /// </summary>
    string StepName { get; }

    /// <summary>
    /// Handles the event, executing the saga step logic.
    /// </summary>
    /// <param name="event">The triggering event.</param>
    /// <param name="context">The current saga context.</param>
    /// <param name="cancellationToken">Cancellation token.</param>
    /// <returns>The updated context.</returns>
    Task<TContext> HandleAsync(TEvent @event, TContext context, CancellationToken cancellationToken = default);

    /// <summary>
    /// Compensates this step.
    /// </summary>
    /// <param name="context">The current saga context.</param>
    /// <param name="cancellationToken">Cancellation token.</param>
    /// <returns>The updated context.</returns>
    Task<TContext> CompensateAsync(TContext context, CancellationToken cancellationToken = default);
}

/// <summary>
/// A step definition in a choreography saga.
/// </summary>
/// <typeparam name="TContext">The saga context type.</typeparam>
public sealed class ChoreographyStep<TContext> where TContext : class
{
    /// <summary>
    /// The step name.
    /// </summary>
    public string Name { get; init; } = string.Empty;

    /// <summary>
    /// The event type name that triggers this step.
    /// </summary>
    public string TriggerEventType { get; init; } = string.Empty;

    /// <summary>
    /// Timeout for this step. If not completed within this duration, the saga is compensated.
    /// </summary>
    public TimeSpan Timeout { get; init; } = TimeSpan.FromSeconds(30);

    /// <summary>
    /// The handler function for this step.
    /// </summary>
    public Func<object, TContext, CancellationToken, Task<TContext>>? Handler { get; init; }

    /// <summary>
    /// The compensation function for this step.
    /// </summary>
    public Func<TContext, CancellationToken, Task<TContext>>? Compensate { get; init; }
}

/// <summary>
/// Fluent builder for constructing choreography saga definitions.
/// </summary>
/// <typeparam name="TContext">The saga context type.</typeparam>
public sealed class ChoreographySagaBuilder<TContext> where TContext : class
{
    private readonly string _name;
    private readonly List<ChoreographyStep<TContext>> _steps = [];

    /// <summary>
    /// Creates a new <see cref="ChoreographySagaBuilder{TContext}"/>.
    /// </summary>
    /// <param name="name">The saga name.</param>
    public ChoreographySagaBuilder(string name)
    {
        _name = name;
    }

    /// <summary>
    /// Adds a step triggered by the specified event type.
    /// </summary>
    /// <typeparam name="TEvent">The event type.</typeparam>
    /// <param name="stepName">The step name.</param>
    /// <param name="handler">The handler.</param>
    /// <param name="compensate">The compensating action.</param>
    /// <param name="timeout">Optional step timeout.</param>
    /// <returns>This builder for chaining.</returns>
    public ChoreographySagaBuilder<TContext> On<TEvent>(
        string stepName,
        Func<TEvent, TContext, CancellationToken, Task<TContext>> handler,
        Func<TContext, CancellationToken, Task<TContext>> compensate,
        TimeSpan? timeout = null) where TEvent : class
    {
        _steps.Add(new ChoreographyStep<TContext>
        {
            Name = stepName,
            TriggerEventType = typeof(TEvent).FullName ?? typeof(TEvent).Name,
            Timeout = timeout ?? TimeSpan.FromSeconds(30),
            Handler = (evt, ctx, ct) => handler((TEvent)evt, ctx, ct),
            Compensate = compensate
        });
        return this;
    }

    /// <summary>
    /// Builds the choreography saga definition.
    /// </summary>
    /// <returns>A new <see cref="ChoreographySagaDefinition{TContext}"/>.</returns>
    public ChoreographySagaDefinition<TContext> Build()
    {
        if (_steps.Count == 0)
        {
            throw new InvalidOperationException("A choreography saga must have at least one step.");
        }

        return new ChoreographySagaDefinition<TContext>(_name, _steps.AsReadOnly());
    }
}

/// <summary>
/// A choreography-based saga definition.
/// </summary>
/// <typeparam name="TContext">The saga context type.</typeparam>
public sealed class ChoreographySagaDefinition<TContext> where TContext : class
{
    /// <summary>
    /// The saga name.
    /// </summary>
    public string Name { get; }

    /// <summary>
    /// The ordered steps.
    /// </summary>
    public IReadOnlyList<ChoreographyStep<TContext>> Steps { get; }

    /// <summary>
    /// Creates a new <see cref="ChoreographySagaDefinition{TContext}"/>.
    /// </summary>
    internal ChoreographySagaDefinition(string name, IReadOnlyList<ChoreographyStep<TContext>> steps)
    {
        Name = name;
        Steps = steps;
    }
}

/// <summary>
/// Background service that monitors choreography sagas for timeout violations.
/// When a step exceeds its timeout, compensation is triggered.
/// </summary>
public sealed class ChoreographyTimeoutMonitor : BackgroundService
{
    private readonly SagaConfig _config;
    private readonly ILogger<ChoreographyTimeoutMonitor> _logger;

    /// <summary>
    /// Creates a new <see cref="ChoreographyTimeoutMonitor"/>.
    /// </summary>
    /// <param name="options">Consensus configuration.</param>
    /// <param name="logger">Logger instance.</param>
    public ChoreographyTimeoutMonitor(
        IOptions<ConsensusConfig> options,
        ILogger<ChoreographyTimeoutMonitor> logger)
    {
        _config = options.Value.Saga;
        _logger = logger;
    }

    /// <inheritdoc />
    protected override async Task ExecuteAsync(CancellationToken stoppingToken)
    {
        _logger.LogInformation("Choreography timeout monitor started with interval {Interval}", _config.TimeoutMonitorInterval);

        using var timer = new PeriodicTimer(_config.TimeoutMonitorInterval);

        while (await timer.WaitForNextTickAsync(stoppingToken).ConfigureAwait(false))
        {
            try
            {
                await CheckTimeoutsAsync(stoppingToken).ConfigureAwait(false);
            }
            catch (OperationCanceledException) when (stoppingToken.IsCancellationRequested)
            {
                break;
            }
            catch (Exception ex)
            {
                _logger.LogError(ex, "Error during choreography timeout check");
            }
        }
    }

    private async Task CheckTimeoutsAsync(CancellationToken cancellationToken)
    {
        var connectionString = ReadConnectionString(_config.ConnectionStringFile);
        await using var conn = new NpgsqlConnection(connectionString);
        await conn.OpenAsync(cancellationToken).ConfigureAwait(false);

        // Find running sagas that haven't been updated within the timeout window
        var sql = $@"
            UPDATE {_config.TableName}
            SET status = @status, updated_at = NOW()
            WHERE status = @running
              AND updated_at < NOW() - INTERVAL '{(int)_config.DefaultStepTimeout.TotalSeconds} seconds'
            RETURNING saga_id";

        await using var cmd = new NpgsqlCommand(sql, conn);
        cmd.Parameters.AddWithValue("status", (int)SagaStatus.Failed);
        cmd.Parameters.AddWithValue("running", (int)SagaStatus.Running);

        await using var reader = await cmd.ExecuteReaderAsync(cancellationToken).ConfigureAwait(false);
        while (await reader.ReadAsync(cancellationToken).ConfigureAwait(false))
        {
            var sagaId = reader.GetString(0);
            _logger.LogWarning("Saga {SagaId} timed out and was marked as failed", sagaId);
            Metrics.SagaMetrics.TimeoutsTotal.Inc();
        }
    }

    private static string ReadConnectionString(string filePath)
    {
        if (string.IsNullOrWhiteSpace(filePath))
        {
            return string.Empty;
        }

        return File.ReadAllText(filePath).Trim();
    }
}
