// =============================================================================
// commitlint.config.js — k1s0 のコミットメッセージ規約（Conventional Commits）
//
// 設計: plan/02_開発環境整備/12_コミットメッセージ規約.md
//       docs/05_実装/30_CI_CD設計/50_branch_protection/01_branch_protection.md（squash merge default）
// 関連 ID: IMP-CI-008（pre-commit / CI 二重防御）
//
// 適用:
//   - ローカル: pre-commit の commit-msg stage で `commitlint --edit` を実行
//   - CI: .github/workflows/commitlint.yml が PR title と全 commit を検証
//
// type / scope の正規一覧は本ファイルの `rules` を正典とし、
// .github/labels.yml の type/* / scope/* と整合させる。
// =============================================================================

module.exports = {
  // Conventional Commits v1.0.0 を継承
  extends: ['@commitlint/config-conventional'],

  rules: {
    // ----- type 軸（.github/labels.yml の type/* と完全一致） -----
    'type-enum': [
      2,
      'always',
      [
        'feat',     // 新機能
        'fix',      // バグ修正
        'docs',     // ドキュメントのみ
        'style',    // フォーマット（コード変更なし）
        'refactor', // リファクタリング
        'perf',     // 性能改善
        'test',     // テスト追加・修正
        'build',    // ビルドシステム / 依存
        'ci',       // CI 設定
        'chore',    // その他（依存更新・雑務）
        'revert',   // revert
      ],
    ],
    'type-case': [2, 'always', 'lower-case'],
    'type-empty': [2, 'never'],

    // ----- scope 軸（.github/labels.yml の scope/* と完全一致） -----
    'scope-enum': [
      2,
      'always',
      [
        // 契約 / SDK
        'contracts',
        'sdk-dotnet',
        'sdk-go',
        'sdk-rust',
        'sdk-typescript',
        // tier1
        'tier1-go',
        'tier1-rust',
        // tier2
        'tier2',
        // tier3
        'tier3-web',
        'tier3-native',
        'tier3-bff',
        'tier3-legacy',
        // platform / infra / deploy / ops / tools / docs / tests
        'platform',
        'infra',
        'deploy',
        'ops',
        'tools',
        'docs',
        'tests',
        // セキュリティ系（横断）
        'security',
        // 依存（Renovate 等）
        'deps',
        // リリース pipeline
        'release',
      ],
    ],
    'scope-case': [2, 'always', 'lower-case'],
    'scope-empty': [1, 'never'], // scope は warning（強制しないが推奨）

    // ----- subject -----
    'subject-case': [2, 'always', ['lower-case', 'sentence-case']],
    'subject-empty': [2, 'never'],
    'subject-full-stop': [2, 'never', '.'],
    'subject-max-length': [2, 'always', 72],

    // ----- body / footer -----
    'body-leading-blank': [2, 'always'],
    'body-max-line-length': [1, 'always', 100], // 日本語文の自然な長さを許容（warning）
    'footer-leading-blank': [2, 'always'],
    'footer-max-line-length': [2, 'always', 100],

    // ----- header -----
    'header-max-length': [2, 'always', 100], // 日本語混在で 72 字制限はやや厳しいため 100 に緩和
  },

  // breaking change footer の判定: `BREAKING CHANGE:` を含むか header に `!` を含むか
  parserPreset: {
    parserOpts: {
      noteKeywords: ['BREAKING CHANGE', 'BREAKING-CHANGE'],
    },
  },

  // ヘルプメッセージ
  helpUrl: 'https://github.com/RyuheiKiso/k1s0/blob/main/CONTRIBUTING.md#commit-message-規約',
};
