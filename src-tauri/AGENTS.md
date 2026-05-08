# TAURI / RUST KNOWLEDGE BASE

**Generated:** 2026-05-08
**Commit:** 0b2a29c

## OVERVIEW

The Rust side is the idle-game simulation engine plus the Tauri command surface that the SvelteKit SPA talks to. It is **not** template-tiny: `src/lib.rs` is ~150 lines of canonical state, emit, command registration, and builder glue; command handlers live under `src/commands/`, runtime projection helpers live in `src/runtime.rs`, and the full backend is split into a `game/` module covering simulation, content, progression, persistence, and IPC DTOs. A background thread drives the simulation tick at ~250 ms cadence, mutating shared state held under a `Mutex` and surfaced to the frontend via 19 `#[tauri::command]` functions (10 production, 9 debug-only). State changes are pushed to the SPA via a single `game://state-changed` Tauri event; every mutating command and the tick loop itself funnel through a `commit_and_emit()` helper that diffs against a `LastEmittedSnapshot` cache and only fires the event when state actually changed.

## STRUCTURE

```text
src-tauri/
├── src/
│   ├── main.rs               # Windows-subsystem guarded entrypoint → idle_spacestation_lib::run()
│   ├── lib.rs                # ~150 lines: state structs, commit_and_emit, all_commands!, Tauri builder, tick thread
│   ├── runtime.rs            # Runtime projection helpers: crew/power/service slots, active modifiers, refresh_runtime_state
│   ├── commands/
│   │   ├── mod.rs            # Command re-exports + action_response helper
│   │   ├── inputs.rs         # Production command input DTOs
│   │   ├── service.rs        # Service activation, crew assignment, priority handlers
│   │   ├── system.rs         # System upgrade handler
│   │   ├── progression.rs    # Doctrine purchase + prestige handlers
│   │   ├── snapshot_cmds.rs  # Snapshot, save/load stubs, survey handler
│   │   └── devtools/         # Debug-only devtools inputs, handlers, state helpers, apply helpers
│   └── game/
│       ├── mod.rs            # Re-exports: content, persistence, progression, snapshot, sim
│       ├── snapshot.rs       # 1480 lines: serde DTOs (camelCase) + state_equals() bit-stable diffing
│       ├── sim/
│       │   ├── mod.rs
│       │   ├── state.rs      # RunState, StationState, ResourceState, ServiceState, SystemState
│       │   ├── tick.rs       # 6-phase tick loop (alloc, activation, production, upkeep, deficit, survey)
│       │   └── deficit.rs    # Power-deficit shutdown ordering
│       ├── content/          # Static game data
│       │   ├── systems.rs    # Reactor / habitat / logistics / survey systems with per-level stats
│       │   ├── services.rs   # Service definitions + categories
│       │   ├── planets.rs    # Planet definitions + modifiers
│       │   ├── doctrines.rs  # Doctrine effects
│       │   └── resources.rs  # Resource metadata
│       ├── progression/
│       │   ├── prestige.rs   # PrestigeProfile, station-tier calc, eligibility, execute_prestige
│       │   ├── doctrines.rs  # Doctrine purchase + effect application
│       │   └── survey.rs     # Planet survey accumulation/unlocks
│       └── persistence/      # Scaffolded; not yet wired into the live save commands
│           ├── mod.rs        # SaveManager, SaveTrigger
│           ├── save.rs       # SaveData, SaveSettings, ProfileState (versioned)
│           ├── recovery.rs   # Corruption recovery, FreshProfileReason, SaveLoadOutcome
│           └── migration.rs  # Version migration helpers
├── icons/                    # Bundle assets
├── capabilities/default.json # core:default + opener:default + mcp-bridge:default
├── Cargo.toml                # Crate `idle_spacestation_lib` (the `_lib` suffix is required)
├── tauri.conf.json           # Window config, dev/build delegation to pnpm, withGlobalTauri, csp:null
├── build.rs                  # `tauri_build::build()` wrapper
├── target/                   # Generated; ignored
└── gen/                      # Generated Tauri schemas; ignored
```

## WHERE TO LOOK

