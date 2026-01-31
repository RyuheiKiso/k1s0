package dev.k1s0.cache

import io.github.oshai.kotlinlogging.KotlinLogging
import kotlin.time.Duration

private val logger = KotlinLogging.logger {}

/**
 * Cache-aside (lazy-loading) pattern implementation.
 *
 * Checks the cache first; on a miss, loads from the source and populates the cache.
 *
 * @property operations The cache operations to use.
 * @property ttl The TTL for cached entries.
 */
public class CacheAside(
    private val operations: CacheOperations,
    private val ttl: Duration,
) {
    /**
     * Gets a value from cache, or loads it from the source on a cache miss.
     *
     * @param key The cache key.
     * @param loader The function to call on a cache miss.
     * @return The cached or freshly loaded value.
     */
    public suspend fun getOrLoad(key: String, loader: suspend () -> String): String {
        val cached = operations.get(key)
        if (cached != null) {
            logger.debug { "Cache hit: $key" }
            return cached
        }

        logger.debug { "Cache miss: $key" }
        val value = loader()
        operations.set(key, value, ttl)
        return value
    }

    /**
     * Invalidates a cache entry.
     *
     * @param key The cache key to invalidate.
     */
    public suspend fun invalidate(key: String) {
        operations.delete(key)
        logger.debug { "Cache invalidated: $key" }
    }
}
