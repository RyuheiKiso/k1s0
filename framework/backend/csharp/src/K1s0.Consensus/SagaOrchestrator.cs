using System.Text.Json;
using Microsoft.Extensions.Logging;
using Microsoft.Extensions.Options;
using Npgsql;

namespace K1s0.Consensus;

/// <summary>
/// Orchestrates saga execution with persistence, retry, and compensation support.
/// Uses PostgreSQL for durable saga state storage.
/// </summary>
public sealed class SagaOrchestrator
{
    private readonly SagaConfig _config;
    private readonly ILogger<SagaOrchestrator> _logger;
    private readonly string _connectionString;

    /// <summary>
    /// Creates a new <see cref="SagaOrchestrator"/>.
    /// </summary>
    /// <param name="options">Consensus configuration.</param>
    /// <param name="logger">Logger instance.</param>
    public SagaOrchestrator(IOptions<ConsensusConfig> options, ILogger<SagaOrchestrator> logger)
    {
        _config = options.Value.Saga;
        _logger = logger;
        _connectionString = ReadConnectionString(_config.ConnectionStringFile);
    }

    /// <summary>
    /// Executes a saga from the beginning.
    /// </summary>
    /// <typeparam name="TContext">The saga context type.</typeparam>
    /// <param name="definition">The saga definition.</param>
    /// <param name="context">The initial context.</param>
    /// <param name="cancellationToken">Cancellation token.</param>
    /// <returns>The saga result.</returns>
    public async Task<SagaResult<TContext>> ExecuteAsync<TContext>(
        SagaDefinition<TContext> definition,
        TContext context,
        CancellationToken cancellationToken = default) where TContext : class
    {
        var sagaId = Guid.NewGuid().ToString("N");
        _logger.LogInformation("Starting saga {SagaName} with ID {SagaId}", definition.Name, sagaId);
        Metrics.SagaMetrics.StartedTotal.Inc();

        var instance = new SagaInstance
        {
            SagaId = sagaId,
            SagaName = definition.Name,
            Status = SagaStatus.Running,
            ContextJson = JsonSerializer.Serialize(context),
            CurrentStep = 0
        };

        await PersistInstanceAsync(instance, cancellationToken).ConfigureAwait(false);

        return await RunStepsAsync(definition, instance, context, 0, cancellationToken).ConfigureAwait(false);
    }

    /// <summary>
    /// Resumes a previously suspended or failed saga from the last completed step.
    /// </summary>
    /// <typeparam name="TContext">The saga context type.</typeparam>
    /// <param name="definition">The saga definition.</param>
    /// <param name="sagaId">The saga instance ID to resume.</param>
    /// <param name="cancellationToken">Cancellation token.</param>
    /// <returns>The saga result.</returns>
    public async Task<SagaResult<TContext>> ResumeAsync<TContext>(
        SagaDefinition<TContext> definition,
        string sagaId,
        CancellationToken cancellationToken = default) where TContext : class
    {
        var instance = await LoadInstanceAsync(sagaId, cancellationToken).ConfigureAwait(false)
            ?? throw new ConsensusException("consensus.saga.not_found", $"Saga instance '{sagaId}' not found.");

        var context = instance.DeserializeContext<TContext>()
            ?? throw new ConsensusException("consensus.saga.invalid_context", $"Failed to deserialize context for saga '{sagaId}'.");

        _logger.LogInformation("Resuming saga {SagaId} from step {Step}", sagaId, instance.CurrentStep);

        return await RunStepsAsync(definition, instance, context, instance.CurrentStep, cancellationToken).ConfigureAwait(false);
    }

