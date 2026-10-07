#!/usr/bin/env bash
#
# smoke-test.sh — live DeepSeek smoke test for deepseek-build.
#
# Repeats the required adapter checks against the real API through the built
# binary and reports PASS/FAIL per check. It NEVER prints the API key.
#
# Usage:
#   export DEEPSEEK_API_KEY=...        # or DEEPSEEK_BUILD_API_KEY
#   scripts/smoke-test.sh
#   BIN=target/debug/deepseek-build scripts/smoke-test.sh
#
# Checks:
#   1. key present (DEEPSEEK_API_KEY or DEEPSEEK_BUILD_API_KEY)
#   2. GET /models returns deepseek-v4-pro and deepseek-flash
#   3. plain headless chat returns a reply, exit 0
#   4. streamed headless chat emits stream events
#   5. 3+ turn thinking tool-call chain runs (num_turns/modelCalls >= 3)
#   6. usage carries reasoning_tokens and cache read/creation fields
#   7. deepseek-flash and the hidden deepseek-v4-flash alias are accepted
#   8. raw /chat/completions cached_tokens == prompt_cache_hit_tokens on a cache hit
#
set -uo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
BIN="${BIN:-$REPO_ROOT/target/release/deepseek-build}"
BASE_URL="${DEEPSEEK_BASE_URL:-https://api.deepseek.com}"
WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT
HOME_DIR="$WORK/home"; mkdir -p "$HOME_DIR"
SCRATCH="$WORK/scratch"; mkdir -p "$SCRATCH"

KEY="${DEEPSEEK_API_KEY:-${DEEPSEEK_BUILD_API_KEY:-}}"
PASS=0; FAIL=0
ok()   { PASS=$((PASS+1)); printf 'PASS  %s\n' "$1"; }
bad()  { FAIL=$((FAIL+1)); printf 'FAIL  %s\n' "$1"; }

if [ -z "$KEY" ]; then
  bad "API key present — set DEEPSEEK_API_KEY or DEEPSEEK_BUILD_API_KEY"
  printf '\n%d passed, %d failed\n' "$PASS" "$FAIL"
  exit 1
fi
ok "API key present (DEEPSEEK_API_KEY or DEEPSEEK_BUILD_API_KEY)"

if [ ! -x "$BIN" ]; then
  bad "binary at $BIN (build with: cargo build --release -p xai-grok-pager-bin)"
  printf '\n%d passed, %d failed\n' "$PASS" "$FAIL"
  exit 1
fi

export HOME="$HOME_DIR"
export DEEPSEEK_API_KEY="$KEY"

# --- 2. /models -----------------------------------------------------------
if api_check_out="$(BASE_URL="$BASE_URL" python3 - <<'PY' 2>&1
import json,os,urllib.request
base=os.environ["BASE_URL"]
key=os.environ["DEEPSEEK_API_KEY"]
req=urllib.request.Request(base+"/models",headers={"Authorization":"Bearer "+key})
try:
    with urllib.request.urlopen(req,timeout=30) as r:
        d=json.loads(r.read()); ids=[m.get("id") for m in d.get("data",[])]
except Exception as e:
    print("ERROR",type(e).__name__); raise SystemExit(1)
missing=[m for m in ("deepseek-v4-pro","deepseek-flash") if m not in ids]
if missing:
    print("MISSING",missing,"have",ids); raise SystemExit(1)
print("OK",ids)
PY
)"; then
  ok "/models accepts deepseek-v4-pro and deepseek-flash ($api_check_out)"
else
  bad "/models check ($api_check_out)"
fi

# --- 3. plain chat --------------------------------------------------------
plain_out="$(timeout 120 "$BIN" -p 'Reply with exactly: pong' 2>"$WORK/plain.err")"; plain_rc=$?
if [ "$plain_rc" -eq 0 ] && printf '%s' "$plain_out" | grep -qi pong; then
  ok "plain headless chat returned a reply (exit 0)"
else
  bad "plain headless chat (exit $plain_rc)"
fi

# --- 4. streamed chat -----------------------------------------------------
timeout 120 "$BIN" -p 'Count from 1 to 5, one number per line.' \
  --output-format streaming-messages-json --include-partial-messages \
  >"$WORK/stream.ndjson" 2>"$WORK/stream.err"; stream_rc=$?
