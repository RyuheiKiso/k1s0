"""CLIフロー.md の仕様準拠テスト。

CLI のソースコード (Rust) がフロー仕様ドキュメントと
一致するかを検証する。
"""
from pathlib import Path

import pytest

ROOT = Path(__file__).resolve().parents[3]
CLI_SRC = ROOT / "CLI" / "src"


class TestMainMenu:
    """CLIフロー.md: メインメニューの選択肢が仕様通りか。"""

    def setup_method(self) -> None:
        self.content = (CLI_SRC / "main.rs").read_text(encoding="utf-8")

    def test_dialoguer_import(self) -> None:
        """CLIフロー.md: すべての操作は dialoguer によるプロンプトを通じて行う。"""
        assert "dialoguer" in self.content

    def test_menu_items_project_init(self) -> None:
        assert "プロジェクト初期化" in self.content

    def test_menu_items_scaffold(self) -> None:
        assert "ひな形生成" in self.content

    def test_menu_items_build(self) -> None:
        assert "ビルド" in self.content

    def test_menu_items_test(self) -> None:
        assert "テスト実行" in self.content

    def test_menu_items_deploy(self) -> None:
        assert "デプロイ" in self.content

    def test_menu_items_exit(self) -> None:
        assert "終了" in self.content

    def test_menu_item_count(self) -> None:
        """CLIフロー.md: メニューは6項目。"""
        assert "MENU_ITEMS" in self.content
        for item in ["プロジェクト初期化", "ひな形生成", "ビルド", "テスト実行", "デプロイ", "終了"]:
            assert item in self.content

    def test_ctrlc_handler(self) -> None:
        """CLIフロー.md: メインメニューで Ctrl+C を押すと CLI を終了する。"""
        assert "ctrlc" in self.content

    def test_interact_opt(self) -> None:
        """CLIフロー.md: Ctrl+C でメインメニューに戻る（interact_opt で None 処理）。"""
        assert "interact_opt" in self.content


class TestProjectInit:
    """CLIフロー.md: プロジェクト初期化フローの検証。"""

    def setup_method(self) -> None:
        self.content = (CLI_SRC / "commands" / "init.rs").read_text(encoding="utf-8")

    def test_init_config_project_name(self) -> None:
        """CLIフロー.md: [1] プロジェクト名を入力してください。"""
        assert "project_name" in self.content

    def test_init_config_git_init(self) -> None:
        """CLIフロー.md: [2] Git リポジトリを初期化しますか？"""
        assert "git_init" in self.content

    def test_init_config_sparse_checkout(self) -> None:
        """CLIフロー.md: [3] sparse-checkout を有効にしますか？"""
        assert "sparse_checkout" in self.content

    def test_tier_selection(self) -> None:
        """CLIフロー.md: [4] チェックアウトするTierを選択。"""
        assert "Tier" in self.content
        assert "System" in self.content
        assert "Business" in self.content
        assert "Service" in self.content

    def test_step_enum(self) -> None:
        """CLIフロー.md: ステップベースの対話フロー。"""
        assert "ProjectName" in self.content
        assert "GitInit" in self.content
        assert "SparseCheckout" in self.content
        assert "TierSelection" in self.content
        assert "Confirm" in self.content


class TestGenerateKinds:
    """CLIフロー.md: ひな形生成 — 種別の選択。"""

    def setup_method(self) -> None:
        self.content = (CLI_SRC / "commands" / "generate.rs").read_text(encoding="utf-8")

    def test_kind_server(self) -> None:
        assert "サーバー" in self.content

    def test_kind_client(self) -> None:
        assert "クライアント" in self.content

    def test_kind_library(self) -> None:
        assert "ライブラリ" in self.content

    def test_kind_database(self) -> None:
        assert "データベース" in self.content


