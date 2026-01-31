package dev.k1s0.auth

import com.nimbusds.jose.JWSAlgorithm
import com.nimbusds.jose.jwk.source.JWKSourceBuilder
import com.nimbusds.jose.proc.DefaultJOSEObjectTypeVerifier
import com.nimbusds.jose.proc.JWSKeySelector
import com.nimbusds.jose.proc.JWSVerificationKeySelector
import com.nimbusds.jose.proc.SecurityContext
import com.nimbusds.jwt.JWTClaimsSet
import com.nimbusds.jwt.proc.DefaultJWTClaimsVerifier
import com.nimbusds.jwt.proc.DefaultJWTProcessor
import dev.k1s0.error.UnauthorizedException
import io.github.oshai.kotlinlogging.KotlinLogging
import java.net.URI
import java.net.URL

private val logger = KotlinLogging.logger {}

/**
 * JWT token verifier using nimbus-jose-jwt.
 *
 * Supports RS256 tokens validated against a JWKS endpoint.
 *
 * @property jwksUrl The URL of the JWKS endpoint.
 * @property issuer Expected issuer claim. If null, issuer is not validated.
 * @property audience Expected audience claim. If null, audience is not validated.
 */
public class JwtVerifier(
    private val jwksUrl: String,
    private val issuer: String? = null,
    private val audience: String? = null,
) {
    private val processor = DefaultJWTProcessor<SecurityContext>().apply {
        jwsTypeVerifier = DefaultJOSEObjectTypeVerifier.JWT
        val keySource = JWKSourceBuilder.create<SecurityContext>(URI(jwksUrl).toURL()).build()
        jwsKeySelector = JWSVerificationKeySelector(JWSAlgorithm.RS256, keySource)

        jwtClaimsSetVerifier = DefaultJWTClaimsVerifier<SecurityContext>(
            JWTClaimsSet.Builder().apply {
                issuer?.let { issuer(it) }
                audience?.let { audience(it) }
            }.build(),
            setOf("sub", "iat", "exp"),
        )
    }

    /**
     * Verifies a JWT token string and extracts claims.
     *
     * @param token The raw JWT token string.
     * @return The extracted [Claims].
     * @throws UnauthorizedException if the token is invalid.
     */
    public fun verify(token: String): Claims {
        return try {
            val claimsSet = processor.process(token, null)
            Claims(
                subject = claimsSet.subject,
                issuer = claimsSet.issuer,
                roles = claimsSet.getStringListClaim("roles") ?: emptyList(),
                permissions = claimsSet.getStringListClaim("permissions") ?: emptyList(),
            )
        } catch (e: Exception) {
            logger.warn { "JWT verification failed: ${e.message}" }
            throw UnauthorizedException(
                serviceErrorCode = "auth.invalid_token",
                detail = "Invalid or expired JWT token",
                cause = e,
            )
        }
    }
}
