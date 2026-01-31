package dev.k1s0.android.navigation

/**
 * Interface for navigation guards that control route access.
 *
 * Implementations can perform checks (e.g. authentication, authorization)
 * and either allow navigation to proceed or redirect to a different route.
 */
interface NavGuard {

    /**
     * Evaluates whether navigation to the given [route] should be allowed.
     *
     * @param route The target route path being navigated to.
     * @return A [NavGuardResult] indicating whether to proceed or redirect.
     */
    suspend fun canActivate(route: String): NavGuardResult
}

/**
 * Result of a navigation guard evaluation.
 */
sealed class NavGuardResult {

    /** Navigation is allowed to proceed. */
    data object Allow : NavGuardResult()

    /**
     * Navigation is denied; redirect to a different route.
     *
     * @property redirectRoute The route to redirect to instead.
     */
    data class Redirect(val redirectRoute: String) : NavGuardResult()

    /** Navigation is denied with no redirect. */
    data object Deny : NavGuardResult()
}
