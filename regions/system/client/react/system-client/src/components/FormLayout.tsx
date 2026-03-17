import type { ReactNode, FormEvent } from 'react';

// FormLayoutのProps定義
interface FormLayoutProps {
  // フォームのタイトル
  title: string;
  // フォーム内のコンテンツ（フォームフィールド等）
  children: ReactNode;
  // フォーム送信時のコールバック
  onSubmit: (e: FormEvent) => void;
  // 送信ボタンのラベル（デフォルト: 送信）
  submitLabel?: string;
  // キャンセルボタン押下時のコールバック（任意: 指定時にキャンセルボタンを表示）
  onCancel?: () => void;
  // キャンセルボタンのラベル（デフォルト: キャンセル）
  cancelLabel?: string;
  // 送信ボタンの無効状態
  disabled?: boolean;
  // エラーメッセージ（任意）
  error?: string;
}

// 標準フォームレイアウトコンポーネント: タイトル・フィールド・アクションボタンの統一的なレイアウトを提供
export function FormLayout({
  title,
  children,
  onSubmit,
  submitLabel = '送信',
  onCancel,
  cancelLabel = 'キャンセル',
  disabled = false,
  error,
}: FormLayoutProps) {
  return (
    <section aria-label={title}>
      <h1>{title}</h1>
      <form onSubmit={onSubmit}>
        {/* フォームフィールド領域 */}
        <div style={{ marginBottom: '16px' }}>{children}</div>

        {/* アクションボタン */}
        <div style={{ display: 'flex', gap: '8px' }}>
          <button type="submit" disabled={disabled} aria-label={submitLabel}>
            {submitLabel}
          </button>
          {onCancel && (
            <button type="button" onClick={onCancel} aria-label={cancelLabel}>
              {cancelLabel}
            </button>
          )}
        </div>

        {/* エラーメッセージ表示 */}
        {error && (
          <p style={{ color: 'red', fontSize: '0.85em', marginTop: '8px' }} role="alert">
            {error}
          </p>
        )}
      </form>
    </section>
  );
}
