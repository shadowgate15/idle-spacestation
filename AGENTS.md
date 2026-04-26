# PROJECT KNOWLEDGE BASE

**Generated:** 2026-04-25
**Commit:** 9ddf6c9
**Branch:** main

## OVERVIEW
Desktop-first Tauri 2 app with a SvelteKit 2 + Svelte 5 frontend and a small Rust backend.
Repo is still close to template scale; keep the hierarchy minimal and ignore generated output.

## INHERITANCE
- Root rules apply everywhere.
- `src/AGENTS.md` adds frontend, Svelte, shadcn-svelte, Storybook, and test guidance.
- `src-tauri/AGENTS.md` adds Rust/Tauri guidance.

## STRUCTURE
```text
./
├── src/                 # SvelteKit app, shared UI, stories, colocated tests
├── src-tauri/           # Rust entrypoint, Tauri config, desktop packaging
├── .storybook/          # Storybook config only
├── static/              # Public assets
├── build/               # Generated frontend output
├── storybook-static/    # Generated Storybook output
├── .svelte-kit/         # Generated SvelteKit output
├── .sisyphus/           # Agent notes/evidence; not app code
└── .opencode/           # Tooling metadata and repo-local OpenCode skills
```

## WHERE TO LOOK
| Task | Location | Notes |
|---|---|---|
| Frontend page/layout work | `src/routes/` | SvelteKit routes, root layout, route tests |
| Shared UI primitives | `src/lib/components/ui/` | shadcn-svelte style exports via `index.ts` |
| Shared frontend helpers | `src/lib/utils.ts` | `cn()` plus reusable TS helper types |
| Storybook behavior | `.storybook/`, `src/stories/` | Config lives in `.storybook`; stories are example/demo oriented |
| Desktop commands/config | `src-tauri/src/`, `src-tauri/tauri.conf.json` | Rust commands exposed to frontend via `invoke` |
| Dev/build/test commands | `package.json`, `playwright.config.ts`, `vite.config.js` | pnpm-oriented Tauri + SvelteKit flow |

## CONVENTIONS
- Package manager is pnpm. Tauri config runs `pnpm dev` and `pnpm build`.
- Frontend runs as SPA mode: `adapter-static` plus `src/routes/+layout.ts` with `ssr = false`.
- Tests are colocated with source: `*.spec.ts` for Vitest, `*.e2e.ts` for Playwright.
- UI primitives live under `src/lib/components/ui/<component>/` with a `.svelte` implementation and `index.ts` re-export.
- Storybook exists for component isolation, but app code does not depend on `src/stories/`.
- Repo-local OpenCode skills live under `.opencode/skills/` and should be preferred when a reusable repo workflow exists.

## WORKFLOW
- For implementation work, start from a dedicated worktree under `.worktree/<task-slug>` on a separate branch instead of editing the primary checkout.
- Use the repo-local OpenCode skill `idle-spacestation-worktree-pr` for the expected workflow: inspect `.worktree/`, reuse an existing worktree only when it is clearly the same task and branch, otherwise create a new `.worktree/<task-slug>` checkout before making implementation changes.
- Running `/start-work` without `--worktree` does not waive this repo rule. If the current directory is the primary checkout, stop and create or attach the dedicated worktree first.

## ANTI-PATTERNS (THIS PROJECT)
- Do not edit generated output: `.svelte-kit/`, `build/`, `storybook-static/`, `src-tauri/target/`, `src-tauri/gen/schemas/`.
- Do not treat `src/stories/`, `src/lib/vitest-examples/`, or `src/routes/demo/` as production patterns unless the task explicitly targets examples/demo code.
- Do not remove the Windows subsystem guard in `src-tauri/src/main.rs` (`DO NOT REMOVE!!`).
- Do not route TypeScript fixes through `no-undef`; ESLint intentionally disables it for this repo.
- Do not start implementation from the primary checkout just because `/start-work` was invoked without `--worktree`; that is a stop condition, not an allowed fallback.

## UNIQUE STYLES
- Svelte 5 runes are in use (`$props`, `$state`, `$bindable`). Match that style in frontend code.
- Tailwind v4 is driven through `src/routes/layout.css` with shadcn-svelte theme tokens and Inter Variable font.
- `components.json` is authoritative for shadcn-svelte aliases and theme style (`mira`, hugeicons, neutral base color).

## COMMANDS
```bash
pnpm dev
pnpm build
pnpm check
pnpm lint
pnpm test:unit
pnpm test:e2e
pnpm storybook
pnpm tauri dev
```

## NOTES
- This repo is small; prefer updating these three AGENTS files over adding deeper ones.
- Candidate future split only if `src/lib/components/ui/` grows materially beyond the current handful of primitives.
- Root `src/app.html` and Tauri config are still template-like; verify before documenting custom behavior as intentional.
