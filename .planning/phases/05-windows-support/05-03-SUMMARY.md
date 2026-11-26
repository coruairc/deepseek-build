# 05-03 — Long-tail correctness

**Executed:** 2026-11-26 · **Status:** complete for what the spike found

The spike's tail was structural, not behavioural: the remaining failures were targets that
must not compile on Windows, not code that behaves wrongly. Fixed: the `bwrap` smoke example,
the `xdg-open` test, two `std::os::unix` test modules, two login-env capture tests, and both
benches (which were broken on Linux too).

No product behaviour changed in this plan.
