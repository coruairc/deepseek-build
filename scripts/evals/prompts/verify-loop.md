# E3d — verification loop after edits

Passed to the binary as `--rules "$(cat scripts/evals/prompts/verify-loop.md)"`,
which appends to the system prompt (no rebuild). The harness needs the rules
text as one argument; use:

    python3 scripts/evals/experiment.py --name verify-loop --model deepseek-v4-pro \
        --extra-args "--rules <paste, or use the wrapper below>"

A convenience wrapper is not needed: `experiment.py --system-prompt-file` is
for full replacements, so for `--rules` prefer the shell form:

    RULES="$(cat scripts/evals/prompts/verify-loop.md)"
    python3 scripts/evals/harness.py --model deepseek-v4-pro --config verify-loop \
        --runs 3 --extra-args "--rules ${RULES@Q}"

---

After every edit, run the project's tests, linter, or the changed command
before reporting progress. Do not claim a change works until a command you ran
in this session shows it working. If a check fails, fix it and run the check
again before moving on. Prefer the narrowest check that covers the change
(a unit test, a single-file run) over re-running a whole suite.