class TestGenerateTierRestrictions:
    """CLIフロー.md: 種別に応じたTier制限の検証。"""

    def setup_method(self) -> None:
        self.content = (CLI_SRC / "commands" / "generate.rs").read_text(encoding="utf-8")

    def test_server_all_tiers(self) -> None:
        """CLIフロー.md: サーバーは system/business/service。"""
        assert "Kind::Server => vec![Tier::System, Tier::Business, Tier::Service]" in self.content

    def test_client_no_system(self) -> None:
        """CLIフロー.md: クライアントは business/service のみ。"""
        assert "Kind::Client => vec![Tier::Business, Tier::Service]" in self.content

    def test_library_no_service(self) -> None:
        """CLIフロー.md: ライブラリは system/business のみ。"""
        assert "Kind::Library => vec![Tier::System, Tier::Business]" in self.content

    def test_database_all_tiers(self) -> None:
        """CLIフロー.md: データベースは system/business/service。"""
        assert "Kind::Database => vec![Tier::System, Tier::Business, Tier::Service]" in self.content


class TestGenerateLanguages:
    """CLIフロー.md: 言語 / フレームワーク選択の検証。"""

    def setup_method(self) -> None:
        self.content = (CLI_SRC / "commands" / "generate.rs").read_text(encoding="utf-8")

    def test_server_languages(self) -> None:
        """CLIフロー.md: サーバーは Go / Rust。"""
        assert 'Language::Go => "Go"' in self.content
        assert 'Language::Rust => "Rust"' in self.content

    def test_client_frameworks(self) -> None:
        """CLIフロー.md: クライアントは React / Flutter。"""
        assert 'Framework::React => "React"' in self.content
        assert 'Framework::Flutter => "Flutter"' in self.content

    def test_library_languages(self) -> None:
        """CLIフロー.md: ライブラリは Go / Rust / TypeScript / Dart。"""
        assert 'Language::TypeScript => "TypeScript"' in self.content
        assert 'Language::Dart => "Dart"' in self.content

    def test_rdbms_options(self) -> None:
        """CLIフロー.md: RDBMS は PostgreSQL / MySQL / SQLite。"""
        assert 'Rdbms::PostgreSQL => "PostgreSQL"' in self.content
        assert 'Rdbms::MySQL => "MySQL"' in self.content
        assert 'Rdbms::SQLite => "SQLite"' in self.content


class TestGenerateApiStyles:
    """CLIフロー.md: API方式選択の検証。"""

    def setup_method(self) -> None:
        self.content = (CLI_SRC / "commands" / "generate.rs").read_text(encoding="utf-8")

    def test_rest_option(self) -> None:
        assert "REST (OpenAPI)" in self.content

    def test_grpc_option(self) -> None:
        assert "gRPC (protobuf)" in self.content

    def test_graphql_option(self) -> None:
        assert "GraphQL" in self.content


class TestNameValidation:
    """CLIフロー.md: 入力のバリデーション。"""

    def setup_method(self) -> None:
        self.content = (CLI_SRC / "prompt" / "mod.rs").read_text(encoding="utf-8")

    def test_regex_pattern(self) -> None:
        """CLIフロー.md: [a-z0-9-]+ のみ許可。"""
        assert "a-z0-9" in self.content

    def test_leading_trailing_hyphen_forbidden(self) -> None:
        """CLIフロー.md: 先頭と末尾のハイフンは禁止する。"""
        assert "先頭末尾のハイフンは禁止" in self.content

    def test_validate_name_function_exists(self) -> None:
        assert "fn validate_name" in self.content

    def test_validate_name_used_in_input(self) -> None:
        """CLIフロー.md: 入力プロンプトでバリデーションが適用される。"""
        assert "validate_name" in self.content


class TestPromptUtilities:
    """CLIフロー.md: 共通プロンプトの検証。"""

    def setup_method(self) -> None:
        self.content = (CLI_SRC / "prompt" / "mod.rs").read_text(encoding="utf-8")

    def test_yes_no_prompt(self) -> None:
        """CLIフロー.md: はい/いいえ プロンプト。"""
        assert "fn yes_no_prompt" in self.content
        assert "はい" in self.content
        assert "いいえ" in self.content

    def test_confirm_prompt(self) -> None:
        """CLIフロー.md: 確認プロンプト（3択）。"""
        assert "fn confirm_prompt" in self.content
        assert "前のステップに戻る" in self.content
        assert "メインメニューに戻る" in self.content

    def test_confirm_result_enum(self) -> None:
        """CLIフロー.md: 確認結果の3パターン。"""
        assert "ConfirmResult" in self.content
        assert "Yes" in self.content
        assert "GoBack" in self.content
        assert "Cancel" in self.content

    def test_multi_select_prompt(self) -> None:
        """CLIフロー.md: 複数選択プロンプト。"""
        assert "fn multi_select_prompt" in self.content
        assert "MultiSelect" in self.content


