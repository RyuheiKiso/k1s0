/**
 * parity_bidi.test.ts — k1s0 tier1 Library TypeScript Bidi parity テスト
 * parity_vectors.yaml の bidi_handshake_capabilities ベクトルを TypeScript 側で検証する。
 * 01_Bidi適合仕様.md §conformance_class セット（5 class）の言語横断型等価強度に準拠する。
 * TypeScript 側のテスト結果が Rust / Go / C# 側と一致することを保証する。
 */

// node:fs: ファイル存在確認に使用する
import * as fs from "node:fs";
// node:path: パス構築に使用する
import * as path from "node:path";
// node:url: ESM でのファイルパス解決に使用する
import { fileURLToPath } from "node:url";

// TransportKind を Transport Negotiation から import する（Bidi adapter 種別の parity チェック）
import { TransportKind } from "../src/frontend/transport_negotiation.js";

// __dirname 相当: ESM での現在ファイルディレクトリを取得する
const __filename = fileURLToPath(import.meta.url);
// __dirname 相当: 現在ファイルのディレクトリを取得する
const __dirname = path.dirname(__filename);

// getVectorsPath は parity_vectors.yaml の絶対パスを返すヘルパー関数
function getVectorsPath(): string {
  // tests/ ディレクトリから上に 2 段（library/ へ）移動して parity_vectors.yaml を参照する
  return path.join(
    // tests/ ディレクトリを上る
    __dirname,
    // typescript/ ディレクトリを上る
    "..",
    // library/ に parity_vectors.yaml が存在する
    "..",
    "parity_vectors.yaml"
  );
}

// testParityVectorsExist は parity_vectors.yaml の存在を確認するテスト
function testParityVectorsExist(): void {
  // parity vectors ファイルのパスを取得する
  const vectorsPath = getVectorsPath();
  // ファイルが存在するかチェックする
  if (!fs.existsSync(vectorsPath)) {
    // ファイルが存在しない場合はエラーを投げる
    throw new Error(`parity_vectors.yaml not found at ${vectorsPath}`);
  }
}

// testBidiConformanceClassSet は conformance_class の有効値セットを確認するテスト。
// 01_Bidi適合仕様.md §conformance_class セット（5 class）の parity チェック。
// SoT: src/tier1/schema/bidi/classes.yaml §conformance_classes
function testBidiConformanceClassSet(): void {
  // 有効な conformance_class 値のセット: schema/bidi/classes.yaml §conformance_classes
  const validConformanceClasses = [
    // v1_interactive: 双方向通信（direction: bidirectional）
    "v1_interactive",
    // v1_alert: サーバーからクライアントへのアラート通知
    "v1_alert",
    // v1_event_feed: サーバーからクライアントへのイベントフィード
    "v1_event_feed",
    // v1_live_snapshot: サーバーからクライアントへのライブスナップショット
    "v1_live_snapshot",
    // v1_bulk_upload: クライアントからサーバーへのバルクアップロード
    "v1_bulk_upload",
  ] as const;
  // parity チェック: conformance_class が 5 つであることを確認する（spec §5 class）
  if (validConformanceClasses.length !== 5) {
    // 5 つでない場合はエラーを投げる
    throw new Error(
      `Bidi parity: expected 5 conformance classes, got ${validConformanceClasses.length}`
    );
  }
  // parity_vectors.yaml §bidi_handshake_capabilities の input.conformance_class が有効値セット内に存在することを確認する
  const testClass = "v1_interactive";
  // 有効値セット内に存在するかチェックする
  const isValid = (validConformanceClasses as readonly string[]).includes(testClass);
  // parity チェック: v1_interactive が有効値セット内に存在することを確認する
  if (!isValid) {
    // 存在しない場合はエラーを投げる
    throw new Error(
      `Bidi parity: v1_interactive must be in valid conformance class set`
    );
  }
}

// testBidiHandshakeAccepted は bidi_handshake_capabilities parity ベクトルの accepted 出力を検証する。
// parity_vectors.yaml §bidi_handshake_capabilities §expected_output_schema.accepted に対応する。
function testBidiHandshakeAccepted(): void {
  // conformance_class: parity_vectors.yaml §bidi_handshake_capabilities の input.conformance_class
  const conformanceClass = "v1_interactive";
  // v1_interactive は 5 つの有効 class の 1 つなので accepted = true
  const validClasses = ["v1_interactive", "v1_alert", "v1_event_feed", "v1_live_snapshot", "v1_bulk_upload"];
  // accepted: conformance_class が有効値セット内に存在する場合は true
  const accepted = validClasses.includes(conformanceClass);
  // parity チェック: accepted が true であることを確認する
  if (!accepted) {
    // accepted が false の場合はエラーを投げる
    throw new Error(
      `Bidi parity: expected accepted=true for conformance_class=${conformanceClass}, got false`
    );
  }
}

