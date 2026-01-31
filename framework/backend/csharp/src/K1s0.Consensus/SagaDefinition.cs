namespace K1s0.Consensus;

/// <summary>
/// Defines a saga workflow as an ordered sequence of steps.
/// </summary>
/// <typeparam name="TContext">The saga context type.</typeparam>
public sealed class SagaDefinition<TContext> where TContext : class
{
    /// <summary>
    /// The unique name of this saga definition.
    /// </summary>
    public string Name { get; }

    /// <summary>
    /// The ordered list of steps in this saga.
    /// </summary>
    public IReadOnlyList<ISagaStep<TContext>> Steps { get; }

    /// <summary>
    /// Per-step retry policies, keyed by step name. Steps not listed use default policy.
    /// </summary>
    public IReadOnlyDictionary<string, RetryPolicy> RetryPolicies { get; }

    /// <summary>
    /// The default retry policy for steps without a specific policy.
    /// </summary>
    public RetryPolicy DefaultRetryPolicy { get; }

    /// <summary>
    /// Creates a new <see cref="SagaDefinition{TContext}"/>.
    /// </summary>
    internal SagaDefinition(
        string name,
        IReadOnlyList<ISagaStep<TContext>> steps,
        IReadOnlyDictionary<string, RetryPolicy> retryPolicies,
        RetryPolicy defaultRetryPolicy)
    {
        Name = name;
        Steps = steps;
        RetryPolicies = retryPolicies;
        DefaultRetryPolicy = defaultRetryPolicy;
    }

    /// <summary>
    /// Gets the retry policy for a specific step.
    /// </summary>
    /// <param name="stepName">The step name.</param>
    /// <returns>The retry policy.</returns>
    public RetryPolicy GetRetryPolicy(string stepName) =>
        RetryPolicies.TryGetValue(stepName, out var policy) ? policy : DefaultRetryPolicy;
}

/// <summary>
/// Fluent builder for constructing <see cref="SagaDefinition{TContext}"/> instances.
/// </summary>
/// <typeparam name="TContext">The saga context type.</typeparam>
public sealed class SagaBuilder<TContext> where TContext : class
{
    private readonly string _name;
    private readonly List<ISagaStep<TContext>> _steps = [];
    private readonly Dictionary<string, RetryPolicy> _retryPolicies = [];
    private RetryPolicy _defaultRetryPolicy = new();

    /// <summary>
    /// Creates a new <see cref="SagaBuilder{TContext}"/>.
    /// </summary>
    /// <param name="name">The saga definition name.</param>
    public SagaBuilder(string name)
    {
        _name = name;
    }

    /// <summary>
    /// Adds a step to the saga.
    /// </summary>
    /// <param name="step">The step to add.</param>
    /// <returns>This builder for chaining.</returns>
    public SagaBuilder<TContext> AddStep(ISagaStep<TContext> step)
    {
        _steps.Add(step);
        return this;
    }

    /// <summary>
    /// Adds a step with an inline execute and compensate action.
    /// </summary>
    /// <param name="name">The step name.</param>
    /// <param name="execute">The forward action.</param>
    /// <param name="compensate">The compensating action.</param>
    /// <returns>This builder for chaining.</returns>
    public SagaBuilder<TContext> AddStep(
        string name,
        Func<TContext, CancellationToken, Task<TContext>> execute,
        Func<TContext, CancellationToken, Task<TContext>> compensate)
    {
        _steps.Add(new DelegateSagaStep<TContext>(name, execute, compensate));
        return this;
    }

    /// <summary>
    /// Sets a retry policy for a specific step.
    /// </summary>
    /// <param name="stepName">The step name.</param>
    /// <param name="policy">The retry policy.</param>
    /// <returns>This builder for chaining.</returns>
    public SagaBuilder<TContext> WithRetryPolicy(string stepName, RetryPolicy policy)
    {
        _retryPolicies[stepName] = policy;
        return this;
    }

    /// <summary>
    /// Sets the default retry policy for all steps.
    /// </summary>
    /// <param name="policy">The default retry policy.</param>
    /// <returns>This builder for chaining.</returns>
    public SagaBuilder<TContext> WithDefaultRetryPolicy(RetryPolicy policy)
    {
        _defaultRetryPolicy = policy;
        return this;
    }

    /// <summary>
    /// Builds the saga definition.
    /// </summary>
    /// <returns>A new <see cref="SagaDefinition{TContext}"/>.</returns>
    /// <exception cref="InvalidOperationException">Thrown when no steps have been added.</exception>
    public SagaDefinition<TContext> Build()
    {
        if (_steps.Count == 0)
        {
            throw new InvalidOperationException("A saga must have at least one step.");
        }

        return new SagaDefinition<TContext>(
            _name,
            _steps.AsReadOnly(),
            _retryPolicies.AsReadOnly(),
            _defaultRetryPolicy);
    }
}

/// <summary>
/// A saga step implemented via delegates.
/// </summary>
/// <typeparam name="TContext">The saga context type.</typeparam>
internal sealed class DelegateSagaStep<TContext> : ISagaStep<TContext> where TContext : class
{
    private readonly Func<TContext, CancellationToken, Task<TContext>> _execute;
    private readonly Func<TContext, CancellationToken, Task<TContext>> _compensate;

    /// <inheritdoc />
    public string Name { get; }

    /// <summary>
    /// Creates a new <see cref="DelegateSagaStep{TContext}"/>.
    /// </summary>
    public DelegateSagaStep(
        string name,
        Func<TContext, CancellationToken, Task<TContext>> execute,
        Func<TContext, CancellationToken, Task<TContext>> compensate)
    {
        Name = name;
        _execute = execute;
        _compensate = compensate;
    }

    /// <inheritdoc />
    public Task<TContext> ExecuteAsync(TContext context, CancellationToken cancellationToken = default) =>
        _execute(context, cancellationToken);

    /// <inheritdoc />
    public Task<TContext> CompensateAsync(TContext context, CancellationToken cancellationToken = default) =>
        _compensate(context, cancellationToken);
}
