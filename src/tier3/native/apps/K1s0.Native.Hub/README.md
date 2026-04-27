# K1s0.Native.Hub

tier3 native の配信ハブ MAUI アプリ。

## エントリ

- `MauiProgram.cs` で DI 構築
- `MainPage.xaml` (Pages/) + `MainViewModel.cs` (ViewModels/) で State Get デモ画面

## ビルド

```bash
dotnet workload install maui
dotnet build apps/K1s0.Native.Hub/K1s0.Native.Hub.csproj -c Release
dotnet publish apps/K1s0.Native.Hub/K1s0.Native.Hub.csproj -f net8.0-android -c Release
```

## 拡張ポイント

- リリース時点 では Android + iOS のみ。リリース時点 で Windows / MacCatalyst を `<TargetFrameworks>` に追加する。
- リリース時点 では BFF を HttpClient で叩く。リリース時点 で K1s0.Sdk.Grpc 経由の gRPC 直接呼出を選択肢に追加（IK1s0Service 越しのため置換可）。
