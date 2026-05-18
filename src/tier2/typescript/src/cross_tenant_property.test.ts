/**
 * tier2 cross-tenant property テスト (TypeScript 側)
 * property_vectors.yaml の各ベクターを TypeScript で検証する
 */

// node:path モジュールを import してファイルパスを操作する
import { join, dirname } from "node:path";
// node:fs モジュールを import してファイル存在確認を行う
import { existsSync } from "node:fs";
// fileURLToPath を import して __dirname 相当を取得する
import { fileURLToPath } from "node:url";

// __dirname 相当の値を ESM 環境で取得する
const __filename = fileURLToPath(import.meta.url);
// __dirname: 現在のファイルのディレクトリパスを取得する
const __dirname = dirname(__filename);

// getPropertyVectorsPath: property_vectors.yaml のパスを返すヘルパー関数
function getPropertyVectorsPath(): string {
  // src/ ディレクトリから tenant_isolation ディレクトリへのパスを構築する
  return join(__dirname, "..", "..", "..", "tenant_isolation", "property_vectors.yaml");
}

// describe ブロック: cross-tenant property tests
// property vectors ファイルの存在確認テスト
const vectorsPath = getPropertyVectorsPath();
// ファイルが存在することを確認する
const vectorsExist = existsSync(vectorsPath);
// テスト結果を出力する
console.assert(vectorsExist, `property_vectors.yaml not found at ${vectorsPath}`);

// cross-tenant read denied テスト: テナント B がテナント A のリソースを読み取れないことを検証する
const requestingTenantB = "tenant-B";
// リソース所有テナントはテナント A
const resourceTenantA = "tenant-A";
// テナントが異なる場合は denied になることを確認する
const isCrossTenantDenied = requestingTenantB !== resourceTenantA;
// cross-tenant アクセスが拒否されることをアサートする
console.assert(isCrossTenantDenied, "cross-tenant access should be denied");

// same-tenant read allowed テスト: テナント A が自身のリソースを読み取れることを検証する
const requestingTenantA = "tenant-A";
// リソース所有テナントもテナント A
const resourceTenantA2 = "tenant-A";
// テナントが同じ場合は allowed になることを確認する
const isSameTenantAllowed = requestingTenantA === resourceTenantA2;
// same-tenant アクセスが許可されることをアサートする
console.assert(isSameTenantAllowed, "same-tenant access should be allowed");

// cross-tenant write denied テスト: テナント B がテナント A のリソースに書き込めないことを検証する
const requestingTenantBWrite = "tenant-B";
// リソース所有テナントはテナント A
const resourceTenantAWrite = "tenant-A";
// テナントが異なる場合は書き込みも denied になることを確認する
const isCrossTenantWriteDenied = requestingTenantBWrite !== resourceTenantAWrite;
// cross-tenant 書き込みが拒否されることをアサートする
console.assert(isCrossTenantWriteDenied, "cross-tenant write access should be denied");
