/**
 * バックオフ計算ユーティリティ
 */

/**
 * バックオフ遅延を計算する
 * @param attempt - 試行回数（0始まり）
 * @param initialDelay - 初回遅延（ms）
 * @param maxDelay - 最大遅延（ms）
 * @param strategy - バックオフ戦略
 */
export function calculateBackoff(
  attempt: number,
  initialDelay: number,
  maxDelay: number,
  strategy: 'linear' | 'exponential',
): number {
  let delay: number;

  if (strategy === 'exponential') {
    delay = initialDelay * Math.pow(2, attempt);
  } else {
    delay = initialDelay * (attempt + 1);
  }

  return Math.min(delay, maxDelay);
}

/**
 * 遅延にランダムなジッターを追加する
 * ±25% の範囲でランダムに変動させる
 * @param delay - 元の遅延（ms）
 */
export function addJitter(delay: number): number {
  const jitterFactor = 0.75 + Math.random() * 0.5; // 0.75 ~ 1.25
  return Math.round(delay * jitterFactor);
}
