import { describe, it, expect } from 'vitest';
import { InMemoryFileClient, FileClientError } from '../src/index.js';

describe('InMemoryFileClient', () => {
  it('アップロードURLを生成する', async () => {
    const client = new InMemoryFileClient();
    const url = await client.generateUploadUrl('uploads/test.png', 'image/png', 3600000);
    expect(url.url).toContain('uploads/test.png');
    expect(url.method).toBe('PUT');
  });

  it('ダウンロードURLを生成する', async () => {
    const client = new InMemoryFileClient();
    await client.generateUploadUrl('uploads/test.png', 'image/png', 3600000);
    const url = await client.generateDownloadUrl('uploads/test.png', 300000);
    expect(url.url).toContain('uploads/test.png');
    expect(url.method).toBe('GET');
  });

  it('存在しないファイルのダウンロードURLでエラー', async () => {
    const client = new InMemoryFileClient();
    await expect(client.generateDownloadUrl('nonexistent.txt', 300000)).rejects.toThrow(FileClientError);
  });

  it('ファイルを削除する', async () => {
    const client = new InMemoryFileClient();
    await client.generateUploadUrl('uploads/test.png', 'image/png', 3600000);
    await client.delete('uploads/test.png');
    await expect(client.getMetadata('uploads/test.png')).rejects.toThrow(FileClientError);
  });

  it('メタデータを取得する', async () => {
    const client = new InMemoryFileClient();
    await client.generateUploadUrl('uploads/test.png', 'image/png', 3600000);
    const meta = await client.getMetadata('uploads/test.png');
    expect(meta.path).toBe('uploads/test.png');
    expect(meta.contentType).toBe('image/png');
  });

  it('プレフィックスでファイルを一覧取得する', async () => {
    const client = new InMemoryFileClient();
    await client.generateUploadUrl('uploads/a.png', 'image/png', 3600000);
    await client.generateUploadUrl('uploads/b.jpg', 'image/jpeg', 3600000);
    await client.generateUploadUrl('other/c.txt', 'text/plain', 3600000);
    const files = await client.list('uploads/');
    expect(files).toHaveLength(2);
  });

  it('ファイルをコピーする', async () => {
    const client = new InMemoryFileClient();
    await client.generateUploadUrl('uploads/test.png', 'image/png', 3600000);
    await client.copy('uploads/test.png', 'archive/test.png');
    const meta = await client.getMetadata('archive/test.png');
    expect(meta.contentType).toBe('image/png');
    expect(meta.path).toBe('archive/test.png');
  });

  it('存在しないファイルのコピーでエラー', async () => {
    const client = new InMemoryFileClient();
    await expect(client.copy('nonexistent.txt', 'dest.txt')).rejects.toThrow(FileClientError);
  });

  it('保存ファイル一覧が初期状態で空', () => {
    const client = new InMemoryFileClient();
    expect(client.getStoredFiles()).toHaveLength(0);
  });
});
