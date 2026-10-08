#!/usr/bin/env bash
#
# egress-check.sh — repeatable, REAL-session egress audit for deepseek-build.
#
# It runs an actual model-driven headless session (not a mock), with an
# LD_PRELOAD shim that ALLOWS and LOGS every connect(), sendto(), sendmsg(),
# sendmmsg() and getaddrinfo() call, then fails if any observed outbound
# destination is outside this allowlist:
#
#   * the model provider (default api.deepseek.com) — as resolved at runtime
#   * the host fetched by the web_fetch tool (default example.com)
#   * loopback (127.0.0.0/8, ::1) and the unspecified address (0.0.0.0, ::)
#   * AF_UNIX sockets (DNS resolver, dbus, MCP stdio pipes, …)
#
# The session exercises: shell commands, file edits in a scratch git repo, a
# real web_fetch over HTTPS, a local stdio MCP tool, headless JSON output, a
# resume of the same session, and a forced auto-compaction. It then runs the
# same session (shell + web_fetch) through scripts/sandbox-run.sh and reports
# exactly which destinations that guard refuses.
#
# The API key is read from DEEPSEEK_API_KEY / DEEPSEEK_BUILD_API_KEY and is
# never printed, logged, or written to disk by this script.
#
# Coverage & limitations (LD_PRELOAD cannot intercept everything):
#   * Covered: libc connect()/sendto()/sendmsg()/sendmmsg() in the binary and
#     every dynamic child, TCP and UDP (unconnected UDP has no connect()).
#   * Public (non-loopback) inet/inet6 destinations must be on port 443.
#   * NOT covered: raw `syscall(SYS_connect|sendto|sendmsg)`, io_uring,
#     statically-linked or setuid children, and children that scrub LD_PRELOAD.
#   * Allowlisting is by resolved IP, so another hostname on the same shared
#     CDN edge IP would pass; use `strace -f -e trace=connect,sendto,sendmsg`
#     where available for kernel-level ground truth.
#
# Usage:
#   export DEEPSEEK_API_KEY=...
#   scripts/egress-check.sh
#
# Env:
#   BIN                     binary to run    (default target/release/deepseek-build)
#   API_HOST                provider host    (default api.deepseek.com)
#   WEB_FETCH_URL           web_fetch URL    (default https://example.com/)
#   EGRESS_WORKDIR          keep artifacts here (default: a fresh mktemp dir)
#   KEEP=1                  do not delete the work dir on exit
#
# This is a bash script (see the shebang); pipefail is intentional. The repo
# also lints scripts with `shellcheck --shell=sh` for consistency, so declare
# that here.
# shellcheck disable=SC3040
set -uo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
BIN="${BIN:-$REPO_ROOT/target/release/deepseek-build}"
API_HOST="${API_HOST:-api.deepseek.com}"
WEB_FETCH_URL="${WEB_FETCH_URL:-https://example.com/}"
WEB_HOST="$(python3 - "$WEB_FETCH_URL" <<'PY'
import sys, urllib.parse
u = urllib.parse.urlparse(sys.argv[1])
print(u.hostname or "")
PY
)"

KEY="${DEEPSEEK_API_KEY:-${DEEPSEEK_BUILD_API_KEY:-}}"
# Keep the key in the process environment only (never on a command line, so it
# cannot appear in `ps`/cmdline), and drop any ambient proxy configuration so
# the audit cannot be steered through a proxy.
export DEEPSEEK_API_KEY="$KEY"
unset http_proxy https_proxy HTTP_PROXY HTTPS_PROXY ALL_PROXY all_proxy \
      DEEPSEEK_BUILD_API_KEY GROK_API_KEY
SESSION_TIMEOUT="${SESSION_TIMEOUT:-360}"
RESUME_TIMEOUT="${RESUME_TIMEOUT:-240}"
SANDBOX_TIMEOUT="${SANDBOX_TIMEOUT:-180}"

FAILED=0
note()  { printf '%s\n' "$*"; }
hdr()   { printf '\n=== %s ===\n' "$*"; }
ok()    { printf 'PASS  %s\n' "$*"; }
bad()   { printf 'FAIL  %s\n' "$*"; FAILED=1; }

