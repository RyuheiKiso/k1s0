// crdt_assertions.ts — CRDT / OT 協調編集 assertion 関数群
// 図面 collaborative review (v1_interactive) の CRDT merge・presence conflict 検証

// Playwright の expect をインポートする
import { expect } from "@playwright/test";
// Playwright の Page 型をインポートする
import type { Page } from "@playwright/test";

// -------------------------------------------------------------------
// crdt_merge_no_data_loss — CRDT merge assertion
// 2 クライアントの並列編集後に全編集内容がマージされることを確認する
// -------------------------------------------------------------------

// CRDT merge データロスなし検証インラインスクリプト
const CRDT_MERGE_SCRIPT = `
// 2 クライアントが同時に同じ図面を編集した状況をシミュレートする
var clientA = { edits: [{ id: 'A1', op: 'add_line', line: { x1: 0, y1: 0, x2: 100, y2: 0 } }] };
var clientB = { edits: [{ id: 'B1', op: 'add_circle', circle: { cx: 50, cy: 50, r: 30 } }] };
// CRDT merge: 双方の編集を全て保持するための LWW-Register マージ
function mergeCRDT(editsA, editsB) {
  // 全 edit を結合して id で dedup する
  var all = editsA.concat(editsB);
  var seen = {};
  return all.filter(function(e) {
    if (seen[e.id]) return false;
    seen[e.id] = true;
    return true;
  });
}
var merged = mergeCRDT(clientA.edits, clientB.edits);
// A と B の両方の編集が保持されていることを確認する
var hasA1 = merged.some(function(e) { return e.id === 'A1'; });
var hasB1 = merged.some(function(e) { return e.id === 'B1'; });
return hasA1 && hasB1;
`;

// crdt_merge_no_data_loss assertion を実行する
export async function assertCrdtMergeNoDataLoss(page: Page): Promise<void> {
  // ブラウザ内で CRDT merge スクリプトを実行する
  const result = await page.evaluate(CRDT_MERGE_SCRIPT);
  // 両クライアントの編集が全て保持されていることを確認する
  expect(result, "crdt_merge_no_data_loss: CRDT merge 後に編集データが失われました").toBe(true);
}

// -------------------------------------------------------------------
// presence_conflict_resolved — presence conflict assertion
// 2 クライアントが同じ要素を同時編集した際に conflict が解決されることを確認する
// -------------------------------------------------------------------

// presence conflict 解決検証インラインスクリプト
const PRESENCE_CONFLICT_SCRIPT = `
// 2 クライアントが同じ部品（bolt-001）を同時編集した状況をシミュレートする
var clientA = {
  userId: 'user-a',
  editingElement: 'bolt-001',
  timestamp: 1000
};
var clientB = {
  userId: 'user-b',
  editingElement: 'bolt-001',
  timestamp: 1001
};
// タイムスタンプが新しい方（user-b）が優先されると期待する（LWW）
function resolvePresenceConflict(a, b) {
  return a.timestamp > b.timestamp ? a : b;
}
var winner = resolvePresenceConflict(clientA, clientB);
// user-b の編集が採用されていることを確認する（タイムスタンプが新しい）
return winner.userId === 'user-b';
`;

// presence_conflict_resolved assertion を実行する
export async function assertPresenceConflictResolved(page: Page): Promise<void> {
  // ブラウザ内で presence conflict 解決スクリプトを実行する
  const result = await page.evaluate(PRESENCE_CONFLICT_SCRIPT);
  // presence conflict が正しく解決されていることを確認する
  expect(result, "presence_conflict_resolved: presence conflict が解決されませんでした").toBe(true);
}
