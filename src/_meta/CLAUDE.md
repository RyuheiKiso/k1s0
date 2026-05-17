# _meta コーディングポリシー

全軸共通ルールは `src/CLAUDE.md` を参照。本ファイルは _meta 固有の制約のみ記述する。

## 配置・構成

- **axis_registry**: `src/_meta/axis_registry/registry.yaml`（手書き + validate）
- **release_gate**: `src/_meta/lock/release_gate.lock.yaml`（全 19 軸 AND-gate）
- **lock.yaml 配置先**: `src/_meta/lock/`

設計方針の詳細は `docs/04_詳細設計/01_適合仕様/00_軸登録適合仕様.md` を単一の真とする。

## コーディング制約

### lock.yaml 生成

- `axis_registry.lock.yaml` は `generate_axis_registry.py` で生成（手書き禁止）
- `release_gate.lock.yaml` は `tools/lock_yaml_generator/generate_release_gate.py` で生成（手書き禁止）
- `ownership_table.lock.yaml` は手書き + validate（dual sign-off 必須）

### 新軸追加の手順

新軸追加は以下の順序を守る（逆順不可）:

1. `docs/04_詳細設計/01_適合仕様/00_軸登録適合仕様.md` の `registry.yaml` に追記
2. dual sign-off（human × 1 + AI evidence × 1）
3. `generate_axis_registry.py` を実行して `axis_registry.lock.yaml` を更新
4. 対応する `src/<新軸>/` ディレクトリを作成
5. `tools/docs_lint/run_lint.py` の `SRC_ALLOWED_AXES` に追加（ARCHITECTURE.md も更新）

### cap 管理

- cap v1 = 20 / 現 19 / 残 1（残 1 を使い切ると cap 変更が必要 → v1 範囲での cap 変更は破壊的変更）
- 残 1 を使い切る前に「本当に必要か」を設計レビューで確認する

### release_gate

- `release_gate.lock.yaml` が全 19 軸 AND-gate green になることが v1.0.0 ship の物理前提条件
- AND-gate の cell を手動で green にすることを禁止（各軸の実装完了が cell を green にする）

## 関連参照

- `docs/04_詳細設計/01_適合仕様/00_軸登録適合仕様.md` — 軸登録 SoT
- `docs/04_詳細設計/05_lock_yaml体系/03_release_gate体系.md` — AND-gate 構造
- `tools/lock_yaml_generator/generate_release_gate.py` — release_gate 生成器