if [ -z "$KEY" ]; then
  printf 'FAIL  API key missing: set DEEPSEEK_API_KEY or DEEPSEEK_BUILD_API_KEY\n' >&2
  exit 2
fi
if [ ! -x "$BIN" ]; then
  printf 'FAIL  binary not found or not executable: %s\n' "$BIN" >&2
  printf '      build with: cargo build --release -p xai-grok-pager-bin\n' >&2
  exit 2
fi
if ! command -v cc >/dev/null 2>&1; then
  printf 'FAIL  C compiler (cc) not found; needed to build the LD_PRELOAD logger\n' >&2
  exit 2
fi
if ! command -v python3 >/dev/null 2>&1; then
  printf 'FAIL  python3 not found; needed for the MCP server and log parser\n' >&2
  exit 2
fi

WORK="${EGRESS_WORKDIR:-$(mktemp -d)}"
mkdir -p "$WORK"
# shellcheck disable=SC2317  # reached via trap
cleanup() { [ "${KEEP:-0}" = "1" ] || rm -rf "$WORK"; }
trap cleanup EXIT

HOME_DIR="$WORK/home"
DSH="$HOME_DIR/.deepseek-build"
SCRATCH="$WORK/scratch"
EGRESS_LOG="$WORK/egress.log"
MARKER="$WORK/mcp_called.txt"
mkdir -p "$HOME_DIR" "$DSH" "$SCRATCH"
: > "$EGRESS_LOG"

note "egress-check: bin=$BIN"
note "egress-check: work=$WORK (KEEP=1 to keep)"
note "egress-check: allowlist: $API_HOST, $WEB_HOST, loopback, AF_UNIX"

# ---------------------------------------------------------------------------
# 1. Build the allow-and-log LD_PRELOAD shim.
# ---------------------------------------------------------------------------
hdr "build connect logger"
cc -shared -fPIC -O2 -o "$WORK/egress_log.so" "$REPO_ROOT/scripts/egress_log.c" -ldl \
  || { bad "compile scripts/egress_log.c"; exit 2; }
ok "compiled $WORK/egress_log.so"

# ---------------------------------------------------------------------------
# 2. Isolated HOME + config: local stdio MCP server, web_fetch enabled.
# ---------------------------------------------------------------------------
hdr "isolated home + local MCP server"
cat >"$WORK/mcp_server.py" <<'PYEOF'
#!/usr/bin/env python3
"""Minimal stdio MCP server: advertises one tool and records calls."""
import json, os, sys

PROTOCOL_VERSION = "2025-06-18"
TOOL_NAME = "egress_ping"
MARKER = os.environ.get("EGRESS_MCP_MARKER", "")


def send(obj):
    sys.stdout.write(json.dumps(obj) + "\n")
    sys.stdout.flush()


def handle(msg):
    method = msg.get("method")
    mid = msg.get("id")
    if method == "initialize":
        proto = (msg.get("params") or {}).get("protocolVersion") or PROTOCOL_VERSION
        return {"jsonrpc": "2.0", "id": mid, "result": {
            "protocolVersion": proto,
            "capabilities": {"tools": {"listChanged": False}},
            "serverInfo": {"name": "egressmcp", "version": "1.0.0"}}}
    if method in ("notifications/initialized", "initialized"):
        return None
    if method == "ping":
        return {"jsonrpc": "2.0", "id": mid, "result": {}}
    if method == "tools/list":
        return {"jsonrpc": "2.0", "id": mid, "result": {"tools": [{
            "name": TOOL_NAME,
            "description": "Local egress-check tool. Call once when asked to verify the local MCP server.",
            "inputSchema": {"type": "object", "properties": {
                "note": {"type": "string", "description": "Optional note to echo back."}},
                "required": []}}]}}
    if method in ("resources/list", "prompts/list"):
        key = "resources" if method.startswith("resources") else "prompts"
        return {"jsonrpc": "2.0", "id": mid, "result": {key: []}}
    if method == "tools/call":
        params = msg.get("params") or {}
        name = params.get("name")
        if name != TOOL_NAME:
            return {"jsonrpc": "2.0", "id": mid, "result": {
                "content": [{"type": "text", "text": "unknown tool %s" % name}], "isError": True}}
        note = (params.get("arguments") or {}).get("note", "")
        if MARKER:
            try:
                with open(MARKER, "a") as fh:
                    fh.write("egress_ping called note=%s\n" % note)
            except OSError as exc:
                sys.stderr.write("marker write failed: %s\n" % exc)
        return {"jsonrpc": "2.0", "id": mid, "result": {
            "content": [{"type": "text", "text": "MCP_OK local server reached; note=%s" % note}],
            "isError": False}}
    if mid is not None:
        return {"jsonrpc": "2.0", "id": mid,
                "error": {"code": -32601, "message": "method not found: %s" % method}}
    return None


