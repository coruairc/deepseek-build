/*
 * egress_guard.c — LD_PRELOAD connect() guard for scripts/sandbox-run.sh.
 *
 * Opposite of scripts/egress_log.c: this shim refuses. Every connect() is
 * allowed only to
 *   - AF_UNIX sockets,
 *   - IPv4 loopback (127.0.0.0/8) and 0.0.0.0/8,
 *   - IPv6 loopback (::1),
 *   - the resolved addresses of $EGRESS_ALLOW_HOST (default
 *     api.deepseek.com),
 * and everything else fails with ECONNREFUSED after one line on stderr:
 *
 *   sandbox-run: BLOCKED connect to <address>
 *
 * The destination is formatted in its own family's notation (inet_ntop over
 * AF_INET or AF_INET6), so an IPv6 refusal names the IPv6 address it refused.
 * "non-ip" is reserved for an address family that is neither.
 *
 * The allow host is resolved with getaddrinfo(AF_UNSPEC), i.e. over A and AAAA.
 * A resolution failure leaves an empty allowlist (fail closed), never an
 * "allow everything" state.
 *
 * The allowlist stores raw address bytes tagged by family, never formatted
 * strings. A v4-mapped IPv6 address (::ffff:a.b.c.d) is normalised to its
 * 4-byte IPv4 form on both sides, so ::ffff:1.2.3.4 and 1.2.3.4 are one
 * allowlist entry rather than two spellings of one destination.
 *
 * EGRESS_ALLOW_DEBUG=1 prints one line per resolved allowlist entry on stderr,
 * using the field vocabulary egress-check.sh already parses:
 *
 *   sandbox-run: allow family=inet addr=<ip> port=<n>
 *   sandbox-run: allow family=inet6 addr=<ip> port=<n>
 *
 * Build (scripts/sandbox-run.sh does this):
 *   cc -shared -fPIC -O2 -o egress_guard.so egress_guard.c -ldl
 */
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

#define MAX_ALLOW 16

struct allow_entry {
  int family;
  unsigned char bytes[16];
};

static int (*real_connect)(int, const struct sockaddr *, socklen_t) = 0;
static struct allow_entry allowed[MAX_ALLOW];
static int allowed_n = 0;

static const unsigned char v4mapped_prefix[12] = {0, 0, 0, 0, 0, 0,
                                                  0, 0, 0, 0, 0xff, 0xff};

/* Store one resolved address. A v4-mapped IPv6 answer is stored as its 4-byte
 * IPv4 form so a mapped spelling cannot split one destination in two. */
static void store(int family, const void *src, size_t n) {
  if (allowed_n >= MAX_ALLOW || n > sizeof allowed[0].bytes) return;
  if (family == AF_INET6 && n == 16 &&
      memcmp(src, v4mapped_prefix, sizeof v4mapped_prefix) == 0) {
    const unsigned char *b = (const unsigned char *)src;
    family = AF_INET;
    src = b + sizeof v4mapped_prefix;
    n = 4;
  }
  allowed[allowed_n].family = family;
  memset(allowed[allowed_n].bytes, 0, sizeof allowed[0].bytes);
  memcpy(allowed[allowed_n].bytes, src, n);
  allowed_n++;
}

/* EGRESS_ALLOW_DEBUG=1: one line per resolved entry, in the same shape
 * egress_log.c logs, so the runtime audit can show what was allowlisted.
 * Only the literal "1" enables it; "0" stays quiet as documented. */
static void debug_entry(const struct addrinfo *ai) {
  const char *env = getenv("EGRESS_ALLOW_DEBUG");
  if (!env || strcmp(env, "1") != 0) return;
  if (ai->ai_family == AF_INET) {
    const struct sockaddr_in *s = (const struct sockaddr_in *)ai->ai_addr;
    char ip[INET_ADDRSTRLEN] = "?";
    inet_ntop(AF_INET, &s->sin_addr, ip, sizeof ip);
    dprintf(2, "sandbox-run: allow family=inet addr=%s port=%u\n", ip,
            (unsigned)ntohs(s->sin_port));
  } else if (ai->ai_family == AF_INET6) {
    const struct sockaddr_in6 *s = (const struct sockaddr_in6 *)ai->ai_addr;
    char ip[INET6_ADDRSTRLEN] = "?";
    inet_ntop(AF_INET6, &s->sin6_addr, ip, sizeof ip);
    dprintf(2, "sandbox-run: allow family=inet6 addr=%s port=%u\n", ip,
            (unsigned)ntohs(s->sin6_port));
  }
}

