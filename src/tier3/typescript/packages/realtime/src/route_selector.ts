// k1s0 tier3 UA aware Transport ルートセレクター
// UA 検出（navigator.userAgent）により WebSocket / SSE / long-poll を選択する
// subscription.ts の UaAdapterRoute と対応し、物理的な Transport を返す

import type { Transport, TransportOptions } from "./transport.js";
import { createWebSocketAdapter } from "./websocket_adapter.js";
import { createSseAdapter } from "./sse_adapter.js";
// long_poll_adapter.ts の完全実装をインポートする（C-7: TODO stub を実装に置き換える）
import { createLongPollAdapter } from "./long_poll_adapter.js";

// サポートするアダプター名（preferredAdapters の値として使用）
export type AdapterName = "websocket" | "sse" | "long-poll";

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

// UA 検出と preferredAdapters リストに基づいて Transport を選択して返す
// preferredAdapters を指定しない場合は UA 自動検出の優先順位に従う
export function selectTransport(
  url: string,
  preferredAdapters?: AdapterName[],
  options?: TransportOptions,
): SelectTransportResult {
  // 優先順位リストを決定する（引数がある場合はそれを使う）
  const adapterOrder = preferredAdapters ?? detectDefaultAdapterOrder();
  // 優先順位の最初のアダプターを選択する（空の場合は long-poll をデフォルト）
  const selected: AdapterName = adapterOrder[0] ?? "long-poll";
  // 選択されたアダプターのファクトリを取得する
  const factory = resolveFactory(selected);
  // Transport インスタンスを生成する
  const transport = factory(url, options);
  // 選択結果を返す
  return { transport, selectedAdapter: selected };
}
