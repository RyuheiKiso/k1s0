// k1s0 tier3 WPF OrderViewModel
// 発注業務の 4 layer state（OL: 楽観的ローカル更新 / PQ: 保留キュー）を WPF DataBinding に公開する
using System.ComponentModel;
using System.Runtime.CompilerServices;

namespace K1s0.Tier3.Wpf.ViewModels
{
    // OrderViewModel: 発注業務 ViewModel（INotifyPropertyChanged 実装）
    public class OrderViewModel : INotifyPropertyChanged
    {
        // PropertyChanged: プロパティ変更通知イベント
        public event PropertyChangedEventHandler? PropertyChanged;

        // _orderId: 発注 ID のバッキングフィールド
        private string _orderId = string.Empty;

        // OrderId: 発注 ID プロパティ（OptimisticLocal layer で管理する）
        public string OrderId
        {
            // OrderId の get: バッキングフィールドを返す
            get => _orderId;
            // OrderId の set: 変更があれば PropertyChanged を発火する
            set
            {
                // 値が変わっていない場合は何もしない
                if (_orderId == value) return;
                // バッキングフィールドを更新する
                _orderId = value;
                // プロパティ変更を通知する
                OnPropertyChanged();
            }
        }

        // _isPending: PendingQueue に入っているかどうかのフラグ
        private bool _isPending;

        // IsPending: PendingQueue layer の状態フラグ
        public bool IsPending
        {
            // IsPending の get: バッキングフィールドを返す
            get => _isPending;
            // IsPending の set: 変更があれば PropertyChanged を発火する
            set
            {
                // 値が変わっていない場合は何もしない
                if (_isPending == value) return;
                // バッキングフィールドを更新する
                _isPending = value;
                // プロパティ変更を通知する
                OnPropertyChanged();
            }
        }

        // OnPropertyChanged: CallerMemberName 属性でプロパティ名を自動取得して通知する
        protected void OnPropertyChanged([CallerMemberName] string? propertyName = null)
        {
            // PropertyChanged イベントを発火する
            PropertyChanged?.Invoke(this, new PropertyChangedEventArgs(propertyName));
        }
    }
}
