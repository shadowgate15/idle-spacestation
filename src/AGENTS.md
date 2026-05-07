# FRONTEND KNOWLEDGE BASE

## OVERVIEW

SvelteKit 2 SPA on Svelte 5 runes, Tailwind v4, and shadcn-svelte primitives. The frontend has three load-bearing pieces:

1. **Routes** under `src/routes/` for the five game screens (`/`, `/systems`, `/services`, `/planets`, `/prestige`) plus the root layout.
2. **Game API gateway** under `src/lib/game/api/` that abstracts every Rust call behind `gameGateway`, with adapters that turn raw snapshots into per-route ViewModels and a fixture transport for tests/previews.
3. **Devtools overlay** under `src/lib/components/DevtoolsOverlay.svelte` + `src/lib/components/devtools/`, with six panels for live game-state mutation. Mounted by the root layout, polling-driven, gated by `cfg(debug_assertions)` on the backend and `fixture` mode on the frontend.

## STRUCTURE

```text
src/
├── routes/                       # SPA routes + root layout + colocated tests
│   ├── +layout.svelte            # 220 lines: header nav, devtools mount, polling, IPC events
│   ├── +layout.ts                # SPA mode: ssr = false
│   ├── layout.css                # Tailwind v4 entry + shadcn theme tokens + Inter font
│   ├── +page.svelte              # 312 lines: Overview (resources, warnings, survey progress)
│   ├── page.svelte.spec.ts       # Vitest browser test (overview)
│   ├── page.svelte.e2e.ts        # Playwright (overview)
│   ├── layout.svelte.spec.ts     # Vitest browser test for devtools/IPC behavior
│   ├── devtools-overlay.e2e.ts   # Playwright devtools regression suite
│   ├── systems/  services/  planets/  prestige/   # Each: +page.svelte + .spec.ts + .e2e.ts
│   └── demo/                     # Template demo + demo/playwright/ examples
├── lib/
│   ├── game/api/                 # Gateway + adapters + types + testing fixtures
│   │   ├── index.ts              # Public barrel (gateway, adapters, types, testing)
│   │   ├── gateway.ts            # 280 lines: createGameGateway + tauriTransport + invoke helpers
│   │   ├── adapters.ts           # adaptGameSnapshot + per-route ViewModel adapters
│   │   ├── types.ts              # 651 lines: snapshots, commands, rejection codes
│   │   ├── api.spec.ts           # Gateway integration tests
│   │   └── testing/              # fixtures.ts (4 named fixtures) + transport.ts (mock invoke)
│   ├── components/
│   │   ├── DevtoolsOverlay.svelte           # Panel registry + section layout
│   │   ├── devtools/                        # 6 panels × {Panel.svelte, panel-state.svelte.ts, *.spec.ts, *.svelte.spec.ts}
│   │   └── ui/                              # shadcn-svelte primitives (button, card, input)
│   ├── utils.ts                  # cn() + WithoutChild / WithoutChildren / WithElementRef helpers
│   └── vitest-examples/          # Low-authority template (greet.ts, Welcome.svelte)
├── stories/                      # Storybook template content (Button/Header/Page/Configure/DesignTokens)
└── app.html                      # Tauri-friendly HTML shell
```

## WHERE TO LOOK

| Task                            | Location                                                      | Notes                                                                |
| ------------------------------- | ------------------------------------------------------------- | -------------------------------------------------------------------- |
| App shell, nav, devtools mount  | `routes/+layout.svelte`, `routes/+layout.ts`, `routes/layout.css` | Polling, IPC event listeners, fixture-mode localStorage hooks       |
| Per-route page + tests          | `routes/<name>/+page.svelte` (+ `.spec.ts`, `.e2e.ts`)        | Five game routes follow the same load → ViewModel → render shape    |
| Frontend → Rust calls           | `lib/game/api/gateway.ts`                                     | `gameGateway` is the only legitimate `invoke` consumer              |
| Per-route ViewModels            | `lib/game/api/adapters.ts`                                    | `adaptOverviewViewModel`, `adaptPlanetsViewModel`, etc.             |
| Game types & rejection codes    | `lib/game/api/types.ts`                                       | Mirror Rust DTOs; rejection enums per command                        |
| Test fixtures & mock transport  | `lib/game/api/testing/`                                       | `starter`, `deficit`, `all-planets`, `prestige-ready` fixtures       |
| Devtools overlay & panels       | `lib/components/DevtoolsOverlay.svelte`, `lib/components/devtools/` | 6 panels: Resources, Crew, Systems, Services, Progression, Session  |
| Shared UI primitives            | `lib/components/ui/`                                          | `button/`, `card/` (7 parts), `input/` — each with `index.ts` barrel |
| Shared helpers                  | `lib/utils.ts`                                                | `cn()` and component-prop type helpers                               |
| Storybook examples              | `../.storybook/`, `stories/`                                  | Template-only; not consumed by the app                               |

## CONVENTIONS

### Working with Svelte files

