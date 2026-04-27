// 本ファイルは k1s0-sdk の Telemetry 動詞統一 facade。
use crate::client::Client;
use crate::proto::k1s0::tier1::telemetry::v1::{
    EmitMetricRequest, EmitSpanRequest, Metric, Span,
    telemetry_service_client::TelemetryServiceClient,
};
use tonic::{Status, transport::Channel};

/// TelemetryFacade は TelemetryService の動詞統一 facade。
pub struct TelemetryFacade {
    client: Client,
    raw: TelemetryServiceClient<Channel>,
}

impl TelemetryFacade {
    pub(crate) fn new(client: Client) -> Self {
        let raw = TelemetryServiceClient::new(client.channel());
        Self { client, raw }
    }

    /// emit_metric はメトリクス送信。
    pub async fn emit_metric(&mut self, metrics: Vec<Metric>) -> Result<(), Status> {
        self.raw
            .emit_metric(EmitMetricRequest {
                metrics,
                context: Some(self.client.tenant_context()),
            })
            .await?;
        Ok(())
    }

    /// emit_span は Span 送信。
    pub async fn emit_span(&mut self, spans: Vec<Span>) -> Result<(), Status> {
        self.raw
            .emit_span(EmitSpanRequest {
                spans,
                context: Some(self.client.tenant_context()),
            })
            .await?;
        Ok(())
    }
}
