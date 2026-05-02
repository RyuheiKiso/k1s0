// 給与計算 HTTP Controller。
//
// 既存システムが本 wrapper の HTTP エンドポイントを叩くと、内部で Legacy ロジック呼出
// + tier1 State 保存 + 監査記録を実行し、結果 JSON を返す。

using K1s0.Legacy.PayrollWrapper.Models;
using K1s0.Legacy.PayrollWrapper.Services;
using Microsoft.AspNetCore.Mvc;

namespace K1s0.Legacy.PayrollWrapper.Controllers;

[ApiController]
[Route("payroll")]
public sealed class PayrollController : ControllerBase
{
    private readonly PayrollService _service;

    public PayrollController(PayrollService service)
    {
        _service = service;
    }

    // POST /payroll/calculate
    // 既存システムが本 endpoint を叩くと、Legacy ロジック呼出 + tier1 永続化 + 監査記録。
    [HttpPost("calculate")]
    public async Task<ActionResult<PayrollRecord>> Calculate([FromBody] PayrollCalculationRequest req, CancellationToken ct)
    {
        // 必須項目を簡易検証する。
        if (string.IsNullOrWhiteSpace(req.EmployeeId) || string.IsNullOrWhiteSpace(req.TargetMonth))
        {
            return BadRequest(new { code = "E-T3-WRAP-PAYROLL-100", message = "employee_id and target_month are required" });
        }
        try
        {
            var record = await _service.CalculateAndPersistAsync(req, ct);
            return Ok(record);
        }
        catch (HttpRequestException ex)
        {
            // tier1 / BFF 側エラーは 502 として転送する（wrapper 側のバグと区別する）。
            return StatusCode(StatusCodes.Status502BadGateway, new { code = "E-T3-WRAP-PAYROLL-200", message = ex.Message });
        }
    }

    // GET /payroll/healthz
    [HttpGet("healthz")]
    public IActionResult Healthz() => Ok(new { status = "ok" });
}
