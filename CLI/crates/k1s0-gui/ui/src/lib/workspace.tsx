import {
  createContext,
  useContext,
  type Dispatch,
  type SetStateAction,
} from 'react';

export const STORAGE_KEY = 'k1s0.workspaceRoot';

export interface WorkspaceContextValue {
  workspaceRoot: string;
  draftPath: string;
  ready: boolean;
  resolving: boolean;
  errorMessage: string;
  setDraftPath: Dispatch<SetStateAction<string>>;
  applyWorkspace: () => Promise<boolean>;
  detectWorkspace: () => Promise<boolean>;
  adoptWorkspace: (path: string) => Promise<boolean>;
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
  adoptWorkspace: async () => false,
};

export const WorkspaceContext = createContext<WorkspaceContextValue>(defaultContextValue);

export function useWorkspace() {
  return useContext(WorkspaceContext);
}
