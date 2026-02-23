"""サービスメッシュ設計.md の仕様準拠テスト。

Istio CRD（Gateway, PeerAuthentication, AuthorizationPolicy,
DestinationRule, VirtualService）、Flagger Canary、
フォールトインジェクションの設定が設計ドキュメントと一致するかを検証する。
"""

from pathlib import Path

import pytest
import yaml  # type: ignore[import-untyped]

ROOT = Path(__file__).resolve().parents[3]
ISTIO = ROOT / "infra" / "istio"


class TestIstioGateway:
    """サービスメッシュ設計.md: Istio Gateway の検証。"""

    def setup_method(self) -> None:
        path = ISTIO / "gateway.yaml"
        assert path.exists()
        self.content = path.read_text(encoding="utf-8")
        self.gateway = yaml.safe_load(self.content)

    def test_gateway_file_exists(self) -> None:
        assert (ISTIO / "gateway.yaml").exists()

    def test_gateway_kind(self) -> None:
        assert self.gateway["kind"] == "Gateway"

    def test_gateway_name(self) -> None:
        assert self.gateway["metadata"]["name"] == "k1s0-mesh-gateway"

    def test_gateway_namespace(self) -> None:
        assert self.gateway["metadata"]["namespace"] == "service-mesh"

    def test_gateway_tls_mode(self) -> None:
        """サービスメッシュ設計.md: mTLS (ISTIO_MUTUAL) を使用。"""
        servers = self.gateway["spec"]["servers"]
        assert servers[0]["tls"]["mode"] == "ISTIO_MUTUAL"

    def test_gateway_hosts(self) -> None:
        """サービスメッシュ設計.md: k1s0 の 3 Namespace をカバー。"""
        hosts = self.gateway["spec"]["servers"][0]["hosts"]
        assert "*.k1s0-system.svc.cluster.local" in hosts
        assert "*.k1s0-business.svc.cluster.local" in hosts
        assert "*.k1s0-service.svc.cluster.local" in hosts


class TestPeerAuthentication:
    """サービスメッシュ設計.md: PeerAuthentication の検証。"""

    def setup_method(self) -> None:
        path = ISTIO / "peerauthentication.yaml"
        assert path.exists()
        self.content = path.read_text(encoding="utf-8")
        self.docs = [d for d in yaml.safe_load_all(self.content) if d]

    def test_peerauthentication_file_exists(self) -> None:
        assert (ISTIO / "peerauthentication.yaml").exists()

    def test_mesh_wide_strict_mtls(self) -> None:
        """サービスメッシュ設計.md: メッシュ全体で STRICT mTLS。"""
        mesh_wide = [
            d
            for d in self.docs
            if d["metadata"]["namespace"] == "service-mesh" and d["metadata"]["name"] == "default"
        ]
        assert len(mesh_wide) == 1
        assert mesh_wide[0]["spec"]["mtls"]["mode"] == "STRICT"

    def test_observability_permissive(self) -> None:
        """サービスメッシュ設計.md: observability は PERMISSIVE。"""
        obs = [d for d in self.docs if d["metadata"]["namespace"] == "observability"]
        assert len(obs) == 1
        assert obs[0]["spec"]["mtls"]["mode"] == "PERMISSIVE"

    @pytest.mark.parametrize(
        "namespace",
        ["k1s0-system", "k1s0-business", "k1s0-service"],
    )
    def test_namespace_strict_mtls(self, namespace: str) -> None:
        """サービスメッシュ設計.md: 各 Tier Namespace で STRICT mTLS。"""
        ns_docs = [d for d in self.docs if d["metadata"]["namespace"] == namespace]
        assert len(ns_docs) >= 1
        assert ns_docs[0]["spec"]["mtls"]["mode"] == "STRICT"


