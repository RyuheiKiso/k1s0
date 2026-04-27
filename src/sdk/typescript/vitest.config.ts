// Vitest 設定。tsconfig の strict モードを継承し、src/__tests__/ 配下のテストを実行する。
import { defineConfig } from "vitest/config";

export default defineConfig({
  test: {
    include: ["src/**/*.test.ts"],
    globals: false,
    coverage: {
      provider: "v8",
      reporter: ["text", "json", "html"],
      include: ["src/**/*.ts"],
      exclude: ["src/proto/**", "src/**/*.test.ts"],
    },
  },
});
