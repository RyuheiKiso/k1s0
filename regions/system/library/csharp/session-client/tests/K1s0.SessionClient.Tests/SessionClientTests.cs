using K1s0.SessionClient;

namespace K1s0.SessionClient.Tests;

public class SessionClientTests
{
    private readonly InMemorySessionClient _client = new();

    [Fact]
    public async Task Create_ReturnsSessionWithGeneratedId()
    {
        var session = await _client.CreateAsync(new CreateSessionRequest("user1", 3600));
        Assert.NotEmpty(session.Id);
        Assert.Equal("user1", session.UserId);
        Assert.NotEmpty(session.Token);
        Assert.False(session.Revoked);
    }

    [Fact]
    public async Task Create_WithMetadata()
    {
        var metadata = new Dictionary<string, string> { ["device"] = "mobile" };
        var session = await _client.CreateAsync(new CreateSessionRequest("user1", 3600, metadata));
        Assert.Equal("mobile", session.Metadata["device"]);
    }

    [Fact]
    public async Task Get_ReturnsExistingSession()
    {
        var created = await _client.CreateAsync(new CreateSessionRequest("user1", 3600));
        var fetched = await _client.GetAsync(created.Id);
        Assert.NotNull(fetched);
        Assert.Equal("user1", fetched!.UserId);
    }

    [Fact]
    public async Task Get_ReturnsNullForNonexistent()
    {
        var result = await _client.GetAsync("nonexistent");
        Assert.Null(result);
    }

    [Fact]
    public async Task Refresh_UpdatesExpiryAndToken()
    {
        var created = await _client.CreateAsync(new CreateSessionRequest("user1", 60));
        var refreshed = await _client.RefreshAsync(new RefreshSessionRequest(created.Id, 7200));
        Assert.Equal(created.Id, refreshed.Id);
        Assert.NotEqual(created.Token, refreshed.Token);
        Assert.True(refreshed.ExpiresAt > created.ExpiresAt);
    }

    [Fact]
    public async Task Refresh_ThrowsForNonexistentSession()
    {
        await Assert.ThrowsAsync<InvalidOperationException>(
            () => _client.RefreshAsync(new RefreshSessionRequest("bad", 60)));
    }

    [Fact]
    public async Task Revoke_MarksSessionAsRevoked()
    {
        var created = await _client.CreateAsync(new CreateSessionRequest("user1", 3600));
        await _client.RevokeAsync(created.Id);
        var fetched = await _client.GetAsync(created.Id);
        Assert.True(fetched!.Revoked);
    }

    [Fact]
    public async Task Revoke_NonexistentDoesNothing()
    {
        await _client.RevokeAsync("nonexistent");
    }

    [Fact]
    public async Task ListUserSessions_ReturnsUserSessions()
    {
        await _client.CreateAsync(new CreateSessionRequest("u1", 60));
        await _client.CreateAsync(new CreateSessionRequest("u1", 60));
        await _client.CreateAsync(new CreateSessionRequest("u2", 60));
        var sessions = await _client.ListUserSessionsAsync("u1");
        Assert.Equal(2, sessions.Count);
    }

    [Fact]
    public async Task RevokeAll_RevokesAllUserSessions()
    {
        await _client.CreateAsync(new CreateSessionRequest("u1", 60));
        await _client.CreateAsync(new CreateSessionRequest("u1", 60));
        await _client.CreateAsync(new CreateSessionRequest("u2", 60));
        var count = await _client.RevokeAllAsync("u1");
        Assert.Equal(2, count);
        var sessions = await _client.ListUserSessionsAsync("u1");
        Assert.All(sessions, s => Assert.True(s.Revoked));
    }

    [Fact]
    public async Task RevokeAll_ReturnsZeroForNoMatchingSessions()
    {
        var count = await _client.RevokeAllAsync("nobody");
        Assert.Equal(0, count);
    }
}
