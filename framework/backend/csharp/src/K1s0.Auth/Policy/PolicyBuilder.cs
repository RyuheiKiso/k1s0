namespace K1s0.Auth.Policy;

/// <summary>
/// Fluent builder for creating policy rules.
/// </summary>
public class PolicyBuilder
{
    private readonly List<PolicyRule> _rules = [];

    /// <summary>
    /// Adds a rule that allows admin access for subjects with the specified roles.
    /// </summary>
    /// <param name="resourcePattern">The resource pattern.</param>
    /// <param name="roles">The required roles.</param>
    /// <returns>This builder for chaining.</returns>
    public PolicyBuilder AllowAdmin(string resourcePattern, params string[] roles)
    {
        _rules.Add(new PolicyRule(Action.Admin, resourcePattern, roles, Array.Empty<string>(), Allow: true));
        return this;
    }

    /// <summary>
    /// Adds a rule that allows read access for subjects with the specified roles.
    /// </summary>
    /// <param name="resourcePattern">The resource pattern.</param>
    /// <param name="roles">The required roles.</param>
    /// <returns>This builder for chaining.</returns>
    public PolicyBuilder AllowRead(string resourcePattern, params string[] roles)
    {
        _rules.Add(new PolicyRule(Action.Read, resourcePattern, roles, Array.Empty<string>(), Allow: true));
        return this;
    }

    /// <summary>
    /// Adds a rule that allows write access for subjects with the specified roles.
    /// </summary>
    /// <param name="resourcePattern">The resource pattern.</param>
    /// <param name="roles">The required roles.</param>
    /// <returns>This builder for chaining.</returns>
    public PolicyBuilder AllowWrite(string resourcePattern, params string[] roles)
    {
        _rules.Add(new PolicyRule(Action.Write, resourcePattern, roles, Array.Empty<string>(), Allow: true));
        return this;
    }

    /// <summary>
    /// Adds a custom policy rule.
    /// </summary>
    /// <param name="rule">The policy rule.</param>
    /// <returns>This builder for chaining.</returns>
    public PolicyBuilder Custom(PolicyRule rule)
    {
        _rules.Add(rule);
        return this;
    }

    /// <summary>
    /// Builds an <see cref="InMemoryPolicyRepository"/> with the configured rules.
    /// </summary>
    /// <returns>A new repository containing all configured rules.</returns>
    public InMemoryPolicyRepository Build()
    {
        var repository = new InMemoryPolicyRepository();
        foreach (var rule in _rules)
        {
            repository.AddRule(rule);
        }

        return repository;
    }
}
