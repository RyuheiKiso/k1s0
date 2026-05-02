// 給与計算の入出力モデル。
//
// docs 正典:
//   docs/05_実装/00_ディレクトリ設計/40_tier3レイアウト/05_レガシーラップ配置.md

namespace K1s0.Legacy.PayrollWrapper.Models;

// 計算入力（API request body）。
public sealed record PayrollCalculationRequest(
    // 従業員 ID（既存システムの主キー）。
    string EmployeeId,
    // 計算対象月（YYYY-MM 形式）。
    string TargetMonth,
    // 月給（円、税引前）。
    decimal MonthlyGross,
    // 控除合計（健康保険 / 厚生年金 / 所得税 / 住民税）。
    decimal Deductions);

// 計算結果（API response body）。
public sealed record PayrollRecord(
    // 従業員 ID。
    string EmployeeId,
    // 計算対象月。
    string TargetMonth,
    // 手取り額。
    decimal NetPay,
    // 計算実行時刻（UTC, RFC3339）。
    string CalculatedAtUtc,
    // 計算ロジックのバージョン（既存 .dll の version などをそのまま転記）。
    string LogicVersion);
