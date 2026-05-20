// MainForm.cs — k1s0 Tier3 Legacy WinForms メインフォーム
// ERP 並行運用向けメインメニュー: Companion.NetFx 初期化 + BFF HTTP/1.1 + SSE 接続 + 業務画面遷移
// docs/03_概要設計/04_tier3設計方針/09_レガシー資産統合.md の設計に準拠する
// Companion + Gateway 経由 HTTP/1.1 + SSE でのみ BFF に接続する（OSS 直接アクセス禁止）

// System 名前空間: 基本型 / 例外処理に使用する
using System;
// System.Net.Http 名前空間: HttpClient 経由の BFF 接続に使用する
using System.Net.Http;
// System.Threading 名前空間: キャンセルトークンに使用する
using System.Threading;
// System.Threading.Tasks 名前空間: 非同期処理に使用する
using System.Threading.Tasks;
// System.Windows.Forms 名前空間: WinForms UI 構築に使用する
using System.Windows.Forms;

// k1s0 Legacy WinForms 名前空間
namespace K1s0.Tier3.Legacy.WinForms
{
    // MainForm: ERP 並行運用向けメインフォーム
    // Companion.NetFx runtime 初期化と業務画面への遷移を担う
    // partial 宣言: MainForm.Designer.cs の partial クラスと合わせて使用する（WinForms Designer 規約）
    internal sealed partial class MainForm : Form
    {
        // 発注入力画面へ遷移するボタン（ERP 並行運用の主業務）
        private readonly Button _btnPurchaseOrderEntry;
        // BFF 接続状態を表示するステータスラベル
        private readonly Label _lblConnectionStatus;
        // BFF への HTTP/1.1 + SSE 接続を行う HttpClient
        // Companion + Gateway 経由でのみアクセスする（OSS 直接接続禁止）
        private readonly HttpClient _httpClient;
        // CancellationTokenSource: SSE ポーリングの停止に使用する
        private readonly CancellationTokenSource _ctsSse;
        // BFF の v1_legacy_http11 専用エンドポイント（8443-legacy ポート）
        // docs/09_レガシー資産統合.md: v1_legacy_http11 専用 listener は 8443-legacy ポートを使用する
        private const string BffLegacyBaseUrl = "http://localhost:8443-legacy";
        // SSE エンドポイントのパス（Companion Gateway 経由の server-streaming）
        private const string SseEventsPath = "/events/v1/stream";

        // MainForm コンストラクタ: UI 構築 + Companion 初期化 + BFF 接続開始
        public MainForm()
        {
            // キャンセルトークンソースを生成する（フォームクローズ時に SSE 接続を停止する）
            _ctsSse = new CancellationTokenSource();
            // HttpClient を生成する（BFF 接続用: Companion + Gateway 経由のみ使用する）
            _httpClient = new HttpClient();
            // BFF の v1_legacy_http11 エンドポイントのベース URL を設定する
            _httpClient.BaseAddress = new Uri(BffLegacyBaseUrl);
            // タイムアウトを 30 秒に設定する（SSE ストリームは別途ポーリングで管理する）
            _httpClient.Timeout = TimeSpan.FromSeconds(30);

            // メインフォームのウィンドウタイトルを設定する
            Text = "k1s0 ERP 業務システム（Legacy WinForms）";
            // フォームサイズを設定する（ERP 画面の標準解像度に合わせる）
            Size = new System.Drawing.Size(800, 600);
            // フォームの最小サイズを設定する（縮小しすぎないようにする）
            MinimumSize = new System.Drawing.Size(640, 480);
            // フォームのスタート位置を画面中央に設定する
            StartPosition = FormStartPosition.CenterScreen;

            // 接続ステータスラベルを生成する（BFF 接続状態を表示する）
            _lblConnectionStatus = new Label
            {
                // ラベルテキストを初期化する（接続確認中を表示する）
                Text = "BFF 接続確認中...",
                // ラベルの表示位置を設定する
                Location = new System.Drawing.Point(10, 10),
                // ラベルのサイズを設定する
                Size = new System.Drawing.Size(760, 30),
                // テキストの配置を左揃えにする
                TextAlign = System.Drawing.ContentAlignment.MiddleLeft,
            };

            // 発注入力ボタンを生成する（PurchaseOrderEntryForm への遷移ボタン）
            _btnPurchaseOrderEntry = new Button
            {
                // ボタンのテキストを設定する
                Text = "発注入力",
                // ボタンの表示位置を設定する
                Location = new System.Drawing.Point(10, 60),
                // ボタンのサイズを設定する
                Size = new System.Drawing.Size(200, 40),
                // ボタンの TabIndex を設定する（キーボード操作順序）
                TabIndex = 0,
            };
            // 発注入力ボタンのクリックイベントを登録する
            _btnPurchaseOrderEntry.Click += OnPurchaseOrderEntryClick;

            // フォームにコントロールを追加する
            Controls.Add(_lblConnectionStatus);
            // 発注入力ボタンをフォームに追加する
            Controls.Add(_btnPurchaseOrderEntry);

            // フォームのロードイベントを登録する（Companion 初期化 + BFF 接続を行う）
            Load += OnFormLoad;
            // フォームのクローズイベントを登録する（リソースを解放する）
            FormClosing += OnFormClosing;
        }

