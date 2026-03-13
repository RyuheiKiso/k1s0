import { createHash } from 'node:crypto';
import { createReadStream } from 'node:fs';
import { ChecksumError } from './error.js';

export class ChecksumVerifier {
  static async calculate(filePath: string): Promise<string> {
    return new Promise((resolve, reject) => {
      const hash = createHash('sha256');
      const stream = createReadStream(filePath);
      stream.on('data', (chunk) => hash.update(chunk));
      stream.on('end', () => resolve(hash.digest('hex')));
      stream.on('error', reject);
    });
  }

  static async verify(filePath: string, expected: string): Promise<boolean> {
    const actual = await ChecksumVerifier.calculate(filePath);
    return actual === expected.toLowerCase();
  }

  static async verifyOrThrow(filePath: string, expected: string): Promise<void> {
    const verified = await ChecksumVerifier.verify(filePath, expected);
    if (!verified) {
      throw new ChecksumError('File checksum did not match.');
    }
  }
}