| Task                       | Location                                                  | Notes                                                                          |
| -------------------------- | --------------------------------------------------------- | ------------------------------------------------------------------------------ |
| App entrypoint             | `src/main.rs`                                             | Calls `idle_spacestation_lib::run()`; preserves Windows subsystem guard        |
| Tauri commands & builder   | `src/lib.rs`, `src/commands/**`                           | `run()` + single `all_commands!` macro in `lib.rs`; handlers split by domain under `commands/` |
| Shared backend state       | `src/lib.rs` (`GameState`, `LastEmittedSnapshot`)         | `Mutex<(RunState, PrestigeProfile, u32 session_ticks)>` + last-emitted cache   |
| Devtools overlay state     | `src/lib.rs` (`DevtoolsState`), `src/commands/devtools/`  | Visibility flag, emitted via `devtools:visibility-changed` event               |
| Runtime projection helpers | `src/runtime.rs`                                          | Recomputes crew/power/service derived runtime fields after command mutations   |
| State-change emit          | `src/lib.rs` (`commit_and_emit`)                          | Single helper that diffs + emits `game://state-changed`; called by every mutator and the tick loop |
| State diff (bit-stable)    | `src/game/snapshot.rs` (`state_equals`)                   | Uses `f32::to_bits()` so NaN bit patterns + FP rounding are deterministic      |
| Simulation tick            | `src/game/sim/tick.rs`                                    | Six-phase loop; called every 250 ms by the background thread                   |
| Game data tables           | `src/game/content/*.rs`                                   | `SYSTEMS`, `SERVICES`, `PLANETS`, `DOCTRINES`, `RESOURCES` constants           |
| Prestige rules             | `src/game/progression/prestige.rs`                        | Tier calc, `PrestigeEligibility`, `execute_prestige`, stable-power timer       |
| IPC DTOs                   | `src/game/snapshot.rs`                                    | `RawGameSnapshot`, `ActionResponse`, `SaveLoadResponse`, route views           |
| Tauri runtime config       | `tauri.conf.json`                                         | Dev/build delegate to pnpm; window 800×600; `withGlobalTauri: true`            |
| Capabilities / permissions | `capabilities/default.json`                               | Permits `core:default`, `opener:default`, `mcp-bridge:default`                 |
| Cargo deps                 | `Cargo.toml`                                              | `tauri 2`, `tauri-plugin-opener 2`, `tauri-plugin-mcp-bridge 0.11`             |
| Build hook                 | `build.rs`                                                | Thin `tauri_build::build()`                                                    |

## TAURI COMMAND SURFACE

The Tauri builder in `lib.rs::run()` registers all commands via a single `all_commands!` macro defined just above `run()`. The macro expands to one `tauri::generate_handler![...]` invocation containing 10 production commands (always registered) plus 9 devtools commands each prefixed with `#[cfg(debug_assertions)]` so they are stripped from release builds. Handler implementations are split under `src/commands/`, but there is still exactly one handler-registration call site to maintain.

**Production commands (always registered):**

- `game_get_snapshot` — read-only, no emit
- `game_toggle_service` — mutating, calls `commit_and_emit`
- `game_upgrade_system` — mutating
- `game_assign_service_crew` — mutating
- `game_reprioritize_service` — mutating
- `game_start_survey` — mutating
- `game_purchase_doctrine` — mutating
- `game_execute_prestige` — mutating
- `game_request_save` — currently a snapshot return (persistence not wired)
- `game_request_load` — mutating once persistence is wired; today returns the live snapshot via `commit_and_emit`

**Debug-only commands (gated by `#[cfg(debug_assertions)]`):**

- `game_devtools_get_state` — read-only
- `game_devtools_set_visibility` — emits `devtools:visibility-changed`
- `game_devtools_apply_resources` — mutating, calls `commit_and_emit`
- `game_devtools_apply_crew`
- `game_devtools_apply_systems`
- `game_devtools_apply_services`
- `game_devtools_apply_progression`
- `game_devtools_advance_ticks`
- `game_devtools_reset_to_starter`

The frontend gateway (`src/lib/game/api/gateway.ts`) issues two of these commands under different names: `game_set_service_activation` (FE) → `game_toggle_service` (Rust), and `game_confirm_prestige` (FE) → `game_execute_prestige` (Rust). When renaming a Rust command, update the alias map in `gateway.ts` accordingly.

