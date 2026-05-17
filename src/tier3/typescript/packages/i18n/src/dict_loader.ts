// k1s0 tier3 i18n 辞書ローダー
// サポートロケールの翻訳辞書を dynamic import で非同期に読み込む
// 未定義 key は CI fail（11_クライアント状態適合仕様 + tier3 コーディングポリシーの規律）

// サポートロケール識別子の型（v1 は ja_JP / en_US のみ）
// ファイルシステム上の辞書ファイル名（dict/ja_JP.json 等）と一致させる
export type DictLocale = "ja_JP" | "en_US";

// 翻訳辞書の型（key → 翻訳文字列のフラットマップ）
export type TranslationDict = Record<string, string>;

// ロケール別辞書キャッシュ（同一ロケールの重複ロードを防ぐ）
// WeakRef ではなく Module-level Map を使用する（モジュールライフタイム内でキャッシュする）
const dictCache = new Map<DictLocale, TranslationDict>();

// 辞書ファイルを dynamic import で読み込む
// locale: 読み込むロケール識別子（"ja_JP" または "en_US"）
// 戻り値: 翻訳辞書（key → 文字列のフラットマップ）
// 例外: 辞書ファイルが存在しない場合は Error をスローする
export async function loadDict(locale: DictLocale): Promise<TranslationDict> {
    // キャッシュにヒットした場合はキャッシュ値を返す（重複ロードを防ぐ）
    const cached = dictCache.get(locale);
    if (cached !== undefined) {
        // キャッシュから辞書を返す
        return cached;
    }
    // ロケールに応じた辞書ファイルを dynamic import で読み込む
    // バンドラー（Vite / webpack）が静的解析できるよう文字列リテラルを使用する
    let dict: TranslationDict;
    // ロケール識別子でブランチを分岐する（switch で網羅性を確認する）
    switch (locale) {
        case "ja_JP": {
            // 日本語辞書ファイルを読み込む（相対パスは dist からの相対パス）
            const module = await import("../dict/ja_JP.json");
            // JSON モジュールのデフォルトエクスポートを辞書として使用する
            dict = module.default as TranslationDict;
            break;
        }
        case "en_US": {
            // 英語辞書ファイルを読み込む
            const module = await import("../dict/en_US.json");
            // JSON モジュールのデフォルトエクスポートを辞書として使用する
            dict = module.default as TranslationDict;
            break;
        }
        default: {
            // 網羅性チェック（新ロケール追加時はコンパイルエラーにする）
            const _exhaustive: never = locale;
            // 到達不能コードであることを TypeScript に伝える
            throw new Error(`サポート外のロケールです: ${String(_exhaustive)}`);
        }
    }
    // 読み込んだ辞書をキャッシュに格納する（次回からキャッシュを返す）
    dictCache.set(locale, dict);
    // 辞書を返す
    return dict;
}

// 辞書キャッシュをクリアする（テスト用途のみ。本番コードでは呼び出さない）
// テストごとに独立した状態を保証するために使用する
export function clearDictCache(): void {
    // キャッシュを全消去する（テスト間の状態汚染を防ぐ）
    dictCache.clear();
}

// 辞書から key を安全に取得する（未定義 key は fallback を返す）
// dict: 翻訳辞書
// key: 取得する翻訳キー
// fallback: キーが存在しない場合のフォールバック文字列（省略時はキー自体を返す）
export function dictGet(dict: TranslationDict, key: string, fallback?: string): string {
    // dict に key が存在する場合は翻訳文字列を返す
    const value = dict[key];
    // key が未定義の場合は fallback または key 自体を返す（CI の未使用 key 検出は別途行う）
    return value !== undefined ? value : (fallback ?? key);
}