def main():
    for line in sys.stdin:
        line = line.strip()
        if not line:
            continue
        try:
            msg = json.loads(line)
        except ValueError:
            continue
        try:
            resp = handle(msg)
        except Exception as exc:  # noqa: BLE001
            resp = ({"jsonrpc": "2.0", "id": msg["id"],
                     "error": {"code": -32603, "message": str(exc)}}
                    if msg.get("id") is not None else None)
        if resp is not None:
            send(resp)


if __name__ == "__main__":
    main()
PYEOF

cat >"$DSH/config.toml" <<EOF
[features]
web_fetch = true

[models]
default = "deepseek-v4-pro"
default_reasoning_effort = "high"

[toolset.web_fetch]
allowed_domains = ["$WEB_HOST"]

[session]
auto_compact_threshold_percent = 1

[mcp_servers.egressmcp]
command = "python3"
args = ["$WORK/mcp_server.py"]
enabled = true

[mcp_servers.egressmcp.env]
EGRESS_MCP_MARKER = "$MARKER"
EOF
ok "config: web_fetch enabled, auto_compact_threshold_percent=1, MCP stdio server egressmcp"

# scratch git repo for the shell/edit work
(
  cd "$SCRATCH"
  git init -q . 2>/dev/null
  git config user.email egress-check@example.invalid
  git config user.name egress-check
  printf 'hello original\n' > edit_me.txt
  git add -A && git commit -qm init
) || { bad "initialize scratch git repo"; }
ok "scratch git repo ready at $SCRATCH"

# ---------------------------------------------------------------------------
# 3. Drive the real headless session.
# ---------------------------------------------------------------------------
run_dsb() {
  # All args are passed to the binary. Per-command VAR=value assignments land in
  # the child's environment (not its argv); DEEPSEEK_API_KEY is inherited from
  # the exported script environment. A clean-ish environment keeps the
  # observation deterministic; ambient proxies were unset above.
  HOME="$HOME_DIR" \
  DEEPSEEK_BUILD_HOME="$DSH" \
  GROK_AUTO_COMPACT_THRESHOLD_PERCENT=1 \
  GROK_WEB_FETCH=1 \
  EGRESS_LOG="$EGRESS_LOG" \
  LD_PRELOAD="$WORK/egress_log.so" \
  TERM=dumb \
  "$BIN" "$@"
}

PROMPT1="You are running an egress self-test. Perform ALL of these steps in order, exactly one tool call per assistant turn, waiting for each result before continuing:
1. Use the shell tool (run_terminal_cmd) to run: printf 'shell-egress\n' > shell_out.txt
2. Use the file editing tool (search_replace) to replace the word original with the word edited in edit_me.txt
3. Use web_fetch to fetch $WEB_FETCH_URL and report its HTML title.
4. Call the local MCP tool whose name ends with egress_ping (server egressmcp) with the argument note set to from-egress-check.
5. Reply with a short one-line summary listing which tools you used."

PROMPT2="Reply with the single word RESUMED and nothing else. Do not call any tools."

