"""CI-CD設計.md の仕様準拠テスト。

.github/workflows/ のワークフロー定義がドキュメントと一致するかを検証する。
"""
from pathlib import Path

import pytest
import yaml  # type: ignore[import-untyped]

ROOT = Path(__file__).resolve().parents[3]
WORKFLOWS = ROOT / ".github" / "workflows"


class TestCIWorkflow:
    """CI-CD設計.md: ci.yaml の検証。"""

    def setup_method(self) -> None:
        path = WORKFLOWS / "ci.yaml"
        assert path.exists()
        with open(path, encoding="utf-8") as f:
            self.config = yaml.safe_load(f)

    def test_name(self) -> None:
        assert self.config["name"] == "CI"

    def test_trigger(self) -> None:
        assert "pull_request" in self.config[True]  # 'on' is True in YAML
        assert self.config[True]["pull_request"]["branches"] == ["main"]

    def test_concurrency(self) -> None:
        assert "concurrency" in self.config
        assert self.config["concurrency"]["cancel-in-progress"] is True

    def test_detect_changes_job(self) -> None:
        jobs = self.config["jobs"]
        assert "detect-changes" in jobs
        outputs = jobs["detect-changes"]["outputs"]
        for lang in ["go", "rust", "ts", "dart", "python", "helm"]:
            assert lang in outputs, f"detect-changes に {lang} の output がありません"

    def test_lint_jobs_exist(self) -> None:
        jobs = self.config["jobs"]
        for job in ["lint-go", "lint-rust", "lint-ts", "lint-dart", "lint-python"]:
            assert job in jobs, f"ジョブ {job} が存在しません"

    def test_test_jobs_exist(self) -> None:
        jobs = self.config["jobs"]
        for job in ["test-go", "test-rust", "test-ts", "test-dart", "test-python"]:
            assert job in jobs, f"ジョブ {job} が存在しません"

    def test_helm_lint_job(self) -> None:
        assert "helm-lint" in self.config["jobs"]

    def test_build_job(self) -> None:
        assert "build" in self.config["jobs"]

    def test_security_scan_job(self) -> None:
        assert "security-scan" in self.config["jobs"]

    def test_go_version(self) -> None:
        steps = self.config["jobs"]["lint-go"]["steps"]
        for step in steps:
            if step.get("uses", "").startswith("actions/setup-go"):
                assert step["with"]["go-version"] == "1.23"

    def test_rust_version(self) -> None:
        steps = self.config["jobs"]["lint-rust"]["steps"]
        for step in steps:
            if step.get("uses", "").startswith("dtolnay/rust-toolchain"):
                assert "1.82" in step["uses"]


class TestDeployWorkflow:
    """CI-CD設計.md: deploy.yaml の検証。"""

    def setup_method(self) -> None:
        path = WORKFLOWS / "deploy.yaml"
        assert path.exists()
        with open(path, encoding="utf-8") as f:
            self.config = yaml.safe_load(f)

    def test_name(self) -> None:
        assert self.config["name"] == "Deploy"

    def test_trigger(self) -> None:
        assert self.config[True]["push"]["branches"] == ["main"]

    def test_registry(self) -> None:
        assert self.config["env"]["REGISTRY"] == "harbor.internal.example.com"

    def test_deploy_jobs(self) -> None:
        jobs = self.config["jobs"]
        assert "detect-services" in jobs
        assert "build-and-push" in jobs
        assert "deploy-dev" in jobs
        assert "deploy-staging" in jobs
        assert "deploy-prod" in jobs

    def test_deploy_chain(self) -> None:
        """dev → staging → prod のデプロイチェーン。"""
        jobs = self.config["jobs"]
        assert "build-and-push" in jobs["deploy-dev"]["needs"]
        assert "deploy-dev" in jobs["deploy-staging"]["needs"]
        assert "deploy-staging" in jobs["deploy-prod"]["needs"]


