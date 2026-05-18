// resume_token.ts — HLC ベースの resume token 実装
// spec 11 §リアルタイム更新: subscription.ts の resume 引数の実装本体
// 05_リアルタイム更新UX.md §resume_token 再接続 に対応する

// resume token のフォーマット
export interface ResumeToken {
  // HLC タイムスタンプ（ミリ秒）: 最後に受信したイベントの論理時刻
  hlcMs: number;
  // HLC カウンタ: 同一ミリ秒内の順序を保証する
  hlcCounter: number;
  // チャンネル識別子
  channelId: string;
}

// ResumeToken を文字列に encode する関数（HTTP ヘッダで安全に送信できる形式）
export function encodeResumeToken(token: ResumeToken): string {
  // JSON を Base64 エンコードして返す
  return btoa(JSON.stringify(token));
}

// 文字列を ResumeToken に decode する関数
export function decodeResumeToken(encoded: string): ResumeToken | null {
  // デコードを試みる
  try {
    // Base64 デコードして JSON パースする
    return JSON.parse(atob(encoded)) as ResumeToken;
  } catch {
    // デコード失敗時は null を返す
    return null;
  }
}

// ResumeToken を生成する関数（現在の HLC 状態から生成する）
export function createResumeToken(channelId: string, hlcMs: number, hlcCounter: number): ResumeToken {
  // 渡された HLC 状態から token を生成する
  return { hlcMs, hlcCounter, channelId };
}
