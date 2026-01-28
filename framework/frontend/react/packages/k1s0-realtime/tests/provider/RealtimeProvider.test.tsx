/**
 * RealtimeProvider のテスト
 */

import { describe, expect, it } from 'vitest';
import { render, screen } from '@testing-library/react';
import { useContext } from 'react';
import { RealtimeProvider } from '../../src/provider/RealtimeProvider.js';
import { RealtimeContext } from '../../src/provider/RealtimeContext.js';

function TestConsumer() {
  const { isOnline, connections } = useContext(RealtimeContext);
  return (
    <div>
      <span data-testid="online">{String(isOnline)}</span>
      <span data-testid="connections">{connections.size}</span>
    </div>
  );
}

describe('RealtimeProvider', () => {
  it('子コンポーネントをレンダリングする', () => {
    render(
      <RealtimeProvider>
        <div data-testid="child">child</div>
      </RealtimeProvider>,
    );

    expect(screen.getByTestId('child')).toBeInTheDocument();
  });

  it('デフォルトで isOnline: true を提供する', () => {
    render(
      <RealtimeProvider>
        <TestConsumer />
      </RealtimeProvider>,
    );

    expect(screen.getByTestId('online').textContent).toBe('true');
  });

  it('デフォルトで空の connections を提供する', () => {
    render(
      <RealtimeProvider>
        <TestConsumer />
      </RealtimeProvider>,
    );

    expect(screen.getByTestId('connections').textContent).toBe('0');
  });
});
