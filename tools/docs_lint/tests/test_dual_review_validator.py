"""tests/test_dual_review_validator.py

dual_review_validator.py の unit tests。
5 ケース: placeholder 検出 / 実署名通過 / 未定義 key 拒否 / oneOf 分岐 / V0 scope 除外
"""
from __future__ import annotations

import sys
import pathlib
import yaml
import pytest

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent.parent.parent.parent))

from tools.docs_lint.dual_review_validator import (
    load_schema,
    validate_file,
    is_v0_scope_out,
)
from jsonschema import Draft202012Validator

REPO_ROOT = pathlib.Path(__file__).resolve().parent.parent.parent.parent


def _validator() -> Draft202012Validator:
    return Draft202012Validator(load_schema())


def _write(tmp_path: pathlib.Path, name: str, content: dict) -> pathlib.Path:
    p = tmp_path / name
    p.write_text(yaml.safe_dump(content), encoding="utf-8")
    return p


def test_placeholder_rejected(tmp_path: pathlib.Path) -> None:
    """zero-hash subject_digest + sigstore://placeholder cosign URI が schema 違反になる。"""
    f = _write(tmp_path, "test_obligation_001.dual_review.lock.yaml", {
        "_AUTO_GENERATED": "DO NOT EDIT.",
        "subject_digest": "sha256:" + "0" * 64,
        "obligation_id": "test_obligation_001",
        "reviewers": [
            {
                "signer_id": "RyuheiKiso",
                "reviewer_kind": "human",
                "signed_at": "2026-05-23T00:00:00Z",
                "cosign_signature_uri": "sigstore://placeholder/test_human",
            },
            {
                "signer_id": "claude-opus-4-7",
                "reviewer_kind": "ai_static_analysis",
                "signed_at": "2026-05-23T00:00:00Z",
                "cosign_signature_uri": "sigstore://placeholder/test_ai",
                "ai_evidence_uri": "urn:k1s0:ai_evidence:test:2026-05-23",
            },
        ],
        "cooldown_check": {
            "last_sign_by_same_author": None,
            "ai_evidence_diff_sha256": "sha256:" + "0" * 64,
        },
    })
    errs = validate_file(f, _validator())
    assert errs, "zero-hash + placeholder URI should cause violations"


def test_real_standard_form_passes(tmp_path: pathlib.Path) -> None:
    """本物 sha256 + sigstore://rekor.sigstore.dev URI を持つ標準形が schema 適合する。"""
    real_digest = "sha256:" + "a" * 64
    f = _write(tmp_path, "test_real_001.dual_review.lock.yaml", {
        "_AUTO_GENERATED": "DO NOT EDIT.",
        "subject_digest": real_digest,
        "obligation_id": "test_real_001",
        "reviewers": [
            {
                "signer_id": "RyuheiKiso",
                "reviewer_kind": "human",
                "signed_at": "2026-05-23T00:00:00Z",
                "cosign_signature_uri": "sigstore://rekor.sigstore.dev/api/v1/log/entries/abc",
            },
            {
                "signer_id": "claude-opus-4-7",
                "reviewer_kind": "ai_static_analysis",
                "signed_at": "2026-05-23T00:00:00Z",
                "cosign_signature_uri": "sigstore://rekor.sigstore.dev/api/v1/log/entries/def",
                "ai_evidence_uri": "urn:k1s0:ai_evidence:test_real_001:2026-05-23",
                "ai_generator_id": "claude-opus-4-7",
                "model_version": "claude-opus-4-7",
                "prompt_hash": "sha256:p11_test_hash_value",
            },
        ],
        "cooldown_check": {
            "last_sign_by_same_author": None,
            "ai_evidence_diff_sha256": real_digest,
        },
    })
    errs = validate_file(f, _validator())
    assert not errs, f"real standard form should pass, but got: {errs}"


