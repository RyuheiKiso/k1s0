using System.Collections.Concurrent;
using System.Text.RegularExpressions;

namespace K1s0.Auth.Policy;

/// <summary>
/// In-memory implementation of <see cref="IPolicyRepository"/>.
/// Stores policy rules in a thread-safe dictionary keyed by resource pattern.
/// </summary>
public class InMemoryPolicyRepository : IPolicyRepository
{
    private readonly ConcurrentDictionary<string, List<PolicyRule>> _rules = new();

    /// <summary>
    /// Adds a policy rule to the repository.
    /// </summary>
    /// <param name="rule">The policy rule to add.</param>
    public void AddRule(PolicyRule rule)
    {
        _rules.AddOrUpdate(
            rule.ResourcePattern,
            _ => [rule],
            (_, existing) =>
            {
                existing.Add(rule);
                return existing;
            });
    }

    /// <inheritdoc />
    public Task<IReadOnlyList<PolicyRule>> GetRulesAsync(string resource, CancellationToken ct = default)
    {
        var matchingRules = new List<PolicyRule>();

        foreach (var (pattern, rules) in _rules)
        {
            if (MatchesPattern(resource, pattern))
            {
                matchingRules.AddRange(rules);
            }
        }

        return Task.FromResult<IReadOnlyList<PolicyRule>>(matchingRules);
    }

    private static bool MatchesPattern(string resource, string pattern)
    {
        if (pattern == "*")
        {
            return true;
        }

        if (!pattern.Contains('*'))
        {
            return string.Equals(resource, pattern, StringComparison.OrdinalIgnoreCase);
        }

        var regexPattern = "^" + Regex.Escape(pattern).Replace("\\*", ".*") + "$";
        return Regex.IsMatch(resource, regexPattern, RegexOptions.IgnoreCase);
    }
}
