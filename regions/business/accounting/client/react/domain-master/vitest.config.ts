import { defineConfig } from 'vitest/config';
import react from '@vitejs/plugin-react';

// Vitest テスト設定: jsdom環境でReactコンポーネントをテスト
export default defineConfig({
  plugins: [react()],
  test: {
    environment: 'jsdom',
    setupFiles: ['./tests/testutil/setup.ts'],
    globals: true,
  },
});
