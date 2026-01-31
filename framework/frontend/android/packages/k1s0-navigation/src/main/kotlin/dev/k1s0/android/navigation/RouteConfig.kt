package dev.k1s0.android.navigation

/**
 * Configuration for the application's navigation routes.
 *
 * Defines the complete set of routes, the start destination,
 * and optional navigation guards applied to all routes.
 *
 * @property routes The list of route entries that define available destinations.
 * @property startDestination The route path used as the initial destination.
 * @property globalGuards Navigation guards applied to every route transition.
 */
data class RouteConfig(
    val routes: List<RouteEntry>,
    val startDestination: String,
    val globalGuards: List<NavGuard> = emptyList(),
)
