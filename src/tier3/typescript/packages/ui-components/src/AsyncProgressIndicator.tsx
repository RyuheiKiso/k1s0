// AsyncProgressIndicator.tsx — 非同期処理 UX progress + cancel ボタン
// 設計方針 23_非同期処理 UX

// React の useState/useEffect/useCallback をインポートする
import { useState, useEffect, useCallback } from 'react';

// AsyncProgressIndicator の Props 型を定義する
interface AsyncProgressIndicatorProps {
  // 処理の説明テキスト
  label: string;
  // キャンセルコールバック（undefined の場合はキャンセル不可）
  onCancel?: () => void;
  // 完了フラグ
  isComplete?: boolean;
}

// AsyncProgressIndicator コンポーネントを定義する
export function AsyncProgressIndicator({
  label,
  onCancel,
  isComplete = false,
}: AsyncProgressIndicatorProps): JSX.Element | null {
  // 表示状態を管理する state
  const [isVisible, setIsVisible] = useState(true);

  // 完了時に 500ms 後に非表示にする副作用
  useEffect(() => {
    // 完了していない場合は何もしない
    if (!isComplete) return;
    // 500ms 後に非表示にするタイマーを設定する
    const timer = setTimeout(() => setIsVisible(false), 500);
    // クリーンアップ関数でタイマーをキャンセルする
    return () => clearTimeout(timer);
  }, [isComplete]);

  // 非表示の場合は null を返す
  if (!isVisible) return null;

  return (
    <div role="status" aria-live="polite" aria-label={`処理中: ${label}`}>
      {/* スピナーアニメーション（スクリーンリーダーには非表示） */}
      <div aria-hidden="true" className="spinner" />
      {/* 処理の説明テキストを表示する */}
      <span>{label}</span>
      {/* onCancel が設定されている場合のみキャンセルボタンを表示する */}
      {onCancel && (
        <button onClick={onCancel} aria-label="処理をキャンセルする">
          キャンセル
        </button>
      )}
    </div>
  );
}
