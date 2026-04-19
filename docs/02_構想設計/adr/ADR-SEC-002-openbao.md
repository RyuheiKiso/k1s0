# ADR-SEC-002: Secrets 管理に OpenBao を採用

- ステータス: Accepted
- 起票日: 2026-04-19
- 決定日: 2026-04-19
- 起票者: kiso ryuhei
- 関係者: システム基盤チーム / セキュリティチーム / 法務部 / 調達部

## コンテキスト

k1s0 は Secrets（DB パスワード、API キー、TLS 秘密鍵、暗号化鍵、JWT 署名鍵等）を大量に扱う。これらを Kubernetes Secret（base64 エンコードのみで暗号化されていない）で管理するのはセキュリティリスクが大きく、専用の Secrets 管理基盤が必要。

従来この用途は HashiCorp Vault が業界デファクトだったが、2023 年 8 月に HashiCorp が Vault を含む全製品を BUSL（Business Source License）に変更。BUSL は「同じ機能を提供する競合サービスの商用提供」を禁じる条項を含み、k1s0 のような PaaS での組込みが法務・事業リスクになる。

2023 年 11 月、Linux Foundation が Vault のフォーク「OpenBao」を設立、MPL-2.0 で OSS 開発継続。IBM、Oracle、数社が支援表明。2024 年に 2.0 リリース、以降活発に開発継続。

## 決定

**Secrets 管理は OpenBao（MPL-2.0、Linux Foundation）を採用する。**

- OpenBao 2.x（Vault 1.14 系互換）
- HA 構成（Raft ストレージバックエンド、3 ノード以上）
- Auto-unseal は社内 HSM または Transit Engine（ダブル暗号化）
- Audit Device で全 API 操作を監査ログに記録
- Kubernetes 統合は Secrets Store CSI Driver + OpenBao Auth Method (kubernetes)
- tier1 Secrets API の内部実装バックエンドとして利用
- JWT 署名鍵、TLS 証明書鍵、DB パスワード、暗号化鍵（ADR-SEC で言及）をすべて集約

将来 Vault が OSS 回帰した場合、または OpenBao が方向性を失った場合の切替え可能性を維持するため、アプリ側は Vault/OpenBao 共通の API のみ使用、固有拡張機能は避ける。

## 検討した選択肢

### 選択肢 A: OpenBao（採用）

- 概要: Linux Foundation 傘下の Vault フォーク、MPL-2.0
- メリット:
  - 真の OSS ライセンス（MPL-2.0）、ベンダーロックなし
  - Vault 1.14 完全互換、既存クライアント・CLI・UI がほぼそのまま動く
  - 全 Vault 機能（KV、Transit、PKI、Database、AWS/K8s Auth Method 等）を継承
  - Audit Device、Namespace 機能で監査・マルチテナント対応
  - Linux Foundation ガバナンスで中立性高い
- デメリット:
  - 2023 年末発足で運用実績がまだ積上げ段階
  - Vault Enterprise の一部機能（HSM 管理、Replication 等）は未対応
  - コミュニティが Vault から移行中、過渡期

### 選択肢 B: HashiCorp Vault (BUSL)

- 概要: Vault 本家の最新版
- メリット: ドキュメント豊富、既存ノウハウそのまま
- デメリット:
  - BUSL ライセンスが k1s0 の商用利用と抵触する可能性
  - 法務リスクを抱えた運用
  - 将来のライセンスさらなる変更リスク

### 選択肢 C: Kubernetes Secrets + SOPS/age + External Secrets Operator

- 概要: K8s Secret を GitOps で暗号化管理
- メリット: シンプル、Vault サーバ不要
- デメリット:
  - Dynamic Secrets（DB パスワードの自動発行・ローテーション）不可
  - PKI 機能なし、TLS 証明書自動発行ができない
  - 監査ログが K8s Audit Log に依存、粒度が粗い

### 選択肢 D: Infisical / Bitwarden Secrets Manager

- 概要: モダンな Secrets 管理 OSS
- メリット: UI が洗練、チーム運用向け
- デメリット:
  - Vault クラスの機能（PKI、Database、Transit Engine）が薄い
  - エンタープライズ採用実績が乏しい

### 選択肢 E: 商用 KMS（AWS KMS、Azure Key Vault 等）

- 概要: クラウド KMS
- メリット: マネージドで運用工数小
- デメリット: オンプレ制約で選択肢外

## 帰結

### ポジティブな帰結

- ライセンスリスクの構造的回避（MPL-2.0）
- Vault 互換で既存ノウハウ・ツール資産を活用
- 将来 Vault 回帰・他選択肢への切替え可能性を維持
- Dynamic Secrets、PKI、Transit Engine 等の強力な機能群
- 全 Secrets を一元管理、監査ログの完整性担保

### ネガティブな帰結

- OpenBao の運用実績が Vault より薄い（2026 時点で 2〜3 年）
- HSM 連携等の Enterprise 機能はコミュニティ版では未成熟
- unseal 手順、鍵ローテーション手順の整備が必要（運用 Runbook）
- 初期セットアップの学習曲線が急（root token、policy、auth method の理解）

## 実装タスク

- OpenBao Helm Chart バージョン固定、Argo CD 管理
- Raft HA 構成（3 ノード）で PersistentVolume 付与
- Auto-unseal 設定（Transit Engine または HSM）
- Kubernetes Auth Method 設定、ServiceAccount 経由で各 Pod が Secrets 取得
- Policy テンプレート（テナント用、運用用、開発用）を Backstage で管理
- Audit Device を Loki に連携、異常検知アラート
- 鍵ローテーション手順 Runbook、四半期訓練
- 切替えシミュレーション（OpenBao → Vault 回帰、または別 Secrets Manager）を年次検証

## 参考文献

- OpenBao 公式: openbao.org
- Linux Foundation OpenBao Announcement (2023-11)
- HashiCorp BUSL 変更の影響分析
- MPL-2.0 本文
- NIST SP 800-57 Part 1: Key Management Recommendations
