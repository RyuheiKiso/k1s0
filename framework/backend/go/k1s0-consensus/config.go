package consensus

import "time"

// ConsensusConfig holds all configuration for the consensus package.
type ConsensusConfig struct {
	// Leader holds leader election configuration.
	Leader LeaderConfig `yaml:"leader"`

	// Lock holds distributed lock configuration.
	Lock LockConfig `yaml:"lock"`

	// Saga holds saga orchestration configuration.
	Saga SagaConfig `yaml:"saga"`
}

// LeaderConfig holds leader election configuration.
type LeaderConfig struct {
	// NodeID is the unique identifier for this node.
	NodeID string `yaml:"node_id"`

	// LeaseDuration is how long a lease is valid before it expires.
	// Default: 15s.
	LeaseDuration time.Duration `yaml:"lease_duration"`

	// RenewInterval is how often the leader renews its lease.
	// Should be less than LeaseDuration. Default: 5s.
	RenewInterval time.Duration `yaml:"renew_interval"`

	// WatchPollInterval is the interval for polling leader changes.
	// Default: 2s.
	WatchPollInterval time.Duration `yaml:"watch_poll_interval"`

	// TableName is the PostgreSQL table name for leader election.
	// Default: "k1s0_leader_election".
	TableName string `yaml:"table_name"`
}

// LockConfig holds distributed lock configuration.
type LockConfig struct {
	// DefaultTTL is the default time-to-live for locks.
	// Default: 30s.
	DefaultTTL time.Duration `yaml:"default_ttl"`

	// RetryInterval is the interval between lock acquisition retries.
	// Default: 100ms.
	RetryInterval time.Duration `yaml:"retry_interval"`

	// TableName is the PostgreSQL table name for distributed locks.
	// Default: "k1s0_distributed_locks".
	TableName string `yaml:"table_name"`

	// KeyPrefix is the Redis key prefix for locks.
	// Default: "k1s0:lock:".
	KeyPrefix string `yaml:"key_prefix"`
}

// SagaConfig holds saga orchestration configuration.
type SagaConfig struct {
	// StepTimeout is the default timeout for each saga step.
	// Default: 30s.
	StepTimeout time.Duration `yaml:"step_timeout"`

	// MaxRetries is the default maximum number of retries per step.
	// Default: 3.
	MaxRetries int `yaml:"max_retries"`

	// TableName is the PostgreSQL table name for saga state.
	// Default: "k1s0_saga_instances".
	TableName string `yaml:"table_name"`

	// DeadLetterTableName is the PostgreSQL table name for dead letter sagas.
	// Default: "k1s0_saga_dead_letters".
	DeadLetterTableName string `yaml:"dead_letter_table_name"`
}

// DefaultConsensusConfig returns a ConsensusConfig with default values.
func DefaultConsensusConfig() ConsensusConfig {
	return ConsensusConfig{
		Leader: DefaultLeaderConfig(),
		Lock:   DefaultLockConfig(),
		Saga:   DefaultSagaConfig(),
	}
}

// DefaultLeaderConfig returns a LeaderConfig with default values.
func DefaultLeaderConfig() LeaderConfig {
	return LeaderConfig{
		LeaseDuration:     15 * time.Second,
		RenewInterval:     5 * time.Second,
		WatchPollInterval: 2 * time.Second,
		TableName:         "k1s0_leader_election",
	}
}

// DefaultLockConfig returns a LockConfig with default values.
func DefaultLockConfig() LockConfig {
	return LockConfig{
		DefaultTTL:    30 * time.Second,
		RetryInterval: 100 * time.Millisecond,
		TableName:     "k1s0_distributed_locks",
		KeyPrefix:     "k1s0:lock:",
	}
}

// DefaultSagaConfig returns a SagaConfig with default values.
func DefaultSagaConfig() SagaConfig {
	return SagaConfig{
		StepTimeout:         30 * time.Second,
		MaxRetries:          3,
		TableName:           "k1s0_saga_instances",
		DeadLetterTableName: "k1s0_saga_dead_letters",
	}
}

// Validate validates the configuration and applies defaults where needed.
func (c *ConsensusConfig) Validate() {
	c.Leader.Validate()
	c.Lock.Validate()
	c.Saga.Validate()
}

// Validate validates leader configuration and applies defaults.
func (c *LeaderConfig) Validate() {
	if c.LeaseDuration <= 0 {
		c.LeaseDuration = 15 * time.Second
	}
	if c.RenewInterval <= 0 {
		c.RenewInterval = 5 * time.Second
	}
	if c.RenewInterval >= c.LeaseDuration {
		c.RenewInterval = c.LeaseDuration / 3
	}
	if c.WatchPollInterval <= 0 {
		c.WatchPollInterval = 2 * time.Second
	}
	if c.TableName == "" {
		c.TableName = "k1s0_leader_election"
	}
}

// Validate validates lock configuration and applies defaults.
func (c *LockConfig) Validate() {
	if c.DefaultTTL <= 0 {
		c.DefaultTTL = 30 * time.Second
	}
	if c.RetryInterval <= 0 {
		c.RetryInterval = 100 * time.Millisecond
	}
	if c.TableName == "" {
		c.TableName = "k1s0_distributed_locks"
	}
	if c.KeyPrefix == "" {
		c.KeyPrefix = "k1s0:lock:"
	}
}

// Validate validates saga configuration and applies defaults.
func (c *SagaConfig) Validate() {
	if c.StepTimeout <= 0 {
		c.StepTimeout = 30 * time.Second
	}
	if c.MaxRetries <= 0 {
		c.MaxRetries = 3
	}
	if c.TableName == "" {
		c.TableName = "k1s0_saga_instances"
	}
	if c.DeadLetterTableName == "" {
		c.DeadLetterTableName = "k1s0_saga_dead_letters"
	}
}
