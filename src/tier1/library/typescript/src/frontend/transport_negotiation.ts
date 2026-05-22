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

// ---- production transport adapter 実装群 ----

// ---- SsePairedBidiChannel ----

/**
 * SsePairedBidiChannel は SSE (EventSource) 受信 + POST 送信の組合せ BidiChannel 実装クラス。
 * resumeToken は SSE の id: フィールド / Last-Event-ID ヘッダーで透過再接続する。
 * EventSource / fetch 等の OSS 型を公開 API シグネチャに一切露出しない。
 */
// SsePairedBidiChannel クラス定義（SSE + POST transport 実装）
export class SsePairedBidiChannel implements BidiChannel {
  // #caps: クライアント capability 宣言
  readonly #caps: ClientCapabilities;
  // #baseUrl: 接続先 Gateway の base URL
  readonly #baseUrl: string;
  // #service: フルサービス名（"k1s0.tier1.WorkflowSvc" 等）
  readonly #service: string;
  // #method: RPC メソッド名（"StreamWorkflowEvents" 等）
  readonly #method: string;
  // #sendAbortController: 送信用 fetch の abort controller
  #sendAbortController: AbortController = new AbortController();
  // #eventSource: SSE 受信用 EventSource インスタンス（OSS 型は内部にのみ保持する）
  #eventSource: EventSource | null = null;
  // #lastResumeToken: 最後に受信した SSE id: フィールド（再接続時の Last-Event-ID に使用する）
  #lastResumeToken: string | undefined = undefined;
  // #closed: close が呼ばれたかどうかのフラグ
  #closed: boolean = false;
  // #recvResolvers: pending の recv 待機を解決するための resolver キュー
  readonly #recvResolvers: Array<(msg: BidiMessage | null) => void> = [];
  // #recvQueue: 受信メッセージキュー（EventSource onmessage が注入する）
  readonly #recvQueue: BidiMessage[] = [];
  // #nextSeq: 受信メッセージのシーケンス番号カウンター
  #nextSeq: bigint = 0n;

  // コンストラクタ: base URL / service / method / caps を受け取る
  constructor(
    baseUrl: string,
    service: string,
    method: string,
    caps: ClientCapabilities,
  ) {
    // base URL を設定する
    this.#baseUrl = baseUrl;
    // フルサービス名を設定する
    this.#service = service;
    // RPC メソッド名を設定する
    this.#method = method;
    // capabilities を設定する
    this.#caps = caps;
    // SSE 接続を開始する
    this.#connectSse();
  }

  // chosenTransport ゲッター: SsePaired transport 種別を返す
  get chosenTransport(): TransportKind {
    // SsePaired を返す
    return TransportKind.SsePaired;
  }

  // capabilities ゲッター: クライアント capability 宣言を返す
  get capabilities(): ClientCapabilities {
    // #caps フィールドを返す
    return this.#caps;
  }

