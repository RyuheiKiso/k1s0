/**
 * フィードバック関連の型定義
 */

/**
 * 通知の種類
 */
export type NotificationSeverity = 'success' | 'error' | 'warning' | 'info';

/**
 * 通知オプション
 */
export interface NotificationOptions {
  /** メッセージ */
  message: string;
  /** 種類 */
  severity?: NotificationSeverity;
  /** 表示時間（ミリ秒、0で自動非表示しない） */
  duration?: number;
  /** アクションボタンのラベル */
  actionLabel?: string;
  /** アクションボタンクリック時のコールバック */
  onAction?: () => void;
  /** 閉じた時のコールバック */
  onClose?: () => void;
}

/**
 * 確認ダイアログのオプション
 */
export interface ConfirmDialogOptions {
  /** タイトル */
  title: string;
  /** メッセージ */
  message: string;
  /** 確認ボタンのラベル */
  confirmLabel?: string;
  /** キャンセルボタンのラベル */
  cancelLabel?: string;
  /** 確認ボタンの色 */
  confirmColor?: 'primary' | 'error' | 'warning' | 'success';
  /** 危険な操作かどうか（確認ボタンが赤くなる） */
  dangerous?: boolean;
}

/**
 * 確認ダイアログの結果
 */
export interface ConfirmDialogResult {
  /** 確認されたかどうか */
  confirmed: boolean;
}
