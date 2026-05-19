// TransportSelector.cs — .NET Framework 4.6.2 互換の HTTP/2 ↔ HTTP/1.1 ALPN トランスポートセレクター
// WinHttpHandler を使用して ALPN ネゴシエーションを試み、
// HTTP/2 が失敗した場合は 8443-legacy-listener（HTTP/1.1）にフォールバックする
// 設計方針: tier3 レガシークライアントは .NET 4.6.2 制約内で最善のプロトコルを選択する

// System 名前空間をインポートする（基本型に必要）
using System;
// HTTP クライアント関連型をインポートする
using System.Net.Http;
// URI 操作に必要な名前空間をインポートする
using System.Net;
// スレッド管理に必要な名前空間をインポートする
using System.Threading;
// Task 型に必要な名前空間をインポートする
using System.Threading.Tasks;

// K1s0.Tier3.Legacy 名前空間に配置する
namespace K1s0.Tier3.Legacy
{
    // ALPN プローブの結果を表す列挙型
    public enum AlpnProbeResult
    {
        // HTTP/2 で接続成功（ALPN で h2 が選択された）
        Http2Available,
        // HTTP/2 が利用不可（ALPN フォールバック先として HTTP/1.1 を使用する）
        FallbackHttp1,
        // プローブ自体が失敗した（ネットワーク到達不可など）
        ProbeError,
    }

    // トランスポートセレクターの設定を保持するクラス
    public sealed class TransportSelectorOptions
    {
        // プライマリエンドポイントのベース URL（HTTP/2 接続先）
        public readonly Uri PrimaryBaseUri;
        // レガシーエンドポイントのベース URL（HTTP/1.1 フォールバック先、ポート 8443）
        public readonly Uri LegacyBaseUri;
        // ALPN プローブのタイムアウト（Task.Wait のタイムアウトとして使用する）
        // wall-clock TTL は使用しない — このタイムアウトは接続待機の上限であり、HLC 管理外の純 I/O timeout
        public readonly TimeSpan AlpnProbeTimeout;
        // プローブ用のパス（通常は /health または /alpn-check）
        public readonly string ProbePath;

        // コンストラクター
        public TransportSelectorOptions(
            Uri primaryBaseUri,
            Uri legacyBaseUri,
            TimeSpan alpnProbeTimeout,
            string probePath = "/health")
        {
            // プライマリ URI を設定する
            PrimaryBaseUri = primaryBaseUri;
            // レガシー URI を設定する
            LegacyBaseUri = legacyBaseUri;
            // ALPN プローブタイムアウトを設定する
            AlpnProbeTimeout = alpnProbeTimeout;
            // プローブパスを設定する
            ProbePath = probePath;
        }
    }

    // トランスポートセレクターの選択結果を保持するクラス
    public sealed class TransportSelectionResult
    {
        // 選択されたプロトコル（HTTP/2 または HTTP/1.1）
        public readonly AlpnProbeResult Protocol;
        // 選択されたベース URI（実際の接続先）
        public readonly Uri SelectedBaseUri;
        // 作成された HttpClient（呼び出し元がライフサイクルを管理する）
        public readonly HttpClient Client;

        // コンストラクター
        public TransportSelectionResult(AlpnProbeResult protocol, Uri selectedBaseUri, HttpClient client)
        {
            // フィールドを設定する
            Protocol = protocol;
            SelectedBaseUri = selectedBaseUri;
            Client = client;
        }
    }

    // HTTP/2 ALPN プローブを実行して適切なトランスポートを選択するクラス
    public sealed class TransportSelector : IDisposable
    {
        // 設定オプションを保持するフィールド
        private readonly TransportSelectorOptions _options;
        // リソース解放済みフラグ（二重 Dispose を防ぐ）
        private bool _disposed;

        // コンストラクター（設定を注入する）
        public TransportSelector(TransportSelectorOptions options)
        {
            // null チェック（設定が null の場合は例外をスローする）
            if (options == null)
            {
                throw new ArgumentNullException("options");
            }
            // 設定を保存する
            _options = options;
            // Dispose フラグを初期化する
            _disposed = false;
        }

