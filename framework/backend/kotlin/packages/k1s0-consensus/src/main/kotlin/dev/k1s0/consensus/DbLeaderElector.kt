package dev.k1s0.consensus

import io.github.oshai.kotlinlogging.KotlinLogging
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.Job
import kotlinx.coroutines.channels.awaitClose
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.channelFlow
import kotlinx.coroutines.isActive
import kotlinx.coroutines.launch
import org.jetbrains.exposed.sql.Database
import org.jetbrains.exposed.sql.transactions.experimental.newSuspendedTransaction
import java.time.Instant
import java.util.UUID

private val logger = KotlinLogging.logger {}

/**
 * PostgreSQL-based leader election using Exposed.
 *
 * Uses INSERT ON CONFLICT for atomic lease acquisition and UPDATE with
 * fence token comparison for safe renewal. A background heartbeat coroutine
 * automatically renews the lease while the node is the leader.
 *
 * @property config Leader election configuration.
 * @property electionConfig Global election settings.
 * @property database The Exposed database instance.
 */
public class DbLeaderElector(
    private val config: LeaderConfig,
    private val electionConfig: LeaderElectionConfig = LeaderElectionConfig(),
    private val database: Database,
    private val metrics: LeaderMetrics? = null,
) : LeaderElector {

    private val scope = CoroutineScope(Dispatchers.IO)
    private var heartbeatJob: Job? = null
    private var currentLease: LeaderLease? = null

    override suspend fun tryAcquire(): LeaderLease? = newSuspendedTransaction(Dispatchers.IO, database) {
        val now = Instant.now()
        val expiresAt = now.plusMillis(config.leaseDurationMs)
        val leaseId = UUID.randomUUID().toString()

        // INSERT ON CONFLICT: atomically try to acquire or take over expired lease
        val sql = """
            INSERT INTO ${electionConfig.tableName} (election_name, lease_id, node_id, acquired_at, expires_at, fence_token)
            VALUES (?, ?, ?, ?, ?, 1)
            ON CONFLICT (election_name) DO UPDATE
            SET lease_id = EXCLUDED.lease_id,
                node_id = EXCLUDED.node_id,
                acquired_at = EXCLUDED.acquired_at,
                expires_at = EXCLUDED.expires_at,
                fence_token = ${electionConfig.tableName}.fence_token + 1
            WHERE ${electionConfig.tableName}.expires_at < ?
               OR ${electionConfig.tableName}.node_id = EXCLUDED.node_id
            RETURNING lease_id, node_id, acquired_at, expires_at, fence_token
        """.trimIndent()

        val result = exec(sql, listOf(
            config.electionName,
            leaseId,
            config.nodeId,
            now.toString(),
            expiresAt.toString(),
            now.toString(),
        )) { rs ->
            if (rs.next()) {
                LeaderLease(
                    leaseId = rs.getString("lease_id"),
                    nodeId = rs.getString("node_id"),
                    acquiredAt = Instant.parse(rs.getString("acquired_at")),
                    expiresAt = Instant.parse(rs.getString("expires_at")),
                    fenceToken = rs.getLong("fence_token"),
                )
            } else {
                null
            }
        }

        if (result != null && result.nodeId == config.nodeId) {
            currentLease = result
            startHeartbeat(result)
            metrics?.leaderElected()
            logger.info { "Acquired leader lease for '${config.electionName}' (node=${config.nodeId}, fence=${result.fenceToken})" }
            result
        } else {
            logger.debug { "Failed to acquire leader lease for '${config.electionName}'" }
            null
        }
    }

    override suspend fun renew(lease: LeaderLease): LeaderLease? = newSuspendedTransaction(Dispatchers.IO, database) {
        val now = Instant.now()
        val newExpiresAt = now.plusMillis(config.leaseDurationMs)

        val sql = """
            UPDATE ${electionConfig.tableName}
            SET expires_at = ?, fence_token = fence_token + 1
            WHERE election_name = ?
              AND node_id = ?
              AND lease_id = ?
              AND expires_at > ?
            RETURNING lease_id, node_id, acquired_at, expires_at, fence_token
        """.trimIndent()

        val result = exec(sql, listOf(
            newExpiresAt.toString(),
            config.electionName,
            config.nodeId,
            lease.leaseId,
            now.toString(),
        )) { rs ->
            if (rs.next()) {
                LeaderLease(
                    leaseId = rs.getString("lease_id"),
                    nodeId = rs.getString("node_id"),
                    acquiredAt = Instant.parse(rs.getString("acquired_at")),
                    expiresAt = Instant.parse(rs.getString("expires_at")),
                    fenceToken = rs.getLong("fence_token"),
                )
            } else {
                null
            }
        }

        if (result != null) {
            currentLease = result
            metrics?.leaseRenewed()
            logger.debug { "Renewed leader lease for '${config.electionName}' (fence=${result.fenceToken})" }
        } else {
            currentLease = null
            stopHeartbeat()
            metrics?.leaderLost()
            logger.warn { "Failed to renew leader lease for '${config.electionName}' — lease expired or taken" }
        }

        result
    }

    override suspend fun release(lease: LeaderLease): Unit = newSuspendedTransaction(Dispatchers.IO, database) {
        val sql = """
            DELETE FROM ${electionConfig.tableName}
            WHERE election_name = ?
              AND node_id = ?
              AND lease_id = ?
        """.trimIndent()

        exec(sql, listOf(config.electionName, config.nodeId, lease.leaseId))
        currentLease = null
        stopHeartbeat()
        metrics?.leaderReleased()
        logger.info { "Released leader lease for '${config.electionName}'" }
    }

    override suspend fun currentLeader(): LeaderLease? = newSuspendedTransaction(Dispatchers.IO, database) {
        val now = Instant.now()

        val sql = """
            SELECT lease_id, node_id, acquired_at, expires_at, fence_token
            FROM ${electionConfig.tableName}
            WHERE election_name = ?
              AND expires_at > ?
        """.trimIndent()

        exec(sql, listOf(config.electionName, now.toString())) { rs ->
            if (rs.next()) {
                LeaderLease(
                    leaseId = rs.getString("lease_id"),
                    nodeId = rs.getString("node_id"),
                    acquiredAt = Instant.parse(rs.getString("acquired_at")),
                    expiresAt = Instant.parse(rs.getString("expires_at")),
                    fenceToken = rs.getLong("fence_token"),
                )
            } else {
                null
            }
        }
    }

    override fun watch(): Flow<LeaderEvent> = channelFlow {
        var lastKnownLeader: String? = null
        var lastFenceToken: Long = 0L

        while (isActive) {
            val leader = currentLeader()

            when {
                // New leader elected and it is this node
                leader != null && leader.nodeId == config.nodeId && lastKnownLeader != config.nodeId -> {
                    send(LeaderEvent.Elected(leader))
                }
                // This node lost leadership
                leader?.nodeId != config.nodeId && lastKnownLeader == config.nodeId && currentLease != null -> {
                    send(LeaderEvent.Lost(currentLease!!))
                }
                // Leader changed to a different node
                leader != null && leader.nodeId != lastKnownLeader && lastKnownLeader != null -> {
                    send(LeaderEvent.Changed(leader.nodeId, leader.fenceToken))
                }
            }

            lastKnownLeader = leader?.nodeId
            lastFenceToken = leader?.fenceToken ?: lastFenceToken
            delay(electionConfig.watchPollIntervalMs)
        }

        awaitClose()
    }

    private fun startHeartbeat(initialLease: LeaderLease) {
        stopHeartbeat()
        heartbeatJob = scope.launch {
            var lease = initialLease
            while (isActive) {
                delay(config.renewIntervalMs)
                val renewed = renew(lease)
                if (renewed != null) {
                    lease = renewed
                } else {
                    logger.warn { "Heartbeat failed — lost leadership for '${config.electionName}'" }
                    break
                }
            }
        }
    }

    private fun stopHeartbeat() {
        heartbeatJob?.cancel()
        heartbeatJob = null
    }

    private fun exec(sql: String, args: List<Any?>, extractor: ((java.sql.ResultSet) -> LeaderLease?)? = null): LeaderLease? {
        val conn = org.jetbrains.exposed.sql.transactions.TransactionManager.current().connection
        val stmt = conn.prepareStatement(sql, false)
        args.forEachIndexed { idx, arg ->
            stmt[idx + 1] = arg
        }
        return if (extractor != null) {
            val rs = stmt.executeQuery()
            extractor(rs.resultSet)
        } else {
            stmt.executeUpdate()
            null
        }
    }
}
