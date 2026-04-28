# テナント越境検知・封じ込め Runbook

> **severity**: SEV1
> **owner**: security-sre
> **estimated_mttr**: 2h（封じ込め）
> **last_updated**: 2026-04-28

## 1. 検出 (Detection)

以下のシグナルが起動トリガーとなる（NFR-E-AC-003 関連）。

- Loki アラート `k1s0_tenant_boundary_breach`: JWT の `tenant_id` クレームと
  アクセス先リソースの `tenant_id` の不一致を tier1 が `K1s0Error.Forbidden` で拒否した
  イベントが **3 min で 5 件以上**
- Falco ルール `k1s0_cross_tenant_db_query`: PostgreSQL でテナント境界を超えた
  SELECT が発行された場合（Row-Level Security バイパスの疑い）
- Kyverno ポリシー違反: `tenant_id` ラベル欠落 Pod のデプロイ試行
- 外部からの越境アクセス報告（採用組織のユーザーから）

```bash
# 直近の Forbidden ログを確認
logcli query '{namespace="k1s0-tier1", job="audit"}
  |= "Forbidden" | json
  | line_format "{{.tenant_id}} {{.target_tenant_id}} {{.user_id}} {{.path}}"' \
  --since=30m | sort | uniq -c | sort -rn | head -30
```

## 2. 初動 (Immediate Action)

**目標: 検知後 2 時間以内に越境経路を封じ込める。**

### Step 1: 越境パターンの特定（〜30 分）

1. 越境の種別を確認する。

   | パターン | 説明 | 緊急度 |
   |---|---|---|
   | A: JWT クレーム偽造 | `tenant_id` クレームが改ざんされた JWT | 最高 |
   | B: ABAC バイパス | 認可ロジックのバグで tenant_id 検証をスキップ | 高 |
   | C: DB RLS バイパス | PostgreSQL Row-Level Security が機能していない | 高 |
   | D: 設定ミス | 誤った namespace / ラベルで他テナントデータにアクセス | 中 |

2. 越境した tenant_id ペアを特定する。

   ```bash
   logcli query '{namespace="k1s0-tier1", job="audit"}
     |= "cross_tenant" | json
     | line_format "{{.source_tenant}} -> {{.target_tenant}} ({{.user_id}})"' \
     --since=2h | sort | uniq
   ```

3. SPIFFE ID で対象 Pod の ID を確認する。

   ```bash
   kubectl get pod -n k1s0-tier1 -o jsonpath='{range .items[*]}
   {.metadata.name}{"\t"}{.metadata.annotations.spiffe\.io/spiffe-id}{"\n"}{end}'
   ```

### Step 2: 侵害 Pod / サービスの隔離（〜1 時間）

4. パターン A（JWT 偽造）の場合: Keycloak でセッションを即時無効化する。

   ```bash
   # 対象 client の全セッションを無効化
   kcadm.sh delete clients/<client-id>/user-sessions -r k1s0
   # JWT 署名鍵をローテーション（OIDC discovery で全クライアントが新鍵を取得）
   kcadm.sh create realms/k1s0/keys -s enabled=true -s providerId=rsa-generated
   ```

5. パターン B/C（バグ起因）の場合: 影響サービスをスケールダウンする。

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

6. パターン D（設定ミス）の場合: 誤ったラベルを修正し、Pod を再起動する。

   ```bash
   kubectl patch deployment/<service> -n k1s0-tier1 \
     --type='json' \
     -p='[{"op":"replace","path":"/spec/template/metadata/labels/tenant_id",
           "value":"<correct-tenant-id>"}]'
   ```

### Step 3: 影響範囲の確定（〜2 時間）

7. 越境によって閲覧・変更された可能性があるデータを特定する。

   ```bash
   # PostgreSQL の監査ログ（pgaudit）から対象 tenant のクエリを抽出
   kubectl exec -it k1s0-pg-0 -n k1s0-data -- psql -U postgres -c "
     SELECT log_time, command_tag, database_name, object_name, statement
     FROM pgaudit_log
     WHERE log_time > NOW() - INTERVAL '3 hours'
       AND statement LIKE '%<target-tenant-id>%'
     ORDER BY log_time DESC LIMIT 100;"
   ```

8. PII が閲覧された可能性がある場合は `pii-leak-detection.md` を並行起動する。

## 3. 復旧 (Recovery)

1. バグ修正 PR をホットフィックスブランチで作成し、4-eyes レビュー後に緊急デプロイする。
2. Kyverno ポリシーで `tenant_id` ラベルの必須化を強制する。

   ```bash
   kubectl get clusterpolicy k1s0-require-tenant-label -o yaml | grep -A5 "spec:"
   ```

3. 影響テナントの採用組織担当者に越境の事実と影響データを報告する。
4. 修正後、Litmus Chaos で越境試行のテストを実行して修正を確認する。

## 4. 原因調査 (Root Cause Analysis)

- JWT 検証コードの全コードパスを trace し、`tenant_id` 検証がスキップされる条件を探す。
- PostgreSQL RLS ポリシー定義を確認する。

  ```bash
  kubectl exec -it k1s0-pg-0 -n k1s0-data -- psql -U postgres -c "
    SELECT schemaname, tablename, policyname, permissive, roles, qual
    FROM pg_policies WHERE tablename LIKE 'tenant_%';"
  ```

- Istio AuthorizationPolicy の `jwt` フィルターが全 API パスに適用されているか確認する。

## 5. 事後処理 (Post-incident)

- ポストモーテム作成（24h 以内）
- NFR-E-AC-003 に基づく Litmus Chaos 越境テストの CI 組み込み PR
- 越境試行を Grafana ダッシュボードに可視化する（`cross_tenant_attempts` メトリクス）
- PII 閲覧が確認された場合: `pii-regulatory-disclosure.md` を起動する

## 関連

- 関連設計書: docs/03_要件定義/30_非機能要件/E_セキュリティ.md (NFR-E-AC-003)
- 関連設計書: docs/03_要件定義/30_非機能要件/G_データ保護とプライバシー.md (NFR-G-AC-001)
- 関連 ADR: ADR-SEC-001 (Keycloak), ADR-SEC-002 (OpenBao), ADR-SEC-003 (SPIRE)
- 関連 Runbook: pii-leak-detection.md, escalation-contacts.md, auth-abuse-detection.md
