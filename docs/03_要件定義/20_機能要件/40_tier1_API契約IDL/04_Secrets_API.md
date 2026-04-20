# 04. Secrets API

OpenBao をバックエンドとする秘密情報取得・ローテーション API。Read（取得）と Rotate（ローテーション）の両面を持ち、テナント境界で分離、ローテーションは WORM 監査ログに記録される（NFR-E-MON-001 に準拠）。tier2/tier3 は OpenBao の認証方式（Token / AppRole / JWT）を直接扱わず、SPIFFE ID による自動認証を通じて秘密情報を取得する。

## 要件対応

- 要件ファイル: [../10_tier1_API要件/04_Secrets_API.md](../10_tier1_API要件/04_Secrets_API.md)
- 要件 ID: FR-T1-SECRETS-001〜004
- 共通型: [00_共通型定義.md](00_共通型定義.md)

## 設計のポイント

シークレット名の指定はテナント境界を越えた参照を即 `PermissionDenied`（`E-PERMISSION_DENIED-AUTH-002`）で拒否する。バージョン指定は省略時が最新、明示指定時は `grace_period` 中の旧バージョンのみ取得可能。`Rotate` はトランザクショナルに動作し、失敗時は OpenBao 側の状態は不変で返される。`grace_period_sec` の既定値 3600 秒は tier2 側の接続プール drain 時間を想定した保守的な値であり、DB 資格情報のように短時間で切替可能なシークレットは呼出側で短縮する。`idempotency_key` により同一キーでの再試行時は同じ `new_version` を返し、ネットワーク分断時のダブルローテーションを防ぐ。

## Protobuf 定義

```protobuf
// Secrets API (FR-T1-SECRETS-001〜004)
syntax = "proto3";
package k1s0.tier1.secrets.v1;
import "k1s0/tier1/common/v1/common.proto";

service SecretsService {
  // 単一シークレット取得
  rpc Get(GetSecretRequest) returns (GetSecretResponse);
  // 一括取得 (テナントに割当された全シークレット)
  rpc BulkGet(BulkGetSecretRequest) returns (BulkGetSecretResponse);
  // ローテーション実行 (FR-T1-SECRETS-004)
  // 成功時は new_version を返し、旧バージョンは grace_period_sec まで Get 可能
  // 失敗時は K1s0Error を返し OpenBao 側は不変 (トランザクショナル)
  rpc Rotate(RotateSecretRequest) returns (RotateSecretResponse);
}

message GetSecretRequest {
  // シークレット名 (テナント境界を超えた参照は即 PermissionDenied)
  string name = 1;
  k1s0.tier1.common.v1.TenantContext context = 2;
  // 省略時は最新、明示で旧バージョン取得可 (grace_period 中のみ)
  optional int32 version = 3;
}

message GetSecretResponse {
  // 値 (Base64 エンコード必要時はクライアント側で判断)
  map<string, string> values = 1;
  // バージョン (ローテーション追跡用)
  int32 version = 2;
}

message BulkGetSecretRequest {
  k1s0.tier1.common.v1.TenantContext context = 1;
}

message BulkGetSecretResponse {
  map<string, GetSecretResponse> results = 1;
}

message RotateSecretRequest {
  // ローテーション対象シークレット名
  string name = 1;
  k1s0.tier1.common.v1.TenantContext context = 2;
  // 旧バージョンの猶予時間 (0 は即無効、既定 3600 秒)
  // tier2 側の接続プール drain 時間を想定
  int32 grace_period_sec = 3;
  // 動的シークレット (DB 資格情報等) の場合の発行ポリシー名
  optional string policy = 4;
  // 冪等性キー (同一キーでの再試行は同じ new_version を返す)
  string idempotency_key = 5;
}

message RotateSecretResponse {
  // ローテーション後の新バージョン
  int32 new_version = 1;
  // 旧バージョン (grace_period_sec まで Get 可能)
  int32 previous_version = 2;
  // 新バージョン発効時刻
  int64 rotated_at_ms = 3;
  // 動的シークレット時の TTL (静的シークレットでは 0)
  int32 ttl_sec = 4;
}
```