- For `.svelte`, `.svelte.ts`, and `.svelte.js` work, prefer `task(subagent_type="svelte-file-editor", load_skills=[], …)`.
- For Svelte/SvelteKit lookup, call `svelte_list-sections` first, then `svelte_get-documentation` for only the relevant sections.
- Always run `svelte_svelte-autofixer` before returning Svelte code.
- Avoid loading `svelte-code-writer` and `svelte-core-bestpractices` until the runtime skill-registry issue is fixed.
- Treat shadcn-svelte docs as canonical for component APIs and theming; defer to local `components.json` for this repo's aliases (`components`, `ui`, `utils`, `hooks`, `lib`) and the `mira` style.

### Game API & data flow

- All backend calls go through `gameGateway` from `$lib/game/api`. Never call `@tauri-apps/api/core` `invoke` directly in components.
- Pages load data with `await gameGateway.getSnapshot()` (or a more specific method) inside `onMount`, store it in `$state`, and pass an adapted `ViewModel` to UI. See `routes/+page.svelte` for the canonical shape (`fullSnapshot`, `overview`, `loading`, `error`, `isPolling`, `destroyed`).
- Action results are typed `GatewayActionResponse<TRejection>`; check `ok` and surface `reasonCode` rather than throwing.
- Add new commands by extending `GameCommandName`/`GameCommandPayloads`/`GameCommandResponses` in `types.ts`, then a method on `createGameGateway`. If the Rust command name differs, update the alias map in `tauriTransport.invoke` (`gateway.ts`).

### Devtools panels

- Each panel under `src/lib/components/devtools/` is split into:
  - `XxxPanel.svelte` — presentation + event wiring only.
  - `xxx-panel-state.svelte.ts` — `createXxxPanelState()` factory using Svelte 5 runes (`$state`, `$derived`) for drafts/dirty/error, with `sync(snapshot)` and `apply()` methods that call the appropriate `gameGateway.devtools…` command.
- Tests come in pairs: `Panel.spec.ts` (state machine, Node) and (where useful) `Panel.svelte.spec.ts` (browser DOM).
- Register new panels inside `DevtoolsOverlay.svelte` so they appear under the right section.
- The overlay is only mounted/active when devtools visibility is on. Polling pauses while an editable devtools input has focus (`isEditableDevtoolsInputFocused`) — preserve that guard when adding inputs.

### UI primitives

- Keep primitives in `lib/components/ui/<component>/` with an `index.ts` barrel and `.svelte` implementation. Match shadcn-svelte file naming.
- Use `cn()` from `$lib/utils` for class composition; for variants follow the `tailwind-variants` pattern in `button.svelte`.
- Match current naming: kebab-case filenames, PascalCase component exports, camelCase helpers/hooks.

### Tests

- Colocate tests with the route/component they cover.
- `*.spec.ts` → Vitest Node project. `*.svelte.spec.ts` → Vitest browser project (Chromium via `vitest-browser-svelte`). `*.e2e.ts` → Playwright against `pnpm preview` on port 4173.
- For Tauri-boundary tests, use `mockIPC`/`clearMocks` from `@tauri-apps/api/mocks` (see `routes/layout.svelte.spec.ts`).
- For E2E and browser tests that need a known starting state, seed via the fixture transport. Pre-set `localStorage['idle-spacestation.transport-mode'] = 'fixture'` (and the chosen fixture key) in `addInitScript`/test setup before navigating.

## ANTI-PATTERNS

- Do not call `invoke()` directly from components — always go through `gameGateway`.
- Do not hardcode game data inside components; consume `GameSnapshot` / route ViewModels from the gateway.
- Do not add a new devtools panel without its `panel-state.svelte.ts` factory and at least a `*.spec.ts` covering validation + apply.
- Do not poll inside individual components when the layout's polling already covers the snapshot you need.
- Do not copy template Storybook components from `stories/` into production UI without adapting them to repo conventions.
- Do not treat `lib/vitest-examples/` as app architecture; it is template-grade reference only.
- Do not add SSR-dependent code; the SPA + Tauri assumption is load-bearing.
- Do not bypass shadcn-svelte composition when a matching primitive already exists.
- Do not write raw class concatenation when `cn()` already covers the case.

## UNIQUE STYLES

- `routes/layout.css` owns Tailwind v4 imports, shadcn theme tokens, and app-wide color variables.
- The root layout polls `gameGateway.getDevtoolsState()` on a 1000 ms interval while the overlay is visible, and pauses polling while a devtools input is focused — both behaviors are tested in `routes/layout.svelte.spec.ts`.
- Browser component tests use `vitest-browser-svelte` plus `@tauri-apps/api/mocks` for boundary tests.
- Playwright E2E runs against a generated build + preview server on port 4173 (see `playwright.config.ts`).
- Fixture mode is opt-in via `localStorage['idle-spacestation.transport-mode'] = 'fixture'`; in that mode the gateway uses `createFixtureTransport()` and the layout also persists the devtools-open flag to `localStorage['idle-spacestation.devtools-open']` so E2E tests can restore visibility across navigations.

## NOTES

- `routes/demo/` and `routes/demo/playwright/` are template/demo content unless the task explicitly targets them.
- `lib/hooks` alias exists in `components.json` but the directory does not exist; verify before building around it.
- `lib/server/` does not exist (SPA mode); do not create it without revisiting the Tauri integration.
- The four named fixtures in `lib/game/api/testing/fixtures.ts` (`starter`, `deficit`, `all-planets`, `prestige-ready`) are the supported entry points for previews and E2E — extend that file rather than building ad-hoc fixtures inline in tests.
