using System.Collections.Concurrent;

namespace K1s0.SessionClient;

public class InMemorySessionClient : ISessionClient
{
    private readonly ConcurrentDictionary<string, Session> _sessions = new();
    private int _counter;

    public Task<Session> CreateAsync(CreateSessionRequest req, CancellationToken ct = default)
    {
        var id = Interlocked.Increment(ref _counter).ToString();
        var now = DateTimeOffset.UtcNow;
        var session = new Session(
            Id: id,
            UserId: req.UserId,
            Token: $"tok-{id}",
            ExpiresAt: now.AddSeconds(req.TtlSeconds),
            CreatedAt: now,
            Revoked: false,
            Metadata: req.Metadata ?? new Dictionary<string, string>());
        _sessions[id] = session;
        return Task.FromResult(session);
    }

    public Task<Session?> GetAsync(string id, CancellationToken ct = default)
    {
        _sessions.TryGetValue(id, out var session);
        return Task.FromResult(session);
    }

    public Task<Session> RefreshAsync(RefreshSessionRequest req, CancellationToken ct = default)
    {
        if (!_sessions.TryGetValue(req.Id, out var existing))
        {
            throw new InvalidOperationException($"Session not found: {req.Id}");
        }

        var refreshed = existing with
        {
            ExpiresAt = DateTimeOffset.UtcNow.AddSeconds(req.TtlSeconds),
            Token = $"tok-{req.Id}-refreshed",
        };
        _sessions[req.Id] = refreshed;
        return Task.FromResult(refreshed);
    }

    public Task RevokeAsync(string id, CancellationToken ct = default)
    {
        if (_sessions.TryGetValue(id, out var existing))
        {
            _sessions[id] = existing with { Revoked = true };
        }

        return Task.CompletedTask;
    }

    public Task<IReadOnlyList<Session>> ListUserSessionsAsync(string userId, CancellationToken ct = default)
    {
        var sessions = _sessions.Values
            .Where(s => s.UserId == userId)
            .ToList();
        return Task.FromResult<IReadOnlyList<Session>>(sessions);
    }

    public Task<int> RevokeAllAsync(string userId, CancellationToken ct = default)
    {
        var count = 0;
        foreach (var kvp in _sessions)
        {
            if (kvp.Value.UserId == userId && !kvp.Value.Revoked)
            {
                _sessions[kvp.Key] = kvp.Value with { Revoked = true };
                count++;
            }
        }

        return Task.FromResult(count);
    }
}
