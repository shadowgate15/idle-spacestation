# FRONTEND KNOWLEDGE BASE

## OVERVIEW

SvelteKit 2 SPA on Svelte 5 runes, Tailwind v4, and shadcn-svelte primitives. The frontend has four load-bearing pieces:

1. **Routes** under `src/routes/` for the five game screens (`/`, `/systems`, `/services`, `/planets`, `/prestige`) plus the root layout.
2. **Game API gateway** under `src/lib/game/api/` that abstracts every Rust call behind `gameGateway`, with adapters that turn raw snapshots into per-route ViewModels and a fixture transport for tests/previews.
3. **Reactive snapshot store** (`gameState` in `src/lib/game/api/state.svelte.ts`) — a Svelte 5 rune singleton that subscribes once to the backend's `game://state-changed` event and exposes the adapted snapshot to every route and the devtools overlay. Polling is gone.
4. **Devtools overlay** under `src/lib/components/DevtoolsOverlay.svelte` + `src/lib/components/devtools/`, with six panels for live game-state mutation. Mounted by the root layout, snapshot driven by `gameState`, gated by `cfg(debug_assertions)` on the backend and `fixture` mode on the frontend.

## STRUCTURE

```text
src/
├── routes/                       # SPA routes + root layout + colocated tests
│   ├── +layout.svelte            # 209 lines: header nav, devtools mount, gameState lifecycle, focus deferral, IPC events
│   ├── +layout.ts                # SPA mode: ssr = false
│   ├── layout.css                # Tailwind v4 entry + shadcn theme tokens + Inter font
│   ├── +page.svelte              # 172 lines: Overview (5 StatPanels: Resources, Crew & Power, Station Tier, Service Utilization, Survey Progress)
│   ├── page.svelte.spec.ts       # Vitest browser test (overview)
│   ├── page.svelte.e2e.ts        # Playwright (overview)
│   ├── layout.svelte.spec.ts     # Vitest browser test for devtools/IPC behavior
│   ├── devtools-overlay.e2e.ts   # Playwright devtools regression suite
│   ├── systems/  services/  planets/  prestige/   # Each: +page.svelte + .spec.ts + .e2e.ts
│   └── demo/                     # Template demo + demo/playwright/ examples
├── lib/
│   ├── game/api/                 # Gateway + adapters + types + reactive store + fixtures
│   │   ├── index.ts              # Public barrel (gateway, adapters, types, state, testing)
│   │   ├── gateway.ts            # 318 lines: createGameGateway + tauriTransport (with subscribeToStateChanges)
│   │   ├── state.svelte.ts       # 124 lines: gameState rune store (init/dispose/snapshot/status/error/applySnapshot/deferUntilBlur)
│   │   ├── state.svelte.spec.ts  # Unit tests for the store: subscribe lifecycle, tick_count reconciliation, deferral
│   │   ├── adapters.ts           # adaptGameSnapshot + per-route ViewModel adapters
│   │   ├── types.ts              # 647 lines: snapshots, commands, rejection codes, GameStateInitError, GameTransport
│   │   ├── api.spec.ts           # Gateway integration tests
│   │   └── testing/              # fixtures.ts (4 named fixtures) + transport.ts (971 lines, notifies subscribers on every mutation) + transport.subscribe.spec.ts
│   ├── components/
│   │   ├── DevtoolsOverlay.svelte           # Panel registry + section layout; receives snapshot prop and forwards to panels
│   │   ├── devtools/                        # 6 panels × {Panel.svelte, panel-state.svelte.ts, *.spec.ts, *.svelte.spec.ts}
│   │   └── ui/                              # shadcn-svelte primitives (button, card, input, stat-tile, stat-panel, stat-row)
│   ├── utils.ts                  # cn() + WithoutChild / WithoutChildren / WithElementRef helpers
│   └── vitest-examples/          # Low-authority template (greet.ts, Welcome.svelte)
├── stories/                      # Storybook template content (Button/Header/Page/Configure/DesignTokens)
└── app.html                      # Tauri-friendly HTML shell
```

