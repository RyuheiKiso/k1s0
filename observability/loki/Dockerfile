FROM grafana/loki:2.9.4

COPY config/loki.yaml /etc/loki/local-config.yaml

ENTRYPOINT ["/usr/bin/loki"]
CMD ["-config.file=/etc/loki/local-config.yaml"]
