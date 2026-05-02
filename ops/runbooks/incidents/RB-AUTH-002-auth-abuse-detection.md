---
runbook_id: RB-AUTH-002
title: 不正ログイン / Secret 大量読取検知・封じ込め
category: AUTH
severity: SEV2（不正ログイン試行）/ SEV1（認証突破 / Secret 大量読取）
owner: 起案者
automation: manual
alertmanager_rule: k1s0_auth_brute_force / k1s0_secret_bulk_read / k1s0_suspicious_token_usage
fmea_id: 間接対応（FMEA-002 / FMEA-004 連鎖）
estimated_recovery: SEV2 4h / SEV1 昇格時 2h
last_updated: 2026-05-02
---

# RB-AUTH-002: 不正ログイン / Secret 大量読取検知・封じ込め

本 Runbook は Keycloak / OpenBao への不正アクセス検知時の封じ込め手順を定める。
ブルートフォース・クレデンシャルスタッフィング・内部侵害（Secret 大量読取）の 2 パターンに対応。
NFR-E-MON-002 / NFR-E-AC-004 / NFR-E-AC-005 に対応する。

## 1. 前提条件

- 実行者は `security-sre` ClusterRole + Keycloak admin + OpenBao admin token を保持。
- 必要ツール: `kubectl` / `kcadm.sh`（Keycloak CLI）/ `bao`（OpenBao CLI）/ `logcli`。
- kubectl context が `k1s0-prod`。
- Keycloak / OpenBao 自体が healthy であること（停止している場合は [`RB-SEC-001`](RB-SEC-001-openbao-raft-failover.md) / [`RB-AUTH-001`](RB-AUTH-001-keycloak-db-failover.md) 先行）。
- Envoy Gateway の SecurityPolicy（IPDeny）を編集する Kyverno ポリシー権限。

## 2. 対象事象

以下のシグナルが起動トリガー:

- Loki アラート `k1s0_auth_brute_force`: 同一 IP から 5 分間に 20 回以上の `INVALID_PASSWORD` または `INVALID_OTP` イベント（Keycloak 監査ログ）。
- Loki アラート `k1s0_secret_bulk_read`: 単一 SPIFFE ID から 5 分間に 100 件以上の OpenBao シークレット読取。
- Falco アラート `k1s0_suspicious_token_usage`: 異なる地理的 IP から同一 JWT の使用。
- Keycloak リアルタイムイベント: `LOGIN_ERROR` が急増。

検知シグナル:

```bash
# Keycloak 認証失敗ログの確認
logcli query '{namespace="k1s0-auth", job="keycloak"}
  | json | type="LOGIN_ERROR"
  | line_format "{{.ipAddress}} {{.userId}} {{.error}}"' \
  --since=30m | sort | uniq -c | sort -rn | head -20

# OpenBao の異常な Secret 読取確認
logcli query '{namespace="k1s0-security", job="openbao"}
  |= "secret/read" | json
  | line_format "{{.remote_address}} {{.request_path}}"' \
  --since=30m | sort | uniq -c | sort -rn | head -20
```

通知経路: PagerDuty `security-sre` → Slack `#incident-security`。

## 3. 初動手順（5 分以内）

最初の 5 分でパターン A（外部ブルートフォース）/ B（内部 Secret 大量読取）を判別する:

```bash
# パターン A の指標: Keycloak 認証失敗が同一 IP に集中
logcli query '{namespace="k1s0-auth", job="keycloak"}
  | json | type="LOGIN_ERROR"
  | line_format "{{.ipAddress}}"' \
  --since=10m | sort | uniq -c | sort -rn | head -5
```

```bash
# パターン B の指標: OpenBao 読取が単一 SPIFFE ID に集中
logcli query '{namespace="k1s0-security", job="openbao"}
  |= "secret/read" | json
  | line_format "{{.client_token_accessor}}"' \
  --since=10m | sort | uniq -c | sort -rn | head -5
```