class TestAuthorizationPolicy:
    """サービスメッシュ設計.md: AuthorizationPolicy の検証。"""

    def setup_method(self) -> None:
        path = ISTIO / "authorizationpolicy.yaml"
        assert path.exists()
        self.content = path.read_text(encoding="utf-8")
        self.docs = [d for d in yaml.safe_load_all(self.content) if d]

    def test_authorizationpolicy_file_exists(self) -> None:
        assert (ISTIO / "authorizationpolicy.yaml").exists()

    def test_system_tier_allow_policy(self) -> None:
        """サービスメッシュ設計.md: system は business, service からアクセス可。"""
        system_policies = [
            d
            for d in self.docs
            if d["metadata"]["namespace"] == "k1s0-system" and d["spec"]["action"] == "ALLOW"
        ]
        assert len(system_policies) >= 1

    def test_business_tier_allow_policy(self) -> None:
        """サービスメッシュ設計.md: business は service からアクセス可。"""
        biz_policies = [
            d
            for d in self.docs
            if d["metadata"]["namespace"] == "k1s0-business" and d["spec"]["action"] == "ALLOW"
        ]
        assert len(biz_policies) >= 1

    def test_service_tier_allow_policy(self) -> None:
        """サービスメッシュ設計.md: service は ingress からアクセス可。"""
        svc_policies = [
            d
            for d in self.docs
            if d["metadata"]["namespace"] == "k1s0-service" and d["spec"]["action"] == "ALLOW"
        ]
        assert len(svc_policies) >= 1

    def test_deny_bff_to_bff(self) -> None:
        """サービスメッシュ設計.md: BFF 間通信の禁止。"""
        deny_policies = [d for d in self.docs if d["spec"]["action"] == "DENY"]
        assert len(deny_policies) >= 1
        deny = deny_policies[0]
        assert deny["metadata"]["name"] == "deny-bff-to-bff"


class TestDestinationRules:
    """サービスメッシュ設計.md: DestinationRule の検証。"""

    def setup_method(self) -> None:
        path = ISTIO / "destinationrules" / "default.yaml"
        assert path.exists()
        self.content = path.read_text(encoding="utf-8")
        self.docs = [d for d in yaml.safe_load_all(self.content) if d]

    def test_destinationrules_file_exists(self) -> None:
        assert (ISTIO / "destinationrules" / "default.yaml").exists()

    def test_order_server_rule(self) -> None:
        names = [d["metadata"]["name"] for d in self.docs]
        assert "order-server" in names

    def test_auth_server_rule(self) -> None:
        names = [d["metadata"]["name"] for d in self.docs]
        assert "auth-server" in names

    def test_order_server_subsets(self) -> None:
        """サービスメッシュ設計.md: stable/canary サブセット。"""
        order = [d for d in self.docs if d["metadata"]["name"] == "order-server"][0]
        subset_names = [s["name"] for s in order["spec"]["subsets"]]
        assert "stable" in subset_names
        assert "canary" in subset_names

    def test_circuit_breaker_config(self) -> None:
        """サービスメッシュ設計.md: outlierDetection (Circuit Breaker)。"""
        order = [d for d in self.docs if d["metadata"]["name"] == "order-server"][0]
        od = order["spec"]["trafficPolicy"]["outlierDetection"]
        assert od["consecutive5xxErrors"] == 5
        assert od["baseEjectionTime"] == "60s"

    def test_load_balancer(self) -> None:
        """サービスメッシュ設計.md: LEAST_REQUEST ロードバランシング。"""
        order = [d for d in self.docs if d["metadata"]["name"] == "order-server"][0]
        lb = order["spec"]["trafficPolicy"]["loadBalancer"]
        assert lb["simple"] == "LEAST_REQUEST"

    def test_mtls_mode(self) -> None:
        """サービスメッシュ設計.md: ISTIO_MUTUAL TLS。"""
        for doc in self.docs:
            tls = doc["spec"]["trafficPolicy"]["tls"]
            assert tls["mode"] == "ISTIO_MUTUAL"


