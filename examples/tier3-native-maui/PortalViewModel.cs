// 本ファイルは tier3-native-maui の代表的な ViewModel（MVVM パターン）。
// MAUI UI（XAML View）から Binding 経由で参照され、tier1 State API への
// 操作を K1s0.Sdk.Grpc 経由で行う。

using System.ComponentModel;
using K1s0.Sdk;

namespace K1s0.Examples.NativeMaui;

// PortalViewModel は MAUI の各 Page から Binding する状態を保持する。
public sealed class PortalViewModel : INotifyPropertyChanged
{
    // k1s0 SDK Client（DI で注入される、テスト時は mock）。
    private readonly K1s0Client _client;

    // 直近に取得した値（UI binding 対象）。
    private string _currentValue = string.Empty;

    // INotifyPropertyChanged の event。
    public event PropertyChangedEventHandler? PropertyChanged;

    public PortalViewModel(K1s0Client client)
    {
        _client = client;
    }

    // UI binding プロパティ（変更時に通知）。
    public string CurrentValue
    {
        get => _currentValue;
        set
        {
            if (_currentValue != value)
            {
                _currentValue = value;
                PropertyChanged?.Invoke(this, new PropertyChangedEventArgs(nameof(CurrentValue)));
            }
        }
    }

    // ボタン押下等から呼ばれる、tier1 State.Get の実行 method。
    public async Task FetchAsync(string store, string key, CancellationToken ct = default)
    {
        // SDK 経由で State API を呼ぶ。
        var result = await _client.State.GetAsync(store, key, ct);
        // 未存在時は "(not found)" 表示。
        if (result is null)
        {
            CurrentValue = "(not found)";
            return;
        }
        // 取得成功時は data を UTF-8 として表示する。
        CurrentValue = System.Text.Encoding.UTF8.GetString(result.Value.Data);
    }
}
