FROM jaegertracing/all-in-one:1.54.0

COPY config/ /etc/jaeger/config/

ENTRYPOINT ["/go/bin/all-in-one"]
CMD ["--query.ui-config=/etc/jaeger/config/default.yaml"]
