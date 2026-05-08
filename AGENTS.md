# PROJECT KNOWLEDGE BASE

**Generated:** 2026-05-08
**Commit:** f4c4b51
**Branch:** main

## OVERVIEW

Desktop-first Tauri 2 app: a SvelteKit 2 + Svelte 5 SPA frontend wired to a Rust idle-game simulation backend. The Rust side runs a tick loop (~250 ms cadence) over a stateful `RunState` and exposes 10 production + 9 debug-only `#[tauri::command]` functions. The frontend talks to the backend exclusively through `src/lib/game/api/gateway.ts` and consumes state via a **push-based reactive store** (`gameState` in `src/lib/game/api/state.svelte.ts`) that subscribes to a `game://state-changed` Tauri event. Polling is gone: the Rust tick loop and every mutating command call a `commit_and_emit()` helper that diffs against a last-emitted snapshot cache and only fires the event when state actually changes. A debug-only devtools overlay (mounted by the root layout, gated by `cfg(debug_assertions)` and the MCP bridge plugin) lets developers mutate game state live across six panels, with a focus-aware deferral so in-flight edits aren't clobbered by inbound snapshots.

The repo is no longer a thin template; treat the existing patterns as load-bearing.

## INHERITANCE

- Root rules apply everywhere.
- `src/AGENTS.md` adds frontend, Svelte 5, shadcn-svelte, devtools, game-API, Storybook, and test guidance.
- `src-tauri/AGENTS.md` adds Rust/Tauri command, simulation, persistence, event-emission, and packaging guidance.

## SELF-MAINTENANCE (READ BEFORE FINISHING A TASK)

Before declaring a task complete, decide whether your changes invalidate any AGENTS.md file. **You are responsible for keeping this hierarchy honest** — stale architecture docs cause measurably worse work from future agents.

**You MUST update an AGENTS.md when your changes do any of the following** (smallest-scoped file first: `src/AGENTS.md` or `src-tauri/AGENTS.md` before root):

- Change the data-flow contract between layers (e.g. polling → push, sync → async, new transport interface).
- Rename, add, or remove a public type/function/event/command that the docs name explicitly (`gameState`, `commit_and_emit`, `STATE_CHANGED_EVENT`, any `#[tauri::command]`, etc.).
- Add or remove a top-level subsystem under `src/lib/`, `src/lib/game/`, `src/lib/components/`, `src-tauri/src/`, or `src-tauri/src/game/`.
- Introduce a new project-wide convention (test layout, file-naming rule, lifecycle owner) or invalidate one already documented.
- Add or remove a directory referenced in the STRUCTURE block, or change a path/filename mentioned in WHERE TO LOOK.
- Discover a new anti-pattern that bit you (footgun, deadlock, race) — write it down so the next agent doesn't repeat it.
- Wire up something currently labelled "scaffolded" / "not wired" (e.g. `persistence/`).

**You MAY skip the update when:**