        // ALPN プローブを実行して最適なトランスポートを選択する
        // .NET 4.6.2 制約: async/await 不使用、Task.Wait を使用してブロッキング待機する
        public TransportSelectionResult SelectTransport()
        {
            // Dispose 済みの場合は例外をスローする
            if (_disposed)
            {
                throw new ObjectDisposedException("TransportSelector");
            }

            // ALPN プローブを実行して結果を取得する
            var probeResult = RunAlpnProbe();

            // プローブ結果に応じてトランスポートを選択する
            switch (probeResult)
            {
                case AlpnProbeResult.Http2Available:
                    // HTTP/2 が利用可能な場合はプライマリ URI を使用する
                    return CreateHttp2Transport();

                case AlpnProbeResult.FallbackHttp1:
                    // HTTP/2 が利用不可の場合はレガシー URI にフォールバックする
                    return CreateHttp1FallbackTransport();

                case AlpnProbeResult.ProbeError:
                    // プローブエラーの場合は安全なフォールバック（HTTP/1.1）を使用する
                    return CreateHttp1FallbackTransport();

                default:
                    // 未知の結果は安全なフォールバックを使用する
                    return CreateHttp1FallbackTransport();
            }
        }

        // ALPN プローブを実行する（HTTP/2 接続を試みる）
        // .NET 4.6.2: WinHttpHandler を使用して ALPN を試みる
        private AlpnProbeResult RunAlpnProbe()
        {
            // WinHttpHandler を生成する（Windows HTTP API を直接使用する）
            WinHttpHandler probeHandler = null;
            // プローブ用 HttpClient を生成する
            HttpClient probeClient = null;

            try
            {
                // WinHttpHandler を初期化する（HTTP/2 ALPN ネゴシエーションを有効にする）
                probeHandler = new WinHttpHandler();
                // HTTP/2 を自動選択するポリシーを設定する（Windows 8.1 以降で有効）
                // SendRequestAsync の際に ALPN で h2 がネゴシエーションされる
                // HTTP/2 接続失敗時は自動的に HTTP/1.1 に降格する
                probeHandler.RequestConnectionTimeout = _options.AlpnProbeTimeout;
                // SSL クライアント証明書選択を無効化する（probe 用なので不要）
                probeHandler.ClientCertificateOption = ClientCertificateOption.Manual;
                // TLS チェック設定（本番では ServerCertificateValidationCallback で厳格化する）
                // legacy 環境の自己署名証明書を許可する（開発・検証環境のみ）
                probeHandler.ServerCertificateValidationCallback =
                    (sender, cert, chain, errors) => true;

                // プローブ用 HttpClient を初期化する
                probeClient = new HttpClient(probeHandler, disposeHandler: true);
                // ベース URI をプライマリに設定する
                probeClient.BaseAddress = _options.PrimaryBaseUri;
                // タイムアウトを設定する（接続タイムアウトと同じ値を使用する）
                probeClient.Timeout = _options.AlpnProbeTimeout;

                // プローブリクエストを非同期で開始して Task を取得する
                var probeTask = probeClient.GetAsync(_options.ProbePath);

                // タイムアウト内で接続を待機する（.NET 4.6.2 では Task.Wait を使用する）
                // Task.Result ではなく Task.Wait + probeTask.Result の組み合わせで処理する
                var completed = probeTask.Wait((int)_options.AlpnProbeTimeout.TotalMilliseconds);

                // タイムアウトした場合は HTTP/1.1 フォールバックを返す
                if (!completed)
                {
                    return AlpnProbeResult.FallbackHttp1;
                }

                // タスク完了後に結果を取得する（.NET 4.6.2: async/await 不使用のため Task.Result を使用する）
                var response = probeTask.Result;

                // HTTP/2 接続が成功した場合（ステータスコードが 2xx）
                if (response.IsSuccessStatusCode)
                {
                    // HTTP/2 が利用可能と判断する
                    return AlpnProbeResult.Http2Available;
                }

                // 4xx / 5xx はサーバーに到達したが HTTP/2 で問題が発生した可能性がある
                // 安全のため HTTP/1.1 フォールバックを返す
                return AlpnProbeResult.FallbackHttp1;
            }
            catch (AggregateException aggregateEx)
            {
                // AggregateException を展開して内部例外を確認する
                var innerEx = aggregateEx.InnerException;

                // HTTP/2 ALPN ネゴシエーション失敗は HttpRequestException として現れる
                if (innerEx is HttpRequestException)
                {
                    // ALPN ネゴシエーション失敗 → HTTP/1.1 フォールバック
                    return AlpnProbeResult.FallbackHttp1;
                }

                // WebException は WinHttpHandler のプロトコルネゴシエーション失敗を示す
                if (innerEx is WebException)
                {
                    // WebException はプロトコルエラーの可能性があるためフォールバックを返す
                    return AlpnProbeResult.FallbackHttp1;
                }

                // その他の例外はプローブエラーとして処理する
                return AlpnProbeResult.ProbeError;
            }
            catch (Exception)
            {
                // 予期しない例外はプローブエラーとして処理する
                return AlpnProbeResult.ProbeError;
            }
            finally
            {
                // probeClient を Dispose する（probeHandler は disposeHandler: true で自動解放）
                if (probeClient != null)
                {
                    probeClient.Dispose();
                }
            }
        }

