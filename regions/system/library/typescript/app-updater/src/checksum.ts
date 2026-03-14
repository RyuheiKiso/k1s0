import { createHash } from 'node:crypto';
import { createReadStream } from 'node:fs';
import { ChecksumError } from './error.js';

/** ファイルの SHA-256 チェックサムを計算・検証するユーティリティ */
export class ChecksumVerifier {
  /** ファイルの SHA-256 チェックサムを計算して16進数文字列で返す */
  static async calculate(filePath: string): Promise<string> {
    return new Promise((resolve, reject) => {
      const hash = createHash('sha256');
      const stream = createReadStream(filePath);
      stream.on('data', (chunk) => hash.update(chunk));
      stream.on('end', () => resolve(hash.digest('hex')));
      stream.on('error', reject);
    });
  }

  /** ファイルのチェックサムが期待値と一致するかを検証する */
  static async verify(filePath: string, expected: string): Promise<boolean> {
    const actual = await ChecksumVerifier.calculate(filePath);
    return actual === expected.toLowerCase();
  }

  /** ファイルのチェックサムが期待値と一致しない場合は例外をスローする */
  static async verifyOrThrow(filePath: string, expected: string): Promise<void> {
    const verified = await ChecksumVerifier.verify(filePath, expected);
    if (!verified) {
      throw new ChecksumError('File checksum did not match.');
    }
  }
}
