// k1s0 tier3 WPF コンフリクト解決 View コードビハインド
// UserControl の InitializeComponent を呼び出す最小実装
using System.Windows.Controls;

namespace K1s0.Tier3.Wpf.Views
{
    // ConflictView: コンフリクト解決画面 UserControl（4 subtype 分岐 UI）
    public partial class ConflictView : UserControl
    {
        // ConflictView コンストラクタ: XAML コンポーネントを初期化する
        public ConflictView()
        {
            // XAML コンポーネントを初期化する
            InitializeComponent();
        }
    }
}