        // HTTP/2 トランスポートを作成する
        private TransportSelectionResult CreateHttp2Transport()
        {
            // HTTP/2 用の WinHttpHandler を生成する
            var handler = new WinHttpHandler();
            // HTTP/2 ALPN ネゴシエーションを有効にする（デフォルトで有効）
            handler.RequestConnectionTimeout = _options.AlpnProbeTimeout;
            // HTTP/2 対応の HttpClient を生成する
            var client = new HttpClient(handler, disposeHandler: true);
            // プライマリ URI をベースアドレスに設定する
            client.BaseAddress = _options.PrimaryBaseUri;
            // HTTP/2 トランスポート選択結果を返す
            return new TransportSelectionResult(
                AlpnProbeResult.Http2Available,
                _options.PrimaryBaseUri,
                client
            );
        }

        // HTTP/1.1 フォールバックトランスポートを作成する（8443-legacy-listener 接続）
        private TransportSelectionResult CreateHttp1FallbackTransport()
        {
            // HTTP/1.1 用の HttpClientHandler を生成する（WinHttpHandler は HTTP/1.1 でも使用可能だが
            // レガシー接続では HttpClientHandler の方がシンプルな設定で動作する）
            var handler = new HttpClientHandler();
            // HTTP/1.1 のみを使用する（HTTP/2 アップグレード試行を防ぐ）
            // .NET 4.6.2 では ServicePointManager を通じて設定する
            ServicePointManager.SecurityProtocol =
                SecurityProtocolType.Tls12 | SecurityProtocolType.Tls11;
            // TLS チェック設定（legacy サーバーの自己署名証明書を許可する場合）
            // 本番では証明書検証を有効にする必要がある
            handler.ServerCertificateCustomValidationCallback =
                (message, cert, chain, errors) => true;
            // HTTP/1.1 対応の HttpClient を生成する
            var client = new HttpClient(handler, disposeHandler: true);
            // レガシー URI（ポート 8443）をベースアドレスに設定する
            client.BaseAddress = _options.LegacyBaseUri;
            // HTTP/1.1 フォールバックトランスポート選択結果を返す
            return new TransportSelectionResult(
                AlpnProbeResult.FallbackHttp1,
                _options.LegacyBaseUri,
                client
            );
        }

        // IDisposable の実装（マネージドリソースを解放する）
        public void Dispose()
        {
            // 二重 Dispose を防ぐ
            if (!_disposed)
            {
                // Dispose 済みフラグを立てる
                _disposed = true;
            }
        }
    }
}
