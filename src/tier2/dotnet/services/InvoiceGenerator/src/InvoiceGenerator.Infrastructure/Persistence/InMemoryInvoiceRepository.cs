// in-memory 請求書リポジトリ実装（リリース時点 最小、リリース時点 で k1s0 State backed に置換）。

using System.Collections.Concurrent;
using K1s0.Tier2.InvoiceGenerator.Domain.Entities;
using K1s0.Tier2.InvoiceGenerator.Domain.Interfaces;
using K1s0.Tier2.InvoiceGenerator.Domain.ValueObjects;

namespace K1s0.Tier2.InvoiceGenerator.Infrastructure.Persistence;

public sealed class InMemoryInvoiceRepository : IInvoiceRepository
{
    private readonly ConcurrentDictionary<InvoiceId, Invoice> _store = new();

    public Task<Invoice?> FindByIdAsync(InvoiceId id, CancellationToken ct)
    {
        _store.TryGetValue(id, out var invoice);
        return Task.FromResult(invoice);
    }

    public Task SaveAsync(Invoice invoice, CancellationToken ct)
    {
        // CA1062: 公開 API では引数 null を必ず明示検証する。
        ArgumentNullException.ThrowIfNull(invoice);
        _store.AddOrUpdate(invoice.Id, invoice, (_, _) => invoice);
        return Task.CompletedTask;
    }
}
