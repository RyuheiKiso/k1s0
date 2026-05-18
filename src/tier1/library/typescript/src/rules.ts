/**
 * rules.ts — k1s0 tier1 Library TypeScript 実装: Rule Engine の L1+ interface
 * 17_ルールエンジン適合仕様.md §RuleEngineClient（OPA / Drools L1+ 深耕）に準拠する。
 * OPA（Open Policy Agent）の full API を Library 独自語彙で表現しつつ、AuthContext 伝播を強制する。
 * OSS 型（@open-policy-agent/opa-wasm 等）を公開シグネチャに一切含まない。
 */

// CacheTtl は HLC ベース TTL のために再利用する（wall-clock TTL 禁止規約）
import type { CacheTtl } from "./cache.js";

/**
 * PolicyPath は OPA ポリシーのパスを宣言する型。
 * OPA の decision path（"data.k1s0.authz.allow" 等のドット記法）を表す。
 */
// PolicyPath 型定義（ブランデッド型で誤用を防ぐ）
export type PolicyPath = string & { readonly __brand: "PolicyPath" };

/**
 * makePolicyPath は PolicyPath を生成するファクトリ関数。
 * 文字列から PolicyPath 型に変換する。
 */
// makePolicyPath ファクトリ関数
export function makePolicyPath(path: string): PolicyPath {
  // string を PolicyPath としてキャストして返す
  return path as PolicyPath;
}

/**
 * PolicyInput は OPA ポリシーへの入力データを宣言する型。
 * OPA の input document に相当する（Record<string, unknown> で表現する）。
 */
// PolicyInput 型定義
export type PolicyInput = Readonly<Record<string, unknown>>;

/**
 * PolicyResult はポリシー評価結果を宣言する型。
 * OPA の decision result を Library 独自語彙で表現する。
 */
// PolicyResult 型定義
export interface PolicyResult {
  // allowed: 評価結果（true = 許可 / false = 拒否）
  readonly allowed: boolean;
  // reason: 判断理由（"insufficient_scope" / "tenant_mismatch" 等）
  readonly reason?: string | undefined;
  // details: 追加詳細情報（監査ログ / デバッグ用）
  readonly details?: Readonly<Record<string, unknown>> | undefined;
}

/**
 * PolicyBundle はポリシーのバンドル情報を宣言する型。
 * OPA の Bundle（複数ポリシーファイルのアーカイブ）を Library 独自語彙で表現する。
 */
// PolicyBundle 型定義
export interface PolicyBundle {
  // bundlePath: バンドルの取得元 URL または Object Storage パス
  readonly bundlePath: string;
  // revision: バンドルのリビジョン（git commit hash 等）
  readonly revision: string;
}

/**
 * PolicyValidateOptions はポリシー検証オプションを宣言する型。
 */
// PolicyValidateOptions 型定義
export interface PolicyValidateOptions {
  // strictMode: 厳格モード（未定義変数参照等をエラーとして扱う）
  readonly strictMode?: boolean | undefined;
  // capabilities: 使用可能な OPA 組み込み関数の制限（未指定 = 全組み込みを許可する）
  readonly capabilities?: readonly string[] | undefined;
}

/**
 * RuleEngineClient は Rule Engine の L1+ 抽象 interface を宣言する。
 * OPA（Open Policy Agent）の full API を Library 独自語彙で表現する。
 * OSS 型（@open-policy-agent/opa-wasm 等）を一切含まない。
 * AuthContext 伝播を強制する（tenant 分離 + 監査に必須）。
 */
// RuleEngineClient インターフェース定義
export interface RuleEngineClient {
  /**
   * evaluate はポリシーを評価して結果を返す（Unary 評価）。
   * path は評価するポリシーパス（"data.k1s0.authz.allow" 等）。
   * input はポリシーへの入力データ（AuthContext のフィールドは実装が自動注入する）。
   */
  // evaluate メソッド: ポリシーを評価する
  evaluate(path: PolicyPath, input: PolicyInput): Promise<PolicyResult>;

