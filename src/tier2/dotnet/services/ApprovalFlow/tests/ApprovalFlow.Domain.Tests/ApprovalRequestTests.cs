// ApprovalRequest エンティティの単体テスト。

using K1s0.Tier2.ApprovalFlow.Domain.Entities;
using K1s0.Tier2.ApprovalFlow.Domain.Events;
using K1s0.Tier2.ApprovalFlow.Domain.ValueObjects;
using Xunit;

namespace K1s0.Tier2.ApprovalFlow.Domain.Tests;

// ApprovalRequest の状態遷移を網羅検証する。
public class ApprovalRequestTests
{
    // 固定時刻ヘルパ。
    private static DateTimeOffset Now() => new(2026, 4, 27, 12, 0, 0, TimeSpan.Zero);

    [Fact]
    [Trait("Category", "Unit")]
    public void NewSubmission_PendingStatus()
    {
        // 新規申請は Pending で開始する。
        var req = ApprovalRequest.NewSubmission("alice", new Money("JPY", 1000), "test", Now());
        Assert.Equal(ApprovalStatus.Pending, req.Status);
        // ApprovalSubmitted イベントが 1 件積まれている。
        Assert.Single(req.PendingEvents);
        Assert.IsType<ApprovalSubmitted>(req.PendingEvents[0]);
    }

    [Fact]
    [Trait("Category", "Unit")]
    public void NewSubmission_RejectsZeroAmount()
    {
        // 0 円は不正。
        Assert.Throws<ArgumentException>(() => ApprovalRequest.NewSubmission("alice", new Money("JPY", 0), "test", Now()));
    }

    [Fact]
    [Trait("Category", "Unit")]
    public void Approve_FromPending_TransitionsToApproved()
    {
        var req = ApprovalRequest.NewSubmission("alice", new Money("JPY", 1000), "test", Now());
        // 承認する。
        req.Approve("bob", Now().AddMinutes(1));
        Assert.Equal(ApprovalStatus.Approved, req.Status);
        Assert.Equal("bob", req.DecidedBy);
        // ApprovalApproved イベントが追加されている。
        Assert.Equal(2, req.PendingEvents.Count);
    }

    [Fact]
    [Trait("Category", "Unit")]
    public void Approve_FromNonPending_Throws()
    {
        var req = ApprovalRequest.NewSubmission("alice", new Money("JPY", 1000), "test", Now());
        req.Approve("bob", Now());
        // すでに Approved なので 2 度目の Approve は不正。
        Assert.Throws<InvalidOperationException>(() => req.Approve("carol", Now()));
    }

    [Fact]
    [Trait("Category", "Unit")]
    public void Reject_FromPending_TransitionsToRejected()
    {
        var req = ApprovalRequest.NewSubmission("alice", new Money("JPY", 1000), "test", Now());
        req.Reject("bob", Now());
        Assert.Equal(ApprovalStatus.Rejected, req.Status);
        Assert.Equal("bob", req.DecidedBy);
    }

    [Fact]
    [Trait("Category", "Unit")]
    public void Cancel_OnlyByRequester()
    {
        var req = ApprovalRequest.NewSubmission("alice", new Money("JPY", 1000), "test", Now());
        // 申請者本人はキャンセル可能。
        req.Cancel("alice", Now());
        Assert.Equal(ApprovalStatus.Cancelled, req.Status);
    }

    [Fact]
    [Trait("Category", "Unit")]
    public void Cancel_ByOther_Throws()
    {
        var req = ApprovalRequest.NewSubmission("alice", new Money("JPY", 1000), "test", Now());
        // 他人によるキャンセルは不正。
        Assert.Throws<InvalidOperationException>(() => req.Cancel("eve", Now()));
    }
}