class TestProtoWorkflow:
    """CI-CD設計.md: proto.yaml の検証。"""

    def setup_method(self) -> None:
        path = WORKFLOWS / "proto.yaml"
        assert path.exists()
        with open(path, encoding="utf-8") as f:
            self.config = yaml.safe_load(f)

    def test_name(self) -> None:
        assert self.config["name"] == "Proto Check"

    def test_has_lint_job(self) -> None:
        jobs = self.config["jobs"]
        # proto.yaml should have lint and breaking check
        job_names = list(jobs.keys())
        assert len(job_names) >= 1


class TestSecurityWorkflow:
    """CI-CD設計.md: security.yaml の検証。"""

    def setup_method(self) -> None:
        path = WORKFLOWS / "security.yaml"
        assert path.exists()
        with open(path, encoding="utf-8") as f:
            self.config = yaml.safe_load(f)

    def test_name(self) -> None:
        assert self.config["name"] == "Security Scan"

    def test_has_schedule(self) -> None:
        """日次スケジュールが設定されていること。"""
        triggers = self.config[True]
        assert "schedule" in triggers


class TestKongSyncWorkflow:
    """CI-CD設計.md: kong-sync.yaml の検証。"""

    def test_workflow_exists(self) -> None:
        assert (WORKFLOWS / "kong-sync.yaml").exists()

    def test_workflow_content(self) -> None:
        with open(WORKFLOWS / "kong-sync.yaml", encoding="utf-8") as f:
            config = yaml.safe_load(f)
        assert config["name"] == "Kong Config Sync"


class TestApiLintWorkflow:
    """CI-CD設計.md: api-lint.yaml の検証。"""

    def test_workflow_exists(self) -> None:
        assert (WORKFLOWS / "api-lint.yaml").exists()

    def test_workflow_content(self) -> None:
        with open(WORKFLOWS / "api-lint.yaml", encoding="utf-8") as f:
            config = yaml.safe_load(f)
        assert config["name"] == "OpenAPI Lint"


class TestCosignSignStep:
    """CI-CD設計.md: Cosign 署名ステップの検証。"""

    def setup_method(self) -> None:
        with open(WORKFLOWS / "deploy.yaml", encoding="utf-8") as f:
            self.config = yaml.safe_load(f)

    def test_cosign_installer_step(self) -> None:
        """CI-CD設計.md: Cosign インストーラーが存在。"""
        build_job = self.config["jobs"]["build-and-push"]
        steps = build_job["steps"]
        cosign_steps = [s for s in steps if "sigstore/cosign-installer" in str(s.get("uses", ""))]
        assert len(cosign_steps) >= 1

    def test_cosign_sign_step(self) -> None:
        """CI-CD設計.md: cosign sign コマンドが存在。"""
        build_job = self.config["jobs"]["build-and-push"]
        steps = build_job["steps"]
        sign_steps = [s for s in steps if "cosign sign" in str(s.get("run", ""))]
        assert len(sign_steps) >= 1


class TestImageTagFormat:
    """CI-CD設計.md: イメージタグ形式の検証。"""

    def setup_method(self) -> None:
        with open(WORKFLOWS / "deploy.yaml", encoding="utf-8") as f:
            self.config = yaml.safe_load(f)

    def test_version_tag_format(self) -> None:
        """CI-CD設計.md: {version} タグが設定されている。"""
        build_job = self.config["jobs"]["build-and-push"]
        steps = build_job["steps"]
        build_steps = [s for s in steps if s.get("uses", "").startswith("docker/build-push-action")]
        assert len(build_steps) >= 1
        tags = build_steps[0]["with"]["tags"]
        assert "version" in tags

    def test_version_sha_tag_format(self) -> None:
        """CI-CD設計.md: {version}-{sha} タグが設定されている。"""
        build_job = self.config["jobs"]["build-and-push"]
        steps = build_job["steps"]
        build_steps = [s for s in steps if s.get("uses", "").startswith("docker/build-push-action")]
        tags = build_steps[0]["with"]["tags"]
        assert "sha" in tags

    def test_latest_tag(self) -> None:
        """CI-CD設計.md: latest タグが設定されている。"""
        build_job = self.config["jobs"]["build-and-push"]
        steps = build_job["steps"]
        build_steps = [s for s in steps if s.get("uses", "").startswith("docker/build-push-action")]
        tags = build_steps[0]["with"]["tags"]
        assert "latest" in tags


