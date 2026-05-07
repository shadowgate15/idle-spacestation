# PROJECT KNOWLEDGE BASE

**Generated:** 2026-05-07
**Commit:** 42c4cb7
**Branch:** main

## OVERVIEW

Desktop-first Tauri 2 app: a SvelteKit 2 + Svelte 5 SPA frontend wired to a Rust idle-game simulation backend. The Rust side runs a tick loop (~250 ms cadence) over a stateful `RunState` and exposes 10 production + 9 debug-only `#[tauri::command]` functions. The frontend talks to the backend exclusively through `src/lib/game/api/gateway.ts`, which can also swap in an in-memory fixture transport for tests and previews. A debug-only devtools overlay (mounted by the root layout, gated by `cfg(debug_assertions)` and the MCP bridge plugin) lets developers mutate game state live across six panels.

The repo is no longer a thin template; treat the existing patterns as load-bearing.

## INHERITANCE

- Root rules apply everywhere.
- `src/AGENTS.md` adds frontend, Svelte 5, shadcn-svelte, devtools, game-API, Storybook, and test guidance.
- `src-tauri/AGENTS.md` adds Rust/Tauri command, simulation, persistence, and packaging guidance.

## STRUCTURE

```text
./
├── src/                 # SvelteKit SPA, game-API gateway, devtools panels, colocated tests
├── src-tauri/           # Rust simulation backend, Tauri commands, packaging
├── .storybook/          # Storybook 10 config (sveltekit framework, addon-vitest, addon-a11y)
├── .opencode/           # Per-repo OpenCode config (skills/ directory exists but is empty)
├── .sisyphus/           # Agent workspace: plans/, evidence/, drafts/, notepads/, boulder.json
├── .worktree/           # Plugin-managed worktrees; gitignored
├── static/              # Public assets served by SvelteKit
├── build/               # Generated frontend output (consumed by Tauri as frontendDist)
├── storybook-static/    # Generated Storybook output
└── .svelte-kit/         # Generated SvelteKit output
```

## WHERE TO LOOK

| Task                          | Location                                                          | Notes                                                                              |
| ----------------------------- | ----------------------------------------------------------------- | ---------------------------------------------------------------------------------- |
| Frontend route/layout work    | `src/routes/`                                                     | SPA routes: `/`, `/systems`, `/services`, `/planets`, `/prestige`, `/demo`         |
| Game state & commands (FE)    | `src/lib/game/api/`                                               | `gateway.ts`, `adapters.ts`, `types.ts` (651 lines), `testing/` fixtures+transport |
| Devtools overlay & panels     | `src/lib/components/DevtoolsOverlay.svelte`, `…/devtools/`        | 6 panels, each `Panel.svelte` + `panel-state.svelte.ts` + tests                    |
| Shared UI primitives          | `src/lib/components/ui/`                                          | shadcn-svelte primitives: `button/`, `card/` (7 parts), `input/`                   |
| Shared frontend helpers       | `src/lib/utils.ts`                                                | `cn()` plus `WithoutChild`, `WithoutChildren`, `WithElementRef` type helpers       |
| Rust commands & state         | `src-tauri/src/lib.rs`                                            | 1711 lines: 19 `#[tauri::command]` fns, `GameState` mutex, tick thread, plugin reg |
| Game simulation core          | `src-tauri/src/game/sim/`                                         | `state.rs`, `tick.rs` (6-phase loop), `deficit.rs`                                 |
| Game content (static data)    | `src-tauri/src/game/content/`                                     | `systems.rs`, `services.rs`, `planets.rs`, `doctrines.rs`, `resources.rs`          |
| Progression & prestige        | `src-tauri/src/game/progression/`                                 | `prestige.rs` (PrestigeProfile), `doctrines.rs`, `survey.rs`                       |
| Persistence (scaffolded)      | `src-tauri/src/game/persistence/`                                 | `SaveManager`, versioned `SaveData`, recovery; not wired into commands yet         |
| IPC DTO layer                 | `src-tauri/src/game/snapshot.rs`                                  | 1214 lines of camelCase serde DTOs returned to the frontend                        |
| Tauri runtime config          | `src-tauri/tauri.conf.json`, `src-tauri/capabilities/default.json`| Window 800×600, `withGlobalTauri: true`, `csp: null`, opener + mcp-bridge perms    |
| Dev/build/test commands       | `package.json`, `playwright.config.ts`, `vite.config.js`          | pnpm-driven; vitest config embedded in `vite.config.js`                            |

## CONVENTIONS

- Package manager is **pnpm**. `pnpm-workspace.yaml` is single-project (only configures `allowBuilds` + `onlyBuiltDependencies` for `@hugeicons/svelte` and `esbuild`); it is not a multi-package monorepo.
- Frontend runs as SPA: `adapter-static` with `fallback: 'index.html'` in `svelte.config.js`, and `ssr = false` in `src/routes/+layout.ts`.
- Tests are colocated with source. Three flavors:
  - `*.spec.ts` — Vitest Node-environment unit tests (gateway, panel state machines).
  - `*.svelte.spec.ts` — Vitest browser tests via `vitest-browser-svelte` + Chromium.
  - `*.e2e.ts` — Playwright tests against `pnpm preview` on port 4173.
