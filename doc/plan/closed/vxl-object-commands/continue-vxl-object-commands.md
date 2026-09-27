# Continue the vxl object commands

1. Read [README.md](README.md), [checklist.md](checklist.md), and
   [implementation-decisions.md](implementation-decisions.md). Do not reopen a
   settled decision.
2. Run `git log --oneline -15` and `git status`.
3. Do the first unchecked step. Split it when it will not review as one
   change. Log code-level choices as you go.
4. Run fmt, clippy, and tests. Check the step off, stage, and stop for review.
