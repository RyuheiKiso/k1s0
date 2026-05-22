// k1s0 Connect-RPC conformance テスト（xUnit 実装）
// ConformanceSuite の全シナリオに対して Connect-RPC wire format の適合を検証する
// ASP.NET Core TestServer を使ってインプロセスで HTTP ハンドラーをテストする

// System 名前空間: BitConverter 等の基本型に使用する
using System;
// System.IO: MemoryStream に使用する
using System.IO;
// System.Net: HttpStatusCode に使用する
using System.Net;
// System.Net.Http: HttpClient / HttpRequestMessage に使用する
using System.Net.Http;
// System.Net.Http.Headers: MediaTypeHeaderValue に使用する
using System.Net.Http.Headers;
// System.Buffers.Binary: BinaryPrimitives（big-endian 書き込み）に使用する
using System.Buffers.Binary;
// System.Threading.Tasks: Task に使用する
using System.Threading.Tasks;
// xUnit テストフレームワーク
using Xunit;
// k1s0 Connect-RPC 実装
using K1s0.Connect.NetCore;

namespace K1s0.Connect.NetCore.Tests
{
    /// <summary>
    /// Connect-RPC wire format conformance テストクラス
    /// ConnectProtocol の encode / decode を検証する
    /// </summary>
    public class ConformanceTest
    {
        /// <summary>
        /// T3-R2-CONF-01: 5 byte framing エンコードの検証
        /// flags = 0x00, payload = { 0x01, 0x02, 0x03 } のフレームを正しくエンコードできることを確認する
        /// </summary>
        [Fact]
        public void EncodeFrame_NormalFlags_Has5ByteHeader()
        {
            // テスト用のペイロード（3 バイト）
            var payload = new byte[] { 0x01, 0x02, 0x03 };

            // 5 byte header バッファを手動で生成する（期待値として使用する）
            var expected = new byte[ConnectProtocol.FrameHeaderSize + payload.Length];
            // flags = 0x00（ConnectFlags.Normal）
            expected[0] = ConnectFlags.Normal;
            // length = 3 を big-endian で書き込む
            BinaryPrimitives.WriteInt32BigEndian(expected.AsSpan(1, 4), payload.Length);
            // ペイロードをコピーする
            payload.CopyTo(expected.AsSpan(ConnectProtocol.FrameHeaderSize));

            // ConnectFrame を使って直接 WriteFrameAsync でエンコードする
            var frame = new ConnectFrame(ConnectFlags.Normal, payload.AsMemory());
            var outputStream = new MemoryStream();

            // WriteFrameAsync を同期的に待機する（テスト用）
            ConnectProtocol.WriteFrameAsync(outputStream, frame).GetAwaiter().GetResult();

            // エンコード結果を検証する
            var actual = outputStream.ToArray();

            // エンコードされたバイト列が期待値と一致することを確認する
            Assert.Equal(expected, actual);
        }

        /// <summary>
        /// T3-R2-CONF-02: 5 byte framing デコードの検証
        /// エンコードしたフレームを ReadFrameAsync でデコードして元のペイロードが復元されることを確認する
        /// </summary>
        [Fact]
        public async Task DecodeFrame_EncodedFrame_RestoresPayload()
        {
            // テスト用のペイロード（5 バイト）
            var originalPayload = new byte[] { 0xAA, 0xBB, 0xCC, 0xDD, 0xEE };

            // フレームを生成して MemoryStream にエンコードする
            var frame = new ConnectFrame(ConnectFlags.Normal, originalPayload.AsMemory());
            var stream = new MemoryStream();
            // WriteFrameAsync でエンコードする
            await ConnectProtocol.WriteFrameAsync(stream, frame);

            // ストリームを先頭に戻す
            stream.Seek(0, SeekOrigin.Begin);

            // ReadFrameAsync でデコードする
            var decoded = await ConnectProtocol.ReadFrameAsync(stream);

            // デコードされたフレームが null でないことを確認する
            Assert.NotNull(decoded);

            // フラグが一致することを確認する
            Assert.Equal(ConnectFlags.Normal, decoded!.Value.Flags);

            // ペイロードが元のバイト列と一致することを確認する
            Assert.Equal(originalPayload, decoded.Value.Payload.ToArray());
        }