## EVENT EMISSION

- **Event name constant:** `STATE_CHANGED_EVENT = "game://state-changed"` (lib.rs).
- **Payload:** `RawGameSnapshot` (the full camelCase JSON DTO from `snapshot.rs`).
- **Emit helper:** `commit_and_emit(app, run, profile, last_emitted)` (lib.rs ~`fn commit_and_emit`):
  1. Builds a fresh `RawGameSnapshot` from the current `RunState` + `PrestigeProfile`.
  2. Locks `LastEmittedSnapshot` (a `Mutex<Option<RawGameSnapshot>>` registered as Tauri state).
  3. If `state_equals(prev, new)` returns `true`, returns early — no event is emitted.
  4. Otherwise, updates the cache, drops the lock, and emits `game://state-changed` with the new snapshot.
  5. Returns `Result<(), String>`; the tick loop logs errors and never panics.
- **Tick loop:** the daemon thread calls `commit_and_emit` on every iteration (every 250 ms). The diff cache makes "tick produced no observable change" cheap on the wire.
- **Lock order (load-bearing):** acquire `GameState`'s `Mutex` first, drop it, then call `commit_and_emit` (which locks `LastEmittedSnapshot`). Inverting the order risks deadlock with the tick thread.
- **Other events:** `devtools:visibility-changed` is emitted from `game_devtools_set_visibility`; it is unrelated to the game-state event stream.

## CONVENTIONS