class TestVirtualServices:
    """サービスメッシュ設計.md: VirtualService の検証。"""

    def test_default_virtualservice_exists(self) -> None:
        assert (ISTIO / "virtualservices" / "default.yaml").exists()

    def test_canary_virtualservice_exists(self) -> None:
        assert (ISTIO / "virtualservices" / "canary.yaml").exists()

    def test_mirror_virtualservice_exists(self) -> None:
        assert (ISTIO / "virtualservices" / "mirror.yaml").exists()

    def test_virtual_service_integrated_exists(self) -> None:
        assert (ISTIO / "virtual-service.yaml").exists()

    def test_system_tier_timeout(self) -> None:
        """サービスメッシュ設計.md: system Tier timeout=5s, retry=3。"""
        path = ISTIO / "virtual-service.yaml"
        content = path.read_text(encoding="utf-8")
        docs = [d for d in yaml.safe_load_all(content) if d]
        system = [d for d in docs if d["metadata"].get("namespace") == "k1s0-system"]
        assert len(system) >= 1
        http = system[0]["spec"]["http"][0]
        assert http["timeout"] == "5s"
        assert http["retries"]["attempts"] == 3

    def test_business_tier_timeout(self) -> None:
        """サービスメッシュ設計.md: business Tier timeout=10s, retry=3。"""
        path = ISTIO / "virtual-service.yaml"
        content = path.read_text(encoding="utf-8")
        docs = [d for d in yaml.safe_load_all(content) if d]
        biz = [d for d in docs if d["metadata"].get("namespace") == "k1s0-business"]
        assert len(biz) >= 1
        http = biz[0]["spec"]["http"][0]
        assert http["timeout"] == "10s"
        assert http["retries"]["attempts"] == 3

    def test_service_tier_timeout(self) -> None:
        """サービスメッシュ設計.md: service Tier timeout=15s, retry=2。"""
        path = ISTIO / "virtual-service.yaml"
        content = path.read_text(encoding="utf-8")
        docs = [d for d in yaml.safe_load_all(content) if d]
        svc = [d for d in docs if d["metadata"].get("namespace") == "k1s0-service"]
        assert len(svc) >= 1
        http = svc[0]["spec"]["http"][0]
        assert http["timeout"] == "15s"
        assert http["retries"]["attempts"] == 2

    def test_canary_weight_routing(self) -> None:
        """サービスメッシュ設計.md: カナリアのウェイトベースルーティング。"""
        path = ISTIO / "virtualservices" / "canary.yaml"
        content = path.read_text(encoding="utf-8")
        assert "stable" in content
        assert "canary" in content
        assert "weight" in content

    def test_header_based_routing(self) -> None:
        """サービスメッシュ設計.md: ヘッダーベースルーティング。"""
        path = ISTIO / "virtualservices" / "canary.yaml"
        content = path.read_text(encoding="utf-8")
        assert "x-canary" in content

    def test_traffic_mirroring(self) -> None:
        """サービスメッシュ設計.md: トラフィックミラーリング。"""
        path = ISTIO / "virtualservices" / "mirror.yaml"
        content = path.read_text(encoding="utf-8")
        assert "mirror" in content
        assert "mirrorPercentage" in content


