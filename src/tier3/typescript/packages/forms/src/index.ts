// src/tier3/typescript/packages/forms/src/index.ts
// tier3 forms パッケージの公開 API
// ESLint カスタムルールとフォーム共通型を公開する

// eslint_no_tenant_id ルールを公開する
export { noTenantIdInFormSchema } from "./eslint_no_tenant_id.js";

// フォームフィールドの基底型を定義する（tenant_id フィールドを含まない）
export type FormField<T> = {
  // フィールドの値
  readonly value: T;
  // バリデーションエラーメッセージ（なければ null）
  readonly error: string | null;
  // フィールドが変更されたかどうか
  readonly touched: boolean;
};

// フォームの基底型を定義する（tenant_id フィールドを明示的に除外する）
export type FormSchema<T extends Record<string, unknown>> = {
  // tenant_id および tenantId を含むキーを型レベルで禁止する
  readonly [K in keyof T as K extends "tenant_id" | "tenantId"
    ? never
    : K]: FormField<T[K]>;
};
