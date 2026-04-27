// Android Application エントリポイント。

using Android.App;
using Android.Runtime;

namespace K1s0.Native.Hub;

[Application]
public class MainApplication : MauiApplication
{
    public MainApplication(IntPtr handle, JniHandleOwnership ownership) : base(handle, ownership) { }

    protected override MauiApp CreateMauiApp() => MauiProgram.CreateMauiApp();
}
