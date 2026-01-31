// Package consensus provides distributed consensus primitives for the k1s0 framework.
//
// This package implements leader election, distributed locking, fencing tokens,
// and saga orchestration patterns for coordinating distributed microservices.
//
// # Leader Election
//
// Leader election ensures only one node acts as the leader for a given lease key.
// Supported backends: PostgreSQL.
//
//	elector := consensus.NewDbLeaderElector(pool, cfg)
//	lease, err := elector.TryAcquire(ctx, "my-service-leader")
//
// # Distributed Locking
//
// Distributed locks provide mutual exclusion across processes.
// Supported backends: PostgreSQL, Redis.
//
//	lock := consensus.NewRedisDistributedLock(client)
//	guard, err := lock.TryLock(ctx, "resource-key", 30*time.Second)
//	defer guard.Close()
//
// # Saga Orchestration
//
// Sagas coordinate multi-step distributed transactions with compensation.
//
//	def := consensus.NewSagaBuilder[MyCtx]("order-saga").
//	    Step(createOrderStep).
//	    Step(reserveInventoryStep).
//	    Build()
//	orch := consensus.NewSagaOrchestrator(pool)
//	result, err := orch.Execute(ctx, def, &myCtx)
//
// # Configuration
//
// All configuration is via YAML files (no environment variables):
//
//	consensus:
//	  leader:
//	    lease_duration: 15s
//	    renew_interval: 5s
//	    node_id: "node-1"
//	  lock:
//	    default_ttl: 30s
//	    retry_interval: 100ms
//	  saga:
//	    step_timeout: 30s
//	    max_retries: 3
package consensus

import "errors"

var (
	// ErrLeaseExpired indicates that a leader lease has expired.
	ErrLeaseExpired = errors.New("consensus: leader lease expired")

	// ErrLockTimeout indicates that a lock acquisition timed out.
	ErrLockTimeout = errors.New("consensus: lock acquisition timed out")

	// ErrFenceTokenViolation indicates that a fence token was not monotonically increasing.
	ErrFenceTokenViolation = errors.New("consensus: fence token violation")

	// ErrSagaFailed indicates that a saga step execution failed.
	ErrSagaFailed = errors.New("consensus: saga execution failed")

	// ErrCompensationFailed indicates that a saga compensation step failed.
	ErrCompensationFailed = errors.New("consensus: saga compensation failed")

	// ErrDeadLetter indicates that a saga was moved to the dead letter queue
	// after compensation failure.
	ErrDeadLetter = errors.New("consensus: saga moved to dead letter queue")

	// ErrNotLeader indicates that the current node is not the leader.
	ErrNotLeader = errors.New("consensus: not the leader")

	// ErrLockNotHeld indicates that the lock is not held by the caller.
	ErrLockNotHeld = errors.New("consensus: lock not held")

	// ErrLeaseNotFound indicates that no lease was found for the given key.
	ErrLeaseNotFound = errors.New("consensus: lease not found")
)