ステークホルダー通知: Slack `#incident-security` に「パターン <A/B>、攻撃源 <IP/SPIFFE ID>、影響範囲調査中」を 5 分以内に投稿。
SEV1 条件（後述 §5 の Step 4）に該当すれば即時 [`oncall/escalation.md`](../../oncall/escalation.md) 起動。

## 4. 原因特定手順

- 認証ログの全タイムラインを作成し、最初の疑わしいアクセスを特定する。
- パスワードリスト攻撃の場合は `haveibeenpwned` 等での流出確認を検討する。
- OpenBao の audit log から読取られた Secret パスを全件確認する:

```bash
kubectl exec -it openbao-0 -n k1s0-security -- \
  bao read sys/audit-hash/file -format=json \
  | jq '.data | select(.type=="request") | .path' | sort | uniq -c | sort -rn
```

よくある原因:

1. **クレデンシャルスタッフィング**: 他社流出パスワードリストでの試行。haveibeenpwned で対象アカウントを確認。
2. **ブルートフォース**: 同一アカウントへの連続試行。Keycloak のブルートフォース保護設定が緩い場合に発生。
3. **内部侵害（SPIFFE ID 経由）**: SVID 取得済みの Pod が攻撃者 / 不正コードに乗っ取られた。
4. **AppRole RoleID/SecretID 漏洩**: GitHub / ログに credential が露出。`gitleaks` 検知と連鎖。
5. **JWT 鍵漏洩**: Keycloak の signing key が漏洩、攻撃者が任意 JWT を発行。

エスカレーション: パターン B（内部侵害）で root token 漏洩疑いがある場合は SEV1 即時昇格、CTO + CPO + 法務に連絡。

## 5. 復旧手順

### パターン A: ブルートフォース / クレデンシャルスタッフィング攻撃

#### Step 1: 攻撃元 IP のブロック（〜15 分）

```bash
# 攻撃元 IP を特定
logcli query '{namespace="k1s0-auth", job="keycloak"}
  | json | type="LOGIN_ERROR"
  | line_format "{{.ipAddress}}"' \
  --since=1h | sort | uniq -c | sort -rn | head -10
```

```bash
# Envoy Gateway の SecurityPolicy で IP をブロック
kubectl apply -f - <<EOF
apiVersion: gateway.envoyproxy.io/v1alpha1
kind: SecurityPolicy
metadata:
  name: block-brute-force-$(date +%Y%m%d)
  namespace: k1s0-ingress
spec:
  targetRef:
    group: gateway.networking.k8s.io
    kind: HTTPRoute
    name: k1s0-public
  authorization:
    defaultAction: Allow
    rules:
    - action: Deny
      principal:
        clientCIDRs:
        - <attacker-ip>/32
EOF
```

Keycloak でアカウントロック閾値を一時的に強化（Realm Settings → Brute Force）。

#### Step 2: 侵害されたアカウントの特定（〜1 時間）

```bash
# LOGIN_SUCCESS に先行する LOGIN_ERROR が多い user_id を確認
logcli query '{namespace="k1s0-auth", job="keycloak"}
  | json | type="LOGIN_SUCCESS"
  | line_format "{{.userId}} {{.ipAddress}}"' \
  --since=6h > /tmp/login_success.txt
grep "<attacker-ip>" /tmp/login_success.txt
```

```bash
# 侵害が疑われるアカウントのセッションを無効化、パスワードリセット強制
kcadm.sh update users/<user-id>/logout -r k1s0
kcadm.sh update users/<user-id> -r k1s0 \
  -s requiredActions='["UPDATE_PASSWORD"]'
```

### パターン B: Secret 大量読取（内部侵害疑い）

#### Step 3: OpenBao の該当 token を即時 revoke（〜30 分）

```bash
# 異常な読取を行っている SPIFFE ID / AppRole を特定
bao audit list
kubectl exec -it openbao-0 -n k1s0-security -- \
  bao read sys/audit-hash/file
```

