import { describe, it, expect } from 'vitest';
import {
  generateCodeVerifier,
  generateCodeChallenge,
  base64UrlEncode,
} from '../src/pkce';

describe('PKCE', () => {
  describe('generateCodeVerifier', () => {
    it('should generate a base64url-encoded string', () => {
      const verifier = generateCodeVerifier();
      // Base64url: only contains [A-Za-z0-9_-]
      expect(verifier).toMatch(/^[A-Za-z0-9_-]+$/);
    });

    it('should generate a string of appropriate length', () => {
      const verifier = generateCodeVerifier();
      // 32 bytes -> ~43 chars in base64url (no padding)
      expect(verifier.length).toBeGreaterThanOrEqual(40);
      expect(verifier.length).toBeLessThanOrEqual(48);
    });

    it('should generate unique values on each call', () => {
      const v1 = generateCodeVerifier();
      const v2 = generateCodeVerifier();
      expect(v1).not.toBe(v2);
    });

    it('should use the injected random function', () => {
      const mockGetRandomValues = (array: Uint8Array): Uint8Array => {
        for (let i = 0; i < array.length; i++) {
          array[i] = i;
        }
        return array;
      };
      const v1 = generateCodeVerifier(mockGetRandomValues);
      const v2 = generateCodeVerifier(mockGetRandomValues);
      expect(v1).toBe(v2);
    });

    it('should not contain +, /, or = characters', () => {
      for (let i = 0; i < 10; i++) {
        const verifier = generateCodeVerifier();
        expect(verifier).not.toContain('+');
        expect(verifier).not.toContain('/');
        expect(verifier).not.toContain('=');
      }
    });
  });

  describe('generateCodeChallenge', () => {
    it('should generate a base64url-encoded SHA-256 hash', async () => {
      const verifier = 'dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk';
      const challenge = await generateCodeChallenge(verifier);
      // base64url encoded
      expect(challenge).toMatch(/^[A-Za-z0-9_-]+$/);
    });

    it('should produce consistent output for the same input', async () => {
      const verifier = 'test-verifier-value';
      const c1 = await generateCodeChallenge(verifier);
      const c2 = await generateCodeChallenge(verifier);
      expect(c1).toBe(c2);
    });

    it('should produce different output for different inputs', async () => {
      const c1 = await generateCodeChallenge('verifier-1');
      const c2 = await generateCodeChallenge('verifier-2');
      expect(c1).not.toBe(c2);
    });

    it('should not contain +, /, or = characters', async () => {
      for (let i = 0; i < 10; i++) {
        const verifier = generateCodeVerifier();
        const challenge = await generateCodeChallenge(verifier);
        expect(challenge).not.toContain('+');
        expect(challenge).not.toContain('/');
        expect(challenge).not.toContain('=');
      }
    });

    it('should use the injected digest function', async () => {
      const mockDigest = async (
        _algorithm: string,
        _data: BufferSource,
      ): Promise<ArrayBuffer> => {
        return new Uint8Array([1, 2, 3, 4]).buffer;
      };
      const challenge = await generateCodeChallenge('test', mockDigest);
      expect(challenge).toBe(base64UrlEncode(new Uint8Array([1, 2, 3, 4])));
    });
  });

  describe('base64UrlEncode', () => {
    it('should encode an empty buffer', () => {
      expect(base64UrlEncode(new Uint8Array([]))).toBe('');
    });

    it('should encode without padding', () => {
      const encoded = base64UrlEncode(new Uint8Array([1]));
      expect(encoded).not.toContain('=');
    });

    it('should replace + with - and / with _', () => {
      // Test with values known to produce + and / in standard base64
      const encoded = base64UrlEncode(new Uint8Array([251, 255, 254]));
      expect(encoded).not.toContain('+');
      expect(encoded).not.toContain('/');
    });
  });
});
