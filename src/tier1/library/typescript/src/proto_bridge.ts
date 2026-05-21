/**
 * proto_bridge.ts — k1s0 tier1 Library TypeScript: proto layer（3-layer split の第 3 層）
 * Bidi 適合仕様 3-layer split 規約（Y-bidi-library-split）に準拠する。
 *
 * ============================================================
 * 3-layer split 規約
 * ============================================================
 * 本モジュールは以下 3 層のうち「proto layer」に該当する。
 *
 *   Layer 1: SDK（公開 API）— index.ts から re-export される型
 *     - tier2 / tier3 / client が消費する公開 API を提供する。
 *     - KeyHandle / AuthContext / ObservabilityProvider 等の型が該当する。
 *     - index.ts / keyHandle.ts / authContext.ts 等が担当する。
 *
 *   Layer 2: Internal（server 内部 API）— export しない型
 *     - tier1 server 実装のみが使う内部 API を提供する。
 *     - TypeScript では export しないことでモジュール外からのアクセスを禁止する。
 *     - （将来） internal.ts に実装する。
 *
 *   Layer 3: ProtoBridge（Buf codegen 出力の薄い wrapper）— 本モジュール
 *     - buf generate 出力の protobuf 型 → Library 型への変換のみを担う。
 *     - proto 型を公開 API（index.ts）に re-export しない。
 *     - 本ファイル（proto_bridge.ts）が担当する。
 *
 * ============================================================
 * 実装方針（stub）
 * ============================================================
 * 本ファイルは Y-bidi-library-split YELLOW 解消のための stub 実装。
 * buf generate 出力型は src/tier1/schema/generated/typescript/ に配置予定。
 * P7（crosscutting）フェーズで Buf codegen（@bufbuild/protobuf）と接続して完全実装に移行する。
 *
 * 注意: 本ファイルは index.ts から re-export しない（proto 型の公開 API 露出禁止）。
 */

/**
 * BidiMessageProto は Buf codegen 出力 BidiMessage proto 型の薄い wrapper。
 * proto 型を直接公開 API に露出せず、Library 型への変換のみを担う。
 * 完全実装では @bufbuild/protobuf が生成した BidiMessage クラスを内包する。
 *
 * @internal proto layer: index.ts から re-export しないこと
 */
// BidiMessageProto クラス: proto 型の薄い wrapper（stub）
export class BidiMessageProto {
  // rawBytes: wire format の proto バイト列を保持する（stub）
  // Buf codegen 接続後は protobuf の BidiMessage オブジェクトに置き換える
  private readonly rawBytes: Uint8Array;

  /**
   * new は raw proto bytes から BidiMessageProto を構築する stub コンストラクタ。
   * Buf codegen 接続後は BidiMessage.fromBinary() 等に置き換える。
   * @param rawBytes wire format の proto バイト列
   */
  // コンストラクタ: rawBytes を受け取って内部状態を初期化する
  constructor(rawBytes: Uint8Array) {
    // rawBytes を内部フィールドに保持する（コピーして所有権を明確にする）
    this.rawBytes = rawBytes;
  }

  /**
   * intoRawBytes は raw proto bytes を返す。
   * Library 型への変換が不要な場合のエスケープハッチ（proto_bridge モジュール内部専用）。
   * @returns wire format の proto バイト列
   */
  // intoRawBytes: 保持している rawBytes を返す（proto_bridge 内部専用）
  public intoRawBytes(): Uint8Array {
    // rawBytes フィールドの値を返す（参照を返す; 変更禁止）
    return this.rawBytes;
  }
}

/**
 * ResumeTokenProto は Buf codegen 出力 ResumeToken proto 型の薄い wrapper。
 * tls_disconnect / resume_after_disconnect scenario の resume_token を表現する。
 * 完全実装では @bufbuild/protobuf が生成した ResumeToken クラスを内包する。
 *
 * @internal proto layer: index.ts から re-export しないこと
 */
// ResumeTokenProto クラス: resume_token proto 型の薄い wrapper（stub）
export class ResumeTokenProto {
  // hlcMicros: HLC タイムスタンプ（マイクロ秒）を保持する（stub）
  // wall-clock ではなく HLC を必ず使用する（src/CLAUDE.md §wall-clock TTL 禁止に準拠）
  private readonly hlcMicros: bigint;

  // sessionId: セッション ID を保持する（stub）
  private readonly sessionId: string;

  /**
   * ResumeTokenProto を HLC タイムスタンプとセッション ID から構築する stub コンストラクタ。
   * @param hlcMicros HLC タイムスタンプ（マイクロ秒、Number 精度超過のため bigint を使用する）
   * @param sessionId セッション ID 文字列
   */
  // コンストラクタ: hlcMicros と sessionId を受け取って初期化する
  constructor(hlcMicros: bigint, sessionId: string) {
    // hlcMicros を内部フィールドに保持する
    this.hlcMicros = hlcMicros;
    // sessionId を内部フィールドに保持する（空文字列は許容するが null/undefined は禁止）
    this.sessionId = sessionId;
  }

  /**
   * getHlcMicros は HLC タイムスタンプを返す（resume_token_hlc_valid assertion に使用する）。
   * @returns HLC タイムスタンプ（マイクロ秒）
   */
  // getHlcMicros: hlcMicros フィールドの値を返す getter
  public getHlcMicros(): bigint {
    // hlcMicros フィールドの値を返す
    return this.hlcMicros;
  }

  /**
   * getSessionId はセッション ID を返す（resume_after_disconnect assertion に使用する）。
   * @returns セッション ID 文字列
   */
  // getSessionId: sessionId フィールドの値を返す getter
  public getSessionId(): string {
    // sessionId フィールドの値を返す
    return this.sessionId;
  }
}