class TestFlaggerCanary:
    """サービスメッシュ設計.md: Flagger Canary リソースの検証。"""

    def setup_method(self) -> None:
        path = ISTIO / "flagger" / "canary.yaml"
        assert path.exists()
        self.content = path.read_text(encoding="utf-8")
        self.canary = yaml.safe_load(self.content)

    def test_canary_file_exists(self) -> None:
        assert (ISTIO / "flagger" / "canary.yaml").exists()

    def test_canary_kind(self) -> None:
        assert self.canary["kind"] == "Canary"

    def test_canary_api_version(self) -> None:
        assert self.canary["apiVersion"] == "flagger.app/v1beta1"

    def test_canary_target(self) -> None:
        assert self.canary["spec"]["targetRef"]["name"] == "order-server"

    def test_canary_analysis_interval(self) -> None:
        """サービスメッシュ設計.md: 5分間隔の分析。"""
        assert self.canary["spec"]["analysis"]["interval"] == "5m"

    def test_canary_max_weight(self) -> None:
        assert self.canary["spec"]["analysis"]["maxWeight"] == 80

    def test_canary_step_weight(self) -> None:
        assert self.canary["spec"]["analysis"]["stepWeight"] == 20

    def test_canary_metrics(self) -> None:
        """サービスメッシュ設計.md: success-rate と duration メトリクス。"""
        metrics = self.canary["spec"]["analysis"]["metrics"]
        metric_names = [m["name"] for m in metrics]
        assert "request-success-rate" in metric_names
        assert "request-duration" in metric_names

    def test_canary_rollback_webhook(self) -> None:
        webhooks = self.canary["spec"]["analysis"]["webhooks"]
        assert any(w["type"] == "rollback" for w in webhooks)


class TestFaultInjection:
    """サービスメッシュ設計.md: フォールトインジェクションの検証。"""

    def test_delay_file_exists(self) -> None:
        assert (ISTIO / "fault-injection" / "delay.yaml").exists()

    def test_abort_file_exists(self) -> None:
        assert (ISTIO / "fault-injection" / "abort.yaml").exists()

    def test_cronjob_file_exists(self) -> None:
        assert (ISTIO / "fault-injection" / "cronjob.yaml").exists()

    def test_delay_config(self) -> None:
        """サービスメッシュ設計.md: 500ms 遅延を 10% に注入。"""
        path = ISTIO / "fault-injection" / "delay.yaml"
        content = path.read_text(encoding="utf-8")
        assert "500ms" in content
        assert "10.0" in content

    def test_abort_config(self) -> None:
        """サービスメッシュ設計.md: 503 エラーを 5% に注入。"""
        path = ISTIO / "fault-injection" / "abort.yaml"
        content = path.read_text(encoding="utf-8")
        assert "503" in content
        assert "5.0" in content

    def test_cronjob_schedule(self) -> None:
        """サービスメッシュ設計.md: 毎週月曜日にスケジュール実行。"""
        path = ISTIO / "fault-injection" / "cronjob.yaml"
        doc = yaml.safe_load(path.read_text(encoding="utf-8"))
        assert doc["kind"] == "CronJob"
        assert "1" in doc["spec"]["schedule"]  # Monday


class TestVirtualServiceConcreteValues:
    """サービスメッシュ設計.md: VirtualService 具体値の検証。"""

    def setup_method(self) -> None:
        path = ISTIO / "virtual-service.yaml"
        content = path.read_text(encoding="utf-8")
        self.docs = [d for d in yaml.safe_load_all(content) if d]

    def test_business_tier_timeout_10s(self) -> None:
        """サービスメッシュ設計.md: business Tier timeout=10s。"""
        biz = [d for d in self.docs if d["metadata"].get("namespace") == "k1s0-business"]
        assert len(biz) >= 1
        assert biz[0]["spec"]["http"][0]["timeout"] == "10s"

    def test_business_tier_retries_3(self) -> None:
        """サービスメッシュ設計.md: business Tier retries.attempts=3。"""
        biz = [d for d in self.docs if d["metadata"].get("namespace") == "k1s0-business"]
        assert biz[0]["spec"]["http"][0]["retries"]["attempts"] == 3

    def test_service_tier_retry_on_includes_retriable_4xx(self) -> None:
        """サービスメッシュ設計.md: service Tier retryOn に retriable-4xx を含む。"""
        svc = [d for d in self.docs if d["metadata"].get("namespace") == "k1s0-service"]
        assert len(svc) >= 1
        retry_on = svc[0]["spec"]["http"][0]["retries"]["retryOn"]
        assert "retriable-4xx" in retry_on

    def test_system_tier_retry_on(self) -> None:
        """サービスメッシュ設計.md: system Tier retryOn = 5xx,reset,connect-failure。"""
        sys = [d for d in self.docs if d["metadata"].get("namespace") == "k1s0-system"]
        assert len(sys) >= 1
        retry_on = sys[0]["spec"]["http"][0]["retries"]["retryOn"]
        assert "5xx" in retry_on
        assert "reset" in retry_on
        assert "connect-failure" in retry_on


