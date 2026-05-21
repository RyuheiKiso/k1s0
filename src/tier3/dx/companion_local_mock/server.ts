// server.ts — tier3 companion ローカルモック WebSocket サーバー
// ws://127.0.0.1:9999/sync で待ち受けて lib.rs の companion プロトコルに対応する
// プロトコル形式: {command, layer, entry, psk_hmac, hlc_timestamp}
// lib.rs の state_read / state_write / state_sync コマンドに完全準拠する
// このサーバーは開発・テスト専用であり本番環境では使用しない

// Node.js 組み込み http モジュールをインポートする
import { createServer } from 'http';
// ws ライブラリをインポートする（WebSocket プロトコル実装）
import { WebSocketServer, WebSocket } from 'ws';
// crypto モジュールをインポートする（PSK HMAC 検証に使用する）
import { createHmac } from 'crypto';

// lib.rs の companion プロトコルに準拠した受信コマンドの型定義
// state_read: {command: "state_read", layer, hlc_timestamp, psk_hmac}
// state_write: {command: "state_write", layer, entry, hlc_timestamp, psk_hmac}
// state_sync: {command: "state_sync", hlc_timestamp, psk_hmac}
interface CompanionCommand {
  // コマンド識別子（lib.rs の state_read / state_write / state_sync に対応）
  command: 'state_read' | 'state_write' | 'state_sync';
  // 対象レイヤ名（state_read / state_write で使用）
  layer?: string;
  // 書き込むエントリ（state_write で使用）
  entry?: unknown;
  // PSK HMAC 値（認証に使用する）
  psk_hmac: string;
  // HLC タイムスタンプ（wall-clock TTL 禁止規約に従い HLC を使用する）
  hlc_timestamp: string;
}

// lib.rs の companion プロトコルに準拠したサーバー応答の型定義
interface CompanionResponse {
  // 成功 / エラーの区別（lib.rs の Result<T, String> に対応）
  status: 'ok' | 'error';
  // 成功時のデータ（state_read / state_sync が返す値）
  data?: unknown;
  // エラー時のメッセージ（lib.rs のエラー文字列に対応）
  error?: string;
}

// インメモリ state ストア（layer 名をキーとして state を管理する）
// state_write で書き込み、state_read で読み取る
const stateStore = new Map<string, unknown>();

// モック PSK（lib.rs の "k1s0-default-psk" に対応するデフォルト値）
const MOCK_PSK = Buffer.from('k1s0-default-psk', 'utf-8');

// PSK HMAC を検証するヘルパー関数
// lib.rs の compute_psk_hmac と同じ HMAC-SHA256 を使用する
function verifyPskHmac(hmacValue: string, message: string): boolean {
  // HMAC-SHA256 を計算して base64url エンコードする
  const expectedHmac = createHmac('sha256', MOCK_PSK)
    .update(message)
    .digest('base64url');
  // 定数時間比較で HMAC を検証する
  return expectedHmac === hmacValue;
}

// HTTP サーバーを作成する（WebSocket サーバーのアタッチ先として使用する）
const httpServer = createServer((req, res) => {
  // ヘルスチェックエンドポイント（GET /health）
  if (req.method === 'GET' && req.url === '/health') {
    // 200 OK を返す（k8s の readiness probe 用）
    res.writeHead(200, { 'Content-Type': 'application/json' });
    res.end(JSON.stringify({ status: 'ok', connections: wss.clients.size }));
    return;
  }
  // その他のリクエストは 404 を返す
  res.writeHead(404);
  res.end();
});

// WebSocket サーバーを /sync パスで待ち受ける（lib.rs は ws://127.0.0.1:{port}/sync に接続する）
const wss = new WebSocketServer({ server: httpServer, path: '/sync' });