def test_extra_field_rejected(tmp_path: pathlib.Path) -> None:
    """schema に無い top-level key が additionalProperties: false で reject される。"""
    real_digest = "sha256:" + "b" * 64
    f = _write(tmp_path, "test_extra_001.dual_review.lock.yaml", {
        "_AUTO_GENERATED": "DO NOT EDIT.",
        "subject_digest": real_digest,
        "obligation_id": "test_extra_001",
        "unknown_field": "this should fail",
        "reviewers": [
            {
                "signer_id": "RyuheiKiso",
                "reviewer_kind": "human",
                "signed_at": "2026-05-23T00:00:00Z",
                "cosign_signature_uri": "sigstore://rekor.sigstore.dev/abc",
            },
            {
                "signer_id": "ai",
                "reviewer_kind": "ai_static_analysis",
                "signed_at": "2026-05-23T00:00:00Z",
                "cosign_signature_uri": "sigstore://rekor.sigstore.dev/def",
            },
        ],
        "cooldown_check": {
            "ai_evidence_diff_sha256": real_digest,
        },
    })
    errs = validate_file(f, _validator())
    assert errs, "extra field should be rejected by additionalProperties: false"


def test_implementation_review_form_branch_requires_subject(tmp_path: pathlib.Path) -> None:
    """obligation_id=v1_implementation_review の分岐で subject 欠落が violation になる。"""
    f = _write(tmp_path, "v1_implementation_review.dual_review.lock.yaml", {
        "_AUTO_GENERATED": "DO NOT EDIT.",
        "obligation_id": "v1_implementation_review",
        # subject / subject_digest / review_scope / ai_analysis_findings / v1_0_0_readiness を欠く
        "reviewers": [
            {
                "signer_id": "RyuheiKiso",
                "reviewer_kind": "human",
                "signed_at": "PENDING",
                "cosign_signature_uri": "sigstore://rekor.sigstore.dev/abc",
            },
        ],
        "cooldown_check": {
            "ai_evidence_diff_sha256": "sha256:" + "a" * 64,
        },
    })
    errs = validate_file(f, _validator())
    assert errs, "missing required fields in implementation_review_form should cause violations"


def test_v0_legacy_excluded_from_scope() -> None:
    """V0 旧形 5 件のファイル名が scope 外として検出され、TSP 形は scope 内になる。"""
    v0_names = [
        "tier1_auth.dual_review.lock.yaml",
        "tier1_key_mgmt.dual_review.lock.yaml",
        "tier1_slo.dual_review.lock.yaml",
        "tier1_oss_lifecycle.dual_review.lock.yaml",
        "tier1_tenant_capacity.dual_review.lock.yaml",
    ]
    in_scope_names = [
        "tier1_auth_tsp.dual_review.lock.yaml",
        "tier1_correctness_001.dual_review.lock.yaml",
        "formal_correctness_045.dual_review.lock.yaml",
        "v1_implementation_review.dual_review.lock.yaml",
    ]
    for name in v0_names:
        assert is_v0_scope_out(pathlib.Path(name)), f"{name} should be V0 scope-out"
    for name in in_scope_names:
        assert not is_v0_scope_out(pathlib.Path(name)), f"{name} should be in scope"


def test_real_repo_files_currently_violate() -> None:
    """実リポジトリの 82 in-scope file が現状全件 schema 違反であることを確認。
    実 cosign 署名完了後に違反数が 0 になることが期待値。"""
    validator = _validator()
    files = sorted(REPO_ROOT.glob("src/formal/dual_review/*.dual_review.lock.yaml"))
    in_scope = [f for f in files if not is_v0_scope_out(f)]
    assert len(in_scope) == 82, f"expected 82 in-scope files, got {len(in_scope)}"
    violators = sum(1 for f in in_scope if validate_file(f, validator))
    assert violators == 82, (
        f"expected all 82 files to violate (placeholder cosign not yet signed), "
        f"but only {violators} violated"
    )