hdr "session 1 (headless json: shell + edit + web_fetch + MCP + compaction)"
run_dsb --cwd "$SCRATCH" --always-approve --no-plan -m deepseek-v4-pro --max-turns 24 \
  --debug-file "$WORK/s1.debug.log" --output-format json -p "$PROMPT1" \
  >"$WORK/s1.json" 2>"$WORK/s1.err"
RC1=$?
if [ "$RC1" -eq 0 ]; then
  ok "session 1 exited 0"
else
  bad "session 1 exit=$RC1 (see $WORK/s1.err)"
fi
[ -s "$WORK/s1.json" ] || { bad "session 1 produced no JSON"; note "stderr:"; sed -n '1,40p' "$WORK/s1.err"; exit 1; }

SID="$(python3 - "$WORK/s1.json" <<'PY'
import json, sys
try:
    print(json.load(open(sys.argv[1])).get("sessionId", ""))
except Exception:
    print("")
PY
)"
if [ -n "$SID" ]; then
  ok "session id: $SID"
else
  bad "could not read sessionId from session 1"
fi

hdr "session 2 (resume the same session)"
if [ -n "$SID" ]; then
  run_dsb --cwd "$SCRATCH" --always-approve --no-plan -m deepseek-v4-pro --max-turns 24 \
    --debug-file "$WORK/s2.debug.log" --output-format json -r "$SID" -p "$PROMPT2" \
    >"$WORK/s2.json" 2>"$WORK/s2.err"
  RC2=$?
  if [ "$RC2" -eq 0 ]; then
    ok "resume exited 0"
  else
    bad "resume exit=$RC2 (see $WORK/s2.err)"
  fi
  SID2="$(python3 - "$WORK/s2.json" <<'PY'
import json, sys
try:
    print(json.load(open(sys.argv[1])).get("sessionId", ""))
except Exception:
    print("")
PY
)"
  if [ "$SID2" = "$SID" ]; then
    ok "resume reused session id ($SID2)"
  else
    bad "resume session id mismatch ('$SID2' != '$SID')"
  fi
else
  bad "skipped resume (no session id)"
fi

# ---------------------------------------------------------------------------
# 4. Verify the session actually exercised each capability.
# ---------------------------------------------------------------------------
hdr "activity evidence"
if grep -q 'shell-egress' "$SCRATCH/shell_out.txt" 2>/dev/null; then
  ok "shell tool ran (shell_out.txt contains shell-egress)"
else
  bad "shell tool output missing in $SCRATCH/shell_out.txt"
fi
if python3 - "$WORK/s2.json" <<'PY'
import json, sys
try:
    txt = json.load(open(sys.argv[1])).get("text", "")
except Exception:
    sys.exit(1)
sys.exit(0 if "RESUMED" in txt.upper() else 1)
PY
then
  ok "resumed headless turn ran and replied RESUMED"
else
  bad "resumed turn did not return the expected reply"
fi
if grep -q 'edited' "$SCRATCH/edit_me.txt" 2>/dev/null && ! grep -q 'original' "$SCRATCH/edit_me.txt" 2>/dev/null; then
  ok "edit tool ran (edit_me.txt now: $(tr -d '\n' <"$SCRATCH/edit_me.txt"))"
else
  bad "edit tool did not change edit_me.txt"
fi
if [ -s "$MARKER" ]; then
  ok "MCP tool ran: $(tr '\n' ' ' <"$MARKER")"
else
  bad "local MCP tool was not called (no marker at $MARKER)"
fi

COMPACT_HITS="$(grep -hE 'auto-compact trigger|Replacing chat history \(compaction\)|compaction_tokens_before' \
  "$WORK/s1.debug.log" "$WORK/s2.debug.log" 2>/dev/null | awk 'END { print NR+0 }')"
[ -z "$COMPACT_HITS" ] && COMPACT_HITS=0
if [ "${COMPACT_HITS:-0}" -gt 0 ]; then
  ok "auto-compaction ran ($COMPACT_HITS debug marker(s); threshold=1%)"
else
  bad "no compaction marker found in debug logs"
fi

