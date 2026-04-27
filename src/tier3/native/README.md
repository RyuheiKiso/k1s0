# tier3 native (.NET MAUI)

iOS / Android / Windows / macOS のクロスプラットフォーム向け MAUI アプリ群。

## docs 正典

- 配置: `docs/05_実装/00_ディレクトリ設計/40_tier3レイアウト/03_maui_native配置.md`
- 全体: `docs/05_実装/00_ディレクトリ設計/40_tier3レイアウト/01_tier3全体配置.md`

## レイアウト

```text
src/tier3/native/
├── README.md
├── Native.sln
├── Directory.Build.props
├── Directory.Packages.props
├── apps/
│   ├── K1s0.Native.Hub/         # 配信ハブ（リリース時点 主アプリ）
│   └── K1s0.Native.Admin/       # 管理アプリ（採用後の運用拡大時 着手）
└── shared/
    └── K1s0.Native.Shared/      # 共通 Controls / Converters / Behaviors
```

## 対象プラットフォーム

リリース時点 では `<TargetFrameworks>` に net8.0-ios / net8.0-android のみ含める想定。リリース時点 で `net8.0-windows10.0.19041.0` / `net8.0-maccatalyst` を追加する。

## ビルド

```bash
# 全プラットフォーム build（MAUI workload インストール必須）
dotnet workload install maui
dotnet build Native.sln -c Release

# Android のみ。
dotnet publish apps/K1s0.Native.Hub/K1s0.Native.Hub.csproj -f net8.0-android -c Release

# iOS（macOS 環境必須）
dotnet publish apps/K1s0.Native.Hub/K1s0.Native.Hub.csproj -f net8.0-ios -c Release
```

## テスト

ViewModel / Service の unit test は `apps/K1s0.Native.Hub.Tests/` に xUnit + NSubstitute で配置する。MAUI UI test は採用後の運用拡大時 で追加。

## 関連 ID

- IMP-DIR-T3-058
- ADR-TIER1-003
