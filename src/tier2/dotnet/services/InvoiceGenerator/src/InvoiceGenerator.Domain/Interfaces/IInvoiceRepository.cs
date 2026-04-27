// Invoice Repository インタフェース（Domain 層の永続化境界）。

using K1s0.Tier2.InvoiceGenerator.Domain.Entities;
using K1s0.Tier2.InvoiceGenerator.Domain.ValueObjects;

namespace K1s0.Tier2.InvoiceGenerator.Domain.Interfaces;

public interface IInvoiceRepository
{
    Task<Invoice?> FindByIdAsync(InvoiceId id, CancellationToken ct);
    Task SaveAsync(Invoice invoice, CancellationToken ct);
}
