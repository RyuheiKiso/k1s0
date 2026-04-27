# k1s0 .NET 依存方向 Roslyn Analyzer

`tier3 → tier2 → sdk → tier1 → infra` 一方向ルールを .NET 側で個別 `using` 文レベルで強制する Roslyn Analyzer。NuGet package として全 `.sln`（`Sdk.sln` / `Tier2.sln` / `Tier3Native.sln` / `LegacyWrap.sln`）に配布する（[`docs/05_実装/00_ディレクトリ設計/10_ルートレイアウト/05_依存方向ルール.md`](../../../docs/05_実装/00_ディレクトリ設計/10_ルートレイアウト/05_依存方向ルール.md) 「.NET: NetArchTest + Roslyn Analyzer」節）。

## 役割

- `<ProjectReference>` 経由の違反は NetArchTest 系 xUnit テストで検出
- 動的型解決 / reflection 経由でも違反する `using K1s0.Tier2.*` / `using K1s0.Tier3.*` のような個別 import は本 Analyzer が build 警告として検出
- 違反が出た場合は CI で `dotnet build /p:TreatWarningsAsErrors=true` と組合せて build を fail させる

## 診断 ID

`K1S0DEPDIR0001`〜（[`docs/05_実装/00_ディレクトリ設計/10_ルートレイアウト/05_依存方向ルール.md`](../../../docs/05_実装/00_ディレクトリ設計/10_ルートレイアウト/05_依存方向ルール.md)）。

| ID | 検出内容 | デフォルト Severity |
|---|---|---|
| `K1S0DEPDIR0001` | SDK アセンブリから `K1s0.Tier2.*` の参照 | Error |
| `K1S0DEPDIR0002` | SDK アセンブリから `K1s0.Tier3.*` の参照 | Error |
| `K1S0DEPDIR0003` | `K1s0.Tier2.*` から `K1s0.Tier3.*` の参照 | Error |
| `K1S0DEPDIR0004` | `K1s0.Tier1.*` から `K1s0.Tier2.*` / `K1s0.Tier3.*` / `K1s0.Sdk.*` の参照 | Error |

アセンブリ判定は `Compilation.AssemblyName` に対して prefix で比較する（`K1s0.Sdk.*` / `K1s0.Tier1.*` / `K1s0.Tier2.*` / `K1s0.Tier3.*`）。

## ディレクトリ構造

```text
src/platform/analyzer/
├── README.md                                                # 本ファイル
├── Directory.Build.props                                    # 共通プロパティ（langversion / nullable）
├── K1s0.DependencyDirection.Analyzer.sln                    # ソリューション
├── src/
│   ├── K1s0.DependencyDirection.Analyzer/
│   │   ├── K1s0.DependencyDirection.Analyzer.csproj         # Roslyn DiagnosticAnalyzer ライブラリ
│   │   ├── DependencyDirectionAnalyzer.cs                   # アナライザ本体
│   │   ├── Diagnostics.cs                                   # 4 件の DiagnosticDescriptor 定義
│   │   └── AssemblyTier.cs                                  # アセンブリ → tier 判定の共通ヘルパ
│   └── K1s0.DependencyDirection.Analyzer.Package/
│       └── K1s0.DependencyDirection.Analyzer.Package.csproj # 上記を NuGet 化（analyzers/ に同梱）
└── tests/
    └── K1s0.DependencyDirection.Analyzer.Tests/
        ├── K1s0.DependencyDirection.Analyzer.Tests.csproj   # xUnit + Microsoft.CodeAnalysis.Testing
        └── DependencyDirectionAnalyzerTests.cs              # 違反例 / 非違反例の golden test
```

## ビルド

```bash
cd src/platform/analyzer
dotnet build K1s0.DependencyDirection.Analyzer.sln -c Release
dotnet pack src/K1s0.DependencyDirection.Analyzer.Package/K1s0.DependencyDirection.Analyzer.Package.csproj
```

NuGet package が `src/K1s0.DependencyDirection.Analyzer.Package/bin/Release/` に出る。各 `.sln` の `Directory.Packages.props` で `<PackageReference Include="K1s0.DependencyDirection.Analyzer" PrivateAssets="all" />` として参照する。

## テスト

```bash
dotnet test tests/K1s0.DependencyDirection.Analyzer.Tests/
```

`Microsoft.CodeAnalysis.Testing` の `CSharpAnalyzerTest` で「違反 source code → 期待 diagnostic」「非違反 source code → diagnostic なし」を golden 検証する。
