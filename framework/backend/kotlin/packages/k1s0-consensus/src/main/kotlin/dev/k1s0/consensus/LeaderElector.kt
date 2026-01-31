package dev.k1s0.consensus

import kotlinx.coroutines.flow.Flow
import java.time.Instant

/**
 * Leader lease held by a node.
 *
 * @property leaseId Unique identifier for this lease.
 * @property nodeId The node that holds the lease.
 * @property acquiredAt When the lease was acquired.
 * @property expiresAt When the lease expires.
 * @property fenceToken Monotonically increasing token for fencing.
 */
public data class LeaderLease(
    val leaseId: String,
    val nodeId: String,
    val acquiredAt: Instant,
    val expiresAt: Instant,
    val fenceToken: Long,
)

/**
 * Configuration for a leader election instance.
 *
 * @property electionName Logical name for the election (e.g., "scheduler").
 * @property nodeId Unique identifier for this node.
 * @property leaseDurationMs Duration of the lease in milliseconds.
 * @property renewIntervalMs Interval between lease renewal attempts in milliseconds.
 */
public data class LeaderConfig(
    val electionName: String,
    val nodeId: String,
    val leaseDurationMs: Long = 15_000L,
    val renewIntervalMs: Long = 5_000L,
)

/**
 * Events emitted by leader election watch.
 */
public sealed class LeaderEvent {
    /** This node has been elected as the leader. */
    public data class Elected(val lease: LeaderLease) : LeaderEvent()

    /** This node has lost leadership. */
    public data class Lost(val previousLease: LeaderLease) : LeaderEvent()

    /** The leader has changed to a different node. */
    public data class Changed(val newLeader: String, val fenceToken: Long) : LeaderEvent()
}

/**
 * Interface for leader election using distributed consensus.
 *
 * Implementations must ensure that at most one node holds the leader lease
 * at any given time.
 */
public interface LeaderElector {

    /**
     * Attempts to acquire the leader lease.
     *
     * @return The lease if successfully acquired, or null if another node is the leader.
     */
    public suspend fun tryAcquire(): LeaderLease?

    /**
     * Renews the current leader lease.
     *
     * @param lease The current lease to renew.
     * @return The renewed lease, or null if the lease could not be renewed.
     */
    public suspend fun renew(lease: LeaderLease): LeaderLease?

    /**
     * Releases the leader lease.
     *
     * @param lease The lease to release.
     */
    public suspend fun release(lease: LeaderLease)

    /**
     * Retrieves the current leader information.
     *
     * @return The current leader lease, or null if there is no leader.
     */
    public suspend fun currentLeader(): LeaderLease?

    /**
     * Watches for leader election events.
     *
     * @return A [Flow] of [LeaderEvent] instances.
     */
    public fun watch(): Flow<LeaderEvent>
}