# ---------------------------------------------------------------------------
# 5. Parse the connect log, build the allowlist, print the table, gate.
# ---------------------------------------------------------------------------
hdr "observed destinations"
python3 - "$EGRESS_LOG" "$API_HOST" "$WEB_HOST" <<'PY'
import ipaddress, socket, sys
from collections import Counter

log_path, api_host, web_host = sys.argv[1], sys.argv[2], sys.argv[3]
hosts = [api_host, web_host]

host_addrs = {h: set() for h in hosts}
for h in hosts:
    try:
        for fam, _, _, _, sa in socket.getaddrinfo(h, None):
            host_addrs[h].add(sa[0])
    except OSError:
        pass

connects, getaddr = [], []
try:
    with open(log_path) as fh:
        for line in fh:
            line = line.strip()
            if not line:
                continue
            op = line.split(None, 1)[0]
            if op in ("CONNECT", "SENDTO", "SENDMSG", "SENDMMSG"):
                kv = dict(p.split("=", 1) for p in line.split()[1:] if "=" in p)
                connects.append((kv.get("family", "other"), kv.get("addr", "?"), kv.get("port", "0")))
            elif line.startswith("GETADDR "):
                kv = dict(p.split("=", 1) for p in line.split()[1:] if "=" in p)
                getaddr.append((kv.get("name", "?"), kv.get("addr", "?")))
except FileNotFoundError:
    print("no egress log found")
    raise SystemExit(2)

addr_host = {}
for name, addr in getaddr:
    addr_host.setdefault(addr, set()).add(name)
    if name in host_addrs:
        host_addrs[name].add(addr)

allowed_ips = set()
for s in host_addrs.values():
    allowed_ips |= s


def classify(family, addr, port):
    if family == "unix":
        return True, "unix"
    if family in ("inet", "inet6"):
        try:
            ip = ipaddress.ip_address(addr)
        except ValueError:
            return False, "unparsable"
        if ip.is_loopback:
            return True, "loopback"
        if ip.is_unspecified:
            return True, "unspecified"
        # Public destinations must be HTTPS: a shared CDN IP on another port is
        # still not the provider/web_fetch contract.
        try:
            p = int(port)
        except ValueError:
            return False, "unparsable-port"
        if p != 443:
            return False, "non-443 port %s" % p
        if addr in allowed_ips:
            return True, "allowlisted"
        if isinstance(ip, ipaddress.IPv6Address) and ip.ipv4_mapped \
                and str(ip.ipv4_mapped) in allowed_ips:
            return True, "allowlisted"
        return False, "NOT-ALLOWED"
    return False, "other-family"


counts = Counter(connects)
web_host_ips = host_addrs.get(web_host, set())
api_host_ips = host_addrs.get(api_host, set())
web_connects = 0
api_connects = 0
violations = []
rows = []
for (family, addr, port), cnt in counts.most_common():
    allowed, reason = classify(family, addr, port)
    host = ",".join(sorted(addr_host.get(addr, []))) or "-"
    rows.append((cnt, family, addr, port, host, "PASS" if allowed else "FAIL", reason))
    if not allowed:
        violations.append((family, addr, port, cnt))
    if family in ("inet", "inet6"):
        if addr in web_host_ips or (addr in addr_host and web_host in addr_host[addr]):
            web_connects += cnt
        if addr in api_host_ips or (addr in addr_host and api_host in addr_host[addr]):
            api_connects += cnt

print("count  family  destination                                      port  host                 status  reason")
print("-----  ------  -----------------------------------------------  ----  -------------------  ------  ----------")
for cnt, family, addr, port, host, status, reason in rows:
    print("%5d  %-6s  %-47s  %4s  %-19s  %-6s  %s" % (cnt, family, addr, port, host, status, reason))
print()
print("allowlisted IPs for %s: %s" % (api_host, ", ".join(sorted(api_host_ips)) or "(none)"))
print("allowlisted IPs for %s: %s" % (web_host, ", ".join(sorted(web_host_ips)) or "(none)"))
print()
print("WEB_FETCH_CONNECTS=%d" % web_connects)
print("API_CONNECTS=%d" % api_connects)
print("EGRESS_VIOLATIONS=%d" % len(violations))