class TestBuildFlow:
    """CLIフロー.md: ビルドフローの検証。"""

    def setup_method(self) -> None:
        self.content = (CLI_SRC / "commands" / "build.rs").read_text(encoding="utf-8")

    def test_build_mode_development(self) -> None:
        assert '"development"' in self.content

    def test_build_mode_production(self) -> None:
        assert '"production"' in self.content

    def test_target_selection(self) -> None:
        """CLIフロー.md: ビルド対象を選択してください。"""
        assert "ビルド対象を選択してください" in self.content

    def test_all_option(self) -> None:
        """CLIフロー.md: 「すべて」選択肢がある。"""
        assert "すべて" in self.content

    def test_scan_buildable_targets(self) -> None:
        """CLIフロー.md: 既存のサーバー・クライアント・ライブラリが一覧表示される。"""
        assert "fn scan_buildable_targets" in self.content


class TestTestFlow:
    """CLIフロー.md: テスト実行フローの検証。"""

    def setup_method(self) -> None:
        self.content = (CLI_SRC / "commands" / "test_cmd.rs").read_text(encoding="utf-8")

    def test_test_kinds(self) -> None:
        """CLIフロー.md: テスト種別の4選択肢。"""
        assert "ユニットテスト" in self.content
        assert "統合テスト" in self.content
        assert "E2Eテスト" in self.content
        assert "すべて" in self.content


class TestDeployFlow:
    """CLIフロー.md: デプロイフローの検証。"""

    def setup_method(self) -> None:
        self.content = (CLI_SRC / "commands" / "deploy.rs").read_text(encoding="utf-8")

    def test_environments(self) -> None:
        """CLIフロー.md: 環境は dev / staging / prod。"""
        assert '"dev"' in self.content
        assert '"staging"' in self.content
        assert '"prod"' in self.content

    def test_prod_confirmation(self) -> None:
        """CLIフロー.md: prod 環境の追加確認ステップ。"""
        assert "ProdConfirm" in self.content

    def test_is_prod_method(self) -> None:
        assert "fn is_prod" in self.content


class TestCLIDependencies:
    """CLIフロー.md: CLI の依存関係が仕様通りか。"""

    def setup_method(self) -> None:
        self.content = (ROOT / "CLI" / "Cargo.toml").read_text(encoding="utf-8")

    def test_dialoguer_dependency(self) -> None:
        """CLIフロー.md: dialoguer によるプロンプト。"""
        assert "dialoguer" in self.content

    def test_tera_dependency(self) -> None:
        """テンプレートエンジン仕様: Tera テンプレートエンジン。"""
        assert "tera" in self.content

    def test_ctrlc_dependency(self) -> None:
        """CLIフロー.md: Ctrl+C ハンドリング。"""
        assert "ctrlc" in self.content

    def test_regex_dependency(self) -> None:
        """CLIフロー.md: 名前バリデーション用 regex。"""
        assert "regex" in self.content

    def test_serde_dependency(self) -> None:
        assert "serde" in self.content

    def test_heck_dependency(self) -> None:
        """命名変換用 (snake_case 等)。"""
        assert "heck" in self.content


class TestCLIModuleStructure:
    """CLIフロー.md: CLI のモジュール構成が仕様通りか。"""

    def test_main_rs_modules(self) -> None:
        content = (CLI_SRC / "main.rs").read_text(encoding="utf-8")
        assert "mod commands" in content
        assert "mod config" in content
        assert "mod prompt" in content
        assert "mod template" in content

    def test_commands_mod(self) -> None:
        content = (CLI_SRC / "commands" / "mod.rs").read_text(encoding="utf-8")
        assert "pub mod init" in content
        assert "pub mod generate" in content
        assert "pub mod build" in content
        assert "pub mod deploy" in content
        assert "pub mod test_cmd" in content

    @pytest.mark.parametrize(
        "module",
        ["init.rs", "generate.rs", "build.rs", "test_cmd.rs", "deploy.rs"],
    )
    def test_command_module_exists(self, module: str) -> None:
        assert (CLI_SRC / "commands" / module).exists()

    def test_prompt_module_exists(self) -> None:
        assert (CLI_SRC / "prompt" / "mod.rs").exists()

    def test_config_module_exists(self) -> None:
        assert (CLI_SRC / "config" / "mod.rs").exists()

    def test_template_module_exists(self) -> None:
        assert (CLI_SRC / "template" / "mod.rs").exists()


