/**
 * テストセットアップファイル
 */

import { vi } from 'vitest';
import '@testing-library/jest-dom';

// React Router DOM のモック
vi.mock('react-router-dom', async () => {
  const actual = await vi.importActual('react-router-dom');
  return {
    ...actual,
    useNavigate: vi.fn(() => vi.fn()),
    useLocation: vi.fn(() => ({ pathname: '/' })),
  };
});