- Your change is a localized bugfix that touches no documented contract or path.
- Line counts drift by <10% on a single file (treat absolute line numbers as approximate; rewrite them only when you're already in the section).
- You are the agent reading this and the request is purely a question, exploration, or analysis with no committed changes.

**When you do update:**

1. Update the most-specific file first (`src/AGENTS.md` or `src-tauri/AGENTS.md`); only touch root `AGENTS.md` if the change crosses both halves or affects the project-wide overview/conventions/anti-patterns.
2. Refresh the `**Generated:**` date and `**Commit:**` short hash at the top of the root file when you touch root. Use `date +%Y-%m-%d` and `git rev-parse --short HEAD`.
3. Verify any line numbers, line counts, or file paths you cite by reading the actual file or running `wc -l` — never guess.
4. Stage the AGENTS.md update **in the same commit** as the code change it documents. Don't leave it as a follow-up "docs:" commit; the docs and the code must travel together so `git blame` stays useful.
5. If you are unsure whether an update is warranted, default to updating — a small inaccuracy compounds across future sessions.

## STRUCTURE

```text
./
├── src/                                      # SvelteKit SPA, game-API gateway, gameState store, devtools panels, colocated tests
├── src-tauri/                                # Rust simulation backend (root crate `idle_spacestation_lib`)
│   ├── src/
│   │   ├── main.rs                           # Windows-subsystem guarded entrypoint
│   │   ├── lib.rs                            # Builder, GameState, commit_and_emit, all_commands! macro, tick thread
│   │   ├── runtime.rs                        # Runtime projection helpers (refresh_runtime_state, power/crew calc)
│   │   ├── commands/                         # Production + devtools Tauri command handlers (split by domain)
│   │   │   └── devtools/                     # Debug-only devtools handlers, inputs, apply helpers
│   │   └── game/                             # Simulation, content, progression, persistence, snapshot DTOs
│   └── idle-spacestation-bit-eq-derive/      # In-repo proc-macro crate for BitEq/BitHash derives
├── .storybook/                               # Storybook 10 config (sveltekit framework, addon-vitest, addon-a11y)
├── .opencode/                                # Per-repo OpenCode config (skills/ directory exists but is empty)
├── .sisyphus/                                # Agent workspace: plans/, evidence/, drafts/, notepads/, boulder.json
├── .worktree/                                # Plugin-managed worktrees; gitignored
├── static/                                   # Public assets served by SvelteKit
├── build/                                    # Generated frontend output (consumed by Tauri as frontendDist)
├── storybook-static/                         # Generated Storybook output
└── .svelte-kit/                              # Generated SvelteKit output
```

## WHERE TO LOOK

| Task                       | Location                                                           | Notes                                                                                                                                                                        |
| -------------------------- | ------------------------------------------------------------------ | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Frontend route/layout work | `src/routes/`                                                      | SPA routes: `/`, `/systems`, `/services`, `/planets`, `/prestige`, `/demo`                                                                                                   |
| Game state & commands (FE) | `src/lib/game/api/`                                                | `gateway.ts` (318), `adapters.ts`, `types.ts` (647), `state.svelte.ts` (124, store)                                                                                          |
| Reactive snapshot store    | `src/lib/game/api/state.svelte.ts`                                 | `gameState` rune singleton: `init/dispose/snapshot/status/error/applySnapshot`                                                                                               |
| Test fixtures + transport  | `src/lib/game/api/testing/`                                        | `fixtures.ts` + `transport.ts` (971, fixture transport notifies subscribers)                                                                                                 |
| Devtools overlay & panels  | `src/lib/components/DevtoolsOverlay.svelte`, `…/devtools/`         | 6 panels, each `Panel.svelte` + `panel-state.svelte.ts` + tests; `sync(snapshot)`                                                                                            |
| Shared UI primitives       | `src/lib/components/ui/`                                           | shadcn-svelte primitives: `button/`, `card/` (7 parts), `input/`                                                                                                             |
| Shared frontend helpers    | `src/lib/utils.ts`                                                 | `cn()` plus `WithoutChild`, `WithoutChildren`, `WithElementRef` type helpers                                                                                                 |
| Rust commands & state      | `src-tauri/src/lib.rs`                                             | 405 lines: builder, `GameState`, `LastEmittedSnapshot`, `commit_and_emit`, tick                                                                                              |
| Event emission helper      | `src-tauri/src/lib.rs` (`commit_and_emit`)                         | Fires `game://state-changed` only when `state_equals()` reports a diff                                                                                                       |
| Tauri command handlers     | `src-tauri/src/commands/`                                          | Production handlers split by domain: `service.rs` (367), `system.rs`, `progression.rs`, `snapshot_cmds.rs`, plus shared `inputs.rs` + `action_response()` helper in `mod.rs` |
| Devtools command handlers  | `src-tauri/src/commands/devtools/`                                 | Debug-only: `mod.rs` (373), `handlers.rs` (319), `apply.rs` (822), `inputs.rs` (142); gated by `#[cfg(debug_assertions)]`                                                    |
| Runtime projection helpers | `src-tauri/src/runtime.rs`                                         | 190 lines: `refresh_runtime_state`, crew/power capacity, projected_power_after_toggle                                                                                        |
| BitEq/BitHash proc-macros  | `src-tauri/idle-spacestation-bit-eq-derive/src/lib.rs`             | 256 lines: in-repo proc-macro crate; expands `#[derive(BitEq)]` / `#[derive(BitHash)]` field-by-field with `#[bit_hash(order = N)]` and `#[bit_hash(sort)]` attrs            |
| Game simulation core       | `src-tauri/src/game/sim/`                                          | `state.rs` (416), `tick.rs` (720, 6-phase loop), `deficit.rs`                                                                                                                |
| Game content (static data) | `src-tauri/src/game/content/`                                      | `systems.rs`, `services.rs`, `planets.rs`, `doctrines.rs`, `resources.rs`                                                                                                    |
| Progression & prestige     | `src-tauri/src/game/progression/`                                  | `prestige.rs` (697, PrestigeProfile), `doctrines.rs`, `survey.rs`                                                                                                            |
| Persistence (scaffolded)   | `src-tauri/src/game/persistence/`                                  | `SaveManager`, versioned `SaveData`, recovery; not wired into commands yet                                                                                                   |
| IPC DTO + state diff       | `src-tauri/src/game/snapshot.rs`                                   | 1502 lines: camelCase serde DTOs + `state_equals()` (derived `BitEq`, uses `f32::to_bits()`)                                                                                 |
| Tauri runtime config       | `src-tauri/tauri.conf.json`, `src-tauri/capabilities/default.json` | Window 800×600, `withGlobalTauri: true`, `csp: null`, opener + mcp-bridge perms                                                                                              |
| Dev/build/test commands    | `package.json`, `playwright.config.ts`, `vite.config.js`           | pnpm-driven; vitest config embedded in `vite.config.js`                                                                                                                      |

## CONVENTIONS

- Package manager is **pnpm**. `pnpm-workspace.yaml` is single-project (only configures `allowBuilds` + `onlyBuiltDependencies` for `@hugeicons/svelte` and `esbuild`); it is not a multi-package monorepo.
- Frontend runs as SPA: `adapter-static` with `fallback: 'index.html'` in `svelte.config.js`, and `ssr = false` in `src/routes/+layout.ts`.
- Tests are colocated with source. Three flavors:
  - `*.spec.ts` — Vitest Node-environment unit tests (gateway, panel state machines, `state.svelte.spec.ts`).
  - `*.svelte.spec.ts` — Vitest browser tests via `vitest-browser-svelte` + Chromium.
  - `*.e2e.ts` — Playwright tests against `pnpm preview` on port 4173.
- Frontend → Rust calls go through **`gameGateway`** from `src/lib/game/api`. Do not call `@tauri-apps/api/core` `invoke` directly from components; the gateway aliases command names (e.g. `game_set_service_activation` → `game_toggle_service`, `game_confirm_prestige` → `game_execute_prestige`) and handles the `{ input: payload }` envelope.
- Frontend → state reads go through **`gameState`** from `$lib/game/api/state.svelte`. Routes derive their data with `$derived(gameState.snapshot?.routes.*)`; they never poll `gameGateway.getSnapshot()` on a timer. The root layout owns `gameState.init()` / `dispose()` lifecycle.
- After a successful mutation (any command that returns a fresh snapshot in `result.snapshot`), the calling component **must** call `gameState.applySnapshot(result.snapshot)` for zero-latency UI updates. The store reconciles by `meta.tickCount` and ignores stale snapshots.
- Devtools panels follow a strict `Panel.svelte` + `panel-state.svelte.ts` split. State files use Svelte 5 runes (`$state`, `$derived`) inside a `createXxxPanelState()` factory; the `.svelte` file owns presentation only. Each factory exposes `sync(snapshot)` which the parent (`DevtoolsOverlay`) calls when its snapshot prop changes — panels do **not** subscribe to `gameState` directly.
- UI primitives live under `src/lib/components/ui/<component>/` with a `.svelte` implementation and `index.ts` re-export, matching shadcn-svelte conventions.
- Repo-local OpenCode skills live under `.opencode/skills/` — currently empty, so there is nothing to prefer over built-in skills yet.

## ANTI-PATTERNS (THIS PROJECT)

- Do not edit generated output: `.svelte-kit/`, `build/`, `storybook-static/`, `src-tauri/target/`, `src-tauri/gen/`.
- Do not call `invoke()` directly from Svelte components; route everything through `gameGateway`. The gateway is also the one place that knows which Rust command name a frontend command maps to.
- Do not poll `gameGateway.getSnapshot()` on a timer in routes or layouts. Subscribe via `gameState` (which subscribes once to `game://state-changed`) and react with `$derived`.
- Do not add a new Tauri command without registering it inside the `all_commands!` macro in `src-tauri/src/lib.rs` (~`lib.rs:86`). Devtools commands must carry `#[cfg(debug_assertions)]` immediately before the identifier so they are stripped from release builds; production commands are unconditional. The macro expands to a single `tauri::generate_handler![...]` — do not split it back into two invocations. Handler implementations themselves live under `src-tauri/src/commands/` (production) and `src-tauri/src/commands/devtools/` (debug-only), not in `lib.rs`.
- Do not add a new mutating Rust command without calling `commit_and_emit(&app, &run, &profile, &last_emitted)` after the mutation. Bypassing it leaves the frontend stale until the next tick.
- Do not emit `game://state-changed` directly from a command; always go through `commit_and_emit` so the diff cache and lock order stay correct.
- Do not invert the lock order. `commit_and_emit` acquires `LastEmittedSnapshot` only after the caller has dropped (or never held) `GameState`'s mutex. Holding both in the wrong order will deadlock.
- Do not treat `src/stories/`, `src/lib/vitest-examples/`, or `src/routes/demo/` as production patterns unless the task explicitly targets examples/demo code.
- Do not remove the Windows subsystem guard in `src-tauri/src/main.rs` (`DO NOT REMOVE!!`).
- Do not add SSR-dependent code; the SPA-only assumption is load-bearing for the Tauri integration.
- Do not bypass the `gameGateway` rejection-code pattern by parsing error strings; failure modes are typed (`SystemUpgradeRejectionCode`, `ServiceActivationRejectionCode`, …).
- Do not route TypeScript fixes through `no-undef`; ESLint intentionally disables it (see `eslint.config.js`).
- Do not compare snapshot floats with raw `==` in Rust — use `f32::to_bits()` (or the helpers in `snapshot.rs`) so NaN bit patterns and FP equality stay deterministic.

## UNIQUE STYLES

- Svelte 5 runes are pervasive: `$state`, `$props`, `$derived`, `$bindable`, `$effect`, plus `Snippet`-typed `children`. Match that style.
- Tailwind v4 is driven through `src/routes/layout.css` with shadcn-svelte theme tokens and Inter Variable font. `components.json` is authoritative for aliases (`components`, `ui`, `utils`, `hooks`, `lib`) and the `mira` style + hugeicons icon library.
- **Push-based snapshot distribution**: backend tick loop runs at 250 ms (4 Hz) on a daemon thread spawned in `setup()` and calls `commit_and_emit()` on every tick. The helper diffs against `LastEmittedSnapshot` via `state_equals()` and only fires `game://state-changed` when something actually changed. Tick errors are logged, never panicked.
- **Float-bit equality**: `state_equals()` (in `snapshot.rs`) uses `f32::to_bits()` for every f32 field so NaN-vs-NaN and rounding noise are handled consistently. Plain integers/strings/enums use `==`.
- **Devtools focus deferral**: while an editable input inside `[data-testid="devtools-overlay"]` has focus, the layout calls `gameState.deferUntilBlur(true)`. Inbound snapshots are buffered and applied on blur so the user's draft isn't overwritten mid-edit.
- Devtools mode is unlocked by `#[cfg(debug_assertions)]` in Rust **and** is also enable-able from the frontend by setting `localStorage['idle-spacestation.transport-mode'] = 'fixture'`, which routes the gateway through the in-memory `createFixtureTransport()` instead of Tauri. The fixture transport notifies all `subscribeToStateChanges` subscribers on every mutation, mirroring real backend events. E2E tests rely on this transport.

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
- `src/lib/game/api/state.svelte.ts` (`gameState`) and `src/lib/game/api/state.svelte.spec.ts` are the canonical entry points for understanding the push-based store. `src/lib/game/api/testing/transport.subscribe.spec.ts` exercises the fixture transport's notify path.
- A future split is justified if `src/lib/game/api/` or `src-tauri/src/game/` grows new top-level subsystems; today, three AGENTS files cover the surface.