# ============================================================================
# CLIフロー.md ギャップ補完テスト
# ============================================================================


class TestEscKeyStepBack:
    """CLIフロー.md: Esc キーで前のステップに戻る動作。

    対話式 CLI の Esc 動作は E2E で直接テストが困難なため、
    Rust ソースコードにステップ戻りロジックが実装されていることを検証する。
    """

    def test_generate_esc_back_to_kind(self) -> None:
        """CLIフロー.md: generate でステップ戻りロジックが存在する。"""
        content = (CLI_SRC / "commands" / "generate.rs").read_text(encoding="utf-8")
        # Esc/None で前のステップに戻る処理（Step::Kind への戻り等）
        assert "Step::Kind" in content
        assert "Step::Tier" in content
        assert "Step::Placement" in content
        assert "StepResult::Back" in content

    def test_init_step_back(self) -> None:
        """CLIフロー.md: init でステップ戻りロジックが存在する。"""
        content = (CLI_SRC / "commands" / "init.rs").read_text(encoding="utf-8")
        # Esc で前のステップに戻る: Step enum と戻り処理
        assert "Step::" in content
        assert "GoBack" in content or "step =" in content

    def test_deploy_esc_back_to_environment(self) -> None:
        """CLIフロー.md: deploy でステップ戻りロジックが存在する。"""
        content = (CLI_SRC / "commands" / "deploy.rs").read_text(encoding="utf-8")
        assert "Step::Environment" in content
        assert "Step::Targets" in content
        # Esc で Environment に戻る
        assert "step = Step::Environment" in content


class TestCtrlCMainMenuReturn:
    """CLIフロー.md: Ctrl+C でメインメニューに戻る。

    すべてのプロンプトで Ctrl+C を押すとメインメニューに戻る。
    interact_opt で None を返すことで Ctrl+C をハンドリングする。
    """

    def test_main_ctrlc_handler_exists(self) -> None:
        """CLIフロー.md: ctrlc ハンドラが main.rs に定義されている。"""
        content = (CLI_SRC / "main.rs").read_text(encoding="utf-8")
        assert "ctrlc_handler" in content
        assert "ctrlc::set_handler" in content

    def test_generate_uses_prompt_module(self) -> None:
        """CLIフロー.md: generate.rs が prompt モジュール経由で Ctrl+C をハンドリングする。"""
        content = (CLI_SRC / "commands" / "generate.rs").read_text(encoding="utf-8")
        # prompt モジュール（内部で interact_opt を使用）を利用
        assert "prompt::" in content

    def test_prompt_interact_opt(self) -> None:
        """CLIフロー.md: prompt/mod.rs で interact_opt を使用している。"""
        content = (CLI_SRC / "prompt" / "mod.rs").read_text(encoding="utf-8")
        assert "interact_opt" in content

    def test_deploy_ctrlc_return(self) -> None:
        """CLIフロー.md: deploy で Ctrl+C/Esc 時に return Ok(()) で抜ける。"""
        content = (CLI_SRC / "commands" / "deploy.rs").read_text(encoding="utf-8")
        assert "return Ok(())" in content


class TestGenerateDuplicateCheck:
    """CLIフロー.md: ひな形生成での重複チェック。

    既存の名前との重複はエラーとする。
    """

    def test_duplicate_check_in_generate(self) -> None:
        """CLIフロー.md: generate.rs に重複チェックロジックが存在する。"""
        content = (CLI_SRC / "commands" / "generate.rs").read_text(encoding="utf-8")
        assert "既に存在します" in content

    def test_validate_name_in_prompt_name_or_select(self) -> None:
        """CLIフロー.md: prompt_name_or_select で名前バリデーション + 重複チェック。"""
        content = (CLI_SRC / "commands" / "generate.rs").read_text(encoding="utf-8")
        assert "validate_name" in content
        assert "existing_names" in content


