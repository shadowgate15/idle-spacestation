# TAURI KNOWLEDGE BASE

## OVERVIEW

Rust/Tauri boundary for the desktop shell, command registration, packaging config, and build integration.

## WHERE TO LOOK

| Task                       | Location          | Notes                                                    |
| -------------------------- | ----------------- | -------------------------------------------------------- |
| App entrypoint             | `src/main.rs`     | Calls `idle_spacestation_lib::run()`                     |
| Tauri commands and plugins | `src/lib.rs`      | `#[tauri::command]`, plugin registration, builder wiring |
| Tauri app config           | `tauri.conf.json` | Dev/build commands, window settings, bundle metadata     |
| Cargo deps                 | `Cargo.toml`      | Tauri crates, MCP bridge, plugin versions                |
| Build hook                 | `build.rs`        | Thin `tauri_build::build()` wrapper                      |

## STRUCTURE

```text
src-tauri/
├── src/                 # Hand-written Rust app code
├── icons/               # Bundle assets
├── capabilities/        # Tauri capability config
├── Cargo.toml           # Rust dependencies and crate metadata
├── tauri.conf.json      # Desktop app/runtime config
├── target/              # Generated Cargo output
└── gen/schemas/         # Generated Tauri schemas
```

## CONVENTIONS

- Keep frontend/backend integration explicit: frontend calls `invoke(...)`; backend exposes `#[tauri::command]` functions in `src/lib.rs`.
- Development/build commands are delegated to pnpm from `tauri.conf.json` (`beforeDevCommand`, `beforeBuildCommand`).
- Debug builds load `tauri_plugin_mcp_bridge`; release behavior differs because that plugin is only added under `debug_assertions`.
- `Cargo.toml` uses a `_lib` suffix for the crate name to avoid Windows conflicts; preserve that pattern.

## ANTI-PATTERNS

- Do not edit `target/` or `gen/schemas/`; both are generated and already ignored in `eslint.config.js` and `.gitignore`.
- Do not remove the Windows release guard in `src/main.rs`.
- Do not document a Rust module split that does not exist yet; current backend is intentionally tiny.
- Do not add frontend assumptions here; UI rules belong in `src/AGENTS.md`.

## UNIQUE STYLES

- `tauri.conf.json` sets `withGlobalTauri: true` and a permissive `csp: null`; changes here affect desktop runtime behavior immediately.
- Current backend surface is one command (`greet`) plus the opener plugin and debug-only MCP bridge.

## NOTES

- Because the Rust side is small, prefer extending `src/lib.rs` before creating extra modules.
- If Tauri command count grows, revisit whether `src-tauri/src/` needs a deeper AGENTS split.
