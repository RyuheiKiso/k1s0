---
runbook_id: RB-AUTH-001
title: Keycloak DB 障害対応
category: AUTH
severity: SEV2
owner: 協力者
automation: manual
alertmanager_rule: KeycloakDbDown
fmea_id: FMEA-004
estimated_recovery: 暫定 30 分 / 恒久 4 時間
last_updated: 2026-05-02
---

# RB-AUTH-001: Keycloak DB 障害対応

本 Runbook は Keycloak 専用 PostgreSQL（CNPG クラスタ `k1s0-keycloak-pg`）の障害時対応を定める。
影響は新規ログイン不可。既存セッションは Refresh Token TTL（24h）内なら継続可能であり SEV2。
NFR-E-AC-001 / FMEA-004 / ADR-SEC-001 に対応する。

## 1. 前提条件

- 実行者は `k1s0-operator` ClusterRole + Keycloak admin token を保持。
- 必要ツール: `kubectl` / `kcadm.sh` / `kubectl cnpg`。
- kubectl context が `k1s0-prod`。
- Keycloak Helm release は `keycloak`、namespace `k1s0-auth`。
- Keycloak DB は CNPG cluster `k1s0-keycloak-pg`、namespace `cnpg-system`（`infra/security/keycloak/values.yaml` 参照）。

## 2. 対象事象

- Alertmanager `KeycloakDbDown` 発火（`cnpg_pg_up{cluster="k1s0-keycloak-pg",role="primary"} == 0`）、または
- 新規ログイン試行で `KC-DB-CONN-FAIL` エラーログ急増、または
- Keycloak Pod の readiness probe 失敗。

検知シグナル:

```promql
# Keycloak 専用 DB の primary 応答状態
cnpg_pg_up{cluster="k1s0-keycloak-pg", role="primary"} == 0

# Keycloak Pod 自身の health
up{namespace="k1s0-auth", job="keycloak"} == 0

# Keycloak ログイン失敗率
rate(keycloak_login_failures_total[5m]) > 1
```

ダッシュボード: **Grafana → k1s0 Keycloak Auth**。
通知経路: PagerDuty `tier1-platform-team` → Slack `#alert-auth`。

## 3. 初動手順（5 分以内）

```bash
# Keycloak DB cluster 状態
kubectl get cluster k1s0-keycloak-pg -n cnpg-system -o wide
kubectl get pods -n cnpg-system -l cnpg.io/cluster=k1s0-keycloak-pg
```

```bash
# Keycloak Pod 状態
kubectl get pods -n k1s0-auth -l app.kubernetes.io/name=keycloak
kubectl logs -n k1s0-auth -l app.kubernetes.io/name=keycloak --tail=50 \
  | grep -iE "FATAL|ERROR|JDBCException"
```

```bash
# Refresh Token TTL の確認（既存ユーザーの猶予時間）
kubectl exec -n k1s0-auth deploy/keycloak -- \
  /opt/keycloak/bin/kc.sh export --realm k1s0 --json --file /tmp/realm.json
grep -E "ssoSessionMaxLifespan|refreshTokenMaxReuse" /tmp/realm.json
```

ステークホルダー通知: Slack `#alert-auth` に「Keycloak DB 障害、新規ログイン不可。既存セッションは Refresh Token 内継続」を投稿。
SEV2 のため `oncall/escalation.md` 起動は不要だが、24h で Refresh Token が枯渇する前に必ず復旧。

## 4. 原因特定手順

```bash
# CNPG Operator ログ
kubectl logs -n cnpg-system deploy/cnpg-controller-manager --tail=100 | grep "keycloak"

# DB primary Pod ログ
PRIM=$(kubectl get cluster k1s0-keycloak-pg -n cnpg-system -o jsonpath='{.status.currentPrimary}')
kubectl logs -n cnpg-system "${PRIM}" --previous | tail -50
```

よくある原因:

