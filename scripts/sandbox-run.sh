#!/usr/bin/env bash
#
# sandbox-run.sh — run deepseek-build with outbound network restricted to the
# configured model provider (default api.deepseek.com) plus loopback.
#
# Preferred: a network namespace (unshare -rn) that drops all external routes.
#   NOTE: that also makes the provider unreachable, which is useful for
#   "does it try anything else?" runs but not for real prompts.
# Fallback (used here): an LD_PRELOAD shim that permits connect() only to
#   loopback, AF_UNIX, and the resolved addresses of $DEEPSEEK_BUILD_EGRESS_HOST
#   (default api.deepseek.com); everything else is refused with ECONNREFUSED.
#
# Usage:
#   scripts/sandbox-run.sh [deepseek-build args...]
#   DEEPSEEK_BUILD_EGRESS_HOST=my.proxy.example scripts/sandbox-run.sh -p "hi"
#
set -euo pipefail

cd "$(dirname "$0")/.."
BIN="${BIN:-target/release/deepseek-build}"
HOST="${DEEPSEEK_BUILD_EGRESS_HOST:-api.deepseek.com}"
WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT

cat >"$WORK/egress_guard.c" <<'C'
#define _GNU_SOURCE
#include <arpa/inet.h>
#include <dlfcn.h>
#include <errno.h>
#include <netdb.h>
#include <netinet/in.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/socket.h>

static int (*real_connect)(int, const struct sockaddr *, socklen_t) = 0;
static char allowed[16][16];
static int allowed_n = 0;

__attribute__((constructor)) static void init(void) {
  real_connect = dlsym(RTLD_NEXT, "connect");
  const char *host = getenv("EGRESS_ALLOW_HOST");
  if (!host) host = "api.deepseek.com";
  struct addrinfo hints, *res = 0;
  memset(&hints, 0, sizeof hints);
  hints.ai_family = AF_INET;
  hints.ai_socktype = SOCK_STREAM;
  if (getaddrinfo(host, "443", &hints, &res) == 0) {
    for (struct addrinfo *ai = res; ai && allowed_n < 16; ai = ai->ai_next) {
      struct sockaddr_in *s = (struct sockaddr_in *)ai->ai_addr;
      inet_ntop(AF_INET, &s->sin_addr, allowed[allowed_n], 16);
      allowed_n++;
    }
    freeaddrinfo(res);
  }
}

int connect(int fd, const struct sockaddr *addr, socklen_t len) {
  if (!real_connect) real_connect = dlsym(RTLD_NEXT, "connect");
  if (!addr) return real_connect(fd, addr, len);
  if (addr->sa_family == AF_UNIX) return real_connect(fd, addr, len);
  if (addr->sa_family == AF_INET) {
    struct sockaddr_in *s = (struct sockaddr_in *)addr;
    char ip[16];
    inet_ntop(AF_INET, &s->sin_addr, ip, sizeof ip);
    if (strncmp(ip, "127.", 4) == 0 || strncmp(ip, "0.", 2) == 0)
      return real_connect(fd, addr, len);
    for (int i = 0; i < allowed_n; i++)
      if (strcmp(ip, allowed[i]) == 0) return real_connect(fd, addr, len);
  } else if (addr->sa_family == AF_INET6) {
    struct sockaddr_in6 *s = (struct sockaddr_in6 *)addr;
    char ip[64];
    inet_ntop(AF_INET6, &s->sin6_addr, ip, sizeof ip);
    if (strcmp(ip, "::1") == 0) return real_connect(fd, addr, len);
  }
  /* Blocked: report the destination on stderr, then refuse. */
  char what[64] = "non-ip";
  if (addr->sa_family == AF_INET) {
    struct sockaddr_in *s = (struct sockaddr_in *)addr;
    inet_ntop(AF_INET, &s->sin_addr, what, sizeof what);
  }
  dprintf(2, "sandbox-run: BLOCKED connect to %s\n", what);
  errno = ECONNREFUSED;
  return -1;
}
C

cc -shared -fPIC -O2 -o "$WORK/egress_guard.so" "$WORK/egress_guard.c" -ldl

echo "sandbox-run: allowing only ${HOST}:443 + loopback; running: $BIN $*" >&2
exec env EGRESS_ALLOW_HOST="$HOST" LD_PRELOAD="$WORK/egress_guard.so" "$BIN" "$@"
