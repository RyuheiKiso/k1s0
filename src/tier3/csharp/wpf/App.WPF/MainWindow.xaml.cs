// k1s0 tier3 WPF メインウィンドウ コードビハインド
// tier3 4 layer state (ServerTruth / OptimisticLocal / PendingQueue / Draft) を消費する
using System.Windows;
// ViewModel 名前空間をインポートする
using K1s0.Tier3.Wpf.ViewModels;

namespace K1s0.Tier3.Wpf
{
    // MainWindow: WPF メインウィンドウクラス
    public partial class MainWindow : Window
    {
        // MainWindow コンストラクタ: ViewModel を注入して DataContext に設定する
        public MainWindow()
        {
            // XAML コンポーネントを初期化する
            InitializeComponent();
            // DataContext に MainViewModel を設定する（tier3 4 layer state を消費）
            DataContext = new MainViewModel();
        }
    }
}