1. **OOM Kill**: Keycloak DB の Pod が memory pressure で kill。`kubectl describe pod ${PRIM} -n cnpg-system` で確認。
2. **PVC フル**: WAL 蓄積でディスク満杯。`df -h` で確認。
3. **Replication 遅延**: standby が古く failover に失敗。`cnpg_pg_replication_lag` を Grafana で確認。
4. **JDBC connection pool 枯渇**: Keycloak が DB に対して接続を持ち過ぎ。`max_connections` パラメータ確認。
5. **CNPG Operator の障害**: Operator が落ちていると failover 不発。

エスカレーション: 原因が確定しないまま 30 分経過したら L3 起案者に連絡。

## 5. 復旧手順

### CNPG 自動 failover を待つ（〜2 分）

```bash
kubectl get cluster k1s0-keycloak-pg -n cnpg-system -w
# status.currentPrimary が切り替われば failover 成功
```

### 自動 failover が動かない場合（手動）

```bash
# standby 一覧確認
kubectl get pods -n cnpg-system -l cnpg.io/cluster=k1s0-keycloak-pg,role=replica

# 手動昇格
kubectl cnpg promote k1s0-keycloak-pg -n cnpg-system --instance k1s0-keycloak-pg-2
```

### Keycloak の rolling restart（接続プールリセット）

```bash
kubectl rollout restart deployment/keycloak -n k1s0-auth
kubectl rollout status deployment/keycloak -n k1s0-auth
```

### Refresh Token TTL 一時延長（万一復旧が長引く場合）

```bash
# Refresh Token TTL を 24h → 72h に一時延長（SEV2 復旧が長引く場合のみ）
kcadm.sh update realms/k1s0 \
  -s ssoSessionMaxLifespan=259200 \
  -s ssoSessionIdleTimeout=259200
# 復旧後に元に戻す（24h = 86400）
```

## 6. 検証手順

復旧完了の判定基準:

- Keycloak DB primary が `cnpg_pg_up{role="primary"} == 1` を 5 分継続。
- Keycloak Pod 全数が `Ready=True`、`up{job="keycloak"} == 1`。
- 新規ログイン成功率 `rate(keycloak_login_success_total[5m]) > 0`。
- 直近 10 分の Loki クエリ `{namespace="k1s0-auth"} |= "JDBCException"` が 0 件。
- ログイン動作確認（手動でテストアカウントでログインし、access token 取得）。
- Refresh Token TTL を恒久値に戻した（一時延長した場合）。

## 7. 予防策

- ポストモーテム起票（72 時間以内、`postmortems/<YYYY-MM-DD>-RB-AUTH-001.md`）。
- Keycloak DB のメモリ Limit / `max_connections` チューニング。
- Keycloak HA 化検討（採用後の運用拡大時で 2 Pod 化）。
- Refresh Token TTL のデフォルト見直し（業務インパクトとセキュリティのバランス）。
- 月次 Chaos Drill 対象に「Keycloak DB Pod kill」シナリオを追加。
- NFR-A-REC-002 の MTTR ログを更新。

## 8. 関連 Runbook

- 関連設計書: `infra/security/keycloak/values.yaml`、`infra/security/keycloak/instance/`
- 関連 ADR: [ADR-SEC-001（Keycloak）](../../../docs/02_構想設計/adr/ADR-SEC-001-keycloak.md)
- 関連 NFR: [NFR-E-AC-001](../../../docs/03_要件定義/30_非機能要件/E_セキュリティ.md)
- 関連 FMEA: [FMEA-004](../../../docs/04_概要設計/55_運用ライフサイクル方式設計/06_FMEA分析方式.md)
- 連鎖 Runbook:
  - [`RB-DB-002-postgres-primary-failover.md`](RB-DB-002-postgres-primary-failover.md) — 同種 CNPG 障害（共通インフラ）
  - [`RB-AUTH-002-auth-abuse-detection.md`](RB-AUTH-002-auth-abuse-detection.md) — 認証エラー急増が検知される場合
