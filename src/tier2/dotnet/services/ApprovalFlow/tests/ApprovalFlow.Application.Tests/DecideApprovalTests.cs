// DecideApprovalUseCase の単体テスト。

using K1s0.Tier2.ApprovalFlow.Application.UseCases;
using K1s0.Tier2.ApprovalFlow.Infrastructure.Persistence;
using Xunit;

namespace K1s0.Tier2.ApprovalFlow.Application.Tests;

// DecideApprovalUseCase の振る舞いを検証する。
public class DecideApprovalTests
{
    // 固定時刻。
    private static DateTimeOffset Now() => new(2026, 4, 27, 12, 0, 0, TimeSpan.Zero);

    [Fact]
    [Trait("Category", "Unit")]
    public async Task Execute_Approve()
    {
        var repo = new InMemoryApprovalRepository();
        // 事前に submit する。
        var submit = new SubmitApprovalUseCase(repo, Now);
        var submitted = await submit.ExecuteAsync(new SubmitApprovalUseCase.Input("alice", "JPY", 1000, "test"), CancellationToken.None);
        // 承認 UseCase。
        var decide = new DecideApprovalUseCase(repo, Now);
        var result = await decide.ExecuteAsync(new DecideApprovalUseCase.Input(submitted.Id, "bob", "APPROVE"), CancellationToken.None);
        Assert.Equal("APPROVED", result.Status);
        Assert.Equal("bob", result.DecidedBy);
    }

    [Fact]
    [Trait("Category", "Unit")]
    public async Task Execute_Reject()
    {
        var repo = new InMemoryApprovalRepository();
        var submit = new SubmitApprovalUseCase(repo, Now);
        var submitted = await submit.ExecuteAsync(new SubmitApprovalUseCase.Input("alice", "JPY", 1000, "test"), CancellationToken.None);
        var decide = new DecideApprovalUseCase(repo, Now);
        var result = await decide.ExecuteAsync(new DecideApprovalUseCase.Input(submitted.Id, "bob", "REJECT"), CancellationToken.None);
        Assert.Equal("REJECTED", result.Status);
    }

    [Fact]
    [Trait("Category", "Unit")]
    public async Task Execute_NotFound_Throws()
    {
        var repo = new InMemoryApprovalRepository();
        var decide = new DecideApprovalUseCase(repo, Now);
        await Assert.ThrowsAsync<KeyNotFoundException>(() => decide.ExecuteAsync(new DecideApprovalUseCase.Input(K1s0.Tier2.ApprovalFlow.Domain.ValueObjects.ApprovalId.NewId(), "bob", "APPROVE"), CancellationToken.None));
    }

    [Fact]
    [Trait("Category", "Unit")]
    public async Task Execute_InvalidDecision_Throws()
    {
        var repo = new InMemoryApprovalRepository();
        var submit = new SubmitApprovalUseCase(repo, Now);
        var submitted = await submit.ExecuteAsync(new SubmitApprovalUseCase.Input("alice", "JPY", 1000, "test"), CancellationToken.None);
        var decide = new DecideApprovalUseCase(repo, Now);
        await Assert.ThrowsAsync<ArgumentException>(() => decide.ExecuteAsync(new DecideApprovalUseCase.Input(submitted.Id, "bob", "MAYBE"), CancellationToken.None));
    }
}
