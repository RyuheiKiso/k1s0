// k1s0 tier3 step-up 認証フローハンドラ
// v1_emergency_step_up 認証フロー（WebAuthn / TOTP / FIDO2）を実装する
// tier3 が直接 step-up フローを実行し、BFF 経由で認証レベルを昇格させる
// access_token は tier3 に公開しない（BFF cookie-only パターン維持）

import type { AuthCheckResult, StepUpMethod } from "./index.js";

// step-up フローの結果型
export type StepUpResult =
  // step-up 成功（認証レベルが昇格した）
  | { readonly status: "success" }
  // step-up 失敗（認証失敗 / タイムアウト）
  | { readonly status: "failure"; readonly reason: string }
  // step-up 中断（ユーザーがキャンセルした）
  | { readonly status: "cancelled" };

// WebAuthn / TOTP challenge のレスポンス型（BFF から取得する）
export interface StepUpChallenge {
  // チャレンジ ID（BFF セッションに紐付く）
  readonly challengeId: string;
  // step-up メソッド（webauthn / totp / fido2）
  readonly method: StepUpMethod;
  // チャレンジデータ（メソッドに応じた形式）
  readonly challengeData: string;
  // チャレンジ有効期限（HLC タイムスタンプ）
  readonly expiresAt: string;
}

// step-up フローの設定（BFF エンドポイント）
const STEP_UP_CHALLENGE_PATH = "/auth/step-up/challenge";
// step-up 完了エンドポイント
const STEP_UP_COMPLETE_PATH = "/auth/step-up/complete";
// CSRF 対策用カスタムヘッダー
const X_REQUESTED_WITH_VALUE = "k1s0-spa";

// step-up フローハンドラクラス
export class StepUpHandler {
  // BFF のベース URL（デフォルトは同一オリジン）
  private readonly _baseUrl: string;

  // コンストラクタ（ベース URL を受け取る）
  constructor(baseUrl = "") {
    // ベース URL を保持する
    this._baseUrl = baseUrl;
  }

  // step-up チャレンジを BFF から取得する
  async requestChallenge(method: StepUpMethod): Promise<StepUpChallenge> {
    // BFF にチャレンジリクエストを送信する
    const response = await this._fetch(STEP_UP_CHALLENGE_PATH, {
      // POST でチャレンジを要求する
      method: "POST",
      // メソッドを body に含める
      body: JSON.stringify({ method }),
      // Content-Type を設定する
      headers: { "Content-Type": "application/json" },
    });
    // HTTP エラーの場合はエラーを投げる
    if (!response.ok) {
      throw new Error(`step-up challenge request failed: HTTP ${response.status}`);
    }
    // チャレンジデータを返す
    return response.json() as Promise<StepUpChallenge>;
  }

  // WebAuthn step-up フローを実行する（PublicKeyCredential を使用する）
  async executeWebAuthn(challenge: StepUpChallenge): Promise<StepUpResult> {
    // WebAuthn API が利用可能かチェックする
    if (typeof window.PublicKeyCredential === "undefined") {
      // WebAuthn が利用できない場合は失敗を返す
      return { status: "failure", reason: "WebAuthn is not supported in this browser" };
    }
    // チャレンジデータを Uint8Array に変換する
    const challengeUint8 = this._base64urlToBytes(challenge.challengeData);
    // Uint8Array を ArrayBuffer に変換する（PublicKeyCredentialRequestOptions.challenge は BufferSource を要求する）
    const challengeBytes: ArrayBuffer = challengeUint8.buffer.slice(
      challengeUint8.byteOffset,
      challengeUint8.byteOffset + challengeUint8.byteLength,
    ) as ArrayBuffer;
    // WebAuthn assertion options を構築する
    const assertionOptions: PublicKeyCredentialRequestOptions = {
      // チャレンジを設定する（replay 防止）
      challenge: challengeBytes,
      // タイムアウト: 60 秒
      timeout: 60000,
      // user presence / user verification を要求する
      userVerification: "required",
    };
    // WebAuthn assertion を実行する
    let credential: PublicKeyCredential | null;
    try {
      // navigator.credentials.get() で WebAuthn assertion を取得する
      credential = await navigator.credentials.get({
        publicKey: assertionOptions,
      }) as PublicKeyCredential | null;
    } catch {
      // ユーザーがキャンセルした場合は cancelled を返す
      return { status: "cancelled" };
    }
    // credential が null の場合は失敗を返す
    if (credential === null) {
      return { status: "failure", reason: "No credential returned" };
    }
    // assertion response を BFF に送信して step-up を完了する
    return this._completeChallenge(challenge.challengeId, credential);
  }

  // TOTP step-up フローを実行する（ユーザー入力 OTP を BFF に送信する）
  async executeTOTP(challenge: StepUpChallenge, otpCode: string): Promise<StepUpResult> {
    // OTP コードの形式チェック（6 桁数字）
    if (!/^\d{6}$/.test(otpCode)) {
      // 不正な OTP コードはエラーを返す
      return { status: "failure", reason: "Invalid OTP code format (must be 6 digits)" };
    }
    // BFF に OTP を送信して step-up を完了する
    const response = await this._fetch(STEP_UP_COMPLETE_PATH, {
      // POST で OTP を送信する
      method: "POST",
      // challenge ID と OTP を body に含める
      body: JSON.stringify({ challengeId: challenge.challengeId, method: "totp", otpCode }),
      // Content-Type を設定する
      headers: { "Content-Type": "application/json" },
    });
    // HTTP エラーの場合は失敗を返す
    if (!response.ok) {
      return { status: "failure", reason: `TOTP verification failed: HTTP ${response.status}` };
    }
    // step-up 成功を返す
    return { status: "success" };
  }

