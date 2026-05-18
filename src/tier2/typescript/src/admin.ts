/**
 * k1s0 tier2 admin TypeScript インターフェース定義
 * Rust 実装（admin/src/admin_boundary.rs, admin/src/admin_operation.rs）と 4 言語等価強度を持つ TypeScript 版
 * すべての管理操作は IAdminBoundaryGuard を通過してから実行する（設計方針 14）
 */

/**
 * AdminOperationKind: 管理境界を通過できる操作の識別子（業界中立語のみ）
 * Rust の AdminOperation enum に対応する
 */
// AdminOperationKind 型定義（文字列リテラル union で型安全を保証する）
export type AdminOperationKind =
  // テナントプロビジョニング（新規テナントのリソース割り当て）
  | "TenantProvision"
  // テナント一時停止（すべての API アクセスを拒否状態にする）
  | "TenantSuspend"
  // クォータ上限の一時的な上書き（緊急措置）
  | "QuotaOverride"
  // ユーザーへの権限付与（デュアル承認を強制する）
  | "UserGrant"
  // 緊急アクセス（インシデント対応時のみ / デュアル承認必須）
  | "EmergencyAccess";

/**
 * requiresDualApproval: この操作がデュアル承認を必要とするか返す
 * Rust の AdminOperation.requires_dual_approval() に対応する
 */
// requiresDualApproval 関数（デュアル承認チェック）
export function requiresDualApproval(kind: AdminOperationKind): boolean {
  // EmergencyAccess と QuotaOverride は常にデュアル承認を要求する
  return kind === "EmergencyAccess" || kind === "QuotaOverride";
}

/**
 * AdminRequest: 管理操作リクエストのエンベロープ型
 * Rust の AdminRequest 構造体に対応する
 */
// AdminRequest インターフェース定義
export interface AdminRequest {
  // リクエスト一意識別子（UUID v4 / 監査ログの相関 ID に使用する）
  readonly requestId: string;
  // 呼び出し元識別子（ユーザー ID またはサービスアカウント ID）
  readonly callerId: string;
  // 実行しようとしている管理操作種別
  readonly operationKind: AdminOperationKind;
  // 操作の正当化理由（監査ログに記録される）
  readonly justification: string;
  // 操作固有パラメータ（JSON シリアライズ形式で渡す）
  readonly operationPayloadJson?: string;
}

/**
 * AdminResponse: 管理操作レスポンスのエンベロープ型
 * Rust の AdminResponse 構造体に対応する
 */
// AdminResponse インターフェース定義
export interface AdminResponse {
  // 対応するリクエスト識別子（相関追跡に使用する）
  readonly requestId: string;
  // 操作が成功したか
  readonly success: boolean;
  // 操作結果メッセージ（成功時は完了詳細 / 失敗時はエラー詳細）
  readonly message: string;
  // 操作が監査ログに記録されたか（必ず true でなければならない）
  readonly auditRecorded: boolean;
}

/**
 * AdminBoundaryErrorKind: 管理境界違反の種別
 * Rust の AdminBoundaryError バリアントに対応する
 */
// AdminBoundaryErrorKind 型定義（文字列リテラル union）
export type AdminBoundaryErrorKind =
  // 呼び出し元トークンが管理スコープを持っていない
  | "InsufficientScope"
  // デュアル承認が必要な操作で承認が足りない
  | "DualApprovalRequired"
  // 操作対象テナントへのアクセス権がない
  | "TenantAccessDenied";

/**
 * AdminBoundaryError: 管理境界違反エラー
 * Rust の AdminBoundaryError に対応する
 */
// AdminBoundaryError クラス定義（Error を継承する）
export class AdminBoundaryError extends Error {
  // 違反種別
  readonly kind: AdminBoundaryErrorKind;

  // コンストラクタ（違反種別とメッセージを受け取る）
  constructor(kind: AdminBoundaryErrorKind, message: string) {
    // 基底クラスのコンストラクタを呼び出す
    super(message);
    // エラー名を設定する
    this.name = "AdminBoundaryError";
    // 違反種別を格納する
    this.kind = kind;
  }
}

/**
 * IAdminBoundaryGuard: 管理操作の境界チェックを実装する TypeScript インターフェース
 * Rust の AdminBoundaryGuard トレイトに対応する（設計方針 14）
 * Keycloak + Kyverno ポリシーと連動してスコープ検証を行う
 * tenant_id は AuthContext から取得するため API 引数で受け取らない
 */
// IAdminBoundaryGuard インターフェース定義
export interface IAdminBoundaryGuard {
  /**
   * 管理操作の実行スコープを検証する
   * token: Bearer トークン（Keycloak JWT）
   * operationKind: 実行しようとしている管理操作種別
   * スコープ不足またはデュアル承認未完了の場合は AdminBoundaryError をスローする
   */
  // checkAdminScope メソッド（スコープ検証操作）
  checkAdminScope(token: string, operationKind: AdminOperationKind): Promise<void>;

  /**
   * 管理リクエストを処理する（スコープ検証後に呼び出す）
   * 操作の実行結果を AdminResponse として返す
   * tenant_id は AuthContext から取得するため引数で受け取らない
   */
  // processRequest メソッド（管理操作実行）
  processRequest(request: AdminRequest): Promise<AdminResponse>;
}
