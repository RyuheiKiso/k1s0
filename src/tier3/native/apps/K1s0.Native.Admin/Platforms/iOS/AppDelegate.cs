// 本ファイルは iOS プラットフォームの AppDelegate 定義。
// MauiUIApplicationDelegate を継承し、`MauiProgram.CreateMauiApp()` を iOS から起動する。

using Foundation;

namespace K1s0.Native.Admin;

[Register("AppDelegate")]
public class AppDelegate : MauiUIApplicationDelegate
{
    protected override MauiApp CreateMauiApp() => MauiProgram.CreateMauiApp();
}