  // v1_emergency_step_up フロー（AuthCheckResult が step_up_required の場合に呼ぶ）
  async handleEmergencyStepUp(
    checkResult: AuthCheckResult,
    options?: { otpCode?: string },
  ): Promise<StepUpResult> {
    // AuthCheckResult が step_up_required でない場合は無視する
    if (checkResult.status !== "step_up_required") {
      return { status: "failure", reason: "Not a step-up required state" };
    }
    // step-up メソッドに応じてフローを分岐する
    const method = checkResult.stepUpMethod;
    // チャレンジを BFF から取得する
    const challenge = await this.requestChallenge(method);
    // メソッドに応じてフローを実行する
    switch (method) {
      case "webauthn":
      case "fido2":
        // WebAuthn / FIDO2 フローを実行する
        return this.executeWebAuthn(challenge);
      case "totp": {
        // TOTP フローを実行する（OTP コードが必要）
        const otpCode = options?.otpCode;
        if (!otpCode) {
          // OTP コードが指定されていない場合はエラーを返す
          return { status: "failure", reason: "OTP code is required for TOTP step-up" };
        }
        return this.executeTOTP(challenge, otpCode);
      }
      default: {
        // 未知のメソッドは exhaustive チェック
        const _exhaustive: never = method;
        return { status: "failure", reason: `Unknown step-up method: ${String(_exhaustive)}` };
      }
    }
  }

  // WebAuthn assertion を BFF に送信して step-up を完了する
  private async _completeChallenge(
    challengeId: string,
    credential: PublicKeyCredential,
  ): Promise<StepUpResult> {
    // assertion response を JSON シリアライズ可能な形式に変換する
    const response = credential.response as AuthenticatorAssertionResponse;
    // 署名データを base64url にエンコードする
    const assertionData = {
      // credential ID を base64url で送信する
      credentialId: credential.id,
      // authenticator data を base64url で送信する
      authenticatorData: this._bytesToBase64url(new Uint8Array(response.authenticatorData)),
      // signature を base64url で送信する
      signature: this._bytesToBase64url(new Uint8Array(response.signature)),
      // client data JSON を base64url で送信する
      clientDataJSON: this._bytesToBase64url(new Uint8Array(response.clientDataJSON)),
    };
    // BFF に assertion を送信する
    const fetchResponse = await this._fetch(STEP_UP_COMPLETE_PATH, {
      // POST で assertion を送信する
      method: "POST",
      // challenge ID と assertion を body に含める
      body: JSON.stringify({ challengeId, method: "webauthn", assertionData }),
      // Content-Type を設定する
      headers: { "Content-Type": "application/json" },
    });
    // HTTP エラーの場合は失敗を返す
    if (!fetchResponse.ok) {
      return { status: "failure", reason: `WebAuthn verification failed: HTTP ${fetchResponse.status}` };
    }
    // step-up 成功を返す
    return { status: "success" };
  }

  // Base64url デコードヘルパー
  private _base64urlToBytes(base64url: string): Uint8Array {
    // base64url → base64 に変換する（- → + / _ → /）
    const base64 = base64url.replace(/-/g, "+").replace(/_/g, "/");
    // padding を追加する
    const padded = base64.padEnd(base64.length + ((4 - (base64.length % 4)) % 4), "=");
    // atob で バイナリ文字列に変換する
    const binary = atob(padded);
    // Uint8Array に変換して返す
    const bytes = new Uint8Array(binary.length);
    for (let i = 0; i < binary.length; i++) {
      // 各バイトをセットする
      bytes[i] = binary.charCodeAt(i);
    }
    return bytes;
  }

  // Uint8Array → Base64url エンコードヘルパー
  private _bytesToBase64url(bytes: Uint8Array): string {
    // バイト列を binary 文字列に変換する
    const binary = Array.from(bytes)
      .map((b) => String.fromCharCode(b))
      .join("");
    // btoa で base64 に変換する
    const base64 = btoa(binary);
    // base64url に変換する（+ → - / / → _ / = を除去）
    return base64.replace(/\+/g, "-").replace(/\//g, "_").replace(/=+$/, "");
  }

  // 共通 fetch ラッパー（credentials: include + X-Requested-With を自動付与する）
  private async _fetch(path: string, init: RequestInit): Promise<Response> {
    // リクエスト URL を生成する
    const url = `${this._baseUrl}${path}`;
    // fetch を呼び出す
    return fetch(url, {
      // 呼び出し元の init をスプレッドする
      ...init,
      // HttpOnly cookie を自動送信するため credentials を include に設定する
      credentials: "include",
      // リクエストヘッダーに CSRF 対策ヘッダーを付与する
      headers: {
        // JSON レスポンスを要求する
        Accept: "application/json",
        // CSRF 対策: BFF がオリジンを確認するためのカスタムヘッダー
        "X-Requested-With": X_REQUESTED_WITH_VALUE,
        // 呼び出し元が追加ヘッダーを渡した場合はマージする
        ...(init.headers ?? {}),
      },
    });
  }
}

// デフォルト StepUpHandler シングルトン（同一オリジン向け）
export const defaultStepUpHandler = new StepUpHandler();