        // OnFormLoad: フォームロード時に Companion 初期化と BFF 接続確認を行う
        private async void OnFormLoad(object? sender, EventArgs e)
        {
            // Companion.NetFx runtime を初期化する（メッセージベースのスタブ）
            // 本実装では CompanionRuntime.Initialize() を呼び出す（docs: CLR Profiler アタッチ方式）
            InitializeCompanionRuntime();
            // BFF への HTTP/1.1 接続確認を非同期で実行する
            await CheckBffConnectionAsync();
            // SSE ストリームのポーリングを開始する（バックグラウンドタスク）
            _ = Task.Run(() => StartSsePollingAsync(_ctsSse.Token));
        }

        // InitializeCompanionRuntime: Companion.NetFx runtime の初期化（メッセージベーススタブ）
        // 本番実装では k1s0.Companion.NetFx.OTelExt が CLR Profiler 経由でアタッチされる
        // docs/09_レガシー資産統合.md: 役割 A: Observability / 認証コンテキスト伝播
        private static void InitializeCompanionRuntime()
        {
            // Companion runtime 初期化のスタブメッセージをデバッグ出力に記録する
            System.Diagnostics.Debug.WriteLine(
                "[Companion.NetFx] runtime initialized (stub): OTel exporter = otlp-grpc, transport = v1_legacy_http11"
            );
            // 本番実装では以下を呼び出す:
            // CompanionRuntime.Initialize(new CompanionOptions
            // {
            //     Transport = TransportKind.Sse,
            //     BffEndpoint = new Uri(BffLegacyBaseUrl),
            //     OtelEndpoint = new Uri("http://localhost:4317"),
            // });
        }

        // CheckBffConnectionAsync: BFF への HTTP/1.1 接続確認を非同期で実行する
        // Companion + Gateway 経由のみ（OSS 直接アクセス禁止）
        private async Task CheckBffConnectionAsync()
        {
            try
            {
                // BFF の health check エンドポイントに GET リクエストを送信する
                var response = await _httpClient.GetAsync("/health").ConfigureAwait(false);
                // 接続成功: ステータスラベルを更新する（UI スレッドで実行する）
                Invoke(new Action(() =>
                {
                    // BFF 接続状態を成功に更新する
                    _lblConnectionStatus.Text = string.Format(
                        "BFF 接続: 正常（HTTP {0}）| エンドポイント: {1}",
                        (int)response.StatusCode,
                        BffLegacyBaseUrl
                    );
                    // テキスト色を緑に設定する（接続成功を視覚的に示す）
                    _lblConnectionStatus.ForeColor = System.Drawing.Color.Green;
                }));
            }
            catch (Exception ex)
            {
                // 接続失敗: ステータスラベルをエラー状態に更新する（UI スレッドで実行する）
                Invoke(new Action(() =>
                {
                    // BFF 接続失敗のメッセージを表示する
                    _lblConnectionStatus.Text = string.Format(
                        "BFF 接続: 失敗（{0}）| エンドポイント: {1}",
                        ex.Message,
                        BffLegacyBaseUrl
                    );
                    // テキスト色を赤に設定する（接続失敗を視覚的に示す）
                    _lblConnectionStatus.ForeColor = System.Drawing.Color.Red;
                }));
            }
        }