class TestExistingDirectoryScan:
    """CLIフロー.md: 既存ディレクトリスキャン・選択肢反映。

    既存の領域/サービスが候補として表示される。
    """

    def test_scan_existing_dirs_function(self) -> None:
        """CLIフロー.md: scan_existing_dirs 関数が存在する。"""
        content = (CLI_SRC / "commands" / "generate.rs").read_text(encoding="utf-8")
        assert "fn scan_existing_dirs" in content

    def test_business_existing_scan(self) -> None:
        """CLIフロー.md: business Tier で既存領域をスキャンする。"""
        content = (CLI_SRC / "commands" / "generate.rs").read_text(encoding="utf-8")
        assert 'scan_existing_dirs("regions/business")' in content

    def test_service_existing_scan(self) -> None:
        """CLIフロー.md: service Tier で既存サービスをスキャンする。"""
        content = (CLI_SRC / "commands" / "generate.rs").read_text(encoding="utf-8")
        assert 'scan_existing_dirs("regions/service")' in content

    def test_new_or_select_prompt(self) -> None:
        """CLIフロー.md: (新規作成) と既存候補の選択肢が表示される。"""
        content = (CLI_SRC / "commands" / "generate.rs").read_text(encoding="utf-8")
        assert "(新規作成)" in content


class TestExistingDatabaseDisplay:
    """CLIフロー.md: 既存DB表示テスト。

    既存のデータベースがRDBMS付きで表示される。
    """

    def test_scan_existing_databases_function(self) -> None:
        """CLIフロー.md: scan_existing_databases 関数が存在する。"""
        content = (CLI_SRC / "commands" / "generate.rs").read_text(encoding="utf-8")
        assert "fn scan_existing_databases" in content

    def test_db_display_with_rdbms(self) -> None:
        """CLIフロー.md: DbInfo の Display 実装が名前(RDBMS)形式。"""
        content = (CLI_SRC / "commands" / "generate.rs").read_text(encoding="utf-8")
        # DbInfo の Display: "{} ({})" format
        assert "DbInfo" in content
        assert 'write!(f, "{} ({})"' in content


class TestE2ESuiteSelection:
    """CLIフロー.md: E2Eスイート選択テスト。

    E2Eテストの場合、テストスイート単位で選択できる。
    """

    def test_scan_e2e_suites_function(self) -> None:
        """CLIフロー.md: scan_e2e_suites 関数が存在する。"""
        content = (CLI_SRC / "commands" / "test_cmd.rs").read_text(encoding="utf-8")
        assert "fn scan_e2e_suites" in content

    def test_e2e_suite_selection_prompt(self) -> None:
        """CLIフロー.md: テストスイートを選択してください。"""
        content = (CLI_SRC / "commands" / "test_cmd.rs").read_text(encoding="utf-8")
        assert "テストスイートを選択してください" in content

    def test_e2e_kind_routes_to_suite_selection(self) -> None:
        """CLIフロー.md: E2E 選択時は step_select_e2e_suites を呼び出す。"""
        content = (CLI_SRC / "commands" / "test_cmd.rs").read_text(encoding="utf-8")
        assert "step_select_e2e_suites" in content


class TestProdDeployConfirmation:
    """CLIフロー.md: prod "deploy" 入力確認テスト。

    prod を選択した場合のみ "deploy" と入力する追加確認ステップがある。
    """

    def test_prod_confirm_step(self) -> None:
        """CLIフロー.md: ProdConfirm ステップが存在する。"""
        content = (CLI_SRC / "commands" / "deploy.rs").read_text(encoding="utf-8")
        assert "ProdConfirm" in content

    def test_deploy_input_prompt(self) -> None:
        """CLIフロー.md: "deploy" と入力する確認プロンプト。"""
        content = (CLI_SRC / "commands" / "deploy.rs").read_text(encoding="utf-8")
        assert '"deploy"' in content
        assert 'deploy' in content

    def test_prod_only_confirmation(self) -> None:
        """CLIフロー.md: prod の場合のみ ProdConfirm に遷移する。"""
        content = (CLI_SRC / "commands" / "deploy.rs").read_text(encoding="utf-8")
        assert "env.is_prod()" in content
        assert "Step::ProdConfirm" in content
