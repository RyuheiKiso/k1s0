/**
 * rpc.ts — k1s0 tier1 Library TypeScript 実装: RPC / Gateway の L3 interface
 * 10_RPC適合仕様.md §RpcClient / §GatewayHandler（OSS 中立 L3）に準拠する。
 * gRPC / ConnectRPC / HTTP/2 等 OSS の API を一切露出しない Wire protocol 抽象 interface を宣言する。
 * 公開シグネチャに OSS 型（@grpc/grpc-js 等）を一切含まない。
 */

// AuthContext は Gateway リクエストの tenant 分離に必要
import type { AuthContext } from "./authContext.js";

/**
 * RpcMetadata は RPC 呼び出しのメタデータ（ヘッダー相当）を宣言する型。
 * OSS の Metadata / Headers を露出せず Library 独自語彙で表現する。
 */
// RpcMetadata 型定義
export type RpcMetadata = ReadonlyMap<string, readonly string[]>;

/**
 * RpcStatusCode は RPC 呼び出しのステータスコードを宣言する enum。
 * gRPC Status Code に準拠した Library 独自語彙とする。
 */
// RpcStatusCode 列挙型定義
export const enum RpcStatusCode {
  // Ok: 成功（gRPC OK = 0）
  Ok = 0,
  // Canceled: キャンセル（gRPC Canceled = 1）
  Canceled = 1,
  // Unknown: 不明なエラー（gRPC Unknown = 2）
  Unknown = 2,
  // InvalidArgument: 無効な引数（gRPC InvalidArgument = 3）
  InvalidArgument = 3,
  // DeadlineExceeded: デッドライン超過（gRPC DeadlineExceeded = 4）
  DeadlineExceeded = 4,
  // NotFound: リソース未発見（gRPC NotFound = 5）
  NotFound = 5,
  // AlreadyExists: リソース重複（gRPC AlreadyExists = 6）
  AlreadyExists = 6,
  // PermissionDenied: 権限エラー（gRPC PermissionDenied = 7）
  PermissionDenied = 7,
  // Unauthenticated: 認証エラー（gRPC Unauthenticated = 16）
  Unauthenticated = 16,
  // ResourceExhausted: リソース枯渇（gRPC ResourceExhausted = 8）
  ResourceExhausted = 8,
  // FailedPrecondition: 前提条件不満（gRPC FailedPrecondition = 9）
  FailedPrecondition = 9,
  // Aborted: 中断（gRPC Aborted = 10）
  Aborted = 10,
  // Internal: 内部エラー（gRPC Internal = 13）
  Internal = 13,
  // Unavailable: サービス不可（gRPC Unavailable = 14）
  Unavailable = 14,
}

/**
 * RpcError は RPC 呼び出しエラーを宣言するクラス。
 * OSS の status.Error / connect.ConnectError を露出せず Library 独自語彙で表現する。
 */
// RpcError クラス定義
export class RpcError extends Error {
  // code: gRPC Status Code 準拠のステータスコード
  readonly code: RpcStatusCode;
  // detail: エラー詳細情報（オプション）
  readonly detail?: unknown;

  // コンストラクタ: code と message を受け取る
  constructor(code: RpcStatusCode, message: string, detail?: unknown) {
    // Error クラスに message を渡す
    super(message);
    // name を設定する（Error サブクラスの標準的な作法）
    this.name = "RpcError";
    // code を設定する
    this.code = code;
    // detail を設定する
    this.detail = detail;
  }
}

/**
 * RpcCallOptions は RPC 呼び出しに渡すオプションを宣言する型。
 */
// RpcCallOptions 型定義
export interface RpcCallOptions {
  // metadata: 送信するリクエストメタデータ（ヘッダー相当）
  readonly metadata?: RpcMetadata | undefined;
  // authContext: RPC 呼び出しに付与する認証コンテキスト（tenant 分離に必須）
  readonly authContext?: AuthContext | undefined;
  // signal: AbortSignal（タイムアウト / キャンセル制御）
  readonly signal?: AbortSignal | undefined;
}

/**
 * RpcUnaryClient は Unary RPC の L3 抽象 interface を宣言する。
 * 全 OSS の transport を透過的に切り替え可能にする。
 * OSS 型（@grpc/grpc-js / @connectrpc/connect 等）を一切含まない。
 */
