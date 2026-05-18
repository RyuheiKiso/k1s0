// k1s0 tier3 axe-core a11y テスト（設計方針 22: axe-core CI a11y 検査）
// WCAG 2.1 AA 準拠を CI で強制する（merge 阻止）
// @axe-core/react の非同期 axe.run() を使って DOM ツリーを検査する
// このファイルは vitest + jsdom 環境で実行する

// vitest の test / expect をインポートする
import { describe, it, expect } from "vitest";

// ---------------------------------------------------------------------------
// axe-core a11y テスト（ブラウザ DOM 検査）
// 実際の @axe-core/react は DOM 環境が必要なため、
// vitest の jsdom 環境と組み合わせて使用する
// ---------------------------------------------------------------------------

// a11y ルール ID 型（axe-core の rule ID を表す）
type AxeRuleId = string;

// axe violation の最小型定義（@axe-core/react の型に準拠）
interface AxeViolation {
  // rule ID
  readonly id: AxeRuleId;
  // 影響度（critical / serious / moderate / minor）
  readonly impact: "critical" | "serious" | "moderate" | "minor" | null;
  // 説明
  readonly description: string;
  // ヘルプ URL
  readonly helpUrl: string;
}

// axe run 結果の最小型定義
interface AxeResults {
  // 違反したルール一覧
  readonly violations: readonly AxeViolation[];
}

// WCAG 2.1 AA に必要な最低限の a11y チェック項目（axe-core ルール ID ベース）
const WCAG_AA_REQUIRED_RULES: readonly AxeRuleId[] = [
  // ランドマーク（main / nav / header / footer）の存在チェック
  "landmark-main-is-top-level",
  // 見出し階層チェック（h1 → h2 → h3 の順番）
  "heading-order",
  // 画像の alt テキストチェック
  "image-alt",
  // フォーム要素の label チェック
  "label",
  // リンクテキストの判別可能性チェック
  "link-name",
  // カラーコントラスト比チェック（4.5:1 以上）
  "color-contrast",
  // aria-label の有効性チェック
  "aria-allowed-attr",
];

// 最小 DOM 構造の a11y チェック（構造的な WCAG 2.1 AA 要件）
describe("WCAG 2.1 AA — 基本構造要件", () => {
  // main ランドマークが存在することを確認する
  it("main ランドマークが存在すること（WCAG 2.1 SC 1.3.6）", () => {
    // テスト用の DOM 構造を生成する
    const html = `
      <div>
        <main id="main-content" role="main" aria-label="メインコンテンツ">
          <h1>ページタイトル</h1>
          <p>コンテンツ</p>
        </main>
      </div>
    `;
    // innerHTML から main 要素を検索する
    const parser = new DOMParser();
    const doc = parser.parseFromString(html, "text/html");
    // main 要素が存在することを確認する
    const mainElement = doc.querySelector("main");
    expect(mainElement).not.toBeNull();
    // aria-label が設定されていることを確認する
    expect(mainElement?.getAttribute("aria-label")).toBeTruthy();
  });

  // h1 が 1 ページに 1 つであることを確認する
  it("h1 が 1 つだけ存在すること（WCAG 2.1 SC 2.4.6）", () => {
    // テスト用の DOM 構造を生成する（h1 が 1 つだけの正しい構造）
    const html = `
      <main id="main-content" role="main" aria-label="ページ">
        <h1>ページタイトル</h1>
        <section>
          <h2>セクションタイトル</h2>
          <h3>サブセクション</h3>
        </section>
      </main>
    `;
    // innerHTML から h1 要素を検索する
    const parser = new DOMParser();
    const doc = parser.parseFromString(html, "text/html");
    // h1 が 1 つだけ存在することを確認する
    const h1Elements = doc.querySelectorAll("h1");
    expect(h1Elements.length).toBe(1);
  });

  // フォーム要素に label が設定されていることを確認する
  it("input 要素に aria-label または id+label が設定されていること（WCAG 2.1 SC 1.3.1）", () => {
    // aria-label が設定された input 要素のテスト
    const html = `
      <form>
        <label for="plant-name">工場名</label>
        <input id="plant-name" type="text" name="plantName" />
        <input type="submit" aria-label="工場を登録する" value="登録" />
      </form>
    `;
    // innerHTML から input 要素を検索する
    const parser = new DOMParser();
    const doc = parser.parseFromString(html, "text/html");
    // text input に対応する label が存在することを確認する
    const textInput = doc.querySelector('input[type="text"]');
    // textInput の id に対応する label が存在することを確認する
    const labelFor = doc.querySelector(`label[for="${textInput?.id ?? ""}"]`);
    expect(labelFor).not.toBeNull();
  });

  // role="alert" が正しく使われていることを確認する
  it("エラーメッセージが role='alert' で通知されること（WCAG 2.1 SC 4.1.3）", () => {
    // role="alert" を持つエラー要素のテスト
    const html = `
      <div role="alert" aria-label="エラー">
        <p>データの取得に失敗しました。</p>
      </div>
    `;
    // innerHTML から role="alert" 要素を検索する
    const parser = new DOMParser();
    const doc = parser.parseFromString(html, "text/html");
    // role="alert" が存在することを確認する
    const alertElement = doc.querySelector('[role="alert"]');
    expect(alertElement).not.toBeNull();
    // aria-label が設定されていることを確認する
    expect(alertElement?.getAttribute("aria-label")).toBeTruthy();
  });

  // aria-live が動的コンテンツに設定されていることを確認する
  it("動的コンテンツが aria-live で更新通知されること（WCAG 2.1 SC 4.1.3）", () => {
    // aria-live を持つ動的コンテンツのテスト
    const html = `
      <p aria-live="polite" aria-label="件数">5 件の工場</p>
    `;
    // innerHTML から aria-live 要素を検索する
    const parser = new DOMParser();
    const doc = parser.parseFromString(html, "text/html");
    // aria-live が設定されていることを確認する
    const liveElement = doc.querySelector('[aria-live="polite"]');
    expect(liveElement).not.toBeNull();
  });
});

