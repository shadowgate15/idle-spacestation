# TAURI / RUST KNOWLEDGE BASE

## OVERVIEW

The Rust side is the idle-game simulation engine plus the Tauri command surface that the SvelteKit SPA talks to. It is **not** template-tiny: `src/lib.rs` is ~1700 lines, and the full backend is split into a `game/` module covering simulation, content, progression, persistence, and IPC DTOs. A background thread drives the simulation tick at ~250 ms cadence, mutating shared state held under a `Mutex` and surfaced to the frontend via 19 `#[tauri::command]` functions (10 production, 9 debug-only).

## STRUCTURE

```text
src-tauri/
├── src/
│   ├── main.rs               # Windows-subsystem guarded entrypoint → idle_spacestation_lib::run()
│   ├── lib.rs                # 1711 lines: command handlers, Tauri builder, state, tick thread
│   └── game/
│       ├── mod.rs            # Re-exports: content, persistence, progression, snapshot, sim
│       ├── snapshot.rs       # 1214 lines: serde DTOs returned to the frontend (camelCase)
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

| Task                       | Location                                    | Notes                                                                    |
| -------------------------- | ------------------------------------------- | ------------------------------------------------------------------------ |
| App entrypoint             | `src/main.rs`                               | Calls `idle_spacestation_lib::run()`; preserves Windows subsystem guard  |
| Tauri commands & builder   | `src/lib.rs`                                | All `#[tauri::command]` fns + `run()` with split debug/release handler   |
| Shared backend state       | `src/lib.rs` (`GameState`, `DevtoolsState`) | `Mutex<(RunState, PrestigeProfile, u32 session_ticks)>` + debug overlay  |
| Simulation tick            | `src/game/sim/tick.rs`                      | Six-phase loop; called every 250 ms by the background thread             |
| Game data tables           | `src/game/content/*.rs`                     | `SYSTEMS`, `SERVICES`, `PLANETS`, `DOCTRINES`, `RESOURCES` constants     |
| Prestige rules             | `src/game/progression/prestige.rs`          | Tier calc, `PrestigeEligibility`, `execute_prestige`, stable-power timer |
| IPC DTOs                   | `src/game/snapshot.rs`                      | `RawGameSnapshot`, `ActionResponse`, `SaveLoadResponse`, route views     |
| Tauri runtime config       | `tauri.conf.json`                           | Dev/build delegate to pnpm; window 800×600; `withGlobalTauri: true`      |
| Capabilities / permissions | `capabilities/default.json`                 | Permits `core:default`, `opener:default`, `mcp-bridge:default`           |
| Cargo deps                 | `Cargo.toml`                                | `tauri 2`, `tauri-plugin-opener 2`, `tauri-plugin-mcp-bridge 0.11`       |
| Build hook                 | `build.rs`                                  | Thin `tauri_build::build()`                                              |

## TAURI COMMAND SURFACE

The Tauri builder in `lib.rs::run()` registers two distinct command sets — one for debug builds and one for release. Both must be kept in sync when adding/removing commands.

**Production commands (always registered):**

- `game_get_snapshot`
- `game_toggle_service`
- `game_upgrade_system`
- `game_assign_service_crew`
- `game_reprioritize_service`
- `game_start_survey`
- `game_purchase_doctrine`
- `game_execute_prestige`
- `game_request_save`
- `game_request_load`

**Debug-only commands (gated by `#[cfg(debug_assertions)]`):**

- `game_devtools_get_state`
- `game_devtools_set_visibility`
- `game_devtools_apply_resources`
- `game_devtools_apply_crew`
- `game_devtools_apply_systems`
- `game_devtools_apply_services`
- `game_devtools_apply_progression`
- `game_devtools_advance_ticks`
- `game_devtools_reset_to_starter`

The frontend gateway (`src/lib/game/api/gateway.ts`) issues two of these commands under different names: `game_set_service_activation` (FE) → `game_toggle_service` (Rust), and `game_confirm_prestige` (FE) → `game_execute_prestige` (Rust). When renaming a Rust command, update the alias map in `gateway.ts` accordingly.

## CONVENTIONS

