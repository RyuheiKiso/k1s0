// MAUI app のエントリポイント。DI 構築を担う。
//
// docs 正典:
//   docs/05_実装/00_ディレクトリ設計/40_tier3レイアウト/03_maui_native配置.md

using CommunityToolkit.Maui;
using K1s0.Native.Hub.Services;
using K1s0.Native.Hub.ViewModels;
using K1s0.Native.Hub.Pages;
using Microsoft.Extensions.Logging;

namespace K1s0.Native.Hub;

public static class MauiProgram
{
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

        // BFF 呼出ラッパー（リリース時点 では HttpClient + 環境設定からの URL のみ）。
        builder.Services.AddSingleton<IK1s0Service, K1s0Service>();

        // ViewModel と Page を DI に登録する。
        builder.Services.AddTransient<MainViewModel>();
        builder.Services.AddTransient<MainPage>();

#if DEBUG
        builder.Logging.AddDebug();
#endif

        return builder.Build();
    }
}
