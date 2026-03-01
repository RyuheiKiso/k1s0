import {
  createCipheriv,
  createDecipheriv,
  randomBytes,
  generateKeyPairSync,
  publicEncrypt,
  privateDecrypt,
  constants,
} from 'crypto';
import argon2 from 'argon2';

export function generateKey(): Buffer {
  return randomBytes(32);
}

export function encrypt(key: Buffer, plaintext: string): string {
  const iv = randomBytes(12);
  const cipher = createCipheriv('aes-256-gcm', key, iv);
  const encrypted = Buffer.concat([cipher.update(plaintext, 'utf8'), cipher.final()]);
  const authTag = cipher.getAuthTag();
  const combined = Buffer.concat([iv, authTag, encrypted]);
  return combined.toString('base64');
}

export function decrypt(key: Buffer, ciphertext: string): string {
  const combined = Buffer.from(ciphertext, 'base64');
  const iv = combined.subarray(0, 12);
  const authTag = combined.subarray(12, 28);
  const encrypted = combined.subarray(28);
  const decipher = createDecipheriv('aes-256-gcm', key, iv);
  decipher.setAuthTag(authTag);
  const decrypted = Buffer.concat([decipher.update(encrypted), decipher.final()]);
  return decrypted.toString('utf8');
}

export async function hashPassword(password: string): Promise<string> {
  return argon2.hash(password, {
    type: argon2.argon2id,
    memoryCost: 19456,
    timeCost: 2,
    parallelism: 1,
  });
}

export async function verifyPassword(password: string, hash: string): Promise<boolean> {
  return argon2.verify(hash, password);
}

export function generateRsaKeyPair(): { publicKey: string; privateKey: string } {
  const { publicKey, privateKey } = generateKeyPairSync('rsa', {
    modulusLength: 2048,
    publicKeyEncoding: { type: 'spki', format: 'pem' },
    privateKeyEncoding: { type: 'pkcs8', format: 'pem' },
  });
  return { publicKey, privateKey };
}

export function rsaEncrypt(publicKeyPem: string, plaintext: Buffer): Buffer {
  return publicEncrypt(
    { key: publicKeyPem, padding: constants.RSA_PKCS1_OAEP_PADDING, oaepHash: 'sha256' },
    plaintext,
  );
}

export function rsaDecrypt(privateKeyPem: string, ciphertext: Buffer): Buffer {
  return privateDecrypt(
    { key: privateKeyPem, padding: constants.RSA_PKCS1_OAEP_PADDING, oaepHash: 'sha256' },
    ciphertext,
  );
}