```bash
# AppRole / Service Account token の revoke
bao token revoke -accessor <token-accessor>
# またはポリシーを即時更新して読取権限を剥奪
bao policy write <service>-deny infra/security/openbao/policies/<service>-deny.hcl
```

```bash
# 影響 Pod を再起動して新 token を取得させる
kubectl rollout restart deployment/<service> -n k1s0-tier1
```

### Step 4: SEV1 昇格の判断（〜30 分）

以下の条件に 1 つでも該当する場合は SEV1 に昇格し、[`oncall/escalation.md`](../../oncall/escalation.md) を起動:

- 管理者アカウント（`realm-admin` ロール）の侵害が疑われる
- OpenBao の root token または unseal share の漏えいが疑われる
- 複数テナントの Secret が読取られた
- PII を含む Secret が読取られた → [`RB-SEC-005-pii-leak-detection.md`](RB-SEC-005-pii-leak-detection.md) を並行起動

## 6. 検証手順

復旧完了の判定基準:

- Loki アラート `k1s0_auth_brute_force` / `k1s0_secret_bulk_read` が 30 分間継続して未発火。
- Keycloak `LOGIN_ERROR` 率が通常レベル（直前 24h 平均）に戻る。
- OpenBao 監査ログに不審な読取パターンが新規発生していない（`bao audit list` + 直近 1h 確認）。
- 侵害アカウントのセッションが全て無効化済み（`kcadm.sh get sessions/count?clientId=<id>` で確認）。
- ブロック IP からの新規アクセス試行が観測されない（Envoy Gateway access log）。
- パターン B の場合、影響 Secret が rotate 完了（[`secret-rotation.md`](../secret-rotation.md) §「漏洩発生時」）。

## 7. 予防策

- ポストモーテム作成（SEV1: 24h / SEV2: 72h 以内）`postmortems/<YYYY-MM-DD>-RB-AUTH-002.md`。
- Keycloak のブルートフォース保護設定を恒久的に強化する PR（`infra/security/keycloak/`）。
- MFA の適用範囲拡大を検討する（NFR-E-AC-005）。全 admin ロールに必須化。
- OpenBao の Secret 読取アラート閾値を調整する（誤検知率 vs 検知漏れバランス）。
- 侵害アカウントが管理者の場合: [`secret-rotation.md`](../secret-rotation.md) で全 Secret ローテーション（24h 以内）。
- 月次 Chaos Drill 対象に「ブルートフォース 1000 req/s」シナリオを追加。
- 攻撃源 IP のブロックリストを四半期見直し、解除可否判定。

## 8. 関連 Runbook

- 関連設計書: [`docs/03_要件定義/30_非機能要件/E_セキュリティ.md`](../../../docs/03_要件定義/30_非機能要件/E_セキュリティ.md) (NFR-E-MON-002, NFR-E-AC-004, NFR-E-AC-005)
- 関連 ADR: [ADR-SEC-001 (Keycloak)](../../../docs/02_構想設計/adr/ADR-SEC-001-keycloak.md), [ADR-SEC-002 (OpenBao)](../../../docs/02_構想設計/adr/ADR-SEC-002-openbao.md), [ADR-SEC-003 (SPIRE)](../../../docs/02_構想設計/adr/ADR-SEC-003-spire.md)
- 連鎖 Runbook:
  - [`../secret-rotation.md`](../secret-rotation.md) — Secret rotation 実施
  - [`RB-SEC-005-pii-leak-detection.md`](RB-SEC-005-pii-leak-detection.md) — PII 含有 Secret が読取られた場合
  - [`RB-SEC-006-tenant-boundary-breach.md`](RB-SEC-006-tenant-boundary-breach.md) — テナント越境が同時発生した場合
- エスカレーション: [`../../oncall/escalation.md`](../../oncall/escalation.md)
