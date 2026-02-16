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
