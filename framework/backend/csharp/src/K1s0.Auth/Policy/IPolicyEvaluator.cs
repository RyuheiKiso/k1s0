namespace K1s0.Auth.Policy;

/// <summary>
/// Evaluates whether a policy request is allowed.
/// </summary>
public interface IPolicyEvaluator
{
    /// <summary>
    /// Evaluates whether the specified policy request is allowed.
    /// </summary>
    /// <param name="request">The policy request to evaluate.</param>
    /// <param name="ct">Cancellation token.</param>
    /// <returns><c>true</c> if the request is allowed; otherwise <c>false</c>.</returns>
    Task<bool> EvaluateAsync(PolicyRequest request, CancellationToken ct = default);
}
