// InvoiceGenerator API エンドポイント定義。

using System.Collections.ObjectModel;
using K1s0.Tier2.InvoiceGenerator.Application.UseCases;

namespace K1s0.Tier2.InvoiceGenerator.Api.Controllers;

public static class InvoiceEndpoints
{
    // POST /api/invoices 入力。
    public sealed record CreateLineBody(string Description, int Quantity, string Currency, long UnitMinorAmount);
    // CA1002: 公開 record の collection は List<T> ではなく Collection<T> を使う。
    public sealed record CreateInvoiceBody(string Customer, Collection<CreateLineBody> Lines);

    public static void MapInvoiceEndpoints(this WebApplication app)
    {
        // POST /api/invoices : 新規請求書生成。
        app.MapPost("/api/invoices", async (CreateInvoiceBody body, GenerateInvoiceUseCase useCase, CancellationToken ct) =>
        {
            try
            {
                // 入力 DTO に変換する。
                var lines = body.Lines.Select(l => new GenerateInvoiceUseCase.LineInput(l.Description, l.Quantity, l.Currency, l.UnitMinorAmount)).ToList();
                var output = await useCase.ExecuteAsync(new GenerateInvoiceUseCase.Input(body.Customer, lines), ct).ConfigureAwait(false);
                return Results.Created($"/api/invoices/{output.Id}", output);
            }
            catch (ArgumentException ex)
            {
                return Results.BadRequest(new { error = new { code = "E-T2-INVOICE-001", message = ex.Message, category = "VALIDATION" } });
            }
            catch (InvalidOperationException ex)
            {
                return Results.BadRequest(new { error = new { code = "E-T2-INVOICE-002", message = ex.Message, category = "VALIDATION" } });
            }
        }).RequireAuthorization();
    }
}
