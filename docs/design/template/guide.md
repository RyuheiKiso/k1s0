# テンプレート追加ガイド

← [テンプレート設計書](./)

---

## 新しいテンプレートを追加する手順

1. **ディレクトリ作成**
   ```
   CLI/templates/{template-name}/feature/
   ```

2. **必須ファイルの配置**
   - `.k1s0/manifest.json.tera`
   - メイン設定ファイル（`Cargo.toml.tera`, `package.json.tera` など）
   - エントリーポイント（`main.rs.tera`, `main.go.tera` など）

3. **Clean Architecture 構造の作成**
   ```
   src/
   ├── domain/
   ├── application/
   ├── presentation/
   └── infrastructure/
   ```

4. **ServiceType への追加**
   `CLI/crates/k1s0-cli/src/commands/new_feature.rs` に追加:
   ```rust
   pub enum ServiceType {
       // ...
       #[value(name = "template-name")]
       TemplateName,
   }
   ```

5. **RequiredFiles への追加**
   `CLI/crates/k1s0-generator/src/lint/required_files.rs` に追加

6. **テスト**
   ```bash
   k1s0 new-feature -t template-name -n test-service
   k1s0 lint feature/{type}/{lang}/test-service
   ```
