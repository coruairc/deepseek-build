# =============================================================================
# deepseek-build production Docker image
# =============================================================================
# Base: debian:bookworm-slim (glibc ~2.36)
# Scope: Official releases only. Multi-arch manifest assembled in CI from the
#        linux-x86_64 and linux-aarch64 tarballs produced by release.yml.
#        NOT validated locally — see docker/Dockerfile.localtest for local use.
# =============================================================================

FROM debian:bookworm-slim

# Prevent interactive prompts during apt install.
ENV DEBIAN_FRONTEND=noninteractive

# Install the runtime requirements. We include:
#   - ca-certificates: TLS root CAs (api.deepseek.com)
#   - libgcc-s1 (or higher): C++ runtime needed by jemalloc (our allocator)
#   - tini: minimal init (PID 1 reaping); the container runs one invocation.
# We deliberately do NOT include:
#   - protobuf-compiler or cmake: build-time only
#   - git: the binary is self-contained; no git needed at runtime
#   - shell or coreutils: the binary is statically linked for its target
# hadolint ignore=DL3008
RUN apt-get update && apt-get install -y --no-install-recommends \
        ca-certificates \
        libgcc-s1 \
        tini \
    && rm -rf /var/lib/apt/lists/*

# Create a non-root user matching the host UID (default 1000; can be overridden
# at docker run time with --user). The binary needs no special privileges.
RUN useradd --create-home --uid 1000 app

WORKDIR /app

# Copy the release tarball contents laid out by release.yml:
#   deepseek-build  LICENSE  NOTICE  THIRD-PARTY-NOTICES  README.txt
COPY deepseek-build deepseek-build
COPY LICENSE NOTICE THIRD-PARTY-NOTICES README.txt ./

# Binary is +x from the tarball; ensure it for the image.
RUN chmod 755 deepseek-build

# tini runs as PID 1 so the binary gets proper signal handling (SIGTERM →
# SIGINT → SIGKILL if it doesn't exit cleanly).
ENTRYPOINT ["/usr/bin/tini", "--"]
CMD ["./deepseek-build"]
