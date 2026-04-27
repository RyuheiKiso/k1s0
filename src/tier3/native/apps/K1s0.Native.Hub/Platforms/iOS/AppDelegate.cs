// iOS プラットフォーム固有エントリ。

using Foundation;

namespace K1s0.Native.Hub;

[Register("AppDelegate")]
public class AppDelegate : MauiUIApplicationDelegate
{
    protected override MauiApp CreateMauiApp() => MauiProgram.CreateMauiApp();
}
