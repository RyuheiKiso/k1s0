import { useState, useEffect, useRef } from 'react';
import styles from './InputDialog.module.css';

// 入力ダイアログのフィールド定義
interface InputField {
  // フィールドの識別キー
  key: string;
  // フィールドのラベル
  label: string;
  // 入力必須かどうか
  required?: boolean;
  // プレースホルダー
  placeholder?: string;
}

// 入力ダイアログのProps定義
interface InputDialogProps {
  // ダイアログの表示状態
  open: boolean;
  // ダイアログのタイトル
  title: string;
  // 入力フィールドの定義リスト
  fields: InputField[];
  // 送信ボタンのラベル（デフォルト: 送信）
  submitLabel?: string;
  // キャンセルボタンのラベル（デフォルト: キャンセル）
  cancelLabel?: string;
  // 送信時のコールバック: フィールドキーと値のマップを受け取る
  onSubmit: (values: Record<string, string>) => void;
  // キャンセル時のコールバック
  onCancel: () => void;
}

// 入力ダイアログコンポーネント: window.promptの代替としてReactコンポーネントベースで実装
export function InputDialog({
  open,
  title,
  fields,
  submitLabel = '送信',
  cancelLabel = 'キャンセル',
  onSubmit,
  onCancel,
}: InputDialogProps) {
  // 各フィールドの入力値を管理
  const [values, setValues] = useState<Record<string, string>>({});
  // 最初の入力フィールドへのref（フォーカス管理用）
  const firstInputRef = useRef<HTMLInputElement>(null);

  // ダイアログ表示時に値をリセットし、最初のフィールドにフォーカス
  useEffect(() => {
    if (open) {
      const initial: Record<string, string> = {};
      fields.forEach((f) => {
        initial[f.key] = '';
      });
      setValues(initial);
      // レンダリング後にフォーカスを設定
      setTimeout(() => firstInputRef.current?.focus(), 0);
    }
  }, [open, fields]);

  // 非表示時はレンダリングしない
  if (!open) return null;

  // フォーム送信: 必須フィールドの検証後にコールバックを呼び出し
  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    // 必須フィールドが空でないか検証
    const allRequiredFilled = fields.every(
      (f) => !f.required || values[f.key]?.trim()
    );
    if (!allRequiredFilled) return;
    onSubmit(values);
  };

  return (
    <div className={styles.overlay} role="dialog" aria-modal="true" aria-labelledby="input-dialog-title">
      <div className={styles.dialog}>
        <h2 id="input-dialog-title" className={styles.title}>{title}</h2>
        <form onSubmit={handleSubmit}>
          {fields.map((field, index) => (
            <div key={field.key} className={styles.field}>
              <label htmlFor={`input-dialog-${field.key}`} className={styles.label}>
                {field.label}
                {field.required && <span className={styles.required}> *</span>}
              </label>
              <input
                ref={index === 0 ? firstInputRef : undefined}
                id={`input-dialog-${field.key}`}
                className={styles.input}
                value={values[field.key] ?? ''}
                placeholder={field.placeholder}
                onChange={(e) =>
                  setValues((prev) => ({ ...prev, [field.key]: e.target.value }))
                }
                required={field.required}
                aria-required={field.required}
              />
            </div>
          ))}
          <div className={styles.actions}>
            <button
              type="button"
              className={styles.cancelButton}
              onClick={onCancel}
              aria-label={cancelLabel}
            >
              {cancelLabel}
            </button>
            <button
              type="submit"
              className={styles.submitButton}
              aria-label={submitLabel}
            >
              {submitLabel}
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}
