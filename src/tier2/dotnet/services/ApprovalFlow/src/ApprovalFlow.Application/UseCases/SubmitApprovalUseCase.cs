// 新規承認申請ユースケース。
//
// docs 正典:
//   docs/05_実装/00_ディレクトリ設計/30_tier2レイアウト/04_サービス単位の内部構造.md

using K1s0.Tier2.ApprovalFlow.Domain.Entities;
using K1s0.Tier2.ApprovalFlow.Domain.Interfaces;
using K1s0.Tier2.ApprovalFlow.Domain.ValueObjects;

namespace K1s0.Tier2.ApprovalFlow.Application.UseCases;

// SubmitApprovalUseCase は新規承認申請を組み立てて永続化する。
public sealed class SubmitApprovalUseCase
{
    // 永続化境界。
    private readonly IApprovalRepository _repo;

    // 時刻取得（テスト容易性）。
    private readonly Func<DateTimeOffset> _now;

    // 既定では UTC 現在時刻を使う。
    public SubmitApprovalUseCase(IApprovalRepository repo, Func<DateTimeOffset>? now = null)
    {
        _repo = repo;
        _now = now ?? (() => DateTimeOffset.UtcNow);
    }

    // 入力 DTO。
    public sealed record Input(string Requester, string Currency, long MinorAmount, string Reason);

    // 出力 DTO。
    public sealed record Output(ApprovalId Id, string Status, DateTimeOffset SubmittedAt);

    // 1 件の承認申請を作成する。
    public async Task<Output> ExecuteAsync(Input input, CancellationToken ct)
    {
        // Money 値オブジェクトを組み立てる（バリデーション含む）。
        var money = new Money(input.Currency, input.MinorAmount);
        // ApprovalRequest を新規生成する。
        var req = ApprovalRequest.NewSubmission(input.Requester, money, input.Reason, _now());
        // 永続化する。
        await _repo.SaveAsync(req, ct).ConfigureAwait(false);
        // ドメインイベント発火（PubSub publish 等）はリリース時点 では割愛、リリース時点 で組み込む。
        req.ClearEvents();
        // 出力 DTO を返す。
        return new Output(req.Id, req.Status.ToString(), req.SubmittedAt);
    }
}