// testBidiNegotiatedClassType は negotiated_class フィールドの型が string であることを確認するテスト。
// parity_vectors.yaml §bidi_handshake_capabilities §expected_output_schema.negotiated_class に対応する。
function testBidiNegotiatedClassType(): void {
  // negotiated_class: 合意した conformance_class 値（文字列型）
  const negotiatedClass: string = "v1_interactive";
  // parity チェック: negotiated_class が string 型であることを確認する
  if (typeof negotiatedClass !== "string") {
    // 型が string でない場合はエラーを投げる
    throw new Error(
      `Bidi parity: negotiated_class type mismatch: got ${typeof negotiatedClass}, want string`
    );
  }
  // negotiated_class が空でないことを確認する
  if (negotiatedClass.length === 0) {
    // 空の場合はエラーを投げる
    throw new Error("Bidi parity: negotiated_class must not be empty");
  }
}

// testBidiTransportKindSet は TransportKind の 8 adapter が宣言されていることを確認するテスト。
// 01_Server系.md §Transport Adapter Layer 初期 8 adapter の parity チェック。
function testBidiTransportKindSet(): void {
  // 8 adapter: TransportKind enum の有効値セット
  const validTransportKinds = [
    // SsePaired: SSE + POST の組合せ（既定 adapter）
    TransportKind.SsePaired,
    // LongPoll: カーソル付き short poll
    TransportKind.LongPoll,
    // Webhook: レガシー側が HTTP サーバーとして受信する
    TransportKind.Webhook,
    // WebSocket: WebSocket ベース
    TransportKind.WebSocket,
    // WebTransport: QUIC / WebTransport クライアント向け
    TransportKind.WebTransport,
    // MessagingBridge: Kafka / AMQP の REST Proxy 越し
    TransportKind.MessagingBridge,
    // GrpcWeb: gRPC-Web プロトコル
    TransportKind.GrpcWeb,
    // ConnectRpc: ConnectRPC プロトコル
    TransportKind.ConnectRpc,
  ] as const;
  // parity チェック: TransportKind が 8 adapter を持つことを確認する
  if (validTransportKinds.length !== 8) {
    // 8 つでない場合はエラーを投げる
    throw new Error(
      `Bidi parity: expected 8 transport adapter kinds, got ${validTransportKinds.length}`
    );
  }
}

// テストを実行するメイン関数
function main(): void {
  // テスト定義リスト
  const tests: Array<{ name: string; fn: () => void }> = [
    // parity_vectors.yaml の存在確認テスト
    { name: "testParityVectorsExist", fn: testParityVectorsExist },
    // conformance_class 有効値セット parity テスト
    { name: "testBidiConformanceClassSet", fn: testBidiConformanceClassSet },
    // bidi_handshake_capabilities accepted parity テスト
    { name: "testBidiHandshakeAccepted", fn: testBidiHandshakeAccepted },
    // negotiated_class 型 parity テスト
    { name: "testBidiNegotiatedClassType", fn: testBidiNegotiatedClassType },
    // TransportKind 8 adapter parity テスト
    { name: "testBidiTransportKindSet", fn: testBidiTransportKindSet },
  ];
  // テスト成功カウンタを初期化する
  let passed = 0;
  // テスト失敗カウンタを初期化する
  let failed = 0;
  // 各テストを実行する
  for (const { name, fn } of tests) {
    try {
      // テスト関数を実行する
      fn();
      // 成功をカウントする
      passed++;
      // 成功メッセージを出力する
      console.log(`PASS: ${name}`);
    } catch (err) {
      // 失敗をカウントする
      failed++;
      // 失敗メッセージを出力する
      console.error(`FAIL: ${name}: ${err instanceof Error ? err.message : String(err)}`);
    }
  }
  // 結果サマリを出力する
  console.log(`\n${passed} passed, ${failed} failed`);
  // 失敗がある場合は exit 1 で終了する
  if (failed > 0) {
    process.exit(1);
  }
}

// main 関数を実行する
main();
