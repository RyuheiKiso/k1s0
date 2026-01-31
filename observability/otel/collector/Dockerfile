FROM otel/opentelemetry-collector-contrib:0.96.0

COPY config/ /etc/otelcol-contrib/config/

ENTRYPOINT ["/otelcol-contrib"]
CMD ["--config=/etc/otelcol-contrib/config/default.yaml"]
