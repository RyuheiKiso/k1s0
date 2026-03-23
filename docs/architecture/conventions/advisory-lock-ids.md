# PostgreSQL Advisory Lock ID 管理規則

## 概要

各サービスのマイグレーション時に使用する PostgreSQL Advisory Lock ID の割り当て規則。

`pg_advisory_lock` による排他制御でマイグレーションの重複実行を防ぐ。

## ID 割り当て一覧

| サービス | ID | パス |
|---------|----|----|
| task | 1000000010 | regions/service/task |
| board | 1000000011 | regions/service/board |
| activity | 1000000012 | regions/service/activity |

## 新規サービス追加時のルール

- 既存 ID との衝突を避けるため、本ファイルを更新してから ID を使用すること
- ID は 1000000000 番台を使用する（system tier: 2000000000 番台を予約）
- ID の重複はデッドロックの原因になるため、必ず一意性を確認すること
