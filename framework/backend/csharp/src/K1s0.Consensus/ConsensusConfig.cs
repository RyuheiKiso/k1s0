namespace K1s0.Consensus;

/// <summary>
/// Root configuration for all consensus features.
/// Loaded from the <c>consensus</c> section of the YAML configuration file.
/// </summary>
public sealed class ConsensusConfig
{
    /// <summary>
    /// Leader election configuration.
    /// </summary>
    public LeaderConfig Leader { get; set; } = new();

    /// <summary>
    /// Distributed lock configuration.
    /// </summary>
    public LockConfig Lock { get; set; } = new();

    /// <summary>
    /// Saga orchestration configuration.
    /// </summary>
    public SagaConfig Saga { get; set; } = new();
}

/// <summary>
/// Configuration for leader election.
/// </summary>
public sealed class LeaderConfig
{
    /// <summary>
    /// Duration of the leader lease. Default is 15 seconds.
    /// </summary>
    public TimeSpan LeaseDuration { get; set; } = TimeSpan.FromSeconds(15);

    /// <summary>
    /// Interval between heartbeat renewals. Default is 5 seconds.
    /// Should be significantly less than <see cref="LeaseDuration"/>.
    /// </summary>
    public TimeSpan RenewInterval { get; set; } = TimeSpan.FromSeconds(5);

    /// <summary>
    /// Grace period after lease expiration before another node can acquire.
    /// Default is 2 seconds.
    /// </summary>
    public TimeSpan GracePeriod { get; set; } = TimeSpan.FromSeconds(2);

    /// <summary>
    /// PostgreSQL connection string file path (secret reference).
    /// </summary>
    public string ConnectionStringFile { get; set; } = string.Empty;

    /// <summary>
    /// Table name for leader leases. Default is <c>consensus_leader_lease</c>.
    /// </summary>
    public string TableName { get; set; } = "consensus_leader_lease";
}

/// <summary>
/// Configuration for distributed locks.
/// </summary>
public sealed class LockConfig
{
    /// <summary>
    /// Default lock expiration. Default is 30 seconds.
    /// </summary>
    public TimeSpan DefaultExpiration { get; set; } = TimeSpan.FromSeconds(30);

    /// <summary>
    /// Default timeout when waiting to acquire a lock. Default is 10 seconds.
    /// </summary>
    public TimeSpan DefaultWaitTimeout { get; set; } = TimeSpan.FromSeconds(10);

    /// <summary>
    /// Retry interval when polling for lock acquisition. Default is 100 milliseconds.
    /// </summary>
    public TimeSpan RetryInterval { get; set; } = TimeSpan.FromMilliseconds(100);

    /// <summary>
    /// Backend type: <c>postgres</c> or <c>redis</c>. Default is <c>postgres</c>.
    /// </summary>
    public string Backend { get; set; } = "postgres";

    /// <summary>
    /// PostgreSQL connection string file path (secret reference).
    /// </summary>
    public string ConnectionStringFile { get; set; } = string.Empty;

    /// <summary>
    /// Redis connection string file path (secret reference).
    /// </summary>
    public string RedisConnectionStringFile { get; set; } = string.Empty;

    /// <summary>
    /// Table name for PostgreSQL-backed locks. Default is <c>consensus_distributed_lock</c>.
    /// </summary>
    public string TableName { get; set; } = "consensus_distributed_lock";
}

/// <summary>
/// Configuration for saga orchestration.
/// </summary>
public sealed class SagaConfig
{
    /// <summary>
    /// PostgreSQL connection string file path (secret reference).
    /// </summary>
    public string ConnectionStringFile { get; set; } = string.Empty;

    /// <summary>
    /// Table name for saga instances. Default is <c>consensus_saga_instance</c>.
    /// </summary>
    public string TableName { get; set; } = "consensus_saga_instance";

    /// <summary>
    /// Table name for saga step log. Default is <c>consensus_saga_step_log</c>.
    /// </summary>
    public string StepLogTableName { get; set; } = "consensus_saga_step_log";

    /// <summary>
    /// Table name for dead letter entries. Default is <c>consensus_saga_dead_letter</c>.
    /// </summary>
    public string DeadLetterTableName { get; set; } = "consensus_saga_dead_letter";

    /// <summary>
    /// Default step timeout. Default is 30 seconds.
    /// </summary>
    public TimeSpan DefaultStepTimeout { get; set; } = TimeSpan.FromSeconds(30);

    /// <summary>
    /// Maximum number of retry attempts per step. Default is 3.
    /// </summary>
    public int MaxRetries { get; set; } = 3;

    /// <summary>
    /// Choreography saga timeout monitoring interval. Default is 5 seconds.
    /// </summary>
    public TimeSpan TimeoutMonitorInterval { get; set; } = TimeSpan.FromSeconds(5);
}
