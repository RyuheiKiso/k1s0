package dev.k1s0.android.navigation

import io.mockk.coEvery
import io.mockk.mockk
import kotlinx.coroutines.test.runTest
import org.junit.jupiter.api.Assertions.assertEquals
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.Test

class K1s0NavHostTest {

    @Test
    fun `RouteConfig holds routes and start destination`() {
        val config = RouteConfig(
            routes = listOf(
                RouteEntry.Screen(route = "home") {},
                RouteEntry.Screen(route = "settings") {},
            ),
            startDestination = "home",
        )

        assertEquals("home", config.startDestination)
        assertEquals(2, config.routes.size)
    }

    @Test
    fun `NavGuard allow result is correct type`() = runTest {
        val guard = mockk<NavGuard>()
        coEvery { guard.canActivate(any()) } returns NavGuardResult.Allow

        val result = guard.canActivate("home")
        assertTrue(result is NavGuardResult.Allow)
    }

    @Test
    fun `NavGuard redirect result carries route`() = runTest {
        val guard = mockk<NavGuard>()
        coEvery { guard.canActivate("admin") } returns NavGuardResult.Redirect("login")

        val result = guard.canActivate("admin")
        assertTrue(result is NavGuardResult.Redirect)
        assertEquals("login", (result as NavGuardResult.Redirect).redirectRoute)
    }

    @Test
    fun `RouteEntry Group contains children`() {
        val group = RouteEntry.Group(
            route = "settings",
            children = listOf(
                RouteEntry.Screen(route = "settings/profile") {},
                RouteEntry.Screen(route = "settings/account") {},
            ),
        )

        assertEquals(2, group.children.size)
        assertEquals("settings/profile", group.children[0].route)
    }
}
