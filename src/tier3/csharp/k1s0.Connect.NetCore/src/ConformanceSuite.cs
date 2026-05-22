// k1s0 Connect-RPC conformance suite: 4 RPC form の適合テストケース定義
// Unary / ServerStreaming / ClientStreaming / BidiStreaming の 4 form の
// Connect-RPC wire format 適合テストシナリオを定義する
// 実際のテスト実行は tests/ConformanceTest.cs の xUnit テストが担う

// System 名前空間: Exception 等の基本型に使用する
using System;
// System.Collections.Generic: List / IAsyncEnumerable に使用する
using System.Collections.Generic;
// System.Threading: CancellationToken に使用する
using System.Threading;
// System.Threading.Tasks: Task に使用する
using System.Threading.Tasks;

namespace K1s0.Connect.NetCore
{
    /// <summary>
    /// ConformanceScenario: Connect-RPC conformance テストのシナリオを表す値型
    /// テストシナリオ ID / RPC form / 説明を保持する
    /// </summary>
    public readonly struct ConformanceScenario
    {
        // テストシナリオの一意識別子（例: "unary_basic"）
        public readonly string Id;

        // テストする RPC form（Unary / ServerStreaming / ClientStreaming / Bidi）
        public readonly RpcForm Form;

        // テストシナリオの説明文
        public readonly string Description;

        // テストが正常系かどうか（true: 正常系, false: エラー系）
        public readonly bool IsPositive;

        /// <summary>
        /// ConformanceScenario のコンストラクタ
        /// </summary>
        public ConformanceScenario(string id, RpcForm form, string description, bool isPositive = true)
        {
            // 識別子を設定する
            Id = id;
            // RPC form を設定する
            Form = form;
            // 説明文を設定する
            Description = description;
            // 正常系フラグを設定する
            IsPositive = isPositive;
        }
    }

    /// <summary>
    /// ConformanceSuite: 4 RPC form の Connect-RPC 適合テストシナリオを列挙するクラス
    /// xUnit テストで IEnumerable<object[]> として使用する
    /// </summary>
    public static class ConformanceSuite
    {
        /// <summary>
        /// 全 conformance シナリオを返す
        /// Unary / ServerStreaming / ClientStreaming / BidiStreaming の 4 form を網羅する
        /// </summary>
        public static IEnumerable<ConformanceScenario> AllScenarios()
        {
            // --- Unary RPC form のシナリオ ---

            // 正常系: 単純なリクエスト/レスポンス
            yield return new ConformanceScenario(
                // シナリオ ID
                "unary_basic",
                // RPC form
                RpcForm.Unary,
                // 説明文
                "Unary RPC の基本シナリオ: リクエストを送信してレスポンスを受信する",
                // 正常系
                isPositive: true);

            // 正常系: 圧縮フレームの送受信
            yield return new ConformanceScenario(
                // シナリオ ID
                "unary_compressed",
                // RPC form
                RpcForm.Unary,
                // 説明文
                "Unary RPC の圧縮フレームシナリオ: flags bit 0 = 1 の圧縮フレームを処理する",
                // 正常系
                isPositive: true);

            // エラー系: ペイロード長超過
            yield return new ConformanceScenario(
                // シナリオ ID
                "unary_oversized_payload",
                // RPC form
                RpcForm.Unary,
                // 説明文
                "Unary RPC のペイロード超過シナリオ: MaxFrameSize を超えるペイロードはエラーになること",
                // エラー系
                isPositive: false);

            // エラー系: 不正な Content-Type
            yield return new ConformanceScenario(
                // シナリオ ID
                "unary_wrong_content_type",
                // RPC form
                RpcForm.Unary,
                // 説明文
                "Unary RPC の Content-Type 不正シナリオ: application/json は 415 エラーになること",
                // エラー系
                isPositive: false);

            // --- ServerStreaming RPC form のシナリオ ---

            // 正常系: サーバーから複数フレームを受信する
            yield return new ConformanceScenario(
                // シナリオ ID
                "server_streaming_basic",
                // RPC form
                RpcForm.ServerStreaming,
                // 説明文
                "ServerStreaming RPC の基本シナリオ: サーバーから複数フレームを受信して end-stream で終了する",
                // 正常系
                isPositive: true);

            // 正常系: 空ストリーム（0 フレーム）
            yield return new ConformanceScenario(
                // シナリオ ID
                "server_streaming_empty",
                // RPC form
                RpcForm.ServerStreaming,
                // 説明文
                "ServerStreaming RPC の空ストリームシナリオ: end-stream のみを受信してシーケンスが終了する",
                // 正常系
                isPositive: true);

            // --- ClientStreaming RPC form のシナリオ ---

            // 正常系: クライアントから複数フレームを送信して単一レスポンスを受信する
            yield return new ConformanceScenario(
                // シナリオ ID
                "client_streaming_basic",
                // RPC form
                RpcForm.ClientStreaming,
                // 説明文
                "ClientStreaming RPC の基本シナリオ: 複数フレームを送信して単一レスポンスを受信する",
                // 正常系
                isPositive: true);

            // 正常系: 単一フレームを送信する（最小ケース）
            yield return new ConformanceScenario(
                // シナリオ ID
                "client_streaming_single_frame",
                // RPC form
                RpcForm.ClientStreaming,
                // 説明文
                "ClientStreaming RPC の単一フレームシナリオ: 1 フレームのみ送信して単一レスポンスを受信する",
                // 正常系
                isPositive: true);

            // --- BidiStreaming RPC form のシナリオ ---

            // 正常系: 双方向ストリーミング
            yield return new ConformanceScenario(
                // シナリオ ID
                "bidi_basic",
                // RPC form
                RpcForm.BidiStreaming,
                // 説明文
                "BidiStreaming RPC の基本シナリオ: クライアントとサーバーが同時にストリーミングする",
                // 正常系
                isPositive: true);

            // 正常系: キャンセルシナリオ
            yield return new ConformanceScenario(
                // シナリオ ID
                "bidi_cancel",
                // RPC form
                RpcForm.BidiStreaming,
                // 説明文
                "BidiStreaming RPC のキャンセルシナリオ: CancellationToken でストリームを中断できること",
                // 正常系
                isPositive: true);
        }

        /// <summary>
        /// xUnit の InlineData / MemberData 用に object[] 形式でシナリオを返す
        /// </summary>
        public static IEnumerable<object[]> ScenariosAsTestData()
        {
            // 全シナリオを object[] にラップして xUnit に渡す
            foreach (var scenario in AllScenarios())
            {
                // シナリオを object[] としてラップして返す
                yield return new object[] { scenario };
            }
        }
    }
}
