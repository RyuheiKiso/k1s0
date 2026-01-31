package dev.k1s0.android.navigation

import androidx.compose.runtime.Composable

/**
 * Represents a single navigable route entry in the application.
 *
 * Each route entry defines a destination with its path, composable content,
 * optional guards, and nested child routes.
 */
sealed class RouteEntry {

    /** The route path string used for navigation (e.g. "home", "profile/{id}"). */
    abstract val route: String

    /** Navigation guards specific to this route entry. */
    abstract val guards: List<NavGuard>

    /**
     * A simple screen destination with no nested routes.
     *
     * @property route The route path string.
     * @property guards Guards applied before navigating to this screen.
     * @property content The composable content to render for this route.
     */
    data class Screen(
        override val route: String,
        override val guards: List<NavGuard> = emptyList(),
        val content: @Composable () -> Unit,
    ) : RouteEntry()

    /**
     * A route group that contains nested child routes, rendered inside a shared layout.
     *
     * @property route The parent route path string.
     * @property guards Guards applied to the group and inherited by children.
     * @property children The nested route entries under this group.
     */
    data class Group(
        override val route: String,
        override val guards: List<NavGuard> = emptyList(),
        val children: List<RouteEntry>,
    ) : RouteEntry()
}