- Crate name uses the `_lib` suffix (`idle_spacestation_lib`) to avoid Windows binary/lib name collisions; preserve that in `Cargo.toml`.
- All commands accept inputs as a single `input: T` argument (matching the gateway's `{ input: payload }` envelope) except for the no-arg commands listed in `gateway.ts` (`game_get_snapshot`, `game_start_survey`, `game_request_save`, `game_request_load`, `game_devtools_get_state`, `game_devtools_reset_to_starter`).
- Commands never panic on bad input. They return `ActionResponse { ok, snapshot, reason_code }` (or a typed devtools response) and surface failures via `reason_code` strings that mirror the rejection enums on the frontend.
- Mutating commands take `&AppHandle`, `State<'_, GameState>`, and `State<'_, LastEmittedSnapshot>` so they can call `commit_and_emit` after the mutation. Read-only commands (`game_get_snapshot`, `game_devtools_get_state`) skip both `AppHandle` and `LastEmittedSnapshot`.
- The pattern inside a mutator is: lock `GameState`, mutate `RunState`/`PrestigeProfile`, build the response snapshot, **drop the lock**, then call `commit_and_emit(&app, &run, &profile, &last_emitted)` and return the response.
- Serde DTOs in `snapshot.rs` use `#[serde(rename_all = "camelCase")]` so the wire shape matches the TypeScript types in `src/lib/game/api/types.ts`.
- The simulation owns mutation; `#[tauri::command]` functions take the lock briefly, mutate, and produce a fresh snapshot. Don't perform long-running work while holding `GameState`'s `Mutex`, and don't hold it across `commit_and_emit`.
- `state_equals` (in `snapshot.rs`) is the source of truth for "did anything change?". It uses `f32::to_bits()` for every f32 field (so NaN-vs-NaN compares equal and rounding noise is deterministic) and standard `==` for integers, strings, booleans, and enums. Use these helpers — never roll your own raw-`==` float comparison for snapshot diffing.
- **Canonical-helper rule**: shared physics helpers live in `src/game/sim/tick.rs` and are re-exported via `src/game/sim/mod.rs`. `lib.rs`/`runtime.rs`/`snapshot.rs` MUST import from there; do not re-implement.
- Plugins:
  - `tauri-plugin-opener` is registered in all builds.
  - `tauri-plugin-mcp-bridge` is registered only under `#[cfg(debug_assertions)]`. Treat its presence as a debug-only assumption.
- Background tick: spawned in `setup()` as a daemon thread with a 250 ms `thread::sleep` between ticks. Each iteration takes the `GameState` lock, runs `tick`, drops the lock, then calls `commit_and_emit`. Don't block this thread or change the cadence without auditing `tick.rs` invariants (deficit timing, autosave throttling, prestige stability counter).
- Persistence module is **scaffolded but not wired**: `game_request_save` currently returns the live snapshot without touching disk; `game_request_load` returns the live snapshot via `commit_and_emit`. Treat the persistence module as load-bearing infrastructure for future work, not as dead code to delete.

## ANTI-PATTERNS

- Do not edit `target/` or `gen/`; both are generated and ignored by `eslint.config.js`, `.prettierignore`, and `.gitignore`.
- Do not add a new `#[tauri::command]` without registering it inside the `all_commands!` macro in `src-tauri/src/lib.rs`. Devtools commands must carry `#[cfg(debug_assertions)]` immediately before the identifier; production commands are unconditional. Do **not** reintroduce two parallel `tauri::generate_handler!` invocations — the macro is the single source of truth.
- Do not add a new mutating command without calling `commit_and_emit(&app, &run, &profile, &last_emitted)` after the mutation. Skipping it leaves the frontend stale until the next tick fires the event for unrelated reasons.
- Do not emit `game://state-changed` directly via `app.emit(...)`. The diff cache + lock order live inside `commit_and_emit`; bypassing it produces spurious events and risks deadlocks.
- Do not invert the lock order. Always drop the `GameState` lock before calling `commit_and_emit` (which acquires `LastEmittedSnapshot`). Holding both simultaneously can deadlock against the tick thread.
- Do not compare snapshot floats with raw `==`. Use `f32::to_bits()` (or the helpers in `snapshot.rs`) so NaN bit patterns and FP rounding stay deterministic across diffs.
- Do not remove the Windows release guard in `src/main.rs`.
- Do not hold `GameState`'s `Mutex` across an `await` or any I/O.
- Do not change `RunState` or `PrestigeProfile` field shapes without bumping the `SAVE_VERSION` and adding a migration in `persistence/migration.rs` — even though save/load aren't wired yet, the on-disk format is the contract.
- Do not surface raw `panic!`/`unwrap` errors to the frontend; use `ActionResponse.reason_code`.
- Do not put new command-handler bulk back into `lib.rs`; keep production handlers in `src/commands/{service,system,progression,snapshot_cmds}.rs`, devtools handlers/helpers in `src/commands/devtools/`, and runtime projection helpers in `src/runtime.rs`.
- Do not add frontend assumptions here; UI rules belong in `src/AGENTS.md`.

## UNIQUE STYLES

- `tauri.conf.json` sets `withGlobalTauri: true` and a permissive `csp: null`; both choices affect runtime behavior immediately, so changes warrant deliberate review.
- `dev`/`build` are delegated to pnpm via `beforeDevCommand` and `beforeBuildCommand` (`pnpm dev` / `pnpm build`), and the frontend dist is `../build` — keep these aligned with `vite.config.js`.
- The MCP bridge plugin enables IDE/agent inspection of the running game state while developing; releases never ship with it.
- The 6-phase tick (alloc → activate → produce → upkeep → deficit → survey) is the load-bearing simulation contract. Cross-phase ordering matters for power deficit and autosave bookkeeping.
- Push-based state distribution: the tick loop and every mutating command share a single emit path (`commit_and_emit`) backed by a `LastEmittedSnapshot` diff cache. The frontend listens once via `gameGateway.subscribeToStateChanges()` and never polls.
- `state_equals` uses `f32::to_bits()` rather than raw `==` so NaN-vs-NaN comparisons resolve to "equal" when the bit patterns match, and floating-point rounding noise doesn't fire spurious events. Tests in `snapshot.rs` (`#[cfg(test)] mod state_equals_tests`) lock that contract in.

## NOTES

- Tests live beside their Rust modules (`src/lib.rs`, `src/commands/**`, and `src/game/**`). Run with `cargo test --manifest-path src-tauri/Cargo.toml` (or `cd src-tauri && cargo test`).
- Persistence integration (writing `SaveData` to disk on `Autosave`/`VisibilityHidden`/`WindowClose`/`BeforePrestige` triggers) is the obvious next wiring task; `SaveManager` already handles primary/backup files and recovery.
- If commands grow past ~25, add focused files under `src/commands/` before splitting the simulation; the simulation already lives under `game/`.
