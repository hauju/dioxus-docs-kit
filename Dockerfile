FROM debian:trixie-slim

RUN apt-get update && export DEBIAN_FRONTEND=noninteractive \
    && apt-get -y install --no-install-recommends \
    ca-certificates curl \
    && rm -rf /var/lib/apt/lists/*

ENV PORT=8080
ENV IP=0.0.0.0

EXPOSE 8080

COPY target/dx/dioxus-docs-kit-example/release/web /usr/local/app

WORKDIR /usr/local/app
ENTRYPOINT ["/usr/local/app/dioxus-docs-kit-example"]