class TestCircuitBreakerTierDefaults:
    """サービスメッシュ設計.md: Circuit Breaker Tier 別デフォルト値の検証。"""

    def setup_method(self) -> None:
        path = ISTIO / "destinationrules" / "default.yaml"
        content = path.read_text(encoding="utf-8")
        self.docs = [d for d in yaml.safe_load_all(content) if d]

    def test_system_tier_consecutive_5xx_errors(self) -> None:
        """サービスメッシュ設計.md: system Tier consecutive5xxErrors=3。"""
        auth = [d for d in self.docs if d["metadata"]["name"] == "auth-server"][0]
        od = auth["spec"]["trafficPolicy"]["outlierDetection"]
        assert od["consecutive5xxErrors"] == 3

    def test_system_tier_interval(self) -> None:
        """サービスメッシュ設計.md: system Tier interval=10s。"""
        auth = [d for d in self.docs if d["metadata"]["name"] == "auth-server"][0]
        od = auth["spec"]["trafficPolicy"]["outlierDetection"]
        assert od["interval"] == "10s"

    def test_system_tier_base_ejection_time(self) -> None:
        """サービスメッシュ設計.md: system Tier baseEjectionTime=30s。"""
        auth = [d for d in self.docs if d["metadata"]["name"] == "auth-server"][0]
        od = auth["spec"]["trafficPolicy"]["outlierDetection"]
        assert od["baseEjectionTime"] == "30s"

    def test_system_tier_max_ejection_percent(self) -> None:
        """サービスメッシュ設計.md: system Tier maxEjectionPercent=30。"""
        auth = [d for d in self.docs if d["metadata"]["name"] == "auth-server"][0]
        od = auth["spec"]["trafficPolicy"]["outlierDetection"]
        assert od["maxEjectionPercent"] == 30

    def test_service_tier_consecutive_5xx_errors(self) -> None:
        """サービスメッシュ設計.md: service Tier consecutive5xxErrors=5。"""
        order = [d for d in self.docs if d["metadata"]["name"] == "order-server"][0]
        od = order["spec"]["trafficPolicy"]["outlierDetection"]
        assert od["consecutive5xxErrors"] == 5

    def test_service_tier_base_ejection_time(self) -> None:
        """サービスメッシュ設計.md: service Tier baseEjectionTime=60s。"""
        order = [d for d in self.docs if d["metadata"]["name"] == "order-server"][0]
        od = order["spec"]["trafficPolicy"]["outlierDetection"]
        assert od["baseEjectionTime"] == "60s"

    def test_service_tier_max_ejection_percent(self) -> None:
        """サービスメッシュ設計.md: service Tier maxEjectionPercent=50。"""
        order = [d for d in self.docs if d["metadata"]["name"] == "order-server"][0]
        od = order["spec"]["trafficPolicy"]["outlierDetection"]
        assert od["maxEjectionPercent"] == 50


