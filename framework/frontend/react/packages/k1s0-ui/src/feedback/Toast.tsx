import React, { createContext, useContext, useCallback, useState, useMemo } from 'react';
import { Snackbar, Alert, Button, type AlertColor } from '@mui/material';
import type { NotificationOptions, NotificationSeverity } from './types.js';

/**
 * トーストの状態
 */
interface ToastState extends NotificationOptions {
  id: number;
  open: boolean;
}

/**
 * トーストコンテキストの値
 */
interface ToastContextValue {
  /** トーストを表示 */
  show: (options: NotificationOptions) => void;
  /** 成功トースト */
  success: (message: string, options?: Partial<NotificationOptions>) => void;
  /** エラートースト */
  error: (message: string, options?: Partial<NotificationOptions>) => void;
  /** 警告トースト */
  warning: (message: string, options?: Partial<NotificationOptions>) => void;
  /** 情報トースト */
  info: (message: string, options?: Partial<NotificationOptions>) => void;
}

const ToastContext = createContext<ToastContextValue | null>(null);

/**
 * ToastProvider のプロパティ
 */
export interface ToastProviderProps {
  children: React.ReactNode;
  /** デフォルトの表示時間（ミリ秒） */
  defaultDuration?: number;
  /** 最大同時表示数 */
  maxToasts?: number;
  /** 表示位置 */
  anchorOrigin?: {
    vertical: 'top' | 'bottom';
    horizontal: 'left' | 'center' | 'right';
  };
}

let toastIdCounter = 0;

/**
 * トーストプロバイダー
 *
 * アプリケーション全体でトースト通知を使用するためのプロバイダー。
 *
 * @example
 * ```tsx
 * function App() {
 *   return (
 *     <ToastProvider>
 *       <MyApp />
 *     </ToastProvider>
 *   );
 * }
 *
 * function MyComponent() {
 *   const toast = useToast();
 *
 *   const handleSave = async () => {
 *     try {
 *       await save();
 *       toast.success('保存しました');
 *     } catch (error) {
 *       toast.error('保存に失敗しました');
 *     }
 *   };
 * }
 * ```
 */
export function ToastProvider({
  children,
  defaultDuration = 5000,
  maxToasts = 3,
  anchorOrigin = { vertical: 'bottom', horizontal: 'right' },
}: ToastProviderProps) {
  const [toasts, setToasts] = useState<ToastState[]>([]);

  const show = useCallback(
    (options: NotificationOptions) => {
      const id = ++toastIdCounter;
      const newToast: ToastState = {
        id,
        open: true,
        severity: options.severity ?? 'info',
        duration: options.duration ?? defaultDuration,
        ...options,
      };

      setToasts((prev) => {
        // 最大数を超えた場合は古いものを削除
        const updated = [...prev, newToast];
        if (updated.length > maxToasts) {
          return updated.slice(-maxToasts);
        }
        return updated;
      });
    },
    [defaultDuration, maxToasts]
  );

  const createShowFn = useCallback(
    (severity: NotificationSeverity) =>
      (message: string, options?: Partial<NotificationOptions>) => {
        show({ message, severity, ...options });
      },
    [show]
  );

  const contextValue = useMemo<ToastContextValue>(
    () => ({
      show,
      success: createShowFn('success'),
      error: createShowFn('error'),
      warning: createShowFn('warning'),
      info: createShowFn('info'),
    }),
    [show, createShowFn]
  );

  const handleClose = useCallback((id: number) => {
    setToasts((prev) =>
      prev.map((t) => (t.id === id ? { ...t, open: false } : t))
    );
    // アニメーション完了後に削除
    setTimeout(() => {
      setToasts((prev) => prev.filter((t) => t.id !== id));
    }, 300);
  }, []);

  return (
    <ToastContext.Provider value={contextValue}>
      {children}
      {toasts.map((toast, index) => (
        <Snackbar
          key={toast.id}
          open={toast.open}
          autoHideDuration={toast.duration === 0 ? null : toast.duration}
          onClose={() => {
            toast.onClose?.();
            handleClose(toast.id);
          }}
          anchorOrigin={anchorOrigin}
          sx={{
            // スタックして表示
            bottom: `${24 + index * 60}px !important`,
          }}
        >
          <Alert
            severity={toast.severity as AlertColor}
            onClose={() => handleClose(toast.id)}
            action={
              toast.actionLabel ? (
                <Button
                  color="inherit"
                  size="small"
                  onClick={() => {
                    toast.onAction?.();
                    handleClose(toast.id);
                  }}
                >
                  {toast.actionLabel}
                </Button>
              ) : undefined
            }
            sx={{ width: '100%' }}
          >
            {toast.message}
          </Alert>
        </Snackbar>
      ))}
    </ToastContext.Provider>
  );
}

/**
 * トーストフック
 *
 * @example
 * ```tsx
 * const toast = useToast();
 * toast.success('成功しました');
 * toast.error('エラーが発生しました');
 * toast.warning('注意してください');
 * toast.info('情報があります');
 * ```
 */
export function useToast(): ToastContextValue {
  const context = useContext(ToastContext);
  if (!context) {
    throw new Error('useToast must be used within a ToastProvider');
  }
  return context;
}
