// k1s0 Connect-RPC wire format 実装
// Connect-RPC 仕様の 5 byte framing（1B flags + 4B big-endian length）を実装する
// https://connectrpc.com/docs/protocol に準拠する

// System 名前空間: BitConverter / Exception 等の基本型に使用する
using System;
// System.Buffers: ReadOnlySequence / IBufferWriter に使用する
using System.Buffers;
// System.Buffers.Binary: BinaryPrimitives（big-endian 読み書き）に使用する
using System.Buffers.Binary;
// System.Collections.Generic: List / IAsyncEnumerable に使用する
using System.Collections.Generic;
// System.IO: Stream の操作に使用する
using System.IO;
// System.Threading: CancellationToken に使用する
using System.Threading;
// System.Threading.Tasks: Task / ValueTask に使用する
using System.Threading.Tasks;
// Google.Protobuf: IMessage（protobuf メッセージのインターフェース）に使用する
using Google.Protobuf;

namespace K1s0.Connect.NetCore
{
    /// <summary>
    /// Connect-RPC フラグバイト定義
    /// Connect wire format の 5 byte header における flags byte の値を定義する
    /// </summary>
    public static class ConnectFlags
    {
        // 通常メッセージ: flags = 0x00（圧縮なし・end-stream なし）
        public const byte Normal = 0x00;

        // 圧縮フラグ: flags の bit 0 が 1 の場合メッセージが圧縮されていることを示す
        public const byte Compressed = 0x01;

        // end-stream フラグ: flags の bit 1 が 1 の場合ストリームの最終フレームであることを示す
        // gRPC-Web と Connect の streaming で使用する
        public const byte EndStream = 0x02;
    }

    /// <summary>
    /// ConnectFrame: Connect-RPC の 1 フレームを表す値型
    /// 5 byte header（flags + length）と payload バイト列を保持する
    /// </summary>
    public readonly struct ConnectFrame
    {
        // フレームのフラグバイト（ConnectFlags の値）
        public readonly byte Flags;

        // ペイロードのバイト列（protobuf serialized message）
        public readonly ReadOnlyMemory<byte> Payload;

        /// <summary>
        /// ConnectFrame のコンストラクタ
        /// </summary>
        /// <param name="flags">フレームのフラグバイト</param>
        /// <param name="payload">ペイロードバイト列</param>
        public ConnectFrame(byte flags, ReadOnlyMemory<byte> payload)
        {
            // フラグを設定する
            Flags = flags;
            // ペイロードを設定する
            Payload = payload;
        }

        // ペイロード長: Payload の Length プロパティから取得する
        public int Length => Payload.Length;

        // end-stream フレームかどうかを判定するプロパティ
        public bool IsEndStream => (Flags & ConnectFlags.EndStream) != 0;

        // 圧縮フレームかどうかを判定するプロパティ
        public bool IsCompressed => (Flags & ConnectFlags.Compressed) != 0;
    }

    /// <summary>
    /// ConnectProtocol: Connect-RPC wire format の encode / decode 実装
    /// 5 byte framing（1B flags + 4B big-endian length + N bytes payload）を実装する
    /// </summary>
    public static class ConnectProtocol
    {
        // Connect フレームヘッダーのサイズ（flags 1B + length 4B = 5B）
        public const int FrameHeaderSize = 5;

        // 最大フレームサイズ: 4MB（DoS 対策のためのサイズ上限）
        public const int MaxFrameSize = 4 * 1024 * 1024;

        /// <summary>
        /// protobuf メッセージを Connect-RPC 5 byte framing でエンコードして byte 配列を返す
        /// </summary>
        /// <param name="message">エンコードするprotobuf メッセージ</param>
        /// <param name="flags">フレームフラグ（ConnectFlags の値）</param>
        /// <returns>5 byte header + serialized message のバイト配列</returns>
        public static byte[] EncodeMessage(IMessage message, byte flags = ConnectFlags.Normal)
        {
            // message が null の場合は例外を投げる
            if (message == null)
            {
                // null メッセージは許容しない
                throw new ArgumentNullException(nameof(message), "Connect-RPC フレームのメッセージが null です");
            }

            // protobuf メッセージをバイト配列にシリアライズする
            var serialized = message.ToByteArray();

            // ペイロード長を取得する
            var payloadLength = serialized.Length;

            // ペイロード長が最大サイズを超える場合は例外を投げる
            if (payloadLength > MaxFrameSize)
            {
                // フレームサイズ超過のエラーを投げる
                throw new InvalidOperationException(
                    $"Connect-RPC フレームサイズが上限を超えています: {payloadLength} > {MaxFrameSize}");
            }

            // 5 byte header + payload を格納するバッファを確保する
            var buffer = new byte[FrameHeaderSize + payloadLength];

            // flags byte をバッファの先頭に書き込む
            buffer[0] = flags;

            // length を big-endian で 4 バイトに書き込む（bytes[1..4]）
            BinaryPrimitives.WriteInt32BigEndian(buffer.AsSpan(1, 4), payloadLength);

            // シリアライズ済みペイロードをバッファの残り部分にコピーする
            serialized.AsSpan().CopyTo(buffer.AsSpan(FrameHeaderSize));

            // エンコード済みバッファを返す
            return buffer;
        }

