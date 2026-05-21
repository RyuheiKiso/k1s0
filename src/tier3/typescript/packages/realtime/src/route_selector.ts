// k1s0 tier3 UA aware Transport ルートセレクター
// spec 12_UA_aware_adapter §routing: ua_subclass に基づき connect_bidi_native / paired_post_sse を決定論的に選択する
// chrome_edge_direct / tauri_native → connect_bidi_native（SLO p99 30ms 系統）
// chrome_edge_via_corp_proxy / firefox_safari / dotnet_companion → paired_post_sse（SLO p99 80ms 系統）

import type { Transport, TransportOptions } from "./transport.js";
import type { UaAdapterRoute } from "./subscription.js";
import { createWebSocketAdapter } from "./websocket_adapter.js";
import { createSseAdapter } from "./sse_adapter.js";
import { createLongPollAdapter } from "./long_poll_adapter.js";

// サポートするアダプター名（ua_subclass routing + 既存 fallback に対応）
// connect_bidi_native: chrome_edge_direct / tauri_native 向け Connect-RPC bidi（SLO p99 30ms）
// paired_post_sse: corp-proxy / firefox_safari / dotnet_companion 向け POST+SSE pair（SLO p99 80ms）
export type AdapterName = "websocket" | "sse" | "long-poll" | "connect_bidi_native" | "paired_post_sse";

// UA 文字列から WebSocket 接続をサポートするかを判定するヘルパー
function uaSupportsWebSocket(): boolean {
  // サーバーサイドレンダリング環境（window なし）では false を返す
  if (typeof window === "undefined") {
    return false;
  }
  // WebSocket が window に定義されているかで判定する
  return typeof WebSocket !== "undefined";
}

// UA 文字列から SSE をサポートするかを判定するヘルパー
function uaSupportsSSE(): boolean {
  // サーバーサイドレンダリング環境（window なし）では false を返す
  if (typeof window === "undefined") {
    return false;
  }
  // EventSource が window に定義されているかで判定する
  return typeof EventSource !== "undefined";
}

// UA 文字列から Tauri 環境かどうかを判定するヘルパー
function isTauriEnvironment(): boolean {
  // Tauri は window.__TAURI__ を注入する
  return typeof window !== "undefined" && "__TAURI__" in window;
}

// UA 文字列から .NET Companion 環境かどうかを判定するヘルパー
function isDotNetCompanion(): boolean {
  // .NET WebView2 は ua に "WebView2" を含む
  if (typeof navigator === "undefined") {
    return false;
  }
  // UserAgent 文字列に WebView2 が含まれるかを確認する
  return navigator.userAgent.includes("WebView2");
}

// UA に基づいてデフォルトの adapter 優先順位を決定するヘルパー
function detectDefaultAdapterOrder(): AdapterName[] {
  // Tauri 環境は WebSocket が最優先
  if (isTauriEnvironment()) {
    return ["websocket", "sse", "long-poll"];
  }
  // .NET Companion（WebView2）は WebSocket をサポートするため最優先
  if (isDotNetCompanion()) {
    return ["websocket", "sse", "long-poll"];
  }
  // WebSocket サポート確認
  if (uaSupportsWebSocket()) {
    // WebSocket → SSE → long-poll の順で試みる
    return ["websocket", "sse", "long-poll"];
  }
  // SSE のみサポート
  if (uaSupportsSSE()) {
    return ["sse", "long-poll"];
  }
  // フォールバック: long-poll のみ
  return ["long-poll"];
}

// ua_subclass から AdapterName を決定論的に選択するヘルパー
// spec 12 §SLO 別二系統管理: chrome_edge_direct / tauri_native → 30ms 系統, それ以外 → 80ms 系統
function adapterFromUaSubclass(ua_subclass: UaAdapterRoute): AdapterName {
  // 30ms SLO 系統: Connect-RPC native bidi（WebSocket H/2 multiplexed）
  if (ua_subclass === "chrome_edge_direct" || ua_subclass === "tauri_native") {
    return "connect_bidi_native";
  }
  // 80ms SLO 系統: POST+SSE pair（corp proxy / Firefox/Safari / .NET Companion）
  return "paired_post_sse";
}

// アダプター名から Transport ファクトリ関数を取得するヘルパー
function resolveFactory(
  name: AdapterName,
): (url: string, options?: TransportOptions) => Transport {
  // アダプター名に対応するファクトリを返す
  switch (name) {
    case "websocket":
      // WebSocket アダプターファクトリを返す
      return createWebSocketAdapter;
    case "sse":
      // SSE アダプターファクトリを返す
      return createSseAdapter;
    case "long-poll":
      // long-poll アダプターファクトリを返す（SSE にフォールバック）
      return createLongPollAdapter;
    case "connect_bidi_native":
      // Connect-RPC bidi native（WebSocket H/2）— chrome_edge_direct / tauri_native 向け
      // TODO: Connect-RPC native adapter は _crosscutting/13_dotnet8_connect_inhouse の TS 移植で置換する
      return createWebSocketAdapter;
    case "paired_post_sse":
      // POST+SSE pair（corp proxy / Firefox/Safari / dotnet_companion 向け）
      // TODO: POST half を追加した完全 paired adapter に置換する
      return createSseAdapter;
    default: {
      // 網羅性チェック（新アダプター追加時はコンパイルエラー）
      const _exhaustive: never = name;
      throw new Error(`Unknown adapter: ${String(_exhaustive)}`);
    }
  }
}

// ルートセレクターの選択結果
export interface SelectTransportResult {
  // 選択された Transport インスタンス
  readonly transport: Transport;
  // 選択されたアダプター名
  readonly selectedAdapter: AdapterName;
}

// ua_subclass または preferredAdapters リストに基づいて Transport を選択して返す
// ua_subclass を指定した場合: spec 12 §routing の決定論的 2 系統ルーティングを適用する
// ua_subclass 未指定の場合: 旧来の UA 自動検出優先順位に従う（後方互換）
export function selectTransport(
  url: string,
  ua_subclass?: UaAdapterRoute,
  options?: TransportOptions,
): SelectTransportResult {
  // ua_subclass が指定された場合は spec 12 §routing の決定論的 2 系統選択を行う
  const selected: AdapterName = ua_subclass != null
    ? adapterFromUaSubclass(ua_subclass)
    : (detectDefaultAdapterOrder()[0] ?? "long-poll");
  // 選択されたアダプターのファクトリを取得する
  const factory = resolveFactory(selected);
  // Transport インスタンスを生成する
  const transport = factory(url, options);
  // 選択結果を返す
  return { transport, selectedAdapter: selected };
}
