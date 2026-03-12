import { beforeEach, describe, expect, it } from 'vitest';
import { screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { mockInvoke } from '../../test/mocks';
import { renderWithProviders } from '../../test/render';
import TemplateMigratePage from '../TemplateMigratePage';

const target = {
  path: '/repo/regions/service/order/server/rust',
  available_version: '1.5.0',
  manifest: {
    apiVersion: 'k1s0/v1',
    kind: 'TemplateInstance',
    metadata: {
      name: 'order-server',
      generatedAt: '2026-03-10T00:00:00Z',
      generatedBy: 'k1s0-cli@0.1.0',
    },
    spec: {
      template: {
        type: 'server',
        language: 'rust',
        version: '1.2.0',
        checksum: 'sha256:abc',
      },
      parameters: {
        tier: 'service',
        placement: 'order',
        serviceName: 'order',
        moduleName: 'order',
        apiStyles: ['rest'],
      },
      customizations: {
        ignorePaths: ['src/domain/**'],
        mergeStrategy: {},
      },
    },
  },
};

beforeEach(() => {
  mockInvoke.mockReset();
  mockInvoke.mockImplementation((command: string) => {
    if (command === 'scan_template_migration_targets') {
      return Promise.resolve([target]);
    }
    if (command === 'list_template_migration_backups') {
      return Promise.resolve(['20260312_010101']);
    }
    if (command === 'preview_template_migration') {
      return Promise.resolve({
        target,
        changes: [
          {
            path: 'Cargo.toml',
            change_type: 'Modified',
            merge_strategy: 'ask',
            merge_result: {
              Conflict: [
                {
                  base: '[package]\nname = "order"\n',
                  ours: '[package]\nname = "order-user"\n',
                  theirs: '[package]\nname = "order-template"\n',
                },
              ],
            },
          },
        ],
      });
    }
    return Promise.resolve(undefined);
  });
});

describe('TemplateMigratePage', () => {
  it('resolves conflicts before executing the migration plan', async () => {
    const user = userEvent.setup();
    renderWithProviders(<TemplateMigratePage />, {
      workspace: { workspaceRoot: '/repo' },
    });

    await waitFor(() =>
      expect(screen.getByTestId('select-template-target')).toBeInTheDocument(),
    );

    await user.click(screen.getByTestId('btn-template-preview'));

    expect(await screen.findByText(/Conflict resolution/)).toBeInTheDocument();
    expect(screen.getByTestId('btn-template-apply')).toBeDisabled();

    await user.click(screen.getByText('Use template'));
    expect(screen.getByTestId('btn-template-apply')).not.toBeDisabled();

    await user.click(screen.getByTestId('btn-template-apply'));

    expect(mockInvoke).toHaveBeenCalledWith(
      'execute_template_migration',
      expect.objectContaining({
        plan: expect.objectContaining({
          changes: [
            expect.objectContaining({
              merge_result: { Clean: '[package]\nname = "order-template"\n' },
            }),
          ],
        }),
      }),
    );
  });

  it('rolls back the selected backup', async () => {
    const user = userEvent.setup();
    renderWithProviders(<TemplateMigratePage />, {
      workspace: { workspaceRoot: '/repo' },
    });

    await waitFor(() =>
      expect(screen.getByTestId('select-template-backup')).toBeInTheDocument(),
    );

    await user.click(screen.getByTestId('btn-template-rollback'));

    expect(mockInvoke).toHaveBeenCalledWith('execute_template_migration_rollback', {
      projectDir: '/repo/regions/service/order/server/rust',
      backupId: '20260312_010101',
    });
  });

  it('clears the stale plan and refreshes the target after a successful apply', async () => {
    const refreshedTarget = {
      ...target,
      manifest: {
        ...target.manifest,
        spec: {
          ...target.manifest.spec,
          template: {
            ...target.manifest.spec.template,
            version: '1.5.0',
            checksum: 'sha256:def',
          },
        },
      },
    };

    let scanCount = 0;
    mockInvoke.mockImplementation((command: string) => {
      if (command === 'scan_template_migration_targets') {
        scanCount += 1;
        return Promise.resolve(scanCount === 1 ? [target] : [refreshedTarget]);
      }
      if (command === 'list_template_migration_backups') {
        return Promise.resolve(
          scanCount >= 2 ? ['20260312_020202', '20260312_010101'] : ['20260312_010101'],
        );
      }
      if (command === 'preview_template_migration') {
        return Promise.resolve({
          target,
          changes: [
            {
              path: 'Cargo.toml',
              change_type: 'Modified',
              merge_strategy: 'template',
              merge_result: { Clean: '[package]\nname = "order-template"\n' },
            },
          ],
        });
      }
      return Promise.resolve(undefined);
    });

    const user = userEvent.setup();
    renderWithProviders(<TemplateMigratePage />, {
      workspace: { workspaceRoot: '/repo' },
    });

    await waitFor(() =>
      expect(screen.getByTestId('select-template-target')).toBeInTheDocument(),
    );

    await user.click(screen.getByTestId('btn-template-preview'));
    expect(await screen.findByTestId('btn-template-apply')).toBeInTheDocument();

    await user.click(screen.getByTestId('btn-template-apply'));

    await waitFor(() =>
      expect(screen.getByTestId('template-success-message')).toHaveTextContent(
        'Template migration completed successfully.',
      ),
    );
    await waitFor(() => expect(screen.queryByTestId('btn-template-apply')).not.toBeInTheDocument());
    await waitFor(() => expect(screen.getByText('Version: v1.5.0')).toBeInTheDocument());

    const scanCalls = mockInvoke.mock.calls.filter(([command]) => command === 'scan_template_migration_targets');
    expect(scanCalls).toHaveLength(2);
  });
});
