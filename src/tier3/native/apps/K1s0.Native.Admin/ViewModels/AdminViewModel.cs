// AdminPage の ViewModel。
//
// CommunityToolkit.Mvvm の重依存を test project に持ち込まないため、
// Hub/MainViewModel と同じく ObservableObject + AsyncRelayCommand を自作する。
//
// 主機能:
//   直近 N 時間の監査イベントを QueryAuditAsync で取得し、Items にバインドする。

using System.Collections.ObjectModel;
using System.ComponentModel;
using System.Runtime.CompilerServices;
using System.Windows.Input;
using K1s0.Native.Admin.Services;

namespace K1s0.Native.Admin.ViewModels;

// AdminViewModel は AdminPage のデータバインディング先。
public sealed class AdminViewModel : INotifyPropertyChanged
{
    // BFF 呼出。
    private readonly IK1s0Service _service;

    // 状態フィールド。
    private int _hours = 24;
    private int _limit = 50;
    private string _statusMessage = "ready";
    private bool _isBusy;

    public event PropertyChangedEventHandler? PropertyChanged;

    public AdminViewModel(IK1s0Service service)
    {
        _service = service;
        Items = new ObservableCollection<AuditEvent>();
        QueryCommand = new AsyncRelayCommand(ExecuteQueryAsync);
    }

    // 取得対象の時間幅（直近 N 時間）。
    public int Hours
    {
        get => _hours;
        set => SetField(ref _hours, value);
    }

    // 取得最大件数。
    public int Limit
    {
        get => _limit;
        set => SetField(ref _limit, value);
    }

    // 結果リスト（CollectionView にバインド）。
    public ObservableCollection<AuditEvent> Items { get; }

    // 状態メッセージ（busy / error）。
    public string StatusMessage
    {
        get => _statusMessage;
        private set => SetField(ref _statusMessage, value);
    }

    // ボタン disabled 用。
    public bool IsNotBusy => !_isBusy;

    // Query ボタンの ICommand。
    public ICommand QueryCommand { get; }

    // テスト用: ExecuteQueryAsync を直接呼べるようにする。
    public Task ExecuteQueryForTestAsync() => ExecuteQueryAsync();

    private async Task ExecuteQueryAsync()
    {
        if (_isBusy)
        {
            return;
        }
        SetBusy(true, "querying…");
        try
        {
            var events = await _service.QueryAuditAsync(_hours, _limit);
            Items.Clear();
            foreach (var e in events)
            {
                Items.Add(e);
            }
            StatusMessage = $"ok ({events.Count} events)";
        }
        catch (Exception ex)
        {
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
