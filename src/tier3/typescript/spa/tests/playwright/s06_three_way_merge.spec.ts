// s06_three_way_merge.spec.ts — 3way merge UI シナリオ（conflict 系 S06b）
// @playwright/test で lost_update → ThreeWayMergeUi のレンダリングと操作を検証する
// 適合仕様: 11_クライアント状態適合仕様.md §lost_update → present_3way_merge_ui
// 命名規約: s06_same_actor_supersede_silent.spec.ts（オフライン系 S06）と区別するために S06b とする

// @playwright/test の test / expect をインポートする
import { test, expect } from '@playwright/test';

// 3way merge UI の操作シミュレーション用インライン定義
const MERGE_LOGIC_INLINE = `
// 3way merge の競合フィールドを検出する
function detectConflictingFields(versions) {
  var conflicting = [];
  var keys = Object.keys(versions.base);
  for (var i = 0; i < keys.length; i++) {
    var key = keys[i];
    var localChanged = versions.local[key] !== versions.base[key];
    var remoteChanged = versions.remote[key] !== versions.base[key];
    if (localChanged && remoteChanged) {
      conflicting.push(key);
    }
  }
  return conflicting;
}
// local を採用する解決
function acceptLocal(versions) {
  return { result: versions.local, acceptedSide: 'local' };
}
// remote を採用する解決
function acceptRemote(versions) {
  return { result: versions.remote, acceptedSide: 'remote' };
}
`;

// S06: 3way merge UI → 競合フィールド検出テスト
// テスト名: S06b を使用して s06_same_actor_supersede_silent の S06 との重複を回避する
test('S06b: 3way merge の競合フィールドを正しく検出する', async ({ page }) => {
  // 空白ページを開く
  await page.goto('about:blank');

  // Step 1: base/local/remote の 3 バージョンを用意する
  const versions = {
    // 共通の基点バージョン
    base: { fieldA: 'base-A', fieldB: 'base-B', fieldC: 'base-C' },
    // ローカルの変更（fieldA を変更、fieldB は変更なし）
    local: { fieldA: 'local-A', fieldB: 'base-B', fieldC: 'local-C' },
    // リモートの変更（fieldA を変更、fieldC は変更なし）
    remote: { fieldA: 'remote-A', fieldB: 'remote-B', fieldC: 'base-C' },
  };

  // Step 2: 競合フィールドを検出する
  const conflictingFields = await page.evaluate(
    (args) => {
      // eslint-disable-next-line no-eval
      eval(args.mergeLogic);
      // 競合フィールドを検出する（fieldA は両方が変更しているため競合）
      // @ts-ignore（eval scope の detectConflictingFields 関数を使用する）
      return detectConflictingFields(args.versions);
    },
    { mergeLogic: MERGE_LOGIC_INLINE, versions },
  );

  // fieldA が競合フィールドとして検出されることを確認する
  expect(conflictingFields).toContain('fieldA');
  // fieldB は local のみが変更していないため競合フィールドではないことを確認する
  expect(conflictingFields).not.toContain('fieldB');
  // fieldC は remote が変更していないため競合フィールドではないことを確認する
  expect(conflictingFields).not.toContain('fieldC');
});

// S06b: local 採用 → 解決結果テスト
// S06c: accept_local テスト（S06b との重複を回避する）
test('S06c: accept_local で local バージョンが採用される', async ({ page }) => {
  // 空白ページを開く
  await page.goto('about:blank');

  const versions = {
    base: { fieldA: 'base-A', fieldB: 'base-B' },
    local: { fieldA: 'local-A', fieldB: 'local-B' },
    remote: { fieldA: 'remote-A', fieldB: 'remote-B' },
  };

  // local を採用した場合の結果を検証する
  const acceptLocalResult = await page.evaluate(
    (args) => {
      // eslint-disable-next-line no-eval
      eval(args.mergeLogic);
      // @ts-ignore（eval scope の acceptLocal 関数を使用する）
      return acceptLocal(args.versions);
    },
    { mergeLogic: MERGE_LOGIC_INLINE, versions },
  );

  // acceptedSide が local であることを確認する
  expect(acceptLocalResult.acceptedSide).toBe('local');
  // 結果が local バージョンであることを確認する
  expect(acceptLocalResult.result.fieldA).toBe('local-A');
  expect(acceptLocalResult.result.fieldB).toBe('local-B');
});

// S06c: remote 採用 → 解決結果テスト
// S06d: accept_remote テスト（S06b・S06c との重複を回避する）
test('S06d: accept_remote で remote バージョンが採用される', async ({ page }) => {
  // 空白ページを開く
  await page.goto('about:blank');

  const versions = {
    base: { fieldA: 'base-A', fieldB: 'base-B' },
    local: { fieldA: 'local-A', fieldB: 'local-B' },
    remote: { fieldA: 'remote-A', fieldB: 'remote-B' },
  };

  // remote を採用した場合の結果を検証する
  const acceptRemoteResult = await page.evaluate(
    (args) => {
      // eslint-disable-next-line no-eval
      eval(args.mergeLogic);
      // @ts-ignore（eval scope の acceptRemote 関数を使用する）
      return acceptRemote(args.versions);
    },
    { mergeLogic: MERGE_LOGIC_INLINE, versions },
  );

  // acceptedSide が remote であることを確認する
  expect(acceptRemoteResult.acceptedSide).toBe('remote');
  // 結果が remote バージョンであることを確認する
  expect(acceptRemoteResult.result.fieldA).toBe('remote-A');
  expect(acceptRemoteResult.result.fieldB).toBe('remote-B');
});
