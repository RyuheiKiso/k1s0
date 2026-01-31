package dev.k1s0.auth

import dev.k1s0.error.UnauthorizedException
import io.github.oshai.kotlinlogging.KotlinLogging
import io.ktor.server.application.ApplicationCall
import io.ktor.server.application.createApplicationPlugin
import io.ktor.server.request.header

private val logger = KotlinLogging.logger {}

/** Key for storing [Claims] in the Ktor call attributes. */
public val ClaimsKey: io.ktor.util.AttributeKey<Claims> = io.ktor.util.AttributeKey("k1s0.claims")

/**
 * Ktor plugin that verifies JWT tokens from the Authorization header.
 *
 * On successful verification, the [Claims] are stored in the call attributes
 * and can be retrieved using [ApplicationCall.claims].
 */
public val K1s0Auth = createApplicationPlugin("K1s0Auth", ::K1s0AuthConfig) {
    val verifier = JwtVerifier(
        jwksUrl = pluginConfig.jwksUrl,
        issuer = pluginConfig.issuer,
        audience = pluginConfig.audience,
    )

    onCall { call ->
        val authHeader = call.request.header("Authorization")
        if (authHeader != null && authHeader.startsWith("Bearer ")) {
            val token = authHeader.removePrefix("Bearer ")
            val claims = verifier.verify(token)
            call.attributes.put(ClaimsKey, claims)
            logger.debug { "Authenticated user: ${claims.subject}" }
        } else if (pluginConfig.requireAuth) {
            throw UnauthorizedException(
                serviceErrorCode = "auth.missing_token",
                detail = "Authorization header is missing or invalid",
            )
        }
    }
}

/** Configuration for the [K1s0Auth] plugin. */
public class K1s0AuthConfig {
    /** The JWKS endpoint URL. */
    public var jwksUrl: String = ""

    /** Expected issuer claim. */
    public var issuer: String? = null

    /** Expected audience claim. */
    public var audience: String? = null

    /** Whether authentication is required for all requests. */
    public var requireAuth: Boolean = true
}

/** Extension to retrieve [Claims] from a Ktor call. */
public val ApplicationCall.claims: Claims?
    get() = attributes.getOrNull(ClaimsKey)
