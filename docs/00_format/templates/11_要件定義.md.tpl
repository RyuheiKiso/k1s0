---
id: req.<axis>.<slug>
axis: <axis>
phase: requirement
kind: requirement
status: draft
depends_on: []
covered_by:
  defense_in_depth_layers: []
  proof_classes: []
---

# <タイトル>

## 一文方針
- <この要件定義ドキュメントの核心を 1 文で>。

## スコープ

### 提供範囲
- <要件 1>
- <要件 2>

### 非提供範囲
- <要件 1>: 理由 <理由>
- <要件 2>: 理由 <理由>

## 機能要件 / 非機能要件 / 制約

### 機能要件
- <要件 ID>: <要件 statement>
    - 検証手段: <検証方法 / 該当 spec への link>
    - 受入基準: <測定可能な基準>

### 非機能要件
- <要件 ID>: <要件 statement>
    - SLO target: <測定可能な値>（→ [SLO 適合仕様](../../04_詳細設計/01_適合仕様/07_SLO適合仕様.md)）
    - 受入基準: <測定可能な基準>

### 制約
- <制約 1>: <statement> + <根拠 + 受入>

## 採用 OSS / 技術選定
- <要件を満たす OSS / 技術>: <根拠>
- 採用しない選択肢: <列挙 + 理由>

## 受入条件（1.0.0 ship blocker）
- 全要件が `release_gate.lock.yaml` の cell として登録済み
- 全要件が <axis> の conformance spec / enforcement / proof certificate のいずれかに bind 済み
- TBD / 未定 / 後述 が `status: locked` ドキュメント中にゼロ

## 至高路線における判断
- <この要件が「機能削減なし」「至高」と整合する根拠>。

## 関連参照
- [<企画側の動機>](../../01_企画/<相対パス>.md)
- [<概要設計側の方針>](../../03_概要設計/<相対パス>.md)
- [<詳細設計側の適合仕様 / 強制機構>](../../04_詳細設計/<相対パス>.md)
