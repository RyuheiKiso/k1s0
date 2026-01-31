namespace K1s0.Auth.Policy;

/// <summary>
/// Repository for retrieving policy rules.
/// </summary>
public interface IPolicyRepository
{
    /// <summary>
    /// Gets the policy rules applicable to the specified resource.
    /// </summary>
    /// <param name="resource">The resource identifier.</param>
    /// <param name="ct">Cancellation token.</param>
    /// <returns>The list of applicable policy rules.</returns>
    Task<IReadOnlyList<PolicyRule>> GetRulesAsync(string resource, CancellationToken ct = default);
}
