// k1s0 tier3 attachments パッケージ ユニットテスト
// checkMimeType / buildHashChain / verifyHashChain / createSandboxUrl の純粋関数テスト
import { describe, it, expect } from "vitest";
import {
  checkMimeType,
  ALLOWED_MIME_TYPES,
  buildHashChain,
  verifyHashChain,
  createSandboxUrl,
  type AttachmentMetadata,
} from "./index.js";

// ---- checkMimeType テスト ----

describe("checkMimeType", () => {
  // 許可 MIME タイプが true を返すことを確認する
  it("許可 MIME タイプは true を返すこと", () => {
    // ALLOWED_MIME_TYPES の各値で true が返ることを確認する
    for (const mime of ALLOWED_MIME_TYPES) {
      // 許可 MIME タイプは true を返すべき
      expect(checkMimeType(mime), `MIME タイプ "${mime}" は許可されるべき`).toBe(true);
    }
  });

  // 非許可 MIME タイプが false を返すことを確認する
  it("非許可 MIME タイプは false を返すこと", () => {
    // 禁止 MIME タイプの一覧
    const forbidden = [
      "application/x-executable",
      "text/html",
      "application/javascript",
      "image/svg+xml",
      "application/zip",
    ];
    // 各禁止 MIME タイプで false が返ることを確認する
    for (const mime of forbidden) {
      // 非許可 MIME タイプは false を返すべき
      expect(checkMimeType(mime), `MIME タイプ "${mime}" は禁止されるべき`).toBe(false);
    }
  });

  // 空文字列が false を返すことを確認する
  it("空文字列は false を返すこと", () => {
    // 空文字列は許可されていない
    expect(checkMimeType("")).toBe(false);
  });
});

// ---- buildHashChain テスト ----

describe("buildHashChain", () => {
  // 単一チャンクのハッシュチェーンが正しいフォーマットになることを確認する
  it("単一チャンクのハッシュチェーンが正しいフォーマットであること", () => {
    // ダミーハッシュを使用する
    const hashes = ["abc123def456"];
    // ハッシュチェーンを生成する
    const chain = buildHashChain(hashes);
    // 期待値: "sha256:abc123def456"
    expect(chain).toBe("sha256:abc123def456");
  });

  // 複数チャンクのハッシュチェーンがパイプ区切りになることを確認する
  it("複数チャンクのハッシュチェーンがパイプ区切りであること", () => {
    // 3 チャンクのダミーハッシュ
    const hashes = ["hash0", "hash1", "hash2"];
    // ハッシュチェーンを生成する
    const chain = buildHashChain(hashes);
    // 期待値: パイプ区切りの sha256: プレフィックス付き文字列
    expect(chain).toBe("sha256:hash0|sha256:hash1|sha256:hash2");
  });

  // 空配列では空文字列が返ることを確認する
  it("空配列では空文字列が返ること", () => {
    // 空配列のハッシュチェーンは空文字列
    expect(buildHashChain([])).toBe("");
  });
});

// ---- verifyHashChain テスト ----

describe("verifyHashChain", () => {
  // 正しいハッシュチェーンで true が返ることを確認する
  it("正しいハッシュチェーンで true を返すこと", () => {
    // テスト用のメタデータを作成する
    const metadata: AttachmentMetadata = {
      // 添付ファイル ID
      id: "test-id-001",
      // テナント識別子
      tenantId: "tenant-001",
      // MIME タイプ
      mimeType: "application/pdf",
      // ファイルサイズ
      sizeBytes: 1024,
      // 正しいハッシュチェーン
      hashChain: "sha256:hash0|sha256:hash1",
      // アップロード時刻
      uploadedAt: "2026-05-21T00:00:00Z",
    };
    // 再計算したハッシュ（metadata と一致する値）
    const computedHashes = ["hash0", "hash1"];
    // 正しいハッシュチェーンで true が返ることを確認する
    expect(verifyHashChain(metadata, computedHashes)).toBe(true);
  });

  // 不正なハッシュチェーンで false が返ることを確認する
  it("不正なハッシュチェーンで false を返すこと", () => {
    // テスト用のメタデータを作成する
    const metadata: AttachmentMetadata = {
      // 添付ファイル ID
      id: "test-id-002",
      // テナント識別子
      tenantId: "tenant-001",
      // MIME タイプ
      mimeType: "application/pdf",
      // ファイルサイズ
      sizeBytes: 1024,
      // 異なるハッシュチェーン（改ざんされた値）
      hashChain: "sha256:hash0|sha256:TAMPERED",
      // アップロード時刻
      uploadedAt: "2026-05-21T00:00:00Z",
    };
    // 再計算したハッシュ（metadata と不一致）
    const computedHashes = ["hash0", "hash1"];
    // 不正なハッシュチェーンで false が返ることを確認する
    expect(verifyHashChain(metadata, computedHashes)).toBe(false);
  });
});

// ---- createSandboxUrl テスト ----

describe("createSandboxUrl", () => {
  // 正しい sandbox URL が生成されることを確認する
  it("正しい sandbox URL が生成されること", () => {
    // テスト用の添付ファイル ID と BFF オリジン
    const attachmentId = "att-uuid-001";
    const bffOrigin = "https://bff.example.com";
    // sandbox URL を生成する
    const url = createSandboxUrl(attachmentId, bffOrigin);
    // 期待値: BFF オリジン + attachments/{id}/view
    expect(url).toBe("https://bff.example.com/attachments/att-uuid-001/view");
  });

  // attachmentId に特殊文字が含まれる場合に URL エンコードされることを確認する
  it("attachmentId の特殊文字が URL エンコードされること", () => {
    // スラッシュを含む attachmentId
    const attachmentId = "att/uuid/001";
    const bffOrigin = "https://bff.example.com";
    // sandbox URL を生成する
    const url = createSandboxUrl(attachmentId, bffOrigin);
    // スラッシュが %2F にエンコードされることを確認する
    expect(url).toContain("att%2Fuuid%2F001");
  });
});