class TestCanaryRolloutStages:
    """サービスメッシュ設計.md: カナリアリリース段階的ロールアウト（5段階）の検証。"""

    def test_doc_defines_5_stages(self) -> None:
        """サービスメッシュ設計.md: 10%→30%→50%→80%→100% の 5 段階が定義されている。"""
        doc = (ROOT / "docs" / "サービスメッシュ設計.md").read_text(encoding="utf-8")
        assert "10%" in doc
        assert "30%" in doc
        assert "50%" in doc
        assert "80%" in doc
        assert "100%" in doc

    def test_flagger_step_weight_20(self) -> None:
        """サービスメッシュ設計.md: Flagger stepWeight=20 で段階的増加。"""
        path = ISTIO / "flagger" / "canary.yaml"
        canary = yaml.safe_load(path.read_text(encoding="utf-8"))
        assert canary["spec"]["analysis"]["stepWeight"] == 20

    def test_flagger_max_weight_80(self) -> None:
        """サービスメッシュ設計.md: Flagger maxWeight=80（最終段階は promotion で 100%）。"""
        path = ISTIO / "flagger" / "canary.yaml"
        canary = yaml.safe_load(path.read_text(encoding="utf-8"))
        assert canary["spec"]["analysis"]["maxWeight"] == 80

    def test_evaluation_interval_5m(self) -> None:
        """サービスメッシュ設計.md: 各段階の評価期間は 5 分。"""
        path = ISTIO / "flagger" / "canary.yaml"
        canary = yaml.safe_load(path.read_text(encoding="utf-8"))
        assert canary["spec"]["analysis"]["interval"] == "5m"


class TestRollbackConditions:
    """サービスメッシュ設計.md: ロールバック条件の検証。"""

    def test_doc_defines_error_rate_rollback(self) -> None:
        """サービスメッシュ設計.md: エラーレート > 5% でロールバック。"""
        doc = (ROOT / "docs" / "サービスメッシュ設計.md").read_text(encoding="utf-8")
        assert "5%" in doc
        assert "ロールバック" in doc

    def test_doc_defines_latency_rollback(self) -> None:
        """サービスメッシュ設計.md: P99 レイテンシ > 1000ms でロールバック。"""
        doc = (ROOT / "docs" / "サービスメッシュ設計.md").read_text(encoding="utf-8")
        assert "1000ms" in doc

    def test_flagger_success_rate_threshold(self) -> None:
        """サービスメッシュ設計.md: Flagger の request-success-rate min=99（エラーレート < 1%）。"""
        path = ISTIO / "flagger" / "canary.yaml"
        canary = yaml.safe_load(path.read_text(encoding="utf-8"))
        metrics = canary["spec"]["analysis"]["metrics"]
        success_rate = [m for m in metrics if m["name"] == "request-success-rate"][0]
        assert success_rate["thresholdRange"]["min"] == 99

    def test_flagger_duration_threshold(self) -> None:
        """サービスメッシュ設計.md: Flagger の request-duration max=500。"""
        path = ISTIO / "flagger" / "canary.yaml"
        canary = yaml.safe_load(path.read_text(encoding="utf-8"))
        metrics = canary["spec"]["analysis"]["metrics"]
        duration = [m for m in metrics if m["name"] == "request-duration"][0]
        assert duration["thresholdRange"]["max"] == 500


class TestBusinessTierCircuitBreaker:
    """サービスメッシュ設計.md: business Tier Circuit Breaker 設定の検証。"""

    def _get_cb_table_row(self, setting_name: str) -> str:
        """Tier 別デフォルト値テーブルから指定設定の行を取得する。"""
        doc = (ROOT / "docs" / "サービスメッシュ設計.md").read_text(encoding="utf-8")
        lines = doc.split("\n")
        for line in lines:
            if f"| {setting_name}" in line:
                return line
        return ""

    def test_doc_defines_business_consecutive_5xx(self) -> None:
        """サービスメッシュ設計.md: business Tier consecutive5xxErrors=5。"""
        row = self._get_cb_table_row("consecutive5xxErrors")
        assert row, "consecutive5xxErrors の行が見つかりません"
        # テーブル: | consecutive5xxErrors | system(3) | business(5) | service(5) |
        cells = [c.strip() for c in row.split("|") if c.strip()]
        assert len(cells) >= 3, "テーブル列が不足しています"
        # cells[0]=設定名, cells[1]=system, cells[2]=business
        assert cells[2] == "5", (
            f"business の consecutive5xxErrors は 5 であるべきですが {cells[2]} でした"
        )

    def test_doc_business_interval_30s(self) -> None:
        """サービスメッシュ設計.md: business Tier interval=30s。"""
        row = self._get_cb_table_row("interval")
        assert row, "interval の行が見つかりません"
        cells = [c.strip() for c in row.split("|") if c.strip()]
        assert len(cells) >= 3
        assert cells[2] == "30s", f"business の interval は 30s であるべきですが {cells[2]} でした"

    def test_doc_business_max_ejection_50(self) -> None:
        """サービスメッシュ設計.md: business Tier maxEjectionPercent=50。"""
        row = self._get_cb_table_row("maxEjectionPercent")
        assert row, "maxEjectionPercent の行が見つかりません"
        cells = [c.strip() for c in row.split("|") if c.strip()]
        assert len(cells) >= 3
        assert cells[2] == "50", (
            f"business の maxEjectionPercent は 50 であるべきですが {cells[2]} でした"
        )


