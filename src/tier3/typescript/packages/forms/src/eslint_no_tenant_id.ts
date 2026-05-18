// src/tier3/typescript/packages/forms/src/eslint_no_tenant_id.ts
// tier3 form の tenant_id フィールド禁止 ESLint カスタムルール
// tier3 CLAUDE.md: 公開 type / form schema / URL routing / クエリパラメータに tenant_id フィールド禁止

// ESLint Rule モジュール型をインポートする
import type { Rule } from "eslint";

// tenant_id を含む禁止プロパティ名パターン
const FORBIDDEN_PATTERNS: readonly RegExp[] = [
  // snake_case の tenant_id を禁止する
  /^tenant_id$/,
  // camelCase の tenantId を禁止する
  /^tenantId$/,
];

// no-tenant-id-in-form-schema: form schema に tenant_id フィールドを禁止するカスタムルール
export const noTenantIdInFormSchema: Rule.RuleModule = {
  // ルールのメタ情報を定義する
  meta: {
    // ルール種別: 問題検出
    type: "problem",
    // ドキュメント情報
    docs: {
      // ルールの説明
      description: "form schema / URL routing / query params に tenant_id フィールドを禁止する",
      // 推奨設定に含めるか
      recommended: true,
    },
    // エラーメッセージを定義する
    messages: {
      // tenant_id 検出時のメッセージ
      noTenantId:
        "form schema / public type に '{{ fieldName }}' を含めることは禁止されています（tier3 CLAUDE.md §データ保護）。tenant は BFF session context から自動注入してください。",
    },
    // スキーマ（追加オプションなし）
    schema: [],
  },

  // AST ノードをチェックするビジター関数を返す
  create(context: Rule.RuleContext): Rule.RuleListener {
    // Property ノード（オブジェクトプロパティ）を検査する
    return {
      // TypeScript / JavaScript のオブジェクトプロパティを検査する
      Property(node: Rule.Node): void {
        // Property ノードにキャストする
        const prop = node as Rule.Node & {
          key: { type: string; name?: string; value?: string };
        };
        // プロパティキーが識別子または文字列リテラルの場合のみ処理する
        const keyNode = prop.key;
        // プロパティ名を取得する
        const fieldName =
          keyNode.type === "Identifier"
            ? keyNode.name
            : keyNode.type === "Literal"
              ? String(keyNode.value)
              : undefined;
        // フィールド名が取得できなかった場合はスキップする
        if (!fieldName) return;
        // 禁止パターンにマッチするかチェックする
        const isForbidden = FORBIDDEN_PATTERNS.some((pattern) =>
          pattern.test(fieldName),
        );
        // 禁止パターンにマッチした場合はエラーを報告する
        if (isForbidden) {
          // エラーを ESLint に報告する
          context.report({
            // エラーが発生したノード
            node,
            // 使用するメッセージ ID
            messageId: "noTenantId",
            // メッセージに挿入するデータ
            data: { fieldName },
          });
        }
      },
    };
  },
};
