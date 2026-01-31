package dev.k1s0.consensus

import dev.k1s0.config.ConfigLoader
import kotlinx.serialization.Serializable

/**
 * Top-level consensus configuration loaded from YAML.
 *
 * @property leader Leader election settings.
 * @property lock Distributed lock settings.
 * @property saga Saga orchestration settings.
 */
@Serializable
public data class ConsensusConfig(
    val leader: LeaderElectionConfig = LeaderElectionConfig(),
    val lock: LockConfig = LockConfig(),
    val saga: SagaConfig = SagaConfig(),
) {
    public companion object {
        /**
         * Loads consensus configuration from YAML files.
         *
         * @param env The environment name (default, dev, stg, prod).
         * @param configDir The directory containing config files.
         */
        public fun load(env: String = "default", configDir: String = "config"): ConsensusConfig =
            ConfigLoader.load(serializer(), env, configDir)
    }
}

/**
 * Leader election configuration.
 *
 * @property leaseDurationMs Duration of a leader lease in milliseconds.
 * @property renewIntervalMs How often to renew the lease in milliseconds.
 * @property watchPollIntervalMs Polling interval for leader watch in milliseconds.
 * @property tableName The database table name for leader leases.
 */
@Serializable
public data class LeaderElectionConfig(
    val leaseDurationMs: Long = 15_000L,
    val renewIntervalMs: Long = 5_000L,
    val watchPollIntervalMs: Long = 2_000L,
    val tableName: String = "k1s0_leader_lease",
)

/**
 * Distributed lock configuration.
 *
 * @property defaultTimeoutMs Default lock acquisition timeout in milliseconds.
 * @property defaultTtlMs Default lock TTL in milliseconds.
 * @property retryIntervalMs Retry interval when lock is held by another node.
 * @property tableName The database table name for distributed locks.
 * @property redisPasswordFile Path to the file containing the Redis password.
 */
@Serializable
public data class LockConfig(
    val defaultTimeoutMs: Long = 10_000L,
    val defaultTtlMs: Long = 30_000L,
    val retryIntervalMs: Long = 100L,
    val tableName: String = "k1s0_distributed_lock",
    val redisPasswordFile: String? = null,
)

/**
 * Saga orchestration configuration.
 *
 * @property maxRetries Maximum retries before moving to dead letter.
 * @property defaultTimeoutMs Default saga step timeout in milliseconds.
 * @property deadLetterEnabled Whether to enable the dead letter queue.
 * @property tableName The database table name for saga instances.
 */
@Serializable
public data class SagaConfig(
    val maxRetries: Int = 3,
    val defaultTimeoutMs: Long = 30_000L,
    val deadLetterEnabled: Boolean = true,
    val tableName: String = "k1s0_saga_instance",
)