class TestGitHubEnvironments:
    """CI-CD設計.md: GitHub Environments protection rules の検証。"""

    def setup_method(self) -> None:
        with open(WORKFLOWS / "deploy.yaml", encoding="utf-8") as f:
            self.config = yaml.safe_load(f)

    def test_dev_environment(self) -> None:
        """CI-CD設計.md: deploy-dev ジョブに environment: dev が設定。"""
        assert self.config["jobs"]["deploy-dev"]["environment"] == "dev"

    def test_staging_environment(self) -> None:
        """CI-CD設計.md: deploy-staging ジョブに environment: staging が設定。"""
        assert self.config["jobs"]["deploy-staging"]["environment"] == "staging"

    def test_prod_environment(self) -> None:
        """CI-CD設計.md: deploy-prod ジョブに environment: prod が設定。"""
        env = self.config["jobs"]["deploy-prod"]["environment"]
        if isinstance(env, dict):
            assert env["name"] == "prod"
        else:
            assert env == "prod"


class TestSecurityScanDependencyCheck:
    """CI-CD設計.md: Security Scan dependency-check の検証。"""

    def setup_method(self) -> None:
        with open(WORKFLOWS / "ci.yaml", encoding="utf-8") as f:
            self.config = yaml.safe_load(f)

    def test_security_scan_job_exists(self) -> None:
        assert "security-scan" in self.config["jobs"]

    def test_trivy_scan_used(self) -> None:
        """CI-CD設計.md: Trivy を使用した脆弱性スキャン。"""
        steps = self.config["jobs"]["security-scan"]["steps"]
        trivy_steps = [s for s in steps if "trivy" in str(s.get("uses", "")).lower()]
        assert len(trivy_steps) >= 1

    def test_trivy_severity_high_critical(self) -> None:
        """CI-CD設計.md: HIGH,CRITICAL レベルのスキャン。"""
        steps = self.config["jobs"]["security-scan"]["steps"]
        trivy_steps = [s for s in steps if "trivy" in str(s.get("uses", "")).lower()]
        severity = trivy_steps[0]["with"].get("severity", "")
        assert "HIGH" in severity
        assert "CRITICAL" in severity


class TestProtoCheckDetails:
    """CI-CD設計.md: Proto Check buf lint/breaking/generate の検証。"""

    def setup_method(self) -> None:
        with open(WORKFLOWS / "proto.yaml", encoding="utf-8") as f:
            self.config = yaml.safe_load(f)

    def test_buf_lint_step(self) -> None:
        """CI-CD設計.md: buf lint ステップが存在。"""
        steps = self.config["jobs"]["proto-lint"]["steps"]
        lint_steps = [s for s in steps if "buf lint" in str(s.get("run", ""))]
        assert len(lint_steps) >= 1

    def test_buf_breaking_step(self) -> None:
        """CI-CD設計.md: buf breaking ステップが存在。"""
        steps = self.config["jobs"]["proto-lint"]["steps"]
        breaking_steps = [s for s in steps if "buf breaking" in str(s.get("run", ""))]
        assert len(breaking_steps) >= 1

    def test_buf_generate_step(self) -> None:
        """CI-CD設計.md: buf generate ステップが存在。"""
        steps = self.config["jobs"]["proto-lint"]["steps"]
        gen_steps = [s for s in steps if "buf generate" in str(s.get("run", ""))]
        assert len(gen_steps) >= 1


class TestApiLintDetails:
    """CI-CD設計.md: api-lint.yaml 詳細の検証。"""

    def setup_method(self) -> None:
        with open(WORKFLOWS / "api-lint.yaml", encoding="utf-8") as f:
            self.config = yaml.safe_load(f)

    def test_openapi_validate_job(self) -> None:
        """CI-CD設計.md: OpenAPI バリデーションジョブが存在。"""
        assert "openapi-validate" in self.config["jobs"]

    def test_openapi_codegen_job(self) -> None:
        """CI-CD設計.md: コード生成ジョブが存在。"""
        assert "openapi-codegen" in self.config["jobs"]

    def test_sdk_generate_typescript_job(self) -> None:
        """CI-CD設計.md: TypeScript SDK 生成ジョブが存在。"""
        assert "sdk-generate-typescript" in self.config["jobs"]

    def test_sdk_generate_dart_job(self) -> None:
        """CI-CD設計.md: Dart SDK 生成ジョブが存在。"""
        assert "sdk-generate-dart" in self.config["jobs"]


