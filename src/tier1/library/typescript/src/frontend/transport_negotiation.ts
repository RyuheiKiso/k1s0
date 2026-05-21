/**
 * transport_negotiation.ts — k1s0 tier1 Library TypeScript: Transport Negotiation Runtime
 * docs/03_概要設計/02_tier1設計方針/02_Library.md §Companion 役割 B に準拠する。
 * tier1 Server 系 Transport Adapter Layer と対をなすクライアント実装を提供する。
 * 8 adapter（sse_paired / long_poll / webhook / websocket / web_transport / messaging_bridge
 *            / grpc_web / connect_rpc）の chosen_transport capability negotiation を担う。
 * Rust frontend/transport_negotiation.rs / Go frontend/transport_negotiation.go /
 * C# Frontend/TransportNegotiation.cs と 4 言語等価強度を保つ。
 * OSS の transport 型（EventSource / WebSocket 等）を公開 API に一切露出しない。
 */

// ---- Transport adapter 種別定義 ----

/**
 * TransportKind は tier1 Server が選択可能な 8 transport adapter 種別を宣言する enum。
 * Gateway の chosen_transport フィールドと 1:1 対応する Library 独自語彙とする。
 * Rust TransportKind / Go TransportKind / C# TransportKind と 4 言語等価強度を保つ。
 */
// TransportKind 列挙型定義
export const enum TransportKind {
  // SsePaired: 既定。SSE フレーミング + 送信用 unary POST の組合せ
  // resume_token は SSE id: フィールド / Last-Event-ID ヘッダーで透過再接続する
  SsePaired = "sse_paired",
  // LongPoll: cursor 付き short poll の組合せ（SSE 非対応環境向け）
  LongPoll = "long_poll",
  // Webhook: レガシー側が HTTP サーバーとして受信する opt-in adapter
  Webhook = "webhook",
  // WebSocket: WebSocket ベース（HTTP Upgrade 対応環境向け）
  WebSocket = "websocket",
  // WebTransport: QUIC / WebTransport クライアント向け（v1 では opt-in adapter 扱い）
  WebTransport = "web_transport",
  // MessagingBridge: Kafka / AMQP の REST Proxy 越しに bidi メッセージを搬送する
  MessagingBridge = "messaging_bridge",
  // GrpcWeb: gRPC-Web プロトコル（HTTP/1.1 対応環境で gRPC を使用する場合）
  GrpcWeb = "grpc_web",
  // ConnectRpc: ConnectRPC プロトコル（gRPC / gRPC-Web との相互運用性が高い）
  ConnectRpc = "connect_rpc",
}

// ---- クライアント Capability 宣言 ----

/**
 * ClientCapabilities は Companion が起動時に Gateway に送出する capability 宣言を定義する型。
 * 利用可能な adapter 一覧 / TLS バージョン / inbound 可否 / max message size 等を宣言する。
 * Open RPC の client_capabilities 形式と互換性を保つ。
 * Rust ClientCapabilities / Go ClientCapabilities / C# ClientCapabilities と 4 言語等価強度を保つ。
 */
// ClientCapabilities 型定義
export interface ClientCapabilities {
  // availableTransports: クライアントが利用可能な transport adapter 一覧
  // Gateway はこの一覧から chosen_transport を選択する
  readonly availableTransports: readonly TransportKind[];
  // tlsMinVersion: クライアントがサポートする TLS 最低バージョン（例: "TLSv1.2" / "TLSv1.3"）
  readonly tlsMinVersion: string;
  // inboundCapable: Webhook adapter を受信できるかどうか（HTTP サーバーとして動作可能な場合 true）
  readonly inboundCapable: boolean;
  // maxMessageSizeBytes: 1 メッセージの最大サイズ（バイト）
  // 0 = 制限なし（実装側の OS / stack の制限に従う）
  readonly maxMessageSizeBytes: number;
  // resumeTokenSupport: resume_token による再接続をサポートするかどうか
  // SsePaired / WebSocket / WebTransport adapter で有効化する
  readonly resumeTokenSupport: boolean;
}

