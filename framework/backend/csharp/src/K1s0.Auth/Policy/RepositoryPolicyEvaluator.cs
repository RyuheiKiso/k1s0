namespace K1s0.Auth.Policy;

/// <summary>
/// Evaluates policy requests against rules from an <see cref="IPolicyRepository"/>.
/// </summary>
public class RepositoryPolicyEvaluator : IPolicyEvaluator
{
    private readonly IPolicyRepository _repository;

    /// <summary>
    /// Initializes a new instance of the <see cref="RepositoryPolicyEvaluator"/> class.
    /// </summary>
    /// <param name="repository">The policy repository to retrieve rules from.</param>
    public RepositoryPolicyEvaluator(IPolicyRepository repository)
    {
        _repository = repository ?? throw new ArgumentNullException(nameof(repository));
    }

    /// <inheritdoc />
    public async Task<bool> EvaluateAsync(PolicyRequest request, CancellationToken ct = default)
    {
        var rules = await _repository.GetRulesAsync(request.Resource, ct).ConfigureAwait(false);

        foreach (var rule in rules.Where(r => r.Action == request.Action))
        {
            if (MatchesRule(request.Subject, rule))
            {
                return rule.Allow;
            }
        }

        return false;
    }

    private static bool MatchesRule(PolicySubject subject, PolicyRule rule)
    {
        if (rule.RequiredRoles.Count > 0)
        {
            var hasRole = rule.RequiredRoles.Any(r =>
                subject.Roles.Contains(r, StringComparer.OrdinalIgnoreCase));
            if (!hasRole)
            {
                return false;
            }
        }

        if (rule.RequiredPermissions.Count > 0)
        {
            var hasPermission = rule.RequiredPermissions.Any(p =>
                subject.Permissions.Contains(p, StringComparer.OrdinalIgnoreCase));
            if (!hasPermission)
            {
                return false;
            }
        }

        return true;
    }
}