## WHERE TO LOOK

| Task                           | Location                                                                                                  | Notes                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                          |
| ------------------------------ | --------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| App shell, nav, devtools mount | `routes/+layout.svelte`, `routes/+layout.ts`, `routes/layout.css`                                         | Owns `gameState.init/dispose`, focus deferral, fixture-mode localStorage hooks                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                 |
| Per-route page + tests         | `routes/<name>/+page.svelte` (+ `.spec.ts`, `.e2e.ts`)                                                    | Each derives state with `$derived(gameState.snapshot?.routes.*)` — no polling                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                  |
| Frontend → Rust calls          | `lib/game/api/gateway.ts`                                                                                 | `gameGateway` is the only legitimate `invoke` consumer; also exposes subscribe                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                 |
| Reactive snapshot store        | `lib/game/api/state.svelte.ts`                                                                            | `gameState` rune singleton — single source of truth for the live snapshot                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                      |
| Per-route ViewModels           | `lib/game/api/adapters.ts`                                                                                | `adaptOverviewViewModel`, `adaptPlanetsViewModel`, etc.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                        |
| Game types & rejection codes   | `lib/game/api/types.ts`                                                                                   | Mirror Rust DTOs; rejection enums per command; `GameStateInitError`                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                            |
| Test fixtures & mock transport | `lib/game/api/testing/`                                                                                   | `starter`, `deficit`, `all-planets`, `prestige-ready`; transport notifies subs                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                 |
| Devtools overlay & panels      | `lib/components/DevtoolsOverlay.svelte`, `lib/components/devtools/`                                       | 6 panels: Resources, Crew, Systems, Services, Progression, Session                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                             |
| Shared UI primitives           | `lib/components/ui/`                                                                                      | `button/`, `card/` (7 parts), `input/`, `stat-tile/`, `stat-panel/`, `stat-row/` — each with `index.ts` barrel                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                 |
| Grouped stat layout            | `lib/components/ui/stat-panel/stat-panel.svelte` (33), `lib/components/ui/stat-row/stat-row.svelte` (231) | StatPanel = heading + `<dl class="grid grid-cols-[auto_1fr_auto]">`; StatRow uses `display: contents` so each row's 3 cells (label \| value+rate \| bar) participate in the parent grid. StatRow takes a discriminated-union `kind` prop: `scalar`/`stock`/`capacity`/`ratio`/`progress`/`label`. Used on Overview (top-level panels) and Planets/Prestige (single panels). Systems and Services use the raw `<dl class="grid grid-cols-[auto_1fr_auto] …">` directly inside `Card.Content` (no StatPanel wrapper — the Card already provides framing) with StatRows for per-entity stats. The legacy `StatTile` primitive is preserved but has no current consumers in routes |
| Shared helpers                 | `lib/utils.ts`                                                                                            | `cn()` and component-prop type helpers                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                         |
| Storybook examples             | `../.storybook/`, `stories/`                                                                              | Template-only; not consumed by the app                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                         |

## CONVENTIONS

### Working with Svelte files

- For `.svelte`, `.svelte.ts`, and `.svelte.js` work, prefer `task(subagent_type="svelte-file-editor", load_skills=[], …)`.
- For Svelte/SvelteKit lookup, call `svelte_list-sections` first, then `svelte_get-documentation` for only the relevant sections.
- Always run `svelte_svelte-autofixer` before returning Svelte code.
- Avoid loading `svelte-code-writer` and `svelte-core-bestpractices` until the runtime skill-registry issue is fixed.
- Treat shadcn-svelte docs as canonical for component APIs and theming; defer to local `components.json` for this repo's aliases (`components`, `ui`, `utils`, `hooks`, `lib`) and the `mira` style.

### Game API & data flow

