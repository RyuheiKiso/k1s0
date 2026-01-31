FROM prom/prometheus:v2.50.1

COPY config/prometheus.yml /etc/prometheus/prometheus.yml
COPY config/rules/ /etc/prometheus/rules/

ENTRYPOINT ["/bin/prometheus"]
CMD ["--config.file=/etc/prometheus/prometheus.yml", "--storage.tsdb.path=/prometheus", "--storage.tsdb.retention.time=15d", "--storage.tsdb.wal-compression", "--web.enable-remote-write-receiver"]
