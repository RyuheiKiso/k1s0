import { defineConfig } from 'vitest/config';

export default defineConfig({
  test: {
    globals: true,
    // dist/にコンパイル済みのテストファイルが存在するため、src/のみを対象にする
    include: ['src/**/*.test.ts'],
  },
});
