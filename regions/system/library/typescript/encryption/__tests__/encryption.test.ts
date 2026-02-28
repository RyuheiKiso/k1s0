import { describe, it, expect } from 'vitest';
import {
  generateKey,
  encrypt,
  decrypt,
  hashPassword,
  verifyPassword,
  generateRsaKeyPair,
  rsaEncrypt,
  rsaDecrypt,
} from '../src/index.js';

describe('generateKey', () => {
  it('32バイトのバッファを返す', () => {
    const key = generateKey();
    expect(key).toBeInstanceOf(Buffer);
    expect(key.length).toBe(32);
  });

  it('毎回異なるキーを生成する', () => {
    const k1 = generateKey();
    const k2 = generateKey();
    expect(k1.equals(k2)).toBe(false);
  });
});

describe('encrypt / decrypt', () => {
  it('暗号化と復号が可逆である', () => {
    const key = generateKey();
    const plaintext = 'Hello, World!';
    const ciphertext = encrypt(key, plaintext);
    expect(decrypt(key, ciphertext)).toBe(plaintext);
  });

  it('同じ平文でも異なる暗号文を生成する', () => {
    const key = generateKey();
    const c1 = encrypt(key, 'test');
    const c2 = encrypt(key, 'test');
    expect(c1).not.toBe(c2);
  });

  it('異なるキーでは復号に失敗する', () => {
    const k1 = generateKey();
    const k2 = generateKey();
    const ciphertext = encrypt(k1, 'secret');
    expect(() => decrypt(k2, ciphertext)).toThrow();
  });
});

describe('hashPassword / verifyPassword', () => {
  it('正しいパスワードで検証成功する', async () => {
    const hash = await hashPassword('mypassword');
    expect(await verifyPassword('mypassword', hash)).toBe(true);
  });

  it('誤ったパスワードで検証失敗する', async () => {
    const hash = await hashPassword('mypassword');
    expect(await verifyPassword('wrong', hash)).toBe(false);
  });

  it('同じパスワードでも異なるハッシュを生成する', async () => {
    const h1 = await hashPassword('test');
    const h2 = await hashPassword('test');
    expect(h1).not.toBe(h2);
  });
});

describe('RSA encryption', () => {
  it('暗号化と復号が可逆である', () => {
    const { publicKey, privateKey } = generateRsaKeyPair();
    const plaintext = Buffer.from('hello RSA-OAEP');
    const ciphertext = rsaEncrypt(publicKey, plaintext);
    const decrypted = rsaDecrypt(privateKey, ciphertext);
    expect(decrypted.toString()).toBe('hello RSA-OAEP');
  });

  it('異なるキーでは復号に失敗する', () => {
    const { publicKey } = generateRsaKeyPair();
    const { privateKey: wrongPriv } = generateRsaKeyPair();
    const ciphertext = rsaEncrypt(publicKey, Buffer.from('secret'));
    expect(() => rsaDecrypt(wrongPriv, ciphertext)).toThrow();
  });
});
