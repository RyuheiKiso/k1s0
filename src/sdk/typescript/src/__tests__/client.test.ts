// k1s0 TypeScript SDK の最小単体テスト雛形。
//
// 範囲（リリース時点）:
//   - K1s0Config の妥当性
//   - K1s0Client の構築（外部接続を行わないモック transport）
//   - tenantContext() が正しい TenantContext proto を返す
//   - Severity 列挙が OTel SeverityNumber と整合
//
// 採用初期で拡張:
//   - mock transport による各 facade method の往復テスト

import { describe, expect, it, beforeEach } from "vitest";
import { K1s0Client, type K1s0Config } from "../client.js";
import { Severity } from "../proto/k1s0/tier1/log/v1/log_service_pb.js";
import { K1s0ErrorCategory } from "../proto/k1s0/tier1/common/v1/common_pb.js";

// 外部接続を抑止するためのダミー transport（unaryFn / streamFn は実装されないが Client 構築には不要）。
const dummyTransport = {
  unary: async () => {
    throw new Error("not connected");
  },
  stream: async () => {
    throw new Error("not connected");
  },
} as unknown as K1s0Config["transport"];

describe("K1s0Client construction", () => {
  let client: K1s0Client;

  beforeEach(() => {
    client = new K1s0Client({
      baseUrl: "https://tier1.k1s0.example.com",
      tenantId: "tenant-A",
      subject: "svc-foo",
      transport: dummyTransport,
    });
  });

  it("holds config", () => {
    expect(client.config.tenantId).toBe("tenant-A");
    expect(client.config.subject).toBe("svc-foo");
  });

  it("exposes 14 facades", () => {
    expect(client.state).toBeDefined();
    expect(client.pubsub).toBeDefined();
    expect(client.secrets).toBeDefined();
    expect(client.log).toBeDefined();
    expect(client.workflow).toBeDefined();
    expect(client.decision).toBeDefined();
    expect(client.audit).toBeDefined();
    expect(client.pii).toBeDefined();
    expect(client.feature).toBeDefined();
    expect(client.binding).toBeDefined();
    expect(client.invoke).toBeDefined();
    expect(client.telemetry).toBeDefined();
    expect(client.decisionAdmin).toBeDefined();
    expect(client.featureAdmin).toBeDefined();
  });

  it("tenantContext propagates config", () => {
    const ctx = client.tenantContext();
    expect(ctx.tenantId).toBe("tenant-A");
    expect(ctx.subject).toBe("svc-foo");
    expect(ctx.correlationId).toBe("");
  });
});

describe("Severity OTel alignment", () => {
  it("matches SeverityNumber from OTel", () => {
    // docs 正典 docs/03_要件定義/20_機能要件/40_tier1_API契約IDL/07_Log_API.md の数値仕様。
    expect(Severity.TRACE).toBe(0);
    expect(Severity.DEBUG).toBe(5);
    expect(Severity.INFO).toBe(9);
    expect(Severity.WARN).toBe(13);
    expect(Severity.ERROR).toBe(17);
    expect(Severity.FATAL).toBe(21);
  });
});

describe("K1s0ErrorCategory", () => {
  it("matches IDL category numbering", () => {
    // docs 正典 docs/03_要件定義/20_機能要件/40_tier1_API契約IDL/00_共通型定義.md。
    expect(K1s0ErrorCategory.K1S0_ERROR_UNSPECIFIED).toBe(0);
    expect(K1s0ErrorCategory.K1S0_ERROR_INVALID_ARGUMENT).toBe(1);
    expect(K1s0ErrorCategory.K1S0_ERROR_DEADLINE_EXCEEDED).toBe(9);
  });
});
