package dev.k1s0.auth

import kotlinx.serialization.Serializable

/**
 * Represents the claims extracted from a verified JWT token.
 *
 * @property subject The subject (user ID) of the token.
 * @property issuer The issuer of the token.
 * @property roles The roles assigned to the subject.
 * @property permissions The permissions assigned to the subject.
 * @property customClaims Additional custom claims as key-value pairs.
 */
@Serializable
public data class Claims(
    val subject: String,
    val issuer: String? = null,
    val roles: List<String> = emptyList(),
    val permissions: List<String> = emptyList(),
    val customClaims: Map<String, String> = emptyMap(),
) {
    /** Checks whether the claims include the specified role. */
    public fun hasRole(role: String): Boolean = roles.contains(role)

    /** Checks whether the claims include the specified permission. */
    public fun hasPermission(permission: String): Boolean = permissions.contains(permission)
}
