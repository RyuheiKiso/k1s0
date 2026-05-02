---
runbook_id: RB-DR-004
title: Keycloak Realm Export からの復旧（経路 D、RTO 15-30 分）
category: AUTH
severity: SEV1
owner: 起案者
automation: manual
alertmanager_rule: TBD
fmea_id: 間接対応
estimated_recovery: 一次 15 分 / 恒久 30 分
last_updated: 2026-05-02
---

# RB-DR-004: Keycloak Realm Export からの復旧（DR drill 経路 D）

ADR-TEST-005 経路 D。Keycloak DB は経路 C（CNPG barman-cloud）で復元される前提で、Realm 設定（Client / Role / IdP / 認証フロー）の JSON Export からのリストアを担う。詳細は採用後の運用拡大時。

## 1. 前提条件

- 経路 C（PostgreSQL barman-cloud restore）が成功している
- `infra/security/keycloak/realm-export/` に Realm JSON が Git 管理されている
- ADR-SEC-001（Keycloak）と整合

## 2. 対象事象

- Keycloak Realm `k1s0` が消失（誤削除 / DB 破損）
- 検知: BFF JWKS verify が `401`、`/admin/realms/k1s0` が `404`

## 3. 初動手順（5 分以内）

```bash
kubectl get keycloak -n keycloak
kubectl logs -n keycloak <keycloak-pod> --tail=100
```

Slack `#status` に「経路 D 起動、Keycloak Realm restore」を宣言。

## 4. 原因特定手順

- Realm 不在: `/admin/realms` で `k1s0` の有無
- DB 破損: 経路 C 起動を判定
- 設定 drift: `kc.sh export` 直近 vs Git の diff

## 5. 復旧手順

```bash
# 経路 C で DB が復元されていることを確認
kubectl get cluster -n cnpg-system

# Realm JSON を Keycloak Admin REST で import
KC_TOKEN=$(curl -s -X POST -d "client_id=admin-cli" -d "username=admin" -d "password=$ADMIN_PASS" -d "grant_type=password" "https://keycloak.k1s0/realms/master/protocol/openid-connect/token" | jq -r .access_token)
curl -X POST -H "Authorization: Bearer $KC_TOKEN" -H "Content-Type: application/json" -d @infra/security/keycloak/realm-export/k1s0-realm.json https://keycloak.k1s0/admin/realms
```

## 6. 検証手順

```bash
# Realm 存在確認
curl -H "Authorization: Bearer $KC_TOKEN" https://keycloak.k1s0/admin/realms/k1s0
# BFF JWKS verify chain HTTP 200
make verify-e2e
```

## 7. 予防策

- Realm 設定変更時に `kc.sh export` を必ず Git commit
- 四半期 DR drill 経路 D の継続実施

## 8. 関連 Runbook

- [RB-DR-003 postgres barman restore](RB-DR-003-postgres-barman-restore.md) — 経路 C（前提）
- ADR-TEST-005 / ADR-SEC-001
