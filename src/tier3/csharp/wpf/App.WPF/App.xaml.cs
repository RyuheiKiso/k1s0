// k1s0 tier3 WPF App コードビハインド
// Application クラスを継承してアプリケーション起動ロジックを実装する
using System.Windows;
// OTel SDK をインポートする（observability 初期化に使用する）
using System.Diagnostics;

namespace K1s0.Tier3.Wpf
{
    // App: WPF Application を継承した k1s0 tier3 デスクトップアプリのエントリポイント
    public partial class App : Application
    {
        // OnStartup: アプリケーション起動時の初期化処理
        protected override void OnStartup(StartupEventArgs e)
        {
            // 基底クラスの OnStartup を呼び出す
            base.OnStartup(e);
            // アクティビティソースを初期化する（OTel トレーシング用）
            InitializeActivitySource();
            // i18n を初期化する（tier3 packages/i18n と等価な初期化）
            InitializeLocalization();
        }

        // InitializeActivitySource: OTel ActivitySource を初期化して tier3_ext SemConv に準拠させる
        private static void InitializeActivitySource()
        {
            // tier3 WPF App の ActivitySource を作成する（name は tier3_ext SemConv に準拠）
            var activitySource = new ActivitySource("k1s0.tier3.wpf", version: "1.0.0");
            // デバッグ用に ActivitySource 名をトレースする
            Trace.WriteLine($"ActivitySource initialized: {activitySource.Name}");
        }

        // InitializeLocalization: リソース辞書からロケールを初期化する
        private static void InitializeLocalization()
        {
            // システムロケールに基づいてリソース辞書を選択する
            var culture = System.Globalization.CultureInfo.CurrentUICulture;
            // en-US の場合は英語リソースを追加ロードする
            if (culture.Name.StartsWith("en", System.StringComparison.OrdinalIgnoreCase))
            {
                // 英語リソース辞書を動的にマージする
                var enDict = new System.Windows.ResourceDictionary
                {
                    // 英語リソース URI を設定する
                    Source = new System.Uri("Resources/en-US.xaml", System.UriKind.Relative),
                };
                // Application.Resources にマージする
                Current.Resources.MergedDictionaries.Add(enDict);
            }
        }
    }
}
