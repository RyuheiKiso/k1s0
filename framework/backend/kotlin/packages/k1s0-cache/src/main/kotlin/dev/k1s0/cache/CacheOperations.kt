package dev.k1s0.cache

import io.lettuce.core.api.coroutines.RedisCoroutinesCommands
import kotlin.time.Duration

/**
 * High-level cache operations wrapping Redis commands.
 *
 * @property commands The underlying Redis coroutines commands.
 */
public class CacheOperations(
    private val commands: RedisCoroutinesCommands<String, String>,
) {
    /** Gets a value by key. */
    public suspend fun get(key: String): String? = commands.get(key)

    /** Sets a value with an optional TTL. */
    public suspend fun set(key: String, value: String, ttl: Duration? = null) {
        if (ttl != null) {
            commands.setex(key, ttl.inWholeSeconds, value)
        } else {
            commands.set(key, value)
        }
    }

    /** Deletes a key. */
    public suspend fun delete(key: String): Long? = commands.del(key)

    /** Checks if a key exists. */
    public suspend fun exists(key: String): Boolean = (commands.exists(key) ?: 0) > 0
}
