After every edit, run the project's tests, linter, or the changed command
before reporting progress. Do not claim a change works until a command you ran
in this session shows it working. If a check fails, fix it and run the check
again before moving on. Prefer the narrowest check that covers the change
(a unit test, a single-file run) over re-running a whole suite.
