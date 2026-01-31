package dev.k1s0.android.auth

/**
 * Represents the current authentication state of the user.
 */
sealed class AuthState {

    /** The user is not authenticated. */
    data object Unauthenticated : AuthState()

    /** Authentication is in progress. */
    data object Loading : AuthState()

    /**
     * The user is authenticated.
     *
     * @property accessToken The current JWT access token.
     * @property refreshToken The refresh token, if available.
     * @property userId The authenticated user's identifier.
     */
    data class Authenticated(
        val accessToken: String,
        val refreshToken: String? = null,
        val userId: String? = null,
    ) : AuthState()

    /**
     * Authentication failed.
     *
     * @property message A human-readable error message.
     * @property cause The underlying cause, if any.
     */
    data class Error(
        val message: String,
        val cause: Throwable? = null,
    ) : AuthState()
}
