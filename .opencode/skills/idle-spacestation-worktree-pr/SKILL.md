---
name: idle-spacestation-worktree-pr
description: Use a dedicated .worktree task checkout, a separate branch, and a GitHub pull request for implementation work in this repository.
compatibility: opencode
---

# Idle Spacestation worktree + PR workflow

Use this skill for implementation work in this repository whenever code, docs, config, or tests need to change.

## Outcome

- Work happens in a dedicated git worktree at `.worktree/<task-slug>`.
- The worktree uses a separate branch, typically `opencode/<task-slug>`.
- The final delivery is a GitHub pull request that the user can review.

## Required workflow

1. If the current directory is the primary checkout, do not start implementation there. Stop, inspect `.worktree/`, and move the work into a dedicated worktree first.
2. Derive a short task slug from the requested work.
3. Check whether `.worktree/<task-slug>` already exists.
   - If it exists and matches the intended branch, reuse it.
   - If it exists but points at unrelated work, choose a new slug instead of overwriting it.
4. Check whether branch `opencode/<task-slug>` already exists.
   - If it exists, attach or create the worktree from that branch.
   - If it does not exist, create it from the current default base branch.
5. Perform implementation, verification, and git operations from inside that worktree instead of the primary checkout.
6. Run the smallest relevant validation for the change. Prefer repo commands from `AGENTS.md`, including `pnpm check`, `pnpm lint`, `pnpm test:unit`, `pnpm test:e2e`, and `pnpm build` when applicable.
7. Commit only when the user explicitly asks for a commit.
8. When the work is ready for review, push the branch and create a GitHub pull request with `gh pr create`.
9. Return the PR URL to the user.

## Guardrails

- Do not implement from the primary checkout when a dedicated worktree can be created.
- Do not treat `/start-work` without `--worktree` as permission to keep working in the primary checkout; create or attach the dedicated worktree first.
- Do not work directly on `main`.
- Do not push directly to `main` or `master`.
- Do not skip PR creation when the user asked for reviewable implementation work.
- Do not delete or overwrite an existing `.worktree/<task-slug>` that may contain unrelated work.

## Repo-local limitation

- This repository can document and reinforce the worktree-first rule in `AGENTS.md` and this skill, but no repo-local override for builtin `/start-work` behavior was found in `.opencode/opencode.json`.

## Naming

- Worktree path: `.worktree/<task-slug>`
- Branch name: `opencode/<task-slug>`

Use a slug that is short, lowercase, and hyphenated.

## Suggested command pattern

```bash
ls -la ".worktree"
GIT_MASTER=1 git branch --list "opencode/<task-slug>"
GIT_MASTER=1 git worktree add -b "opencode/<task-slug>" ".worktree/<task-slug>" main
```

If the branch already exists, prefer:

```bash
GIT_MASTER=1 git worktree add ".worktree/<task-slug>" "opencode/<task-slug>"
```

After implementation and validation, if review is requested:

```bash
GIT_MASTER=1 git push -u origin "opencode/<task-slug>"
gh pr create --title "<title>" --body "<body>"
```

## Acceptance checks

- A request to implement a feature starts in `.worktree/<task-slug>` instead of the main checkout.
- A request to fix a bug uses a separate `opencode/<task-slug>` branch.
- If the target slug already exists, the workflow reuses the matching worktree or chooses a safe new slug.
- If execution begins in the primary checkout, the next step is to stop and create or attach the dedicated worktree before implementing anything.
- The final handoff includes a PR URL once the user asks for reviewable changes.
