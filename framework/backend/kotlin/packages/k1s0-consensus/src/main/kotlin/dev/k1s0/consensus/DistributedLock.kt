package dev.k1s0.consensus

import java.io.Closeable
import java.time.Instant

/**
 * A guard representing a held distributed lock.
 *
 * Implements [Closeable] so the lock can be released with a `use` block.
 *
 * @property lockName The name of the lock resource.
 * @property ownerId The ID of the owner that holds the lock.
 * @property fenceToken Monotonically increasing token for fencing.
 * @property acquiredAt When the lock was acquired.
 * @property expiresAt When the lock expires.
 * @property releaser Function to release the lock.
 */
public data class LockGuard(
    val lockName: String,
    val ownerId: String,
    val fenceToken: Long,
    val acquiredAt: Instant,
    val expiresAt: Instant,
    private val releaser: suspend () -> Unit,
) : Closeable {

    /** Releases the lock synchronously. For suspend usage, call [suspendClose]. */
    override fun close() {
        // Non-suspend close is a best-effort; prefer suspendClose in coroutine contexts
    }

    /** Releases the lock in a suspending context. */
    public suspend fun suspendClose() {
        releaser()
    }
}

/**
 * Interface for distributed lock implementations.
 *
 * Distributed locks ensure mutual exclusion across multiple nodes. All
 * implementations must provide fence tokens for safe resource access.
 */
public interface DistributedLock {

    /**
     * Attempts to acquire the lock without waiting.
     *
     * @param lockName The name of the lock resource.
     * @param ownerId The unique ID of the caller.
     * @param ttlMs Time-to-live for the lock in milliseconds.
     * @return A [LockGuard] if acquired, or null if the lock is held by another owner.
     */
    public suspend fun tryLock(lockName: String, ownerId: String, ttlMs: Long): LockGuard?

    /**
     * Acquires the lock, waiting up to [timeoutMs] milliseconds.
     *
     * @param lockName The name of the lock resource.
     * @param ownerId The unique ID of the caller.
     * @param ttlMs Time-to-live for the lock in milliseconds.
     * @param timeoutMs Maximum time to wait for lock acquisition.
     * @return A [LockGuard] if acquired.
     * @throws ConsensusError.LockTimeout if the lock could not be acquired within the timeout.
     */
    public suspend fun lock(lockName: String, ownerId: String, ttlMs: Long, timeoutMs: Long): LockGuard

    /**
     * Extends the TTL of a held lock.
     *
     * @param guard The current lock guard.
     * @param ttlMs New TTL in milliseconds.
     * @return An updated [LockGuard], or null if the lock was lost.
     */
    public suspend fun extend(guard: LockGuard, ttlMs: Long): LockGuard?

    /**
     * Releases a held lock.
     *
     * @param guard The lock guard to release.
     */
    public suspend fun unlock(guard: LockGuard)
}
