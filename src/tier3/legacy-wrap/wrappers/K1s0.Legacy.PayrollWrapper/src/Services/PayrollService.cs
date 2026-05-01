// PayrollService: Legacy ロジック呼出と k1s0 SDK 連携を Domain として束ねる Application 層。
//
// 責務:
//   1. Legacy.PayrollLegacy.CalculateNetPay (既存 .NET Framework ロジック) を呼ぶ
//   2. 結果を tier1 State に永続化する
//   3. 監査イベントを記録する
//
// tier1 アクセスは IK1s0SdkAdapter 越しに行うため、本クラスは HTTP / gRPC を意識しない。

using System.Text.Json;
using K1s0.Legacy.PayrollWrapper.Legacy;
using K1s0.Legacy.PayrollWrapper.Models;

namespace K1s0.Legacy.PayrollWrapper.Services;

public sealed class PayrollService
{
    // tier1 アクセス境界。
    private readonly IK1s0SdkAdapter _sdk;
    // State store 名（appsettings から DI で注入される想定、リリース時点 minimum は固定）。
    private const string StateStore = "payroll-store";

    public PayrollService(IK1s0SdkAdapter sdk)
    {
        _sdk = sdk;
    }

    // 給与計算 → State 保存 → 監査記録 を 1 トランザクションとして実行する。
    public async Task<PayrollRecord> CalculateAndPersistAsync(PayrollCalculationRequest req, CancellationToken ct = default)
    {
        // 既存 .NET Framework ロジックを純同期で呼ぶ（async/await は wrapper 層で被せる）。
        var net = PayrollLegacy.CalculateNetPay(req.MonthlyGross, req.Deductions);
        // 結果レコードを組み立てる。
        var record = new PayrollRecord(
            EmployeeId: req.EmployeeId,
            TargetMonth: req.TargetMonth,
            NetPay: net,
            CalculatedAtUtc: DateTimeOffset.UtcNow.ToString("yyyy-MM-ddTHH:mm:ssZ"),
            LogicVersion: PayrollLegacy.LogicVersion);
        // tier1 State に保存する（key は employee/{id}/{month} で uniq）。
        var key = $"{req.EmployeeId}/{req.TargetMonth}";
        var json = JsonSerializer.Serialize(record);
        await _sdk.SaveStateAsync(StateStore, key, json, ct).ConfigureAwait(false);
        // 監査イベントを記録する（actor は wrapper 主体名、SUCCESS は確定後）。
        await _sdk.RecordAuditAsync(
            actor: "payroll-wrapper",
            action: "PAYROLL_CALCULATE",
            resource: $"payroll/{key}",
            outcome: "SUCCESS",
            ct: ct).ConfigureAwait(false);
        return record;
    }
}