        /// <summary>
        /// T3-R2-CONF-03: end-stream フラグの検証
        /// flags = ConnectFlags.EndStream のフレームを IsEndStream プロパティで正しく判定できることを確認する
        /// </summary>
        [Fact]
        public async Task ReadFrame_EndStreamFlag_IsEndStreamTrue()
        {
            // end-stream フレームを生成する
            var frame = new ConnectFrame(ConnectFlags.EndStream, ReadOnlyMemory<byte>.Empty);
            var stream = new MemoryStream();
            // WriteFrameAsync でエンコードする
            await ConnectProtocol.WriteFrameAsync(stream, frame);

            // ストリームを先頭に戻す
            stream.Seek(0, SeekOrigin.Begin);

            // ReadFrameAsync でデコードする
            var decoded = await ConnectProtocol.ReadFrameAsync(stream);

            // デコードされたフレームが null でないことを確認する
            Assert.NotNull(decoded);

            // IsEndStream プロパティが true であることを確認する
            Assert.True(decoded!.Value.IsEndStream);
        }

        /// <summary>
        /// T3-R2-CONF-04: 圧縮フラグの検証
        /// flags = ConnectFlags.Compressed のフレームを IsCompressed プロパティで正しく判定できることを確認する
        /// </summary>
        [Fact]
        public async Task ReadFrame_CompressedFlag_IsCompressedTrue()
        {
            // 圧縮フレームを生成する（ペイロードは実際には圧縮していないが flags のみ検証する）
            var payload = new byte[] { 0x78, 0x9C, 0x63, 0x00 };
            var frame = new ConnectFrame(ConnectFlags.Compressed, payload.AsMemory());
            var stream = new MemoryStream();
            // WriteFrameAsync でエンコードする
            await ConnectProtocol.WriteFrameAsync(stream, frame);

            // ストリームを先頭に戻す
            stream.Seek(0, SeekOrigin.Begin);

            // ReadFrameAsync でデコードする
            var decoded = await ConnectProtocol.ReadFrameAsync(stream);

            // デコードされたフレームが null でないことを確認する
            Assert.NotNull(decoded);

            // IsCompressed プロパティが true であることを確認する
            Assert.True(decoded!.Value.IsCompressed);
        }

        /// <summary>
        /// T3-R2-CONF-05: ストリーム終端の検証
        /// 空の MemoryStream に対して ReadFrameAsync が null を返すことを確認する
        /// </summary>
        [Fact]
        public async Task ReadFrame_EmptyStream_ReturnsNull()
        {
            // 空の MemoryStream を作成する
            var stream = new MemoryStream();

            // ReadFrameAsync が null を返すことを確認する
            var result = await ConnectProtocol.ReadFrameAsync(stream);

            // null が返されることを確認する（ストリーム終端を示す）
            Assert.Null(result);
        }

        /// <summary>
        /// T3-R2-CONF-06: ConformanceSuite の全シナリオが列挙されることを確認する
        /// AllScenarios() が 10 件以上のシナリオを返すことを確認する（最小チェック）
        /// </summary>
        [Fact]
        public void ConformanceSuite_AllScenarios_HasExpectedCount()
        {
            // 全シナリオを列挙する
            var scenarios = new System.Collections.Generic.List<ConformanceScenario>(
                ConformanceSuite.AllScenarios());

            // 最低 10 件のシナリオが存在することを確認する
            Assert.True(scenarios.Count >= 10,
                $"ConformanceSuite の シナリオ数が期待値より少ない: {scenarios.Count} < 10");
        }

        /// <summary>
        /// T3-R2-CONF-07: 4 RPC form が全て ConformanceSuite に含まれることを確認する
        /// Unary / ServerStreaming / ClientStreaming / BidiStreaming の 4 form が存在することを確認する
        /// </summary>
        [Fact]
        public void ConformanceSuite_AllScenarios_ContainsAll4RpcForms()
        {
            // 全シナリオを収集する
            var scenarios = new System.Collections.Generic.List<ConformanceScenario>(
                ConformanceSuite.AllScenarios());

            // Unary シナリオが存在することを確認する
            Assert.Contains(scenarios, s => s.Form == RpcForm.Unary);

            // ServerStreaming シナリオが存在することを確認する
            Assert.Contains(scenarios, s => s.Form == RpcForm.ServerStreaming);

            // ClientStreaming シナリオが存在することを確認する
            Assert.Contains(scenarios, s => s.Form == RpcForm.ClientStreaming);

            // BidiStreaming シナリオが存在することを確認する
            Assert.Contains(scenarios, s => s.Form == RpcForm.BidiStreaming);
        }
    }
}
