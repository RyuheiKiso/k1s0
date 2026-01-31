using System.Net;
using K1s0.Error;

namespace K1s0.Consensus;

/// <summary>
/// Base exception for all consensus-related errors.
/// </summary>
public class ConsensusException : K1s0Exception
{
    /// <summary>
    /// Creates a new <see cref="ConsensusException"/>.
    /// </summary>
    /// <param name="errorCode">Structured error code.</param>
    /// <param name="message">Human-readable message.</param>
    /// <param name="httpStatus">HTTP status code.</param>
    /// <param name="innerException">Optional inner exception.</param>
    public ConsensusException(
        string errorCode,
        string message,
        HttpStatusCode httpStatus = HttpStatusCode.InternalServerError,
        Exception? innerException = null)
        : base(errorCode, message, httpStatus, innerException)
    {
    }
}

/// <summary>
/// Thrown when leader election fails (e.g., lease already held).
/// </summary>
public class LeaderElectionException : ConsensusException
{
    /// <summary>
    /// Creates a new <see cref="LeaderElectionException"/>.
    /// </summary>
    public LeaderElectionException(string message, Exception? innerException = null)
        : base("consensus.leader.election_failed", message, HttpStatusCode.Conflict, innerException)
    {
    }
}

/// <summary>
/// Thrown when a lease has expired and can no longer be renewed.
/// </summary>
public class LeaseExpiredException : ConsensusException
{
    /// <summary>
    /// Creates a new <see cref="LeaseExpiredException"/>.
    /// </summary>
    public LeaseExpiredException(string leaseKey)
        : base("consensus.leader.lease_expired", $"Lease '{leaseKey}' has expired.")
    {
    }
}

/// <summary>
/// Thrown when acquiring a distributed lock fails.
/// </summary>
public class LockAcquisitionException : ConsensusException
{
    /// <summary>
    /// Creates a new <see cref="LockAcquisitionException"/>.
    /// </summary>
    public LockAcquisitionException(string lockKey, Exception? innerException = null)
        : base("consensus.lock.acquisition_failed", $"Failed to acquire lock '{lockKey}'.", HttpStatusCode.Conflict, innerException)
    {
    }
}

/// <summary>
/// Thrown when a lock operation is attempted but the lock is not held.
/// </summary>
public class LockNotHeldException : ConsensusException
{
    /// <summary>
    /// Creates a new <see cref="LockNotHeldException"/>.
    /// </summary>
    public LockNotHeldException(string lockKey)
        : base("consensus.lock.not_held", $"Lock '{lockKey}' is not held by this instance.")
    {
    }
}

/// <summary>
/// Thrown when a fencing token is stale (lower than the current known token).
/// </summary>
public class StaleFenceTokenException : ConsensusException
{
    /// <summary>
    /// The stale token that was presented.
    /// </summary>
    public ulong PresentedToken { get; }

    /// <summary>
    /// The current highest known token.
    /// </summary>
    public ulong CurrentToken { get; }

    /// <summary>
    /// Creates a new <see cref="StaleFenceTokenException"/>.
    /// </summary>
    public StaleFenceTokenException(ulong presentedToken, ulong currentToken)
        : base(
            "consensus.fencing.stale_token",
            $"Fence token {presentedToken} is stale; current is {currentToken}.",
            HttpStatusCode.Conflict)
    {
        PresentedToken = presentedToken;
        CurrentToken = currentToken;
    }
}

/// <summary>
/// Thrown when a saga step fails and compensation is required.
/// </summary>
public class SagaExecutionException : ConsensusException
{
    /// <summary>
    /// The name of the step that failed.
    /// </summary>
    public string StepName { get; }

    /// <summary>
    /// Creates a new <see cref="SagaExecutionException"/>.
    /// </summary>
    public SagaExecutionException(string stepName, string message, Exception? innerException = null)
        : base("consensus.saga.step_failed", $"Saga step '{stepName}' failed: {message}", HttpStatusCode.InternalServerError, innerException)
    {
        StepName = stepName;
    }
}

/// <summary>
/// Thrown when saga compensation fails.
/// </summary>
public class SagaCompensationException : ConsensusException
{
    /// <summary>
    /// The name of the step whose compensation failed.
    /// </summary>
    public string StepName { get; }

    /// <summary>
    /// Creates a new <see cref="SagaCompensationException"/>.
    /// </summary>
    public SagaCompensationException(string stepName, string message, Exception? innerException = null)
        : base("consensus.saga.compensation_failed", $"Compensation for step '{stepName}' failed: {message}", HttpStatusCode.InternalServerError, innerException)
    {
        StepName = stepName;
    }
}
