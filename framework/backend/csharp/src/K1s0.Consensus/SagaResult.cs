using System.Text.Json;

namespace K1s0.Consensus;

/// <summary>
/// Status of a saga instance.
/// </summary>
public enum SagaStatus
{
    /// <summary>
    /// Saga is currently executing steps.
    /// </summary>
    Running,

    /// <summary>
    /// All steps completed successfully.
    /// </summary>
    Completed,

    /// <summary>
    /// A step failed and compensation was executed.
    /// </summary>
    Compensated,

    /// <summary>
    /// A step failed and compensation also failed. Requires manual intervention.
    /// </summary>
    Failed,

    /// <summary>
    /// Saga is paused, awaiting resume.
    /// </summary>
    Suspended
}

/// <summary>
/// The result of a saga execution.
/// </summary>
/// <typeparam name="TContext">The saga context type.</typeparam>
public sealed class SagaResult<TContext> where TContext : class
{
    /// <summary>
    /// The final status of the saga.
    /// </summary>
    public SagaStatus Status { get; init; }

    /// <summary>
    /// The final context state.
    /// </summary>
    public TContext? Context { get; init; }

    /// <summary>
    /// The saga instance identifier.
    /// </summary>
    public string SagaId { get; init; } = string.Empty;

    /// <summary>
    /// Errors encountered during execution or compensation, keyed by step name.
    /// </summary>
    public IReadOnlyDictionary<string, string> Errors { get; init; } = new Dictionary<string, string>();

    /// <summary>
    /// The index of the last successfully completed step (0-based), or -1 if none completed.
    /// </summary>
    public int LastCompletedStep { get; init; } = -1;
}

/// <summary>
/// A persisted saga instance record.
/// </summary>
public sealed class SagaInstance
{
    /// <summary>
    /// Unique identifier for this saga instance.
    /// </summary>
    public string SagaId { get; init; } = string.Empty;

    /// <summary>
    /// The saga definition name.
    /// </summary>
    public string SagaName { get; init; } = string.Empty;

    /// <summary>
    /// Current status.
    /// </summary>
    public SagaStatus Status { get; set; }

    /// <summary>
    /// The serialized context JSON.
    /// </summary>
    public string ContextJson { get; set; } = "{}";

    /// <summary>
    /// The index of the current step being executed.
    /// </summary>
    public int CurrentStep { get; set; }

    /// <summary>
    /// When this instance was created.
    /// </summary>
    public DateTimeOffset CreatedAt { get; init; } = DateTimeOffset.UtcNow;

    /// <summary>
    /// When this instance was last updated.
    /// </summary>
    public DateTimeOffset UpdatedAt { get; set; } = DateTimeOffset.UtcNow;

    /// <summary>
    /// Serialized error information as JSON.
    /// </summary>
    public string ErrorsJson { get; set; } = "{}";

    /// <summary>
    /// Deserializes the context from JSON.
    /// </summary>
    /// <typeparam name="TContext">The context type.</typeparam>
    /// <returns>The deserialized context.</returns>
    public TContext? DeserializeContext<TContext>() where TContext : class =>
        JsonSerializer.Deserialize<TContext>(ContextJson);
}
