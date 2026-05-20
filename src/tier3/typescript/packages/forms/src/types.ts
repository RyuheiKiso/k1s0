// tier3 forms package の型定義
// tenant_id を含まない業務フォームフィールドの型

// FormFieldBase: 1 フォームフィールドの型定義
export type FormFieldBase = {
  // フィールド名 (tenant_id / tenantId は含まない)
  name: string;
  // フィールドの値型
  type: 'string' | 'number' | 'boolean' | 'date';
  // 必須フラグ
  required: boolean;
  // フィールドの表示ラベル
  label?: string;
};

// TenantIdExcluded: F から tenant_id / tenantId キーを再帰的に除外するマッピング型
// tier3/CLAUDE.md: 公開 type / form schema に tenant_id フィールド禁止
export type TenantIdExcluded<F> = {
  // tenant_id および tenantId を含むキーを型レベルで除外する
  [K in keyof F as K extends 'tenant_id' | 'tenantId' ? never : K]: F[K] extends Record<string, unknown>
    // ネストされたオブジェクト型を再帰的に処理する
    ? TenantIdExcluded<F[K]>
    // プリミティブ型はそのまま使用する
    : F[K];
};

// FormSection: フォームをセクション単位で分割する型
// F: セクション内のフォームフィールドの型（tenant_id / tenantId は再帰的に除外される）
export type FormSection<F> = {
  // セクション識別子（画面表示の section_id として使用する）
  readonly sectionId: string;
  // セクション表示ラベル（i18n キー）
  readonly labelKey: string;
  // セクション内のフォームフィールド（tenant_id を再帰的に除外する）
  readonly fields: TenantIdExcluded<F>;
  // セクションが折りたたみ可能かどうか（省略時は false）
  readonly collapsible?: boolean;
};

// FormStep: マルチステップフォームの 1 ステップを表す型
// F: ステップ内のフォームフィールドの型（tenant_id / tenantId は再帰的に除外される）
export type FormStep<F> = {
  // ステップ番号（1 始まり）
  readonly stepIndex: number;
  // ステップ識別子（URL パラメータや状態管理のキーとして使用する）
  readonly stepId: string;
  // ステップ表示ラベル（i18n キー）
  readonly labelKey: string;
  // ステップ内のセクション一覧（各セクションが tenant_id を再帰的に除外する）
  readonly sections: readonly FormSection<F>[];
  // このステップが完了済みかどうか（wizard の進行管理に使用する）
  readonly completed: boolean;
};

// FormWizard: マルチステップウィザード形式のフォーム全体を表す型
// F: ウィザード全体のフォームフィールドの型（tenant_id / tenantId は再帰的に除外される）
export type FormWizard<F> = {
  // ウィザード識別子（フォーム種別を識別するキー）
  readonly wizardId: string;
  // ウィザード表示ラベル（i18n キー）
  readonly titleKey: string;
  // ウィザードのステップ一覧（順序通りに並べる）
  readonly steps: readonly FormStep<F>[];
  // 現在アクティブなステップのインデックス（1 始まり）
  readonly currentStepIndex: number;
  // ウィザードの送信状態（submitting 中は true）
  readonly submitting: boolean;
};
