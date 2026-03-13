import { describe, it, expect } from 'vitest';
import { writeFileSync, mkdtempSync, rmSync } from 'node:fs';
import { join } from 'node:path';
import { tmpdir } from 'node:os';
import { ChecksumVerifier, ChecksumError } from '../src/index.js';

describe('ChecksumVerifier', () => {
  let tempDir: string;
  let tempFile: string;

  const setupTempFile = (content: string) => {
    tempDir = mkdtempSync(join(tmpdir(), 'checksum-test-'));
    tempFile = join(tempDir, 'test.txt');
    writeFileSync(tempFile, content, 'utf-8');
  };

  const cleanup = () => {
    if (tempDir) {
      rmSync(tempDir, { recursive: true, force: true });
    }
  };

  it('ファイルのSHA-256チェックサムを計算できる', async () => {
    setupTempFile('hello world');
    try {
      const checksum = await ChecksumVerifier.calculate(tempFile);
      // SHA-256 of "hello world"
      expect(checksum).toBe(
        'b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9',
      );
    } finally {
      cleanup();
    }
  });

  it('正しいチェックサムでverifyがtrueを返す', async () => {
    setupTempFile('hello world');
    try {
      const result = await ChecksumVerifier.verify(
        tempFile,
        'b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9',
      );
      expect(result).toBe(true);
    } finally {
      cleanup();
    }
  });

  it('不正なチェックサムでverifyがfalseを返す', async () => {
    setupTempFile('hello world');
    try {
      const result = await ChecksumVerifier.verify(tempFile, 'invalid-checksum');
      expect(result).toBe(false);
    } finally {
      cleanup();
    }
  });

  it('大文字のチェックサムでも正しく検証できる', async () => {
    setupTempFile('hello world');
    try {
      const result = await ChecksumVerifier.verify(
        tempFile,
        'B94D27B9934D3E08A52E52D7DA7DABFAC484EFE37A5380EE9088F7ACE2EFCDE9',
      );
      expect(result).toBe(true);
    } finally {
      cleanup();
    }
  });

  it('正しいチェックサムでverifyOrThrowが成功する', async () => {
    setupTempFile('hello world');
    try {
      await expect(
        ChecksumVerifier.verifyOrThrow(
          tempFile,
          'b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9',
        ),
      ).resolves.toBeUndefined();
    } finally {
      cleanup();
    }
  });

  it('不正なチェックサムでverifyOrThrowがChecksumErrorを投げる', async () => {
    setupTempFile('hello world');
    try {
      await expect(
        ChecksumVerifier.verifyOrThrow(tempFile, 'wrong-checksum'),
      ).rejects.toThrow(ChecksumError);
    } finally {
      cleanup();
    }
  });
});
