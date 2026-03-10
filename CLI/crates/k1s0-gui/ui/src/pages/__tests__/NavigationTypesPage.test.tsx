import { beforeEach, describe, expect, it } from 'vitest';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { mockInvoke } from '../../test/mocks';
import NavigationTypesPage from '../NavigationTypesPage';

beforeEach(() => {
  mockInvoke.mockReset();
  mockInvoke.mockImplementation((command: string, args?: { target?: string }) => {
    if (command === 'execute_generate_navigation_types') {
      return Promise.resolve(`${args?.target} preview`);
    }
    if (command === 'write_navigation_types') {
      return Promise.resolve([
        { path: 'src/navigation/__generated__/route-types.ts', preview: 'typescript preview' },
        { path: 'src/navigation/__generated__/route_ids.dart', preview: 'dart preview' },
      ]);
    }
    return Promise.resolve(undefined);
  });
});

describe('NavigationTypesPage', () => {
  it('writes both navigation targets', async () => {
    const user = userEvent.setup();
    render(<NavigationTypesPage />);

    await user.click(screen.getByTestId('btn-generate'));

    expect(await screen.findByText('typescript preview')).toBeInTheDocument();
    expect(screen.getByText('dart preview')).toBeInTheDocument();
    expect(mockInvoke).toHaveBeenCalledWith('write_navigation_types', {
      navPath: 'config/navigation.yaml',
      outputDir: 'src/navigation/__generated__',
      targets: ['typescript', 'dart'],
      baseDir: '.',
    });
  });
});
