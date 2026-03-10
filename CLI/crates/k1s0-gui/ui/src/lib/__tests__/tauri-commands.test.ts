import { beforeEach, describe, expect, it, vi } from 'vitest';
import { mockInvoke } from '../../test/mocks';
import {
  clearAuthSession,
  detectWorkspaceRoot,
  executeBuildWithProgress,
  executeGenerateAt,
  executeTestWithProgressAt,
  getAuthSession,
  getFailedProdRollbackTarget,
  pollDeviceAuthorization,
  resolveWorkspaceRoot,
  scanBuildableTargets,
  startDeviceAuthorization,
} from '../tauri-commands';

beforeEach(() => {
  mockInvoke.mockReset();
});

describe('tauri-commands', () => {
  it('invokes workspace scan commands with the provided base dir', async () => {
    mockInvoke.mockResolvedValue(['regions/system/server/rust/auth']);
    const result = await scanBuildableTargets('/repo');
    expect(mockInvoke).toHaveBeenCalledWith('scan_buildable_targets', { baseDir: '/repo' });
    expect(result).toEqual(['regions/system/server/rust/auth']);
  });

  it('invokes execute_generate_at with workspace root', async () => {
    mockInvoke.mockResolvedValue(undefined);
    await executeGenerateAt(
      {
        kind: 'Server',
        tier: 'System',
        placement: null,
        lang_fw: { Language: 'Go' },
        detail: {
          name: 'auth',
          api_styles: ['Rest'],
          db: null,
          kafka: false,
          redis: false,
          bff_language: null,
        },
      },
      '/repo',
    );

    expect(mockInvoke).toHaveBeenCalledWith('execute_generate_at', {
      config: expect.objectContaining({ kind: 'Server', tier: 'System' }),
      baseDir: '/repo',
    });
  });

  it('invokes execute_test_with_progress_at with a channel', async () => {
    mockInvoke.mockResolvedValue(undefined);
    const onEvent = vi.fn();

    await executeTestWithProgressAt({ kind: 'All', targets: [] }, '/repo', onEvent);

    expect(mockInvoke).toHaveBeenCalledWith('execute_test_with_progress_at', {
      config: { kind: 'All', targets: [] },
      baseDir: '/repo',
      onEvent: expect.any(Object),
    });
  });

  it('invokes execute_build_with_progress with a channel', async () => {
    mockInvoke.mockResolvedValue(undefined);
    const onEvent = vi.fn();

    await executeBuildWithProgress({ targets: ['target-1'], mode: 'Development' }, onEvent);

    expect(mockInvoke).toHaveBeenCalledWith('execute_build_with_progress', {
      config: { targets: ['target-1'], mode: 'Development' },
      onEvent: expect.any(Object),
    });
  });

  it('wraps workspace root commands', async () => {
    mockInvoke.mockResolvedValueOnce('/repo').mockResolvedValueOnce('/repo');

    expect(await detectWorkspaceRoot()).toBe('/repo');
    expect(await resolveWorkspaceRoot('/repo')).toBe('/repo');

    expect(mockInvoke).toHaveBeenNthCalledWith(1, 'detect_workspace_root');
    expect(mockInvoke).toHaveBeenNthCalledWith(2, 'resolve_workspace_root', { path: '/repo' });
  });

  it('wraps device authorization commands', async () => {
    const challenge = {
      issuer: 'https://issuer.example.com',
      client_id: 'client',
      scope: 'openid',
      token_endpoint: 'https://issuer.example.com/token',
      device_code: 'device-code',
      user_code: 'user-code',
      verification_uri: 'https://issuer.example.com/verify',
      verification_uri_complete: 'https://issuer.example.com/verify?user_code=user-code',
      interval: 5,
      expires_in: 600,
    };

    mockInvoke
      .mockResolvedValueOnce(challenge)
      .mockResolvedValueOnce({ status: 'Pending', interval: 5, message: 'pending' });

    expect(await startDeviceAuthorization()).toEqual(challenge);
    expect(await pollDeviceAuthorization(challenge)).toEqual({
      status: 'Pending',
      interval: 5,
      message: 'pending',
    });
  });

  it('wraps auth session commands', async () => {
    const session = {
      issuer: 'https://issuer.example.com',
      authenticated_at_epoch_secs: 1_700_000_000,
      expires_at_epoch_secs: 1_700_000_600,
      token_type: 'Bearer',
      scope: 'openid profile',
      can_refresh: true,
    };

    mockInvoke
      .mockResolvedValueOnce(session)
      .mockResolvedValueOnce(undefined)
      .mockResolvedValueOnce('regions/system/server/rust/auth');

    expect(await getAuthSession()).toEqual(session);
    await clearAuthSession();
    expect(await getFailedProdRollbackTarget()).toBe('regions/system/server/rust/auth');

    expect(mockInvoke).toHaveBeenNthCalledWith(1, 'get_auth_session');
    expect(mockInvoke).toHaveBeenNthCalledWith(2, 'clear_auth_session');
    expect(mockInvoke).toHaveBeenNthCalledWith(3, 'get_failed_prod_rollback_target');
  });
});