class TestCIPathsFilter:
    """CI-CD設計.md: detect-changes の paths フィルタ内容検証。"""

    def setup_method(self) -> None:
        with open(WORKFLOWS / "ci.yaml", encoding="utf-8") as f:
            self.ci = yaml.safe_load(f)
        self.raw_content = (WORKFLOWS / "ci.yaml").read_text(encoding="utf-8")

    def test_go_paths_filter(self) -> None:
        """CI-CD設計.md: Go の paths フィルタに regions/**/go/** が含まれる。"""
        assert "regions/**/go/**" in self.raw_content

    def test_rust_paths_filter(self) -> None:
        """CI-CD設計.md: Rust の paths フィルタに regions/**/rust/** と CLI/** が含まれる。"""
        assert "regions/**/rust/**" in self.raw_content
        assert "CLI/**" in self.raw_content

    def test_ts_paths_filter(self) -> None:
        """CI-CD設計.md: TS の paths フィルタに regions/**/react/** が含まれる。"""
        assert "regions/**/react/**" in self.raw_content

    def test_dart_paths_filter(self) -> None:
        """CI-CD設計.md: Dart の paths フィルタに regions/**/flutter/** が含まれる。"""
        assert "regions/**/flutter/**" in self.raw_content

    def test_python_paths_filter(self) -> None:
        """CI-CD設計.md: Python の paths フィルタに e2e/** が含まれる。"""
        assert "e2e/**" in self.raw_content

    def test_helm_paths_filter(self) -> None:
        """CI-CD設計.md: Helm の paths フィルタに infra/helm/** が含まれる。"""
        assert "infra/helm/**" in self.raw_content


class TestCILintJobConditions:
    """CI-CD設計.md: lint ジョブの条件付き実行検証。"""

    def setup_method(self) -> None:
        with open(WORKFLOWS / "ci.yaml", encoding="utf-8") as f:
            self.config = yaml.safe_load(f)

    @pytest.mark.parametrize(
        "job,lang",
        [
            ("lint-go", "go"),
            ("lint-rust", "rust"),
            ("lint-ts", "ts"),
            ("lint-dart", "dart"),
            ("lint-python", "python"),
        ],
    )
    def test_lint_job_needs_detect_changes(self, job: str, lang: str) -> None:
        """CI-CD設計.md: lint ジョブが detect-changes に依存。"""
        assert self.config["jobs"][job]["needs"] == "detect-changes"

    @pytest.mark.parametrize(
        "job,lang",
        [
            ("lint-go", "go"),
            ("lint-rust", "rust"),
            ("lint-ts", "ts"),
            ("lint-dart", "dart"),
            ("lint-python", "python"),
        ],
    )
    def test_lint_job_conditional_execution(self, job: str, lang: str) -> None:
        """CI-CD設計.md: lint ジョブが detect-changes の出力で条件実行。"""
        condition = self.config["jobs"][job]["if"]
        assert lang in condition
        assert "detect-changes" in condition


class TestCITestJobNeeds:
    """CI-CD設計.md: test ジョブの needs 依存関係検証。"""

    def setup_method(self) -> None:
        with open(WORKFLOWS / "ci.yaml", encoding="utf-8") as f:
            self.config = yaml.safe_load(f)

    @pytest.mark.parametrize(
        "test_job,lint_job",
        [
            ("test-go", "lint-go"),
            ("test-rust", "lint-rust"),
            ("test-ts", "lint-ts"),
            ("test-dart", "lint-dart"),
            ("test-python", "lint-python"),
        ],
    )
    def test_test_needs_lint(self, test_job: str, lint_job: str) -> None:
        """CI-CD設計.md: test ジョブが対応する lint ジョブに依存。"""
        needs = self.config["jobs"][test_job]["needs"]
        if isinstance(needs, list):
            assert lint_job in needs
        else:
            assert needs == lint_job


