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
