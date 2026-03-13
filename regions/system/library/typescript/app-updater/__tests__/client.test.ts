import { describe, it, expect } from 'vitest';
import {
  compareVersions,
  determineUpdateType,
  InMemoryAppUpdater,
} from '../src/index.js';
import type { AppVersionInfo } from '../src/index.js';

describe('compareVersions', () => {
  it('等しいバージョンは0を返す', () => {
    expect(compareVersions('1.0.0', '1.0.0')).toBe(0);
  });

  it('左が大きい場合は1を返す', () => {
    expect(compareVersions('2.0.0', '1.0.0')).toBe(1);
  });

  it('左が小さい場合は-1を返す', () => {
    expect(compareVersions('1.0.0', '2.0.0')).toBe(-1);
  });

  it('マイナーバージョンの比較ができる', () => {
    expect(compareVersions('1.2.0', '1.1.0')).toBe(1);
    expect(compareVersions('1.1.0', '1.2.0')).toBe(-1);
  });

  it('パッチバージョンの比較ができる', () => {
    expect(compareVersions('1.0.2', '1.0.1')).toBe(1);
    expect(compareVersions('1.0.1', '1.0.2')).toBe(-1);
  });

  it('異なる長さのバージョンを比較できる', () => {
    expect(compareVersions('1.0', '1.0.0')).toBe(0);
    expect(compareVersions('1.0.1', '1.0')).toBe(1);
    expect(compareVersions('1.0', '1.0.1')).toBe(-1);
  });

  it('プレリリースの数値以外を除去して比較できる', () => {
    expect(compareVersions('1.0.0-beta', '1.0.0')).toBe(0);
    expect(compareVersions('1.0.0-alpha', '1.0.0-beta')).toBe(0);
  });
});

describe('determineUpdateType', () => {
  it('最低バージョン未満の場合はmandatoryを返す', () => {
    const info: AppVersionInfo = {
      latestVersion: '2.0.0',
      minimumVersion: '1.5.0',
      mandatory: false,
    };
    expect(determineUpdateType('1.0.0', info)).toBe('mandatory');
  });

  it('mandatoryフラグが立っている場合はmandatoryを返す', () => {
    const info: AppVersionInfo = {
      latestVersion: '2.0.0',
      minimumVersion: '1.0.0',
      mandatory: true,
    };
    expect(determineUpdateType('1.5.0', info)).toBe('mandatory');
  });

  it('最新バージョン未満の場合はoptionalを返す', () => {
    const info: AppVersionInfo = {
      latestVersion: '2.0.0',
      minimumVersion: '1.0.0',
      mandatory: false,
    };
    expect(determineUpdateType('1.5.0', info)).toBe('optional');
  });

  it('最新バージョンの場合はnoneを返す', () => {
    const info: AppVersionInfo = {
      latestVersion: '2.0.0',
      minimumVersion: '1.0.0',
      mandatory: false,
    };
    expect(determineUpdateType('2.0.0', info)).toBe('none');
  });

  it('最新バージョンより新しい場合はnoneを返す', () => {
    const info: AppVersionInfo = {
      latestVersion: '2.0.0',
      minimumVersion: '1.0.0',
      mandatory: false,
    };
    expect(determineUpdateType('3.0.0', info)).toBe('none');
  });
});

describe('InMemoryAppUpdater', () => {
  const defaultVersionInfo: AppVersionInfo = {
    latestVersion: '2.0.0',
    minimumVersion: '1.0.0',
    mandatory: false,
    releaseNotes: 'New features',
  };

  it('バージョン情報を取得できる', async () => {
    const updater = new InMemoryAppUpdater(defaultVersionInfo, '1.5.0');
    const info = await updater.fetchVersionInfo();
    expect(info.latestVersion).toBe('2.0.0');
    expect(info.minimumVersion).toBe('1.0.0');
    expect(info.releaseNotes).toBe('New features');
  });

  it('アップデートチェックでoptionalを返す', async () => {
    const updater = new InMemoryAppUpdater(defaultVersionInfo, '1.5.0');
    const result = await updater.checkForUpdate();
    expect(result.updateType).toBe('optional');
    expect(result.currentVersion).toBe('1.5.0');
    expect(result.latestVersion).toBe('2.0.0');
  });

  it('アップデートチェックでnoneを返す', async () => {
    const updater = new InMemoryAppUpdater(defaultVersionInfo, '2.0.0');
    const result = await updater.checkForUpdate();
    expect(result.updateType).toBe('none');
  });

  it('アップデートチェックでmandatoryを返す', async () => {
    const updater = new InMemoryAppUpdater(defaultVersionInfo, '0.5.0');
    const result = await updater.checkForUpdate();
    expect(result.updateType).toBe('mandatory');
  });

  it('バージョン情報を更新できる', async () => {
    const updater = new InMemoryAppUpdater(defaultVersionInfo, '1.5.0');
    updater.setVersionInfo({
      latestVersion: '3.0.0',
      minimumVersion: '2.0.0',
      mandatory: false,
    });
    const info = await updater.fetchVersionInfo();
    expect(info.latestVersion).toBe('3.0.0');
  });

  it('現在のバージョンを更新できる', async () => {
    const updater = new InMemoryAppUpdater(defaultVersionInfo, '1.5.0');
    updater.setCurrentVersion('2.0.0');
    const result = await updater.checkForUpdate();
    expect(result.updateType).toBe('none');
  });
});
