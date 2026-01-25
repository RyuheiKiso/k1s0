import React from 'react';
import { ToastProvider, type ToastProviderProps } from './Toast.js';
import { ConfirmDialogProvider } from './ConfirmDialog.js';

/**
 * FeedbackProvider のプロパティ
 */
export interface FeedbackProviderProps extends ToastProviderProps {}

/**
 * フィードバックプロバイダー
 *
 * トーストと確認ダイアログのプロバイダーを統合。
 * アプリケーションのルートで使用することで、
 * どこからでも通知や確認ダイアログを使用できる。
 *
 * @example
 * ```tsx
 * import { FeedbackProvider } from '@k1s0/ui/feedback';
 *
 * function App() {
 *   return (
 *     <FeedbackProvider>
 *       <MyApp />
 *     </FeedbackProvider>
 *   );
 * }
 * ```
 */
export function FeedbackProvider({
  children,
  ...toastProps
}: FeedbackProviderProps) {
  return (
    <ToastProvider {...toastProps}>
      <ConfirmDialogProvider>
        {children}
      </ConfirmDialogProvider>
    </ToastProvider>
  );
}
