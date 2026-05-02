---
runbook_id: RB-SEC-006
title: テナント越境検知・封じ込め
category: SEC
severity: SEV1
owner: 起案者
automation: manual
alertmanager_rule: k1s0_tenant_boundary_breach (Loki) / k1s0_cross_tenant_db_query (Falco)
fmea_id: 間接対応
estimated_recovery: 封じ込め 2h
last_updated: 2026-05-02
---

# RB-SEC-006: テナント越境検知・封じ込め

本 Runbook は JWT `tenant_id` 偽造、ABAC バイパス、PostgreSQL RLS バイパス、設定ミス等によるテナント境界違反検知時の封じ込めを定める。
NFR-E-AC-003 / NFR-G-AC-001 に対応する。テナント越境は採用検討の前提を崩す重大事象であり SEV1 即時。

## 1. 前提条件

- 実行者は `security-sre` ClusterRole + Keycloak admin + PostgreSQL pgaudit 読取権限を保持。
- 必要ツール: `kubectl` / `kcadm.sh` / `psql` / `logcli`。
- kubectl context が `k1s0-prod`。
- Keycloak / Istio AuthorizationPolicy / Kyverno ClusterPolicy が起動中。
- pgaudit 拡張が CNPG cluster で有効化されていること（`infra/data/cloudnativepg/cluster.yaml` の `postgresql.parameters.shared_preload_libraries` に `pgaudit` 含有）。

## 2. 対象事象

以下のシグナルが起動トリガー（NFR-E-AC-003 関連）:

- Loki アラート `k1s0_tenant_boundary_breach`: JWT の `tenant_id` クレームとアクセス先リソースの `tenant_id` の不一致を tier1 が `K1s0Error.Forbidden` で拒否したイベントが **3 分で 5 件以上**。
- Falco ルール `k1s0_cross_tenant_db_query`: PostgreSQL でテナント境界を超えた SELECT が発行された場合（Row-Level Security バイパスの疑い）。
- Kyverno ポリシー違反: `tenant_id` ラベル欠落 Pod のデプロイ試行。
- 外部からの越境アクセス報告（採用組織のユーザーから）。

検知シグナル:

```bash
# 直近の Forbidden ログを確認
logcli query '{namespace="k1s0-tier1", job="audit"}
  |= "Forbidden" | json
  | line_format "{{.tenant_id}} {{.target_tenant_id}} {{.user_id}} {{.path}}"' \
  --since=30m | sort | uniq -c | sort -rn | head -30
```

通知経路: PagerDuty `security-sre` → Slack `#incident-sev1` → CTO + 採用組織セキュリティ担当へ即時連絡。

## 3. 初動手順（5 分以内）

最初の 5 分で SEV1 を確定し、越境パターン（A/B/C/D）の仮説を立てる。

```bash
# 直近 5 分の越境イベント数を確認
logcli query '{namespace="k1s0-tier1", job="audit"}
  |= "Forbidden" |= "tenant"' \
  --since=5m | wc -l
```

```bash
# 影響テナント ペア
logcli query '{namespace="k1s0-tier1", job="audit"}
  |= "cross_tenant" | json
  | line_format "{{.source_tenant}} -> {{.target_tenant}}"' \
  --since=30m | sort | uniq -c | sort -rn | head -10
```

ステークホルダー通知（必須）:

- SEV1 即時宣言、Slack `#incident-sev1` に「テナント越境検知、封じ込め開始」を投稿。
- [`oncall/escalation.md`](../../oncall/escalation.md) を起動、CTO + 採用組織セキュリティ担当に連絡。
- PII 閲覧の可能性があれば [`RB-SEC-005-pii-leak-detection.md`](RB-SEC-005-pii-leak-detection.md) を並行起動。

## 4. 原因特定手順

```bash
# JWT 検証コードの全コードパスを trace（src/tier1/auth-go の grep）
grep -rn "tenant_id" src/tier1/auth-go/

# PostgreSQL RLS ポリシー定義を確認
kubectl exec -it k1s0-pg-0 -n k1s0-data -- psql -U postgres -c "
  SELECT schemaname, tablename, policyname, permissive, roles, qual
  FROM pg_policies WHERE tablename LIKE 'tenant_%';"

# Istio AuthorizationPolicy の jwt フィルターが全 API パスに適用されているか
kubectl get authorizationpolicy -A -o yaml | grep -E "jwt|tenant"
```

越境パターン分類（初動と並行で確定）:

| パターン | 説明 | 緊急度 | 対応 |
|---|---|---|---|
| A: JWT クレーム偽造 | `tenant_id` クレームが改ざんされた JWT | 最高 | Keycloak signing key rotate |
| B: ABAC バイパス | 認可ロジックのバグで tenant_id 検証をスキップ | 高 | サービススケールダウン + コード修正 |
| C: DB RLS バイパス | PostgreSQL Row-Level Security が機能していない | 高 | RLS ポリシー修正 + 再 enable |
| D: 設定ミス | 誤った namespace / ラベルで他テナントデータにアクセス | 中 | ラベル修正 + Kyverno 強化 |

エスカレーション: 原因が確定しなくても封じ込めを優先。原因調査は封じ込め後に時間をかけて実施。

## 5. 復旧手順

### Step 1: 越境した tenant_id ペアと SPIFFE ID の特定（〜30 分）

```bash
# 越境した tenant_id ペア
logcli query '{namespace="k1s0-tier1", job="audit"}
  |= "cross_tenant" | json
  | line_format "{{.source_tenant}} -> {{.target_tenant}} ({{.user_id}})"' \
  --since=2h | sort | uniq

# SPIFFE ID で対象 Pod の ID を確認
kubectl get pod -n k1s0-tier1 -o jsonpath='{range .items[*]}
{.metadata.name}{"\t"}{.metadata.annotations.spiffe\.io/spiffe-id}{"\n"}{end}'
```

