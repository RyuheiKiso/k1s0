package dev.k1s0.consensus

import io.github.oshai.kotlinlogging.KotlinLogging
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.delay
import org.jetbrains.exposed.sql.Database
import org.jetbrains.exposed.sql.transactions.experimental.newSuspendedTransaction
import java.time.Instant

private val logger = KotlinLogging.logger {}

/**
 * PostgreSQL-based distributed lock using Exposed.
 *
 * Uses INSERT ON CONFLICT for atomic lock acquisition. Fence tokens are
 * auto-incremented on each acquisition to enable safe fencing.
 *
 * @property lockConfig Lock configuration.
 * @property database The Exposed database instance.
 * @property metrics Optional lock metrics.
 */
public class DbDistributedLock(
    private val lockConfig: LockConfig = LockConfig(),
    private val database: Database,
    private val metrics: LockMetrics? = null,
) : DistributedLock {

    override suspend fun tryLock(lockName: String, ownerId: String, ttlMs: Long): LockGuard? =
        newSuspendedTransaction(Dispatchers.IO, database) {
            val now = Instant.now()
            val expiresAt = now.plusMillis(ttlMs)

            val sql = """
                INSERT INTO ${lockConfig.tableName} (lock_name, owner_id, fence_token, acquired_at, expires_at)
                VALUES (?, ?, 1, ?, ?)
                ON CONFLICT (lock_name) DO UPDATE
                SET owner_id = EXCLUDED.owner_id,
                    fence_token = ${lockConfig.tableName}.fence_token + 1,
                    acquired_at = EXCLUDED.acquired_at,
                    expires_at = EXCLUDED.expires_at
                WHERE ${lockConfig.tableName}.expires_at < ?
                   OR ${lockConfig.tableName}.owner_id = EXCLUDED.owner_id
                RETURNING lock_name, owner_id, fence_token, acquired_at, expires_at
            """.trimIndent()

            val result = execLock(sql, listOf(lockName, ownerId, now.toString(), expiresAt.toString(), now.toString()))

            if (result != null && result.ownerId == ownerId) {
                metrics?.lockAcquired(lockName)
                logger.debug { "Acquired lock '$lockName' (owner=$ownerId, fence=${result.fenceToken})" }
                result.toLockGuard { unlock(result) }
            } else {
                logger.debug { "Failed to acquire lock '$lockName' (owner=$ownerId)" }
                null
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
        throw ConsensusError.LockTimeout("Lock '$lockName' acquisition timed out after ${timeoutMs}ms")
    }

    override suspend fun extend(guard: LockGuard, ttlMs: Long): LockGuard? =
        newSuspendedTransaction(Dispatchers.IO, database) {
            val now = Instant.now()
            val newExpiresAt = now.plusMillis(ttlMs)

            val sql = """
                UPDATE ${lockConfig.tableName}
                SET expires_at = ?, fence_token = fence_token + 1
                WHERE lock_name = ?
                  AND owner_id = ?
                  AND fence_token = ?
                  AND expires_at > ?
                RETURNING lock_name, owner_id, fence_token, acquired_at, expires_at
            """.trimIndent()

            val result = execLock(sql, listOf(
                newExpiresAt.toString(),
                guard.lockName,
                guard.ownerId,
                guard.fenceToken,
                now.toString(),
            ))

            if (result != null) {
                metrics?.lockExtended(guard.lockName)
                logger.debug { "Extended lock '${guard.lockName}' (fence=${result.fenceToken})" }
                result.toLockGuard { unlock(result) }
            } else {
                logger.warn { "Failed to extend lock '${guard.lockName}' — lost or expired" }
                null
            }
        }

    override suspend fun unlock(guard: LockGuard): Unit =
        newSuspendedTransaction(Dispatchers.IO, database) {
            val sql = """
                DELETE FROM ${lockConfig.tableName}
                WHERE lock_name = ?
                  AND owner_id = ?
                  AND fence_token = ?
            """.trimIndent()

            val conn = org.jetbrains.exposed.sql.transactions.TransactionManager.current().connection
            val stmt = conn.prepareStatement(sql, false)
            stmt[1] = guard.lockName
            stmt[2] = guard.ownerId
            stmt[3] = guard.fenceToken
            stmt.executeUpdate()

            metrics?.lockReleased(guard.lockName)
            logger.debug { "Released lock '${guard.lockName}' (owner=${guard.ownerId})" }
        }

    private data class LockRow(
        val lockName: String,
        val ownerId: String,
        val fenceToken: Long,
        val acquiredAt: Instant,
        val expiresAt: Instant,
    ) {
        fun toLockGuard(releaser: suspend () -> Unit): LockGuard = LockGuard(
            lockName = lockName,
            ownerId = ownerId,
            fenceToken = fenceToken,
            acquiredAt = acquiredAt,
            expiresAt = expiresAt,
            releaser = releaser,
        )
    }

    private fun execLock(sql: String, args: List<Any?>): LockRow? {
        val conn = org.jetbrains.exposed.sql.transactions.TransactionManager.current().connection
        val stmt = conn.prepareStatement(sql, false)
        args.forEachIndexed { idx, arg ->
            stmt[idx + 1] = arg
        }
        val rs = stmt.executeQuery().resultSet
        return if (rs.next()) {
            LockRow(
                lockName = rs.getString("lock_name"),
                ownerId = rs.getString("owner_id"),
                fenceToken = rs.getLong("fence_token"),
                acquiredAt = Instant.parse(rs.getString("acquired_at")),
                expiresAt = Instant.parse(rs.getString("expires_at")),
            )
        } else {
            null
        }
    }
}