        // StartSsePollingAsync: BFF SSE ストリームへのポーリングを開始する
        // HTTP/1.1 + SSE でサーバーからのイベントを受信する（HTTP/2 非対応環境向け）
        // docs/09_レガシー資産統合.md: v1_legacy_http11 専用 listener (8443-legacy) 経由でのみ接続する
        private async Task StartSsePollingAsync(CancellationToken cancellationToken)
        {
            // SSE エンドポイントの完全 URI を組み立てる
            var sseUri = BffLegacyBaseUrl + SseEventsPath;
            // SSE ポーリングのデバッグログを出力する
            System.Diagnostics.Debug.WriteLine(
                string.Format("[SSE] polling started: {0}", sseUri)
            );
            // キャンセルされるまで SSE ポーリングを繰り返す
            while (!cancellationToken.IsCancellationRequested)
            {
                try
                {
                    // SSE ストリームに GET リクエストを送信する（HttpCompletionOption.ResponseHeadersRead で即時受信開始）
                    using (var request = new HttpRequestMessage(HttpMethod.Get, SseEventsPath))
                    {
                        // Accept ヘッダーを text/event-stream に設定する（SSE プロトコル）
                        request.Headers.Accept.Add(
                            new System.Net.Http.Headers.MediaTypeWithQualityHeaderValue("text/event-stream")
                        );
                        // SSE ストリームへのリクエストを送信する
                        using (var response = await _httpClient
                            .SendAsync(request, HttpCompletionOption.ResponseHeadersRead, cancellationToken)
                            .ConfigureAwait(false))
                        {
                            // SSE ストリームを読み取る
                            using (var stream = await response.Content.ReadAsStreamAsync().ConfigureAwait(false))
                            using (var reader = new System.IO.StreamReader(stream))
                            {
                                // キャンセルされるまでストリームを読み取る
                                while (!cancellationToken.IsCancellationRequested && !reader.EndOfStream)
                                {
                                    // 1 行読み取る（SSE は行単位でイベントを送信する）
                                    var line = await reader.ReadLineAsync().ConfigureAwait(false);
                                    // data: プレフィックスの SSE イベントを処理する
                                    if (line != null && line.StartsWith("data: ", StringComparison.Ordinal))
                                    {
                                        // イベントデータを取得する
                                        var eventData = line.Substring(6);
                                        // 受信イベントをデバッグログに出力する
                                        System.Diagnostics.Debug.WriteLine(
                                            string.Format("[SSE] event received: {0}", eventData)
                                        );
                                    }
                                }
                            }
                        }
                    }
                }
                catch (OperationCanceledException)
                {
                    // キャンセルされた場合はループを終了する
                    break;
                }
                catch (Exception ex)
                {
                    // 接続エラー: 5 秒後にリトライする
                    System.Diagnostics.Debug.WriteLine(
                        string.Format("[SSE] connection error: {0} — retrying in 5s", ex.Message)
                    );
                    // 5 秒待機してリトライする
                    try
                    {
                        // 5 秒間待機する（キャンセルされた場合は即座に終了する）
                        await Task.Delay(TimeSpan.FromSeconds(5), cancellationToken).ConfigureAwait(false);
                    }
                    catch (OperationCanceledException)
                    {
                        // キャンセルされた場合はループを終了する
                        break;
                    }
                }
            }
            // SSE ポーリング終了ログを出力する
            System.Diagnostics.Debug.WriteLine("[SSE] polling stopped");
        }

        // OnPurchaseOrderEntryClick: 発注入力ボタンのクリックイベントハンドラ
        // PurchaseOrderEntryForm を開く
        private void OnPurchaseOrderEntryClick(object? sender, EventArgs e)
        {
            // PurchaseOrderEntryForm を生成する（tier2 SDK WinForms wrapper 経由で BFF に送信する）
            using (var form = new PurchaseOrderEntryForm(_httpClient))
            {
                // 発注入力フォームをモーダルダイアログとして表示する
                form.ShowDialog(this);
            }
        }

        // OnFormClosing: フォームクローズ時にリソースを解放する
        private void OnFormClosing(object? sender, FormClosingEventArgs e)
        {
            // SSE ポーリングをキャンセルする
            _ctsSse.Cancel();
            // CancellationTokenSource を破棄する
            _ctsSse.Dispose();
            // HttpClient を破棄する
            _httpClient.Dispose();
        }

        // Dispose: 管理リソースを解放する（Finalizer の補完）
        protected override void Dispose(bool disposing)
        {
            // マネージドリソースを解放する
            if (disposing)
            {
                // キャンセルトークンソースを解放する（既にクローズイベントで解放済みの場合は無視する）
                try { _ctsSse.Dispose(); } catch { /* 既に解放済み */ }
                // HttpClient を解放する
                try { _httpClient.Dispose(); } catch { /* 既に解放済み */ }
            }
            // 基底クラスの Dispose を呼び出す
            base.Dispose(disposing);
        }
    }
}