### Step 2: 侵害 Pod / サービスの隔離（〜1 時間）

パターン A（JWT 偽造）の場合: Keycloak でセッションを即時無効化:

```bash
# 対象 client の全セッションを無効化
kcadm.sh delete clients/<client-id>/user-sessions -r k1s0
# JWT 署名鍵をローテーション（OIDC discovery で全クライアントが新鍵を取得）
kcadm.sh create realms/k1s0/keys -s enabled=true -s providerId=rsa-generated
```

パターン B/C（バグ起因）の場合: 影響サービスをスケールダウン:

```bash
kubectl scale deployment/<breached-service> --replicas=0 -n k1s0-tier1
# Istio AuthorizationPolicy でさらに制限
kubectl apply -f - <<EOF
apiVersion: security.istio.io/v1
kind: AuthorizationPolicy
metadata:
  name: deny-all-<breached-service>
  namespace: k1s0-tier1
spec:
  selector:
    matchLabels:
      app: <breached-service>
  action: DENY
  rules:
  - {}
EOF
```

パターン D（設定ミス）の場合: 誤ったラベルを修正し、Pod を再起動:

```bash
kubectl patch deployment/<service> -n k1s0-tier1 \
  --type='json' \
  -p='[{"op":"replace","path":"/spec/template/metadata/labels/tenant_id",
        "value":"<correct-tenant-id>"}]'
```

### Step 3: 影響範囲の確定（〜2 時間）

```bash
# PostgreSQL の監査ログ（pgaudit）から対象 tenant のクエリを抽出
kubectl exec -it k1s0-pg-0 -n k1s0-data -- psql -U postgres -c "
  SELECT log_time, command_tag, database_name, object_name, statement
  FROM pgaudit_log
  WHERE log_time > NOW() - INTERVAL '3 hours'
    AND statement LIKE '%<target-tenant-id>%'
  ORDER BY log_time DESC LIMIT 100;"
```

PII が閲覧された可能性がある場合は [`RB-SEC-005-pii-leak-detection.md`](RB-SEC-005-pii-leak-detection.md) を並行起動。

### Step 4: 段階的復旧

1. バグ修正 PR をホットフィックスブランチで作成し、4-eyes レビュー後に緊急デプロイ。
2. Kyverno ポリシーで `tenant_id` ラベルの必須化を強制:

```bash
kubectl get clusterpolicy k1s0-require-tenant-label -o yaml | grep -A5 "spec:"
```

3. 影響テナントの採用組織担当者に越境の事実と影響データを報告。
4. 修正後、Litmus Chaos で越境試行のテストを実行して修正を確認。

## 6. 検証手順

復旧完了の判定基準:

- Loki アラート `k1s0_tenant_boundary_breach` が 24h 間継続して未発火。
- Falco アラート `k1s0_cross_tenant_db_query` が 24h 間 0 件。
- pgaudit ログで `tenant_id` ミスマッチクエリが 24h 間 0 件。
- 影響テナントのアクセストークンが全て無効化済み（Keycloak audit log で確認）。
- バグ修正 PR がマージ・デプロイ済み（パターン B/C の場合）。
- Kyverno ClusterPolicy `k1s0-require-tenant-label` が有効で全 namespace で `enforce` モード。
- Litmus Chaos の越境試行テストが PASS（CI で自動実行）。
- 採用組織担当者に書面通知済み（影響テナントの場合）。

## 7. 予防策

- ポストモーテム作成（24h 以内、`postmortems/<YYYY-MM-DD>-RB-SEC-006.md`）。
- NFR-E-AC-003 に基づく Litmus Chaos 越境テストの CI 組み込み PR（`tools/test/tenant-isolation-chaos/`）。
- 越境試行を Grafana ダッシュボードに可視化する（`cross_tenant_attempts` メトリクス）。
- PII 閲覧が確認された場合: [`RB-COMP-002-pii-regulatory-disclosure.md`](RB-COMP-002-pii-regulatory-disclosure.md) を起動。
- JWT 検証コードのレビュー観点を PR テンプレに追加。
- 月次 Chaos Drill 対象に「JWT tenant_id 改ざん試行」シナリオを追加。

## 8. 関連 Runbook

- 関連設計書: [`docs/03_要件定義/30_非機能要件/E_セキュリティ.md`](../../../docs/03_要件定義/30_非機能要件/E_セキュリティ.md) (NFR-E-AC-003)、[`G_データ保護とプライバシー.md`](../../../docs/03_要件定義/30_非機能要件/G_データ保護とプライバシー.md) (NFR-G-AC-001)
- 関連 ADR: [ADR-SEC-001 (Keycloak)](../../../docs/02_構想設計/adr/ADR-SEC-001-keycloak.md), [ADR-SEC-002 (OpenBao)](../../../docs/02_構想設計/adr/ADR-SEC-002-openbao.md), [ADR-SEC-003 (SPIRE)](../../../docs/02_構想設計/adr/ADR-SEC-003-spire.md)
- 連鎖 Runbook:
  - [`RB-SEC-005-pii-leak-detection.md`](RB-SEC-005-pii-leak-detection.md) — PII 閲覧の場合
  - [`RB-AUTH-002-auth-abuse-detection.md`](RB-AUTH-002-auth-abuse-detection.md) — 内部 actor / Secret 大量読取が並行発生した場合
  - [`RB-COMP-002-pii-regulatory-disclosure.md`](RB-COMP-002-pii-regulatory-disclosure.md) — 規制報告が必要な場合
- エスカレーション: [`../../oncall/escalation.md`](../../oncall/escalation.md)
