# =============================================================================
# deepseek-build production Docker image
# =============================================================================
# Independent project, not affiliated with DeepSeek.
#
# Base: debian:bookworm-slim (glibc ~2.36).
# Scope: official releases only. The image is assembled in CI from the
#        linux-x86_64 / linux-aarch64 release tarballs produced by release.yml;
#        the tarball contents (binary + license files) are the build context.
#        For local testing use docker/Dockerfile.localtest.
# =============================================================================

FROM debian:bookworm-slim

# OCI labels (the CI metadata-action adds version/revision/source on top).
LABEL org.opencontainers.image.title="deepseek-build" \
      org.opencontainers.image.description="DeepSeek-first terminal coding agent. Independent project, not affiliated with DeepSeek." \
      org.opencontainers.image.licenses="Apache-2.0" \
      org.opencontainers.image.vendor="Independent project, not affiliated with DeepSeek"

ENV DEBIAN_FRONTEND=noninteractive

# Runtime tools a coding agent needs inside the container: TLS roots, git,
# ripgrep, curl, ssh, bash, procps. No build toolchain.
# hadolint ignore=DL3008
RUN apt-get update && apt-get install -y --no-install-recommends \
        ca-certificates \
        git \
        ripgrep \
        curl \
        openssh-client \
        bash \
        procps \
        libgcc-s1 \
    && rm -rf /var/lib/apt/lists/*

# Non-root user, uid 1000 (override at run time with --user "$(id -u):$(id -g)").
RUN useradd --create-home --uid 1000 --shell /bin/bash app

# License/attribution files live in the conventional docs directory.
COPY LICENSE NOTICE THIRD-PARTY-NOTICES /usr/share/doc/deepseek-build/
COPY README.txt /usr/share/doc/deepseek-build/README.txt

# The binary is installed on PATH. The build context stages one binary per
# target arch as deepseek-build-amd64 / deepseek-build-arm64; buildx sets
# TARGETARCH for each platform in a multi-platform build.
ARG TARGETARCH
COPY deepseek-build-${TARGETARCH} /usr/local/bin/deepseek-build
RUN chmod 755 /usr/local/bin/deepseek-build

WORKDIR /work
USER 1000:1000

ENTRYPOINT ["deepseek-build"]