- All backend calls go through `gameGateway` from `$lib/game/api`. Never call `@tauri-apps/api/core` `invoke` directly in components.
- All snapshot reads go through `gameState` from `$lib/game/api/state.svelte`. The root layout calls `gameState.init()` once on mount and `gameState.dispose()` on unmount; routes never call `init`/`dispose` themselves and never poll `gameGateway.getSnapshot()` on a timer.
- Pages derive their data reactively with `$derived(gameState.snapshot?.routes.<route>)` plus `$derived(gameState.status !== 'ready')` for loading and `$derived(gameState.error?.message ?? null)` for errors. See `routes/+page.svelte`, `routes/systems/+page.svelte`, etc. for the canonical shape.
- After a mutating gateway call, **always** push the returned snapshot back into the store: `gameState.applySnapshot(result.snapshot)`. The store reconciles by `meta.tickCount` and silently ignores stale snapshots, so the call is safe even if a backend event has already arrived.
- Action results are typed `GatewayActionResponse<TRejection>`; check `ok` and surface `reasonCode` rather than throwing.
- Add new commands by extending `GameCommandName`/`GameCommandPayloads`/`GameCommandResponses` in `types.ts`, then a method on `createGameGateway`. If the Rust command name differs, update the alias map in `tauriTransport.invoke` (`gateway.ts`).
- Add new transports by implementing `GameTransport` (and optionally `DevtoolsCommandTransport`) from `types.ts`. The `subscribeToStateChanges(callback, onError?) → unsubscribe` contract is mandatory; the fixture transport in `testing/transport.ts` is the reference implementation.

### Devtools panels

- Each panel under `src/lib/components/devtools/` is split into:
  - `XxxPanel.svelte` — presentation + event wiring only.
  - `xxx-panel-state.svelte.ts` — `createXxxPanelState()` factory using Svelte 5 runes (`$state`, `$derived`) for drafts/dirty/error, with `sync(snapshot)` and `apply()` methods that call the appropriate `gameGateway.devtools…` command.
- Panels do **not** import or subscribe to `gameState` directly. The overlay (`DevtoolsOverlay.svelte`) accepts the snapshot as a prop and forwards it; `sync(snapshot)` is invoked when the snapshot reference changes so the panel can reseed drafts.
- Tests come in pairs: `Panel.spec.ts` (state machine, Node) and (where useful) `Panel.svelte.spec.ts` (browser DOM).
- Register new panels inside `DevtoolsOverlay.svelte` so they appear under the right section.
- The overlay is only mounted/active when devtools visibility is on. While an editable input inside `[data-testid="devtools-overlay"]` has focus the layout calls `gameState.deferUntilBlur(true)`, which buffers inbound snapshots until blur — preserve that guard when adding new editable inputs.

### UI primitives

- Keep primitives in `lib/components/ui/<component>/` with an `index.ts` barrel and `.svelte` implementation. Match shadcn-svelte file naming.
- Use `cn()` from `$lib/utils` for class composition; for variants follow the `tailwind-variants` pattern in `button.svelte`.
- Match current naming: kebab-case filenames, PascalCase component exports, camelCase helpers/hooks.

### Tests

- Colocate tests with the route/component they cover.
- `*.spec.ts` → Vitest Node project. `*.svelte.spec.ts` → Vitest browser project (Chromium via `vitest-browser-svelte`). `*.e2e.ts` → Playwright against `pnpm preview` on port 4173.
- For Tauri-boundary tests, use `mockIPC`/`clearMocks` from `@tauri-apps/api/mocks` (see `routes/layout.svelte.spec.ts`).
- For E2E and browser tests that need a known starting state, seed via the fixture transport. Pre-set `localStorage['idle-spacestation.transport-mode'] = 'fixture'` (and the chosen fixture key) in `addInitScript`/test setup before navigating. The fixture transport calls every registered `subscribeToStateChanges` callback after each mutation, so reactive UI updates work the same way they do against the real backend.

## ANTI-PATTERNS