    /// <summary>
    /// Returns all saga instances that have reached the dead letter state (failed compensation).
    /// </summary>
    /// <param name="sagaName">Optional filter by saga name.</param>
    /// <param name="cancellationToken">Cancellation token.</param>
    /// <returns>Dead letter saga instances.</returns>
    public async Task<IReadOnlyList<SagaInstance>> DeadLettersAsync(string? sagaName = null, CancellationToken cancellationToken = default)
    {
        await using var conn = new NpgsqlConnection(_connectionString);
        await conn.OpenAsync(cancellationToken).ConfigureAwait(false);

        var sql = sagaName is not null
            ? $"SELECT saga_id, saga_name, status, context_json, current_step, created_at, updated_at, errors_json FROM {_config.DeadLetterTableName} WHERE saga_name = @name ORDER BY created_at DESC"
            : $"SELECT saga_id, saga_name, status, context_json, current_step, created_at, updated_at, errors_json FROM {_config.DeadLetterTableName} ORDER BY created_at DESC";

        await using var cmd = new NpgsqlCommand(sql, conn);
        if (sagaName is not null)
        {
            cmd.Parameters.AddWithValue("name", sagaName);
        }

        var results = new List<SagaInstance>();
        await using var reader = await cmd.ExecuteReaderAsync(cancellationToken).ConfigureAwait(false);
        while (await reader.ReadAsync(cancellationToken).ConfigureAwait(false))
        {
            results.Add(ReadInstance(reader));
        }

        return results.AsReadOnly();
    }

    private async Task<SagaResult<TContext>> RunStepsAsync<TContext>(
        SagaDefinition<TContext> definition,
        SagaInstance instance,
        TContext context,
        int startStep,
        CancellationToken cancellationToken) where TContext : class
    {
        var errors = new Dictionary<string, string>();
        var completedSteps = new List<int>();

        for (var i = startStep; i < definition.Steps.Count; i++)
        {
            var step = definition.Steps[i];
            var retryPolicy = definition.GetRetryPolicy(step.Name);

            instance.CurrentStep = i;
            instance.ContextJson = JsonSerializer.Serialize(context);
            await UpdateInstanceAsync(instance, cancellationToken).ConfigureAwait(false);

            var success = false;

            for (var attempt = 0; attempt <= retryPolicy.MaxRetries; attempt++)
            {
                try
                {
                    using var stepCts = CancellationTokenSource.CreateLinkedTokenSource(cancellationToken);
                    stepCts.CancelAfter(_config.DefaultStepTimeout);

                    context = await step.ExecuteAsync(context, stepCts.Token).ConfigureAwait(false);
                    completedSteps.Add(i);
                    success = true;
                    _logger.LogDebug("Saga {SagaId} step {StepName} completed", instance.SagaId, step.Name);
                    Metrics.SagaMetrics.StepsCompletedTotal.Inc();
                    break;
                }
                catch (OperationCanceledException) when (cancellationToken.IsCancellationRequested)
                {
                    throw;
                }
                catch (Exception ex)
                {
                    _logger.LogWarning(ex, "Saga {SagaId} step {StepName} attempt {Attempt} failed",
                        instance.SagaId, step.Name, attempt + 1);

                    if (attempt < retryPolicy.MaxRetries)
                    {
                        var delay = retryPolicy.GetDelay(attempt);
                        await Task.Delay(delay, cancellationToken).ConfigureAwait(false);
                    }
                    else
                    {
                        errors[step.Name] = ex.Message;
                        Metrics.SagaMetrics.StepsFailedTotal.Inc();
                    }
                }
            }

            if (!success)
            {
                // Compensate completed steps in reverse
                _logger.LogWarning("Saga {SagaId} compensating from step {Step}", instance.SagaId, i);
                return await CompensateAsync(definition, instance, context, completedSteps, errors, cancellationToken).ConfigureAwait(false);
            }
        }

        instance.Status = SagaStatus.Completed;
        instance.ContextJson = JsonSerializer.Serialize(context);
        instance.UpdatedAt = DateTimeOffset.UtcNow;
        await UpdateInstanceAsync(instance, cancellationToken).ConfigureAwait(false);

        _logger.LogInformation("Saga {SagaId} completed successfully", instance.SagaId);
        Metrics.SagaMetrics.CompletedTotal.Inc();

        return new SagaResult<TContext>
        {
            Status = SagaStatus.Completed,
            Context = context,
            SagaId = instance.SagaId,
            LastCompletedStep = definition.Steps.Count - 1
        };
    }

