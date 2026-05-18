// k1s0 tier3 pseudolocalization テストファイル
// pseudolocalize / pseudolocalizeDict / isExpanded の動作を検証する
// vitest を使用したユニットテスト

// vitest の test / expect をインポートする
import { describe, it, expect } from "vitest";
// テスト対象の関数をインポートする
import {
  pseudolocalize,
  pseudolocalizeDict,
  isExpanded,
  estimateTextWidth,
} from "./pseudoloc";

// pseudolocalize 関数のテストスイート
describe("pseudolocalize", () => {
  // 空文字は変換せずそのまま返すことを確認する
  it("空文字は変換しない", () => {
    // 空文字を入力して空文字が返ることを確認する
    expect(pseudolocalize("")).toBe("");
  });

  // ASCII 文字が accented 文字に変換されることを確認する
  it("ASCII 文字を accented 文字に変換する", () => {
    // "a" → "ä" に変換されることを確認する
    const result = pseudolocalize("a", { bracket: false, expansionRatio: 1.0 });
    // "ä" が含まれることを確認する
    expect(result).toContain("ä");
  });

  // デフォルトオプションでブラケットが付与されることを確認する
  it("デフォルトでブラケット '[...]' を付与する", () => {
    // テスト文字列を pseudolocalize する
    const result = pseudolocalize("Hello");
    // ブラケットが付与されていることを確認する
    expect(result.startsWith("[")).toBe(true);
    // 末尾にブラケットが付与されていることを確認する
    expect(result.endsWith("]")).toBe(true);
  });

  // bracket: false の場合はブラケットが付与されないことを確認する
  it("bracket: false の場合はブラケットを付与しない", () => {
    // ブラケットなしで pseudolocalize する
    const result = pseudolocalize("Hello", { bracket: false });
    // ブラケットが付与されていないことを確認する
    expect(result.startsWith("[")).toBe(false);
    // 末尾にブラケットが付与されていないことを確認する
    expect(result.endsWith("]")).toBe(false);
  });

  // 30% 伸長されることを確認する
  it("文字列長が 30% 以上伸長されている（幅溢れ検出）", () => {
    // 10 文字の文字列を入力する
    const original = "HelloWorld";
    // デフォルトオプション（1.3 倍）で pseudolocalize する
    const result = pseudolocalize(original);
    // isExpanded で伸長確認する
    expect(isExpanded(original, result)).toBe(true);
  });

  // expansionRatio: 2.0 の場合は 2 倍伸長されることを確認する
  it("expansionRatio: 2.0 の場合は 2 倍伸長される", () => {
    // テスト文字列を定義する
    const original = "Test";
    // 2 倍伸長で pseudolocalize する
    const result = pseudolocalize(original, { expansionRatio: 2.0, bracket: false });
    // 元文字列より長いことを確認する
    expect(estimateTextWidth(result)).toBeGreaterThanOrEqual(
      Math.ceil(estimateTextWidth(original) * 2.0),
    );
  });

  // 大文字も変換されることを確認する
  it("大文字も accented 文字に変換する", () => {
    // 大文字 "A" を含む文字列を入力する
    const result = pseudolocalize("ABC", { bracket: false, expansionRatio: 1.0 });
    // "Ä" が含まれることを確認する
    expect(result).toContain("Ä");
    // "Ƀ" が含まれることを確認する
    expect(result).toContain("Ƀ");
    // "Ç" が含まれることを確認する
    expect(result).toContain("Ç");
  });
});

// pseudolocalizeDict 関数のテストスイート
describe("pseudolocalizeDict", () => {
  // 辞書の全 value が pseudolocalize されることを確認する
  it("辞書の全 value を pseudolocalize する", () => {
    // テスト用の翻訳辞書を定義する
    const dict = {
      // key: hello, value: Hello
      hello: "Hello",
      // key: world, value: World
      world: "World",
    };
    // 辞書全体を pseudolocalize する
    const result = pseudolocalizeDict(dict);
    // hello の値がブラケット付きで変換されていることを確認する
    expect(result["hello"]).toMatch(/^\[.*\]$/);
    // world の値がブラケット付きで変換されていることを確認する
    expect(result["world"]).toMatch(/^\[.*\]$/);
  });

  // key は変換されないことを確認する
  it("key は変換せずそのまま保持する", () => {
    // テスト用の翻訳辞書を定義する
    const dict = { greeting: "Hello" };
    // 辞書全体を pseudolocalize する
    const result = pseudolocalizeDict(dict);
    // "greeting" キーが存在することを確認する
    expect(Object.keys(result)).toContain("greeting");
  });

  // 空辞書は空辞書を返すことを確認する
  it("空辞書は空辞書を返す", () => {
    // 空辞書を pseudolocalize する
    const result = pseudolocalizeDict({});
    // 空オブジェクトを返すことを確認する
    expect(result).toEqual({});
  });
});

// isExpanded 関数のテストスイート
describe("isExpanded", () => {
  // 伸長された文字列が true を返すことを確認する
  it("伸長された文字列は true を返す", () => {
    // 元文字列を定義する
    const original = "TestString";
    // pseudolocalize で伸長する
    const pseudolocalized = pseudolocalize(original);
    // isExpanded が true を返すことを確認する
    expect(isExpanded(original, pseudolocalized)).toBe(true);
  });

  // 伸長されていない文字列が false を返すことを確認する
  it("伸長率が低い場合は false を返す", () => {
    // 元文字列を定義する
    const original = "TestString";
    // 元文字列と同じ長さの pseudolocalized を渡す（ブラケット "[...]" で 2 文字増加）
    // ブラケット 2 文字を引いた長さが元文字列より短い場合は false
    const tooShort = `[${original}]`; // len = 12, original = 10, 12-2=10, 10 >= ceil(10*1.3)=13? → false
    // isExpanded が false を返すことを確認する
    expect(isExpanded(original, tooShort, 1.3)).toBe(false);
  });
});
