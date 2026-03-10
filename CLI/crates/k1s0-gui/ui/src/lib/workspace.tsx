import {
  createContext,
  useContext,
  useEffect,
  useState,
  type Dispatch,
  type ReactNode,
  type SetStateAction,
} from 'react';
import { detectWorkspaceRoot, resolveWorkspaceRoot } from './tauri-commands';

const STORAGE_KEY = 'k1s0.workspaceRoot';

interface WorkspaceContextValue {
  workspaceRoot: string;
  draftPath: string;
  ready: boolean;
  resolving: boolean;
  errorMessage: string;
  setDraftPath: Dispatch<SetStateAction<string>>;
  applyWorkspace: () => Promise<boolean>;
  detectWorkspace: () => Promise<boolean>;
}

const defaultContextValue: WorkspaceContextValue = {
  workspaceRoot: '.',
  draftPath: '.',
  ready: true,
  resolving: false,
  errorMessage: '',
  setDraftPath: () => undefined,
  applyWorkspace: async () => false,
  detectWorkspace: async () => false,
};

const WorkspaceContext = createContext<WorkspaceContextValue>(defaultContextValue);

export function WorkspaceProvider({ children }: { children: ReactNode }) {
  const [workspaceRoot, setWorkspaceRoot] = useState('');
  const [draftPath, setDraftPath] = useState('');
  const [ready, setReady] = useState(false);
  const [resolving, setResolving] = useState(false);
  const [errorMessage, setErrorMessage] = useState('');

  async function resolveAndStore(path: string): Promise<boolean> {
    if (!path.trim()) {
      setWorkspaceRoot('');
      setErrorMessage('Enter a workspace path.');
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
  }

  async function detectAndStore(): Promise<boolean> {
    setResolving(true);
    try {
      const detected = await detectWorkspaceRoot();
      if (!detected) {
        setWorkspaceRoot('');
        setErrorMessage('No k1s0 workspace was detected from the current working directory.');
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
  }

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
  }, []);

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
      }}
    >
      {children}
    </WorkspaceContext.Provider>
  );
}

export function useWorkspace() {
  return useContext(WorkspaceContext);
}
