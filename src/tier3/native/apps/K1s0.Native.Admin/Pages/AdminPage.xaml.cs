// MAUI AdminPage。AdminViewModel を DI 経由で受け取りバインドする。

using K1s0.Native.Admin.ViewModels;

namespace K1s0.Native.Admin.Pages;

public partial class AdminPage : ContentPage
{
    // ViewModel は DI 経由で注入する。
    public AdminPage(AdminViewModel viewModel)
    {
        InitializeComponent();
        BindingContext = viewModel;
    }
}
