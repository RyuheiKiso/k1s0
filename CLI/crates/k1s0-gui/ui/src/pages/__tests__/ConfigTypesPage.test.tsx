import { beforeEach, describe, expect, it } from 'vitest';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { mockInvoke } from '../../test/mocks';
import ConfigTypesPage from '../ConfigTypesPage';

beforeEach(() => {
  mockInvoke.mockReset();
  mockInvoke.mockImplementation((command: string, args?: { target?: string }) => {
    if (command === 'execute_generate_config_types') {
      return Promise.resolve(`${args?.target} preview`);
    }
    if (command === 'write_config_types') {
      return Promise.resolve([
        { path: 'src/config/__generated__/config-types.ts', preview: 'typescript preview' },
        { path: 'src/config/__generated__/config_types.dart', preview: 'dart preview' },
      ]);
    }
    return Promise.resolve(undefined);
  });
});

describe('ConfigTypesPage', () => {
  it('previews both targets from the workspace root', async () => {
    const user = userEvent.setup();
    render(<ConfigTypesPage />);

    await user.click(screen.getByTestId('btn-preview'));

    expect(await screen.findByText('typescript preview')).toBeInTheDocument();
    expect(screen.getByText('dart preview')).toBeInTheDocument();
    expect(mockInvoke).toHaveBeenCalledWith('execute_generate_config_types', {
      schemaPath: 'config/config-schema.yaml',
      target: 'typescript',
      baseDir: '.',
    });
    expect(mockInvoke).toHaveBeenCalledWith('execute_generate_config_types', {
      schemaPath: 'config/config-schema.yaml',
      target: 'dart',
      baseDir: '.',
    });
  });

  it('writes generated files', async () => {
    const user = userEvent.setup();
    render(<ConfigTypesPage />);

    await user.click(screen.getByTestId('btn-generate'));

    expect(await screen.findByText('typescript preview')).toBeInTheDocument();
    expect(screen.getByText('dart preview')).toBeInTheDocument();
    expect(mockInvoke).toHaveBeenCalledWith('write_config_types', {
      schemaPath: 'config/config-schema.yaml',
      outputDir: 'src/config/__generated__',
      targets: ['typescript', 'dart'],
      baseDir: '.',
    });
  });
});
