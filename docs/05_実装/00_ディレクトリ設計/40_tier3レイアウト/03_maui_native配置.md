# 03. MAUI Native 配置

本ファイルは `src/tier3/native/` 配下の .NET MAUI アプリ配置を確定する。iOS / Android / Windows / macOS のクロスプラットフォーム対応を前提とする。

## レイアウト

```
src/tier3/native/
├── README.md
├── Native.sln
├── Directory.Build.props
├── Directory.Packages.props
├── apps/
│   ├── K1s0.Native.Hub/
│   │   ├── K1s0.Native.Hub.csproj
│   │   ├── MauiProgram.cs
│   │   ├── App.xaml
│   │   ├── App.xaml.cs
│   │   ├── AppShell.xaml
│   │   ├── AppShell.xaml.cs
│   │   ├── MainPage.xaml
│   │   ├── MainPage.xaml.cs
│   │   ├── Pages/
│   │   ├── ViewModels/
│   │   ├── Services/               # 認証 / API クライアントラッパー
│   │   ├── Platforms/
│   │   │   ├── Android/
│   │   │   ├── iOS/
│   │   │   ├── MacCatalyst/
│   │   │   └── Windows/
│   │   ├── Resources/
│   │   │   ├── AppIcon/
│   │   │   ├── Splash/
│   │   │   ├── Images/
│   │   │   ├── Fonts/
│   │   │   ├── Styles/
│   │   │   └── Raw/
│   │   └── Properties/
│   └── K1s0.Native.Admin/          # Phase 1c 以降
│       └── ...
└── shared/
    └── K1s0.Native.Shared/
        ├── K1s0.Native.Shared.csproj
        ├── Controls/               # カスタム MAUI Control
        ├── Converters/
        ├── Behaviors/
        └── Extensions/
```

## 技術スタック

- **Framework**: .NET MAUI（`net8.0-ios` / `net8.0-android` / `net8.0-maccatalyst` / `net8.0-windows10.0.19041.0`）
  - `.csproj` の `<TargetFrameworks>` には最低 build (19041 = Windows 10 2004) を記述する。Windows 11（22000+）側の Runtime も同一 TFM で実行可能（バックワード互換）だが、Windows 11 限定 API を利用する場合は `<TargetFrameworks>...;net8.0-windows10.0.22621.0</TargetFrameworks>` を併記し、`#if WINDOWS10_0_22621_0_OR_GREATER` で条件分岐する
  - `<SupportedOSPlatformVersion>` は `10.0.19041.0`、`<TargetPlatformMinVersion>` で配信時の最低 OS を強制する
- **UI**: XAML + MVVM
- **MVVM**: CommunityToolkit.Mvvm
- **DI**: Microsoft.Extensions.DependencyInjection
- **認証**: `K1s0.Sdk.Auth`（Keycloak OIDC 連携）
- **ネットワーク**: HttpClient + Grpc.Net.Client（gRPC）
- **データ**: sqlite-net-pcl（ローカルキャッシュ）/ `K1s0.Sdk` で tier1 アクセス

## 対象プラットフォーム

Phase 1b では iOS + Android のみ。Phase 1c で Windows + macOS を追加。

- iOS: .NET 8 + Xcode 15
- Android: .NET 8 + AndroidX + API 34（Android 14）以上
- Windows: .NET 8 + WinAppSDK 1.5 以上
- macOS: .NET 8 + MacCatalyst

## MauiProgram.cs の雛形

