// 承認が完了した時のドメインイベント。

using K1s0.Tier2.ApprovalFlow.Domain.ValueObjects;

namespace K1s0.Tier2.ApprovalFlow.Domain.Events;

// ApprovalApproved は承認が確定した時に発火する。
public sealed record ApprovalApproved(ApprovalId Id, string DecidedBy, DateTimeOffset OccurredAt);