    private async Task<SagaResult<TContext>> CompensateAsync<TContext>(
        SagaDefinition<TContext> definition,
        SagaInstance instance,
        TContext context,
        List<int> completedSteps,
        Dictionary<string, string> errors,
        CancellationToken cancellationToken) where TContext : class
    {
        var compensationFailed = false;

        for (var i = completedSteps.Count - 1; i >= 0; i--)
        {
            var stepIndex = completedSteps[i];
            var step = definition.Steps[stepIndex];

            try
            {
                context = await step.CompensateAsync(context, cancellationToken).ConfigureAwait(false);
                _logger.LogDebug("Saga {SagaId} compensated step {StepName}", instance.SagaId, step.Name);
            }
            catch (Exception ex)
            {
                _logger.LogError(ex, "Saga {SagaId} compensation failed for step {StepName}", instance.SagaId, step.Name);
                errors[$"{step.Name}_compensation"] = ex.Message;
                compensationFailed = true;
            }
        }

        var status = compensationFailed ? SagaStatus.Failed : SagaStatus.Compensated;

        instance.Status = status;
        instance.ContextJson = JsonSerializer.Serialize(context);
        instance.ErrorsJson = JsonSerializer.Serialize(errors);
        instance.UpdatedAt = DateTimeOffset.UtcNow;
        await UpdateInstanceAsync(instance, cancellationToken).ConfigureAwait(false);

        if (compensationFailed)
        {
            _logger.LogError("Saga {SagaId} moved to dead letter", instance.SagaId);
            await MoveToDeadLetterAsync(instance, cancellationToken).ConfigureAwait(false);
            Metrics.SagaMetrics.DeadLetterTotal.Inc();
        }
        else
        {
            Metrics.SagaMetrics.CompensatedTotal.Inc();
        }

        return new SagaResult<TContext>
        {
            Status = status,
            Context = context,
            SagaId = instance.SagaId,
            Errors = errors.AsReadOnly(),
            LastCompletedStep = completedSteps.Count > 0 ? completedSteps[^1] : -1
        };
    }

    private async Task PersistInstanceAsync(SagaInstance instance, CancellationToken cancellationToken)
    {
        await using var conn = new NpgsqlConnection(_connectionString);
        await conn.OpenAsync(cancellationToken).ConfigureAwait(false);

        var sql = $@"
            INSERT INTO {_config.TableName} (saga_id, saga_name, status, context_json, current_step, created_at, updated_at, errors_json)
            VALUES (@id, @name, @status, @context::jsonb, @step, @created, @updated, @errors::jsonb)";

        await using var cmd = new NpgsqlCommand(sql, conn);
        cmd.Parameters.AddWithValue("id", instance.SagaId);
        cmd.Parameters.AddWithValue("name", instance.SagaName);
        cmd.Parameters.AddWithValue("status", (int)instance.Status);
        cmd.Parameters.AddWithValue("context", instance.ContextJson);
        cmd.Parameters.AddWithValue("step", instance.CurrentStep);
        cmd.Parameters.AddWithValue("created", instance.CreatedAt);
        cmd.Parameters.AddWithValue("updated", instance.UpdatedAt);
        cmd.Parameters.AddWithValue("errors", instance.ErrorsJson);

        await cmd.ExecuteNonQueryAsync(cancellationToken).ConfigureAwait(false);
    }