/**
 * defaultClientCapabilities は frontend 向け安全なデフォルト ClientCapabilities を返す。
 * SSE + LongPoll + WebSocket の 3 adapter をデフォルトで利用可能とする。
 */
// defaultClientCapabilities 関数: デフォルト ClientCapabilities を返す
export function defaultClientCapabilities(): ClientCapabilities {
  // frontend 向け安全なデフォルト設定を返す
  return {
    // SSE + LongPoll + WebSocket の 3 adapter をデフォルトで利用可能とする
    availableTransports: [
      TransportKind.SsePaired,
      TransportKind.LongPoll,
      TransportKind.WebSocket,
    ],
    // TLS 1.2 をデフォルト最低バージョンとする（TLS 1.3 推奨だが互換性のため 1.2 を下限とする）
    tlsMinVersion: "TLSv1.2",
    // デフォルトでは inbound を受け入れない（Webhook opt-in が必要）
    inboundCapable: false,
    // デフォルト最大メッセージサイズ: 4MB（大半の業務 RPC に十分な値）
    maxMessageSizeBytes: 4 * 1024 * 1024,
    // デフォルトで resume_token をサポートする（SsePaired の id: フィールドを使用する）
    resumeTokenSupport: true,
  };
}

// ---- 双方向チャンネル抽象 ----

/**
 * BidiMessage はアプリ側が送受信する双方向メッセージを宣言する型。
 * transport 種別を意識しない統一型（プロトコルバッファのバイト列を運ぶ）。
 * Rust BidiMessage / Go BidiMessage / C# BidiMessage と 4 言語等価強度を保つ。
 */
// BidiMessage 型定義
export interface BidiMessage {
  // payload: メッセージボディ（protobuf バイト列）
  readonly payload: Uint8Array;
  // sequenceId: メッセージ順序番号（resume_token 透過化に使用する）
  // 送信側が単調増加で付与し、受信側は順序検証に使用する
  readonly sequenceId: bigint;
  // resumeToken: セッション再接続トークン（transport 切替後の継続に使用する）
  // SsePaired の場合は SSE id: フィールド、WebSocket の場合は拡張ヘッダーで搬送する
  readonly resumeToken?: string | undefined;
}

// ---- BidiChannel interface ----

/**
 * BidiChannel は双方向 RPC セッションのアプリ向け抽象 interface を宣言する。
 * アプリ側は transport 種別を一切意識せず send / recv の API のみを使用する。
 * 再接続は BidiChannel 実装内部で透過的に処理される（アプリに漏れない）。
 * Rust BidiChannel / Go BidiChannel / C# IBidiChannel と 4 言語等価強度を保つ。
 */
// BidiChannel インターフェース定義
export interface BidiChannel {
  /**
   * send はアプリ側からメッセージを送信する。
   * transport が切り替わっても呼び出し元は継続できる（透過再接続）。
   * signal で中断制御を行う（AbortSignal）。
   */
  // send メソッド: メッセージを送信する
  send(payload: Uint8Array, signal?: AbortSignal): Promise<void>;

  /**
   * recv は受信メッセージを取得する非同期イテレーターを返す。
   * セッション終了まで受信し続ける（transport 切替による中断は内部でハンドルする）。
   */
  // recv メソッド: 受信メッセージを非同期イテレーターで返す
  recv(signal?: AbortSignal): AsyncIterable<BidiMessage>;

  /**
   * close はセッションをグレースフルに終了する（送信未完了メッセージをフラッシュする）。
   */
  // close メソッド: セッションを終了する
  close(): Promise<void>;

  /** chosenTransport は現在アクティブな transport 種別を返す（デバッグ / ログ用）。 */
  // chosenTransport プロパティ: アクティブな transport 種別
  readonly chosenTransport: TransportKind;

  /** capabilities は起動時に送出したクライアント capability 宣言を返す（デバッグ用）。 */
  // capabilities プロパティ: クライアント capability 宣言
  readonly capabilities: ClientCapabilities;
}

// ---- TransportNegotiationClient interface ----

/**
 * TransportNegotiationClient は transport negotiation を行って BidiChannel を開設する facade interface。
 * Gateway との negotiation フローを抽象化し、アプリ側は transport 選択結果を受け取るだけでよい。
 * Rust TransportNegotiationClient / Go TransportNegotiationClient /
 * C# ITransportNegotiationClient と 4 言語等価強度を保つ。
 */
