# `K1s0.Legacy.PayrollWrapper`

`tier3/legacy-wrap/wrappers/` 配下の参照実装。既存 .NET Framework 4.8 の給与計算ライブラリを .NET 8 でラップし、Linux container として k1s0 基盤上で動作させる事例である。Sidecar パターン（Windows Node 必須）からの段階移行先として位置付ける。

docs 正典:

- `docs/02_構想設計/adr/ADR-MIG-001-net-framework-sidecar.md`
- `docs/05_実装/00_ディレクトリ設計/40_tier3レイアウト/05_レガシーラップ配置.md`
- `src/tier3/legacy-wrap/migration-guide/steps/03_framework_to_net8.md`

## なぜ Wrapper パターンか

Sidecar パターンは既存 .NET Framework プロセスをそのまま動かせる点で導入コストが低いが、Windows Node 必須（`mcr.microsoft.com/dotnet/framework/aspnet:4.8-windowsservercore-ltsc2022`）であり、Linux Node のみで構成された k1s0 クラスタには持ち込めない。Wrapper パターンは既存 .dll を .NET 8 から `<Reference>` で取り込み、Linux container として再パッケージするため、Windows Node 依存を解消できる。代償として、`System.Web` / `WCF Server` / `WPF` / `Forms` といった .NET Framework 固有 API に依存している既存資産は再実装が必要になる（`migration-guide/steps/03_framework_to_net8.md` 参照）。

## レイアウト

```text
K1s0.Legacy.PayrollWrapper/
├── K1s0.Legacy.PayrollWrapper.csproj   # SDK-style (Microsoft.NET.Sdk.Web) / net8.0
├── Dockerfile                          # Linux multi-stage / nonroot
├── README.md                           # 本ファイル
└── src/
    ├── Program.cs                      # ASP.NET Core minimal hosting + DI
    ├── Controllers/PayrollController.cs # POST /payroll/calculate, GET /payroll/healthz
    ├── Services/
    │   ├── IK1s0SdkAdapter.cs          # tier1 アクセス境界 (BFF or SDK gRPC で切替可)
    │   ├── K1s0SdkAdapter.cs           # 既定実装 (BFF REST 経由、HttpClient ベース)
    │   └── PayrollService.cs           # Legacy ロジック呼出 + State 保存 + 監査記録
    ├── Models/PayrollRecord.cs         # 入出力レコード (Request / Response)
    └── Legacy/PayrollLegacy.cs         # 既存 .dll の擬似シム（実運用では削除して <Reference> 化）
```

## 既存 .dll の取り込み（実運用時）

本リポジトリには「既存ライブラリの擬似シム」として `Legacy/PayrollLegacy.cs` を含めているが、実運用では以下の差し替えを行う。

1. 既存 `.dll`（例: `PayrollLib.dll`、`.NET Standard 2.0+` 互換）を `third_party/PayrollLib/` または社内 NuGet server に配置する。
2. `Legacy/PayrollLegacy.cs` を削除する。
3. csproj に以下を追加する:

   ```xml
   <ItemGroup>
     <Reference Include="PayrollLib">
       <HintPath>..\..\..\..\third_party\PayrollLib\PayrollLib.dll</HintPath>
       <Private>True</Private>
     </Reference>
   </ItemGroup>
   ```

4. `PayrollService.cs` の `using K1s0.Legacy.PayrollWrapper.Legacy;` を `using PayrollLib;` などに置き換え、`PayrollLegacy.CalculateNetPay(...)` を実 DLL の API 呼出に差し替える。

`Reference Include` のマネージド相互運用が成立する条件は、当該 DLL が `.NET Standard 2.0+` か `net48` のうち `.NET 8` から呼出可能な API のみを露出している場合である。`System.Web` などフレームワーク固有 API に依存する場合は再実装またはインタロップ層が必要になる。

## ビルドと起動

```bash
# csproj をローカル復元 + ビルド
dotnet restore K1s0.Legacy.PayrollWrapper.csproj
dotnet build K1s0.Legacy.PayrollWrapper.csproj -c Release

# 起動（環境変数で BFF URL とテナントを指定）
K1S0_BFF_URL="http://localhost:8080" \
K1S0_TENANT_ID="tenant-dev" \
  dotnet run --project K1s0.Legacy.PayrollWrapper.csproj

# 動作確認
curl -X POST http://localhost:5000/payroll/calculate \
  -H 'Content-Type: application/json' \
  -d '{
    "employeeId": "E-001",
    "targetMonth": "2026-04",
    "monthlyGross": 350000,
    "deductions": 65000
  }'
```

## Container ビルド

```bash
docker build -f Dockerfile -t ghcr.io/k1s0/legacy-payroll-wrapper:dev .
```

`Dockerfile` は multi-stage（`mcr.microsoft.com/dotnet/sdk:8.0` → `aspnet:8.0`）の Linux container で、runtime stage は nonroot (`USER 1000:1000`) にする。

## tier1 アクセスの差し替え

リリース時点 minimum では BFF REST (`POST /api/state/save`, `POST /api/audit/record`) を `HttpClient` で叩く実装 (`K1s0SdkAdapter`) を既定とする。`IK1s0SdkAdapter` を境界として切ってあるため、リリース時点 で K1s0.Sdk.Grpc を使う `K1s0GrpcSdkAdapter` 実装に差し替えれば、HTTP / gRPC の選択は wrapper 利用者側のオプションになる。

## 関連 ID

- IMP-DIR-T3-060（legacy-wrap 配置）
- ADR-MIG-001（.NET Framework sidecar）/ ADR-MIG-002（API Gateway）
- NFR-D-MIG-\* / 制約 8
