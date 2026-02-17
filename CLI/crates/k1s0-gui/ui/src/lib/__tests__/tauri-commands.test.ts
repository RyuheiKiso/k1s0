import { describe, it, expect, beforeEach, vi } from 'vitest';
import { mockInvoke } from '../../test/mocks';
import { getConfig, executeInit, validateName, scanBuildableTargets, scanDeployableTargets, scanPlacements, executeBuild, executeTest, executeDeploy, executeGenerate, executeTestWithProgress, executeBuildWithProgress, executeDeployWithProgress } from '../tauri-commands';

beforeEach(() => {
  mockInvoke.mockReset();
});

describe('tauri-commands', () => {
  describe('getConfig', () => {
    it('should invoke get_config with correct args', async () => {
      mockInvoke.mockResolvedValue({ docker_registry: 'test', go_module_base: 'test' });
      const result = await getConfig('/path/to/config.yaml');
      expect(mockInvoke).toHaveBeenCalledWith('get_config', { configPath: '/path/to/config.yaml' });
      expect(result.docker_registry).toBe('test');
    });
  });

  describe('executeInit', () => {
    it('should invoke execute_init with correct args', async () => {
      mockInvoke.mockResolvedValue(undefined);
      await executeInit({
        project_name: 'test-project',
        git_init: true,
        sparse_checkout: false,
        tiers: ['System', 'Business', 'Service'],
      });
      expect(mockInvoke).toHaveBeenCalledWith('execute_init', {
        config: {
          project_name: 'test-project',
          git_init: true,
          sparse_checkout: false,
          tiers: ['System', 'Business', 'Service'],
        },
      });
    });
  });

  describe('validateName', () => {
    it('should invoke validate_name with correct args', async () => {
      mockInvoke.mockResolvedValue(undefined);
      await validateName('my-service');
      expect(mockInvoke).toHaveBeenCalledWith('validate_name', { name: 'my-service' });
    });

    it('should propagate errors', async () => {
      mockInvoke.mockRejectedValue('Invalid name');
      await expect(validateName('-invalid')).rejects.toBe('Invalid name');
    });
  });

  describe('scanPlacements', () => {
    it('should invoke scan_placements with correct args', async () => {
      mockInvoke.mockResolvedValue(['accounting', 'fa']);
      const result = await scanPlacements('Business', '.');
      expect(mockInvoke).toHaveBeenCalledWith('scan_placements', { tier: 'Business', baseDir: '.' });
      expect(result).toEqual(['accounting', 'fa']);
    });
  });

  describe('scanBuildableTargets', () => {
    it('should invoke scan_buildable_targets', async () => {
      mockInvoke.mockResolvedValue(['regions/system/server/go/auth']);
      const result = await scanBuildableTargets('.');
      expect(mockInvoke).toHaveBeenCalledWith('scan_buildable_targets', { baseDir: '.' });
      expect(result).toEqual(['regions/system/server/go/auth']);
    });
  });

  describe('scanDeployableTargets', () => {
    it('should invoke scan_deployable_targets', async () => {
      mockInvoke.mockResolvedValue([]);
      const result = await scanDeployableTargets('.');
      expect(mockInvoke).toHaveBeenCalledWith('scan_deployable_targets', { baseDir: '.' });
      expect(result).toEqual([]);
    });
  });

  describe('executeBuild', () => {
    it('should invoke execute_build', async () => {
      mockInvoke.mockResolvedValue(undefined);
      await executeBuild({ targets: ['target1'], mode: 'Development' });
      expect(mockInvoke).toHaveBeenCalledWith('execute_build', {
        config: { targets: ['target1'], mode: 'Development' },
      });
    });
  });

  describe('executeTest', () => {
    it('should invoke execute_test', async () => {
      mockInvoke.mockResolvedValue(undefined);
      await executeTest({ kind: 'Unit', targets: ['target1'] });
      expect(mockInvoke).toHaveBeenCalledWith('execute_test', {
        config: { kind: 'Unit', targets: ['target1'] },
      });
    });
  });

  describe('executeDeploy', () => {
    it('should invoke execute_deploy', async () => {
      mockInvoke.mockResolvedValue(undefined);
      await executeDeploy({ environment: 'Dev', targets: ['target1'] });
      expect(mockInvoke).toHaveBeenCalledWith('execute_deploy', {
        config: { environment: 'Dev', targets: ['target1'] },
      });
    });
  });

  describe('executeGenerate', () => {
    it('should invoke execute_generate', async () => {
      mockInvoke.mockResolvedValue(undefined);
      await executeGenerate({
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
      });
      expect(mockInvoke).toHaveBeenCalledWith('execute_generate', {
        config: expect.objectContaining({ kind: 'Server', tier: 'System' }),
      });
    });
  });

  describe('executeTestWithProgress', () => {
    it('should invoke execute_test_with_progress with channel', async () => {
      mockInvoke.mockResolvedValue(undefined);
      const onEvent = vi.fn();
      await executeTestWithProgress({ kind: 'Unit', targets: ['t1'] }, onEvent);
      expect(mockInvoke).toHaveBeenCalledWith('execute_test_with_progress', {
        config: { kind: 'Unit', targets: ['t1'] },
        onEvent: expect.any(Object),
      });
    });
  });

  describe('executeBuildWithProgress', () => {
    it('should invoke execute_build_with_progress with channel', async () => {
      mockInvoke.mockResolvedValue(undefined);
      const onEvent = vi.fn();
      await executeBuildWithProgress({ targets: ['t1'], mode: 'Development' }, onEvent);
      expect(mockInvoke).toHaveBeenCalledWith('execute_build_with_progress', {
        config: { targets: ['t1'], mode: 'Development' },
        onEvent: expect.any(Object),
      });
    });
  });

  describe('executeDeployWithProgress', () => {
    it('should invoke execute_deploy_with_progress with channel', async () => {
      mockInvoke.mockResolvedValue(undefined);
      const onEvent = vi.fn();
      await executeDeployWithProgress({ environment: 'Dev', targets: ['t1'] }, onEvent);
      expect(mockInvoke).toHaveBeenCalledWith('execute_deploy_with_progress', {
        config: { environment: 'Dev', targets: ['t1'] },
        onEvent: expect.any(Object),
      });
    });
  });
});
