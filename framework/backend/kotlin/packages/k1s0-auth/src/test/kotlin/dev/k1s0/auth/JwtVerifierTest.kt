package dev.k1s0.auth

import dev.k1s0.error.ForbiddenException
import io.kotest.assertions.throwables.shouldThrow
import io.kotest.matchers.shouldBe
import org.junit.jupiter.api.Test

class JwtVerifierTest {

    @Test
    fun `Claims hasRole returns true when role exists`() {
        val claims = Claims(
            subject = "user-1",
            roles = listOf("admin", "user"),
        )

        claims.hasRole("admin") shouldBe true
        claims.hasRole("guest") shouldBe false
    }

    @Test
    fun `Claims hasPermission returns true when permission exists`() {
        val claims = Claims(
            subject = "user-1",
            permissions = listOf("read:users", "write:users"),
        )

        claims.hasPermission("read:users") shouldBe true
        claims.hasPermission("delete:users") shouldBe false
    }

    @Test
    fun `PolicyEnforcer requireRole throws on missing role`() {
        val claims = Claims(subject = "user-1", roles = listOf("user"))

        shouldThrow<ForbiddenException> {
            PolicyEnforcer.requireRole(claims, "admin")
        }
    }

    @Test
    fun `PolicyEnforcer requireAnyRole passes when one role matches`() {
        val claims = Claims(subject = "user-1", roles = listOf("editor"))

        PolicyEnforcer.requireAnyRole(claims, "admin", "editor")
    }
}
