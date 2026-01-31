package dev.k1s0.ratelimit

import io.github.oshai.kotlinlogging.KotlinLogging
import io.ktor.http.*
import io.ktor.server.application.*
import io.ktor.server.response.*

private val logger = KotlinLogging.logger {}

/**
 * Ktor application plugin for rate limiting.
 *
 * Applies a [RateLimiter] to all incoming calls. Requests exceeding the
 * rate limit receive a 429 Too Many Requests response.
 *
 * Usage:
 * ```kotlin
 * install(RateLimitPlugin) {
 *     limiter = TokenBucket(TokenBucketConfig(capacity = 100, refillRate = 10.0))
 * }
 * ```
 */
public val RateLimitPlugin: ApplicationPlugin<RateLimitPluginConfig> =
    createApplicationPlugin("RateLimit", ::RateLimitPluginConfig) {
        val limiter = pluginConfig.limiter ?: TokenBucket()

        onCall { call ->
            if (!limiter.tryAcquire()) {
                val stats = limiter.stats()
                logger.warn { "Rate limit exceeded (allowed=${stats.allowed}, rejected=${stats.rejected})" }
                call.response.header(
                    HttpHeaders.RetryAfter,
                    limiter.timeUntilAvailable().inWholeSeconds.toString(),
                )
                call.respond(HttpStatusCode.TooManyRequests, "Rate limit exceeded")
            }
        }
    }

/** Configuration for [RateLimitPlugin]. */
public class RateLimitPluginConfig {
    /** The rate limiter to use. Defaults to [TokenBucket] with default settings. */
    public var limiter: RateLimiter? = null
}
