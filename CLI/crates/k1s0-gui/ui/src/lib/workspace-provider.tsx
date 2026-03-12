import { useCallback, useEffect, useState, type ReactNode } from 'react';
import { detectWorkspaceRoot, resolveWorkspaceRoot } from './tauri-commands';
import { STORAGE_KEY, WorkspaceContext } from './workspace';

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
      window.localStorage.setItem(STORAGE_KEY, resolved);
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
      const saved = window.localStorage.getItem(STORAGE_KEY);
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
