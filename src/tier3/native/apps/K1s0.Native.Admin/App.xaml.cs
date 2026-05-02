// 本ファイルは K1s0.Native.Admin（MAUI 管理者アプリ）のエントリ Application クラス。
// AppShell を MainPage に設定する（リリース時点 minimum、Hub と対称構成）。

namespace K1s0.Native.Admin;

public partial class App : Application
{
    public App()
    {
        InitializeComponent();
        // AppShell を MainPage に設定する（リリース時点 minimum）。
        MainPage = new AppShell();
    }
}
