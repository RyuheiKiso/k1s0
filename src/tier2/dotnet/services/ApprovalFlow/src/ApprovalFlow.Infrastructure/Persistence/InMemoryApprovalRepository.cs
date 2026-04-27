// 承認の in-memory Repository 実装。
//
// docs 正典:
//   docs/05_実装/00_ディレクトリ設計/30_tier2レイアウト/04_サービス単位の内部構造.md
//
// scope:
//   リリース時点 の最小実装として ConcurrentDictionary backed の永続化を提供する。
//   リリース時点 で k1s0 State / Postgres EF Core 実装に置換する（同 interface のため
//   Application 層を変更せずに差し替え可能）。

using System.Collections.Concurrent;
using K1s0.Tier2.ApprovalFlow.Domain.Entities;
using K1s0.Tier2.ApprovalFlow.Domain.Interfaces;
using K1s0.Tier2.ApprovalFlow.Domain.ValueObjects;

namespace K1s0.Tier2.ApprovalFlow.Infrastructure.Persistence;

// InMemoryApprovalRepository はテスト / dev 起動時用の in-memory 実装。
public sealed class InMemoryApprovalRepository : IApprovalRepository
{
    // ID -> 集約のマップ。Concurrent でスレッドセーフに保つ。
    private readonly ConcurrentDictionary<ApprovalId, ApprovalRequest> _store = new();

    // ID で集約を取得する。未存在は null。
    public Task<ApprovalRequest?> FindByIdAsync(ApprovalId id, CancellationToken ct)
    {
        // 取得試行。
        _store.TryGetValue(id, out var req);
        return Task.FromResult(req);
    }

    // 集約を保存する（新規 / 更新両対応）。
    public Task SaveAsync(ApprovalRequest request, CancellationToken ct)
    {
        // ConcurrentDictionary の AddOrUpdate で冪等。
        _store.AddOrUpdate(request.Id, request, (_, _) => request);
        return Task.CompletedTask;
    }
}