        /// <summary>
        /// Connect-RPC 5 byte framing のバイト列を読み取って ConnectFrame を返す非同期メソッド
        /// </summary>
        /// <param name="stream">読み取り元ストリーム（HTTP request / response body）</param>
        /// <param name="cancellationToken">キャンセレーショントークン</param>
        /// <returns>読み取った ConnectFrame（ストリーム終端の場合は null）</returns>
        public static async Task<ConnectFrame?> ReadFrameAsync(
            Stream stream,
            CancellationToken cancellationToken = default)
        {
            // stream が null の場合は例外を投げる
            if (stream == null)
            {
                // null ストリームは許容しない
                throw new ArgumentNullException(nameof(stream), "Connect-RPC フレームを読み取るストリームが null です");
            }

            // 5 byte header を読み取るバッファを確保する
            var headerBuffer = new byte[FrameHeaderSize];

            // ヘッダーバッファを完全に読み取る（ReadExactlyAsync .NET 7+ / 下位互換のため手書き）
            var headerRead = await ReadExactlyAsync(stream, headerBuffer, 0, FrameHeaderSize, cancellationToken);

            // ストリーム終端（0 バイト読み取り）の場合は null を返す
            if (headerRead == 0)
            {
                // ストリーム終端を示す null を返す
                return null;
            }

            // ヘッダーが不完全な場合は例外を投げる
            if (headerRead < FrameHeaderSize)
            {
                // 不完全なヘッダーはプロトコル違反
                throw new InvalidDataException(
                    $"Connect-RPC フレームヘッダーが不完全です: 期待 {FrameHeaderSize} バイト, 実際 {headerRead} バイト");
            }

            // flags byte を読み取る（header[0]）
            var flags = headerBuffer[0];

            // ペイロード長を big-endian で読み取る（header[1..4]）
            var payloadLength = BinaryPrimitives.ReadInt32BigEndian(headerBuffer.AsSpan(1, 4));

            // ペイロード長が負または最大サイズ超過の場合は例外を投げる
            if (payloadLength < 0 || payloadLength > MaxFrameSize)
            {
                // 不正なペイロード長はプロトコル違反
                throw new InvalidDataException(
                    $"Connect-RPC フレームのペイロード長が不正です: {payloadLength}（上限: {MaxFrameSize}）");
            }

            // ペイロードを読み取るバッファを確保する（長さ 0 のフレームも許容する）
            var payload = new byte[payloadLength];

            // ペイロード長が 0 の場合はバッファを返す（空フレーム）
            if (payloadLength > 0)
            {
                // ペイロードを完全に読み取る
                var payloadRead = await ReadExactlyAsync(stream, payload, 0, payloadLength, cancellationToken);

                // ペイロードが不完全な場合は例外を投げる
                if (payloadRead < payloadLength)
                {
                    // 不完全なペイロードはプロトコル違反
                    throw new InvalidDataException(
                        $"Connect-RPC フレームペイロードが不完全です: 期待 {payloadLength} バイト, 実際 {payloadRead} バイト");
                }
            }

            // ConnectFrame を生成して返す
            return new ConnectFrame(flags, payload.AsMemory());
        }

        /// <summary>
        /// ConnectFrame を Stream に書き込む非同期メソッド
        /// </summary>
        /// <param name="stream">書き込み先ストリーム</param>
        /// <param name="frame">書き込む ConnectFrame</param>
        /// <param name="cancellationToken">キャンセレーショントークン</param>
        public static async Task WriteFrameAsync(
            Stream stream,
            ConnectFrame frame,
            CancellationToken cancellationToken = default)
        {
            // stream が null の場合は例外を投げる
            if (stream == null)
            {
                // null ストリームは許容しない
                throw new ArgumentNullException(nameof(stream), "Connect-RPC フレームを書き込むストリームが null です");
            }

            // 5 byte header バッファを確保する
            var header = new byte[FrameHeaderSize];

            // flags byte を書き込む
            header[0] = frame.Flags;

            // ペイロード長を big-endian で書き込む
            BinaryPrimitives.WriteInt32BigEndian(header.AsSpan(1, 4), frame.Length);

            // ヘッダーをストリームに書き込む
            await stream.WriteAsync(header, 0, FrameHeaderSize, cancellationToken);

            // ペイロードが存在する場合はストリームに書き込む
            if (frame.Length > 0)
            {
                // ペイロードをストリームに書き込む
                await stream.WriteAsync(frame.Payload.ToArray(), 0, frame.Length, cancellationToken);
            }
        }

        /// <summary>
        /// ストリームから指定バイト数を完全に読み取る補助メソッド
        /// Stream.ReadAsync は要求バイト数を返さない場合があるため繰り返し読み取る
        /// </summary>
        private static async Task<int> ReadExactlyAsync(
            Stream stream,
            byte[] buffer,
            int offset,
            int count,
            CancellationToken cancellationToken)
        {
            // 読み取り済みバイト数を追跡する
            var totalRead = 0;

            // 要求バイト数に達するまで繰り返し読み取る
            while (totalRead < count)
            {
                // ストリームからバイトを読み取る
                var read = await stream.ReadAsync(
                    buffer,
                    offset + totalRead,
                    count - totalRead,
                    cancellationToken);

                // 0 バイト読み取り（ストリーム終端）の場合はループを終了する
                if (read == 0)
                {
                    // ストリーム終端で終了する
                    break;
                }

                // 読み取り済みバイト数を加算する
                totalRead += read;
            }

            // 読み取り済みバイト数を返す
            return totalRead;
        }
    }
}
