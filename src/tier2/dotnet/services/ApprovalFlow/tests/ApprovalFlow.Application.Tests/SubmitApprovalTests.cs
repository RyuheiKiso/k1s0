// SubmitApprovalUseCase の単体テスト。

using K1s0.Tier2.ApprovalFlow.Application.UseCases;
using K1s0.Tier2.ApprovalFlow.Infrastructure.Persistence;
using Xunit;

namespace K1s0.Tier2.ApprovalFlow.Application.Tests;

// SubmitApprovalUseCase の振る舞いを検証する。
public class SubmitApprovalTests
{
    // 固定時刻。
    private static DateTimeOffset Now() => new(2026, 4, 27, 12, 0, 0, TimeSpan.Zero);

    [Fact]
    [Trait("Category", "Unit")]
    public async Task Execute_PersistsAndReturnsId()
    {
        // in-memory repo。
        var repo = new InMemoryApprovalRepository();
        // UseCase。
        var useCase = new SubmitApprovalUseCase(repo, Now);
        // 実行する。
        var output = await useCase.ExecuteAsync(new SubmitApprovalUseCase.Input("alice", "JPY", 1000, "test"), CancellationToken.None);
        // 状態は Pending。
        Assert.Equal("PENDING", output.Status);
        // repo に永続化されている。
        var persisted = await repo.FindByIdAsync(output.Id, CancellationToken.None);
        Assert.NotNull(persisted);
        Assert.Equal("alice", persisted!.Requester);
    }

    [Fact]
    [Trait("Category", "Unit")]
    public async Task Execute_ZeroAmount_Throws()
    {
        var repo = new InMemoryApprovalRepository();
        var useCase = new SubmitApprovalUseCase(repo, Now);
        await Assert.ThrowsAsync<ArgumentException>(() => useCase.ExecuteAsync(new SubmitApprovalUseCase.Input("alice", "JPY", 0, "test"), CancellationToken.None));
    }
}
