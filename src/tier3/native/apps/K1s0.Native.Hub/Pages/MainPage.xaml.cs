// MAUI MainPage。
//
// docs 正典:
//   docs/05_実装/00_ディレクトリ設計/40_tier3レイアウト/03_maui_native配置.md

using K1s0.Native.Hub.ViewModels;

namespace K1s0.Native.Hub.Pages;

public partial class MainPage : ContentPage
{
    // ViewModel は DI 経由で注入する。
    public MainPage(MainViewModel viewModel)
    {
        InitializeComponent();
        BindingContext = viewModel;
    }
}
