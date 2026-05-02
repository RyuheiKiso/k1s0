// 既存 .NET Framework 4.8 給与計算ロジックの擬似シム。
//
// 実運用では、第三者ライブラリ (PayrollLib.dll など) を third_party/ または
// 社内 NuGet server に登録し、本ファイルを削除して以下を csproj に追加する:
//
//   <ItemGroup>
//     <Reference Include="PayrollLib">
//       <HintPath>..\..\..\..\third_party\PayrollLib\PayrollLib.dll</HintPath>
//       <Private>True</Private>
//     </Reference>
//   </ItemGroup>
//
// マネージド相互運用 (.NET Standard 2.0+ 互換 DLL) であれば .NET 8 から
// そのまま呼び出せる。本シムは外部 DLL の代理として、.NET Framework 由来
// の同期 API を simulate する（async/await 化はラッパー層で被せる）。

namespace K1s0.Legacy.PayrollWrapper.Legacy;

// 既存ライブラリと同形のクラス（namespace は実際の DLL に合わせて名称固定する想定）。
public static class PayrollLegacy
{
    // 擬似ロジックのバージョン（実運用では DLL の AssemblyVersion を引く）。
    public const string LogicVersion = "legacy-1.0.0";

    // 既存 .NET Framework 由来の純同期 API（async ではない点が wrapper 化の動機）。
    // monthlyGross - deductions を 1 円単位で切り捨てる（マイナスは 0 にクランプ）。
    public static decimal CalculateNetPay(decimal monthlyGross, decimal deductions)
    {
        var net = monthlyGross - deductions;
        if (net < 0)
        {
            return 0m;
        }
        return decimal.Truncate(net);
    }
}
