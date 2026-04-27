// 本ファイルは Android プラットフォームの Application クラス。
// MauiApplication を継承し、`MauiProgram.CreateMauiApp()` を Android から起動する責務を持つ。

using Android.App;
using Android.Runtime;

namespace K1s0.Native.Admin;

[Application]
public class MainApplication : MauiApplication
{
    public MainApplication(IntPtr handle, JniHandleOwnership ownership) : base(handle, ownership) { }

    protected override MauiApp CreateMauiApp() => MauiProgram.CreateMauiApp();
}
