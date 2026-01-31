package dev.k1s0.android.navigation

import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.remember
import androidx.navigation.NavHostController
import androidx.navigation.compose.NavHost
import androidx.navigation.compose.composable
import androidx.navigation.compose.rememberNavController

/**
 * A config-driven NavHost wrapper that builds the navigation graph
 * from a [RouteConfig] and applies navigation guards.
 *
 * @param config The route configuration defining all destinations and guards.
 * @param navController The [NavHostController] to use. Defaults to a new one via [rememberNavController].
 */
@Composable
fun K1s0NavHost(
    config: RouteConfig,
    navController: NavHostController = rememberNavController(),
) {
    val allGuards = remember(config) { config.globalGuards }

    NavHost(
        navController = navController,
        startDestination = config.startDestination,
    ) {
        config.routes.forEach { entry ->
            when (entry) {
                is RouteEntry.Screen -> {
                    composable(entry.route) {
                        GuardedContent(
                            route = entry.route,
                            guards = allGuards + entry.guards,
                            navController = navController,
                        ) {
                            entry.content()
                        }
                    }
                }
                is RouteEntry.Group -> {
                    entry.children.forEach { child ->
                        if (child is RouteEntry.Screen) {
                            composable(child.route) {
                                GuardedContent(
                                    route = child.route,
                                    guards = allGuards + entry.guards + child.guards,
                                    navController = navController,
                                ) {
                                    child.content()
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/**
 * Wraps content with navigation guard evaluation.
 * If any guard denies or redirects, the content is not rendered.
 */
@Composable
private fun GuardedContent(
    route: String,
    guards: List<NavGuard>,
    navController: NavHostController,
    content: @Composable () -> Unit,
) {
    if (guards.isEmpty()) {
        content()
        return
    }

    LaunchedEffect(route) {
        for (guard in guards) {
            when (val result = guard.canActivate(route)) {
                is NavGuardResult.Allow -> continue
                is NavGuardResult.Redirect -> {
                    navController.navigate(result.redirectRoute) {
                        popUpTo(route) { inclusive = true }
                    }
                    return@LaunchedEffect
                }
                is NavGuardResult.Deny -> {
                    navController.popBackStack()
                    return@LaunchedEffect
                }
            }
        }
    }

    content()
}
