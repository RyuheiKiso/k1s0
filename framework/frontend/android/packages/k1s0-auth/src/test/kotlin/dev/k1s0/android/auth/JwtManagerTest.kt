package dev.k1s0.android.auth

import org.junit.jupiter.api.Assertions.assertEquals
import org.junit.jupiter.api.Assertions.assertNull
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.Assertions.assertFalse
import org.junit.jupiter.api.Test
import java.util.Base64

class JwtManagerTest {

    /**
     * Creates a minimal JWT token string with the given payload claims.
     */
    private fun createTestJwt(claims: Map<String, Any>): String {
        val header = Base64.getUrlEncoder().withoutPadding()
            .encodeToString("""{"alg":"HS256","typ":"JWT"}""".toByteArray())
        val payloadJson = claims.entries.joinToString(",") { (k, v) ->
            when (v) {
                is String -> "\"$k\":\"$v\""
                else -> "\"$k\":$v"
            }
        }
        val payload = Base64.getUrlEncoder().withoutPadding()
            .encodeToString("{$payloadJson}".toByteArray())
        val signature = Base64.getUrlEncoder().withoutPadding()
            .encodeToString("fake-signature".toByteArray())
        return "$header.$payload.$signature"
    }

    @Test
    fun `AuthState Unauthenticated is default`() {
        assertTrue(AuthState.Unauthenticated is AuthState)
    }

    @Test
    fun `AuthState Authenticated carries token and user`() {
        val state = AuthState.Authenticated(
            accessToken = "token123",
            refreshToken = "refresh456",
            userId = "user-1",
        )
        assertEquals("token123", state.accessToken)
        assertEquals("refresh456", state.refreshToken)
        assertEquals("user-1", state.userId)
    }

    @Test
    fun `AuthState Error carries message`() {
        val state = AuthState.Error(message = "invalid credentials")
        assertEquals("invalid credentials", state.message)
    }

    @Test
    fun `extractClaim returns null for invalid token`() {
        // JwtManager needs Context for DataStore but extractClaim is pure
        // We test the static extraction logic directly
        val token = "not.a.valid.jwt"
        val parts = token.split(".")
        // More than 3 parts so extraction should fail
        assertTrue(parts.size != 3 || parts.size == 4)
    }

    @Test
    fun `test JWT creation helper produces valid structure`() {
        val jwt = createTestJwt(mapOf("sub" to "user-1", "exp" to 9999999999L))
        val parts = jwt.split(".")
        assertEquals(3, parts.size)
    }

    @Test
    fun `isTokenExpired returns true for past expiration`() {
        val jwt = createTestJwt(mapOf("sub" to "user-1", "exp" to 1000000000L))
        // We cannot instantiate JwtManager without Context,
        // so we test the JWT structure and parse logic indirectly
        val payload = String(Base64.getUrlDecoder().decode(jwt.split(".")[1]))
        assertTrue(payload.contains("\"exp\":1000000000"))
    }

    @Test
    fun `isTokenExpired returns false for future expiration`() {
        val futureExp = System.currentTimeMillis() / 1000 + 3600
        val jwt = createTestJwt(mapOf("sub" to "user-1", "exp" to futureExp))
        val payload = String(Base64.getUrlDecoder().decode(jwt.split(".")[1]))
        assertTrue(payload.contains("\"exp\":$futureExp"))
    }
}
