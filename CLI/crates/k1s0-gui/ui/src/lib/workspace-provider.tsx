// MEDIUM-009 監査対応: ワークスペースパスの保存先を localStorage から Tauri Store に変更する。
// WebView 内のスクリプトから読み取り可能な localStorage を使用しないことで、
// XSS 攻撃時のパス情報窃取リスクを排除する。
// LazyStore はモジュールトップレベルで同期的に初期化可能で、初回アクセス時にストアをロードする。
// new Store() は Tauri plugin-store v2 で deprecated のため LazyStore を使用する。
import { LazyStore } from '@tauri-apps/plugin-store';
import { useCallback, useEffect, useState, type ReactNode } from 'react';

import { detectWorkspaceRoot, resolveWorkspaceRoot } from './tauri-commands';
import { STORAGE_KEY, WorkspaceContext } from './workspace';

// Tauri Store のファイル名（OS アプリデータディレクトリに保存される）
const WORKSPACE_STORE_FILENAME = 'workspace.json';

// Tauri Store 内でのキー名
const WORKSPACE_STORE_KEY = 'path';

// LazyStore インスタンスを生成する（モジュール読み込み時に一度だけ作成する）。
// LazyStore は初回の get/set 呼び出し時に自動的にストアをロードするため、
// await なしでモジュールトップレベルに定義できる。
const workspaceStore = new LazyStore(WORKSPACE_STORE_FILENAME);

/**
 * localStorage から Tauri Store へのマイグレーションを行う（初回のみ）。
 * localStorage に既存のワークスペースパスが存在する場合は Tauri Store へ移行し、localStorage から削除する。
 */
async function migrateWorkspaceFromLocalStorage(): Promise<void> {
  try {
    const legacyValue = window.localStorage.getItem(STORAGE_KEY);
    if (!legacyValue) {
      return;
    }
    // Tauri Store への書き込みが成功した場合のみ localStorage から削除する
    await workspaceStore.set(WORKSPACE_STORE_KEY, legacyValue);
    await workspaceStore.save();
    window.localStorage.removeItem(STORAGE_KEY);
  } catch {
    // マイグレーション失敗は無視する（次回起動時に再試行される）
  }
}

export function WorkspaceProvider({ children }: { children: ReactNode }) {
  const [workspaceRoot, setWorkspaceRoot] = useState('');
  const [draftPath, setDraftPath] = useState('');
  const [ready, setReady] = useState(false);
  const [resolving, setResolving] = useState(false);
  const [errorMessage, setErrorMessage] = useState('');

  const resolveAndStore = useCallback(async (path: string): Promise<boolean> => {
    if (!path.trim()) {
      setWorkspaceRoot('');
      setErrorMessage('ワークスペースパスを入力してください。');
      return false;
    }

    setResolving(true);
    try {
      const resolved = await resolveWorkspaceRoot(path.trim());
      setWorkspaceRoot(resolved);
      setDraftPath(resolved);
      setErrorMessage('');
      // Tauri Store にワークスペースパスを保存する（localStorage に代わる安全なストレージ）
      await workspaceStore.set(WORKSPACE_STORE_KEY, resolved);
      await workspaceStore.save();
      return true;
    } catch (error) {
      setWorkspaceRoot('');
      setErrorMessage(String(error));
      return false;
    } finally {
      setResolving(false);
      setReady(true);
    }
  }, []);

  const detectAndStore = useCallback(async (): Promise<boolean> => {
    setResolving(true);
    try {
      const detected = await detectWorkspaceRoot();
      if (!detected) {
        setWorkspaceRoot('');
        setErrorMessage('カレントディレクトリからk1s0ワークスペースが検出されませんでした。');
        return false;
      }
      return await resolveAndStore(detected);
    } catch (error) {
      setWorkspaceRoot('');
      setErrorMessage(String(error));
      return false;
    } finally {
      setResolving(false);
      setReady(true);
    }
  }, [resolveAndStore]);

  useEffect(() => {
    let cancelled = false;

    async function bootstrap() {
      // localStorage にレガシーデータが存在する場合は先に Tauri Store へ移行する
      await migrateWorkspaceFromLocalStorage();

      // Tauri Store から保存済みのワークスペースパスを読み込む
      let saved: string | null = null;
      try {
        // plugin-store v2 の get() は T | undefined を返すため null に変換する
        saved = (await workspaceStore.get<string>(WORKSPACE_STORE_KEY)) ?? null;
      } catch {
        // 読み込み失敗は無視して自動検出にフォールバックする
      }

      if (saved) {
        setDraftPath(saved);
        const ok = await resolveAndStore(saved);
        if (!cancelled && ok) {
          return;
        }
      }

      if (!cancelled) {
        await detectAndStore();
      }
    }

    void bootstrap();

    return () => {
      cancelled = true;
    };
  }, [detectAndStore, resolveAndStore]);

  return (
    <WorkspaceContext.Provider
      value={{
        workspaceRoot,
        draftPath,
        ready,
        resolving,
        errorMessage,
        setDraftPath,
        applyWorkspace: () => resolveAndStore(draftPath),
        detectWorkspace: detectAndStore,
        adoptWorkspace: resolveAndStore,
      }}
    >
      {children}
    </WorkspaceContext.Provider>
  );
}