class TestCIBuildJobDetails:
    """CI-CD設計.md: build ジョブの needs と always() 条件検証。"""

    def setup_method(self) -> None:
        with open(WORKFLOWS / "ci.yaml", encoding="utf-8") as f:
            self.config = yaml.safe_load(f)

    def test_build_needs_test_jobs(self) -> None:
        """CI-CD設計.md: build ジョブが test-go, test-rust, test-ts, test-dart に依存。"""
        needs = self.config["jobs"]["build"]["needs"]
        for dep in ["test-go", "test-rust", "test-ts", "test-dart"]:
            assert dep in needs, f"build ジョブが {dep} に依存していません"

    def test_build_always_condition(self) -> None:
        """CI-CD設計.md: build ジョブに always() 条件が設定されている。"""
        condition = self.config["jobs"]["build"]["if"]
        assert "always()" in condition
        assert "failure" in condition


class TestCILintStepDetails:
    """CI-CD設計.md: 各言語の lint ステップ詳細検証。"""

    def setup_method(self) -> None:
        self.raw_content = (WORKFLOWS / "ci.yaml").read_text(encoding="utf-8")
        with open(WORKFLOWS / "ci.yaml", encoding="utf-8") as f:
            self.config = yaml.safe_load(f)

    def test_ts_lint_eslint_prettier(self) -> None:
        """CI-CD設計.md: TypeScript lint で eslint + prettier を実行。"""
        steps = self.config["jobs"]["lint-ts"]["steps"]
        run_cmds = [s.get("run", "") for s in steps]
        assert any("eslint" in cmd for cmd in run_cmds), "eslint が lint-ts に含まれていません"
        assert any("prettier" in cmd for cmd in run_cmds), "prettier が lint-ts に含まれていません"

    def test_dart_lint_analyze_format(self) -> None:
        """CI-CD設計.md: Dart lint で dart analyze + dart format を実行。"""
        steps = self.config["jobs"]["lint-dart"]["steps"]
        run_cmds = [s.get("run", "") for s in steps]
        assert any("dart analyze" in cmd for cmd in run_cmds)
        assert any("dart format" in cmd for cmd in run_cmds)

    def test_python_lint_ruff_mypy(self) -> None:
        """CI-CD設計.md: Python lint で ruff + mypy を実行。"""
        steps = self.config["jobs"]["lint-python"]["steps"]
        run_cmds = [s.get("run", "") for s in steps]
        assert any("ruff" in cmd for cmd in run_cmds)
        assert any("mypy" in cmd for cmd in run_cmds)


class TestCILanguageVersions:
    """CI-CD設計.md: 各言語のバージョン検証。"""

    def setup_method(self) -> None:
        with open(WORKFLOWS / "ci.yaml", encoding="utf-8") as f:
            self.config = yaml.safe_load(f)

    def test_node_version_22(self) -> None:
        """CI-CD設計.md: Node.js バージョン 22。"""
        steps = self.config["jobs"]["lint-ts"]["steps"]
        for step in steps:
            if step.get("uses", "").startswith("actions/setup-node"):
                assert step["with"]["node-version"] == "22"

    def test_flutter_version_3_24_0(self) -> None:
        """CI-CD設計.md: Flutter バージョン 3.24.0。"""
        steps = self.config["jobs"]["lint-dart"]["steps"]
        for step in steps:
            if step.get("uses", "").startswith("subosito/flutter-action"):
                assert step["with"]["flutter-version"] == "3.24.0"

    def test_python_version_3_12(self) -> None:
        """CI-CD設計.md: Python バージョン 3.12。"""
        steps = self.config["jobs"]["lint-python"]["steps"]
        for step in steps:
            if step.get("uses", "").startswith("actions/setup-python"):
                assert step["with"]["python-version"] == "3.12"


