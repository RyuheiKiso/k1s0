// ApprovalRepository インタフェース。Domain 層の永続化境界。
//
// docs 正典:
//   docs/05_実装/00_ディレクトリ設計/30_tier2レイアウト/04_サービス単位の内部構造.md

using K1s0.Tier2.ApprovalFlow.Domain.Entities;
using K1s0.Tier2.ApprovalFlow.Domain.ValueObjects;

namespace K1s0.Tier2.ApprovalFlow.Domain.Interfaces;

// IApprovalRepository は ApprovalRequest の永続化を抽象化する。
public interface IApprovalRepository
{
    // ID から取得する。未存在は null を返す。
    Task<ApprovalRequest?> FindByIdAsync(ApprovalId id, CancellationToken ct);

    // 保存する（新規 / 更新は実装側で判定）。
    Task SaveAsync(ApprovalRequest request, CancellationToken ct);
}
