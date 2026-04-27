// 本ファイルは Android プラットフォームの起動 Activity 定義。
// MAUI の MauiAppCompatActivity を継承し、SplashTheme と画面回転設定を Android に伝える。

using Android.App;
using Android.Content.PM;

namespace K1s0.Native.Admin;

[Activity(Theme = "@style/Maui.SplashTheme", MainLauncher = true, ConfigurationChanges = ConfigChanges.ScreenSize | ConfigChanges.Orientation | ConfigChanges.UiMode | ConfigChanges.ScreenLayout | ConfigChanges.SmallestScreenSize | ConfigChanges.Density)]
public class MainActivity : MauiAppCompatActivity
{
}