// TransportNegotiationClient インターフェース定義
export interface TransportNegotiationClient {
  /**
   * negotiate は Gateway との capability exchange を行い、最適な BidiChannel を返す。
   * capabilities はクライアントが利用可能な adapter 宣言（ClientCapabilities）。
   * service はフルサービス名（"k1s0.tier1.WorkflowSvc" 等）。
   * method は RPC メソッド名（"StreamWorkflowEvents" 等の bidi streaming method）。
   * signal で中断制御を行う（AbortSignal）。
   */
  // negotiate メソッド: transport negotiation を行って BidiChannel を返す
  negotiate(
    service: string,
    method: string,
    capabilities: ClientCapabilities,
    signal?: AbortSignal,
  ): Promise<BidiChannel>;

  /** baseUrl は接続先 Gateway の base URL を返す（デバッグ / ログ用）。 */
  // baseUrl プロパティ: Gateway の base URL
  readonly baseUrl: string;
}

// ---- NegotiationResult 型 ----

/**
 * NegotiationResult は Gateway との capability exchange の結果を宣言する型。
 * アプリがデバッグ / ログ目的で確認できる情報を保持する。
 */
// NegotiationResult 型定義
export interface NegotiationResult {
  // chosenTransport: Gateway が選択した transport 種別
  readonly chosenTransport: TransportKind;
  // gatewayVersion: Gateway が報告したサービスバージョン
  readonly gatewayVersion: string;
  // negotiationLatencyMs: negotiation フロー全体のレイテンシ（ミリ秒）
  readonly negotiationLatencyMs: number;
  // resumeToken: 初期セッションの resume_token（再接続時に使用する）
  readonly resumeToken?: string | undefined;
}

// ---- in-memory BidiChannel 実装（テスト / ドライラン用） ----

/**
 * InMemoryBidiChannel は BidiChannel の in-memory stub 実装クラス。
 * テスト / ドライラン用に送受信メッセージをキュー経由でシミュレートする。
 * BidiChannel interface を実装し、transport 種別に依存しない。
 */
// InMemoryBidiChannel クラス定義（テスト用の公開クラス）
export class InMemoryBidiChannel implements BidiChannel {
  // #chosenTransport: シミュレートする transport 種別
  readonly #chosenTransport: TransportKind;
  // #caps: 起動時に送出したクライアント capability 宣言
  readonly #caps: ClientCapabilities;
  // #sendQueue: 送信メッセージキュー（テストが読む）
  readonly #sendQueue: Uint8Array[] = [];
  // #recvQueue: 受信メッセージキュー（テストが注入する）
  readonly #recvQueue: BidiMessage[] = [];
  // #recvResolvers: pending の recv 待機を解決するための resolver キュー
  readonly #recvResolvers: Array<(msg: BidiMessage | null) => void> = [];
  // #closed: close が呼ばれたかどうかのフラグ
  #closed: boolean = false;
  // #nextSeq: 送信メッセージのシーケンス番号カウンター
  #nextSeq: bigint = 0n;

  // コンストラクタ: chosenTransport と caps を受け取る
  constructor(chosenTransport: TransportKind, caps?: ClientCapabilities) {
    // transport 種別を設定する
    this.#chosenTransport = chosenTransport;
    // capabilities を設定する（デフォルト値を使用する場合は defaultClientCapabilities() を呼ぶ）
    this.#caps = caps ?? defaultClientCapabilities();
  }

  /** chosenTransport は現在アクティブな transport 種別を返す。 */
  // chosenTransport ゲッター
  get chosenTransport(): TransportKind {
    // #chosenTransport フィールドを返す
    return this.#chosenTransport;
  }

  /** capabilities は起動時に送出したクライアント capability 宣言を返す。 */
  // capabilities ゲッター
  get capabilities(): ClientCapabilities {
    // #caps フィールドを返す
    return this.#caps;
  }