    private async Task UpdateInstanceAsync(SagaInstance instance, CancellationToken cancellationToken)
    {
        await using var conn = new NpgsqlConnection(_connectionString);
        await conn.OpenAsync(cancellationToken).ConfigureAwait(false);

        var sql = $@"
            UPDATE {_config.TableName}
            SET status = @status, context_json = @context::jsonb, current_step = @step, updated_at = @updated, errors_json = @errors::jsonb
            WHERE saga_id = @id";

        await using var cmd = new NpgsqlCommand(sql, conn);
        cmd.Parameters.AddWithValue("id", instance.SagaId);
        cmd.Parameters.AddWithValue("status", (int)instance.Status);
        cmd.Parameters.AddWithValue("context", instance.ContextJson);
        cmd.Parameters.AddWithValue("step", instance.CurrentStep);
        cmd.Parameters.AddWithValue("updated", DateTimeOffset.UtcNow);
        cmd.Parameters.AddWithValue("errors", instance.ErrorsJson);

        await cmd.ExecuteNonQueryAsync(cancellationToken).ConfigureAwait(false);
    }

    private async Task<SagaInstance?> LoadInstanceAsync(string sagaId, CancellationToken cancellationToken)
    {
        await using var conn = new NpgsqlConnection(_connectionString);
        await conn.OpenAsync(cancellationToken).ConfigureAwait(false);

        var sql = $"SELECT saga_id, saga_name, status, context_json, current_step, created_at, updated_at, errors_json FROM {_config.TableName} WHERE saga_id = @id";

        await using var cmd = new NpgsqlCommand(sql, conn);
        cmd.Parameters.AddWithValue("id", sagaId);

        await using var reader = await cmd.ExecuteReaderAsync(cancellationToken).ConfigureAwait(false);
        if (!await reader.ReadAsync(cancellationToken).ConfigureAwait(false))
        {
            return null;
        }

        return ReadInstance(reader);
    }

    private async Task MoveToDeadLetterAsync(SagaInstance instance, CancellationToken cancellationToken)
    {
        await using var conn = new NpgsqlConnection(_connectionString);
        await conn.OpenAsync(cancellationToken).ConfigureAwait(false);

        var sql = $@"
            INSERT INTO {_config.DeadLetterTableName} (saga_id, saga_name, status, context_json, current_step, created_at, updated_at, errors_json)
            VALUES (@id, @name, @status, @context::jsonb, @step, @created, @updated, @errors::jsonb)
            ON CONFLICT (saga_id) DO UPDATE SET status = @status, context_json = @context::jsonb, updated_at = @updated, errors_json = @errors::jsonb";

        await using var cmd = new NpgsqlCommand(sql, conn);
        cmd.Parameters.AddWithValue("id", instance.SagaId);
        cmd.Parameters.AddWithValue("name", instance.SagaName);
        cmd.Parameters.AddWithValue("status", (int)instance.Status);
        cmd.Parameters.AddWithValue("context", instance.ContextJson);
        cmd.Parameters.AddWithValue("step", instance.CurrentStep);
        cmd.Parameters.AddWithValue("created", instance.CreatedAt);
        cmd.Parameters.AddWithValue("updated", instance.UpdatedAt);
        cmd.Parameters.AddWithValue("errors", instance.ErrorsJson);

        await cmd.ExecuteNonQueryAsync(cancellationToken).ConfigureAwait(false);
    }

    private static SagaInstance ReadInstance(NpgsqlDataReader reader) => new()
    {
        SagaId = reader.GetString(0),
        SagaName = reader.GetString(1),
        Status = (SagaStatus)reader.GetInt32(2),
        ContextJson = reader.GetString(3),
        CurrentStep = reader.GetInt32(4),
        CreatedAt = reader.GetFieldValue<DateTimeOffset>(5),
        UpdatedAt = reader.GetFieldValue<DateTimeOffset>(6),
        ErrorsJson = reader.GetString(7)
    };

    private static string ReadConnectionString(string filePath)
    {
        if (string.IsNullOrWhiteSpace(filePath))
        {
            throw new ConsensusException(
                "consensus.config.missing_connection_string",
                "Saga connection_string_file is not configured.");
        }

        return File.ReadAllText(filePath).Trim();
    }
}