- Crate name uses the `_lib` suffix (`idle_spacestation_lib`) to avoid Windows binary/lib name collisions; preserve that in `Cargo.toml`.
- All commands accept inputs as a single `input: T` argument (matching the gateway's `{ input: payload }` envelope) except for the no-arg commands listed in `gateway.ts` (`game_get_snapshot`, `game_start_survey`, `game_request_save`, `game_request_load`, `game_devtools_get_state`, `game_devtools_reset_to_starter`).
- Commands never panic on bad input. They return `ActionResponse { ok, snapshot, reason_code }` (or a typed devtools response) and surface failures via `reason_code` strings that mirror the rejection enums on the frontend.
- Serde DTOs in `snapshot.rs` use `#[serde(rename_all = "camelCase")]` so the wire shape matches the TypeScript types in `src/lib/game/api/types.ts`.
- The simulation owns mutation; `#[tauri::command]` functions take the lock briefly, mutate, and produce a fresh snapshot. Don't perform long-running work while holding `GameState`'s `Mutex`.
- Plugins:
  - `tauri-plugin-opener` is registered in all builds.
  - `tauri-plugin-mcp-bridge` is registered only under `#[cfg(debug_assertions)]`. Treat its presence as a debug-only assumption.
- Background tick: spawned in `setup()` as a daemon thread with a 250 ms `thread::sleep` between ticks. Don't block this thread or change the cadence without auditing `tick.rs` invariants (deficit timing, autosave throttling, prestige stability counter).
- Persistence module is **scaffolded but not wired**: `game_request_save`/`game_request_load` currently return the live snapshot without touching disk. Treat the persistence module as load-bearing infrastructure for future work, not as dead code to delete.

## ANTI-PATTERNS

- Do not edit `target/` or `gen/`; both are generated and ignored by `eslint.config.js`, `.prettierignore`, and `.gitignore`.
- Do not add a new `#[tauri::command]` and forget to register it in **both** `tauri::generate_handler!` macros inside `run()` (debug branch ~`lib.rs:1672`, release branch ~`lib.rs:1695`).
- Do not remove the Windows release guard in `src/main.rs`.
- Do not hold `GameState`'s `Mutex` across an `await` or any I/O.
- Do not change `RunState` or `PrestigeProfile` field shapes without bumping the `SAVE_VERSION` and adding a migration in `persistence/migration.rs` — even though save/load aren't wired yet, the on-disk format is the contract.
- Do not surface raw `panic!`/`unwrap` errors to the frontend; use `ActionResponse.reason_code`.
- Do not document a Rust module split that does not exist; today the boundaries are `game/{sim, content, progression, persistence, snapshot}` and that's it.
- Do not add frontend assumptions here; UI rules belong in `src/AGENTS.md`.

## UNIQUE STYLES

- `tauri.conf.json` sets `withGlobalTauri: true` and a permissive `csp: null`; both choices affect runtime behavior immediately, so changes warrant deliberate review.
- `dev`/`build` are delegated to pnpm via `beforeDevCommand` and `beforeBuildCommand` (`pnpm dev` / `pnpm build`), and the frontend dist is `../build` — keep these aligned with `vite.config.js`.
- The MCP bridge plugin enables IDE/agent inspection of the running game state while developing; releases never ship with it.
- The 6-phase tick (alloc → activate → produce → upkeep → deficit → survey) is the load-bearing simulation contract. Cross-phase ordering matters for power deficit and autosave bookkeeping.

## NOTES

- Tests live inside `src/lib.rs` (`#[cfg(test)] mod tests`) and across the `game/` submodules. Run with `cargo test --manifest-path src-tauri/Cargo.toml` (or `cd src-tauri && cargo test`).
- Persistence integration (writing `SaveData` to disk on `Autosave`/`VisibilityHidden`/`WindowClose`/`BeforePrestige` triggers) is the obvious next wiring task; `SaveManager` already handles primary/backup files and recovery.
- If commands grow past ~25 or `lib.rs` becomes unmanageable, split command handlers into a `commands/` module before splitting the simulation; the simulation already lives under `game/`.
