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

1. Derive a short task slug from the requested work.
2. Check whether `.worktree/<task-slug>` already exists.
   - If it exists and matches the intended branch, reuse it.
   - If it exists but points at unrelated work, choose a new slug instead of overwriting it.
3. Check whether branch `opencode/<task-slug>` already exists.
   - If it exists, attach or create the worktree from that branch.
   - If it does not exist, create it from the current default base branch.
4. Perform implementation, verification, and git operations from inside that worktree instead of the primary checkout.
5. Run the smallest relevant validation for the change. Prefer repo commands from `AGENTS.md`, including `pnpm check`, `pnpm lint`, `pnpm test:unit`, `pnpm test:e2e`, and `pnpm build` when applicable.
6. Commit only when the user explicitly asks for a commit.
7. When the work is ready for review, push the branch and create a GitHub pull request with `gh pr create`.
8. Return the PR URL to the user.

## Guardrails

- Do not implement from the primary checkout when a dedicated worktree can be created.
- Do not work directly on `main`.
- Do not push directly to `main` or `master`.
- Do not skip PR creation when the user asked for reviewable implementation work.
- Do not delete or overwrite an existing `.worktree/<task-slug>` that may contain unrelated work.

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
- The final handoff includes a PR URL once the user asks for reviewable changes.
