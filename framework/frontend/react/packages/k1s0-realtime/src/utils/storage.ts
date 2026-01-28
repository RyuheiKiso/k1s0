/**
 * localStorage ラッパー
 */

/**
 * JSON データの永続化ユーティリティ
 */
export const storage = {
  /**
   * データを取得する
   * @param key - ストレージキー
   * @returns パースされたデータ、またはキーが存在しない場合は null
   */
  get<T>(key: string): T | null {
    try {
      const raw = localStorage.getItem(key);
      if (raw === null) return null;
      return JSON.parse(raw) as T;
    } catch {
      return null;
    }
  },

  /**
   * データを保存する
   * @param key - ストレージキー
   * @param value - 保存するデータ
   */
  set<T>(key: string, value: T): void {
    try {
      localStorage.setItem(key, JSON.stringify(value));
    } catch {
      // ストレージ容量超過時は無視
    }
  },

  /**
   * データを削除する
   * @param key - ストレージキー
   */
  remove(key: string): void {
    try {
      localStorage.removeItem(key);
    } catch {
      // 無視
    }
  },
};
