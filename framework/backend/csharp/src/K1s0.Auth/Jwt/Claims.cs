namespace K1s0.Auth.Jwt;

/// <summary>
/// Represents the verified claims extracted from a JWT token.
/// </summary>
/// <param name="Sub">The subject identifier.</param>
/// <param name="Roles">The roles assigned to the subject.</param>
/// <param name="Permissions">The permissions granted to the subject.</param>
/// <param name="Groups">The groups the subject belongs to.</param>
/// <param name="TenantId">The tenant identifier for multi-tenant scenarios.</param>
/// <param name="Custom">Additional custom claims.</param>
public record Claims(
    string Sub,
    IReadOnlyList<string> Roles,
    IReadOnlyList<string> Permissions,
    IReadOnlyList<string> Groups,
    string? TenantId,
    IReadOnlyDictionary<string, object> Custom)
{
    /// <summary>
    /// Checks whether the subject has the specified role.
    /// </summary>
    /// <param name="role">The role to check.</param>
    /// <returns><c>true</c> if the subject has the role; otherwise <c>false</c>.</returns>
    public bool HasRole(string role) =>
        Roles.Contains(role, StringComparer.OrdinalIgnoreCase);

    /// <summary>
    /// Checks whether the subject has the specified permission.
    /// </summary>
    /// <param name="permission">The permission to check.</param>
    /// <returns><c>true</c> if the subject has the permission; otherwise <c>false</c>.</returns>
    public bool HasPermission(string permission) =>
        Permissions.Contains(permission, StringComparer.OrdinalIgnoreCase);

    /// <summary>
    /// Checks whether the subject has any of the specified roles.
    /// </summary>
    /// <param name="roles">The roles to check.</param>
    /// <returns><c>true</c> if the subject has at least one of the roles; otherwise <c>false</c>.</returns>
    public bool HasAnyRole(IEnumerable<string> roles) =>
        roles.Any(r => HasRole(r));
}
