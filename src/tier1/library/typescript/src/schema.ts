/**
 * schema.ts — k1s0 tier1 Library TypeScript 実装: Schema Registry の L2* interface
 * 12_スキーマレジストリ適合仕様.md §SchemaRegistryClient（族内共通 L2*）に準拠する。
 * Confluent Schema Registry / Apicurio 等の OSS API を Library 独自語彙に翻訳する L2* facade を宣言する。
 * 公開シグネチャに OSS 型を一切含まない。
 */

/**
 * SchemaFormat はスキーマフォーマットを宣言する enum。
 * Confluent Schema Registry の schema type に準拠した Library 独自語彙とする。
 */
// SchemaFormat 列挙型定義
export const enum SchemaFormat {
  // Avro: Apache Avro フォーマット
  Avro = "avro",
  // Json: JSON Schema フォーマット
  Json = "json",
  // Protobuf: Protocol Buffers フォーマット
  Protobuf = "protobuf",
}

/**
 * CompatibilityMode はスキーマ互換性モードを宣言する enum。
 * Confluent Schema Registry の compatibility 設定に準拠した Library 独自語彙とする。
 */
// CompatibilityMode 列挙型定義
export const enum CompatibilityMode {
  // Backward: 後方互換（新スキーマで旧データが読める）
  Backward = "backward",
  // Forward: 前方互換（旧スキーマで新データが読める）
  Forward = "forward",
  // Full: 双方向互換（Backward + Forward）
  Full = "full",
  // None: 互換性チェックなし（開発環境のみ使用可）
  None = "none",
}

/**
 * SchemaReference は別のスキーマを参照する定義を宣言する型。
 * Confluent Schema Registry の SchemaReference に準拠した Library 独自語彙とする。
 */
// SchemaReference 型定義
export interface SchemaReference {
  // name: 参照名（Protobuf の import パス等）
  readonly name: string;
  // subject: 参照先 Subject 名
  readonly subject: string;
  // version: 参照するバージョン
  readonly version: number;
}

/**
 * SchemaInfo は Schema Registry に登録されたスキーマ情報を宣言する型。
 * OSS の SchemaInfo レスポンス型を露出せず Library 独自語彙で表現する。
 */
// SchemaInfo 型定義
export interface SchemaInfo {
  // id: Schema Registry が発行したスキーマ ID（グローバルユニーク）
  readonly id: bigint;
  // subject: スキーマが属する Subject 名（"{topic}-value" / "{topic}-key" 形式）
  readonly subject: string;
  // version: スキーマバージョン（1 始まりの単調増加整数）
  readonly version: number;
  // format: スキーマフォーマット（Avro / JSON / Protobuf）
  readonly format: SchemaFormat;
  // schema: スキーマ定義文字列（Avro JSON / JSON Schema / .proto 等）
  readonly schema: string;
  // references: 参照するスキーマの一覧（Protobuf import 等）
  readonly references: readonly SchemaReference[];
}

/**
 * SchemaRegistryClient は Schema Registry の L2* 抽象 interface を宣言する。
 * Confluent Schema Registry / Apicurio / AWS Glue Schema Registry 等を抽象化する。
 * OSS 型を引数・戻り値に一切含まない。
 */
// SchemaRegistryClient インターフェース定義
export interface SchemaRegistryClient {
  /**
   * register は Subject にスキーマを登録して ID を返す。
   * 同一スキーマが既に登録されている場合は既存 ID を返す（idempotent 操作）。
   */
  // register メソッド: スキーマを登録する
  register(subject: string, schema: string, format: SchemaFormat): Promise<bigint>;

  /**
   * getById は ID に対応するスキーマ情報を取得する。
   * スキーマが存在しない場合は null を返す（エラーと区別する）。
   */
  // getById メソッド: ID でスキーマを取得する
  getById(id: bigint): Promise<SchemaInfo | null>;

  /**
   * getLatest は Subject の最新バージョンのスキーマ情報を取得する。
   */
  // getLatest メソッド: Subject の最新スキーマを取得する
  getLatest(subject: string): Promise<SchemaInfo | null>;

  /**
   * getByVersion は Subject の指定バージョンのスキーマ情報を取得する。
   */
  // getByVersion メソッド: Subject のバージョン指定でスキーマを取得する
  getByVersion(subject: string, version: number): Promise<SchemaInfo | null>;

  /**
   * listSubjects は登録されている Subject 名の一覧を返す。
   */
  // listSubjects メソッド: Subject 名の一覧を取得する
  listSubjects(): Promise<readonly string[]>;

  /**
   * listVersions は Subject のバージョン番号一覧を返す。
   */
  // listVersions メソッド: Subject のバージョン番号一覧を取得する
  listVersions(subject: string): Promise<readonly number[]>;

  /**
   * checkCompatibility は新しいスキーマが Subject の既存スキーマと互換性があるかチェックする。
   */
  // checkCompatibility メソッド: スキーマ互換性をチェックする
  checkCompatibility(subject: string, schema: string, format: SchemaFormat): Promise<boolean>;

  /**
   * setCompatibilityMode は Subject のスキーマ互換性モードを設定する。
   */
  // setCompatibilityMode メソッド: スキーマ互換性モードを設定する
  setCompatibilityMode(subject: string, mode: CompatibilityMode): Promise<void>;

  /**
   * deleteSubject は Subject を全バージョン削除する。
   * permanent=true の場合は hard delete（復元不可）、false は soft delete。
   */
  // deleteSubject メソッド: Subject を削除する
  deleteSubject(subject: string, permanent: boolean): Promise<void>;
}

/**
 * SchemaCodec はスキーマを使ってメッセージをエンコード / デコードする L2* interface を宣言する。
 * Confluent Wire Format（magic byte + schema ID）を Library 独自語彙で抽象化する。
 */
// SchemaCodec インターフェース定義
export interface SchemaCodec {
  /**
   * encode はスキーマ ID とデータバイト列から Confluent Wire Format のバイト列を生成する。
   * schemaId は Schema Registry で登録されたスキーマ ID（magic byte 付きで先頭に埋め込む）。
   * data はスキーマに従ってシリアライズ済みのバイト列（Avro binary / Protobuf wire 等）。
   */
  // encode メソッド: Confluent Wire Format にエンコードする
  encode(schemaId: bigint, data: Uint8Array): Promise<Uint8Array>;

  /**
   * decode は Confluent Wire Format のバイト列からスキーマ ID とデータバイト列を取り出す。
   * 戻り値は [schemaId, data] のタプル。
   */
  // decode メソッド: Confluent Wire Format からデコードする
  decode(wireFormatBytes: Uint8Array): Promise<[bigint, Uint8Array]>;
}
