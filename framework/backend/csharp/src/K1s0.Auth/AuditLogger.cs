using Microsoft.Extensions.Logging;

namespace K1s0.Auth;

/// <summary>
/// Structured audit logger for authentication and authorization events.
/// </summary>
public class AuditLogger
{
    private readonly ILogger<AuditLogger> _logger;

    /// <summary>
    /// Initializes a new instance of the <see cref="AuditLogger"/> class.
    /// </summary>
    /// <param name="logger">The logger instance.</param>
    public AuditLogger(ILogger<AuditLogger> logger)
    {
        _logger = logger ?? throw new ArgumentNullException(nameof(logger));
    }

    /// <summary>
    /// Logs an authentication event.
    /// </summary>
    /// <param name="sub">The subject identifier.</param>
    /// <param name="success">Whether authentication was successful.</param>
    /// <param name="reason">Optional reason for failure.</param>
    public void LogAuthentication(string sub, bool success, string? reason = null)
    {
        if (success)
        {
            _logger.LogInformation("Authentication succeeded for subject {Sub}", sub);
        }
        else
        {
            _logger.LogWarning("Authentication failed for subject {Sub}: {Reason}", sub, reason ?? "unknown");
        }
    }

    /// <summary>
    /// Logs an authorization event.
    /// </summary>
    /// <param name="sub">The subject identifier.</param>
    /// <param name="action">The action attempted.</param>
    /// <param name="resource">The resource accessed.</param>
    /// <param name="allowed">Whether the action was allowed.</param>
    public void LogAuthorization(string sub, string action, string resource, bool allowed)
    {
        if (allowed)
        {
            _logger.LogInformation(
                "Authorization granted for subject {Sub}: {Action} on {Resource}",
                sub, action, resource);
        }
        else
        {
            _logger.LogWarning(
                "Authorization denied for subject {Sub}: {Action} on {Resource}",
                sub, action, resource);
        }
    }
}