// RpcUnaryClient インターフェース定義
export interface RpcUnaryClient {
  /**
   * call は Unary RPC を呼び出す。
   * service はフルサービス名（"k1s0.tier1.KeySvc" 等）。
   * method は RPC メソッド名（"Sign" 等）。
   * req はリクエストペイロード（protobuf の JSON バイト列）。
   * 戻り値はレスポンスペイロード（protobuf の JSON バイト列）とレスポンスメタデータ。
   */
  // call メソッド: Unary RPC を呼び出す
  call(
    service: string,
    method: string,
    req: Uint8Array,
    opts?: RpcCallOptions,
  ): Promise<[Uint8Array, RpcMetadata]>;
}

/**
 * RpcStreamClient は Streaming RPC の L3 抽象 interface を宣言する。
 */
// RpcStreamClient インターフェース定義
export interface RpcStreamClient {
  /**
   * send はストリームにメッセージを送信する（Client streaming / Bidirectional 用）。
   */
  // send メソッド: ストリームにメッセージを送信する
  send(payload: Uint8Array): Promise<void>;

  /**
   * recv はストリームからメッセージを受信する非同期イテレーターを返す。
   * ストリーム終了時にイテレーションが完了する。
   */
  // recv メソッド: ストリームからメッセージを受信する
  recv(): AsyncIterable<Uint8Array>;

  /**
   * closeSend はクライアント側の送信を完了する（Half-close）。
   */
  // closeSend メソッド: 送信を完了する
  closeSend(): Promise<void>;

  /**
   * metadata は受信したレスポンスメタデータ（トレーラー等）を返す。
   */
  // metadata メソッド: レスポンスメタデータを返す
  metadata(): RpcMetadata;
}

/**
 * RpcStreamingClient は Streaming RPC セッションを開始する L3 interface を宣言する。
 */
// RpcStreamingClient インターフェース定義
export interface RpcStreamingClient {
  /**
   * openStream は Streaming RPC セッションを開始して RpcStreamClient を返す。
   * service / method はフルサービス名・RPC メソッド名。
   */
  // openStream メソッド: Streaming RPC セッションを開始する
  openStream(
    service: string,
    method: string,
    opts?: RpcCallOptions,
  ): Promise<RpcStreamClient>;
}

/**
 * GatewayRequest は Gateway 経由の HTTP/gRPC リクエストを宣言する型。
 * OSS の Request を露出せず Library 独自語彙で表現する。
 */
// GatewayRequest 型定義
export interface GatewayRequest {
  // method: HTTP メソッド（"GET" / "POST" 等）
  readonly method: string;
  // path: リクエストパス（"/v1/keys/sign" 等）
  readonly path: string;
  // headers: リクエストヘッダー
  readonly headers: RpcMetadata;
  // body: リクエストボディバイト列
  readonly body: Uint8Array;
  // authContext: Gateway が検証済み認証コンテキスト（tenant 分離必須）
  readonly authContext: AuthContext;
}

/**
 * GatewayResponse は Gateway から返す HTTP レスポンスを宣言する型。
 */
// GatewayResponse 型定義
export interface GatewayResponse {
  // statusCode: HTTP ステータスコード
  readonly statusCode: number;
  // headers: レスポンスヘッダー
  readonly headers: RpcMetadata;
  // body: レスポンスボディバイト列
  readonly body: Uint8Array;
}

/**
 * GatewayHandler は Gateway のリクエストハンドラー interface を宣言する。
 */
// GatewayHandler インターフェース定義
export interface GatewayHandler {
  /**
   * handle は GatewayRequest を受け取って GatewayResponse を返す。
   * signal で中断制御を行う（AbortSignal）。
   */
  // handle メソッド: リクエストを処理する
  handle(req: GatewayRequest, signal?: AbortSignal): Promise<GatewayResponse>;
}

/**
 * GatewayMiddleware は Gateway ミドルウェアを宣言する型。
 * GatewayHandler をラップして前処理・後処理を挿入する。
 */
// GatewayMiddleware 型定義
export type GatewayMiddleware = (next: GatewayHandler) => GatewayHandler;
