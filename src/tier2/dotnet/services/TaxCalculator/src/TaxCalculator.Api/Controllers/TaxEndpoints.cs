// TaxCalculator API エンドポイント定義。

using K1s0.Tier2.TaxCalculator.Application.UseCases;

namespace K1s0.Tier2.TaxCalculator.Api.Controllers;

public static class TaxEndpoints
{
    // POST /api/tax/calculate 入力 DTO。
    public sealed record CalculateBody(string Mode, string Currency, long MinorAmount, int RateBasisPoints);

    public static void MapTaxEndpoints(this WebApplication app)
    {
        app.MapPost("/api/tax/calculate", (CalculateBody body, CalculateTaxUseCase useCase) =>
        {
            try
            {
                var output = useCase.Execute(new CalculateTaxUseCase.Input(body.Mode, body.Currency, body.MinorAmount, body.RateBasisPoints));
                return Results.Ok(output);
            }
            catch (ArgumentException ex)
            {
                return Results.BadRequest(new { error = new { code = "E-T2-TAX-001", message = ex.Message, category = "VALIDATION" } });
            }
        }).RequireAuthorization();
    }
}