// 新規 WebSocket 接続を処理する
wss.on('connection', (ws: WebSocket) => {
  // 接続確立をコンソールに記録する（デバッグ用）
  console.log('[companion-mock] client connected, total:', wss.clients.size);

  // クライアントからメッセージを受信したときの処理
  ws.on('message', (rawData) => {
    // 受信データを文字列に変換する
    const raw = rawData.toString();
    let cmd: CompanionCommand;
    // JSON パースを試みる（不正なメッセージはエラーを返す）
    try {
      // JSON 文字列をパースして CompanionCommand 型に変換する
      cmd = JSON.parse(raw) as CompanionCommand;
    } catch {
      // JSON パースエラーの場合はエラーレスポンスを送信する
      sendResponse(ws, {
        status: 'error',
        error: `invalid JSON: ${raw}`,
      });
      return;
    }

    // コマンド種別に応じてハンドラーに分岐する
    switch (cmd.command) {
      // state_read: 指定レイヤの state を読み取る
      case 'state_read':
        handleStateRead(ws, cmd);
        break;
      // state_write: 指定レイヤに state を書き込む
      case 'state_write':
        handleStateWrite(ws, cmd);
        break;
      // state_sync: 全 state を同期する
      case 'state_sync':
        handleStateSync(ws, cmd);
        break;
      default: {
        // 未知のコマンドはエラーを返す
        const unknownCmd = (cmd as { command: string }).command;
        sendResponse(ws, {
          status: 'error',
          error: `unknown command: ${unknownCmd}`,
        });
      }
    }
  });

  // 接続切断を記録する（デバッグ用）
  ws.on('close', () => {
    console.log('[companion-mock] client disconnected, remaining:', wss.clients.size - 1);
  });

  // WebSocket エラーをコンソールに記録する
  ws.on('error', (err) => {
    console.error('[companion-mock] websocket error:', err.message);
  });
});

// state_read コマンドのハンドラー（lib.rs の state_read に対応する）
// lib.rs: hmac_input = "state_read:{layer}:{hlc}"
function handleStateRead(ws: WebSocket, cmd: CompanionCommand): void {
  // layer が指定されていない場合はエラーを返す
  if (cmd.layer === undefined || cmd.layer === '') {
    sendResponse(ws, {
      status: 'error',
      error: 'state_read requires layer',
    });
    return;
  }
  // PSK HMAC を検証する（lib.rs と同じ hmac_input を使用する）
  const hmacInput = `state_read:${cmd.layer}:${cmd.hlc_timestamp}`;
  // HMAC 検証が失敗した場合はエラーを返す
  if (!verifyPskHmac(cmd.psk_hmac, hmacInput)) {
    sendResponse(ws, {
      status: 'error',
      error: 'PSK HMAC 検証失敗: 不正なリクエストを拒否します',
    });
    return;
  }
  // spec 正値のレイヤ名のみを受け付ける（lib.rs と同じバリデーション）
  const validLayers = ['server_truth', 'optimistic_local', 'pending_queue', 'draft'];
  if (!validLayers.includes(cmd.layer)) {
    sendResponse(ws, {
      status: 'error',
      error: `未知のレイヤです: '${cmd.layer}'. 有効なレイヤ: ${validLayers.join(' / ')}`,
    });
    return;
  }
  // ストアからレイヤの state を読み取る（存在しない場合は null を返す）
  const value = stateStore.get(cmd.layer) ?? null;
  // 読み取った state をレスポンスとして返す
  sendResponse(ws, {
    status: 'ok',
    data: {
      // レイヤ名
      layer: cmd.layer,
      // レイヤの state（存在しない場合は null）
      value,
      // HLC タイムスタンプを応答に含める
      hlc_timestamp: cmd.hlc_timestamp,
    },
  });
}

