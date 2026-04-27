// メインページの ViewModel。
//
// CommunityToolkit.Mvvm の ObservableObject + ICommand 風 RelayCommand を使う。

using System.ComponentModel;
using System.Runtime.CompilerServices;
using System.Windows.Input;
using K1s0.Native.Hub.Services;

namespace K1s0.Native.Hub.ViewModels;

// MainViewModel は MainPage のデータバインディング先。
public sealed class MainViewModel : INotifyPropertyChanged
{
    // BFF 呼出。
    private readonly IK1s0Service _service;

    // 状態フィールド。
    private string _stateKey = "user/123";
    private string _fetchedValue = string.Empty;
    private string _statusMessage = "ready";
    private bool _isBusy;

    public event PropertyChangedEventHandler? PropertyChanged;

    public MainViewModel(IK1s0Service service)
    {
        _service = service;
        FetchStateCommand = new AsyncRelayCommand(ExecuteFetchAsync);
    }

    // バインドキー入力。
    public string StateKey
    {
        get => _stateKey;
        set => SetField(ref _stateKey, value);
    }

    // 取得結果。
    public string FetchedValue
    {
        get => _fetchedValue;
        private set => SetField(ref _fetchedValue, value);
    }

    // 状態メッセージ（busy / error）。
    public string StatusMessage
    {
        get => _statusMessage;
        private set => SetField(ref _statusMessage, value);
    }

    // ボタン disabled 用。
    public bool IsNotBusy => !_isBusy;

    // Fetch ボタンの ICommand。
    public ICommand FetchStateCommand { get; }

    private async Task ExecuteFetchAsync()
    {
        if (_isBusy)
        {
            return;
        }
        SetBusy(true, "fetching…");
        try
        {
            var value = await _service.GetStateAsync("postgres", StateKey);
            FetchedValue = value ?? "(not found)";
            StatusMessage = "ok";
        }
        catch (Exception ex)
        {
            FetchedValue = string.Empty;
            StatusMessage = $"error: {ex.Message}";
        }
        finally
        {
            SetBusy(false, StatusMessage);
        }
    }

    private void SetBusy(bool isBusy, string message)
    {
        _isBusy = isBusy;
        OnPropertyChanged(nameof(IsNotBusy));
        StatusMessage = message;
    }

    private void SetField<T>(ref T field, T value, [CallerMemberName] string? propertyName = null)
    {
        if (EqualityComparer<T>.Default.Equals(field, value))
        {
            return;
        }
        field = value;
        OnPropertyChanged(propertyName);
    }

    private void OnPropertyChanged([CallerMemberName] string? propertyName = null)
    {
        PropertyChanged?.Invoke(this, new PropertyChangedEventArgs(propertyName ?? string.Empty));
    }
}

// AsyncRelayCommand は CommunityToolkit.Mvvm.Input.AsyncRelayCommand の最小代替実装。
// テスト容易性のため自作（重依存をテスト project が継承しないように）。
internal sealed class AsyncRelayCommand : ICommand
{
    private readonly Func<Task> _execute;
    private bool _executing;

    public event EventHandler? CanExecuteChanged;

    public AsyncRelayCommand(Func<Task> execute)
    {
        _execute = execute;
    }

    public bool CanExecute(object? parameter) => !_executing;

    public async void Execute(object? parameter)
    {
        _executing = true;
        CanExecuteChanged?.Invoke(this, EventArgs.Empty);
        try
        {
            await _execute().ConfigureAwait(false);
        }
        finally
        {
            _executing = false;
            CanExecuteChanged?.Invoke(this, EventArgs.Empty);
        }
    }
}
