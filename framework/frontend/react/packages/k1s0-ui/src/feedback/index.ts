/**
 * フィードバックコンポーネント（通知・ダイアログ）
 *
 * @packageDocumentation
 */

// 型定義
export type {
  NotificationSeverity,
  NotificationOptions,
  ConfirmDialogOptions,
  ConfirmDialogResult,
} from './types.js';

// トースト
export {
  ToastProvider,
  useToast,
  type ToastProviderProps,
} from './Toast.js';

// 確認ダイアログ
export {
  ConfirmDialogProvider,
  useConfirmDialog,
  StandaloneConfirmDialog,
  type ConfirmDialogProviderProps,
  type StandaloneConfirmDialogProps,
} from './ConfirmDialog.js';

// 統合プロバイダー
export {
  FeedbackProvider,
  type FeedbackProviderProps,
} from './FeedbackProvider.js';
