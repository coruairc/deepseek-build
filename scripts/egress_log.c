/*
 * egress_log.c — LD_PRELOAD connect()/getaddrinfo() logger for
 * scripts/egress-check.sh.
 *
 * Unlike scripts/sandbox-run.sh this shim never blocks: it ALLOWS every
 * connect()/sendto()/sendmsg()/sendmmsg() and appends one line per call to the
 * file named by $EGRESS_LOG:
 *
 *   CONNECT family=<inet|inet6|unix|other> addr=<ip-or-path> port=<n>
 *   SENDTO  family=... addr=... port=...      (also SENDMSG / SENDMMSG)
 *   GETADDR name=<host> addr=<ip>
 *
 * connect() plus the unconnected-socket send paths are the egress sources of
 * truth (the allow/deny decision is made by egress-check.sh from the log).
 * getaddrinfo() is logged too so an observed address can be attributed to the
 * hostname the process resolved.
 *
 * Build (the script does this): cc -shared -fPIC -O2 -o egress_log.so egress_log.c -ldl
 * The environment and the preload are inherited by child processes, so the
 * binary and every dynamic child it forks log into the same file.
 */
#define _GNU_SOURCE
#include <arpa/inet.h>
#include <dlfcn.h>
#include <errno.h>
#include <fcntl.h>
#include <netdb.h>
#include <netinet/in.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/socket.h>
#include <sys/types.h>
#include <sys/un.h>
#include <unistd.h>

static int (*real_connect)(int, const struct sockaddr *, socklen_t) = 0;
static ssize_t (*real_sendto)(int, const void *, size_t, int,
                              const struct sockaddr *, socklen_t) = 0;
static ssize_t (*real_sendmsg)(int, const struct msghdr *, int) = 0;
static int (*real_sendmmsg)(int, struct mmsghdr *, unsigned int, int) = 0;
static int (*real_getaddrinfo)(const char *, const char *,
                               const struct addrinfo *, struct addrinfo **) = 0;
static int log_fd = -1;

static void emit(const char *line, size_t n);

static void open_log(void) {
  if (log_fd >= 0) return;
  const char *path = getenv("EGRESS_LOG");
  if (path && *path) {
    log_fd = open(path, O_WRONLY | O_CREAT | O_APPEND | O_CLOEXEC, 0644);
  }
}

__attribute__((constructor)) static void egress_init(void) {
  real_connect = dlsym(RTLD_NEXT, "connect");
  real_sendto = dlsym(RTLD_NEXT, "sendto");
  real_sendmsg = dlsym(RTLD_NEXT, "sendmsg");
  real_sendmmsg = dlsym(RTLD_NEXT, "sendmmsg");
  real_getaddrinfo = dlsym(RTLD_NEXT, "getaddrinfo");
  open_log();
}

/* Log a destination address as a SEND event (unconnected UDP/raw sockets do not
 * call connect(); sendto/sendmsg are their egress path). */
static void log_sockaddr(const char *kind, const struct sockaddr *addr) {
  if (!addr || log_fd < 0) return;
  char buf[512];
  int n = 0;
  if (addr->sa_family == AF_INET) {
    struct sockaddr_in *s = (struct sockaddr_in *)addr;
    char ip[INET_ADDRSTRLEN] = "?";
    inet_ntop(AF_INET, &s->sin_addr, ip, sizeof ip);
    n = snprintf(buf, sizeof buf, "%s family=inet addr=%s port=%u\n", kind, ip,
                 (unsigned)ntohs(s->sin_port));
  } else if (addr->sa_family == AF_INET6) {
    struct sockaddr_in6 *s = (struct sockaddr_in6 *)addr;
    char ip[INET6_ADDRSTRLEN] = "?";
    inet_ntop(AF_INET6, &s->sin6_addr, ip, sizeof ip);
    n = snprintf(buf, sizeof buf, "%s family=inet6 addr=%s port=%u\n", kind, ip,
                 (unsigned)ntohs(s->sin6_port));
  } else if (addr->sa_family == AF_UNIX) {
    struct sockaddr_un *s = (struct sockaddr_un *)addr;
    n = snprintf(buf, sizeof buf, "%s family=unix addr=%s port=0\n", kind, s->sun_path);
  } else {
    n = snprintf(buf, sizeof buf, "%s family=other addr=family-%d port=0\n", kind,
                 addr->sa_family);
  }
  if (n > 0) emit(buf, (size_t)n);
}

