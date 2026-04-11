// Appコンポーネントのユニットテストファイル。

// テストユーティリティをインポートする
import { render, screen } from "@testing-library/react";
// Vitestのテスト関数をインポートする
import { describe, it, expect } from "vitest";
// テスト対象のAppコンポーネントをインポートする
import App from "../App";

// Appコンポーネントのテストスイートを定義する
describe("App", () => {
  // タイトルが表示されることを確認するテスト
  it("タイトル『k1s0 セットアップツール』が表示される", () => {
    // Appコンポーネントをレンダリングする
    render(<App />);
    // タイトルテキストが存在することを確認する
    expect(screen.getByText("k1s0 セットアップツール")).toBeInTheDocument();
  });

  // インストール確認セクションが表示されることを確認するテスト
  it("インストール確認セクションが表示される", () => {
    // Appコンポーネントをレンダリングする
    render(<App />);
    // インストール確認の見出しが存在することを確認する
    expect(screen.getByText("インストール確認")).toBeInTheDocument();
  });

  // 確認ボタンが表示されることを確認するテスト
  it("確認を実行するボタンが表示される", () => {
    // Appコンポーネントをレンダリングする
    render(<App />);
    // ボタンが存在することを確認する
    expect(screen.getByRole("button", { name: "確認を実行する" })).toBeInTheDocument();
  });
});
