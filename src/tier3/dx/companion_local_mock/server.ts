// server.ts — tier3 companion ローカルモック WebSocket サーバー
// ws://127.0.0.1:9999 で待ち受けて state_read/state_write/state_sync コマンドに応答する
// このサーバーは開発・テスト専用であり本番環境では使用しない

// Node.js 組み込み http モジュールをインポートする
import { createServer } from 'http';
// ws ライブラリをインポートする（WebSocket プロトコル実装）
import { WebSocketServer, WebSocket } from 'ws';

// 受信メッセージの型定義（companion プロトコルのコマンド形式）
interface CompanionCommand {
  // コマンド識別子（state_read / state_write / state_sync）
  cmd: 'state_read' | 'state_write' | 'state_sync';
  // リクエスト追跡用 ID（クライアントが設定する）
  requestId: string;
  // state_read 用: 読み取るキー名
  key?: string;
  // state_write 用: 書き込む値
  value?: unknown;
  // state_sync 用: 同期するエンティティ ID
  entityId?: string;
}

// サーバー応答の型定義
interface CompanionResponse {
  // 元のリクエスト ID（コマンドの requestId に対応する）
  requestId: string;
  // 成功 / エラーの区別
  status: 'ok' | 'error';
  // 成功時のデータ（state_read / state_sync が返す値）
  data?: unknown;
  // エラー時のメッセージ
  error?: string;
}

// インメモリ state ストア（state_write で書き込み、state_read で読み取る）
const stateStore = new Map<string, unknown>();

// HTTP サーバーを作成する（WebSocket サーバーのアタッチ先として使用する）
const httpServer = createServer((req, res) => {
  // ヘルスチェックエンドポイント（GET /health）
  if (req.method === 'GET' && req.url === '/health') {
    // 200 OK を返す（Tilt の readiness probe 用）
    res.writeHead(200, { 'Content-Type': 'application/json' });
    res.end(JSON.stringify({ status: 'ok', connections: wss.clients.size }));
    return;
  }
  // その他のリクエストは 404 を返す
  res.writeHead(404);
  res.end();
});

// WebSocket サーバーを作成して HTTP サーバーにアタッチする
const wss = new WebSocketServer({ server: httpServer });

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
        requestId: 'unknown',
        status: 'error',
        error: `invalid JSON: ${raw}`,
      });
      return;
    }

    // コマンド種別に応じてハンドラーに分岐する
    switch (cmd.cmd) {
      // state_read: 指定キーの値をストアから読み取る
      case 'state_read':
        handleStateRead(ws, cmd);
        break;
      // state_write: 指定キーに値をストアへ書き込む
      case 'state_write':
        handleStateWrite(ws, cmd);
        break;
      // state_sync: エンティティ ID に紐づく全 state を同期する
      case 'state_sync':
        handleStateSync(ws, cmd);
        break;
      default: {
        // 未知のコマンドはエラーを返す
        const unknownCmd = (cmd as { cmd: string }).cmd;
        sendResponse(ws, {
          requestId: cmd.requestId,
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

// state_read コマンドのハンドラー
function handleStateRead(ws: WebSocket, cmd: CompanionCommand): void {
  // key が指定されていない場合はエラーを返す
  if (cmd.key === undefined || cmd.key === '') {
    sendResponse(ws, {
      requestId: cmd.requestId,
      status: 'error',
      error: 'state_read requires key',
    });
    return;
  }
  // ストアから値を読み取る（存在しない場合は null を返す）
  const value = stateStore.get(cmd.key) ?? null;
  // 読み取った値をレスポンスとして返す
  sendResponse(ws, {
    requestId: cmd.requestId,
    status: 'ok',
    data: {
      // キー名
      key: cmd.key,
      // 値（存在しない場合は null）
      value,
      // ストアに存在するか否かのフラグ
      found: stateStore.has(cmd.key),
    },
  });
}

// state_write コマンドのハンドラー
function handleStateWrite(ws: WebSocket, cmd: CompanionCommand): void {
  // key が指定されていない場合はエラーを返す
  if (cmd.key === undefined || cmd.key === '') {
    sendResponse(ws, {
      requestId: cmd.requestId,
      status: 'error',
      error: 'state_write requires key',
    });
    return;
  }
  // ストアに値を書き込む
  stateStore.set(cmd.key, cmd.value);
  // 書き込み成功のレスポンスを返す
  sendResponse(ws, {
    requestId: cmd.requestId,
    status: 'ok',
    data: {
      // 書き込んだキー名
      key: cmd.key,
      // 書き込んだ値
      writtenValue: cmd.value,
    },
  });
  // 接続中の全クライアントに state 変更を通知する（broadcast）
  broadcastStateChange(cmd.key, cmd.value);
}

// state_sync コマンドのハンドラー
function handleStateSync(ws: WebSocket, cmd: CompanionCommand): void {
  // entityId が指定されていない場合は全 state を返す
  const entityId = cmd.entityId;
  // entityId プレフィックスでフィルタリングした state エントリを収集する
  const syncEntries: Record<string, unknown> = {};
  for (const [key, value] of stateStore.entries()) {
    // entityId が指定されている場合は該当エントリのみを返す
    if (entityId === undefined || key.startsWith(entityId)) {
      // フィルタリングされたエントリをオブジェクトに追加する
      syncEntries[key] = value;
    }
  }
  // sync 結果をレスポンスとして返す
  sendResponse(ws, {
    requestId: cmd.requestId,
    status: 'ok',
    data: {
      // 同期対象の entityId（undefined の場合は全件）
      entityId: entityId ?? '*',
      // 同期したエントリ数
      count: Object.keys(syncEntries).length,
      // 同期したエントリ一覧
      entries: syncEntries,
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

// 全接続クライアントに state 変更を broadcast するヘルパー関数
function broadcastStateChange(key: string, value: unknown): void {
  // broadcast メッセージを構築する
  const notification = JSON.stringify({
    // 通知種別（state_changed は予約済みイベント名）
    event: 'state_changed',
    // 変更されたキー名
    key,
    // 変更後の値
    value,
  });
  // 全 WebSocket クライアントに通知を送信する
  for (const client of wss.clients) {
    // クライアントが OPEN 状態の場合のみ送信する
    if (client.readyState === WebSocket.OPEN) {
      // 通知を送信する
      client.send(notification);
    }
  }
}

// HTTP サーバーを 9999 番ポートで起動する
httpServer.listen(9999, '127.0.0.1', () => {
  // 起動完了をコンソールに記録する
  console.log('[companion-mock] listening on ws://127.0.0.1:9999');
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
