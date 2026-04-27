// 本ファイルは K1s0.Native.Admin（MAUI 管理者アプリ）のエントリ Application クラス。
// XAML を伴わない最小実装で、ウィンドウ初期化と暫定 ContentPage の組み立てを担う。

namespace K1s0.Native.Admin;

public partial class App : Application
{
    public App()
    {
        InitializeComponent();
        MainPage = new ContentPage
        {
            Title = "k1s0 Admin",
            Content = new VerticalStackLayout
            {
                Padding = new Thickness(20),
                Spacing = 12,
                Children =
                {
                    new Label { Text = "k1s0 Admin", FontSize = 28, FontAttributes = FontAttributes.Bold },
                    new Label { Text = "管理者向け機能はリリース時点 で順次実装します。" },
                },
            },
        };
    }
}
