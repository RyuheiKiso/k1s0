// ============================================================
// textlint 規約（日本語文体・禁止表現）
// ============================================================
// - docs/00_format/style_guide.md の禁止表現を機械的に enforce する。
// - 実装は Phase 5 (tools/docs_lint/textlint runner)。
// - status: locked のドキュメントで違反 1 件あれば CI fail。
// - status: draft では warn 経路（ship 前に解消必須）。
// ============================================================

export default {
  rules: {

    // ----------------------------------------------------------
    // 禁止表現: status: locked で CI fail (style_guide.md 参照)
    // ----------------------------------------------------------
    "prh": {
      rulePaths: ["./prh-rules/forbidden_locked.yaml"],
      // forbidden_locked.yaml の中身（規約宣言）:
      //   version: 1
      //   rules:
      //     - expected: ""  # 削除して具体内容を書く
      //       patterns:
      //         - /TBD/
      //         - /TODO/
      //         - /未定/
      //         - /あとで書く/
      //         - /後述/
      //         - /将来検討/
      //         - /たぶん/
      //         - /おそらく/
      //         - /〜したほうが良い/
      //         - /〜が望ましい/
      //         - /〜できる場合がある/
    },

    // ----------------------------------------------------------
    // 禁止表現: status: draft でも warn (主観表現)
    // ----------------------------------------------------------
    "@textlint-ja/no-dropping-i": false,

    "@textlint-rule/no-unmatched-pair": true,

    "ja-technical-writing/no-mix-dearu-desumasu": {
      preferInHeader: "である",
      preferInBody: "である",
      preferInList: "である",
      strict: true,
    },

    "ja-technical-writing/sentence-length": {
      max: 200,
      // 一文が長すぎる場合は段落分割を推奨
    },

    "ja-technical-writing/max-comma": {
      max: 4,
    },

    "ja-technical-writing/no-doubled-conjunction": true,

    "ja-technical-writing/no-doubled-conjunctive-particle-ga": true,

    "ja-technical-writing/no-doubled-joshi": {
      min_interval: 1,
      strict: false,
    },

    "ja-technical-writing/no-double-negative-ja": true,

    "ja-technical-writing/no-exclamation-question-mark": {
      allowFullWidthExclamation: false,
      allowFullWidthQuestion: false,
      allowHalfWidthExclamation: false,
      allowHalfWidthQuestion: false,
    },

    "ja-technical-writing/no-hankaku-kana": true,

    "ja-technical-writing/ja-no-mixed-period": {
      periodMark: "。",
      allowPeriodMarks: [],
      forceAppendPeriod: false,
    },

    "ja-technical-writing/no-invalid-control-character": true,

    "ja-technical-writing/no-zero-width-spaces": true,

    "ja-spacing/ja-space-around-code": {
      before: true,
      after: true,
    },

    "ja-spacing/ja-space-between-half-and-full-width": {
      space: "always",
      exceptPunctuation: true,
    },

    "ja-spacing/ja-no-space-around-parentheses": true,

    "ja-spacing/ja-no-space-between-full-width": true,

    // ----------------------------------------------------------
    // 構造規約 (style_guide.md / section_marker.md と整合)
    // ----------------------------------------------------------
    "ja-technical-writing/ja-no-redundant-expression": true,

    "ja-technical-writing/no-unmatched-pair": true,
  },

  // ----------------------------------------------------------
  // 対象ファイル
  // ----------------------------------------------------------
  files: [
    "docs/01_企画/**/*.md",
    "docs/02_要件定義/**/*.md",
    "docs/03_概要設計/**/*.md",
    "docs/04_詳細設計/**/*.md",
    "docs/00_format/**/*.md",
  ],

  // ----------------------------------------------------------
  // 除外（知識層は textlint 対象外）
  // ----------------------------------------------------------
  ignore: [
    "docs/90_archive/**",
    "docs/00_format/templates/*.tpl",
  ],
};