class TestFaultInjectionGrafanaDashboard:
    """サービスメッシュ設計.md: フォールトインジェクション結果 Grafana ダッシュボードの検証。"""

    def test_doc_defines_fault_injection_dashboard_panels(self) -> None:
        """サービスメッシュ設計.md: フォールトインジェクション結果のダッシュボードパネルが定義されている。"""
        doc = (ROOT / "docs" / "サービスメッシュ設計.md").read_text(encoding="utf-8")
        assert "エラーレート推移" in doc
        assert "P99 レイテンシ推移" in doc
        assert "Circuit Breaker 発動状況" in doc
        assert "リトライ回数" in doc

    def test_doc_references_fault_injection_results_dashboard(self) -> None:
        """サービスメッシュ設計.md: Fault Injection Results ダッシュボード名が記載されている。"""
        doc = (ROOT / "docs" / "サービスメッシュ設計.md").read_text(encoding="utf-8")
        assert "Fault Injection Results" in doc


class TestConnectionPoolSettings:
    """サービスメッシュ設計.md: DestinationRule の connectionPool 設定の検証。"""

    def setup_method(self) -> None:
        path = ISTIO / "destinationrules" / "default.yaml"
        content = path.read_text(encoding="utf-8")
        self.docs = [d for d in yaml.safe_load_all(content) if d]

    def test_order_server_tcp_max_connections(self) -> None:
        """サービスメッシュ設計.md: order-server tcp.maxConnections=100。"""
        order = [d for d in self.docs if d["metadata"]["name"] == "order-server"][0]
        cp = order["spec"]["trafficPolicy"]["connectionPool"]
        assert cp["tcp"]["maxConnections"] == 100

    def test_order_server_http2_max_requests(self) -> None:
        """サービスメッシュ設計.md: order-server http.http2MaxRequests=1000。"""
        order = [d for d in self.docs if d["metadata"]["name"] == "order-server"][0]
        cp = order["spec"]["trafficPolicy"]["connectionPool"]
        assert cp["http"]["http2MaxRequests"] == 1000

    def test_order_server_max_requests_per_connection(self) -> None:
        """サービスメッシュ設計.md: order-server http.maxRequestsPerConnection=10。"""
        order = [d for d in self.docs if d["metadata"]["name"] == "order-server"][0]
        cp = order["spec"]["trafficPolicy"]["connectionPool"]
        assert cp["http"]["maxRequestsPerConnection"] == 10

    def test_auth_server_tcp_max_connections(self) -> None:
        """サービスメッシュ設計.md: auth-server tcp.maxConnections=200。"""
        auth = [d for d in self.docs if d["metadata"]["name"] == "auth-server"][0]
        cp = auth["spec"]["trafficPolicy"]["connectionPool"]
        assert cp["tcp"]["maxConnections"] == 200

    def test_auth_server_http2_max_requests(self) -> None:
        """サービスメッシュ設計.md: auth-server http.http2MaxRequests=2000。"""
        auth = [d for d in self.docs if d["metadata"]["name"] == "auth-server"][0]
        cp = auth["spec"]["trafficPolicy"]["connectionPool"]
        assert cp["http"]["http2MaxRequests"] == 2000
