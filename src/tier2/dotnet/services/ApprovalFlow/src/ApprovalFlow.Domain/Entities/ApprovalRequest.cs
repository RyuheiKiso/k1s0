// 承認リクエストエンティティ。
//
// docs 正典:
//   docs/05_実装/00_ディレクトリ設計/30_tier2レイアウト/04_サービス単位の内部構造.md
//
// 役割:
//   1 件の承認を表す集約ルート。状態遷移（Pending → Approved/Rejected/Cancelled）を
//   不変条件付きで提供する。

using K1s0.Tier2.ApprovalFlow.Domain.Events;
using K1s0.Tier2.ApprovalFlow.Domain.ValueObjects;

namespace K1s0.Tier2.ApprovalFlow.Domain.Entities;

// ApprovalRequest は承認 1 件を表す集約ルート。
public sealed class ApprovalRequest
{
    // 識別子。
    public ApprovalId Id { get; }

    // リクエスター（subject）。
    public string Requester { get; }

    // 申請金額。
    public Money Amount { get; }

    // 申請理由（人間可読）。
    public string Reason { get; }

    // 現在の状態。
    public ApprovalStatus Status { get; private set; }

    // 申請日時（UTC）。
    public DateTimeOffset SubmittedAt { get; }

    // 最終更新日時（UTC）。
    public DateTimeOffset UpdatedAt { get; private set; }

    // 確定承認者 / 却下者（Approved / Rejected 時に設定）。
    public string? DecidedBy { get; private set; }

    // 集約発火イベント（pull 型、UseCase が読み取って publish する）。
    private readonly List<object> _events = [];

    // private コンストラクタ。生成は NewSubmission 経由のみ。
    private ApprovalRequest(ApprovalId id, string requester, Money amount, string reason, ApprovalStatus status, DateTimeOffset submittedAt, DateTimeOffset updatedAt)
    {
        Id = id;
        Requester = requester;
        Amount = amount;
        Reason = reason;
        Status = status;
        SubmittedAt = submittedAt;
        UpdatedAt = updatedAt;
    }

    // 新規申請を組み立てる。Pending 状態で開始。
    public static ApprovalRequest NewSubmission(string requester, Money amount, string reason, DateTimeOffset now)
    {
        // 申請者は必須。
        if (string.IsNullOrWhiteSpace(requester))
        {
            throw new ArgumentException("requester is required", nameof(requester));
        }
        // 申請理由も必須。
        if (string.IsNullOrWhiteSpace(reason))
        {
            throw new ArgumentException("reason is required", nameof(reason));
        }
        // 0 円は申請として無効。
        if (amount.MinorAmount <= 0)
        {
            throw new ArgumentException("amount must be > 0", nameof(amount));
        }
        // 新規 ID を発行する。
        var req = new ApprovalRequest(ApprovalId.NewId(), requester.Trim(), amount, reason.Trim(), ApprovalStatus.Pending, now, now);
        // ドメインイベントを集約に積む。
        req._events.Add(new ApprovalSubmitted(req.Id, req.Requester, req.Amount, req.SubmittedAt));
        return req;
    }

    // Pending → Approved への状態遷移。
    public void Approve(string approver, DateTimeOffset now)
    {
        // approver は必須。
        if (string.IsNullOrWhiteSpace(approver))
        {
            throw new ArgumentException("approver is required", nameof(approver));
        }
        // 現状が Pending でない限り遷移不可。
        if (Status != ApprovalStatus.Pending)
        {
            throw new InvalidOperationException($"cannot approve from status {Status}");
        }
        // 状態を変える。
        Status = ApprovalStatus.Approved;
        DecidedBy = approver.Trim();
        UpdatedAt = now;
        // ドメインイベントを積む。
        _events.Add(new ApprovalApproved(Id, DecidedBy, now));
    }

    // Pending → Rejected への状態遷移。
    public void Reject(string approver, DateTimeOffset now)
    {
        if (string.IsNullOrWhiteSpace(approver))
        {
            throw new ArgumentException("approver is required", nameof(approver));
        }
        if (Status != ApprovalStatus.Pending)
        {
            throw new InvalidOperationException($"cannot reject from status {Status}");
        }
        Status = ApprovalStatus.Rejected;
        DecidedBy = approver.Trim();
        UpdatedAt = now;
    }

    // Pending → Cancelled への状態遷移（申請者本人による取り消し）。
    public void Cancel(string requester, DateTimeOffset now)
    {
        if (!string.Equals(requester?.Trim(), Requester, StringComparison.Ordinal))
        {
            throw new InvalidOperationException("only requester can cancel");
        }
        if (Status != ApprovalStatus.Pending)
        {
            throw new InvalidOperationException($"cannot cancel from status {Status}");
        }
        Status = ApprovalStatus.Cancelled;
        UpdatedAt = now;
    }

    // 集約に蓄積されたイベントを取り出す（読取のみ）。
    public IReadOnlyList<object> PendingEvents => _events;

    // イベント flush 後に呼ぶ（UseCase が publish 完了したら clear する）。
    public void ClearEvents() => _events.Clear();

    // 永続化からの復元用ファクトリ（イベント発火しない）。
    public static ApprovalRequest Rehydrate(ApprovalId id, string requester, Money amount, string reason, ApprovalStatus status, DateTimeOffset submittedAt, DateTimeOffset updatedAt, string? decidedBy)
    {
        var req = new ApprovalRequest(id, requester, amount, reason, status, submittedAt, updatedAt);
        req.DecidedBy = decidedBy;
        return req;
    }
}
