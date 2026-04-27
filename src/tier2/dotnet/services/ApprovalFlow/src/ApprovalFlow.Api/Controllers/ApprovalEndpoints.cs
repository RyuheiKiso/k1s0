// ApprovalFlow API エンドポイント定義（ASP.NET Core minimal API）。
//
// docs 正典:
//   docs/05_実装/00_ディレクトリ設計/30_tier2レイアウト/04_サービス単位の内部構造.md

using K1s0.Tier2.ApprovalFlow.Application.UseCases;
using K1s0.Tier2.ApprovalFlow.Domain.ValueObjects;

namespace K1s0.Tier2.ApprovalFlow.Api.Controllers;

// 承認 API のエンドポイント登録ヘルパ。
public static class ApprovalEndpoints
{
    // POST /api/approvals 入力 DTO。
    public sealed record SubmitRequestBody(string Requester, string Currency, long MinorAmount, string Reason);

    // POST /api/approvals/{id}/decide 入力 DTO。
    public sealed record DecideRequestBody(string Approver, string Decision);

    // ルートを組み立てる。
    public static void MapApprovalEndpoints(this WebApplication app)
    {
        // POST /api/approvals : 新規承認申請。
        app.MapPost("/api/approvals", async (SubmitRequestBody body, SubmitApprovalUseCase useCase, CancellationToken ct) =>
        {
            try
            {
                // Application UseCase に委譲する。
                var output = await useCase.ExecuteAsync(new SubmitApprovalUseCase.Input(body.Requester, body.Currency, body.MinorAmount, body.Reason), ct).ConfigureAwait(false);
                // 201 Created で返却。
                return Results.Created($"/api/approvals/{output.Id}", output);
            }
            catch (ArgumentException ex)
            {
                // ドメイン入力不正は 400。
                return Results.BadRequest(new { error = new { code = "E-T2-APPR-001", message = ex.Message, category = "VALIDATION" } });
            }
        });

        // POST /api/approvals/{id}/decide : 承認 / 却下。
        app.MapPost("/api/approvals/{id}/decide", async (string id, DecideRequestBody body, DecideApprovalUseCase useCase, CancellationToken ct) =>
        {
            try
            {
                // ID を ApprovalId に変換する。
                var approvalId = ApprovalId.Parse(id);
                // UseCase 実行。
                var output = await useCase.ExecuteAsync(new DecideApprovalUseCase.Input(approvalId, body.Approver, body.Decision), ct).ConfigureAwait(false);
                // 200 OK で返却。
                return Results.Ok(output);
            }
            catch (FormatException ex)
            {
                return Results.BadRequest(new { error = new { code = "E-T2-APPR-002", message = ex.Message, category = "VALIDATION" } });
            }
            catch (ArgumentException ex)
            {
                return Results.BadRequest(new { error = new { code = "E-T2-APPR-003", message = ex.Message, category = "VALIDATION" } });
            }
            catch (KeyNotFoundException ex)
            {
                // 集約が存在しない場合は 404。
                return Results.NotFound(new { error = new { code = "E-T2-APPR-004", message = ex.Message, category = "NOT_FOUND" } });
            }
            catch (InvalidOperationException ex)
            {
                // 状態遷移違反は 409。
                return Results.Conflict(new { error = new { code = "E-T2-APPR-005", message = ex.Message, category = "CONFLICT" } });
            }
        });
    }
}