if violations:
    print()
    print("VIOLATIONS:")
    for family, addr, port, cnt in violations:
        print("  %s %s:%s (x%d)" % (family, addr, port, cnt))
    raise SystemExit(1)
raise SystemExit(0)
PY
PARSE_RC=$?
if [ "$PARSE_RC" -eq 0 ]; then
  ok "egress allowlist: all observed destinations are allowed"
else
  bad "egress allowlist: unexpected destination(s) observed"
fi

# web_fetch must have produced a real network connection to its host.
if python3 - "$EGRESS_LOG" "$WEB_HOST" <<'PY'
import socket, sys
log_path, web_host = sys.argv[1], sys.argv[2]
ips = {sa[0] for _, _, _, _, sa in socket.getaddrinfo(web_host, None)}
seen = set()
for line in open(log_path):
    if line.startswith("CONNECT "):
        for p in line.split()[1:]:
            if p.startswith("addr="):
                seen.add(p.split("=", 1)[1])
raise SystemExit(0 if (ips & seen) else 1)
PY
then
  ok "web_fetch made a real connection to $WEB_HOST"
else
  bad "no connect() to $WEB_HOST observed; web_fetch may not have run"
fi

# ---------------------------------------------------------------------------
# 6. sandbox-run.sh comparison (blocks everything but the provider + loopback).
# ---------------------------------------------------------------------------
hdr "sandbox-run.sh (blocking guard) comparison"
SB_PROMPT="Use the shell tool (run_terminal_cmd) to run: echo sandbox-probe. Then use web_fetch to fetch $WEB_FETCH_URL. Then reply with one short line."
HOME="$HOME_DIR" \
DEEPSEEK_BUILD_HOME="$DSH" \
GROK_AUTO_COMPACT_THRESHOLD_PERCENT=1 \
GROK_WEB_FETCH=1 \
BIN="$BIN" \
TERM=dumb \
timeout "$SANDBOX_TIMEOUT" "$REPO_ROOT/scripts/sandbox-run.sh" \
  --cwd "$SCRATCH" --always-approve --no-plan -m deepseek-v4-pro \
  --output-format json -p "$SB_PROMPT" \
  >"$WORK/sandbox.json" 2>"$WORK/sandbox.err"
SB_RC=$?
if [ "$SB_RC" -eq 0 ]; then
  note "sandbox-run.sh session exited 0"
else
  note "sandbox-run.sh session exit=$SB_RC"
fi
if [ -s "$WORK/sandbox.json" ]; then
  note "sandbox-run.sh result: $(python3 - "$WORK/sandbox.json" <<'PY'
import json, sys
try:
    d = json.load(open(sys.argv[1]))
    print(d.get("text", "").strip().replace("\n", " ")[:200])
except Exception:
    print("(unreadable)")
PY
)"
fi
BLOCKED="$(grep -c 'sandbox-run: BLOCKED connect' "$WORK/sandbox.err" 2>/dev/null || true)"
BLOCKED="${BLOCKED:-0}"
note "sandbox-run.sh BLOCKED connect lines: $BLOCKED"
grep 'sandbox-run: BLOCKED connect' "$WORK/sandbox.err" 2>/dev/null | sort | uniq -c || true
note ""
note "note: sandbox-run.sh formats only AF_INET destinations; its AF_INET6/other"
note "      refusals print as 'non-ip'. The provider is reached over its IPv6"
note "      NAT64 address (see the allow-log table), which sandbox-run.sh does not"
note "      allowlist, so those IPv6 attempts are refused before falling back to"
note "      the allowlisted IPv4 provider address. The web_fetch host is refused"
note "      as expected, so the fetch fails while the session still completes."

# ---------------------------------------------------------------------------
# 7. Verdict.
# ---------------------------------------------------------------------------
hdr "verdict"
if [ "$FAILED" -eq 0 ] && [ "$PARSE_RC" -eq 0 ]; then
  ok "EGRESS-CHECK PASS"
  exit 0
else
  bad "EGRESS-CHECK FAIL"
  exit 1
fi