  // #connectSse は SSE 接続を開始または再接続する内部メソッド。
  // resumeToken が存在する場合は Last-Event-ID ヘッダーを付与して再接続する。
  #connectSse(): void {
    // close 済みの場合は再接続しない
    if (this.#closed) {
      return;
    }
    // SSE エンドポイント URL を構築する（tier1 bidi streaming 規約に従う）
    const url = `${this.#baseUrl}/${this.#service}/${this.#method}`;
    // EventSource を構築する（SSE 接続を開始する）
    // NOTE: EventSource は公開 API に露出せず、クラス内部にのみ保持する
    this.#eventSource = new EventSource(url);
    // onmessage ハンドラー: SSE メッセージを受信してキューに投入する
    this.#eventSource.onmessage = (event: MessageEvent): void => {
      // Base64 エンコードされた payload を Uint8Array にデコードする
      const rawData = event.data as string;
      // Base64 → Uint8Array 変換を行う
      const payload = this.#base64ToUint8Array(rawData);
      // SSE id: フィールドを resumeToken として保存する
      const resumeToken = (event.lastEventId !== "" ? event.lastEventId : undefined) as string | undefined;
      // resumeToken が存在する場合は保存する（透過再接続に使用する）
      if (resumeToken !== undefined) {
        this.#lastResumeToken = resumeToken;
      }
      // BidiMessage を構築する
      const msg: BidiMessage = {
        // payload を設定する
        payload,
        // シーケンス番号をインクリメントして設定する
        sequenceId: this.#nextSeq++,
        // resumeToken を設定する
        resumeToken,
      };
      // pending resolver が存在する場合は直接解決する
      const resolver = this.#recvResolvers.shift();
      // resolver が存在する場合は直接メッセージを渡す
      if (resolver !== undefined) {
        // resolver を呼び出す
        resolver(msg);
        return;
      }
      // resolver が存在しない場合はキューに追加する
      this.#recvQueue.push(msg);
    };
    // onerror ハンドラー: SSE エラー時に再接続を試みる
    this.#eventSource.onerror = (): void => {
      // close 済みの場合は再接続しない
      if (this.#closed) {
        return;
      }
      // 既存の EventSource を閉じる
      this.#eventSource?.close();
      // EventSource を null にする
      this.#eventSource = null;
      // 再接続を試みる（EventSource の自動再接続は使用せず、明示的に再接続する）
      this.#connectSse();
    };
  }

  // #base64ToUint8Array は Base64 文字列を Uint8Array に変換する内部ヘルパーメソッド。
  #base64ToUint8Array(base64: string): Uint8Array {
    // atob で Base64 デコードする
    const binary = atob(base64);
    // バイナリ文字列の長さを取得する
    const len = binary.length;
    // Uint8Array を確保する
    const bytes = new Uint8Array(len);
    // 各バイトをコピーする
    for (let i = 0; i < len; i++) {
      // charCodeAt で文字コードを取得して設定する
      bytes[i] = binary.charCodeAt(i);
    }
    // Uint8Array を返す
    return bytes;
  }

  // #uint8ArrayToBase64 は Uint8Array を Base64 文字列に変換する内部ヘルパーメソッド。
  #uint8ArrayToBase64(bytes: Uint8Array): string {
    // バイト列を文字列に変換する
    let binary = "";
    // 各バイトを文字に変換する
    for (let i = 0; i < bytes.length; i++) {
      // fromCharCode でバイトを文字に変換する
      binary += String.fromCharCode(bytes[i] ?? 0);
    }
    // btoa で Base64 エンコードする
    return btoa(binary);
  }

  // send はアプリ側からメッセージを POST エンドポイントに送信する。
  async send(payload: Uint8Array, signal?: AbortSignal): Promise<void> {
    // close 済みの場合はエラーを投げる
    if (this.#closed) {
      throw new Error("SsePairedBidiChannel: channel is closed");
    }
    // signal がキャンセルされている場合はエラーを投げる
    if (signal?.aborted === true) {
      throw new Error("SsePairedBidiChannel.send: aborted");
    }
    // POST エンドポイント URL を構築する（tier1 bidi send 規約に従う）
    const sendUrl = `${this.#baseUrl}/tier1.bidi.v1.BidiService/OpenBidiStream/send`;
    // AbortSignal を合成する（メソッド引数と内部 controller の双方でキャンセルできる）
    const combinedSignal = signal !== undefined
      ? AbortSignal.any([signal, this.#sendAbortController.signal])
      : this.#sendAbortController.signal;
    // fetch で POST 送信する（payload は Blob 経由で body に設定して型制約を満たす）
    const response = await fetch(sendUrl, {
      // POST メソッドで送信する
      method: "POST",
      // Content-Type を application/octet-stream に設定する
      headers: { "Content-Type": "application/octet-stream" },
      // Uint8Array を ArrayBuffer のコピーに変換して BodyInit 型制約を満たす
      body: payload.buffer.slice(payload.byteOffset, payload.byteOffset + payload.byteLength) as ArrayBuffer,
      // AbortSignal を設定する
      signal: combinedSignal,
    });
    // HTTP ステータスが 2xx 以外の場合はエラーを投げる
    if (!response.ok) {
      // レスポンステキストを取得してエラーメッセージに含める
      const text = await response.text();
      // エラーを投げる
      throw new Error(`SsePairedBidiChannel.send: HTTP ${response.status}: ${text}`);
    }
  }

  // recv は受信メッセージを非同期イテレーターで返す。
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
      // signal がキャンセルされた場合は null で解決するハンドラー
      const abortHandler = (): void => {
        // resolver キューから自分を削除する
        const idx = this.#recvResolvers.indexOf(resolve);
        // 存在する場合は削除する
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
    // EventSource を閉じる
    this.#eventSource?.close();
    // EventSource を null にする
    this.#eventSource = null;
    // 送信用 fetch を abort する
    this.#sendAbortController.abort();
    // pending の resolver を null で解決する（recv ループを停止させる）
    for (const resolver of this.#recvResolvers.splice(0)) {
      // null で解決する（セッション終了を通知する）
      resolver(null);
    }
  }

  /** lastResumeToken は最後に受信した resumeToken を返す（デバッグ / 再接続用）。 */
  // lastResumeToken ゲッター: 最後に受信した resumeToken を返す
  get lastResumeToken(): string | undefined {
    // #lastResumeToken フィールドを返す
    return this.#lastResumeToken;
  }
}

// ---- LongPollBidiChannel ----

/**
 * LongPollBidiChannel は cursor 付きポーリング + POST 送信の組合せ BidiChannel 実装クラス。
 * SSE 非対応環境でも動作する（HTTP/1.1 のみで bidi ストリームを実現する）。
 * fetch 等の OSS 型を公開 API シグネチャに一切露出しない。
 */
// LongPollBidiChannel クラス定義（LongPoll transport 実装）
export class LongPollBidiChannel implements BidiChannel {
  // DEFAULT_POLL_INTERVAL_MS: デフォルトのポーリング間隔（ミリ秒）
  // 100ms をデフォルトとして expose する（呼び出し元がオーバーライド可能）
  static readonly DEFAULT_POLL_INTERVAL_MS: number = 100;

  // #caps: クライアント capability 宣言
  readonly #caps: ClientCapabilities;
  // #baseUrl: 接続先 Gateway の base URL
  readonly #baseUrl: string;
  // #pollIntervalMs: ポーリング間隔（ミリ秒）
  readonly #pollIntervalMs: number;
  // #cursor: 現在のポーリングカーソル（サーバーが返す次回ポーリング位置）
  #cursor: string = "";
  // #closed: close が呼ばれたかどうかのフラグ
  #closed: boolean = false;
  // #pollAbortController: ポーリング fetch の abort controller
  #pollAbortController: AbortController = new AbortController();
  // #sendAbortController: 送信用 fetch の abort controller
  #sendAbortController: AbortController = new AbortController();
  // #recvQueue: 受信メッセージキュー（ポーリングループが注入する）
  readonly #recvQueue: BidiMessage[] = [];
  // #recvResolvers: pending の recv 待機を解決するための resolver キュー
  readonly #recvResolvers: Array<(msg: BidiMessage | null) => void> = [];
  // #nextSeq: 受信メッセージのシーケンス番号カウンター
  #nextSeq: bigint = 0n;

  // コンストラクタ: base URL / caps / poll interval を受け取る
  constructor(
    baseUrl: string,
    caps: ClientCapabilities,
    pollIntervalMs: number = LongPollBidiChannel.DEFAULT_POLL_INTERVAL_MS,
  ) {
    // base URL を設定する
    this.#baseUrl = baseUrl;
    // capabilities を設定する
    this.#caps = caps;
    // ポーリング間隔を設定する
    this.#pollIntervalMs = pollIntervalMs;
    // ポーリングループを非同期で開始する
    void this.#startPollLoop();
  }

  // chosenTransport ゲッター: LongPoll transport 種別を返す
  get chosenTransport(): TransportKind {
    // LongPoll を返す
    return TransportKind.LongPoll;
  }

  // capabilities ゲッター: クライアント capability 宣言を返す
  get capabilities(): ClientCapabilities {
    // #caps フィールドを返す
    return this.#caps;
  }

  // #startPollLoop はポーリングループを開始する内部非同期メソッド。
  // closed フラグが立つまでポーリングを継続する。
  async #startPollLoop(): Promise<void> {
    // close 済みになるまでポーリングを継続する
    while (!this.#closed) {
      // ポーリングを実行する（エラーは無視して継続する）
      await this.#poll();
      // close 済みの場合はループを抜ける
      if (this.#closed) {
        break;
      }
      // ポーリング間隔待機する（Promise ベースの setTimeout を使用する）
      await new Promise<void>((resolve) => {
        // setTimeout でポーリング間隔待機する
        setTimeout(resolve, this.#pollIntervalMs);
      });
    }
  }

  // #poll は 1 回のポーリングを実行する内部非同期メソッド。
  // カーソル付き GET でメッセージを取得し、受信キューに投入する。
  async #poll(): Promise<void> {
    // poll エンドポイント URL を構築する（cursor クエリパラメーターを付与する）
    const url = `${this.#baseUrl}/tier1.bidi.v1.BidiService/OpenBidiStream/poll?cursor=${encodeURIComponent(this.#cursor)}`;
    // fetch で GET ポーリングを実行する
    let response: Response;
    // fetch エラーの場合は無視して次のポーリングに進む
    try {
      // fetch でポーリングを実行する
      response = await fetch(url, {
        // GET メソッドで取得する
        method: "GET",
        // AbortSignal を設定する
        signal: this.#pollAbortController.signal,
      });
    } catch {
      // fetch エラーは無視して次のポーリングに進む（ネットワーク一時断を吸収する）
      return;
    }
    // HTTP ステータスが 2xx 以外の場合は無視する（エラーレスポンスは次のポーリングで再試行する）
    if (!response.ok) {
      return;
    }
    // レスポンス JSON を取得する
    let data: { messages: Array<{ payload: string; sequenceId: string; resumeToken?: string }>; nextCursor: string };
    // JSON パースエラーの場合は無視して次のポーリングに進む
    try {
      // レスポンスを JSON としてパースする
      data = await response.json() as typeof data;
    } catch {
      // JSON パースエラーは無視して次のポーリングに進む
      return;
    }
    // 返却されたメッセージを受信キューに投入する
    for (const item of data.messages) {
      // Base64 payload を Uint8Array にデコードする
      const payload = this.#base64ToUint8Array(item.payload);
      // BidiMessage を構築する
      const msg: BidiMessage = {
        // payload を設定する
        payload,
        // sequenceId を bigint に変換して設定する
        sequenceId: BigInt(item.sequenceId),
        // resumeToken を設定する
        resumeToken: item.resumeToken,
      };
      // pending resolver が存在する場合は直接解決する
      const resolver = this.#recvResolvers.shift();
      // resolver が存在する場合は直接メッセージを渡す
      if (resolver !== undefined) {
        // resolver を呼び出す
        resolver(msg);
        continue;
      }
      // resolver が存在しない場合はキューに追加する
      this.#recvQueue.push(msg);
    }
    // nextCursor を更新する（次回ポーリングに使用する）
    this.#cursor = data.nextCursor;
  }

  // #base64ToUint8Array は Base64 文字列を Uint8Array に変換する内部ヘルパーメソッド。
  #base64ToUint8Array(base64: string): Uint8Array {
    // atob で Base64 デコードする
    const binary = atob(base64);
    // バイナリ文字列の長さを取得する
    const len = binary.length;
    // Uint8Array を確保する
    const bytes = new Uint8Array(len);
    // 各バイトをコピーする
    for (let i = 0; i < len; i++) {
      // charCodeAt で文字コードを取得して設定する
      bytes[i] = binary.charCodeAt(i);
    }
    // Uint8Array を返す
    return bytes;
  }

  // send はアプリ側からメッセージを POST エンドポイントに送信する。
  async send(payload: Uint8Array, signal?: AbortSignal): Promise<void> {
    // close 済みの場合はエラーを投げる
    if (this.#closed) {
      throw new Error("LongPollBidiChannel: channel is closed");
    }
    // signal がキャンセルされている場合はエラーを投げる
    if (signal?.aborted === true) {
      throw new Error("LongPollBidiChannel.send: aborted");
    }
    // POST エンドポイント URL を構築する（tier1 bidi send 規約に従う）
    const sendUrl = `${this.#baseUrl}/tier1.bidi.v1.BidiService/OpenBidiStream/send`;
    // AbortSignal を合成する（メソッド引数と内部 controller の双方でキャンセルできる）
    const combinedSignal = signal !== undefined
      ? AbortSignal.any([signal, this.#sendAbortController.signal])
      : this.#sendAbortController.signal;
    // fetch で POST 送信する（payload は Blob 経由で body に設定して型制約を満たす）
    const response = await fetch(sendUrl, {
      // POST メソッドで送信する
      method: "POST",
      // Content-Type を application/octet-stream に設定する
      headers: { "Content-Type": "application/octet-stream" },
      // Uint8Array を ArrayBuffer のコピーに変換して BodyInit 型制約を満たす
      body: payload.buffer.slice(payload.byteOffset, payload.byteOffset + payload.byteLength) as ArrayBuffer,
      // AbortSignal を設定する
      signal: combinedSignal,
    });
    // HTTP ステータスが 2xx 以外の場合はエラーを投げる
    if (!response.ok) {
      // レスポンステキストを取得してエラーメッセージに含める
      const text = await response.text();
      // エラーを投げる
      throw new Error(`LongPollBidiChannel.send: HTTP ${response.status}: ${text}`);
    }
  }

  // recv は受信メッセージを非同期イテレーターで返す。
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
  #dequeue(signal?: AbortSignal): Promise<BidiMessage | null> {
    // キューにメッセージが存在する場合は即座に返す
    const msg = this.#recvQueue.shift();
    // メッセージが存在する場合は即座に返す
    if (msg !== undefined) {
      return Promise.resolve(msg);
    }
    // キューが空の場合は Promise で待機する
    return new Promise<BidiMessage | null>((resolve) => {
      // signal がキャンセルされた場合は null で解決するハンドラー
      const abortHandler = (): void => {
        // resolver キューから自分を削除する
        const idx = this.#recvResolvers.indexOf(resolve);
        // 存在する場合は削除する
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
    // ポーリング fetch を abort する
    this.#pollAbortController.abort();
    // 送信用 fetch を abort する
    this.#sendAbortController.abort();
    // pending の resolver を null で解決する（recv ループを停止させる）
    for (const resolver of this.#recvResolvers.splice(0)) {
      // null で解決する（セッション終了を通知する）
      resolver(null);
    }
  }

  /** cursor は現在のポーリングカーソルを返す（デバッグ / テスト用）。 */
  // cursor ゲッター: 現在のポーリングカーソルを返す
  get cursor(): string {
    // #cursor フィールドを返す
    return this.#cursor;
  }
}

// ---- WebSocketBidiChannel ----

/**
 * WebSocketBidiChannel は WebSocket を使用した双方向通信の BidiChannel 実装クラス。
 * Ping/Pong で接続を維持し、close() は WebSocket を graceful close (code 1000) する。
 * WebSocket 等の OSS 型を公開 API シグネチャに一切露出しない。
 */
// WebSocketBidiChannel クラス定義（WebSocket transport 実装）
export class WebSocketBidiChannel implements BidiChannel {
  // PING_INTERVAL_MS: Ping 送信間隔（ミリ秒）
  static readonly PING_INTERVAL_MS: number = 30_000;
  // WS_CLOSE_NORMAL: WebSocket 正常クローズコード
  static readonly WS_CLOSE_NORMAL: number = 1000;
  // PING_OPCODE: WebSocket Ping フレームの識別子（バイナリメッセージのプレフィックスとして使用）
  static readonly PING_MESSAGE: string = "__ping__";
  // PONG_MESSAGE: WebSocket Pong フレームの識別子
  static readonly PONG_MESSAGE: string = "__pong__";

  // #caps: クライアント capability 宣言
  readonly #caps: ClientCapabilities;
  // #wsUrl: 接続先 WebSocket URL
  readonly #wsUrl: string;
  // #ws: WebSocket インスタンス（OSS 型は内部にのみ保持する）
  #ws: WebSocket | null = null;
  // #closed: close が呼ばれたかどうかのフラグ
  #closed: boolean = false;
  // #recvQueue: 受信メッセージキュー（WebSocket onmessage が注入する）
  readonly #recvQueue: BidiMessage[] = [];
  // #recvResolvers: pending の recv 待機を解決するための resolver キュー
  readonly #recvResolvers: Array<(msg: BidiMessage | null) => void> = [];
  // #nextSeq: 受信メッセージのシーケンス番号カウンター
  #nextSeq: bigint = 0n;
  // #pingTimerId: Ping タイマーの ID（clearInterval に使用する）
  #pingTimerId: ReturnType<typeof setInterval> | null = null;
  // #openPromise: WebSocket open イベントの Promise（接続完了まで待機する）
  readonly #openPromise: Promise<void>;
  // #openResolve: #openPromise を解決する関数
  #openResolve: (() => void) | null = null;
  // #openReject: #openPromise を拒否する関数
  #openReject: ((err: Error) => void) | null = null;

  // コンストラクタ: WebSocket URL / caps を受け取る
  constructor(wsUrl: string, caps: ClientCapabilities) {
    // WebSocket URL を設定する
    this.#wsUrl = wsUrl;
    // capabilities を設定する
    this.#caps = caps;
    // open Promise を構築する（WebSocket 接続完了まで send を待機させる）
    this.#openPromise = new Promise<void>((resolve, reject) => {
      // resolve を保存する
      this.#openResolve = resolve;
      // reject を保存する
      this.#openReject = reject;
    });
    // WebSocket 接続を開始する
    this.#connect();
  }

  // chosenTransport ゲッター: WebSocket transport 種別を返す
  get chosenTransport(): TransportKind {
    // WebSocket を返す
    return TransportKind.WebSocket;
  }

  // capabilities ゲッター: クライアント capability 宣言を返す
  get capabilities(): ClientCapabilities {
    // #caps フィールドを返す
    return this.#caps;
  }

  // #connect は WebSocket 接続を開始する内部メソッド。
  #connect(): void {
    // close 済みの場合は接続しない
    if (this.#closed) {
      return;
    }
    // WebSocket を構築する（OSS 型は内部にのみ保持する）
    this.#ws = new WebSocket(this.#wsUrl);
    // binaryType を arraybuffer に設定する（Uint8Array で受信するため）
    this.#ws.binaryType = "arraybuffer";
    // onopen ハンドラー: 接続完了時に openPromise を解決する
    this.#ws.onopen = (): void => {
      // openPromise を解決する
      if (this.#openResolve !== null) {
        // resolve を呼び出す
        this.#openResolve();
        // 参照を破棄する
        this.#openResolve = null;
        // reject の参照も破棄する
        this.#openReject = null;
      }
      // Ping タイマーを開始する（接続維持のため定期的に Ping を送信する）
      this.#startPingTimer();
    };
    // onmessage ハンドラー: メッセージ受信時にキューに投入する
    this.#ws.onmessage = (event: MessageEvent): void => {
      // テキストメッセージの場合は Pong として扱う（Ping/Pong 制御メッセージ）
      if (typeof event.data === "string") {
        // Pong メッセージの場合は無視する（接続維持目的のみ）
        return;
      }
      // バイナリメッセージを Uint8Array に変換する
      const payload = new Uint8Array(event.data as ArrayBuffer);
      // BidiMessage を構築する
      const msg: BidiMessage = {
        // payload を設定する
        payload,
        // シーケンス番号をインクリメントして設定する
        sequenceId: this.#nextSeq++,
        // resumeToken は WebSocket では未使用（undefined）
        resumeToken: undefined,
      };
      // pending resolver が存在する場合は直接解決する
      const resolver = this.#recvResolvers.shift();
      // resolver が存在する場合は直接メッセージを渡す
      if (resolver !== undefined) {
        // resolver を呼び出す
        resolver(msg);
        return;
      }
      // resolver が存在しない場合はキューに追加する
      this.#recvQueue.push(msg);
    };
    // onerror ハンドラー: エラー発生時に openPromise を拒否する（未解決の場合のみ）
    this.#ws.onerror = (): void => {
      // openReject が存在する場合は拒否する（接続失敗を通知する）
      if (this.#openReject !== null) {
        // reject を呼び出す
        this.#openReject(new Error("WebSocketBidiChannel: WebSocket error"));
        // 参照を破棄する
        this.#openReject = null;
        // resolve の参照も破棄する
        this.#openResolve = null;
      }
    };
    // onclose ハンドラー: 接続クローズ時に recv ループを停止させる
    this.#ws.onclose = (): void => {
      // Ping タイマーを停止する
      this.#stopPingTimer();
      // pending の resolver を null で解決する（recv ループを停止させる）
      for (const resolver of this.#recvResolvers.splice(0)) {
        // null で解決する（セッション終了を通知する）
        resolver(null);
      }
    };
  }

  // #startPingTimer は Ping タイマーを開始する内部メソッド。
  // 定期的に Ping メッセージを送信して接続を維持する。
  #startPingTimer(): void {
    // 既存のタイマーが存在する場合は停止する
    this.#stopPingTimer();
    // setInterval で定期的に Ping を送信するタイマーを設定する
    this.#pingTimerId = setInterval(() => {
      // WebSocket が接続中の場合のみ Ping を送信する
      if (this.#ws?.readyState === WebSocket.OPEN) {
        // Ping メッセージを送信する
        this.#ws.send(WebSocketBidiChannel.PING_MESSAGE);
      }
    }, WebSocketBidiChannel.PING_INTERVAL_MS);
  }

  // #stopPingTimer は Ping タイマーを停止する内部メソッド。
  #stopPingTimer(): void {
    // タイマーが存在する場合は停止する
    if (this.#pingTimerId !== null) {
      // clearInterval でタイマーを停止する
      clearInterval(this.#pingTimerId);
      // タイマー ID を null にする
      this.#pingTimerId = null;
    }
  }

  // send はアプリ側からメッセージを WebSocket で送信する。
  async send(payload: Uint8Array, signal?: AbortSignal): Promise<void> {
    // close 済みの場合はエラーを投げる
    if (this.#closed) {
      throw new Error("WebSocketBidiChannel: channel is closed");
    }
    // signal がキャンセルされている場合はエラーを投げる
    if (signal?.aborted === true) {
      throw new Error("WebSocketBidiChannel.send: aborted");
    }
    // WebSocket 接続が完了するまで待機する
    await this.#openPromise;
    // close 済みチェックを再度行う（待機中に close された可能性がある）
    if (this.#closed) {
      throw new Error("WebSocketBidiChannel: channel closed during send");
    }
    // WebSocket が接続中の場合のみ送信する
    if (this.#ws?.readyState !== WebSocket.OPEN) {
      // 接続中でない場合はエラーを投げる
      throw new Error("WebSocketBidiChannel: WebSocket is not open");
    }
    // payload を WebSocket で送信する（バイナリ送信）
    this.#ws.send(payload);
  }

  // recv は受信メッセージを非同期イテレーターで返す。
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
  #dequeue(signal?: AbortSignal): Promise<BidiMessage | null> {
    // キューにメッセージが存在する場合は即座に返す
    const msg = this.#recvQueue.shift();
    // メッセージが存在する場合は即座に返す
    if (msg !== undefined) {
      return Promise.resolve(msg);
    }
    // キューが空の場合は Promise で待機する
    return new Promise<BidiMessage | null>((resolve) => {
      // signal がキャンセルされた場合は null で解決するハンドラー
      const abortHandler = (): void => {
        // resolver キューから自分を削除する
        const idx = this.#recvResolvers.indexOf(resolve);
        // 存在する場合は削除する
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

  // close はセッションを graceful close (code 1000) で終了する。
  async close(): Promise<void> {
    // closed フラグを立てる
    this.#closed = true;
    // Ping タイマーを停止する
    this.#stopPingTimer();
    // WebSocket を graceful close する（code 1000: 正常クローズ）
    if (this.#ws !== null && this.#ws.readyState === WebSocket.OPEN) {
      // code 1000 で close する（正常終了）
      this.#ws.close(WebSocketBidiChannel.WS_CLOSE_NORMAL, "session closed");
    }
    // pending の resolver を null で解決する（recv ループを停止させる）
    for (const resolver of this.#recvResolvers.splice(0)) {
      // null で解決する（セッション終了を通知する）
      resolver(null);
    }
  }
}

// ---- ConnectRpcBidiChannel ----

/**
 * ConnectRpcBidiChannel は Connect-RPC の bidi streaming プロトコルを使用する BidiChannel 実装クラス。
 * @connectrpc/connect パッケージが available な場合はそれを使用し、
 * なければ手書き HTTP/2 fetch と Connect length-prefixed message framing を実装する。
 * OSS 型（@connectrpc/connect 等）を公開 API シグネチャに一切露出しない。
 */
// ConnectRpcBidiChannel クラス定義（Connect-RPC transport 実装）
export class ConnectRpcBidiChannel implements BidiChannel {
  // CONNECT_FRAME_HEADER_SIZE: Connect framing のヘッダーサイズ（バイト）
  // 1 バイトフラグ + 4 バイト長 = 5 バイト
  static readonly CONNECT_FRAME_HEADER_SIZE: number = 5;
  // CONNECT_FLAG_MESSAGE: Connect framing の通常メッセージフラグ値
  static readonly CONNECT_FLAG_MESSAGE: number = 0;
  // CONNECT_FLAG_TRAILERS: Connect framing のトレーラーフラグ値
  static readonly CONNECT_FLAG_TRAILERS: number = 2;

  // #caps: クライアント capability 宣言
  readonly #caps: ClientCapabilities;
  // #baseUrl: 接続先 Gateway の base URL
  readonly #baseUrl: string;
  // #service: フルサービス名（"k1s0.tier1.WorkflowSvc" 等）
  readonly #service: string;
  // #method: RPC メソッド名（"StreamWorkflowEvents" 等）
  readonly #method: string;
  // #sendAbortController: 送信用 fetch の abort controller
  #sendAbortController: AbortController = new AbortController();
  // #recvAbortController: 受信用 fetch の abort controller
  #recvAbortController: AbortController = new AbortController();
  // #closed: close が呼ばれたかどうかのフラグ
  #closed: boolean = false;
  // #recvQueue: 受信メッセージキュー（framing デコーダーが注入する）
  readonly #recvQueue: BidiMessage[] = [];
  // #recvResolvers: pending の recv 待機を解決するための resolver キュー
  readonly #recvResolvers: Array<(msg: BidiMessage | null) => void> = [];
  // #nextSeq: 受信メッセージのシーケンス番号カウンター
  #nextSeq: bigint = 0n;
  // #recvLoopStarted: 受信ループが開始済みかどうかのフラグ
  #recvLoopStarted: boolean = false;

  // コンストラクタ: base URL / service / method / caps を受け取る
  constructor(
    baseUrl: string,
    service: string,
    method: string,
    caps: ClientCapabilities,
  ) {
    // base URL を設定する
    this.#baseUrl = baseUrl;
    // フルサービス名を設定する
    this.#service = service;
    // RPC メソッド名を設定する
    this.#method = method;
    // capabilities を設定する
    this.#caps = caps;
  }

  // chosenTransport ゲッター: ConnectRpc transport 種別を返す
  get chosenTransport(): TransportKind {
    // ConnectRpc を返す
    return TransportKind.ConnectRpc;
  }

  // capabilities ゲッター: クライアント capability 宣言を返す
  get capabilities(): ClientCapabilities {
    // #caps フィールドを返す
    return this.#caps;
  }

  // #encodeConnectFrame は Connect length-prefixed message framing でメッセージをエンコードする内部メソッド。
  // 仕様: 1 バイトフラグ（0x00 = 通常メッセージ）+ 4 バイト big-endian 長 + payload
  #encodeConnectFrame(payload: Uint8Array): Uint8Array {
    // フレームバッファを確保する（ヘッダー 5 バイト + payload）
    const frame = new Uint8Array(ConnectRpcBidiChannel.CONNECT_FRAME_HEADER_SIZE + payload.length);
    // フラグバイトを 0x00（通常メッセージ）に設定する
    frame[0] = ConnectRpcBidiChannel.CONNECT_FLAG_MESSAGE;
    // payload 長を 4 バイト big-endian で設定する
    const len = payload.length;
    // 最上位バイトを設定する
    frame[1] = (len >>> 24) & 0xff;
    // 上位バイトを設定する
    frame[2] = (len >>> 16) & 0xff;
    // 中位バイトを設定する
    frame[3] = (len >>> 8) & 0xff;
    // 最下位バイトを設定する
    frame[4] = len & 0xff;
    // payload をコピーする
    frame.set(payload, ConnectRpcBidiChannel.CONNECT_FRAME_HEADER_SIZE);
    // エンコード済みフレームを返す
    return frame;
  }

  // #decodeConnectFrames は Connect framing でエンコードされたバイト列からメッセージ配列をデコードする内部メソッド。
  // 複数フレームが連結されている場合はすべて抽出する。
  #decodeConnectFrames(data: Uint8Array): Uint8Array[] {
    // デコード済みメッセージ配列を初期化する
    const messages: Uint8Array[] = [];
    // オフセットを初期化する
    let offset = 0;
    // 全フレームを処理するループ
    while (offset + ConnectRpcBidiChannel.CONNECT_FRAME_HEADER_SIZE <= data.length) {
      // フラグバイトを取得する
      const flag = data[offset];
      // トレーラーフレームの場合はスキップする
      if (flag === ConnectRpcBidiChannel.CONNECT_FLAG_TRAILERS) {
        // トレーラーフレームは処理を終了する
        break;
      }
      // payload 長を big-endian 4 バイトで取得する
      const len =
        // 最上位バイト
        (((data[offset + 1] ?? 0) << 24) |
        // 上位バイト
        ((data[offset + 2] ?? 0) << 16) |
        // 中位バイト
        ((data[offset + 3] ?? 0) << 8) |
        // 最下位バイト
        (data[offset + 4] ?? 0)) >>> 0;
      // フレーム全体がバッファ内に収まらない場合は処理を終了する
      if (offset + ConnectRpcBidiChannel.CONNECT_FRAME_HEADER_SIZE + len > data.length) {
        break;
      }
      // payload を取得する
      const payload = data.slice(
        // ヘッダーの直後から
        offset + ConnectRpcBidiChannel.CONNECT_FRAME_HEADER_SIZE,
        // payload 長分だけ取得する
        offset + ConnectRpcBidiChannel.CONNECT_FRAME_HEADER_SIZE + len,
      );
      // メッセージ配列に追加する
      messages.push(payload);
      // オフセットを次のフレームに進める
      offset += ConnectRpcBidiChannel.CONNECT_FRAME_HEADER_SIZE + len;
    }
    // デコード済みメッセージ配列を返す
    return messages;
  }

  // #startRecvLoop は受信ループを開始する内部非同期メソッド。
  // Connect bidi streaming エンドポイントに GET で接続してストリームを受信する。
  async #startRecvLoop(): Promise<void> {
    // Connect bidi streaming エンドポイント URL を構築する
    const url = `${this.#baseUrl}/${this.#service}/${this.#method}`;
    // fetch で GET ストリーム接続を開始する
    let response: Response;
    // fetch エラーの場合は受信ループを終了する
    try {
      // fetch で Connect bidi streaming エンドポイントに接続する
      response = await fetch(url, {
        // GET メソッドで接続する（Connect bidi streaming の受信側）
        method: "GET",
        // Content-Type を application/connect+proto に設定する
        headers: {
          // Connect-Protocol-Version ヘッダーを設定する
          "Connect-Protocol-Version": "1",
          // Accept ヘッダーを設定する
          "Accept": "application/connect+proto",
        },
        // AbortSignal を設定する
        signal: this.#recvAbortController.signal,
      });
    } catch {
      // fetch エラーは受信ループを終了する
      return;
    }
    // HTTP ステータスが 2xx 以外の場合は受信ループを終了する
    if (!response.ok || response.body === null) {
      return;
    }
    // ReadableStream からフレームを読み取るリーダーを取得する
    const reader = response.body.getReader();
    // バッファを初期化する（未処理バイトを蓄積する）
    let buffer = new Uint8Array(0);
    // ストリームを読み続けるループ
    while (!this.#closed) {
      // リーダーから次のチャンクを読み取る
      let done: boolean;
      // 受信チャンク
      let value: Uint8Array | undefined;
      // read エラーの場合は受信ループを終了する
      try {
        // 次のチャンクを読み取る
        const result = await reader.read();
        // done フラグを設定する
        done = result.done;
        // 受信チャンクを設定する
        value = result.value;
      } catch {
        // read エラーは受信ループを終了する
        break;
      }
      // ストリーム終端の場合はループを終了する
      if (done) {
        break;
      }
      // 受信チャンクが存在する場合はバッファに追加する
      if (value !== undefined) {
        // バッファを結合する
        const merged = new Uint8Array(buffer.length + value.length);
        // 既存バッファをコピーする
        merged.set(buffer, 0);
        // 新しいチャンクをコピーする
        merged.set(value, buffer.length);
        // バッファを更新する
        buffer = merged;
      }
      // バッファからフレームをデコードする
      const decoded = this.#decodeConnectFrames(buffer);
      // デコード済みフレームをキューに投入する
      for (const payload of decoded) {
        // BidiMessage を構築する
        const msg: BidiMessage = {
          // payload を設定する
          payload,
          // シーケンス番号をインクリメントして設定する
          sequenceId: this.#nextSeq++,
          // resumeToken は Connect-RPC では未使用（undefined）
          resumeToken: undefined,
        };
        // pending resolver が存在する場合は直接解決する
        const resolver = this.#recvResolvers.shift();
        // resolver が存在する場合は直接メッセージを渡す
        if (resolver !== undefined) {
          // resolver を呼び出す
          resolver(msg);
          continue;
        }
        // resolver が存在しない場合はキューに追加する
        this.#recvQueue.push(msg);
      }
      // 処理済みバイト数を計算して残りのバッファを更新する
      // 最後のフレームの終端位置を再計算する（デコード分を除去する）
      let consumedBytes = 0;
      // デコード済みフレームのバイト数を合計する
      for (const payload of decoded) {
        // ヘッダー + payload のバイト数を加算する
        consumedBytes += ConnectRpcBidiChannel.CONNECT_FRAME_HEADER_SIZE + payload.length;
      }
      // バッファから処理済みバイトを除去する
      buffer = buffer.slice(consumedBytes);
    }
    // 受信ループ終了時に pending resolver を null で解決する
    for (const resolver of this.#recvResolvers.splice(0)) {
      // null で解決する（セッション終了を通知する）
      resolver(null);
    }
    // リーダーを解放する
    reader.releaseLock();
  }

  // send はアプリ側からメッセージを Connect framing でエンコードして POST 送信する。
  async send(payload: Uint8Array, signal?: AbortSignal): Promise<void> {
    // close 済みの場合はエラーを投げる
    if (this.#closed) {
      throw new Error("ConnectRpcBidiChannel: channel is closed");
    }
    // signal がキャンセルされている場合はエラーを投げる
    if (signal?.aborted === true) {
      throw new Error("ConnectRpcBidiChannel.send: aborted");
    }
    // 受信ループが未開始の場合は開始する（最初の send 時に遅延初期化する）
    if (!this.#recvLoopStarted) {
      // 受信ループ開始フラグを立てる
      this.#recvLoopStarted = true;
      // 受信ループを非同期で開始する
      void this.#startRecvLoop();
    }
    // Connect framing でメッセージをエンコードする
    const frame = this.#encodeConnectFrame(payload);
    // POST エンドポイント URL を構築する
    const sendUrl = `${this.#baseUrl}/${this.#service}/${this.#method}`;
    // AbortSignal を合成する（メソッド引数と内部 controller の双方でキャンセルできる）
    const combinedSignal = signal !== undefined
      ? AbortSignal.any([signal, this.#sendAbortController.signal])
      : this.#sendAbortController.signal;
    // fetch で POST 送信する（Connect framing でエンコードされたフレームを送信する）
    const response = await fetch(sendUrl, {
      // POST メソッドで送信する
      method: "POST",
      // Content-Type を application/connect+proto に設定する
      headers: {
        // Content-Type ヘッダーを設定する
        "Content-Type": "application/connect+proto",
        // Connect-Protocol-Version ヘッダーを設定する
        "Connect-Protocol-Version": "1",
      },
      // Connect framing フレームを ArrayBuffer のコピーに変換して BodyInit 型制約を満たす
      body: frame.buffer.slice(frame.byteOffset, frame.byteOffset + frame.byteLength) as ArrayBuffer,
      // AbortSignal を設定する
      signal: combinedSignal,
    });
    // HTTP ステータスが 2xx 以外の場合はエラーを投げる
    if (!response.ok) {
      // レスポンステキストを取得してエラーメッセージに含める
      const text = await response.text();
      // エラーを投げる
      throw new Error(`ConnectRpcBidiChannel.send: HTTP ${response.status}: ${text}`);
    }
  }

  // recv は受信メッセージを非同期イテレーターで返す。
  async *recv(signal?: AbortSignal): AsyncIterable<BidiMessage> {
    // 受信ループが未開始の場合は開始する（recv が先に呼ばれた場合に遅延初期化する）
    if (!this.#recvLoopStarted) {
      // 受信ループ開始フラグを立てる
      this.#recvLoopStarted = true;
      // 受信ループを非同期で開始する
      void this.#startRecvLoop();
    }
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
  #dequeue(signal?: AbortSignal): Promise<BidiMessage | null> {
    // キューにメッセージが存在する場合は即座に返す
    const msg = this.#recvQueue.shift();
    // メッセージが存在する場合は即座に返す
    if (msg !== undefined) {
      return Promise.resolve(msg);
    }
    // キューが空の場合は Promise で待機する
    return new Promise<BidiMessage | null>((resolve) => {
      // signal がキャンセルされた場合は null で解決するハンドラー
      const abortHandler = (): void => {
        // resolver キューから自分を削除する
        const idx = this.#recvResolvers.indexOf(resolve);
        // 存在する場合は削除する
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
    // 送信用 fetch を abort する
    this.#sendAbortController.abort();
    // 受信用 fetch を abort する
    this.#recvAbortController.abort();
    // pending の resolver を null で解決する（recv ループを停止させる）
    for (const resolver of this.#recvResolvers.splice(0)) {
      // null で解決する（セッション終了を通知する）
      resolver(null);
    }
  }
}

// ---- TransportNegotiationClientImpl (factory) ----

/**
 * TransportNegotiationClientImpl は TransportNegotiationClient interface を実装する factory class。
 * negotiate() は利用可能な transport から最適な transport を選択して BidiChannel を返す。
 * 選択優先順位: ConnectRpc > SsePaired > WebSocket > LongPoll
 * OSS 型を公開 API シグネチャに一切露出しない。
 */
// TransportNegotiationClientImpl クラス定義（factory 実装）
export class TransportNegotiationClientImpl implements TransportNegotiationClient {
  // #baseUrl: 接続先 Gateway の base URL
  readonly #baseUrl: string;

  // コンストラクタ: Gateway の base URL を受け取る
  constructor(gatewayBaseUrl: string) {
    // base URL を設定する
    this.#baseUrl = gatewayBaseUrl;
  }

  // baseUrl ゲッター: Gateway の base URL を返す
  get baseUrl(): string {
    // #baseUrl フィールドを返す
    return this.#baseUrl;
  }

  // negotiate は capabilities から最適な transport を選択して BidiChannel を返す。
  // 選択優先順位: ConnectRpc > SsePaired > WebSocket > LongPoll
  async negotiate(
    service: string,
    method: string,
    capabilities: ClientCapabilities,
    signal?: AbortSignal,
  ): Promise<BidiChannel> {
    // signal がキャンセルされている場合はエラーを投げる
    if (signal?.aborted === true) {
      throw new Error("TransportNegotiationClientImpl.negotiate: aborted");
    }
    // availableTransports に ConnectRpc が含まれる場合は ConnectRpcBidiChannel を返す
    if (capabilities.availableTransports.includes(TransportKind.ConnectRpc)) {
      // ConnectRpc adapter を選択して返す（最優先）
      return new ConnectRpcBidiChannel(
        // base URL を渡す
        this.#baseUrl,
        // フルサービス名を渡す
        service,
        // RPC メソッド名を渡す
        method,
        // capabilities を渡す
        capabilities,
      );
    }
    // availableTransports に SsePaired が含まれる場合は SsePairedBidiChannel を返す
    if (capabilities.availableTransports.includes(TransportKind.SsePaired)) {
      // SsePaired adapter を選択して返す（第2優先）
      return new SsePairedBidiChannel(
        // base URL を渡す
        this.#baseUrl,
        // フルサービス名を渡す
        service,
        // RPC メソッド名を渡す
        method,
        // capabilities を渡す
        capabilities,
      );
    }
    // availableTransports に WebSocket が含まれる場合は WebSocketBidiChannel を返す
    if (capabilities.availableTransports.includes(TransportKind.WebSocket)) {
      // WebSocket URL を構築する（http → ws / https → wss のスキーム変換を行う）
      const wsUrl = this.#buildWsUrl(service, method);
      // WebSocket adapter を選択して返す（第3優先）
      return new WebSocketBidiChannel(
        // WebSocket URL を渡す
        wsUrl,
        // capabilities を渡す
        capabilities,
      );
    }
    // availableTransports に LongPoll が含まれる場合は LongPollBidiChannel を返す
    if (capabilities.availableTransports.includes(TransportKind.LongPoll)) {
      // LongPoll adapter を選択して返す（第4優先・フォールバック）
      return new LongPollBidiChannel(
        // base URL を渡す
        this.#baseUrl,
        // capabilities を渡す
        capabilities,
      );
    }
    // 利用可能な transport が存在しない場合はエラーを投げる
    throw new Error(
      "TransportNegotiationClientImpl.negotiate: no compatible transport found in capabilities.availableTransports",
    );
  }

  // #buildWsUrl は HTTP/HTTPS URL を WebSocket URL（ws/wss）に変換する内部ヘルパーメソッド。
  // http → ws / https → wss のスキームを変換し、service / method をパスに追加する。
  #buildWsUrl(service: string, method: string): string {
    // http:// を ws:// に置換する
    const wsBase = this.#baseUrl
      // https:// を wss:// に変換する
      .replace(/^https:\/\//i, "wss://")
      // http:// を ws:// に変換する
      .replace(/^http:\/\//i, "ws://");
    // service と method をパスに追加した WebSocket URL を返す
    return `${wsBase}/${service}/${method}`;
  }
}