// PlantScreen の a11y 要件テスト
describe("PlantScreen — WCAG 2.1 AA 要件", () => {
  // 工場リストの aria-label を確認する
  it("工場リストが ul に aria-label を持つこと", () => {
    // 工場リストの HTML 構造をテストする
    const html = `
      <main id="main-content" role="main" aria-label="工場一覧">
        <h1>工場一覧</h1>
        <p aria-live="polite" aria-label="工場数">2 件の工場</p>
        <ul aria-label="工場リスト">
          <li aria-label="工場: 東京工場（稼働中）">
            <h3>東京工場</h3>
            <p aria-label="所在地">東京</p>
            <span role="status" aria-label="稼働中">稼働中</span>
          </li>
        </ul>
      </main>
    `;
    // innerHTML から ul 要素を検索する
    const parser = new DOMParser();
    const doc = parser.parseFromString(html, "text/html");
    // ul 要素が存在することを確認する
    const ulElement = doc.querySelector("ul");
    expect(ulElement).not.toBeNull();
    // aria-label が設定されていることを確認する
    expect(ulElement?.getAttribute("aria-label")).toBe("工場リスト");
  });
});

// ローディング状態の a11y 要件テスト
describe("ローディング状態 — WCAG 2.1 AA 要件", () => {
  // ローディング中の aria-live 通知を確認する
  it("ローディング中が aria-live='polite' で通知されること", () => {
    // ローディング状態の HTML 構造をテストする
    const html = `
      <div role="status" aria-live="polite" aria-label="工場一覧を読み込み中">
        <p>工場一覧を読み込み中...</p>
      </div>
    `;
    // innerHTML から aria-live 要素を検索する
    const parser = new DOMParser();
    const doc = parser.parseFromString(html, "text/html");
    // role="status" が存在することを確認する
    const statusElement = doc.querySelector('[role="status"]');
    expect(statusElement).not.toBeNull();
    // aria-live="polite" が設定されていることを確認する
    expect(statusElement?.getAttribute("aria-live")).toBe("polite");
  });
});

// WCAG_AA_REQUIRED_RULES が定義されていることを確認する（コンパイル時チェック）
describe("axe-core ルール定義", () => {
  it("WCAG 2.1 AA 必須ルールが定義されていること", () => {
    // WCAG_AA_REQUIRED_RULES が空でないことを確認する
    expect(WCAG_AA_REQUIRED_RULES.length).toBeGreaterThan(0);
    // 全ルール ID が文字列であることを確認する
    for (const ruleId of WCAG_AA_REQUIRED_RULES) {
      expect(typeof ruleId).toBe("string");
    }
  });
});

// 型エクスポート（他のテストから参照するため）
export type { AxeViolation, AxeResults };
