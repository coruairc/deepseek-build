# E3d — verification loop after edits

`--rules-file` passes a file's contents as `--rules` (appends to the system
prompt; no rebuild). The rules file is pure rules text with no header, because
every line is sent to the model. Reproduce:

```sh
python3 scripts/evals/experiment.py --name verify-loop --model deepseek-v4-pro \
    --rules-file scripts/evals/prompts/verify-loop.md
```

Candidate rules text (the whole file):

> After every edit, run the project's tests, linter, or the changed command
> before reporting progress. Do not claim a change works until a command you
> ran in this session shows it working. If a check fails, fix it and run the
> check again before moving on. Prefer the narrowest check that covers the
> change (a unit test, a single-file run) over re-running a whole suite.
