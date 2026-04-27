// 承認確定 / 却下ユースケース。
//
// docs 正典:
//   docs/05_実装/00_ディレクトリ設計/30_tier2レイアウト/04_サービス単位の内部構造.md

using K1s0.Tier2.ApprovalFlow.Domain.Interfaces;
using K1s0.Tier2.ApprovalFlow.Domain.ValueObjects;

namespace K1s0.Tier2.ApprovalFlow.Application.UseCases;

// DecideApprovalUseCase は Pending → Approved/Rejected の状態遷移を実行する。
public sealed class DecideApprovalUseCase
{
    // 永続化境界。
    private readonly IApprovalRepository _repo;

    // 時刻取得。
    private readonly Func<DateTimeOffset> _now;

    public DecideApprovalUseCase(IApprovalRepository repo, Func<DateTimeOffset>? now = null)
    {
        _repo = repo;
        _now = now ?? (() => DateTimeOffset.UtcNow);
    }

    // 入力 DTO。decision は "APPROVE" / "REJECT" のみ。
    public sealed record Input(ApprovalId Id, string Approver, string Decision);

    // 出力 DTO。
    public sealed record Output(ApprovalId Id, string Status, string? DecidedBy, DateTimeOffset UpdatedAt);

    // 承認 / 却下を実行する。
    public async Task<Output> ExecuteAsync(Input input, CancellationToken ct)
    {
        // 既存の承認を取得する。
        var req = await _repo.FindByIdAsync(input.Id, ct).ConfigureAwait(false)
            ?? throw new KeyNotFoundException($"approval not found: {input.Id}");
        // decision で分岐する。
        switch (input.Decision?.Trim().ToUpperInvariant())
        {
            // 承認。
            case "APPROVE":
                req.Approve(input.Approver, _now());
                break;
            // 却下。
            case "REJECT":
                req.Reject(input.Approver, _now());
                break;
            // 不明な decision は不正。
            default:
                throw new ArgumentException($"invalid decision: {input.Decision}", nameof(input));
        }
        // 永続化する。
        await _repo.SaveAsync(req, ct).ConfigureAwait(false);
        // ドメインイベントの flush。
        req.ClearEvents();
        // 出力 DTO を返す。
        return new Output(req.Id, req.Status.ToString(), req.DecidedBy, req.UpdatedAt);
    }
}