```csharp
// MauiProgram.cs
//
// MAUI アプリのエントリポイント。DI 構築を担う
using CommunityToolkit.Maui;
using K1s0.Sdk;
using Microsoft.Extensions.Logging;

namespace K1s0.Native.Hub;

public static class MauiProgram
{
    // MAUI アプリのビルド構築
    public static MauiApp CreateMauiApp()
    {
        var builder = MauiApp.CreateBuilder();
        builder
            .UseMauiApp<App>()
            .UseMauiCommunityToolkit()
            .ConfigureFonts(fonts =>
            {
                fonts.AddFont("OpenSans-Regular.ttf", "OpenSansRegular");
                fonts.AddFont("OpenSans-Semibold.ttf", "OpenSansSemibold");
            });

        // k1s0 SDK クライアントを DI に登録
        // AddK1s0Sdk() は src/sdk/dotnet/src/K1s0.Sdk/ が提供する
        // IServiceCollection 拡張メソッド（K1s0.Sdk.Extensions.DependencyInjection 名前空間）
        builder.Services.AddK1s0Sdk(options =>
        {
            options.Endpoint = "https://api.k1s0.internal";
            options.AuthProvider = "keycloak";
        });

        // ViewModel を DI に登録
        builder.Services.AddTransient<ViewModels.MainViewModel>();

#if DEBUG
        builder.Logging.AddDebug();
#endif

        return builder.Build();
    }
}
```

## Platforms フォルダ

MAUI の各プラットフォーム固有コード（Android の `MainActivity.cs`、iOS の `AppDelegate.cs` 等）は `Platforms/` 配下に配置。この構造は MAUI 標準に従う。

## shared/ の位置付け

tier3 Native 内の共通コンポーネント（カスタム Control / Converter / Behavior）は `shared/K1s0.Native.Shared/` に集約する。複数アプリ（Hub / Admin）から参照される想定。

## NuGet パッケージ

`src/tier3/native/Directory.Packages.props` で tier2 と同じく Central Package Management を採用。主要パッケージ:

- `CommunityToolkit.Maui` / `CommunityToolkit.Mvvm`
- `Microsoft.Maui.Controls` / `Microsoft.Maui.Controls.Compatibility`
- `Microsoft.Extensions.DependencyInjection`
- `Grpc.Net.Client` / `Google.Protobuf`
- `K1s0.Sdk` / `K1s0.Sdk.Auth`
- `sqlite-net-pcl`
- `CommunityToolkit.Mvvm`
- `Sentry.Maui`（クラッシュログ収集、Phase 1c 以降）

## ビルド

```bash
# Android APK ビルド
dotnet publish apps/K1s0.Native.Hub/K1s0.Native.Hub.csproj \
    -f net8.0-android \
    -c Release

# iOS archive ビルド（macOS 環境必須）
dotnet publish apps/K1s0.Native.Hub/K1s0.Native.Hub.csproj \
    -f net8.0-ios \
    -c Release
```

## CI / CD

Phase 1b 以降で以下を整備。

- Android: GitHub Actions `macos-latest` または `ubuntu-latest` で APK ビルド、署名は internal distribution でのみ
- iOS: GitHub Actions `macos-latest` で archive ビルド、TestFlight 配布
- Windows: GitHub Actions `windows-latest` で MSIX ビルド
- macOS: GitHub Actions `macos-latest` で dmg ビルド

署名証明書の管理は Phase 1b で詳細設計。

## テスト戦略

- unit test: xUnit + NSubstitute で ViewModel / Service を検証
- UI test: MAUI UI Testing（Phase 1c 以降）
- 手動検証: 各プラットフォーム実機での動作確認

## 依存方向

- tier3 Native は `src/sdk/dotnet/` を介して tier1 にアクセス
- tier2 の .NET サービスを直接呼び出すことは禁止（tier2 は tier3 BFF 経由または tier1 経由で利用）
- 他 tier3 アプリ（Web など）の内部を参照することは禁止

## CODEOWNERS

```
/src/tier3/native/                              @k1s0/tier3-native
```

## スパースチェックアウト cone

- `tier3-native-dev` cone に `src/tier3/native/` + `src/sdk/dotnet/` を含む

## 対応 IMP-DIR ID

- IMP-DIR-T3-058（maui native 配置）

## 対応 ADR / DS-SW-COMP / 要件

- ADR-TIER1-003
- FR-\* / DX-GP-\* / NFR-G-PRV-\*（モバイル端末でのデータ保護）