if [ "$stream_rc" -eq 0 ] && grep -q '"stream_event"' "$WORK/stream.ndjson" && grep -q '"assistant"' "$WORK/stream.ndjson"; then
  ok "streamed headless chat emitted stream events"
else
  bad "streamed headless chat (exit $stream_rc)"
fi

# --- 5/6. 3-turn thinking tool chain + usage fields -----------------------
(
  cd "$SCRATCH" && rm -f a.txt
  timeout 300 "$BIN" --always-approve --no-plan -m deepseek-v4-pro --output-format json \
    -p 'Do these steps strictly in order, exactly one tool call per assistant turn, waiting for each tool result before the next call: (1) use the shell to write the single line alpha to a.txt; (2) use the shell to cat a.txt; (3) use the shell to append the single line beta to a.txt; (4) use the shell to cat a.txt; (5) report the final contents.' \
    >"$WORK/tools.json" 2>"$WORK/tools.err"
); tools_rc=$?
if [ "$tools_rc" -eq 0 ] && python3 - "$WORK/tools.json" <<'PY'
import json,sys
d=json.load(open(sys.argv[1]))
assert d.get("num_turns",0)>=3, d.get("num_turns")
assert sum(v.get("modelCalls",0) for v in (d.get("modelUsage") or {}).values())>=3
PY
then
  ok "3-turn thinking tool-call chain (num_turns>=3)"
else
  bad "3-turn thinking tool-call chain (exit $tools_rc)"
fi

if usage_line="$(python3 - "$WORK/tools.json" <<'PY'
import json,sys
u=json.load(open(sys.argv[1])).get("usage") or {}
for k in ("reasoning_tokens","cache_read_input_tokens","cache_creation_input_tokens","input_tokens","output_tokens"):
    assert k in u, "missing "+k
print("reasoning=%s cache_read=%s cache_create=%s" % (u["reasoning_tokens"],u["cache_read_input_tokens"],u["cache_creation_input_tokens"]))
PY
)"; then
  ok "usage carries reasoning/cache fields ($usage_line)"
else
  bad "usage fields (reasoning_tokens / cache_read_input_tokens / cache_creation_input_tokens)"
fi

# --- 7. flash models ------------------------------------------------------
flash_ok=1
for m in deepseek-flash deepseek-v4-flash; do
  timeout 120 "$BIN" -m "$m" --output-format json -p 'Reply with exactly: pong' \
    >"$WORK/$m.json" 2>"$WORK/$m.err" || flash_ok=0
  python3 - "$WORK/$m.json" <<'PY' || flash_ok=0
import json,sys
d=json.load(open(sys.argv[1]))
assert "deepseek-flash" in (d.get("modelUsage") or {}), d.get("modelUsage")
PY
done
[ "$flash_ok" -eq 1 ] && ok "deepseek-flash and hidden deepseek-v4-flash alias accepted" || bad "flash model acceptance"

# --- 8. cached_tokens == prompt_cache_hit_tokens --------------------------
if BASE_URL="$BASE_URL" python3 - <<'PY' 2>&1
import json,os,urllib.request
base=os.environ["BASE_URL"]; key=os.environ["DEEPSEEK_API_KEY"]
prefix="You are a helpful assistant. "*400
def call(n):
    body={"model":"deepseek-v4-pro","thinking":{"type":"enabled"},
          "messages":[{"role":"system","content":prefix},{"role":"user","content":f"ping {n}"}]}
    req=urllib.request.Request(base+"/chat/completions",data=json.dumps(body).encode(),
        headers={"Authorization":"Bearer "+key,"Content-Type":"application/json"})
    with urllib.request.urlopen(req,timeout=90) as r: return json.loads(r.read())
call(1); u=call(2)["usage"]
cached=u["prompt_tokens_details"]["cached_tokens"]; hit=u["prompt_cache_hit_tokens"]
assert hit>0, "no cache hit observed"
assert cached==hit, (cached,hit)
assert u["prompt_tokens"]==hit+u["prompt_cache_miss_tokens"]
PY
then
  ok "raw usage: cached_tokens == prompt_cache_hit_tokens on a cache hit"
else
  bad "raw usage cached_tokens == prompt_cache_hit_tokens"
fi

printf '\n%d passed, %d failed\n' "$PASS" "$FAIL"
[ "$FAIL" -eq 0 ]
