FROM grafana/grafana:10.3.3

COPY config/grafana.ini /etc/grafana/grafana.ini
COPY provisioning/ /etc/grafana/provisioning/
COPY dashboards/ /var/lib/grafana/dashboards/

EXPOSE 3000
