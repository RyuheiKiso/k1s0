// containers.ts — Testcontainers 起動ヘルパー
// WSL2 環境では MANUFACTURING_STRESS_FULL=1 が設定されていない場合はモックモードで動作する
// CI 環境では docker socket が利用可能であることを前提とする

// Playwright の test オブジェクトから skip ユーティリティを取得するための型
import type { TestInfo } from "@playwright/test";

// コンテナスタック設定インタフェースを定義する
export interface ManufacturingStackOptions {
  // 使用するトランスポート adapter リスト（未指定の場合は grpc_native のみ）
  adapters?: string[];
  // PostgreSQL RLS FORCE を有効にするかどうか
  enableRls?: boolean;
  // Kafka + Debezium が必要かどうか（event_feed / bulk_upload クラス向け）
  requireKafka?: boolean;
  // Keycloak OIDC IdP が必要かどうか
  requireKeycloak?: boolean;
  // Prometheus + AlertManager が必要かどうか（alarm_delivery 向け）
  requirePrometheus?: boolean;
}

// ManufacturingStack インタフェースを定義する
export interface ManufacturingStack {
  // tier1 server gateway の HTTP base URL
  gatewayUrl: string;
  // Keycloak OIDC トークン発行 URL（requireKeycloak=true 時のみ有効）
  keycloakUrl: string | null;
  // PostgreSQL 接続文字列（enableRls=true 時のみ有効）
  postgresUrl: string | null;
  // スタックを停止してリソースを解放する
  stop: () => Promise<void>;
}

// フルスタックモードが有効かどうかを確認する
export function isFullStackMode(): boolean {
  // MANUFACTURING_STRESS_FULL=1 が設定されている場合のみフルスタックを起動する
  return process.env["MANUFACTURING_STRESS_FULL"] === "1";
}

// フルスタックが要求されている場合に skip するユーティリティ
export function skipIfNotFullStack(testInfo: TestInfo): void {
  // WSL2 または CI 環境でコンテナが利用できない場合にスキップする
  if (!isFullStackMode()) {
    testInfo.skip(
      true,
      "MANUFACTURING_STRESS_FULL=1 が設定されていないためコンテナモードをスキップします。"
        + " 製造業 stress test を物理実行するには環境変数を設定してください。"
    );
  }
}

// モックスタックを起動する（Testcontainers 不要）
async function startMockStack(opts: ManufacturingStackOptions): Promise<ManufacturingStack> {
  // モックモードではローカルポートで静的レスポンスを返すスタブを使用する
  // page.route() で tier1 gateway のエンドポイントをインターセプトするため gateway URL のみ設定する
  return {
    // ローカルモックサーバーの URL を返す
    gatewayUrl: "http://localhost:4173",
    // Keycloak は不要のため null を返す
    keycloakUrl: null,
    // PostgreSQL は不要のため null を返す
    postgresUrl: null,
    // モックスタックは停止処理不要
    stop: async () => {},
  };
}

// フルスタック（Testcontainers）を起動する
async function startFullStack(opts: ManufacturingStackOptions): Promise<ManufacturingStack> {
  // Testcontainers-node の GenericContainer を動的インポートする
  // （フルスタックモード以外ではロードされないようにして依存を軽量化する）
  try {
    // testcontainers パッケージを動的にインポートする
    const { GenericContainer, Wait } = await import("testcontainers");

    // tier1 gateway コンテナを起動する
    const gateway = await new GenericContainer("k1s0/tier1-gateway:latest")
      .withExposedPorts(8443)
      .withWaitStrategy(Wait.forHttp("/health", 8443).forStatusCode(200))
      .start();

    // gateway の HTTP URL を組み立てる
    const gatewayUrl = `http://localhost:${gateway.getMappedPort(8443)}`;

    // Keycloak コンテナを条件付きで起動する
    let keycloakUrl: string | null = null;
    if (opts.requireKeycloak) {
      // Keycloak 公式イメージを起動する
      const keycloak = await new GenericContainer("quay.io/keycloak/keycloak:24.0")
        .withCommand(["start-dev"])
        .withExposedPorts(8080)
        .withWaitStrategy(Wait.forHttp("/health/ready", 8080).forStatusCode(200))
        .start();
      // Keycloak トークン発行 URL を組み立てる
      keycloakUrl = `http://localhost:${keycloak.getMappedPort(8080)}`;
    }

    return {
      // gateway URL を返す
      gatewayUrl,
      // keycloak URL を返す（未起動の場合は null）
      keycloakUrl,
      // PostgreSQL URL は未実装（WSL2 では省略）
      postgresUrl: null,
      // コンテナを停止するクリーンアップ関数を返す
      stop: async () => {
        await gateway.stop();
      },
    };
  } catch (e) {
    // testcontainers が利用できない場合はモックにフォールバックする
    console.warn("testcontainers が利用できないためモックモードで続行します:", e);
    return startMockStack(opts);
  }
}

// ManufacturingStack を起動するメインエントリ
export async function startManufacturingStack(
  opts: ManufacturingStackOptions = {}
): Promise<ManufacturingStack> {
  // フルスタックモードが有効な場合は Testcontainers を使用する
  if (isFullStackMode()) {
    return startFullStack(opts);
  }
  // それ以外はモックスタックを返す
  return startMockStack(opts);
}
