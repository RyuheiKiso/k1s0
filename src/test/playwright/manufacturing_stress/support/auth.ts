// auth.ts — Keycloak OIDC トークン取得ヘルパー
// 製造業 stress test で tier1 gateway に認証付きリクエストを送る際に使用する
// フルスタックモード以外では固定のスタブトークンを返す

// Keycloak トークンレスポンスの型定義
interface KeycloakTokenResponse {
  // JWT アクセストークン
  access_token: string;
  // トークン有効期限（秒）
  expires_in: number;
  // トークンタイプ（常に Bearer）
  token_type: string;
}

// スタブトークンの固定文字列（モックモードで使用する）
const STUB_TOKEN = "stub-manufacturing-token-k1s0-test";

// Keycloak からアクセストークンを取得する
export async function getKeycloakToken(
  keycloakUrl: string | null,
  username: string,
  password: string = "test-password"
): Promise<string> {
  // keycloakUrl が null の場合はスタブトークンを返す（モックモード）
  if (!keycloakUrl) {
    return STUB_TOKEN;
  }

  // Keycloak の realm とクライアント ID を設定する
  const realm = "k1s0-manufacturing";
  // クライアント ID は製造業ロール専用のものを使用する
  const clientId = "k1s0-manufacturing-client";

  // Keycloak トークンエンドポイント URL を組み立てる
  const tokenUrl = `${keycloakUrl}/realms/${realm}/protocol/openid-connect/token`;

  // トークン取得リクエストのフォームデータを組み立てる
  const params = new URLSearchParams({
    grant_type: "password",
    client_id: clientId,
    username,
    password,
  });

  // Keycloak トークンエンドポイントに POST リクエストを送信する
  const response = await fetch(tokenUrl, {
    method: "POST",
    headers: {
      "Content-Type": "application/x-www-form-urlencoded",
    },
    body: params.toString(),
  });

  // レスポンスが成功でない場合はエラーをスローする
  if (!response.ok) {
    throw new Error(`Keycloak トークン取得失敗: ${response.status} ${response.statusText}`);
  }

  // JSON レスポンスからアクセストークンを取り出す
  const data = (await response.json()) as KeycloakTokenResponse;
  return data.access_token;
}

// 製造業オペレーター用トークンを取得する（テナント A 固定）
export async function getManufacturingOperatorToken(
  keycloakUrl: string | null
): Promise<string> {
  // テナント A の製造業オペレーターアカウントでトークンを取得する
  return getKeycloakToken(keycloakUrl, "tenant-a-fa-operator");
}
