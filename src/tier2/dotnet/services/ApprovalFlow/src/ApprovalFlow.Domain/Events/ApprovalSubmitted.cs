// 承認申請が完了した時のドメインイベント。

using K1s0.Tier2.ApprovalFlow.Domain.ValueObjects;

namespace K1s0.Tier2.ApprovalFlow.Domain.Events;

// ApprovalSubmitted は承認申請が新規登録された時に発火する。
public sealed record ApprovalSubmitted(ApprovalId Id, string Requester, Money Amount, DateTimeOffset OccurredAt);
