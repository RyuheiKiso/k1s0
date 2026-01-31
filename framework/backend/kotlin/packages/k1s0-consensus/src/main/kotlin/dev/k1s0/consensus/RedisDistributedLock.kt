package dev.k1s0.consensus

import dev.k1s0.config.SecretResolver
import io.github.oshai.kotlinlogging.KotlinLogging
import io.lettuce.core.RedisClient
import io.lettuce.core.ScriptOutputType
import io.lettuce.core.SetArgs
import io.lettuce.core.api.StatefulRedisConnection
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.delay
import kotlinx.coroutines.withContext
import java.time.Instant
import java.util.concurrent.atomic.AtomicLong

private val logger = KotlinLogging.logger {}

/**
 * Redis-based distributed lock using Lettuce.
 *
 * Uses SET NX PX for atomic lock acquisition and Lua scripts for
 * safe unlock and extend operations.
 *
 * @property redisClient The Lettuce Redis client.
 * @property lockConfig Lock configuration.
 * @property metrics Optional lock metrics.
 */
public class RedisDistributedLock(
    private val redisClient: RedisClient,
    private val lockConfig: LockConfig = LockConfig(),
    private val metrics: LockMetrics? = null,
) : DistributedLock {

    private val fenceTokenCounter = AtomicLong(0)

    private fun connection(): StatefulRedisConnection<String, String> = redisClient.connect()

    override suspend fun tryLock(lockName: String, ownerId: String, ttlMs: Long): LockGuard? =
        withContext(Dispatchers.IO) {
            val conn = connection()
            try {
                val key = lockKey(lockName)
                val fenceToken = fenceTokenCounter.incrementAndGet()
                val value = "$ownerId:$fenceToken"

                val result = conn.sync().set(
                    key,
                    value,
                    SetArgs().nx().px(ttlMs),
                )

                if (result == "OK") {
                    val now = Instant.now()
                    metrics?.lockAcquired(lockName)
                    logger.debug { "Acquired Redis lock '$lockName' (owner=$ownerId, fence=$fenceToken)" }
                    LockGuard(
                        lockName = lockName,
                        ownerId = ownerId,
                        fenceToken = fenceToken,
                        acquiredAt = now,
                        expiresAt = now.plusMillis(ttlMs),
                        releaser = { unlock(lockName, ownerId, fenceToken) },
                    )
                } else {
                    logger.debug { "Failed to acquire Redis lock '$lockName'" }
                    null
                }
            } finally {
                conn.close()
            }
        }

    override suspend fun lock(lockName: String, ownerId: String, ttlMs: Long, timeoutMs: Long): LockGuard {
        val deadline = Instant.now().plusMillis(timeoutMs)

        while (Instant.now().isBefore(deadline)) {
            val guard = tryLock(lockName, ownerId, ttlMs)
            if (guard != null) return guard
            delay(lockConfig.retryIntervalMs)
        }

        metrics?.lockTimeout(lockName)
        throw ConsensusError.LockTimeout("Redis lock '$lockName' acquisition timed out after ${timeoutMs}ms")
    }

    override suspend fun extend(guard: LockGuard, ttlMs: Long): LockGuard? =
        withContext(Dispatchers.IO) {
            val conn = connection()
            try {
                val key = lockKey(guard.lockName)
                val expectedValue = "${guard.ownerId}:${guard.fenceToken}"

                // Lua script: extend only if current value matches
                val script = """
                    if redis.call("get", KEYS[1]) == ARGV[1] then
                        return redis.call("pexpire", KEYS[1], ARGV[2])
                    else
                        return 0
                    end
                """.trimIndent()

                val result = conn.sync().eval<Long>(
                    script,
                    ScriptOutputType.INTEGER,
                    arrayOf(key),
                    expectedValue,
                    ttlMs.toString(),
                )

                if (result == 1L) {
                    val now = Instant.now()
                    metrics?.lockExtended(guard.lockName)
                    logger.debug { "Extended Redis lock '${guard.lockName}' by ${ttlMs}ms" }
                    guard.copy(expiresAt = now.plusMillis(ttlMs))
                } else {
                    logger.warn { "Failed to extend Redis lock '${guard.lockName}' — not held" }
                    null
                }
            } finally {
                conn.close()
            }
        }

    override suspend fun unlock(guard: LockGuard) {
        unlock(guard.lockName, guard.ownerId, guard.fenceToken)
    }

    private suspend fun unlock(lockName: String, ownerId: String, fenceToken: Long): Unit =
        withContext(Dispatchers.IO) {
            val conn = connection()
            try {
                val key = lockKey(lockName)
                val expectedValue = "$ownerId:$fenceToken"

                // Lua script: delete only if current value matches
                val script = """
                    if redis.call("get", KEYS[1]) == ARGV[1] then
                        return redis.call("del", KEYS[1])
                    else
                        return 0
                    end
                """.trimIndent()

                conn.sync().eval<Long>(
                    script,
                    ScriptOutputType.INTEGER,
                    arrayOf(key),
                    expectedValue,
                )

                metrics?.lockReleased(lockName)
                logger.debug { "Released Redis lock '$lockName' (owner=$ownerId)" }
            } finally {
                conn.close()
            }
        }

    private fun lockKey(lockName: String): String = "k1s0:lock:$lockName"

    public companion object {
        /**
         * Creates a RedisDistributedLock from configuration.
         *
         * @param redisUri The Redis URI (e.g., "redis://localhost:6379").
         * @param lockConfig Lock configuration.
         * @param metrics Optional lock metrics.
         */
        public fun create(
            redisUri: String,
            lockConfig: LockConfig = LockConfig(),
            metrics: LockMetrics? = null,
        ): RedisDistributedLock {
            val uri = if (lockConfig.redisPasswordFile != null) {
                val password = SecretResolver.resolve(lockConfig.redisPasswordFile)
                redisUri.replaceFirst("://", "://:$password@")
            } else {
                redisUri
            }
            val client = RedisClient.create(uri)
            return RedisDistributedLock(client, lockConfig, metrics)
        }
    }
}
