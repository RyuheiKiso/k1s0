# SLO: 基本

- 対象読者: Web サービスの開発・運用経験がある開発者
- 学習目標: SLO の設計・導入・運用を自チームで実践できるようになる
- 所要時間: 約 30 分
- 対象バージョン: -（方法論のため特定バージョンなし）
- 最終更新日: 2026-04-13

## 1. このドキュメントで学べること

- SLI・SLO・SLA・Error Budget の関係を正確に説明できる
- CUJ に基づいて SLO を設計する手順を実践できる
- Prometheus でバーンレートアラートを実装できる
- Error Budget Policy を策定し、チームで運用する方法を理解できる

## 2. 前提知識

- SRE の基本概念を理解していること
  - 参照: [SRE: 基本](./sre_basics.md)
- Prometheus と Grafana の基本的な操作
- YAML の基本記法

## 3. 概要

SLO（Service Level Objective: サービスレベル目標）は、サービスの信頼性に対する目標値である。「このサービスはどの程度の信頼性を目指すか」をデータで定義し、開発速度と信頼性のバランスを定量的に管理する仕組みである。

SLO が重要な理由は 3 つある。第一に、信頼性の議論を「感覚」から「データ」に変える。「最近よく落ちる」ではなく「今月の Error Budget 消費率は 80%」と数値で会話できる。第二に、開発チームと運用チームの間の意思決定基準を統一する。Error Budget が残っていれば積極的にリリースし、不足していれば信頼性改善を優先するという明確なルールができる。第三に、過剰な信頼性投資を防ぐ。100% の信頼性はコストが膨大であり、ユーザーが実際に求める水準に合わせて投資を最適化できる。

## 4. 用語の整理

| 用語 | 説明 |
|------|------|
| CUJ（Critical User Journey） | ユーザー視点で重要な操作フロー。ログイン、商品購入、検索など |
| SLI（Service Level Indicator） | サービスの信頼性を計測する定量的な指標。リクエスト成功率やレイテンシなど |
| SLO（Service Level Objective） | SLI に対する目標値。例: 可用性 99.9%、p99 レイテンシ 300ms 以下 |
| SLA（Service Level Agreement） | SLO を基に顧客と結ぶ契約。SLO 未達時の補償（返金等）を定める |
| Error Budget | SLO が許容する障害の量。100% − SLO で算出する |
| バーンレート | Error Budget の消費速度。1x は SLO 期間でちょうど使い切る速度 |
| Error Budget Policy | Error Budget の消費状況に応じた対応ルールをチームで合意した文書 |

## 5. 仕組み・アーキテクチャ

SLO は CUJ → SLI → SLO → Error Budget という段階で構成される。各概念は前段の定義に依存しており、この順序で設計する必要がある。

![SLO の構成要素と関係](./img/slo_basics_components.svg)

CUJ からユーザーにとって重要な操作を特定し、それを計測する SLI を選定する。SLI に目標値（SLO）を設定し、その余裕分が Error Budget となる。SLO を基に顧客契約（SLA）を結ぶ場合は、SLO より緩い値を設定するのが一般的である。Error Budget の残量に応じて、機能開発を加速するか信頼性改善を優先するかを判断する。

SLO の導入は以下の 5 ステップで進める。

![SLO 定義プロセス](./img/slo_basics_process.svg)

ステップ④の Error Budget Policy が特に重要である。Budget が一定以上消費された場合の対応（リリース凍結、障害対応の優先等）を事前に合意しておくことで、有事の際に迅速な意思決定ができる。

## 6. 環境構築

### 6.1 必要なもの

- Prometheus（メトリクス収集・アラート評価）
- Grafana（SLO ダッシュボード）
- Alertmanager（アラート通知）

### 6.2 SLI の計測準備

サービスが Prometheus メトリクスを公開していることを前提とする。以下の 2 つのメトリクスが最低限必要である。

- `http_requests_total`: HTTP リクエスト総数（ステータスコード別）
- `http_request_duration_seconds`: HTTP リクエスト処理時間

### 6.3 動作確認

Prometheus の UI で以下のクエリが値を返すことを確認する。

```promql
# 直近5分間のリクエスト成功率を確認する
sum(rate(http_requests_total{status!~"5.."}[5m])) / sum(rate(http_requests_total[5m]))
```

## 7. 基本の使い方

可用性 99.9% の SLO を Prometheus Recording Rule で実装する最小構成を示す。

```yaml
# Prometheus Recording Rule 定義ファイル
# SLI を定期的に事前計算して SLO 監視に使用する
groups:
  # SLO 用の Recording Rule グループを定義する
  - name: slo-recording-rules
    rules:
      # 直近5分間のエラー率を事前計算する
      - record: slo:error_rate:ratio_rate5m
        # 5xx レスポンスの割合を計算する
        expr: |
          sum(rate(http_requests_total{status=~"5.."}[5m]))
          /
          sum(rate(http_requests_total[5m]))
      # 直近1時間のエラー率を事前計算する
      - record: slo:error_rate:ratio_rate1h
        # 1時間窓でのエラー率を計算する
        expr: |
          sum(rate(http_requests_total{status=~"5.."}[1h]))
          /
          sum(rate(http_requests_total[1h]))
```