ssize_t sendto(int fd, const void *buf, size_t len, int flags,
               const struct sockaddr *dest, socklen_t alen) {
  if (!real_sendto) real_sendto = dlsym(RTLD_NEXT, "sendto");
  open_log();
  log_sockaddr("SENDTO", dest);
  return real_sendto(fd, buf, len, flags, dest, alen);
}

ssize_t sendmsg(int fd, const struct msghdr *msg, int flags) {
  if (!real_sendmsg) real_sendmsg = dlsym(RTLD_NEXT, "sendmsg");
  open_log();
  if (msg) log_sockaddr("SENDMSG", (const struct sockaddr *)msg->msg_name);
  return real_sendmsg(fd, msg, flags);
}

int sendmmsg(int fd, struct mmsghdr *vmessages, unsigned int vlen, int flags) {
  if (!real_sendmmsg) real_sendmmsg = dlsym(RTLD_NEXT, "sendmmsg");
  open_log();
  if (vmessages) {
    for (unsigned int i = 0; i < vlen; i++)
      log_sockaddr("SENDMMSG", (const struct sockaddr *)vmessages[i].msg_hdr.msg_name);
  }
  return real_sendmmsg(fd, vmessages, vlen, flags);
}

static void emit(const char *line, size_t n) {
  if (log_fd < 0) return;
  ssize_t r = write(log_fd, line, n);
  (void)r;
}

int connect(int fd, const struct sockaddr *addr, socklen_t len) {
  if (!real_connect) real_connect = dlsym(RTLD_NEXT, "connect");
  open_log();
  if (addr && log_fd >= 0) {
    char buf[512];
    int n = 0;
    if (addr->sa_family == AF_INET) {
      struct sockaddr_in *s = (struct sockaddr_in *)addr;
      char ip[INET_ADDRSTRLEN] = "?";
      inet_ntop(AF_INET, &s->sin_addr, ip, sizeof ip);
      n = snprintf(buf, sizeof buf, "CONNECT family=inet addr=%s port=%u\n", ip,
                   (unsigned)ntohs(s->sin_port));
    } else if (addr->sa_family == AF_INET6) {
      struct sockaddr_in6 *s = (struct sockaddr_in6 *)addr;
      char ip[INET6_ADDRSTRLEN] = "?";
      inet_ntop(AF_INET6, &s->sin6_addr, ip, sizeof ip);
      n = snprintf(buf, sizeof buf, "CONNECT family=inet6 addr=%s port=%u\n", ip,
                   (unsigned)ntohs(s->sin6_port));
    } else if (addr->sa_family == AF_UNIX) {
      struct sockaddr_un *s = (struct sockaddr_un *)addr;
      n = snprintf(buf, sizeof buf, "CONNECT family=unix addr=%s port=0\n", s->sun_path);
    } else {
      n = snprintf(buf, sizeof buf, "CONNECT family=other addr=family-%d port=0\n",
                   addr->sa_family);
    }
    if (n > 0) emit(buf, (size_t)n);
  }
  return real_connect(fd, addr, len);
}

int getaddrinfo(const char *node, const char *service,
                const struct addrinfo *hints, struct addrinfo **res) {
  if (!real_getaddrinfo) real_getaddrinfo = dlsym(RTLD_NEXT, "getaddrinfo");
  int rc = real_getaddrinfo(node, service, hints, res);
  open_log();
  if (rc == 0 && res && *res && log_fd >= 0) {
    for (struct addrinfo *ai = *res; ai; ai = ai->ai_next) {
      char ip[INET6_ADDRSTRLEN] = "?";
      if (ai->ai_family == AF_INET) {
        inet_ntop(AF_INET, &((struct sockaddr_in *)ai->ai_addr)->sin_addr, ip, sizeof ip);
      } else if (ai->ai_family == AF_INET6) {
        inet_ntop(AF_INET6, &((struct sockaddr_in6 *)ai->ai_addr)->sin6_addr, ip, sizeof ip);
      } else {
        continue;
      }
      char buf[512];
      int n = snprintf(buf, sizeof buf, "GETADDR name=%s addr=%s\n",
                       node ? node : "(null)", ip);
      if (n > 0) emit(buf, (size_t)n);
    }
  }
  return rc;
}