__attribute__((constructor)) static void guard_init(void) {
  real_connect = dlsym(RTLD_NEXT, "connect");
  const char *host = getenv("EGRESS_ALLOW_HOST");
  if (!host || !*host) host = "api.deepseek.com";
  struct addrinfo hints, *res = 0;
  memset(&hints, 0, sizeof hints);
  /* AF_UNSPEC resolves A and AAAA. Do not add AI_ADDRCONFIG (it hides AAAA on
   * a host the resolver considers v4-only) or AI_NUMERICHOST (hostnames must
   * still resolve). */
  hints.ai_family = AF_UNSPEC;
  hints.ai_socktype = SOCK_STREAM;
  if (getaddrinfo(host, "443", &hints, &res) != 0) return;
  for (struct addrinfo *ai = res; ai && allowed_n < MAX_ALLOW; ai = ai->ai_next) {
    if (ai->ai_family == AF_INET) {
      store(AF_INET, &((struct sockaddr_in *)ai->ai_addr)->sin_addr, 4);
    } else if (ai->ai_family == AF_INET6) {
      store(AF_INET6, &((struct sockaddr_in6 *)ai->ai_addr)->sin6_addr, 16);
    } else {
      continue;
    }
    debug_entry(ai);
  }
  freeaddrinfo(res);
}

/* IPv4 policy: loopback, 0.0.0.0/8, then the v4 allowlist by raw bytes. */
static int v4_allowed(const unsigned char b[4]) {
  if (b[0] == 127 || b[0] == 0) return 1;
  for (int i = 0; i < allowed_n; i++) {
    if (allowed[i].family == AF_INET && memcmp(allowed[i].bytes, b, 4) == 0)
      return 1;
  }
  return 0;
}

/* IPv6 policy: ::1, then the v6 allowlist by raw bytes. v4-mapped addresses
 * are rewritten to the v4 path by the connect interposer below before they
 * reach this check. */
static int v6_allowed(const unsigned char b[16]) {
  static const unsigned char loopback[16] = {0, 0, 0, 0, 0, 0, 0, 0,
                                             0, 0, 0, 0, 0, 0, 0, 1};
  if (memcmp(b, loopback, sizeof loopback) == 0) return 1;
  for (int i = 0; i < allowed_n; i++) {
    if (allowed[i].family == AF_INET6 && memcmp(allowed[i].bytes, b, 16) == 0)
      return 1;
  }
  return 0;
}

static void format_dest(const struct sockaddr *addr, char *out, size_t n) {
  if (addr->sa_family == AF_INET) {
    inet_ntop(AF_INET, &((const struct sockaddr_in *)addr)->sin_addr, out,
              (socklen_t)n);
  } else if (addr->sa_family == AF_INET6) {
    inet_ntop(AF_INET6, &((const struct sockaddr_in6 *)addr)->sin6_addr, out,
              (socklen_t)n);
  } else {
    snprintf(out, n, "non-ip");
  }
}

/* Pass through to libc's connect, or refuse when the symbol cannot be resolved.
 * Resolving on every call keeps a late-loaded libc working; a NULL resolution is
 * a fail-closed refusal, never a call through a NULL pointer. */
static int call_real(int fd, const struct sockaddr *addr, socklen_t len) {
  if (!real_connect) real_connect = dlsym(RTLD_NEXT, "connect");
  if (!real_connect) {
    dprintf(2, "sandbox-run: BLOCKED connect: dlsym(connect) unavailable\n");
    errno = ECONNREFUSED;
    return -1;
  }
  return real_connect(fd, addr, len);
}

int connect(int fd, const struct sockaddr *addr, socklen_t len) {
  if (!addr) return call_real(fd, addr, len);
  if (addr->sa_family == AF_UNIX) return call_real(fd, addr, len);
  if (addr->sa_family == AF_INET) {
    const struct sockaddr_in *s = (const struct sockaddr_in *)addr;
    unsigned char b[4];
    memcpy(b, &s->sin_addr, sizeof b);
    if (v4_allowed(b)) return call_real(fd, addr, len);
  } else if (addr->sa_family == AF_INET6) {
    const struct sockaddr_in6 *s = (const struct sockaddr_in6 *)addr;
    if (IN6_IS_ADDR_V4MAPPED(&s->sin6_addr)) {
      unsigned char b[4];
      memcpy(b, s->sin6_addr.s6_addr + sizeof v4mapped_prefix, sizeof b);
      if (v4_allowed(b)) return call_real(fd, addr, len);
    } else {
      unsigned char b[16];
      memcpy(b, &s->sin6_addr, sizeof b);
      if (v6_allowed(b)) return call_real(fd, addr, len);
    }
  }
  /* Blocked: report the destination on stderr, then refuse. */
  char what[INET6_ADDRSTRLEN] = "non-ip";
  format_dest(addr, what, sizeof what);
  dprintf(2, "sandbox-run: BLOCKED connect to %s\n", what);
  errno = ECONNREFUSED;
  return -1;
}
