import { beforeEach, describe, expect, it, vi } from 'vitest';
import { mockInvoke } from '../../test/mocks';
import {
  clearAuthSession,
  detectWorkspaceRoot,
  executeBuildWithProgress,
  executeInitAt,
  executeGenerateAt,
  executeTemplateMigration,
  executeTestWithProgressAt,
  getCurrentDirectory,
  getDeviceAuthorizationDefaults,
  getAuthSession,
  getFailedProdRollbackTarget,
  previewTemplateMigration,
  pollDeviceAuthorization,
  resolveWorkspaceRoot,
  scanBuildableTargets,
  scanGenerateConflicts,
  scanTemplateMigrationTargets,
  startDeviceAuthorization,
  validateDeviceAuthorizationSettings,
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

  it('wraps generate conflict scanning with the resolved workspace root', async () => {
    mockInvoke.mockResolvedValue(['/repo/regions/system/server/rust/auth']);

    const conflicts = await scanGenerateConflicts(
      {
        kind: 'Server',
        tier: 'System',
        placement: null,
        lang_fw: { Language: 'Rust' },
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

    expect(conflicts).toEqual(['/repo/regions/system/server/rust/auth']);
    expect(mockInvoke).toHaveBeenCalledWith('scan_generate_conflicts', {
      config: expect.objectContaining({ kind: 'Server', tier: 'System' }),
      baseDir: '/repo',
    });
  });

  it('wraps template migration commands', async () => {
    const target = {
      path: '/repo/regions/service/order/server/rust',
      available_version: '1.5.0',
      manifest: {
        apiVersion: 'k1s0/v1',
        kind: 'TemplateInstance',
        metadata: {
          name: 'order-server',
          generatedAt: '2026-03-12T00:00:00Z',
          generatedBy: 'k1s0-cli@0.1.0',
        },
        spec: {
          template: {
            type: 'server',
            language: 'rust',
            version: '1.2.0',
            checksum: 'sha256:abc',
          },
          parameters: { tier: 'service' },
          customizations: { ignorePaths: [], mergeStrategy: {} },
        },
      },
    };

    mockInvoke
      .mockResolvedValueOnce([target])
      .mockResolvedValueOnce({ target, changes: [] })
      .mockResolvedValueOnce(undefined);

    expect(await scanTemplateMigrationTargets('/repo')).toEqual([target]);
    expect(await previewTemplateMigration(target)).toEqual({ target, changes: [] });
    await executeTemplateMigration({ target, changes: [] });

    expect(mockInvoke).toHaveBeenNthCalledWith(1, 'scan_template_migration_targets', {
      baseDir: '/repo',
    });
    expect(mockInvoke).toHaveBeenNthCalledWith(2, 'preview_template_migration', { target });
    expect(mockInvoke).toHaveBeenNthCalledWith(3, 'execute_template_migration', {
      plan: { target, changes: [] },
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
    mockInvoke.mockResolvedValueOnce('/repo').mockResolvedValueOnce('C:/work').mockResolvedValueOnce('/repo');

    expect(await detectWorkspaceRoot()).toBe('/repo');
    expect(await getCurrentDirectory()).toBe('C:/work');
    expect(await resolveWorkspaceRoot('/repo')).toBe('/repo');

    expect(mockInvoke).toHaveBeenNthCalledWith(1, 'detect_workspace_root');
    expect(mockInvoke).toHaveBeenNthCalledWith(2, 'get_current_directory');
    expect(mockInvoke).toHaveBeenNthCalledWith(3, 'resolve_workspace_root', { path: '/repo' });
  });

  it('wraps execute_init_at with an explicit parent directory', async () => {
    mockInvoke.mockResolvedValue('C:/work/my-project');

    const result = await executeInitAt(
      {
        project_name: 'my-project',
        git_init: true,
        sparse_checkout: false,
        tiers: ['System'],
      },
      'C:/work',
    );

    expect(result).toBe('C:/work/my-project');
    expect(mockInvoke).toHaveBeenCalledWith('execute_init_at', {
      config: expect.objectContaining({ project_name: 'my-project' }),
      baseDir: 'C:/work',
    });
  });

  it('wraps device authorization commands', async () => {
    const settings = {
      discovery_url: 'https://issuer.example.com/.well-known/openid-configuration',
      client_id: 'client',
      scope: 'openid',
    };
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
      .mockResolvedValueOnce(settings)
      .mockResolvedValueOnce({
        issuer: 'https://issuer.example.com',
        token_endpoint: 'https://issuer.example.com/token',
        device_authorization_endpoint: 'https://issuer.example.com/device',
      })
      .mockResolvedValueOnce(challenge)
      .mockResolvedValueOnce({ status: 'Pending', interval: 5, message: 'pending' });

    expect(await getDeviceAuthorizationDefaults()).toEqual(settings);
    expect(await validateDeviceAuthorizationSettings(settings)).toEqual({
      issuer: 'https://issuer.example.com',
      token_endpoint: 'https://issuer.example.com/token',
      device_authorization_endpoint: 'https://issuer.example.com/device',
    });
    expect(await startDeviceAuthorization(settings)).toEqual(challenge);
    expect(await pollDeviceAuthorization(challenge)).toEqual({
      status: 'Pending',
      interval: 5,
      message: 'pending',
    });

    expect(mockInvoke).toHaveBeenNthCalledWith(1, 'get_device_authorization_defaults');
    expect(mockInvoke).toHaveBeenNthCalledWith(2, 'validate_device_authorization_settings', {
      settings,
    });
    expect(mockInvoke).toHaveBeenNthCalledWith(3, 'start_device_authorization', { settings });
  });

  it('propagates rejection from scanBuildableTargets', async () => {
    // ワークスペースが見つからない場合のエラーを呼び出し元に伝播することを確認する
    mockInvoke.mockRejectedValueOnce(new Error('workspace not found'));

    await expect(scanBuildableTargets('/nonexistent')).rejects.toThrow('workspace not found');
    expect(mockInvoke).toHaveBeenCalledWith('scan_buildable_targets', { baseDir: '/nonexistent' });
  });

  it('propagates rejection from executeGenerateAt', async () => {
    // コード生成失敗時のエラーを呼び出し元に伝播することを確認する
    mockInvoke.mockRejectedValueOnce(new Error('template not found'));

    await expect(
      executeGenerateAt(
        {
          kind: 'Server',
          tier: 'System',
          placement: null,
          lang_fw: { Language: 'Rust' },
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
      ),
    ).rejects.toThrow('template not found');
  });

  it('propagates rejection from executeBuildWithProgress', async () => {
    // ビルド失敗時のエラーを呼び出し元に伝播することを確認する
    mockInvoke.mockRejectedValueOnce(new Error('docker not found'));
    const onEvent = vi.fn();

    await expect(
      executeBuildWithProgress({ targets: ['target-1'], mode: 'Production' }, onEvent),
    ).rejects.toThrow('docker not found');
  });

  it('propagates rejection from detectWorkspaceRoot when outside a repo', async () => {
    // リポジトリ外で実行された場合のエラーを呼び出し元に伝播することを確認する
    mockInvoke.mockRejectedValueOnce(new Error('not a git repository'));

    await expect(detectWorkspaceRoot()).rejects.toThrow('not a git repository');
    expect(mockInvoke).toHaveBeenCalledWith('detect_workspace_root');
  });

  it('propagates rejection from auth session commands', async () => {
    mockInvoke.mockRejectedValueOnce(new Error('no session'));

    await expect(getAuthSession()).rejects.toThrow('no session');
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