- Do not call `invoke()` directly from components — always go through `gameGateway`.
- Do not poll `gameGateway.getSnapshot()` (or any other gateway method) on a timer/interval. Subscribe via `gameState` and read with `$derived`.
- Do not call `gameState.init()` or `gameState.dispose()` from individual routes or panels. Only `routes/+layout.svelte` is allowed to manage the lifecycle.
- Do not forget to call `gameState.applySnapshot(result.snapshot)` after a successful mutating gateway call. Skipping it leaves the UI stale until the next backend event lands (still fast, but visibly slower).
- Do not bypass the focus-deferral guard when adding editable inputs inside the devtools overlay; new inputs must participate in the `[data-testid="devtools-overlay"]`-scoped focusin/focusout flow that drives `gameState.deferUntilBlur()`.
- Do not subscribe a devtools panel directly to `gameState`. Snapshots flow in through the overlay's prop and the panel's `sync(snapshot)` method — keep that boundary.
- Do not hardcode game data inside components; consume `GameSnapshot` / route ViewModels from the gateway adapters.
- Do not add a new devtools panel without its `panel-state.svelte.ts` factory and at least a `*.spec.ts` covering validation + apply.
- Do not copy template Storybook components from `stories/` into production UI without adapting them to repo conventions.
- Do not treat `lib/vitest-examples/` as app architecture; it is template-grade reference only.
- Do not add SSR-dependent code; the SPA + Tauri assumption is load-bearing.
- Do not bypass shadcn-svelte composition when a matching primitive already exists.
- Do not write raw class concatenation when `cn()` already covers the case.
- Do not duplicate the global header nav inside individual route pages. The `<header data-testid="game-header">` nav in `routes/+layout.svelte` (lines 176–196) is the single source of truth for navigation. Use the header links (which support middle-click, right-click, and bookmarking) instead of in-page button navs.

## UNIQUE STYLES

- `routes/layout.css` owns Tailwind v4 imports, shadcn theme tokens, and app-wide color variables.
- The root layout owns the snapshot subscription lifecycle: `gameState.init()` on mount (which calls `gameGateway.subscribeToStateChanges()` exactly once) and `gameState.dispose()` on destroy. There is no polling loop in the layout or anywhere else in the frontend.
- The store reconciles by `meta.tickCount`: `applySnapshot` ignores any incoming snapshot whose tick count is older than the one currently held. This makes "apply locally then receive event" race-safe without explicit dedup logic in callers.
- Snapshot delivery is paused while a devtools input is focused. The layout listens for focusin/focusout events scoped to `[data-testid="devtools-overlay"]` and toggles `gameState.deferUntilBlur(focused)`; pending snapshots flush automatically on blur. This is the modern replacement for the old "pause polling while editing" guard.
- Browser component tests use `vitest-browser-svelte` plus `@tauri-apps/api/mocks` for boundary tests.
- Playwright E2E runs against a generated build + preview server on port 4173 (see `playwright.config.ts`).
- Fixture mode is opt-in via `localStorage['idle-spacestation.transport-mode'] = 'fixture'`; in that mode the gateway uses `createFixtureTransport()` which keeps a `Set<callback>` of subscribers and notifies them on every mutation, mirroring the real backend's `game://state-changed` event. The layout also persists the devtools-open flag to `localStorage['idle-spacestation.devtools-open']` so E2E tests can restore visibility across navigations.

## NOTES

- `routes/demo/` and `routes/demo/playwright/` are template/demo content unless the task explicitly targets them.
- `lib/hooks` alias exists in `components.json` but the directory does not exist; verify before building around it.
- `lib/server/` does not exist (SPA mode); do not create it without revisiting the Tauri integration.
- The four named fixtures in `lib/game/api/testing/fixtures.ts` (`starter`, `deficit`, `all-planets`, `prestige-ready`) are the supported entry points for previews and E2E — extend that file rather than building ad-hoc fixtures inline in tests.
- `state.svelte.spec.ts` and `testing/transport.subscribe.spec.ts` are the canonical references for the push-based contract; read them before changing the store API or transport interface.
