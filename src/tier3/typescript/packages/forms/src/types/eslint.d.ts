// eslint.d.ts — forms パッケージ専用 eslint モジュール型 stub
// @types/eslint が devDependency に追加されるまでの型定義ブリッジ。
// eslint_no_tenant_id.ts カスタムルール実装で必要な Rule.RuleModule 型を宣言する。

// eslint モジュールの最小型定義 (eslint Rule.RuleModule と互換する stub)
declare module "eslint" {
  // Rule 名前空間: カスタムルール実装に必要な型を宣言する
  export namespace Rule {
    // RuleContext: ESLint がルールに提供するコンテキストオブジェクト
    interface RuleContext {
      // report: 問題箇所を報告するメソッド
      report(descriptor: ReportDescriptor): void;
    }

    // ReportDescriptor: report() メソッドに渡すディスクリプター
    interface ReportDescriptor {
      // node: 問題が発生した AST ノード
      node: Node;
      // messageId: messages マップのキー（文字列リテラル）
      messageId: string;
      // data: メッセージテンプレートの補完データ
      data?: Record<string, string>;
    }

    // Node: AST ノードの最小型定義
    interface Node {
      // type: AST ノードの種別 (Identifier / Property 等)
      type: string;
      // [key]: 追加プロパティ（any で受け入れる）
      [key: string]: unknown;
    }

    // RuleMetaData: ルールのメタ情報型定義
    interface RuleMetaData {
      // type: ルール種別 (problem / suggestion / layout)
      type?: "problem" | "suggestion" | "layout";
      // docs: ドキュメント情報
      docs?: {
        description?: string;
        recommended?: boolean;
        url?: string;
      };
      // messages: エラーメッセージテンプレートのマップ
      messages?: Record<string, string>;
      // schema: ルールオプションの JSON Schema 定義
      schema?: unknown[];
    }

    // RuleListener: ESLint ビジターのリスナー型（AST ノード種別 → ハンドラ関数のマップ）
    type RuleListener = Record<string, (node: Node) => void>;

    // RuleModule: ESLint カスタムルールのモジュール定義型
    interface RuleModule {
      // meta: ルールのメタ情報
      meta?: RuleMetaData;
      // create: ルールのファクトリ関数（RuleContext を受け取り RuleListener を返す）
      create(context: RuleContext): RuleListener;
    }
  }
}
