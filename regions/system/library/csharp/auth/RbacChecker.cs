namespace K1s0.System.Auth;

public static class RbacChecker
{
    public static bool CheckPermission(TokenClaims claims, string resource, string action)
    {
        // Check realm-level admin role
        if (claims.Roles.Contains("admin"))
        {
            return true;
        }

        // Check scope-based permission
        if (claims.Scope is not null)
        {
            var scopes = claims.Scope.Split(' ', StringSplitOptions.RemoveEmptyEntries);
            if (scopes.Contains(action) || scopes.Contains($"{resource}:{action}"))
            {
                return true;
            }
        }

        // Check resource_access roles
        if (claims.ResourceAccess.TryGetValue(resource, out var roles))
        {
            if (roles.Contains(action) || roles.Contains("admin"))
            {
                return true;
            }
        }

        // Check all resource_access for the action or admin role
        foreach (var entry in claims.ResourceAccess)
        {
            if (entry.Value.Contains(action) || entry.Value.Contains("admin"))
            {
                return true;
            }
        }

        return false;
    }
}
