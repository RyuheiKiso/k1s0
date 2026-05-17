# TLA+ / Apalache モデル検査ツールチェイン
FROM ubuntu:22.04

ENV APALACHE_VERSION=0.45.4
ENV DEBIAN_FRONTEND=noninteractive

# JDK 21 + wget インストール
RUN apt-get update && apt-get install -y --no-install-recommends \
    openjdk-21-jdk \
    wget \
    unzip \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# TLA+ Toolbox（tla2tools）を取得
RUN wget -q \
    "https://github.com/tlaplus/tlaplus/releases/download/v1.8.0/TLAToolbox-1.8.0-linux.gtk.x86_64.zip" \
    -O /tmp/tla-toolbox.zip && \
    unzip /tmp/tla-toolbox.zip -d /opt/tla && \
    rm /tmp/tla-toolbox.zip

# Apalache モデル検査器を取得・展開
RUN wget -q \
    "https://github.com/apalache-mc/apalache/releases/download/v${APALACHE_VERSION}/apalache.zip" \
    -O /tmp/apalache.zip && \
    unzip /tmp/apalache.zip -d /opt/ && \
    mv /opt/apalache /opt/apalache-bin && \
    ln -s /opt/apalache-bin /opt/apalache && \
    rm /tmp/apalache.zip

ENV PATH="/opt/apalache/bin:${PATH}"

WORKDIR /workspace

CMD ["apalache-mc", "version"]