class TestCIGoCoverageOutput:
    """CI-CD設計.md: Go テストのカバレッジ出力検証。"""

    def setup_method(self) -> None:
        with open(WORKFLOWS / "ci.yaml", encoding="utf-8") as f:
            self.config = yaml.safe_load(f)

    def test_go_coverage_output(self) -> None:
        """CI-CD設計.md: Go テストで coverprofile を出力。"""
        steps = self.config["jobs"]["test-go"]["steps"]
        run_cmds = [s.get("run", "") for s in steps]
        assert any("coverprofile" in cmd for cmd in run_cmds)

    def test_go_coverage_artifact_upload(self) -> None:
        """CI-CD設計.md: Go カバレッジを artifact としてアップロード。"""
        steps = self.config["jobs"]["test-go"]["steps"]
        upload_steps = [s for s in steps if "upload-artifact" in s.get("uses", "")]
        assert len(upload_steps) >= 1


class TestDeployJobDetails:
    """CI-CD設計.md: Deploy ジョブの詳細検証。"""

    def setup_method(self) -> None:
        with open(WORKFLOWS / "deploy.yaml", encoding="utf-8") as f:
            self.config = yaml.safe_load(f)
        self.raw_content = (WORKFLOWS / "deploy.yaml").read_text(encoding="utf-8")

    def test_deploy_self_hosted_runner(self) -> None:
        """CI-CD設計.md: Deploy ジョブが self-hosted ランナーを使用。"""
        for env_name in ["deploy-dev", "deploy-staging", "deploy-prod"]:
            runs_on = self.config["jobs"][env_name]["runs-on"]
            assert "self-hosted" in runs_on, f"{env_name} が self-hosted ランナーを使用していません"

    def test_deploy_cosign_verify(self) -> None:
        """CI-CD設計.md: Deploy ジョブで cosign verify を実行。"""
        for env_name in ["deploy-dev", "deploy-staging", "deploy-prod"]:
            steps = self.config["jobs"][env_name]["steps"]
            verify_steps = [s for s in steps if "cosign verify" in str(s.get("run", ""))]
            assert len(verify_steps) >= 1, f"{env_name} に cosign verify がありません"

    def test_deploy_helm_version_3_16(self) -> None:
        """CI-CD設計.md: Deploy ジョブで Helm バージョン 3.16 を使用。"""
        for env_name in ["deploy-dev", "deploy-staging", "deploy-prod"]:
            steps = self.config["jobs"][env_name]["steps"]
            helm_steps = [s for s in steps if "azure/setup-helm" in s.get("uses", "")]
            assert len(helm_steps) >= 1
            assert helm_steps[0]["with"]["version"] == "3.16"

    def test_deploy_helm_upgrade_install(self) -> None:
        """CI-CD設計.md: Deploy ジョブで helm upgrade --install を実行。"""
        for env_name in ["deploy-dev", "deploy-staging", "deploy-prod"]:
            steps = self.config["jobs"][env_name]["steps"]
            run_cmds = [s.get("run", "") for s in steps]
            assert any("helm upgrade --install" in cmd for cmd in run_cmds), (
                f"{env_name} に helm upgrade --install がありません"
            )

    def test_deploy_values_env_yaml_reference(self) -> None:
        """CI-CD設計.md: Deploy ジョブで values-{env}.yaml を参照。"""
        assert "values-dev.yaml" in self.raw_content
        assert "values-staging.yaml" in self.raw_content
        assert "values-prod.yaml" in self.raw_content

    def test_deploy_image_tag_format(self) -> None:
        """CI-CD設計.md: Deploy ジョブで image.tag を設定。"""
        for env_name in ["deploy-dev", "deploy-staging", "deploy-prod"]:
            steps = self.config["jobs"][env_name]["steps"]
            run_cmds = [s.get("run", "") for s in steps]
            assert any("image.tag" in cmd for cmd in run_cmds), (
                f"{env_name} に image.tag の設定がありません"
            )

    def test_deploy_prod_manual_approval(self) -> None:
        """CI-CD設計.md: prod デプロイに手動承認ゲート（environment.name: prod）。"""
        env = self.config["jobs"]["deploy-prod"]["environment"]
        if isinstance(env, dict):
            assert env["name"] == "prod"
        else:
            assert env == "prod"