- Frontend → Rust calls go through **`gameGateway`** from `src/lib/game/api`. Do not call `@tauri-apps/api/core` `invoke` directly from components; the gateway aliases command names (e.g. `game_set_service_activation` → `game_toggle_service`, `game_confirm_prestige` → `game_execute_prestige`) and handles the `{ input: payload }` envelope.
- Devtools panels follow a strict `Panel.svelte` + `panel-state.svelte.ts` split. State files use Svelte 5 runes (`$state`, `$derived`) inside a `createXxxPanelState()` factory; the `.svelte` file owns presentation only.
- UI primitives live under `src/lib/components/ui/<component>/` with a `.svelte` implementation and `index.ts` re-export, matching shadcn-svelte conventions.
- Repo-local OpenCode skills live under `.opencode/skills/` — currently empty, so there is nothing to prefer over built-in skills yet.

## ANTI-PATTERNS (THIS PROJECT)

- Do not edit generated output: `.svelte-kit/`, `build/`, `storybook-static/`, `src-tauri/target/`, `src-tauri/gen/`.
- Do not call `invoke()` directly from Svelte components; route everything through `gameGateway`. The gateway is also the one place that knows which Rust command name a frontend command maps to.
- Do not add a new Tauri command without also adding it to **both** `invoke_handler` lists in `src-tauri/src/lib.rs::run()` (debug and release branches are separate `tauri::generate_handler!` macros).
- Do not treat `src/stories/`, `src/lib/vitest-examples/`, or `src/routes/demo/` as production patterns unless the task explicitly targets examples/demo code.
- Do not remove the Windows subsystem guard in `src-tauri/src/main.rs` (`DO NOT REMOVE!!`).
- Do not add SSR-dependent code; the SPA-only assumption is load-bearing for the Tauri integration.
- Do not bypass the `gameGateway` rejection-code pattern by parsing error strings; failure modes are typed (`SystemUpgradeRejectionCode`, `ServiceActivationRejectionCode`, …).
- Do not route TypeScript fixes through `no-undef`; ESLint intentionally disables it (see `eslint.config.js`).

## UNIQUE STYLES

- Svelte 5 runes are pervasive: `$state`, `$props`, `$derived`, `$bindable`, `$effect`, plus `Snippet`-typed `children`. Match that style.
- Tailwind v4 is driven through `src/routes/layout.css` with shadcn-svelte theme tokens and Inter Variable font. `components.json` is authoritative for aliases (`components`, `ui`, `utils`, `hooks`, `lib`) and the `mira` style + hugeicons icon library.
- Backend tick loop runs at ~250 ms (4 Hz) on a daemon thread spawned in `setup()` (`lib.rs:1638-…`); the frontend reflects state by polling `gameGateway.getSnapshot()` from each route's `onMount`/`$effect`.
- Devtools mode is unlocked by `#[cfg(debug_assertions)]` in Rust **and** is also enable-able from the frontend by setting `localStorage['idle-spacestation.transport-mode'] = 'fixture'`, which routes the gateway through the in-memory `createFixtureTransport()` instead of Tauri. E2E tests rely on this fixture transport.

## COMMANDS

```bash
pnpm dev              # Vite dev server (port 1420, strict)
pnpm build            # Vite production build into ./build
pnpm preview          # Serve ./build on port 4173 (Playwright target)
pnpm check            # svelte-kit sync && svelte-check
pnpm lint             # prettier --check . && eslint .
pnpm format           # prettier --write .
pnpm test             # pnpm test:unit -- --run && pnpm test:e2e
pnpm test:unit        # vitest (browser + server projects, see vite.config.js)
pnpm test:e2e         # playwright test (boots pnpm build && pnpm preview)
pnpm storybook        # storybook dev on :6006
pnpm tauri dev        # tauri dev (uses pnpm dev as beforeDevCommand)
pnpm tauri build      # tauri build (uses pnpm build as beforeBuildCommand)
```

`pnpm prepare` runs `playwright install` automatically — required before E2E tests work on a fresh checkout.

## NOTES

- `.sisyphus/` is the agent workspace (plans, evidence, drafts) and is gitignored. Treat it as scratch space, not project documentation.
- `src/lib/hooks/` does not exist on disk even though `components.json` aliases `hooks → $lib/hooks`. Verify before importing from that alias.
- `src/lib/server/` does not exist; the SPA mode means there is no server-side route code.
- Vitest config is embedded in `vite.config.js` (no standalone `vitest.config.*`); it defines two projects: `client` (browser/Chromium) and `server` (Node). Don't add a separate config without removing this one.
- A future split is justified if `src/lib/game/api/` or `src-tauri/src/game/` grows new top-level subsystems; today, three AGENTS files cover the surface.
