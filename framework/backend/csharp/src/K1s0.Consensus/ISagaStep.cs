namespace K1s0.Consensus;

/// <summary>
/// Represents a single step in a saga workflow.
/// </summary>
/// <typeparam name="TContext">The saga context type that carries state between steps.</typeparam>
public interface ISagaStep<TContext> where TContext : class
{
    /// <summary>
    /// The unique name of this step within the saga.
    /// </summary>
    string Name { get; }

    /// <summary>
    /// Executes the forward action of this step.
    /// </summary>
    /// <param name="context">The saga context.</param>
    /// <param name="cancellationToken">Cancellation token.</param>
    /// <returns>The updated context.</returns>
    Task<TContext> ExecuteAsync(TContext context, CancellationToken cancellationToken = default);

    /// <summary>
    /// Executes the compensating action to undo this step.
    /// </summary>
    /// <param name="context">The saga context.</param>
    /// <param name="cancellationToken">Cancellation token.</param>
    /// <returns>The updated context.</returns>
    Task<TContext> CompensateAsync(TContext context, CancellationToken cancellationToken = default);
}