  // send はアプリ側からメッセージをキューに送信する（テストが受け取る）。
  async send(payload: Uint8Array, signal?: AbortSignal): Promise<void> {
    // close 済みの場合はエラーを投げる
    if (this.#closed) {
      throw new Error("InMemoryBidiChannel: channel is closed");
    }
    // signal がキャンセルされている場合はエラーを投げる
    if (signal?.aborted === true) {
      throw new Error("InMemoryBidiChannel.send: aborted");
    }
    // キューにメッセージを追加する
    this.#sendQueue.push(payload);
    // シーケンス番号をインクリメントする
    this.#nextSeq++;
  }

  // recv は受信メッセージを非同期イテレーターで返す（テストが注入したメッセージを返す）。
  async *recv(signal?: AbortSignal): AsyncIterable<BidiMessage> {
    // close 済みの場合はイテレーションを終了する
    while (!this.#closed) {
      // signal がキャンセルされている場合はイテレーションを終了する
      if (signal?.aborted === true) {
        break;
      }
      // キューからメッセージを取得する（キューが空の場合は待機する）
      const msg = await this.#dequeue(signal);
      // null の場合はセッション終了（イテレーションを終了する）
      if (msg === null) {
        break;
      }
      // メッセージを yield する
      yield msg;
    }
  }

  // #dequeue はキューからメッセージを取得する内部メソッド。
  // キューが空の場合は次のメッセージが注入されるまで待機する。
  #dequeue(signal?: AbortSignal): Promise<BidiMessage | null> {
    // キューにメッセージが存在する場合は即座に返す
    const msg = this.#recvQueue.shift();
    // メッセージが存在する場合は即座に返す
    if (msg !== undefined) {
      return Promise.resolve(msg);
    }
    // キューが空の場合は Promise で待機する
    return new Promise<BidiMessage | null>((resolve) => {
      // signal がキャンセルされた場合は null で解決する
      const abortHandler = (): void => {
        // resolver キューから自分を削除する
        const idx = this.#recvResolvers.indexOf(resolve);
        // 削除する
        if (idx !== -1) {
          this.#recvResolvers.splice(idx, 1);
        }
        // null で解決する（キャンセルを通知する）
        resolve(null);
      };
      // signal の abort イベントを監視する（signal が定義されている場合のみ）
      if (signal !== undefined) {
        signal.addEventListener("abort", abortHandler, { once: true });
      }
      // resolver キューに追加する
      this.#recvResolvers.push((deliveredMsg) => {
        // abort イベントリスナーを削除する（signal が定義されている場合のみ）
        if (signal !== undefined) {
          signal.removeEventListener("abort", abortHandler);
        }
        // メッセージを解決する
        resolve(deliveredMsg);
      });
    });
  }

  // close はセッションをグレースフルに終了する。
  async close(): Promise<void> {
    // closed フラグを立てる
    this.#closed = true;
    // pending の resolver を null で解決する（recv ループを停止させる）
    for (const resolver of this.#recvResolvers.splice(0)) {
      // null で解決する（セッション終了を通知する）
      resolver(null);
    }
  }

  /**
   * injectMessage はテスト用にメッセージを受信キューに注入するヘルパーメソッド。
   * BidiChannel interface 外のメソッド（テスト用）。
   */
  // injectMessage メソッド: メッセージを受信キューに注入する
  injectMessage(msg: BidiMessage): void {
    // close 済みの場合はエラーを投げる
    if (this.#closed) {
      throw new Error("InMemoryBidiChannel: channel is closed");
    }
    // pending の resolver が存在する場合は直接解決する
    const resolver = this.#recvResolvers.shift();
    // resolver が存在する場合は直接メッセージを渡す
    if (resolver !== undefined) {
      // resolver を呼び出す
      resolver(msg);
      return;
    }
    // resolver が存在しない場合はキューに追加する
    this.#recvQueue.push(msg);
  }

  /**
   * sentMessages はテスト用に送信済みメッセージ配列を返すヘルパーメソッド。
   * BidiChannel interface 外のメソッド（テスト用）。
   */
  // sentMessages ゲッター: 送信済みメッセージ配列を返す
  get sentMessages(): readonly Uint8Array[] {
    // #sendQueue のコピーを返す
    return [...this.#sendQueue];
  }
}
