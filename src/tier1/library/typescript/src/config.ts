/**
 * config.ts — k1s0 tier1 Library TypeScript 実装: Configuration / Feature Flag の L2* interface
 * 07_設定適合仕様.md §ConfigClient / §FeatureFlagClient（族内共通 API）に準拠する。
 * OpenFeature / LaunchDarkly 等の OSS API を Library 独自語彙に翻訳する L2* facade を宣言する。
 * 公開シグネチャに OSS 型を露出しない（ldclient 等は一切含まない）。
 */

/**
 * EvalContext は L2* Feature Flag の評価コンテキストを宣言する型。
 * OpenFeature EvaluationContext に準拠した Library 独自語彙とする。
 * tenantId / userId を必須フィールドとして持ち、追加属性は attrs に格納する。
 */
// EvalContext 型定義
export interface EvalContext {
  // tenantId: フラグ評価の対象テナント識別子（必須: tenant 分離フラグ評価に使用する）
  readonly tenantId: string;
  // userId: フラグ評価の対象ユーザー識別子（省略可能: A/B test 等で使用する）
  readonly userId?: string | undefined;
  // attrs: 追加評価属性（region / plan / custom 等）
  readonly attrs?: Readonly<Record<string, string>> | undefined;
}

/**
 * FlagEvalReason はフラグ評価理由を宣言する enum。
 * OpenFeature EvaluationReason に準拠した Library 独自語彙とする。
 */
// FlagEvalReason 列挙型定義
export const enum FlagEvalReason {
  // Static: 静的（ルールマッチなし / デフォルト値）
  Static = "static",
  // Targeting: ターゲティングルールに一致して評価した
  Targeting = "targeting",
  // Split: A/B split test により評価した
  Split = "split",
  // Default: デフォルト値にフォールバックした（エラー発生時等）
  Default = "default",
}

/**
 * BoolEvalResult は bool 型フラグの評価結果を宣言する型。
 * 値だけでなく評価理由も含めて返す（監査ログ・デバッグに使用する）。
 */
// BoolEvalResult 型定義
export interface BoolEvalResult {
  // value: 評価されたフラグ値
  readonly value: boolean;
  // reason: フラグ評価理由
  readonly reason: FlagEvalReason;
  // variant: 評価されたバリアント名
  readonly variant: string;
}

/**
 * StringEvalResult は string 型フラグの評価結果を宣言する型。
 */
// StringEvalResult 型定義
export interface StringEvalResult {
  // value: 評価されたフラグ値
  readonly value: string;
  // reason: フラグ評価理由
  readonly reason: FlagEvalReason;
  // variant: 評価されたバリアント名
  readonly variant: string;
}

/**
 * NumberEvalResult は number 型フラグの評価結果を宣言する型。
 */
// NumberEvalResult 型定義
export interface NumberEvalResult {
  // value: 評価されたフラグ値
  readonly value: number;
  // reason: フラグ評価理由
  readonly reason: FlagEvalReason;
  // variant: 評価されたバリアント名
  readonly variant: string;
}

/**
 * FeatureFlagClient は L2* Feature Flag 評価 interface を宣言する。
 * OpenFeature Provider を Library 独自語彙で抽象化する。
 * OSS 型（ldclient 等）を引数・戻り値に一切含まない。
 */
// FeatureFlagClient インターフェース定義
export interface FeatureFlagClient {
  /**
   * boolValue は bool 型フラグを評価して値を返す。
   * key はフラグキー（"feature.new-ui" 等のドット記法を推奨する）。
   * ec は評価コンテキスト（tenantId は必須: tenant 分離評価を保証する）。
   * defaultVal はフォールバック値（エラー発生時に使用する）。
   */
  // boolValue メソッド: bool フラグを評価する
  boolValue(key: string, ec: EvalContext, defaultVal: boolean): Promise<boolean>;

  /**
   * boolDetail は bool 型フラグを評価して詳細結果を返す（評価理由を含む）。
   * 監査ログ・デバッグ用途で boolValue より詳細な情報が必要な場合に使用する。
   */
  // boolDetail メソッド: bool フラグの詳細評価結果を返す
  boolDetail(key: string, ec: EvalContext, defaultVal: boolean): Promise<BoolEvalResult>;

  /**
   * stringValue は string 型フラグを評価して値を返す。
   */
  // stringValue メソッド: string フラグを評価する
  stringValue(key: string, ec: EvalContext, defaultVal: string): Promise<string>;

  /**
   * stringDetail は string 型フラグを評価して詳細結果を返す。
   */
  // stringDetail メソッド: string フラグの詳細評価結果を返す
  stringDetail(key: string, ec: EvalContext, defaultVal: string): Promise<StringEvalResult>;

  /**
   * numberValue は number 型フラグを評価して値を返す。
   */
  // numberValue メソッド: number フラグを評価する
  numberValue(key: string, ec: EvalContext, defaultVal: number): Promise<number>;
}

/**
 * ConfigValue は設定値と付随するメタデータを宣言する型。
 * OSS の config 型を露出せず Library 独自語彙で表現する。
 */
// ConfigValue 型定義
export interface ConfigValue {
  // raw: 設定の生値（文字列表現）
  readonly raw: string;
  // source: 設定の取得元（"env" / "file" / "remote" 等）
  readonly source: string;
  // version: 設定バージョン（リモート設定ストアの revision 等）
  readonly version: number;
}

/**
 * ConfigClient は L2* 設定取得 interface を宣言する。
 * Consul / etcd / ConfigMap 等の設定ストアを Library 独自語彙で抽象化する。
 * OSS 型（consul.Client / etcd.Client 等）を引数・戻り値に一切含まない。
 */
// ConfigClient インターフェース定義
export interface ConfigClient {
  /**
   * getString は string 型設定値を取得する。
   * key はドット記法のキー（"service.timeout" 等）。
   * 値が存在しない場合は defaultVal を返す。
   */
  // getString メソッド: string 設定値を取得する
  getString(key: string, defaultVal: string): Promise<string>;

  /**
   * getNumber は number 型設定値を取得する。
   */
  // getNumber メソッド: number 設定値を取得する
  getNumber(key: string, defaultVal: number): Promise<number>;

  /**
   * getBool は boolean 型設定値を取得する。
   */
  // getBool メソッド: boolean 設定値を取得する
  getBool(key: string, defaultVal: boolean): Promise<boolean>;

  /**
   * getValue は ConfigValue（メタデータ付き）で設定値を取得する。
   * バージョン管理・監査ログが必要な場合に getString の代わりに使用する。
   */
  // getValue メソッド: ConfigValue でメタデータ付き設定値を取得する
  getValue(key: string): Promise<ConfigValue | null>;

  /**
   * watch は設定キーの変更を監視して AsyncIterable でイベントを提供する。
   * AbortSignal でストリームを停止する（MemLeak 防止のため必ず signal を渡す）。
   */
  // watch メソッド: 設定変更を監視する
  watch(key: string, signal: AbortSignal): AsyncIterable<ConfigValue>;
}
