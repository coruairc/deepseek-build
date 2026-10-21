# deepseek-build clone (not available)

The upstream Grok Build `clone` command (a Grove content-store clone with a
projected working tree) is **not part of deepseek-build**. The binary has no
`clone` subcommand, and `deepseek-build worktree` has no `redirect`
subcommands; running `deepseek-build clone` fails with an unrecognized
subcommand error.

To work in a separate checkout, use a normal `git clone`, or a session
worktree with `deepseek-build -w [NAME]` / `deepseek-build worktree create`
(see [Sessions](17-sessions.md)).
