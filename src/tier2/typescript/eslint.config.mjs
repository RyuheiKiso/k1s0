// tier2 TypeScript ESLint 設定: import boundaries と業界中立性を enforce する
// eslint-plugin-import-x と eslint-plugin-boundaries を使用する
import boundaries from 'eslint-plugin-boundaries';
// import-x プラグインをインポートする
import importPlugin from 'eslint-plugin-import-x';

// ESLint フラット設定オブジェクトの export
export default [
  {
    // tier2 TypeScript ソースファイルを対象とする
    files: ['**/*.ts', '**/*.tsx'],
    plugins: {
      // boundaries プラグインの登録
      boundaries,
      // import プラグインの登録
      'import-x': importPlugin,
    },
    rules: {
      // tier3 から tier1 直接 import を禁止する
      'boundaries/element-types': ['error', {
        // デフォルト: disallow（明示許可なき import を全て禁止）
        default: 'disallow',
        rules: [
          {
            // tier2 から tier1 への import は facade 経由のみ許可する
            from: 'tier2',
            allow: ['tier2-internal', 'tier1-facade'],
          }
        ]
      }],
      // 業界固有語を含む import を禁止する
      'no-restricted-imports': ['error', {
        patterns: [
          // 製造業パック直接 import 禁止 (業界中立性規則)
          '**/pack/manufacturing/**',
        ]
      }],
    },
  },
];
