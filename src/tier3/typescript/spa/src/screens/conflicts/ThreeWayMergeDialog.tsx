// ThreeWayMergeDialog.tsx — lost_update / stale_write rebase_dirty の 3way merge UI
// spec 11 §reducer PRESENT_3WAY_MERGE_UI action を受けて表示するダイアログコンポーネント
// reducer が PRESENT_3WAY_MERGE_UI action を出した際にこのコンポーネントを表示する

// React の必要なフックをインポートする
import React from 'react';

// 3way merge ダイアログの props 型定義
interface ThreeWayMergeDialogProps {
  // ダイアログの表示状態
  open: boolean;
  // 現在のサーバー状態（server truth）
  serverVersion: unknown;
  // ローカルの変更内容
  localChange: unknown;
  // マージ結果を選択した際のコールバック
  onResolve: (resolution: 'accept_server' | 'accept_local' | 'merge') => void;
  // ダイアログを閉じるコールバック
  onClose: () => void;
}

// ThreeWayMergeDialog コンポーネント
export const ThreeWayMergeDialog: React.FC<ThreeWayMergeDialogProps> = ({
  // ダイアログの開閉状態を受け取る
  open,
  // サーバー状態を受け取る
  serverVersion,
  // ローカル変更を受け取る
  localChange,
  // 解決コールバックを受け取る
  onResolve,
  // 閉じるコールバックを受け取る
  onClose,
}) => {
  // ダイアログが閉じている場合は null を返す
  if (!open) return null;

  // サーバー状態の JSON を文字列化する
  const serverJson = JSON.stringify(serverVersion, null, 2);
  // ローカル変更の JSON を文字列化する
  const localJson = JSON.stringify(localChange, null, 2);

  return (
    // モーダルオーバーレイを表示する
    <div role="dialog" aria-modal="true" aria-label="競合解決ダイアログ"
      style={{ position: 'fixed', inset: 0, background: 'rgba(0,0,0,0.5)', zIndex: 9999 }}>
      {/* ダイアログ本体 */}
      <div style={{ background: '#fff', margin: '10vh auto', padding: 24, maxWidth: 600, borderRadius: 8 }}>
        {/* タイトル */}
        <h2>編集競合の解決</h2>
        {/* 説明文 */}
        <p>同じデータが同時に変更されました。どちらを採用するか選択してください。</p>
        {/* サーバー側の変更を表示する */}
        <details>
          <summary>サーバー側の変更</summary>
          {/* サーバーの JSON を整形表示する */}
          <pre style={{ overflow: 'auto', maxHeight: 200 }}>{serverJson}</pre>
        </details>
        {/* ローカル側の変更を表示する */}
        <details>
          <summary>ローカルの変更</summary>
          {/* ローカルの JSON を整形表示する */}
          <pre style={{ overflow: 'auto', maxHeight: 200 }}>{localJson}</pre>
        </details>
        {/* 解決ボタン群 */}
        <div style={{ display: 'flex', gap: 8, marginTop: 16 }}>
          {/* サーバー側を採用するボタン */}
          <button onClick={() => onResolve('accept_server')}>サーバーを採用</button>
          {/* ローカル側を採用するボタン */}
          <button onClick={() => onResolve('accept_local')}>ローカルを採用</button>
          {/* キャンセルボタン */}
          <button onClick={onClose}>後で決める</button>
        </div>
      </div>
    </div>
  );
};