  /**
   * evaluateToAny はポリシーを評価して任意型の結果を返す（ルール結果が bool 以外の場合）。
   */
  // evaluateToAny メソッド: 任意型の結果を返す評価
  evaluateToAny(path: PolicyPath, input: PolicyInput): Promise<unknown>;

  /**
   * batchEvaluate は複数のポリシーパスを一括評価する。
   * paths は評価するポリシーパスの配列、input は全パスに共通の入力。
   */
  // batchEvaluate メソッド: 複数ポリシーを一括評価する
  batchEvaluate(
    paths: readonly PolicyPath[],
    input: PolicyInput,
  ): Promise<readonly PolicyResult[]>;

  /**
   * loadBundle は OPA ポリシーバンドルをロードする（hot reload 対応）。
   */
  // loadBundle メソッド: ポリシーバンドルをロードする
  loadBundle(bundle: PolicyBundle): Promise<void>;

  /**
   * validatePolicy はポリシー文字列の構文 / 意味論的正当性を検証する（デプロイ前の CI 用）。
   * policySource は Rego ポリシーソースコード文字列。
   */
  // validatePolicy メソッド: ポリシーの正当性を検証する
  validatePolicy(policySource: string, opts?: PolicyValidateOptions): Promise<void>;

  /**
   * listPolicies は現在ロードされているポリシーのパス一覧を返す。
   */
  // listPolicies メソッド: ポリシーパス一覧を取得する
  listPolicies(): Promise<readonly PolicyPath[]>;
}

/**
 * CachedRuleEngineClient はポリシー評価結果をキャッシュする L1+ 拡張 interface を宣言する。
 * 高頻度な認可チェックのレイテンシを改善するためのキャッシュレイヤー。
 * wall-clock TTL 禁止規約に準拠して HLC ベースの TTL のみを受け付ける。
 */
// CachedRuleEngineClient インターフェース定義
export interface CachedRuleEngineClient extends RuleEngineClient {
  /**
   * evaluateCached はキャッシュ付きでポリシーを評価する。
   * cacheKey はキャッシュキー（tenant_id + policy_path + input hash で構成を推奨する）。
   * ttl は HLC ベースのキャッシュ有効期限（未指定 = キャッシュしない）。
   */
  // evaluateCached メソッド: キャッシュ付きでポリシーを評価する
  evaluateCached(
    path: PolicyPath,
    input: PolicyInput,
    cacheKey: string,
    ttl?: CacheTtl,
  ): Promise<PolicyResult>;

  /**
   * invalidateCache は指定キーのキャッシュを無効化する（ポリシー更新時に呼び出す）。
   */
  // invalidateCache メソッド: 指定キーのキャッシュを無効化する
  invalidateCache(cacheKey: string): Promise<void>;

  /**
   * invalidateAllCache は全キャッシュを無効化する（バンドル更新時に使用する）。
   */
  // invalidateAllCache メソッド: 全キャッシュを無効化する
  invalidateAllCache(): Promise<void>;
}

/**
 * PolicyAuditEntry はポリシー評価の監査ログエントリを宣言する型。
 * OPA の decision log を Library 独自語彙で表現する。
 */
// PolicyAuditEntry 型定義
export interface PolicyAuditEntry {
  // decisionId: 評価の識別子（UUID v7 形式を推奨する）
  readonly decisionId: string;
  // path: 評価したポリシーパス
  readonly path: PolicyPath;
  // input: 評価に使用した入力データ（PII を除外した safe 版）
  readonly input: PolicyInput;
  // result: 評価結果
  readonly result: PolicyResult;
  // tenantId: 評価を行ったテナント識別子
  readonly tenantId: string;
  // timestampTick: 評価時刻（HLC tick 値: wall-clock TTL 禁止規約に準拠する）
  readonly timestampTick: bigint;
}
