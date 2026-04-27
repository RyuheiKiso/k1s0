// MAUI Application 本体。
//
// docs 正典:
//   docs/05_実装/00_ディレクトリ設計/40_tier3レイアウト/03_maui_native配置.md

namespace K1s0.Native.Hub;

public partial class App : Application
{
    public App()
    {
        InitializeComponent();
        // AppShell を MainPage に設定する（リリース時点 minimum）。
        MainPage = new AppShell();
    }
}
