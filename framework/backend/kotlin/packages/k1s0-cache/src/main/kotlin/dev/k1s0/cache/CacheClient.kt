package dev.k1s0.cache

import io.github.oshai.kotlinlogging.KotlinLogging
import io.lettuce.core.RedisClient
import io.lettuce.core.api.StatefulRedisConnection
import io.lettuce.core.api.coroutines
import io.lettuce.core.api.coroutines.RedisCoroutinesCommands

private val logger = KotlinLogging.logger {}

/**
 * Redis cache client using Lettuce with Kotlin coroutine support.
 *
 * @property redisUri The Redis connection URI (e.g., "redis://localhost:6379").
 */
public class CacheClient(private val redisUri: String) {

    private var client: RedisClient? = null
    private var connection: StatefulRedisConnection<String, String>? = null

    /**
     * Connects to the Redis server.
     *
     * @return Coroutine-friendly Redis commands interface.
     */
    public fun connect(): RedisCoroutinesCommands<String, String> {
        client = RedisClient.create(redisUri)
        connection = client!!.connect()
        logger.info { "Connected to Redis at $redisUri" }
        return connection!!.coroutines()
    }

    /** Closes the Redis connection and client. */
    public fun close() {
        connection?.close()
        client?.shutdown()
        logger.info { "Redis connection closed" }
    }
}
