FROM ghcr.io/actions/actions-runner:latest

# Match the build tools supplied by GitHub's ubuntu-latest VM. In particular,
# setup-mold downloads its release with wget, absent from the base runner image.
USER root
RUN apt-get update \
    && apt-get install -y --no-install-recommends build-essential clang pkg-config wget unzip ca-certificates \
    && rm -rf /var/lib/apt/lists/*
USER runner