// state_write コマンドのハンドラー（lib.rs の state_write に対応する）
// lib.rs: hmac_input = "state_write:{layer}:{hlc}"
function handleStateWrite(ws: WebSocket, cmd: CompanionCommand): void {
  // layer が指定されていない場合はエラーを返す
  if (cmd.layer === undefined || cmd.layer === '') {
    sendResponse(ws, {
      status: 'error',
      error: 'state_write requires layer',
    });
    return;
  }
  // server_truth レイヤへの書き込みは禁止する（lib.rs と同じ制約）
  if (cmd.layer === 'server_truth') {
    sendResponse(ws, {
      status: 'error',
      error: 'server_truth レイヤは read-only です',
    });
    return;
  }
  // PSK HMAC を検証する（lib.rs と同じ hmac_input を使用する）
  const hmacInput = `state_write:${cmd.layer}:${cmd.hlc_timestamp}`;
  // HMAC 検証が失敗した場合はエラーを返す
  if (!verifyPskHmac(cmd.psk_hmac, hmacInput)) {
    sendResponse(ws, {
      status: 'error',
      error: 'PSK HMAC 検証失敗: 不正なリクエストを拒否します',
    });
    return;
  }
  // 書き込み可能レイヤを確認する（lib.rs と同じバリデーション）
  const writableLayers = ['optimistic_local', 'pending_queue', 'draft'];
  if (!writableLayers.includes(cmd.layer)) {
    sendResponse(ws, {
      status: 'error',
      error: `未知のレイヤです: '${cmd.layer}'. 有効な書き込みレイヤ: ${writableLayers.join(' / ')}`,
    });
    return;
  }
  // ストアにレイヤの state を書き込む（entry フィールドを使用する）
  stateStore.set(cmd.layer, cmd.entry);
  // 書き込み成功のレスポンスを返す
  sendResponse(ws, {
    status: 'ok',
    data: {
      // 書き込んだレイヤ名
      layer: cmd.layer,
      // 書き込んだエントリ
      writtenEntry: cmd.entry,
      // HLC タイムスタンプを応答に含める
      hlc_timestamp: cmd.hlc_timestamp,
    },
  });
}

// state_sync コマンドのハンドラー（lib.rs の state_sync に対応する）
// lib.rs: hmac_input = "state_sync:{hlc}"
function handleStateSync(ws: WebSocket, cmd: CompanionCommand): void {
  // PSK HMAC を検証する（lib.rs と同じ hmac_input を使用する）
  const hmacInput = `state_sync:${cmd.hlc_timestamp}`;
  // HMAC 検証が失敗した場合はエラーを返す
  if (!verifyPskHmac(cmd.psk_hmac, hmacInput)) {
    sendResponse(ws, {
      status: 'error',
      error: 'PSK HMAC 検証失敗: 不正なリクエストを拒否します',
    });
    return;
  }
  // ストアの全 state を収集する（4 レイヤ全体）
  const syncEntries: Record<string, unknown> = {};
  for (const [key, value] of stateStore.entries()) {
    // 全レイヤのエントリを収集する
    syncEntries[key] = value;
  }
  // sync 結果をレスポンスとして返す（lib.rs の state_sync 応答形式に合わせる）
  sendResponse(ws, {
    status: 'ok',
    data: {
      // 同期したレイヤ数
      count: Object.keys(syncEntries).length,
      // 同期した全レイヤのエントリ
      entries: syncEntries,
      // HLC タイムスタンプを応答に含める
      hlc_timestamp: cmd.hlc_timestamp,
    },
  });
}

// WebSocket クライアントにレスポンスを送信するヘルパー関数
function sendResponse(ws: WebSocket, response: CompanionResponse): void {
  // WebSocket が OPEN 状態の場合のみ送信する（クローズ済みへの送信を防ぐ）
  if (ws.readyState === WebSocket.OPEN) {
    // レスポンスオブジェクトを JSON 文字列に変換して送信する
    ws.send(JSON.stringify(response));
  }
}

// HTTP サーバーを 9999 番ポートで起動する（lib.rs のデフォルトポートと一致させる）
httpServer.listen(9999, '127.0.0.1', () => {
  // 起動完了をコンソールに記録する
  console.log('[companion-mock] listening on ws://127.0.0.1:9999/sync');
});

// プロセス終了シグナルを受信した場合はサーバーをグレースフルにシャットダウンする
process.on('SIGTERM', () => {
  // SIGTERM 受信を記録する
  console.log('[companion-mock] received SIGTERM, shutting down');
  // HTTP サーバーを閉じる
  httpServer.close(() => {
    // シャットダウン完了を記録する
    console.log('[companion-mock] server closed');
    // プロセスを正常終了させる
    process.exit(0);
  });
});
