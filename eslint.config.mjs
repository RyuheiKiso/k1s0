// ESLint flat config（ルート設定）
import eslint from "@eslint/js";
import tseslint from "typescript-eslint";
import react from "eslint-plugin-react";
import reactHooks from "eslint-plugin-react-hooks";
import importPlugin from "eslint-plugin-import";

export default tseslint.config(
  eslint.configs.recommended,
  ...tseslint.configs.strictTypeChecked,
  {
    plugins: { react, "react-hooks": reactHooks, import: importPlugin },
    // TypeScript型チェック対応のパーサーオプション
    languageOptions: {
      parserOptions: {
        // tsconfig.json を自動検出して型情報を利用する
        project: true,
        tsconfigRootDir: import.meta.dirname,
      },
    },
    rules: {
      "react-hooks/rules-of-hooks": "error",
      // L-18 監査対応: exhaustive-deps を warn から error に厳格化する。
      // フックの依存配列の不備はバグの温床となるため、ビルドエラーとして検出する。
      "react-hooks/exhaustive-deps": "error",
      "import/order": ["error", {
        "groups": ["builtin", "external", "internal", "parent", "sibling"],
        "newlines-between": "always",
        "alphabetize": { "order": "asc" }
      }],
      "@typescript-eslint/no-unused-vars": ["error", { "argsIgnorePattern": "^_" }],
      "@typescript-eslint/explicit-function-return-type": "off",
      "@typescript-eslint/no-floating-promises": "error",
    },
  }
);