### 解説

- `record`: クエリ結果を新しいメトリクス名で保存する。`slo:` プレフィックスで SLO 関連であることを示す
- `ratio_rate5m`: 5 分間のレートで算出した比率であることを命名規則で表現する
- Recording Rule を使う理由は、アラートやダッシュボードで同じ計算を繰り返さずに済むためである

## 8. ステップアップ

### 8.1 バーンレートアラート

従来のエラー率閾値アラートは、短時間のスパイクで誤報が多発するか、検知が遅れるかのどちらかになりやすい。バーンレートアラートはこの問題を解決する。バーンレート（消費速度）は Error Budget をどれだけ速く消費しているかを示す倍率である。

| バーンレート | Budget 消費期間（30 日 SLO） | 推奨重要度 | 対応 |
|-------------|---------------------------|-----------|------|
| 14.4x | 約 2 日 | Critical | 即時対応（ページ） |
| 6x | 約 5 日 | Warning | 当日中に調査 |
| 1x | 30 日 | Ticket | 次スプリントで対応 |

バーンレートアラートの Prometheus 実装例を示す。

```yaml
# バーンレートアラート定義ファイル
# Error Budget の消費速度に基づいてアラートを発火する
groups:
  # バーンレートアラートグループを定義する
  - name: slo-burn-rate-alerts
    rules:
      # 高速バーン（14.4x）のアラートを定義する
      - alert: SLOHighBurnRate
        # 1時間のエラー率が14.4倍の閾値を超過したか判定する
        expr: slo:error_rate:ratio_rate1h > (14.4 * 0.001)
        # 5分間の持続を確認する
        for: 5m
        labels:
          # 重要度を critical に設定する
          severity: critical
        annotations:
          # アラートの要約を記載する
          summary: "SLO 高速バーン検出（14.4x）"
```

### 8.2 Error Budget Policy

Error Budget Policy は、Budget の消費状況に応じたチームの行動指針を文書化したものである。以下は策定例である。

| Budget 残量 | 状態 | チームの対応 |
|------------|------|------------|
| 75% 以上 | 正常 | 通常どおり機能開発を推進する |
| 50〜75% | 注意 | リスクの高いリリースを延期する |
| 25〜50% | 警告 | 新機能リリースを凍結し、信頼性改善に集中する |
| 25% 未満 | 危険 | 全エンジニアリングリソースを信頼性改善に投入する |

Policy の運用では以下の 3 点が重要である。チーム全員（開発・SRE・マネジメント）が合意すること、文書として残し定期的に見直すこと、Policy に従った判断を実際に実行することである。

## 9. よくある落とし穴

- **SLA と SLO を同じ値にする**: SLO は SLA より厳しく設定する。SLA 違反は契約上の補償が発生するため、SLO で早期に検知する余裕が必要である
- **内部メトリクスだけで SLI を定義する**: CPU 使用率やメモリ使用量はユーザー体験と直結しない。SLI はユーザーから見た成功・失敗で定義する
- **SLO を一度決めたら変えない**: ビジネス要件やユーザー期待は変化する。四半期ごとに SLO の妥当性を見直す
- **全サービスに同じ SLO を適用する**: サービスの重要度に応じて SLO を変える。決済システムと社内ツールでは求められる信頼性が異なる
- **Error Budget Policy を作らない**: SLO だけでは意思決定に使えない。Budget 消費時の対応ルールがなければ、結局感覚的な判断に戻る

## 10. ベストプラクティス

- SLO は少数精鋭にする。サービスあたり 3〜5 個の SLI に絞り、重要なものだけを SLO 化する
- SLA は SLO の 1 段階下に設定する（SLO 99.9% なら SLA 99.5% 等）
- バーンレートアラートを導入し、従来の静的閾値アラートを置き換える
- Error Budget の消費状況を週次でチームに共有する
- SLO は四半期ごとにレビューし、実績データに基づいて調整する
- 新サービスでは最初は緩い SLO から始め、運用データが蓄積されてから厳しくする

## 11. 演習問題

1. 自チームのサービスから CUJ を 3 つ選び、それぞれに SLI と SLO を定義せよ
2. 定義した SLO に対して、月間の Error Budget を分単位で算出せよ
3. チーム向けの Error Budget Policy を 4 段階で策定し、各段階の対応を記述せよ

## 12. さらに学ぶには

- Google SLO ガイド（The Art of SLOs）: <https://sre.google/resources/practices-and-processes/art-of-slos/>
- Google SRE Workbook Chapter 2（SLOs）: <https://sre.google/workbook/implementing-slos/>
- 関連 Knowledge: [SRE: 基本](./sre_basics.md)
- 関連 Knowledge: [Chaos Engineering: 基本](./chaos-engineering_basics.md)

## 13. 参考資料

- Google, "The Art of SLOs": <https://sre.google/resources/practices-and-processes/art-of-slos/>
- Betsy Beyer et al., "The Site Reliability Workbook", Chapter 2: Implementing SLOs, O'Reilly Media, 2018
- Google SRE Book, Chapter 4: Service Level Objectives: <https://sre.google/sre-book/service-level-objectives/>
- Alex Hidalgo, "Implementing Service Level Objectives", O'Reilly Media, 2020
