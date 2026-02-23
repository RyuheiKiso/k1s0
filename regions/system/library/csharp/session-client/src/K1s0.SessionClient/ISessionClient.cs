namespace K1s0.SessionClient;

public interface ISessionClient
{
    Task<Session> CreateAsync(CreateSessionRequest req, CancellationToken ct = default);

    Task<Session?> GetAsync(string id, CancellationToken ct = default);

    Task<Session> RefreshAsync(RefreshSessionRequest req, CancellationToken ct = default);

    Task RevokeAsync(string id, CancellationToken ct = default);

    Task<IReadOnlyList<Session>> ListUserSessionsAsync(string userId, CancellationToken ct = default);

    Task<int> RevokeAllAsync(string userId, CancellationToken ct = default);
}
