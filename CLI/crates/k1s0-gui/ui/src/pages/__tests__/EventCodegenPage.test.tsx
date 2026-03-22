import { beforeEach, describe, expect, it } from 'vitest';
import { screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { mockInvoke } from '../../test/mocks';
import { renderWithProviders } from '../../test/render';
import EventCodegenPage from '../EventCodegenPage';

beforeEach(() => {
  mockInvoke.mockReset();
  mockInvoke.mockImplementation((command: string) => {
    if (command === 'preview_event_codegen') {
      return Promise.resolve('  Event count: 1');
    }
    if (command === 'execute_event_codegen') {
      return Promise.resolve(['proto/taskmanagement/events/v1/task_created.proto']);
    }
    return Promise.resolve(undefined);
  });
});

describe('EventCodegenPage', () => {
  it('previews and generates event assets', async () => {
    const user = userEvent.setup();
    renderWithProviders(<EventCodegenPage />);

    await user.click(screen.getByTestId('btn-preview-event'));
    expect(await screen.findByText(/Event count: 1/)).toBeInTheDocument();

    await user.click(screen.getByTestId('btn-generate-event'));
    expect(await screen.findByText('proto/taskmanagement/events/v1/task_created.proto')).toBeInTheDocument();
  });
});
