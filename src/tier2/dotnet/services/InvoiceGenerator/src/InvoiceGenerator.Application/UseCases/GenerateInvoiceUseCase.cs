// 請求書生成ユースケース。

using K1s0.Tier2.InvoiceGenerator.Domain.Entities;
using K1s0.Tier2.InvoiceGenerator.Domain.Interfaces;
using K1s0.Tier2.InvoiceGenerator.Domain.ValueObjects;

namespace K1s0.Tier2.InvoiceGenerator.Application.UseCases;

// GenerateInvoiceUseCase は行リストから Invoice を生成して永続化する。
public sealed class GenerateInvoiceUseCase
{
    private readonly IInvoiceRepository _repo;
    private readonly Func<DateTimeOffset> _now;

    public GenerateInvoiceUseCase(IInvoiceRepository repo, Func<DateTimeOffset>? now = null)
    {
        _repo = repo;
        _now = now ?? (() => DateTimeOffset.UtcNow);
    }

    // 入力 DTO。
    public sealed record LineInput(string Description, int Quantity, string Currency, long UnitMinorAmount);

    // 入力 DTO。
    public sealed record Input(string Customer, IReadOnlyList<LineInput> Lines);

    // 出力 DTO。
    public sealed record Output(InvoiceId Id, string Customer, long TotalMinorAmount, string Currency, DateTimeOffset IssuedAt);

    // 1 件の請求書を生成して保存する。
    public async Task<Output> ExecuteAsync(Input input, CancellationToken ct)
    {
        if (input.Lines.Count == 0)
        {
            throw new ArgumentException("lines is required", nameof(input));
        }
        // Domain エンティティを構築する。
        var lines = input.Lines.Select(l => new InvoiceLine(l.Description, l.Quantity, new Money(l.Currency, l.UnitMinorAmount))).ToList();
        // Invoice を組み立てる。
        var invoice = Invoice.Create(input.Customer, lines, _now());
        // 保存する。
        await _repo.SaveAsync(invoice, ct).ConfigureAwait(false);
        return new Output(invoice.Id, invoice.Customer, invoice.Total.MinorAmount, invoice.Total.Currency, invoice.IssuedAt);
    }
}
