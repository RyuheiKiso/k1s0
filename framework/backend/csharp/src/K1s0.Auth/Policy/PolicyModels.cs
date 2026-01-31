namespace K1s0.Auth.Policy;

/// <summary>
/// Defines the types of actions that can be performed on a resource.
/// </summary>
public enum Action
{
    /// <summary>Read access.</summary>
    Read,

    /// <summary>Write access.</summary>
    Write,

    /// <summary>Delete access.</summary>
    Delete,

    /// <summary>Administrative access.</summary>
    Admin,
}

/// <summary>
/// Represents the subject of a policy evaluation.
/// </summary>
/// <param name="Sub">The subject identifier.</param>
/// <param name="Roles">The subject's roles.</param>
/// <param name="Permissions">The subject's permissions.</param>
/// <param name="Groups">The subject's groups.</param>
/// <param name="TenantId">The subject's tenant identifier.</param>
public record PolicySubject(
    string Sub,
    IReadOnlyList<string> Roles,
    IReadOnlyList<string> Permissions,
    IReadOnlyList<string> Groups,
    string? TenantId);

/// <summary>
/// Represents a request to evaluate a policy.
/// </summary>
/// <param name="Subject">The subject requesting access.</param>
/// <param name="Action">The action being requested.</param>
/// <param name="Resource">The resource being accessed.</param>
public record PolicyRequest(
    PolicySubject Subject,
    Action Action,
    string Resource);

/// <summary>
/// Represents a single policy rule.
/// </summary>
/// <param name="Action">The action this rule applies to.</param>
/// <param name="ResourcePattern">The resource pattern (supports wildcard *).</param>
/// <param name="RequiredRoles">Roles required for this rule. Empty means no role requirement.</param>
/// <param name="RequiredPermissions">Permissions required for this rule. Empty means no permission requirement.</param>
/// <param name="Allow">Whether the rule allows or denies access.</param>
public record PolicyRule(
    Action Action,
    string ResourcePattern,
    IReadOnlyList<string> RequiredRoles,
    IReadOnlyList<string> RequiredPermissions,
    bool Allow);