class TestSecurityWorkflowDetails:
    """CI-CD設計.md: Security ワークフロー詳細検証。"""

    def setup_method(self) -> None:
        with open(WORKFLOWS / "security.yaml", encoding="utf-8") as f:
            self.config = yaml.safe_load(f)

    def test_dependency_check_job(self) -> None:
        """CI-CD設計.md: dependency-check ジョブが存在。"""
        assert "dependency-check" in self.config["jobs"]

    def test_image_scan_job(self) -> None:
        """CI-CD設計.md: image-scan ジョブが存在。"""
        assert "image-scan" in self.config["jobs"]

    def test_schedule_cron(self) -> None:
        """CI-CD設計.md: 日次スケジュール（cron）が設定。"""
        triggers = self.config[True]
        schedules = triggers["schedule"]
        assert len(schedules) >= 1
        assert "cron" in schedules[0]


class TestKongSyncWorkflowDetails:
    """CI-CD設計.md: Kong Sync ワークフローのトリガー条件検証。"""

    def setup_method(self) -> None:
        with open(WORKFLOWS / "kong-sync.yaml", encoding="utf-8") as f:
            self.config = yaml.safe_load(f)

    def test_kong_sync_trigger_push_main(self) -> None:
        """CI-CD設計.md: Kong Sync は main ブランチへの push でトリガー。"""
        triggers = self.config[True]
        assert "push" in triggers
        assert triggers["push"]["branches"] == ["main"]

    def test_kong_sync_trigger_paths(self) -> None:
        """CI-CD設計.md: Kong Sync は infra/kong/** の変更でトリガー。"""
        raw = (WORKFLOWS / "kong-sync.yaml").read_text(encoding="utf-8")
        assert "infra/kong/**" in raw


class TestCICacheStrategy:
    """CI-CD設計.md: キャッシュ戦略の検証。"""

    def setup_method(self) -> None:
        self.doc = (ROOT / "docs" / "CI-CD設計.md").read_text(encoding="utf-8")

    @pytest.mark.parametrize(
        "lang,cache_target",
        [
            ("Go", "~/go/pkg/mod"),
            ("Rust", "~/.cargo"),
            ("Node", "node_modules/"),
            ("Dart", "~/.pub-cache"),
            ("Python", "~/.cache/pip"),
        ],
    )
    def test_cache_strategy_documented(self, lang: str, cache_target: str) -> None:
        """CI-CD設計.md: 各言語のキャッシュ戦略が文書化されている。"""
        assert cache_target in self.doc, f"{lang} のキャッシュ対象 '{cache_target}' が文書に記載されていません"


class TestDockerLayerCache:
    """CI-CD設計.md: Docker layer cache の検証。"""

    def setup_method(self) -> None:
        with open(WORKFLOWS / "deploy.yaml", encoding="utf-8") as f:
            self.config = yaml.safe_load(f)

    def test_docker_cache_from_gha(self) -> None:
        """CI-CD設計.md: Docker ビルドで cache-from: type=gha を使用。"""
        steps = self.config["jobs"]["build-and-push"]["steps"]
        build_steps = [s for s in steps if s.get("uses", "").startswith("docker/build-push-action")]
        assert len(build_steps) >= 1
        assert "type=gha" in build_steps[0]["with"]["cache-from"]

    def test_docker_cache_to_gha(self) -> None:
        """CI-CD設計.md: Docker ビルドで cache-to: type=gha を使用。"""
        steps = self.config["jobs"]["build-and-push"]["steps"]
        build_steps = [s for s in steps if s.get("uses", "").startswith("docker/build-push-action")]
        assert "type=gha" in build_steps[0]["with"]["cache-to"]


class TestHelmLintLoop:
    """CI-CD設計.md: Helm lint ループ処理の検証。"""

    def setup_method(self) -> None:
        self.raw_content = (WORKFLOWS / "ci.yaml").read_text(encoding="utf-8")

    def test_helm_lint_for_loop(self) -> None:
        """CI-CD設計.md: Helm lint で for ループを使用して全チャートを検証。"""
        assert "for chart in" in self.raw_content
        assert "helm lint" in self.raw_content
