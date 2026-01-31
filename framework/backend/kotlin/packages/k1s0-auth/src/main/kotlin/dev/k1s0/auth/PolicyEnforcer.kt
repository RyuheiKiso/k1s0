package dev.k1s0.auth

import dev.k1s0.error.ForbiddenException

/**
 * Policy-based authorization enforcer.
 *
 * Evaluates whether a set of [Claims] satisfies a given policy.
 */
public object PolicyEnforcer {

    /**
     * Requires that the claims contain the specified role.
     *
     * @param claims The authenticated claims.
     * @param role The required role.
     * @throws ForbiddenException if the role is not present.
     */
    public fun requireRole(claims: Claims, role: String) {
        if (!claims.hasRole(role)) {
            throw ForbiddenException(
                serviceErrorCode = "auth.insufficient_role",
                detail = "Required role '$role' is not present",
            )
        }
    }

    /**
     * Requires that the claims contain the specified permission.
     *
     * @param claims The authenticated claims.
     * @param permission The required permission.
     * @throws ForbiddenException if the permission is not present.
     */
    public fun requirePermission(claims: Claims, permission: String) {
        if (!claims.hasPermission(permission)) {
            throw ForbiddenException(
                serviceErrorCode = "auth.insufficient_permission",
                detail = "Required permission '$permission' is not present",
            )
        }
    }

    /**
     * Requires that the claims contain any of the specified roles.
     *
     * @param claims The authenticated claims.
     * @param roles The set of acceptable roles.
     * @throws ForbiddenException if none of the roles are present.
     */
    public fun requireAnyRole(claims: Claims, vararg roles: String) {
        if (roles.none { claims.hasRole(it) }) {
            throw ForbiddenException(
                serviceErrorCode = "auth.insufficient_role",
                detail = "None of the required roles [${roles.joinToString()}] are present",
            )
        }
    }
}
