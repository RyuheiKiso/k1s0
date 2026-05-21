// ProtoBridge.cs — k1s0 tier1 Library C#: proto layer（3-layer split の第 3 層）
// Bidi 適合仕様 3-layer split 規約（Y-bidi-library-split）に準拠する。
//
// ============================================================
// 3-layer split 規約
// ============================================================
// 本クラス群は以下 3 層のうち「proto layer」に該当する。
//
//   Layer 1: SDK（公開 API）— K1s0.Tier1 名前空間
//     - tier2 / tier3 / client が消費する公開 API を提供する。
//     - IKeyHandle / AuthContext / ICache 等の型が該当する。
//     - 本クレートの KeyHandle.cs / AuthContext.cs / ICache.cs 等が担当する。
//
//   Layer 2: Internal（server 内部 API）— K1s0.Tier1.Internal 名前空間
//     - tier1 server 実装のみが使う内部 API を提供する。
//     - internal アクセス修飾子で外部アセンブリからのアクセスを禁止する。
//     - （将来） K1s0.Tier1.Internal 名前空間に実装する。
//
//   Layer 3: ProtoBridge（Buf codegen 出力の薄い wrapper）— K1s0.Tier1.ProtoBridge 名前空間
//     - buf generate 出力の protobuf 型 → Library 型への変換のみを担う。
//     - proto 型を公開 API シグネチャに露出しない（internal スコープに限定する）。
//     - 本ファイル（ProtoBridge.cs）が担当する。
//
// ============================================================
// 実装方針（stub）
// ============================================================
// 本ファイルは Y-bidi-library-split YELLOW 解消のための stub 実装。
// buf generate 出力型は src/tier1/schema/generated/csharp/ に配置予定。
// P7（crosscutting）フェーズで Buf codegen と接続して完全実装に移行する。

// System: 基本型（byte[] / ArgumentNullException 等）
using System;

// K1s0.Tier1.ProtoBridge 名前空間: proto layer の stub 実装を収容する
namespace K1s0.Tier1.ProtoBridge;

/// <summary>
/// BidiMessageProto は Buf codegen 出力 BidiMessage proto 型の薄い wrapper。
/// proto 型を直接公開 API に露出せず、Library 型への変換のみを担う。
/// 完全実装では Google.Protobuf が生成した BidiMessage クラスを内包する。
/// </summary>
// internal: 外部アセンブリからのアクセスを禁止する（proto 型を公開しない）
internal sealed class BidiMessageProto
{
    /// <summary>
    /// RawBytes は wire format の proto バイト列を保持する（stub）。
    /// Buf codegen 接続後は BidiMessage proto オブジェクトに置き換える。
    /// </summary>
    // RawBytes プロパティ: 読み取り専用で raw proto bytes を保持する
    public ReadOnlyMemory<byte> RawBytes { get; }

    /// <summary>
    /// BidiMessageProto を raw proto bytes から構築する stub コンストラクタ。
    /// Buf codegen 接続後は Google.Protobuf.MessageParser 等に置き換える。
    /// </summary>
    /// <param name="rawBytes">wire format の proto バイト列</param>
    // コンストラクタ: rawBytes が null の場合は ArgumentNullException を投げる
    public BidiMessageProto(ReadOnlyMemory<byte> rawBytes)
    {
        // rawBytes を保持する（null チェックは ReadOnlyMemory<byte> が値型のため不要）
        RawBytes = rawBytes;
    }
}

/// <summary>
/// ResumeTokenProto は Buf codegen 出力 ResumeToken proto 型の薄い wrapper。
/// tls_disconnect / resume_after_disconnect scenario の resume_token を表現する。
/// 完全実装では Google.Protobuf が生成した ResumeToken クラスを内包する。
/// </summary>
// internal: 外部アセンブリからのアクセスを禁止する（proto 型を公開しない）
internal sealed class ResumeTokenProto
{
    /// <summary>
    /// HlcMicros は HLC タイムスタンプ（マイクロ秒）を保持する（stub）。
    /// wall-clock ではなく HLC を必ず使用する（src/CLAUDE.md §wall-clock TTL 禁止に準拠）。
    /// </summary>
    // HlcMicros プロパティ: resume_token_hlc_valid assertion に使用する HLC タイムスタンプ
    public ulong HlcMicros { get; }

    /// <summary>
    /// SessionId はセッション ID を保持する（stub）。
    /// resume_after_disconnect assertion に使用する。
    /// </summary>
    // SessionId プロパティ: 読み取り専用のセッション ID 文字列
    public string SessionId { get; }

    /// <summary>
    /// ResumeTokenProto を HLC タイムスタンプとセッション ID から構築する stub コンストラクタ。
    /// </summary>
    /// <param name="hlcMicros">HLC タイムスタンプ（マイクロ秒）</param>
    /// <param name="sessionId">セッション ID 文字列</param>
    // コンストラクタ: hlcMicros と sessionId を初期化する
    public ResumeTokenProto(ulong hlcMicros, string sessionId)
    {
        // hlcMicros を HlcMicros プロパティに設定する
        HlcMicros = hlcMicros;
        // sessionId が null の場合は ArgumentNullException を投げる（型安全性の物理保証）
        SessionId = sessionId ?? throw new ArgumentNullException(nameof(sessionId),
            "ResumeTokenProto の sessionId は null 禁止");
    }
}
