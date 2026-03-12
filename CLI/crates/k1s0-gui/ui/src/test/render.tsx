import { render, type RenderOptions } from '@testing-library/react';
import type { ReactElement } from 'react';
import { AuthContext, type AuthContextValue } from '../lib/auth';
import { WorkspaceContext, type WorkspaceContextValue } from '../lib/workspace';

type ProviderOptions = {
  auth?: Partial<AuthContextValue>;
  workspace?: Partial<WorkspaceContextValue>;
} & Omit<RenderOptions, 'wrapper'>;

const defaultAuthContext: AuthContextValue = {
  session: {
    issuer: 'https://issuer.example.com',
    authenticated_at_epoch_secs: 1_700_000_000,
    expires_at_epoch_secs: 1_800_000_000,
    token_type: 'Bearer',
    scope: 'openid profile',
    can_refresh: true,
  },
  isAuthenticated: true,
  loading: false,
  refreshSession: async () => null,
  setSession: () => undefined,
  clearSession: async () => undefined,
};

const defaultWorkspaceContext: WorkspaceContextValue = {
  workspaceRoot: '.',
  draftPath: '.',
  ready: true,
  resolving: false,
  errorMessage: '',
  setDraftPath: () => undefined,
  applyWorkspace: async () => true,
  detectWorkspace: async () => true,
  adoptWorkspace: async () => true,
};

export function renderWithProviders(ui: ReactElement, options: ProviderOptions = {}) {
  const { auth, workspace, ...renderOptions } = options;
  const authValue: AuthContextValue = {
    ...defaultAuthContext,
    ...auth,
  };
  const workspaceValue: WorkspaceContextValue = {
    ...defaultWorkspaceContext,
    ...workspace,
  };

  return render(
    <WorkspaceContext.Provider value={workspaceValue}>
      <AuthContext.Provider value={authValue}>{ui}</AuthContext.Provider>
    </WorkspaceContext.Provider>,
    renderOptions,
  );
}
