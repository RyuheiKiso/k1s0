import React, { createContext, useContext, useCallback, useState, useMemo, useRef } from 'react';
import {
  Dialog,
  DialogTitle,
  DialogContent,
  DialogContentText,
  DialogActions,
  Button,
} from '@mui/material';
import type { ConfirmDialogOptions, ConfirmDialogResult } from './types.js';

/**
 * 確認ダイアログの状態
 */
interface ConfirmDialogState extends ConfirmDialogOptions {
  open: boolean;
}

/**
 * 確認ダイアログコンテキストの値
 */
interface ConfirmDialogContextValue {
  /** 確認ダイアログを表示 */
  confirm: (options: ConfirmDialogOptions) => Promise<boolean>;
}

const ConfirmDialogContext = createContext<ConfirmDialogContextValue | null>(null);

const defaultState: ConfirmDialogState = {
  open: false,
  title: '',
  message: '',
  confirmLabel: '確認',
  cancelLabel: 'キャンセル',
  confirmColor: 'primary',
  dangerous: false,
};

/**
 * ConfirmDialogProvider のプロパティ
 */
export interface ConfirmDialogProviderProps {
  children: React.ReactNode;
}

/**
 * 確認ダイアログプロバイダー
 *
 * アプリケーション全体で確認ダイアログを使用するためのプロバイダー。
 *
 * @example
 * ```tsx
 * function App() {
 *   return (
 *     <ConfirmDialogProvider>
 *       <MyApp />
 *     </ConfirmDialogProvider>
 *   );
 * }
 *
 * function DeleteButton() {
 *   const { confirm } = useConfirmDialog();
 *
 *   const handleDelete = async () => {
 *     const confirmed = await confirm({
 *       title: '削除の確認',
 *       message: 'このアイテムを削除しますか？この操作は取り消せません。',
 *       confirmLabel: '削除',
 *       dangerous: true,
 *     });
 *
 *     if (confirmed) {
 *       await deleteItem();
 *     }
 *   };
 *
 *   return <Button onClick={handleDelete}>削除</Button>;
 * }
 * ```
 */
export function ConfirmDialogProvider({ children }: ConfirmDialogProviderProps) {
  const [state, setState] = useState<ConfirmDialogState>(defaultState);
  const resolveRef = useRef<((confirmed: boolean) => void) | null>(null);

  const confirm = useCallback((options: ConfirmDialogOptions): Promise<boolean> => {
    return new Promise((resolve) => {
      resolveRef.current = resolve;
      setState({
        ...defaultState,
        ...options,
        open: true,
        confirmColor: options.dangerous ? 'error' : (options.confirmColor ?? 'primary'),
      });
    });
  }, []);

  const handleClose = useCallback((confirmed: boolean) => {
    setState((prev) => ({ ...prev, open: false }));
    resolveRef.current?.(confirmed);
    resolveRef.current = null;
  }, []);

  const contextValue = useMemo<ConfirmDialogContextValue>(
    () => ({ confirm }),
    [confirm]
  );

  return (
    <ConfirmDialogContext.Provider value={contextValue}>
      {children}
      <Dialog
        open={state.open}
        onClose={() => handleClose(false)}
        aria-labelledby="confirm-dialog-title"
        aria-describedby="confirm-dialog-description"
      >
        <DialogTitle id="confirm-dialog-title">{state.title}</DialogTitle>
        <DialogContent>
          <DialogContentText id="confirm-dialog-description">
            {state.message}
          </DialogContentText>
        </DialogContent>
        <DialogActions>
          <Button onClick={() => handleClose(false)} color="inherit">
            {state.cancelLabel}
          </Button>
          <Button
            onClick={() => handleClose(true)}
            color={state.confirmColor}
            variant="contained"
            autoFocus
          >
            {state.confirmLabel}
          </Button>
        </DialogActions>
      </Dialog>
    </ConfirmDialogContext.Provider>
  );
}

/**
 * 確認ダイアログフック
 *
 * @example
 * ```tsx
 * const { confirm } = useConfirmDialog();
 *
 * const handleAction = async () => {
 *   const confirmed = await confirm({
 *     title: '確認',
 *     message: '続行しますか？',
 *   });
 *
 *   if (confirmed) {
 *     // 確認された場合の処理
 *   }
 * };
 * ```
 */
export function useConfirmDialog(): ConfirmDialogContextValue {
  const context = useContext(ConfirmDialogContext);
  if (!context) {
    throw new Error('useConfirmDialog must be used within a ConfirmDialogProvider');
  }
  return context;
}

/**
 * スタンドアロン確認ダイアログコンポーネント
 *
 * プロバイダーを使わずに確認ダイアログを表示したい場合に使用。
 */
export interface StandaloneConfirmDialogProps extends ConfirmDialogOptions {
  /** 開いているかどうか */
  open: boolean;
  /** 結果のコールバック */
  onResult: (confirmed: boolean) => void;
}

export function StandaloneConfirmDialog({
  open,
  title,
  message,
  confirmLabel = '確認',
  cancelLabel = 'キャンセル',
  confirmColor = 'primary',
  dangerous = false,
  onResult,
}: StandaloneConfirmDialogProps) {
  const effectiveColor = dangerous ? 'error' : confirmColor;

  return (
    <Dialog
      open={open}
      onClose={() => onResult(false)}
      aria-labelledby="confirm-dialog-title"
      aria-describedby="confirm-dialog-description"
    >
      <DialogTitle id="confirm-dialog-title">{title}</DialogTitle>
      <DialogContent>
        <DialogContentText id="confirm-dialog-description">
          {message}
        </DialogContentText>
      </DialogContent>
      <DialogActions>
        <Button onClick={() => onResult(false)} color="inherit">
          {cancelLabel}
        </Button>
        <Button
          onClick={() => onResult(true)}
          color={effectiveColor}
          variant="contained"
          autoFocus
        >
          {confirmLabel}
        </Button>
      </DialogActions>
    </Dialog>
  );
}
