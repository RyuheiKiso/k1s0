// Program.cs — k1s0 Tier3 Legacy WinForms エントリポイント
// ERP 並行運用向け WinForms ホスト: .NET Framework 4.8 + HTTP/1.1 + SSE
// docs/03_概要設計/04_tier3設計方針/09_レガシー資産統合.md の Companion + Gateway 経由設計に準拠する

// System 名前空間: 基本型 / 例外処理に使用する
using System;
// System.Windows.Forms 名前空間: WinForms アプリケーション実行に使用する
using System.Windows.Forms;

// k1s0 Legacy WinForms 名前空間
namespace K1s0.Tier3.Legacy.WinForms
{
    // Program: WinForms アプリケーションのエントリポイントクラス
    internal static class Program
    {
        // Main: WinForms エントリポイント（STAThread でシングルスレッドアパートメントを設定する）
        [STAThread]
        private static void Main()
        {
            // High DPI 対応: SetHighDpiMode を ApplicationHighDpiMode.PerMonitorV2 に設定する
            Application.SetHighDpiMode(HighDpiMode.PerMonitorV2);
            // VisualStyles を有効化する（モダンな外観で表示する）
            Application.EnableVisualStyles();
            // テキストレンダリングを互換モードに設定する
            Application.SetCompatibleTextRenderingDefault(false);
            // 未処理例外をハンドリングするイベントを登録する
            Application.ThreadException += OnThreadException;
            // AppDomain の未処理例外をハンドリングするイベントを登録する
            AppDomain.CurrentDomain.UnhandledException += OnUnhandledException;
            // MainForm を生成してアプリケーションを起動する（Companion 初期化は MainForm コンストラクタ内で行う）
            Application.Run(new MainForm());
        }

        // OnThreadException: UI スレッドの未処理例外をログに記録する
        private static void OnThreadException(object sender, System.Threading.ThreadExceptionEventArgs e)
        {
            // エラーダイアログを表示する（ERP 並行運用中のクラッシュを防ぐためメッセージボックスで通知する）
            MessageBox.Show(
                // 例外メッセージを表示する
                string.Format("未処理例外が発生しました:\n{0}", e.Exception.Message),
                // ダイアログタイトル
                "k1s0 WinForms — エラー",
                // ボタン: OK のみ
                MessageBoxButtons.OK,
                // アイコン: エラー
                MessageBoxIcon.Error
            );
        }

        // OnUnhandledException: AppDomain の未処理例外をログに記録する
        private static void OnUnhandledException(object sender, UnhandledExceptionEventArgs e)
        {
            // 未処理例外のメッセージを組み立てる
            var msg = e.ExceptionObject?.ToString() ?? "不明なエラー";
            // エラーダイアログを表示する（アプリケーション終了前にメッセージを表示する）
            MessageBox.Show(
                // 例外詳細を表示する
                string.Format("致命的なエラーが発生しました:\n{0}", msg),
                // ダイアログタイトル
                "k1s0 WinForms — 致命的エラー",
                // ボタン: OK のみ
                MessageBoxButtons.OK,
                // アイコン: エラー
                MessageBoxIcon.Error
            );
        }
    }
}
