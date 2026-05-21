// k1s0 tier3 WPF MainViewModel
// tier3 4 layer state (ST/OL/PQ/DR) を WPF DataBinding に公開する
// 11_クライアント状態適合仕様.md §4 layer SDK C# wrapper
using System.ComponentModel;
using System.Runtime.CompilerServices;
using System.Windows.Input;
// tier3 State Library からインポートする（tier1 直接 import 禁止規律に準拠）
using K1s0.Tier3.State;

namespace K1s0.Tier3.Wpf.ViewModels
{
    // MainViewModel: INotifyPropertyChanged 実装の ViewModel
    public class MainViewModel : INotifyPropertyChanged
    {
        // PropertyChanged: プロパティ変更通知イベント（INotifyPropertyChanged 実装）
        public event PropertyChangedEventHandler? PropertyChanged;

        // _currentView: 現在表示している子 View のバッキングフィールド
        private object? _currentView;

        // CurrentView: 現在の画面コンテンツ（ContentControl.Content にバインドされる）
        public object? CurrentView
        {
            // CurrentView の get: バッキングフィールドを返す
            get => _currentView;
            // CurrentView の set: 値が変わった場合は PropertyChanged を発火する
            set
            {
                // 値が変わっていない場合は何もしない
                if (Equals(_currentView, value)) return;
                // バッキングフィールドを更新する
                _currentView = value;
                // プロパティ変更を通知する
                OnPropertyChanged();
            }
        }

        // _state: tier3 4 layer state（ServerTruth / OptimisticLocal / PendingQueue / Draft）
        private readonly ClientState _state;

        // MainViewModel コンストラクタ: AppState を初期化する
        public MainViewModel()
        {
            // ClientState を初期値で初期化する（tier3 State.cs の ClientState を使用）
            _state = ClientState.Initial();
            // NavigateToPlantCommand を初期化する
            NavigateToPlantCommand = new RelayCommand(_ => NavigateToPlant());
            // NavigateToConflictsCommand を初期化する
            NavigateToConflictsCommand = new RelayCommand(_ => NavigateToConflicts());
            // 初期画面として PlantView を設定する
            NavigateToPlant();
        }

        // NavigateToPlantCommand: 工場現場画面へのナビゲーションコマンド
        public ICommand NavigateToPlantCommand { get; }

        // NavigateToConflictsCommand: コンフリクト解決画面へのナビゲーションコマンド
        public ICommand NavigateToConflictsCommand { get; }

        // NavigateToPlant: 工場現場画面に切り替える
        private void NavigateToPlant()
        {
            // PlantView を現在の画面として設定する
            CurrentView = new Views.PlantView();
        }

        // NavigateToConflicts: コンフリクト解決画面に切り替える
        private void NavigateToConflicts()
        {
            // ConflictView を現在の画面として設定する
            CurrentView = new Views.ConflictView();
        }

        // OnPropertyChanged: CallerMemberName 属性でプロパティ名を自動取得して通知する
        protected void OnPropertyChanged([CallerMemberName] string? propertyName = null)
        {
            // PropertyChanged イベントを発火する
            PropertyChanged?.Invoke(this, new PropertyChangedEventArgs(propertyName));
        }
    }

    // RelayCommand: ICommand の簡易実装
    internal sealed class RelayCommand : ICommand
    {
        // _execute: 実行するアクション
        private readonly System.Action<object?> _execute;

        // RelayCommand コンストラクタ: 実行アクションを受け取る
        public RelayCommand(System.Action<object?> execute)
        {
            // アクションを設定する
            _execute = execute;
        }

        // CanExecuteChanged: 常に変更なし（simplified）
        public event System.EventHandler? CanExecuteChanged;

        // CanExecute: 常に true を返す
        public bool CanExecute(object? parameter) => true;

        // Execute: 設定されたアクションを実行する
        public void Execute(object? parameter) => _execute(parameter);
    }
}
