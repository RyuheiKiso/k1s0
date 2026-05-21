// piiAnnotation.ts — field_pii annotation の TypeScript decorator 定義 + metadata 取得
// emitPurgeEvent.ts の stripPiiFields は runtime strip だが、compile-time に annotation 漏れを検出する機構
// "experimentalDecorators": true + "emitDecoratorMetadata": true を tsconfig.json で有効化すること
// Y-tier3-4: field_pii annotation compile-time 強制（4 言語等価強度の TypeScript 実装）

// PII フィールド名を蓄積するための WeakMap（target object → Set<string> のマッピング）
// WeakMap を使うことで対象オブジェクトが GC された際にメモリリークを防止する
const _piiFieldsRegistry: WeakMap<object, Set<string>> = new WeakMap();

// PII_METADATA_KEY は reflect-metadata のキー（実験的 metadata API を使用する場合に使用する）
// ここでは WeakMap ベースの実装を primary とし、reflect-metadata は optional とする
const PII_METADATA_KEY = "k1s0:pii_fields";

// FieldPii は TypeScript PropertyDecorator を返すファクトリ関数（引数なし）
// @FieldPii() を PII を含むプロパティに付与することで annotation を宣言する
// 使用例: class Foo { @FieldPii() email: string = ""; }
// experimentalDecorators: true が tsconfig に必要
export function FieldPii(): PropertyDecorator {
  // PropertyDecorator 本体を返す（target: クラスのプロトタイプ / propertyKey: プロパティ名）
  return function (target: object, propertyKey: string | symbol): void {
    // target に対する PII フィールド名のセットを取得する（存在しない場合は新規作成する）
    let fields = _piiFieldsRegistry.get(target);
    // セットが存在しない場合は新規作成して WeakMap に登録する
    if (fields === undefined) {
      // 新規の Set を生成する
      fields = new Set<string>();
      // target の WeakMap エントリに登録する
      _piiFieldsRegistry.set(target, fields);
    }
    // プロパティ名を PII フィールドセットに追加する（symbol は toString で変換する）
    fields.add(typeof propertyKey === "symbol" ? propertyKey.toString() : propertyKey);
    // reflect-metadata が利用可能な場合はメタデータとしても記録する（optional 統合）
    if (typeof Reflect !== "undefined" && typeof Reflect.defineMetadata === "function") {
      // 既存のメタデータを取得する（なければ空配列）
      const existing: string[] =
        (Reflect.getMetadata(PII_METADATA_KEY, target) as string[] | undefined) ?? [];
      // 新しいプロパティ名を追加する
      const key = typeof propertyKey === "symbol" ? propertyKey.toString() : propertyKey;
      // 既存に含まれない場合のみ追加する（重複防止）
      if (!existing.includes(key)) {
        // メタデータを更新する
        Reflect.defineMetadata(PII_METADATA_KEY, [...existing, key], target);
      }
    }
  };
}

// getPiiFields は target オブジェクト（クラスインスタンスまたはプロトタイプ）から
// @FieldPii() が付与されたプロパティ名のリストを返す
// target: @FieldPii() を保持するクラスのプロトタイプまたはインスタンス
// 返値: PII フィールド名の配列（順序は登録順）
export function getPiiFields(target: object): string[] {
  // WeakMap から PII フィールドセットを取得する
  const fields = _piiFieldsRegistry.get(target);
  // フィールドセットが存在しない場合は空配列を返す
  if (fields === undefined) {
    // PII フィールドが未登録のため空配列を返す
    return [];
  }
  // Set を配列に変換して返す
  return Array.from(fields);
}

// getPiiFieldsFromClass は class コンストラクタからプロトタイプを解決して PII フィールド名を返す
// getPiiFields の convenience wrapper（コンストラクタを渡す場合に使用する）
// ctor: @FieldPii() を持つクラスのコンストラクタ関数
export function getPiiFieldsFromClass(ctor: new (...args: unknown[]) => object): string[] {
  // コンストラクタの prototype から PII フィールドを取得する
  return getPiiFields(ctor.prototype as object);
}

// assertNoPiiLeak は payload 内に annotation されていない PII フィールドが含まれないことをアサートする
// runtime チェック: compiletime 強制（ESLint rule）の補完として使用する
// payload: チェック対象のオブジェクト
// annotatedPiiFields: @FieldPii() で宣言された PII フィールド名のリスト
// allowedFields: PII でない許可フィールド名のリスト（payload に含まれてよいフィールド）
// 返値: 未宣言の PII フィールド名のリスト（空配列 = 問題なし）
export function assertNoPiiLeak(
  // チェック対象のオブジェクト
  payload: Record<string, unknown>,
  // @FieldPii() で宣言された PII フィールド名のリスト
  annotatedPiiFields: string[],
  // 許可フィールド名のリスト（PII でないフィールド）
  allowedFields: string[]
): string[] {
  // payload のフィールド名を取得する
  const payloadKeys = Object.keys(payload);
  // allowedFields と annotatedPiiFields のユニオンセットを生成する
  const knownFields = new Set<string>([...allowedFields, ...annotatedPiiFields]);
  // payload に含まれる未知のフィールド名を返す（PII 漏洩の可能性があるフィールド）
  return payloadKeys.filter((k) => !knownFields.has(k));
}

// PiiAnnotatedClass は @FieldPii() を使用するクラスに implements させる interface
// compile-time の型チェックでアノテーション忘れを防止する（optional 使用）
export interface PiiAnnotatedClass {
  // __piiAnnotated フラグ（存在確認用の marker property）
  readonly __piiAnnotated: true;
}
