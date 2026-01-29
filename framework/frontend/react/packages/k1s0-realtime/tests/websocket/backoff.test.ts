/**
 * バックオフ計算のテスト
 */

import { describe, expect, it } from 'vitest';
import { calculateBackoff, addJitter } from '../../src/utils/backoff.js';

describe('calculateBackoff', () => {
  describe('linear backoff', () => {
    it('試行回数に比例して遅延が増加する', () => {
      expect(calculateBackoff(0, 1000, 30000, 'linear')).toBe(1000);
      expect(calculateBackoff(1, 1000, 30000, 'linear')).toBe(2000);
      expect(calculateBackoff(2, 1000, 30000, 'linear')).toBe(3000);
      expect(calculateBackoff(4, 1000, 30000, 'linear')).toBe(5000);
    });

    it('最大遅延を超えない', () => {
      expect(calculateBackoff(100, 1000, 30000, 'linear')).toBe(30000);
    });
  });

  describe('exponential backoff', () => {
    it('試行回数の指数関数で遅延が増加する', () => {
      expect(calculateBackoff(0, 1000, 60000, 'exponential')).toBe(1000);
      expect(calculateBackoff(1, 1000, 60000, 'exponential')).toBe(2000);
      expect(calculateBackoff(2, 1000, 60000, 'exponential')).toBe(4000);
      expect(calculateBackoff(3, 1000, 60000, 'exponential')).toBe(8000);
    });

    it('最大遅延を超えない', () => {
      expect(calculateBackoff(20, 1000, 30000, 'exponential')).toBe(30000);
    });
  });
});

describe('addJitter', () => {
  it('元の値の 75%~125% の範囲内に収まる', () => {
    const delay = 1000;
    for (let i = 0; i < 100; i++) {
      const result = addJitter(delay);
      expect(result).toBeGreaterThanOrEqual(750);
      expect(result).toBeLessThanOrEqual(1250);
    }
  });

  it('整数値を返す', () => {
    const result = addJitter(1000);
    expect(Number.isInteger(result)).toBe(true);
  });
});
